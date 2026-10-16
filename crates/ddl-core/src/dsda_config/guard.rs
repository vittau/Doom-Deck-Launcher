//! The run guard (spec 005, R4): callers hold it for the whole lifetime of an
//! engine run, and any provisioning, migration or reset attempted while a run
//! is active fails with a typed error.
//!
//! The lock lives in a small public type so spec 004's engine manager and spec
//! 005's config manager can share exactly one guard per machine.
//!
//! Two mechanisms keep a crashed DDL from wedging the guard:
//!
//! 1. The lock file is held open with an advisory OS lock. When the process
//!    dies the OS releases it automatically, so the next [`RunGuard::acquire`]
//!    succeeds even though the file and its PID remain on disk.
//! 2. The file records the owning PID. If the OS lock is still held but the PID
//!    is provably dead (see [`PidLiveness`]), the guard treats the lock as stale,
//!    clears the file and retries once.
//!
//! Mechanism 1 is what makes recovery reliable on every platform; mechanism 2 is
//! the explicit "a dead PID clears it" rule and also covers exotic filesystems
//! where advisory locks do not work.

use std::fs::{File, OpenOptions, TryLockError};
use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::path::{Path, PathBuf};

/// Name of the lock file inside DDL's DSDA config directory.
pub const LOCK_FILE: &str = "run.lock";

/// Why acquiring or inspecting the run guard failed.
#[derive(Debug)]
pub enum GuardError {
    /// An engine run is active, so the caller must not touch the config.
    RunActive {
        /// The PID recorded in the lock file, when it could be read.
        pid: Option<u32>,
    },
    /// The lock file could not be read or written.
    Io(std::io::Error),
}

impl std::fmt::Display for GuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RunActive { pid: Some(pid) } => {
                write!(f, "a DSDA-Doom run (PID {pid}) is active")
            }
            Self::RunActive { pid: None } => f.write_str("a DSDA-Doom run is active"),
            Self::Io(err) => write!(f, "run lock i/o error: {err}"),
        }
    }
}

impl std::error::Error for GuardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::RunActive { .. } => None,
        }
    }
}

impl From<std::io::Error> for GuardError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// Decides whether a PID belongs to a live process.
///
/// Kept behind a trait so tests can simulate a crashed DDL without spawning
/// processes, and so the platform-specific probe stays out of the guard's logic
/// (decision D-003).
pub trait PidLiveness {
    /// Returns `true` when `pid` might still be running.
    fn is_alive(&self, pid: u32) -> bool;
}

/// The real probe. On Linux it checks `/proc/<pid>`; on other platforms it is
/// deliberately conservative and reports "alive", because the release target is
/// Linux and the advisory OS lock already recovers from a crash everywhere.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemLiveness;

impl PidLiveness for SystemLiveness {
    fn is_alive(&self, pid: u32) -> bool {
        if pid == 0 {
            return false;
        }
        if pid == std::process::id() {
            return true;
        }
        #[cfg(target_os = "linux")]
        {
            Path::new("/proc").join(pid.to_string()).exists()
        }
        #[cfg(not(target_os = "linux"))]
        {
            true
        }
    }
}

/// An active engine run. Drop it (or call [`RunGuard::end`]) when DSDA exits.
#[derive(Debug)]
pub struct RunGuard {
    path: PathBuf,
    file: File,
}

impl RunGuard {
    /// Acquires the guard in `dir` (DDL's DSDA config directory).
    ///
    /// # Errors
    /// Returns [`GuardError::RunActive`] when another run already holds it.
    pub fn acquire(dir: &Path) -> Result<Self, GuardError> {
        Self::acquire_with(dir, &SystemLiveness)
    }

    /// Like [`RunGuard::acquire`] with an injected liveness probe (tests).
    ///
    /// # Errors
    /// See [`RunGuard::acquire`].
    pub fn acquire_with(dir: &Path, probe: &dyn PidLiveness) -> Result<Self, GuardError> {
        Self::acquire_inner(dir, probe, true)
    }

    fn acquire_inner(
        dir: &Path,
        probe: &dyn PidLiveness,
        allow_stale_clear: bool,
    ) -> Result<Self, GuardError> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(LOCK_FILE);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        match file.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => {
                let pid = read_pid(&mut file);
                if allow_stale_clear
                    && let Some(pid) = pid
                    && !probe.is_alive(pid)
                {
                    // The owner is gone but the lock is still held (for example
                    // on a filesystem without working advisory locks): clear the
                    // stale file and retry exactly once.
                    drop(file);
                    std::fs::remove_file(&path)?;
                    return Self::acquire_inner(dir, probe, false);
                }
                return Err(GuardError::RunActive { pid });
            }
            Err(TryLockError::Error(err)) => return Err(GuardError::Io(err)),
        }
        write_pid(&mut file, std::process::id())?;
        Ok(Self { path, file })
    }

    /// Whatever the caller does with the guard, the OS releases the lock when
    /// this file handle closes. This method exists so call sites read as
    /// `begin_run()` / `end_run()` instead of relying on `drop`.
    ///
    /// # Errors
    /// Returns [`GuardError::Io`] if the advisory lock could not be released.
    pub fn end(self) -> Result<(), GuardError> {
        self.file.unlock().map_err(GuardError::Io)
    }

    /// The lock file this guard owns.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether an engine run currently holds the guard in `dir`.
    ///
    /// # Errors
    /// Returns [`GuardError::Io`] when the lock file exists but cannot be read.
    pub fn is_active(dir: &Path) -> Result<bool, GuardError> {
        let path = dir.join(LOCK_FILE);
        if !path.exists() {
            return Ok(false);
        }
        let file = OpenOptions::new().read(true).write(true).open(&path)?;
        match file.try_lock() {
            Ok(()) => {
                // We could take it, so nobody holds it: release immediately.
                let _ = file.unlock();
                Ok(false)
            }
            Err(TryLockError::WouldBlock) => Ok(true),
            Err(TryLockError::Error(err)) => Err(GuardError::Io(err)),
        }
    }
}

/// The PID recorded in the lock file, if the file exists and holds a number.
///
/// # Errors
/// Returns [`GuardError::Io`] when the file exists but cannot be read.
pub fn recorded_pid(dir: &Path) -> Result<Option<u32>, GuardError> {
    let path = dir.join(LOCK_FILE);
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(text.trim().parse().ok()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(GuardError::Io(err)),
    }
}

fn read_pid(file: &mut File) -> Option<u32> {
    let mut text = String::new();
    file.seek(SeekFrom::Start(0)).ok()?;
    file.read_to_string(&mut text).ok()?;
    text.trim().parse().ok()
}

fn write_pid(file: &mut File, pid: u32) -> Result<(), GuardError> {
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(pid.to_string().as_bytes())?;
    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Alive(bool);
    impl PidLiveness for Alive {
        fn is_alive(&self, _pid: u32) -> bool {
            self.0
        }
    }

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("create temp dir")
    }

    #[test]
    fn second_acquire_is_refused_while_held() {
        let dir = temp_dir();
        let guard = RunGuard::acquire(dir.path()).expect("first acquire");

        let error = RunGuard::acquire(dir.path()).expect_err("second acquire must fail");
        match error {
            GuardError::RunActive { pid } => assert_eq!(pid, Some(std::process::id())),
            other => panic!("expected RunActive, got {other:?}"),
        }

        guard.end().expect("release");
        RunGuard::acquire(dir.path()).expect("acquire after release");
    }

    #[test]
    fn is_active_reflects_the_guard_lifetime() {
        let dir = temp_dir();
        assert!(!RunGuard::is_active(dir.path()).expect("no lock file"));

        let guard = RunGuard::acquire(dir.path()).expect("acquire");
        assert!(RunGuard::is_active(dir.path()).expect("held"));

        guard.end().expect("release");
        assert!(!RunGuard::is_active(dir.path()).expect("released"));
    }

    #[test]
    fn lock_file_records_the_owning_pid() {
        let dir = temp_dir();
        let guard = RunGuard::acquire(dir.path()).expect("acquire");
        let text = std::fs::read_to_string(guard.path()).expect("read lock");
        assert_eq!(text.trim().parse::<u32>().ok(), Some(std::process::id()));
    }

    #[test]
    fn a_dead_pid_with_a_held_os_lock_is_cleared() {
        let dir = temp_dir();
        let path = dir.path().join(LOCK_FILE);

        // Simulate a lock that is still held but whose owner is dead: the OS
        // lock is taken by this test's handle while the file names PID 999_999.
        let holder = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .expect("open lock");
        holder.try_lock().expect("hold lock");
        std::fs::write(&path, "999999").expect("write stale pid");

        let guard = RunGuard::acquire_with(dir.path(), &Alive(false))
            .expect("a dead owner must be cleared");
        let text = std::fs::read_to_string(guard.path()).expect("read lock");
        assert_eq!(text.trim().parse::<u32>().ok(), Some(std::process::id()));
    }

    #[test]
    fn a_live_pid_with_a_held_os_lock_is_refused() {
        let dir = temp_dir();
        let path = dir.path().join(LOCK_FILE);
        let holder = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .expect("open lock");
        holder.try_lock().expect("hold lock");
        std::fs::write(&path, "4242").expect("write pid");

        let error =
            RunGuard::acquire_with(dir.path(), &Alive(true)).expect_err("a live owner must block");
        match error {
            GuardError::RunActive { pid } => assert_eq!(pid, Some(4242)),
            other => panic!("expected RunActive, got {other:?}"),
        }
    }

    #[test]
    fn a_stale_pid_left_behind_by_a_crash_is_overwritten() {
        // No OS lock is held here, exactly as after a crash: the recorded PID
        // must not wedge the guard.
        let dir = temp_dir();
        std::fs::write(dir.path().join(LOCK_FILE), "999999").expect("write stale pid");

        let guard = RunGuard::acquire(dir.path()).expect("stale lock must be cleared");
        let text = std::fs::read_to_string(guard.path()).expect("read lock");
        assert_eq!(text.trim().parse::<u32>().ok(), Some(std::process::id()));
    }
}

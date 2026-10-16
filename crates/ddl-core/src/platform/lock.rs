//! A cross-process lock around DDL's mutable state (spec 014, R2).
//!
//! The UI and the headless CLI share the same XDG state, so two processes must
//! never mutate it at the same time. Spec 005's run guard already covers the
//! engine run and the DSDA configuration; it deliberately does **not** cover
//! installs, profile edits or registry refreshes. [`StateLock`] closes that gap
//! with one advisory OS lock, so
//!
//! * a second DDL process (UI or CLI) refuses a mutation with a clear error
//!   instead of racing, and
//! * a crashed process cannot wedge the lock: the OS releases an advisory lock
//!   when the owning process dies, so no stale-PID recovery is needed.
//!
//! It is intentionally narrower than [`crate::dsda_config::RunGuard`]: this lock
//! says "DDL is changing its files", not "DSDA-Doom is running".

use std::fmt;
use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};

/// Name of the lock file inside DDL's config directory.
pub const STATE_LOCK_FILE: &str = "state.lock";

/// Why acquiring the state lock failed.
#[derive(Debug)]
pub enum LockError {
    /// Another DDL process is changing the same state.
    Busy,
    /// The lock file could not be created or locked.
    Io(io::Error),
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Busy => {
                f.write_str("another DDL process is changing DDL's state; close it and try again")
            }
            Self::Io(error) => write!(f, "state lock i/o error: {error}"),
        }
    }
}

impl std::error::Error for LockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Busy => None,
        }
    }
}

impl From<io::Error> for LockError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// An acquired cross-process mutation lock. Drop it (or call
/// [`StateLock::end`]) when the mutation is done.
#[derive(Debug)]
pub struct StateLock {
    path: PathBuf,
    file: File,
}

impl StateLock {
    /// Acquires the lock in `dir` (DDL's config directory), never blocking.
    ///
    /// # Errors
    ///
    /// Returns [`LockError::Busy`] when another process holds the lock, and
    /// [`LockError::Io`] when the directory or file cannot be used.
    pub fn acquire(dir: &Path) -> Result<Self, LockError> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(STATE_LOCK_FILE);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        match file.try_lock() {
            Ok(()) => Ok(Self { path, file }),
            Err(TryLockError::WouldBlock) => Err(LockError::Busy),
            Err(TryLockError::Error(error)) => Err(LockError::Io(error)),
        }
    }

    /// The lock file this guard owns.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Releases the lock explicitly, mirroring [`StateLock::acquire`].
    ///
    /// # Errors
    ///
    /// Returns [`LockError::Io`] when the advisory lock cannot be released.
    pub fn end(self) -> Result<(), LockError> {
        self.file.unlock().map_err(LockError::Io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn a_second_acquire_is_refused_until_the_first_is_released() {
        let dir = tempdir().expect("temp dir");
        let lock = StateLock::acquire(dir.path()).expect("first acquire");
        assert_eq!(lock.path(), dir.path().join(STATE_LOCK_FILE));

        let error = StateLock::acquire(dir.path()).expect_err("second acquire must fail");
        assert!(matches!(error, LockError::Busy));
        assert!(error.to_string().contains("another DDL process"));

        lock.end().expect("release");
        StateLock::acquire(dir.path()).expect("acquire after release");
    }

    #[test]
    fn dropping_the_guard_releases_the_advisory_lock() {
        let dir = tempdir().expect("temp dir");
        {
            let _lock = StateLock::acquire(dir.path()).expect("acquire");
        }
        StateLock::acquire(dir.path()).expect("dropped lock must not wedge the guard");
    }
}

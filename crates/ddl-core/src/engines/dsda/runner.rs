//! Starting the engine and reporting how it ended.
//!
//! `std::process::Command` with separate arguments is the only way DDL runs the
//! engine (`AGENTS.md`): there is no shell string anywhere, and
//! [`RealEngineRunner`] clears the four `DOOM*` variables the research lists so
//! a stale shell environment cannot override the plan's explicit `-iwad`,
//! `-data`, `-save` or search path.
//!
//! [`EngineRunner`] is the seam (decision D-003): the real runner starts a
//! process, [`FakeEngineRunner`] records what would have been started.

use std::fmt;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::plan::{AssignAllowlist, LaunchPlan, ValidationError};

/// Environment variables DDL removes from the child's environment.
///
/// The research documents them as alternative sources for the IWAD search
/// directory, the data directory and the save directory; DDL always passes
/// explicit absolute `-iwad`, `-data` and `-save` values instead, and a stale
/// variable would silently win over parts of the plan.
///
/// Source: `docs/research/dsda-engine.md` §1 "Search paths, autoload, and
/// probe/error behavior" and §6.
pub const SCRUBBED_ENV: [&str; 4] = ["DOOMWADDIR", "DOOMWADPATH", "DOOMDATADIR", "DOOMSAVEDIR"];

/// How an engine process ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LaunchExit {
    /// The process exited normally with status 0.
    Clean,
    /// The process exited normally with a non-zero status. DSDA-Doom's
    /// `I_Error` path calls `I_SafeExit(-1)`, whose observable low-byte status
    /// on Linux is expected to be `255` (research §1, **Proposed** until the
    /// asset is run on a Deck).
    Failed {
        /// The exit code.
        code: i32,
    },
    /// The process was terminated by a signal (Unix). `signal` is the signal
    /// number, e.g. `6` for `SIGABRT` or `9` for `SIGKILL`.
    Signalled {
        /// The signal number.
        signal: i32,
    },
}

impl LaunchExit {
    /// Classifies a raw [`ExitStatus`].
    ///
    /// On Unix, a process killed by a signal has no exit code, and DDL must not
    /// confuse "killed" with "exited with an error".
    #[must_use]
    #[cfg(unix)]
    pub fn from_exit_status(status: ExitStatus) -> Self {
        use std::os::unix::process::ExitStatusExt as _;

        match (status.code(), status.signal()) {
            (Some(0), _) => Self::Clean,
            (Some(code), _) => Self::Failed { code },
            (None, Some(signal)) => Self::Signalled { signal },
            // Neither a code nor a signal: not reachable on Unix, but reported
            // as a failure rather than silently as success.
            (None, None) => Self::Failed { code: -1 },
        }
    }

    /// Classifies a raw [`ExitStatus`] on platforms without Unix signals.
    #[must_use]
    #[cfg(not(unix))]
    pub fn from_exit_status(status: ExitStatus) -> Self {
        match status.code() {
            Some(0) => Self::Clean,
            Some(code) => Self::Failed { code },
            None => Self::Failed { code: -1 },
        }
    }

    /// Whether the engine exited cleanly.
    #[must_use]
    pub const fn is_clean(self) -> bool {
        matches!(self, Self::Clean)
    }
}

impl fmt::Display for LaunchExit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clean => f.write_str("exited cleanly"),
            Self::Failed { code } => write!(f, "exited with status {code}"),
            Self::Signalled { signal } => write!(f, "was killed by signal {signal}"),
        }
    }
}

/// What [`EngineRunner::launch`] reports back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchOutcome {
    /// How the engine process ended.
    pub exit_status: LaunchExit,
    /// How long the process ran.
    pub duration: Duration,
    /// The per-launch log file holding the engine's stdout and stderr.
    pub log_path: PathBuf,
}

/// Why a launch could not be started.
#[derive(Debug)]
pub enum LaunchError {
    /// The plan is invalid, so no process was started.
    Invalid(ValidationError),
    /// The process could not be started, or its output could not be captured.
    Io(io::Error),
}

impl fmt::Display for LaunchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(err) => write!(f, "invalid launch plan: {err}"),
            Self::Io(err) => write!(f, "could not start the engine: {err}"),
        }
    }
}

impl std::error::Error for LaunchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Invalid(err) => Some(err),
            Self::Io(err) => Some(err),
        }
    }
}

impl From<ValidationError> for LaunchError {
    fn from(err: ValidationError) -> Self {
        Self::Invalid(err)
    }
}

impl From<io::Error> for LaunchError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Starts the engine and waits for it.
///
/// Implementations must never use a shell and must never write to the plan's
/// config file (decision D-012).
pub trait EngineRunner {
    /// Validates `plan`, starts the engine, waits for it, and reports how it
    /// ended.
    ///
    /// # Errors
    ///
    /// Returns [`LaunchError::Invalid`] when the plan fails
    /// [`LaunchPlan::validate`], and [`LaunchError::Io`] when the process or its
    /// log file cannot be created or waited on.
    fn launch(
        &self,
        plan: &LaunchPlan,
        assign_allowlist: &AssignAllowlist,
    ) -> Result<LaunchOutcome, LaunchError>;
}

/// The real runner: a `std::process::Command` child, no shell.
///
/// Logs are written into a directory the caller owns; production uses
/// [`RealEngineRunner::for_state`], which points at DDL's state directory
/// (spec 003, R4), while tests pick a scratch directory.
#[derive(Debug, Clone)]
pub struct RealEngineRunner {
    log_dir: PathBuf,
    /// The environment the child starts from. Defaults to the process
    /// environment, matching "inherited", but an explicit value makes the
    /// scrubbing policy testable without mutating the test process's global
    /// environment.
    base_env: Vec<(std::ffi::OsString, std::ffi::OsString)>,
}

/// Name of the subdirectory of DDL's state directory that holds launch logs.
pub const LOG_DIR_NAME: &str = "logs";

impl RealEngineRunner {
    /// A runner that writes logs into `log_dir` and gives children the current
    /// process environment.
    pub fn new(log_dir: impl Into<PathBuf>) -> Self {
        Self::with_base_env(log_dir, std::env::vars_os().collect())
    }

    /// A runner whose per-launch logs live under DDL's state directory
    /// (`<state>/logs`), which is what spec 003 R4 asks for: logs are state
    /// that should survive a reboot, not cache.
    #[must_use]
    pub fn for_state(paths: &crate::platform::paths::Paths) -> Self {
        Self::new(paths.state.join(LOG_DIR_NAME))
    }

    /// A runner whose children start from `base_env` instead of the current
    /// process environment. Tests use this to prove [`SCRUBBED_ENV`] is
    /// removed; production uses [`RealEngineRunner::new`].
    pub fn with_base_env(
        log_dir: impl Into<PathBuf>,
        base_env: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    ) -> Self {
        Self {
            log_dir: log_dir.into(),
            base_env,
        }
    }

    /// The directory per-launch logs are written to.
    #[must_use]
    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    /// A unique log path: `launch-<nanos>-<pid>-<sequence>.log`.
    fn next_log_path(&self) -> PathBuf {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);

        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        self.log_dir.join(format!(
            "launch-{nanos}-{}-{sequence}.log",
            std::process::id()
        ))
    }
}

impl EngineRunner for RealEngineRunner {
    fn launch(
        &self,
        plan: &LaunchPlan,
        assign_allowlist: &AssignAllowlist,
    ) -> Result<LaunchOutcome, LaunchError> {
        let mut command = plan.build_command(assign_allowlist)?;

        fs::create_dir_all(&self.log_dir)?;
        let log_path = self.next_log_path();
        let log = File::create(&log_path)?;
        command.stdout(log.try_clone()?);
        command.stderr(log);
        command.stdin(Stdio::null());

        // Inherit the base environment, minus the variables that would let a
        // stale shell override parts of the plan.
        command.env_clear();
        command.envs(
            self.base_env
                .iter()
                .map(|(key, value)| (key.as_os_str(), value.as_os_str())),
        );
        for key in SCRUBBED_ENV {
            command.env_remove(key);
        }

        let started = Instant::now();
        let mut child = command.spawn()?;
        let status = child.wait()?;
        Ok(LaunchOutcome {
            exit_status: LaunchExit::from_exit_status(status),
            duration: started.elapsed(),
            log_path,
        })
    }
}

/// A test double for [`EngineRunner`].
///
/// It touches no filesystem and starts no process; it records the plans it was
/// handed and returns a preset [`LaunchExit`], so UI and orchestration tests can
/// drive every branch without the engine.
#[derive(Debug)]
pub struct FakeEngineRunner {
    exit: LaunchExit,
    log_path: PathBuf,
    launched: Mutex<Vec<RecordedLaunch>>,
}

/// One call recorded by [`FakeEngineRunner`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedLaunch {
    /// The executable the real runner would have started.
    pub executable: PathBuf,
    /// The argv the real runner would have passed, in plan order.
    pub argv: Vec<std::ffi::OsString>,
    /// The working directory the real runner would have used.
    pub working_dir: PathBuf,
}

impl FakeEngineRunner {
    /// A fake that reports `exit` for every launch.
    #[must_use]
    pub fn new(exit: LaunchExit) -> Self {
        Self {
            exit,
            log_path: PathBuf::from("launch.log"),
            launched: Mutex::new(Vec::new()),
        }
    }

    /// Sets the log path reported in every [`LaunchOutcome`].
    #[must_use]
    pub fn with_log_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.log_path = path.into();
        self
    }

    /// The exit the fake reports from now on.
    pub fn set_exit(&mut self, exit: LaunchExit) {
        self.exit = exit;
    }

    /// The launches recorded so far, oldest first.
    #[must_use]
    pub fn launches(&self) -> Vec<RecordedLaunch> {
        self.launched
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// How many launches were recorded.
    #[must_use]
    pub fn launch_count(&self) -> usize {
        self.launched
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
}

impl EngineRunner for FakeEngineRunner {
    fn launch(
        &self,
        plan: &LaunchPlan,
        _assign_allowlist: &AssignAllowlist,
    ) -> Result<LaunchOutcome, LaunchError> {
        self.launched
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(RecordedLaunch {
                executable: plan.executable.clone(),
                argv: plan.to_command(),
                working_dir: plan.working_dir.clone(),
            });
        Ok(LaunchOutcome {
            exit_status: self.exit,
            duration: Duration::ZERO,
            log_path: self.log_path.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::plan::{Complevel, Skill};
    use super::*;

    fn sample_plan() -> LaunchPlan {
        LaunchPlan {
            executable: PathBuf::from("/opt/ddl/dsda-doom.appimage"),
            iwad: PathBuf::from("/opt/ddl/doom2.wad"),
            files: Vec::new(),
            patches: Vec::new(),
            complevel: Complevel::DOOM_1_9,
            skill: Skill::new(3).expect("in range"),
            warp: None,
            pistol_start: false,
            record: None,
            config: PathBuf::from("/opt/ddl/dsda.cfg"),
            save_dir: PathBuf::from("/opt/ddl/save"),
            data_dir: PathBuf::from("/opt/ddl/data"),
            shot_dir: PathBuf::from("/opt/ddl/shots"),
            working_dir: PathBuf::from("/opt/ddl/run"),
            assigns: Vec::new(),
        }
    }

    #[test]
    fn default_exit_status_is_clean() {
        // `ExitStatus::default()` is "exited with 0", which lets the mapping be
        // tested without spawning anything.
        assert_eq!(
            LaunchExit::from_exit_status(ExitStatus::default()),
            LaunchExit::Clean
        );
        assert!(LaunchExit::Clean.is_clean());
        assert!(!LaunchExit::Failed { code: 255 }.is_clean());
        assert_eq!(
            LaunchExit::Failed { code: 255 }.to_string(),
            "exited with status 255"
        );
        assert_eq!(
            LaunchExit::Signalled { signal: 9 }.to_string(),
            "was killed by signal 9"
        );
    }

    #[test]
    fn fake_runner_records_the_plan_argv_and_reports_its_exit() {
        let plan = sample_plan();
        let runner = FakeEngineRunner::new(LaunchExit::Failed { code: 255 });

        let outcome = runner
            .launch(&plan, &AssignAllowlist::new())
            .expect("the fake never fails");

        assert_eq!(outcome.exit_status, LaunchExit::Failed { code: 255 });
        assert_eq!(outcome.log_path, PathBuf::from("launch.log"));
        let recorded = runner.launches();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].executable, plan.executable);
        assert_eq!(recorded[0].working_dir, plan.working_dir);
        assert_eq!(recorded[0].argv, plan.to_command());
        assert_eq!(runner.launch_count(), 1);
    }

    #[test]
    fn fake_runner_can_change_its_exit_between_launches() {
        let plan = sample_plan();
        let mut runner = FakeEngineRunner::new(LaunchExit::Clean);
        let outcome = runner
            .launch(&plan, &AssignAllowlist::new())
            .expect("the fake never fails");
        assert!(outcome.exit_status.is_clean());

        runner.set_exit(LaunchExit::Signalled { signal: 6 });
        let outcome = runner
            .launch(&plan, &AssignAllowlist::new())
            .expect("the fake never fails");
        assert_eq!(outcome.exit_status, LaunchExit::Signalled { signal: 6 });
        assert_eq!(runner.launch_count(), 2);
    }

    #[test]
    fn fake_runner_can_report_another_log_path() {
        let runner = FakeEngineRunner::new(LaunchExit::Clean).with_log_path("/tmp/probe.log");
        let outcome = runner
            .launch(&sample_plan(), &AssignAllowlist::new())
            .expect("the fake never fails");
        assert_eq!(outcome.log_path, PathBuf::from("/tmp/probe.log"));
    }

    #[test]
    fn launch_error_wraps_both_causes() {
        let invalid = LaunchError::from(ValidationError::PistolStartWithRecord);
        assert!(invalid.to_string().contains("invalid launch plan"));
        assert!(std::error::Error::source(&invalid).is_some());

        let io_err = LaunchError::from(io::Error::other("boom"));
        assert!(io_err.to_string().contains("could not start the engine"));
        assert!(std::error::Error::source(&io_err).is_some());
    }

    #[test]
    fn log_paths_are_unique_and_stay_inside_the_log_directory() {
        let runner = RealEngineRunner::with_base_env(PathBuf::from("/var/log/ddl"), Vec::new());
        let first = runner.next_log_path();
        let second = runner.next_log_path();
        assert_ne!(first, second);
        assert_eq!(first.parent(), Some(Path::new("/var/log/ddl")));
        assert_eq!(runner.log_dir(), Path::new("/var/log/ddl"));
        let name = first.file_name().expect("has a name").to_string_lossy();
        assert!(name.starts_with("launch-"), "{name}");
        assert!(name.ends_with(".log"), "{name}");
    }

    #[test]
    fn for_state_places_logs_under_the_state_directory() {
        let paths = crate::platform::paths::Paths {
            config: PathBuf::from("/home/deck/.config/ddl"),
            data: PathBuf::from("/home/deck/.local/share/ddl"),
            cache: PathBuf::from("/home/deck/.cache/ddl"),
            state: PathBuf::from("/home/deck/.local/state/ddl"),
        };
        assert_eq!(
            RealEngineRunner::for_state(&paths).log_dir(),
            Path::new("/home/deck/.local/state/ddl/logs")
        );
    }
}

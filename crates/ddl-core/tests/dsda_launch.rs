//! Integration tests for the engine launch plan and process runner (spec 003).
//!
//! These tests need real files and a real process, so they live outside the
//! crate. They never start DSDA-Doom and never open a window (`AGENTS.md`):
//!
//! * process behaviour is tested with `ddl-launch-probe`, a tiny binary built
//!   from this workspace (`src/bin/ddl-launch-probe.rs`);
//! * the smoke test is tested with throw-away shell scripts written by the
//!   tests themselves.
//!
//! There is no network access.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use ddl_core::engines::dsda::{
    AssignAllowlist, AssignOverride, Complevel, EngineRunner, FakeEngineRunner, LaunchError,
    LaunchExit, LaunchPlan, PathField, RealEngineRunner, RuntimeError, Skill, SmokeError,
    ValidationError, Warp, smoke_test, smoke_test_with_timeout,
};

/// The workspace-built helper the runner tests start instead of the engine.
const PROBE: &str = env!("CARGO_BIN_EXE_ddl-launch-probe");

/// A unique scratch directory, removed on drop.
struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(label: &str) -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ddl-spec-003-{label}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the scratch directory");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    /// Creates (or returns) a file inside the scratch directory.
    fn file(&self, name: &str) -> PathBuf {
        let path = self.path.join(name);
        fs::write(&path, b"fixture").expect("write the fixture file");
        path
    }

    fn dir(&self, name: &str) -> PathBuf {
        let path = self.path.join(name);
        fs::create_dir_all(&path).expect("create the fixture directory");
        path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// A plan with real fixture files and directories that do not exist yet.
fn plan_for(tree: &TestDir, executable: &str) -> LaunchPlan {
    LaunchPlan {
        executable: PathBuf::from(executable),
        iwad: tree.file("doom2.wad"),
        files: vec![
            tree.file("DBP37_AUGZEN.wad"),
            tree.file("DBP37_DeckardBlaster.wad"),
        ],
        patches: vec![tree.file("augzen.deh")],
        complevel: Some(Complevel::DOOM_1_9),
        skill: Skill::new(3).expect("3 is a valid skill"),
        warp: Some(Warp::single(1).expect("1 is a valid warp")),
        pistol_start: false,
        record: None,
        config: tree.path().join("config/dsda-doom.cfg"),
        save_dir: tree.path().join("profile/save"),
        data_dir: tree.path().join("profile/data"),
        shot_dir: tree.path().join("profile/shots"),
        working_dir: tree.path().join("profile/run"),
        assigns: vec![AssignOverride::new("dsda_fps", "35")],
    }
}

fn allowlist() -> AssignAllowlist {
    AssignAllowlist::with_keys(["dsda_fps"])
}

/// Reads a launch log as text.
fn read_log(path: &Path) -> String {
    fs::read_to_string(path).expect("read the launch log")
}

#[test]
fn prepare_dirs_creates_every_directory_dsda_does_not() {
    let tree = TestDir::new("prepare-dirs");
    let plan = plan_for(&tree, PROBE);

    // Directories the engine is expected to use but not create must be
    // reported before anything starts.
    assert!(
        matches!(
            plan.validate(&allowlist()),
            Err(ValidationError::DirectoryMissing { .. })
        ),
        "a plan whose directories do not exist yet must not validate"
    );

    plan.prepare_dirs().expect("prepare_dirs succeeds");
    assert!(plan.validate(&allowlist()).is_ok());
    for dir in [
        &plan.save_dir,
        &plan.data_dir,
        &plan.shot_dir,
        &plan.working_dir,
    ] {
        assert!(dir.is_dir(), "{} should have been created", dir.display());
    }
    assert!(
        plan.config.parent().expect("has a parent").is_dir(),
        "the config file's parent must exist: the engine does not create it"
    );

    // Creating them twice is fine.
    plan.prepare_dirs().expect("prepare_dirs is idempotent");
}

#[test]
fn prepare_dirs_creates_the_recording_directory_too() {
    let tree = TestDir::new("record-dir");
    let plan = LaunchPlan {
        record: Some(tree.path().join("demos/run1.lmp")),
        ..plan_for(&tree, PROBE)
    };

    plan.prepare_dirs().expect("prepare_dirs succeeds");
    assert!(plan.validate(&allowlist()).is_ok());
    assert!(
        tree.path().join("demos").is_dir(),
        "the demo directory must exist: the engine writes the file but not the directory"
    );
}

#[test]
fn real_runner_reports_a_clean_exit_and_captures_stdout_and_stderr() {
    let tree = TestDir::new("clean-exit");
    let log_dir = tree.dir("logs");
    let plan = plan_for(&tree, PROBE);
    plan.prepare_dirs().expect("prepare_dirs succeeds");

    let runner = RealEngineRunner::with_base_env(
        &log_dir,
        vec![(
            OsString::from("DDL_LAUNCH_PROBE_MODE"),
            OsString::from("ok"),
        )],
    );
    let outcome = runner
        .launch(&plan, &allowlist())
        .expect("the probe starts");

    assert!(outcome.exit_status.is_clean(), "{:?}", outcome.exit_status);
    assert!(
        outcome.log_path.starts_with(&log_dir),
        "the log belongs in the runner's log directory"
    );
    let log = read_log(&outcome.log_path);
    assert!(log.contains("probe ok"), "stdout is captured: {log}");
    assert!(
        log.contains("probe diagnostics"),
        "stderr is captured: {log}"
    );
    // Argument order and the single `-file` list survive the process boundary,
    // and each value stayed its own token.
    let expected = format!(
        r#""-iwad", {:?}, "-file", {:?}, {:?}, "-deh", {:?}, "-complevel", "2", "-skill", "3", "-warp", "1", "-noautoload", "-config", "#,
        plan.iwad.display().to_string(),
        plan.files[0].display().to_string(),
        plan.files[1].display().to_string(),
        plan.patches[0].display().to_string(),
    );
    assert!(log.contains(&expected), "expected {expected}, got {log}");
    assert!(log.contains(r#""-assign", "dsda_fps=35""#), "{log}");
}

#[test]
fn real_runner_reports_a_non_zero_exit() {
    let tree = TestDir::new("non-zero");
    let log_dir = tree.dir("logs");
    let plan = plan_for(&tree, PROBE);
    plan.prepare_dirs().expect("prepare_dirs succeeds");
    let runner = RealEngineRunner::with_base_env(
        &log_dir,
        vec![(
            OsString::from("DDL_LAUNCH_PROBE_MODE"),
            OsString::from("exit:255"),
        )],
    );

    let outcome = runner
        .launch(&plan, &allowlist())
        .expect("the probe starts");

    // `I_Error` exits through `I_SafeExit(-1)`; 255 is what the research
    // expects the status byte to be on Linux.
    assert_eq!(outcome.exit_status, LaunchExit::Failed { code: 255 });
    assert!(read_log(&outcome.log_path).contains("probe diagnostics"));
}

#[test]
fn real_runner_distinguishes_death_by_signal() {
    let tree = TestDir::new("signal");
    let log_dir = tree.dir("logs");
    let plan = plan_for(&tree, PROBE);
    plan.prepare_dirs().expect("prepare_dirs succeeds");
    let runner = RealEngineRunner::with_base_env(
        &log_dir,
        vec![(
            OsString::from("DDL_LAUNCH_PROBE_MODE"),
            OsString::from("signal"),
        )],
    );

    let outcome = runner
        .launch(&plan, &allowlist())
        .expect("the probe starts");

    // SIGABRT is 6 on both macOS and Linux; the important part is that a
    // signal is not confused with a non-zero exit status.
    assert_eq!(outcome.exit_status, LaunchExit::Signalled { signal: 6 });
}

#[test]
fn real_runner_sets_the_working_directory_from_the_plan() {
    let tree = TestDir::new("cwd");
    let log_dir = tree.dir("logs");
    let plan = plan_for(&tree, PROBE);
    plan.prepare_dirs().expect("prepare_dirs succeeds");
    let runner = RealEngineRunner::with_base_env(
        &log_dir,
        vec![(
            OsString::from("DDL_LAUNCH_PROBE_MODE"),
            OsString::from("dump-env"),
        )],
    );

    let outcome = runner
        .launch(&plan, &allowlist())
        .expect("the probe starts");

    let log = read_log(&outcome.log_path);
    // `current_dir` reports the resolved path (on macOS `/var` is a symlink to
    // `/private/var`), so compare against the canonical form.
    let expected = fs::canonicalize(&plan.working_dir).expect("canonicalize the working dir");
    assert!(
        log.contains(&format!("cwd={}", expected.display())),
        "{log}"
    );
}

#[test]
fn real_runner_scrubs_the_doom_environment_variables() {
    let tree = TestDir::new("scrub-env");
    let log_dir = tree.dir("logs");
    let plan = plan_for(&tree, PROBE);
    plan.prepare_dirs().expect("prepare_dirs succeeds");

    // A deliberately hostile base environment: every variable DDL must clear is
    // set, plus one it must keep, plus the probe's own mode.
    let base_env = vec![
        (
            OsString::from("DOOMWADDIR"),
            OsString::from("/hostile/wads"),
        ),
        (
            OsString::from("DOOMWADPATH"),
            OsString::from("/hostile/one:/hostile/two"),
        ),
        (
            OsString::from("DOOMDATADIR"),
            OsString::from("/hostile/data"),
        ),
        (
            OsString::from("DOOMSAVEDIR"),
            OsString::from("/hostile/saves"),
        ),
        (OsString::from("DDL_PROBE_KEEP"), OsString::from("kept")),
        (
            OsString::from("DDL_LAUNCH_PROBE_MODE"),
            OsString::from("dump-env"),
        ),
    ];
    let runner = RealEngineRunner::with_base_env(&log_dir, base_env);

    let outcome = runner
        .launch(&plan, &allowlist())
        .expect("the probe starts");

    let log = read_log(&outcome.log_path);
    for key in ["DOOMWADDIR", "DOOMWADPATH", "DOOMDATADIR", "DOOMSAVEDIR"] {
        assert!(
            log.contains(&format!("{key}=unset")),
            "{key} must be cleared, log was: {log}"
        );
    }
    assert!(
        log.contains("DDL_PROBE_KEEP=kept"),
        "an unrelated variable must be inherited: {log}"
    );
}

#[test]
fn real_runner_refuses_an_invalid_plan_without_starting_the_engine() {
    let tree = TestDir::new("invalid");
    let log_dir = tree.dir("logs");
    let plan = LaunchPlan {
        iwad: tree.path().join("missing.wad"),
        ..plan_for(&tree, PROBE)
    };
    plan.prepare_dirs().expect("prepare_dirs succeeds");
    let runner = RealEngineRunner::with_base_env(&log_dir, Vec::new());

    let error = runner
        .launch(&plan, &allowlist())
        .expect_err("an invalid plan must not start");

    assert!(matches!(
        error,
        LaunchError::Invalid(ValidationError::MissingFile {
            field: PathField::Iwad,
            ..
        })
    ));
    assert!(
        fs::read_dir(&log_dir)
            .expect("the log directory exists")
            .next()
            .is_none(),
        "no log file is created for a rejected plan"
    );
}

#[test]
fn runner_trait_is_object_safe_and_the_fake_can_stand_in() {
    let tree = TestDir::new("trait-object");
    let plan = plan_for(&tree, PROBE);
    let runner: Box<dyn EngineRunner> = Box::new(FakeEngineRunner::new(LaunchExit::Clean));

    let outcome = runner
        .launch(&plan, &allowlist())
        .expect("the fake never fails");

    assert!(outcome.exit_status.is_clean());
}

/// Writes an executable shell script and returns its path.
#[cfg(unix)]
fn script(tree: &TestDir, name: &str, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt as _;

    let path = tree.path().join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write the script");
    let mut permissions = fs::metadata(&path).expect("stat the script").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("make the script executable");
    path
}

#[cfg(unix)]
mod smoke {
    use super::*;

    #[test]
    fn smoke_test_reports_the_version() {
        let tree = TestDir::new("smoke-version");
        let executable = script(
            &tree,
            "dsda-doom",
            "echo 'dsda-doom v0.30.1 (https://github.com/dsda-org/dsda-doom)'\nexit 0",
        );

        assert_eq!(
            smoke_test(&executable).expect("the probe succeeds"),
            "0.30.1"
        );
    }

    #[test]
    fn smoke_test_rejects_output_that_is_not_a_version_line() {
        let tree = TestDir::new("smoke-wrong-output");
        let executable = script(&tree, "dsda-doom", "echo 'prboom v2.6'\nexit 0");

        assert!(matches!(
            smoke_test(&executable),
            Err(SmokeError::UnexpectedOutput { .. })
        ));
    }

    #[test]
    fn smoke_test_rejects_a_non_zero_exit() {
        let tree = TestDir::new("smoke-non-zero");
        let executable = script(
            &tree,
            "dsda-doom",
            "echo 'boom' >&2\necho 'dsda-doom v0.30.1'\nexit 3",
        );

        match smoke_test(&executable) {
            Err(SmokeError::Exited { exit, stderr }) => {
                assert_eq!(exit, LaunchExit::Failed { code: 3 });
                assert!(stderr.contains("boom"), "{stderr}");
            }
            other => panic!("expected a non-zero exit, got {other:?}"),
        }
    }

    #[test]
    fn smoke_test_kills_a_child_that_outlives_the_timeout() {
        let tree = TestDir::new("smoke-timeout");
        let executable = script(&tree, "dsda-doom", "exec sleep 30");
        let timeout = Duration::from_millis(250);

        let started = Instant::now();
        let error = smoke_test_with_timeout(&executable, timeout)
            .expect_err("a hanging engine must time out");

        assert!(matches!(error, SmokeError::Timeout { .. }), "{error:?}");
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the child was killed instead of waited for"
        );
    }

    #[test]
    fn smoke_test_reports_a_startup_failure() {
        let tree = TestDir::new("smoke-startup");
        let missing = tree.path().join("not-an-engine");

        assert!(matches!(smoke_test(&missing), Err(SmokeError::Startup(_))));
    }

    /// The real CI run 38980417407 failure: a bare Ubuntu runner lacks
    /// `libOpenGL.so.0`.
    #[test]
    fn smoke_test_classifies_a_missing_shared_library() {
        let tree = TestDir::new("smoke-runtime-lib");
        let executable = script(
            &tree,
            "dsda-doom",
            "echo 'dsda-doom.AppImage: error while loading shared libraries: libOpenGL.so.0: cannot \
             open shared object file: No such file or directory' >&2\nexit 127",
        );

        match smoke_test(&executable) {
            Err(SmokeError::Runtime(RuntimeError::MissingLibrary { library })) => {
                assert_eq!(library, "libOpenGL.so.0");
            }
            other => panic!("expected a runtime error, got {other:?}"),
        }
    }

    /// The real CI run 38980417407 failure: the extracted `AppRun` inside the
    /// Steam Runtime SDK container needs glibc ≥ 2.34.
    #[test]
    fn smoke_test_classifies_an_old_glibc() {
        let tree = TestDir::new("smoke-runtime-glibc");
        let executable = script(
            &tree,
            "dsda-doom",
            "echo \"/lib/x86_64-linux-gnu/libc.so.6: version 'GLIBC_2.34' not found\" >&2\nexit 1",
        );

        match smoke_test(&executable) {
            Err(SmokeError::Runtime(RuntimeError::Glibc { missing })) => {
                assert_eq!(missing, vec!["GLIBC_2.34".to_owned()]);
            }
            other => panic!("expected a runtime error, got {other:?}"),
        }
    }
}

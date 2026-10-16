//! The engine smoke test: `dsda-doom -v`.
//!
//! The research identifies `-v` as the source-safe, non-interactive probe: it
//! prints the version and returns success **before** config load, subsystem and
//! window initialisation or IWAD lookup, so it works on a headless machine and
//! cannot open a window (`AGENTS.md`).
//!
//! A passing smoke test proves that the executable starts and reports its
//! version. It does **not** prove that WAD loading, rendering, audio, input,
//! saves or Deck packaging work — that stays in `docs/deck-testing.md`.

use std::fmt;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::runner::LaunchExit;

/// Default timeout for [`smoke_test`].
pub const SMOKE_TIMEOUT: Duration = Duration::from_secs(10);

/// The prefix DSDA-Doom's version formatter prints
/// (`PROJECT_NAME vPROJECT_VERSION`).
pub const VERSION_PREFIX: &str = "dsda-doom v";

/// How often the smoke test checks whether a slow child has finished.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Runs `<executable> -v` with the default timeout and returns the version it
/// reported (for example `0.30.1`).
///
/// # Errors
///
/// See [`smoke_test_with_timeout`].
pub fn smoke_test(executable: &Path) -> Result<String, SmokeError> {
    smoke_test_with_timeout(executable, SMOKE_TIMEOUT)
}

/// Runs `<executable> -v`, waiting at most `timeout`.
///
/// Succeeds only on exit status 0 with stdout starting with `dsda-doom v`.
///
/// # Errors
///
/// Returns:
///
/// * [`SmokeError::Startup`] when the process cannot be started;
/// * [`SmokeError::Timeout`] when it is still running after `timeout`, in which
///   case it is killed;
/// * [`SmokeError::Exited`] when it exits with a non-zero status;
/// * [`SmokeError::UnexpectedOutput`] when it exits 0 but stdout does not begin
///   with the expected version line.
pub fn smoke_test_with_timeout(executable: &Path, timeout: Duration) -> Result<String, SmokeError> {
    let mut child = Command::new(executable)
        .arg("-v")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(SmokeError::Startup)?;

    // Drain both pipes from separate threads so a chatty child cannot fill a
    // pipe buffer and block while the parent waits.
    let mut stdout_pipe = child.stdout.take().expect("stdout is piped");
    let mut stderr_pipe = child.stderr.take().expect("stderr is piped");
    let stdout_reader = thread::spawn(move || {
        let mut text = String::new();
        let _ = stdout_pipe.read_to_string(&mut text);
        text
    });
    let stderr_reader = thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr_pipe.read_to_string(&mut text);
        text
    });

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(SmokeError::Timeout { timeout });
            }
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(err) => {
                let _ = child.kill();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(SmokeError::Startup(err));
            }
        }
    };

    let stdout = stdout_reader.join().unwrap_or_default();
    let stderr = stderr_reader.join().unwrap_or_default();

    let exit = LaunchExit::from_exit_status(status);
    if !exit.is_clean() {
        // A loader/glibc failure is a host problem, not a bad engine: classify
        // it so the UI and CLI can name the cause instead of dumping stderr
        // (spec 014 follow-up; confirmed on CI run 38980417407).
        if let Some(runtime) = classify_runtime(&stderr) {
            return Err(SmokeError::Runtime(runtime));
        }
        return Err(SmokeError::Exited { exit, stderr });
    }
    match parse_version(&stdout) {
        Some(version) => Ok(version.to_owned()),
        None => Err(SmokeError::UnexpectedOutput { stdout }),
    }
}

/// A host-runtime problem the engine reported when it tried to start.
///
/// The official DSDA-Doom AppImage does not bundle glibc or the GL driver, so
/// these failures mean the *host* is missing something, not that the download
/// was bad (spec 014 follow-up; confirmed on CI run 38980417407).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// The engine needs a newer glibc/libstdc++ than this host provides.
    Glibc {
        /// The missing symbol versions, e.g. `GLIBC_2.34`, `GLIBCXX_3.4.29`.
        missing: Vec<String>,
    },
    /// The dynamic loader could not find a shared library.
    MissingLibrary {
        /// The library name, e.g. `libOpenGL.so.0`.
        library: String,
    },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Glibc { missing } => write!(
                f,
                "DSDA-Doom needs a newer C library than this system provides (missing: {}). It must run \
                 on the host system: on the Steam Deck, remove any Compatibility tool from DDL's \
                 shortcut so DSDA-Doom runs on SteamOS directly.",
                missing.join(", ")
            ),
            Self::MissingLibrary { library } => write!(
                f,
                "DSDA-Doom could not start because the system library `{library}` is missing. \
                 Install the package that provides it, or run DDL on a system that has it."
            ),
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Classifies engine startup stderr, or returns `None` when it is an ordinary
/// engine error (an `I_Error`, a FUSE message, …).
///
/// The two patterns come from the real CI failure: a bare Ubuntu runner could
/// not find `libOpenGL.so.0`, and the Steam Runtime SDK container's extracted
/// `AppRun` needed `GLIBC_2.34`/`GLIBCXX_3.4.29`.
#[must_use]
pub fn classify_runtime(stderr: &str) -> Option<RuntimeError> {
    let mut missing = Vec::new();
    for line in stderr.lines() {
        // `dsda-doom.AppImage: error while loading shared libraries: libOpenGL.so.0: cannot open ...`
        if let Some((_, rest)) = line.split_once("error while loading shared libraries:") {
            let library = rest
                .trim_start()
                .split([':', ' '])
                .next()
                .unwrap_or("")
                .trim();
            if !library.is_empty() {
                return Some(RuntimeError::MissingLibrary {
                    library: library.to_owned(),
                });
            }
        }
        // `.../libc.so.6: version 'GLIBC_2.34' not found (required by ...)`
        if line.contains("not found") {
            for raw in line.split_whitespace() {
                let token = raw
                    .trim_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_'));
                if (token.starts_with("GLIBC_") || token.starts_with("GLIBCXX_"))
                    && token.len() > 6
                    && !missing.iter().any(|held| held == token)
                {
                    missing.push(token.to_owned());
                }
            }
        }
    }
    if missing.is_empty() {
        None
    } else {
        Some(RuntimeError::Glibc { missing })
    }
}

/// Extracts the version from `-v` output.
///
/// Returns `None` unless `stdout` starts with `dsda-doom v`, matching the
/// research's description of the version formatter; the version is the token
/// right after the prefix.
fn parse_version(stdout: &str) -> Option<&str> {
    let rest = stdout.strip_prefix(VERSION_PREFIX)?;
    let version = rest.split_whitespace().next()?;
    (!version.is_empty()).then_some(version)
}

/// Why a smoke test failed.
#[derive(Debug)]
pub enum SmokeError {
    /// The executable could not be started.
    Startup(io::Error),
    /// The executable was still running after the timeout and was killed.
    Timeout {
        /// The timeout that elapsed.
        timeout: Duration,
    },
    /// The executable exited with a non-zero status or a signal.
    Exited {
        /// How it ended.
        exit: LaunchExit,
        /// Anything it wrote to stderr, for diagnostics.
        stderr: String,
    },
    /// The engine could not start because the host runtime is missing something
    /// (a newer glibc, or a shared library). Spec 014 follow-up.
    Runtime(RuntimeError),
    /// The executable exited 0 but stdout did not start with the expected
    /// version line.
    UnexpectedOutput {
        /// What it wrote instead.
        stdout: String,
    },
}

impl fmt::Display for SmokeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Startup(err) => write!(f, "could not start the engine for `-v`: {err}"),
            Self::Timeout { timeout } => write!(
                f,
                "`-v` did not finish within {} ms and was killed",
                timeout.as_millis()
            ),
            Self::Exited { exit, stderr } => {
                write!(f, "`-v` {exit}")?;
                if !stderr.trim().is_empty() {
                    write!(f, ": {}", stderr.trim())?;
                }
                Ok(())
            }
            Self::Runtime(runtime) => write!(f, "{runtime}"),
            Self::UnexpectedOutput { stdout } => write!(
                f,
                "`-v` did not print the expected `{VERSION_PREFIX}` line (stdout: {:?})",
                first_line(stdout)
            ),
        }
    }
}

impl std::error::Error for SmokeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Startup(err) => Some(err),
            _ => None,
        }
    }
}

/// The first line of `text`, for compact error messages.
fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_parsed_from_the_first_token_after_the_prefix() {
        assert_eq!(
            parse_version("dsda-doom v0.30.1\nhttps://dsda-doom.github.io/\n"),
            Some("0.30.1")
        );
        assert_eq!(parse_version("dsda-doom v0.30.1 (abc123)"), Some("0.30.1"));
        assert_eq!(
            parse_version(VERSION_PREFIX),
            None,
            "empty version is not a version"
        );
    }

    #[test]
    fn only_the_documented_prefix_is_accepted() {
        assert_eq!(parse_version("prboom v2.6"), None);
        assert_eq!(
            parse_version("\ndsda-doom v0.30.1"),
            None,
            "the prefix must be at the start"
        );
        assert_eq!(
            parse_version("dsda-doom 0.30.1"),
            None,
            "the `v` is part of the format"
        );
    }

    #[test]
    fn error_messages_name_the_failure() {
        let error = SmokeError::Timeout {
            timeout: Duration::from_millis(250),
        };
        assert_eq!(
            error.to_string(),
            "`-v` did not finish within 250 ms and was killed"
        );

        let error = SmokeError::Exited {
            exit: LaunchExit::Failed { code: 255 },
            stderr: "I_Error: boom\n".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "`-v` exited with status 255: I_Error: boom"
        );

        let error = SmokeError::UnexpectedOutput {
            stdout: "hello\nmore".to_owned(),
        };
        assert!(error.to_string().contains("\"hello\""));
    }

    /// The real stderr from CI run 38980417407: a bare Ubuntu runner.
    const MISSING_OPENGL: &str = "dsda-doom.AppImage: error while loading shared libraries: \
        libOpenGL.so.0: cannot open shared object file: No such file or directory\n";

    /// The real stderr from CI run 38980417407: the extracted AppRun inside the
    /// Steam Runtime SDK container (glibc 2.31).
    const OLD_GLIBC: &str = "\
/tmp/ddl-e3/squashfs-root/AppRun: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.34' not found \
(required by /tmp/ddl-e3/squashfs-root/AppRun)
/tmp/ddl-e3/squashfs-root/AppRun: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.33' not found \
(required by /tmp/ddl-e3/squashfs-root/AppRun)
/tmp/ddl-e3/squashfs-root/AppRun: /usr/lib/x86_64-linux-gnu/libstdc++.so.6: version `GLIBCXX_3.4.29' not found \
(required by /tmp/ddl-e3/squashfs-root/AppRun)
";

    #[test]
    fn a_missing_shared_library_is_named() {
        assert_eq!(
            classify_runtime(MISSING_OPENGL),
            Some(RuntimeError::MissingLibrary {
                library: "libOpenGL.so.0".to_owned(),
            })
        );
        let message = classify_runtime(MISSING_OPENGL).unwrap().to_string();
        assert!(message.contains("libOpenGL.so.0"), "{message}");
    }

    #[test]
    fn old_glibc_is_classified_with_every_missing_symbol() {
        let runtime = classify_runtime(OLD_GLIBC).expect("glibc failure");
        assert_eq!(
            runtime,
            RuntimeError::Glibc {
                missing: vec![
                    "GLIBC_2.34".to_owned(),
                    "GLIBC_2.33".to_owned(),
                    "GLIBCXX_3.4.29".to_owned(),
                ],
            }
        );
        let message = runtime.to_string();
        assert!(message.contains("GLIBC_2.34"), "{message}");
        assert!(message.contains("Compatibility tool"), "{message}");
    }

    #[test]
    fn ordinary_engine_errors_are_not_misclassified() {
        assert_eq!(classify_runtime(""), None);
        assert_eq!(classify_runtime("I_Error: boom\n"), None);
        assert_eq!(
            classify_runtime("Cannot mount AppImage, please check your FUSE setup.\n"),
            None,
            "a FUSE message must stay an Exited error so the extraction fallback runs"
        );
    }

    #[test]
    fn the_runtime_variant_renders_its_message() {
        let error = SmokeError::Runtime(RuntimeError::MissingLibrary {
            library: "libOpenGL.so.0".to_owned(),
        });
        assert_eq!(
            error.to_string(),
            "DSDA-Doom could not start because the system library `libOpenGL.so.0` is missing. \
             Install the package that provides it, or run DDL on a system that has it."
        );
        // The message must stay neutral: DDL cannot claim an arbitrary host
        // provides an arbitrary library (spec 014 follow-up 2).
        assert!(!error.to_string().contains("SteamOS"), "{error}");
    }
}

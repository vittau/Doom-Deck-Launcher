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
        return Err(SmokeError::Exited { exit, stderr });
    }
    match parse_version(&stdout) {
        Some(version) => Ok(version.to_owned()),
        None => Err(SmokeError::UnexpectedOutput { stdout }),
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
}

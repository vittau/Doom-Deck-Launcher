//! Installing a DSDA-Doom AppImage: download, verify, smoke test, atomic move,
//! and the FUSE extraction fallback (spec 004, R4/R5; decision D-013).
//!
//! The whole install happens in a staging directory under `<data>/engines/
//! dsda-doom/`; only when every step has passed is the directory renamed into
//! its final `<version>/` name. A failure at any step removes the staging
//! directory, so a failed install never leaves a partial engine behind (R4).
//!
//! Verification order (research §6.1): the download pipeline checks the exact
//! size and SHA-256 while streaming; DDL then runs the spec 003 smoke test
//! (`-v`) and requires it to report the requested version. If the smoke test
//! fails with FUSE/fusermount on stderr, DDL extracts the AppImage once
//! (`--appimage-extract`) and runs `squashfs-root/AppRun` instead (R5).
//!
//! This module never records state; [`super::manager`] records the successful
//! install so state and on-disk content stay in step.

use std::fs;
use std::io::{self, Read as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::clock::Clock;
use super::smoke::{SmokeError, smoke_test_with_timeout};
use super::state::{InstalledEngine, Mode, Origin};
use crate::downloads::{CancellationToken, DownloadRequest, Fetcher, HostAllowlist, Sha256};

/// File name of the installed AppImage.
pub const APPIMAGE_FILE: &str = "dsda-doom.AppImage";

/// Directory `--appimage-extract` creates next to the AppImage.
pub const EXTRACT_DIR: &str = "squashfs-root";

/// Entry point inside the extracted tree.
pub const EXTRACT_ENTRY: &str = "AppRun";

/// Timeout for the one-off `--appimage-extract`.
pub const EXTRACT_TIMEOUT: Duration = Duration::from_secs(120);

/// How often the extraction wait loop checks the child.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// What to install and how to verify it.
#[derive(Debug, Clone)]
pub struct InstallRequest {
    /// Version string; also the final directory name.
    pub version: String,
    /// Upstream git tag, when known (attribution).
    pub tag: Option<String>,
    /// The URL to download from.
    pub source_url: String,
    /// Exact expected size in bytes.
    pub expected_size: u64,
    /// The SHA-256 the download must have.
    pub expected_sha256: Sha256,
    /// Whether the version was on the reviewed list.
    pub reviewed: bool,
}

/// Downloads, verifies, smoke-tests and atomically installs one engine.
///
/// `allowed_hosts` must permit the host of `source_url`; GitHub's signed
/// redirect target is handled by the download pipeline's default exception.
///
/// # Errors
///
/// See [`InstallError`]. On failure the staging directory is removed and no
/// state is written by this function.
pub fn install(
    engines_root: &Path,
    request: &InstallRequest,
    fetcher: &Fetcher,
    allowed_hosts: &HostAllowlist,
    clock: &dyn Clock,
) -> Result<InstalledEngine, InstallError> {
    install_with_progress(
        engines_root,
        request,
        fetcher,
        allowed_hosts,
        clock,
        &CancellationToken::new(),
        &mut |_| {},
    )
}

/// Installs with transfer notifications and cancellation before atomic commit.
pub fn install_with_progress(
    engines_root: &Path,
    request: &InstallRequest,
    fetcher: &Fetcher,
    allowed_hosts: &HostAllowlist,
    clock: &dyn Clock,
    cancel: &CancellationToken,
    progress: &mut dyn FnMut(crate::downloads::InstallProgress),
) -> Result<InstalledEngine, InstallError> {
    use crate::downloads::{InstallProgress, InstallStage};
    if cancel.is_cancelled() {
        return Err(crate::downloads::Error::Cancelled.into());
    }
    if !is_safe_component(&request.version) {
        return Err(InstallError::UnsafeVersion(request.version.clone()));
    }
    fs::create_dir_all(engines_root)
        .map_err(|error| io_error("create the engine directory", error))?;

    let staging = unique_staging_dir(engines_root, &request.version);
    fs::create_dir_all(&staging).map_err(|error| io_error("create staging directory", error))?;

    match stage(&staging, request, fetcher, allowed_hosts, cancel, progress) {
        Ok((mode, relative)) => {
            if cancel.is_cancelled() {
                let _ = fs::remove_dir_all(&staging);
                return Err(crate::downloads::Error::Cancelled.into());
            }
            progress(InstallProgress::stage(InstallStage::Commit));
            let final_dir = engines_root.join(&request.version);
            // A directory without a state record is a leftover from an
            // interrupted install; replace it.
            if final_dir.exists() {
                fs::remove_dir_all(&final_dir)
                    .map_err(|error| io_error("clear a stale engine directory", error))?;
            }
            if let Err(error) = fs::rename(&staging, &final_dir) {
                let _ = fs::remove_dir_all(&staging);
                return Err(io_error("move the engine into place", error));
            }
            Ok(InstalledEngine {
                version: request.version.clone(),
                path: final_dir.join(relative),
                mode,
                sha256: request.expected_sha256.to_hex(),
                source_url: Some(request.source_url.clone()),
                tag: request.tag.clone(),
                reviewed: request.reviewed,
                origin: Origin::Downloaded,
                installed_at: clock.now_unix_secs(),
            })
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            Err(error)
        }
    }
}

/// Performs every step inside `staging`, returning the mode and the executable's
/// path relative to the version directory.
fn stage(
    staging: &Path,
    request: &InstallRequest,
    fetcher: &Fetcher,
    allowed_hosts: &HostAllowlist,
    cancel: &CancellationToken,
    progress: &mut dyn FnMut(crate::downloads::InstallProgress),
) -> Result<(Mode, PathBuf), InstallError> {
    let download = DownloadRequest::new(
        request.source_url.clone(),
        request.expected_sha256,
        request.expected_size,
        allowed_hosts.clone(),
    )
    .with_size(request.expected_size);
    use crate::downloads::{InstallProgress, InstallStage};
    progress(InstallProgress::transfer(crate::downloads::Progress {
        downloaded: 0,
        total: Some(request.expected_size),
    }));
    let cached = fetcher.download(&download, cancel, &mut |p| {
        progress(InstallProgress::transfer(p))
    })?;
    if cancel.is_cancelled() {
        return Err(crate::downloads::Error::Cancelled.into());
    }
    progress(InstallProgress::stage(InstallStage::Verify));

    let appimage = staging.join(APPIMAGE_FILE);
    fs::copy(&cached, &appimage)
        .map_err(|error| io_error("copy the engine into staging", error))?;
    let copied = fs::metadata(&appimage)
        .map_err(|error| io_error("read the staged engine", error))?
        .len();
    if copied != request.expected_size {
        return Err(InstallError::SizeMismatch {
            expected: request.expected_size,
            actual: copied,
        });
    }
    set_executable(&appimage).map_err(|error| io_error("make the engine executable", error))?;

    match smoke_test_with_timeout(&appimage, super::smoke::SMOKE_TIMEOUT) {
        Ok(version) if version == request.version => {
            Ok((Mode::Appimage, PathBuf::from(APPIMAGE_FILE)))
        }
        Ok(version) => Err(InstallError::VersionMismatch {
            expected: request.version.clone(),
            reported: version,
        }),
        Err(SmokeError::Exited { stderr, exit: _ }) if indicates_fuse(&stderr) => {
            if cancel.is_cancelled() {
                return Err(crate::downloads::Error::Cancelled.into());
            }
            progress(InstallProgress::stage(InstallStage::Extract));
            extract_and_stage_apprun(staging, &appimage, request, cancel)
        }
        Err(error) => Err(InstallError::Smoke(Box::new(error))),
    }
}

/// The FUSE fallback (R5): extract once and use `squashfs-root/AppRun`.
fn extract_and_stage_apprun(
    staging: &Path,
    appimage: &Path,
    request: &InstallRequest,
    cancel: &CancellationToken,
) -> Result<(Mode, PathBuf), InstallError> {
    run_extraction(appimage, staging, cancel)?;

    let relative = PathBuf::from(EXTRACT_DIR).join(EXTRACT_ENTRY);
    let apprun = staging.join(&relative);
    if !apprun.is_file() {
        return Err(InstallError::ExtractionMissing { path: apprun });
    }
    set_executable(&apprun).map_err(|error| io_error("make AppRun executable", error))?;

    match smoke_test_with_timeout(&apprun, super::smoke::SMOKE_TIMEOUT) {
        Ok(version) if version == request.version => Ok((Mode::Extracted, relative)),
        Ok(version) => Err(InstallError::VersionMismatch {
            expected: request.version.clone(),
            reported: version,
        }),
        Err(error) => Err(InstallError::Smoke(Box::new(error))),
    }
}

/// Whether smoke-test stderr shows FUSE is unavailable (R5). The AppImage
/// runtime logs "Cannot mount AppImage, please check your FUSE setup." and
/// "No suitable fusermount binary" (research §2.3), so matching `fuse`
/// case-insensitively covers both.
#[must_use]
pub fn indicates_fuse(stderr: &str) -> bool {
    stderr.to_ascii_lowercase().contains("fuse")
}

/// Runs `<appimage> --appimage-extract` in `workdir`, bounded by
/// [`EXTRACT_TIMEOUT`].
fn run_extraction(
    appimage: &Path,
    workdir: &Path,
    cancel: &CancellationToken,
) -> Result<(), InstallError> {
    let mut child = Command::new(appimage)
        .arg("--appimage-extract")
        .current_dir(workdir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| io_error("start --appimage-extract", error))?;

    let mut stderr_pipe = child.stderr.take().expect("stderr is piped");
    let stderr_reader = thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr_pipe.read_to_string(&mut text);
        text
    });

    let deadline = Instant::now() + EXTRACT_TIMEOUT;
    let status = loop {
        if cancel.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stderr_reader.join();
            return Err(crate::downloads::Error::Cancelled.into());
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stderr_reader.join();
                return Err(InstallError::ExtractionTimeout {
                    timeout: EXTRACT_TIMEOUT,
                });
            }
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(error) => {
                let _ = child.kill();
                let _ = stderr_reader.join();
                return Err(io_error("wait for --appimage-extract", error));
            }
        }
    };
    let stderr = stderr_reader.join().unwrap_or_default();
    let exit = super::runner::LaunchExit::from_exit_status(status);
    if !exit.is_clean() {
        return Err(InstallError::ExtractionFailed { exit, stderr });
    }
    Ok(())
}

/// Whether `version` is a safe single path component for the engine directory.
fn is_safe_component(version: &str) -> bool {
    !version.is_empty()
        && version != "."
        && version != ".."
        && version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

/// A unique staging directory name under `root`.
fn unique_staging_dir(root: &Path, version: &str) -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    root.join(format!(
        ".staging-{version}-{}-{unique}",
        std::process::id()
    ))
}

/// Sets the Unix executable bits (0755). A no-op elsewhere, where releases are
/// not built.
#[cfg(unix)]
fn set_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
}

/// No-op on non-Unix hosts (macOS dev host still uses this on Unix; Windows is
/// not a release target).
#[cfg(not(unix))]
fn set_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn io_error(context: &str, error: io::Error) -> InstallError {
    InstallError::Io {
        context: context.to_owned(),
        detail: error.to_string(),
    }
}

/// Why an install failed (R4/R5/R6).
#[derive(Debug)]
pub enum InstallError {
    /// The version string is not a safe directory name.
    UnsafeVersion(String),
    /// The download pipeline failed (network, host, size or hash).
    Download(crate::downloads::Error),
    /// The staged file's size did not match the expected size.
    SizeMismatch {
        /// Expected bytes.
        expected: u64,
        /// Actual bytes.
        actual: u64,
    },
    /// The smoke test failed for a reason other than FUSE.
    Smoke(Box<SmokeError>),
    /// The smoke test reported a different version than requested.
    VersionMismatch {
        /// The version DDL requested.
        expected: String,
        /// The version the executable reported.
        reported: String,
    },
    /// `--appimage-extract` exited non-zero.
    ExtractionFailed {
        /// How it ended.
        exit: super::runner::LaunchExit,
        /// Anything it wrote to stderr.
        stderr: String,
    },
    /// `--appimage-extract` did not finish in time.
    ExtractionTimeout {
        /// The timeout that elapsed.
        timeout: Duration,
    },
    /// The extraction did not produce `squashfs-root/AppRun`.
    ExtractionMissing {
        /// The path that was expected.
        path: PathBuf,
    },
    /// A local filesystem operation failed.
    Io {
        /// What was being attempted.
        context: String,
        /// The underlying error.
        detail: String,
    },
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsafeVersion(version) => {
                write!(
                    f,
                    "`{version}` is not a usable engine version directory name"
                )
            }
            Self::Download(error) => write!(f, "{error}"),
            Self::SizeMismatch { expected, actual } => write!(
                f,
                "the downloaded engine is the wrong size (expected {expected} bytes, got {actual})"
            ),
            Self::Smoke(error) => write!(f, "the engine smoke test failed: {error}"),
            Self::VersionMismatch { expected, reported } => write!(
                f,
                "the engine reports version {reported} but {expected} was expected"
            ),
            Self::ExtractionFailed { exit, stderr } => {
                write!(f, "`--appimage-extract` {exit}")?;
                if !stderr.trim().is_empty() {
                    write!(f, ": {}", stderr.trim())?;
                }
                Ok(())
            }
            Self::ExtractionTimeout { timeout } => write!(
                f,
                "`--appimage-extract` did not finish within {} ms and was killed",
                timeout.as_millis()
            ),
            Self::ExtractionMissing { path } => {
                write!(f, "`--appimage-extract` did not create {}", path.display())
            }
            Self::Io { context, detail } => write!(f, "{context}: {detail}"),
        }
    }
}

impl std::error::Error for InstallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Download(error) => Some(error),
            Self::Smoke(error) => Some(error),
            _ => None,
        }
    }
}

impl From<crate::downloads::Error> for InstallError {
    fn from(error: crate::downloads::Error) -> Self {
        Self::Download(error)
    }
}

//! The DSDA-Doom engine manager (spec 004, R3–R9).
//!
//! [`EngineManager`] ties the reviewed list, release discovery, the install
//! pipeline and installed state into the operations the UI needs:
//!
//! * [`EngineManager::discover`] — refresh the cached release list;
//! * [`EngineManager::install_reviewed`] / [`EngineManager::install_unreviewed`]
//!   — install a release (the latter needs an explicit opt-in);
//! * [`EngineManager::select_engine`] / [`EngineManager::rollback`] /
//!   [`EngineManager::remove`] — switch, roll back and remove versions;
//! * [`EngineManager::register_manual`] — register an existing executable (R8);
//! * [`EngineManager::attribution`] — licence and upstream URLs for Settings (R9).
//!
//! Select, rollback and remove are refused while an engine run holds the shared
//! run guard (R7), which is the same lock spec 005's config manager uses.

use std::fmt;
use std::fs;
use std::io::{self, Read as _};
use std::path::{Path, PathBuf};

use sha2::Digest as _;

use super::clock::{Clock, SystemClock};
use super::discovery::{
    self, CACHE_DIR, CACHE_FILE, DiscoveredRelease, DiscoveryError, DiscoveryOutcome, ReleaseApi,
};
use super::install::{self, InstallError, InstallRequest};
use super::reviewed::{RELEASE_DOWNLOAD_BASE, RELEASE_TAG_BASE, ReviewedList, SOURCE_TREE_BASE};
use super::smoke::{SmokeError, smoke_test};
use super::state::{InstalledEngine, InstalledState, Mode, Origin, STATE_FILE, StateError};
use crate::downloads::{Fetcher, HostAllowlist, Sha256};
use crate::dsda_config::{DSDA_DIR, GuardError, RunGuard, recorded_pid};

/// The upstream licence of the DSDA-Doom payload (research §4.1, R9).
pub const ENGINE_LICENSE: &str = "GPL-2.0-or-later";

/// Where the engine manager reads and writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineDirs {
    /// `<data>/engines/dsda-doom`: version directories and the state file.
    pub engines: PathBuf,
    /// DDL's cache directory: the discovery cache and the download cache.
    pub cache: PathBuf,
    /// The shared run-guard directory (`<config>/dsda`, spec 005).
    pub guard: PathBuf,
}

impl EngineDirs {
    /// The standard directories under DDL's resolved XDG paths.
    #[must_use]
    pub fn for_paths(paths: &crate::platform::paths::Paths) -> Self {
        Self {
            engines: paths.data.join("engines").join("dsda-doom"),
            cache: paths.cache.clone(),
            guard: paths.config.join(DSDA_DIR),
        }
    }
}

/// The licence and upstream URLs of a version, for the Settings screen (R9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribution {
    /// The upstream licence.
    pub license: String,
    /// The upstream release page, when known.
    pub release_url: Option<String>,
    /// The tagged source tree, when known.
    pub source_url: Option<String>,
}

/// The DSDA-Doom engine manager.
pub struct EngineManager {
    dirs: EngineDirs,
    reviewed: ReviewedList,
    clock: Box<dyn Clock>,
    api: Box<dyn ReleaseApi>,
    download_base: String,
    #[cfg(any(test, feature = "test-support"))]
    allow_http_for_tests: bool,
}

impl EngineManager {
    /// Builds a manager from explicit parts (used by tests and by the binary's
    /// wiring).
    #[must_use]
    pub fn with_parts(
        dirs: EngineDirs,
        reviewed: ReviewedList,
        api: Box<dyn ReleaseApi>,
        clock: Box<dyn Clock>,
    ) -> Self {
        Self {
            dirs,
            reviewed,
            clock,
            api,
            download_base: RELEASE_DOWNLOAD_BASE.to_owned(),
            #[cfg(any(test, feature = "test-support"))]
            allow_http_for_tests: false,
        }
    }

    /// Builds a manager for DDL's resolved paths, using the embedded reviewed
    /// list, the real GitHub client and the real clock.
    ///
    /// # Panics
    ///
    /// Panics only if the embedded reviewed list is invalid, which a unit test
    /// forbids.
    #[must_use]
    pub fn for_paths(paths: &crate::platform::paths::Paths) -> Self {
        Self::with_parts(
            EngineDirs::for_paths(paths),
            ReviewedList::embedded(),
            Box::new(super::discovery::GithubApi::from_env()),
            Box::new(SystemClock),
        )
    }

    /// Allows plain-HTTP downloads. Test-only: production is HTTPS-only.
    #[cfg(any(test, feature = "test-support"))]
    pub fn with_http_for_tests(mut self) -> Self {
        self.allow_http_for_tests = true;
        self
    }

    /// Overrides the release download base URL. Test-only.
    #[cfg(any(test, feature = "test-support"))]
    pub fn with_download_base(mut self, base: impl Into<String>) -> Self {
        self.download_base = base.into();
        self
    }

    /// The directories in use.
    #[must_use]
    pub fn dirs(&self) -> &EngineDirs {
        &self.dirs
    }

    /// The reviewed list in use.
    #[must_use]
    pub fn reviewed(&self) -> &ReviewedList {
        &self.reviewed
    }

    /// Path of the installed-engine state file.
    #[must_use]
    pub fn state_path(&self) -> PathBuf {
        self.dirs.engines.join(STATE_FILE)
    }

    /// Path of the discovery cache file.
    #[must_use]
    pub fn discovery_cache_path(&self) -> PathBuf {
        self.dirs.cache.join(CACHE_DIR).join(CACHE_FILE)
    }

    /// The installed-engine state (empty when nothing is installed).
    ///
    /// # Errors
    ///
    /// Returns [`Error::State`] when the state file is corrupt.
    pub fn installed(&self) -> Result<InstalledState, Error> {
        InstalledState::load(&self.state_path()).map_err(Error::State)
    }

    /// The active engine, if one is installed.
    ///
    /// # Errors
    ///
    /// See [`EngineManager::installed`].
    pub fn active_engine(&self) -> Result<Option<InstalledEngine>, Error> {
        Ok(self.installed()?.active_entry().cloned())
    }

    /// Refreshes the release list, honoring the 24 h throttle unless `force`
    /// (R2).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Discovery`] when the request fails and no cache exists.
    pub fn discover(&self, force: bool) -> Result<DiscoveryOutcome, Error> {
        discovery::refresh(
            self.api.as_ref(),
            &self.reviewed,
            &self.discovery_cache_path(),
            self.clock.as_ref(),
            force,
        )
        .map_err(Error::Discovery)
    }

    /// Installs a reviewed version by version string (e.g. `0.30.1`) or tag
    /// (e.g. `v0.30.1`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownVersion`] when the version is not on the reviewed
    /// list, [`Error::AlreadyInstalled`] when it already is, and the install
    /// errors in [`Error::Install`] otherwise.
    pub fn install_reviewed(&self, version: &str) -> Result<InstalledEngine, Error> {
        self.install_reviewed_with_progress(
            version,
            &crate::downloads::CancellationToken::new(),
            &mut |_| {},
        )
    }

    /// Reviewed install with UI cancellation and progress callbacks.
    pub fn install_reviewed_with_progress(
        &self,
        version: &str,
        cancel: &crate::downloads::CancellationToken,
        progress: &mut dyn FnMut(crate::downloads::InstallProgress),
    ) -> Result<InstalledEngine, Error> {
        let entry = self
            .reviewed
            .get_by_version(version)
            .or_else(|| self.reviewed.get_by_tag(version))
            .ok_or_else(|| Error::UnknownVersion(version.to_owned()))?;
        let url = format!("{}/{}/{}", self.download_base, entry.tag, entry.asset);
        let request = InstallRequest {
            version: entry.version().to_owned(),
            tag: Some(entry.tag.clone()),
            source_url: url.clone(),
            expected_size: entry.size,
            expected_sha256: entry.sha256(),
            reviewed: true,
        };
        self.perform_install_with_progress(request, &url, cancel, progress)
    }

    /// Installs an unreviewed release. Requires `allow_unreviewed == true` and a
    /// non-null API digest, which is verified (R3).
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnreviewedNeedsConsent`] without the flag,
    /// [`Error::NoIntegrityValue`] when the release has no published SHA-256,
    /// [`Error::NotUnreviewed`] for a reviewed or unsupported release, and the
    /// install errors in [`Error::Install`] otherwise.
    pub fn install_unreviewed(
        &self,
        release: &DiscoveredRelease,
        allow_unreviewed: bool,
    ) -> Result<InstalledEngine, Error> {
        if !allow_unreviewed {
            return Err(Error::UnreviewedNeedsConsent);
        }
        if !release.is_unreviewed() {
            return Err(Error::NotUnreviewed(release.tag.clone()));
        }
        let asset = release
            .asset
            .as_ref()
            .ok_or_else(|| Error::UnsupportedRelease(release.tag.clone()))?;
        let Some(digest) = asset.digest else {
            // Refused by design (D-007): without an independent hash, DDL cannot
            // verify the download.
            return Err(Error::NoIntegrityValue {
                tag: release.tag.clone(),
            });
        };
        let request = InstallRequest {
            version: release.version.clone(),
            tag: Some(release.tag.clone()),
            source_url: asset.url.clone(),
            expected_size: asset.size,
            expected_sha256: digest,
            reviewed: false,
        };
        self.perform_install(request, &asset.url)
    }

    /// Runs the install pipeline and records the result (R4/R6). The first
    /// installed engine becomes active.
    fn perform_install(
        &self,
        request: InstallRequest,
        source_url: &str,
    ) -> Result<InstalledEngine, Error> {
        self.perform_install_with_progress(
            request,
            source_url,
            &crate::downloads::CancellationToken::new(),
            &mut |_| {},
        )
    }

    fn perform_install_with_progress(
        &self,
        request: InstallRequest,
        source_url: &str,
        cancel: &crate::downloads::CancellationToken,
        progress: &mut dyn FnMut(crate::downloads::InstallProgress),
    ) -> Result<InstalledEngine, Error> {
        let mut state = self.installed()?;
        if state.get(&request.version).is_some() {
            return Err(Error::AlreadyInstalled(request.version.clone()));
        }
        let host =
            host_of(source_url).ok_or_else(|| Error::InvalidSourceUrl(source_url.to_owned()))?;
        let allowed = HostAllowlist::new([host]);
        let fetcher = self.fetcher();
        let engine = install::install_with_progress(
            &self.dirs.engines,
            &request,
            &fetcher,
            &allowed,
            self.clock.as_ref(),
            cancel,
            progress,
        )
        .map_err(Error::Install)?;

        state.insert(engine.clone());
        if state.active.is_none() {
            state.active = Some(engine.version.clone());
        }
        state.save(&self.state_path()).map_err(Error::State)?;
        Ok(engine)
    }

    /// Makes `version` the active engine (R7). Records the previous active
    /// version as the rollback target.
    ///
    /// # Errors
    ///
    /// Returns [`Error::RunActive`] while an engine run holds the run guard and
    /// [`Error::NotInstalled`] when `version` is not installed.
    pub fn select_engine(&self, version: &str) -> Result<(), Error> {
        self.ensure_no_run()?;
        let mut state = self.installed()?;
        if state.get(version).is_none() {
            return Err(Error::NotInstalled(version.to_owned()));
        }
        if state.active.as_deref() == Some(version) {
            return Ok(());
        }
        state.previous_good = state.active.take();
        state.active = Some(version.to_owned());
        state.save(&self.state_path()).map_err(Error::State)
    }

    /// Re-activates the previous known-good version (R7), returning it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoPreviousVersion`] when there is nothing to roll back
    /// to, and [`Error::RunActive`] while an engine run holds the run guard.
    pub fn rollback(&self) -> Result<String, Error> {
        self.ensure_no_run()?;
        let mut state = self.installed()?;
        let previous = state
            .previous_good
            .clone()
            .ok_or(Error::NoPreviousVersion)?;
        if state.get(&previous).is_none() {
            return Err(Error::NoPreviousVersion);
        }
        // Swap so rollback is its own inverse.
        state.previous_good = state.active.take();
        state.active = Some(previous.clone());
        state.save(&self.state_path()).map_err(Error::State)?;
        Ok(previous)
    }

    /// Deletes a non-active, non-manual version (R7).
    ///
    /// # Errors
    ///
    /// Returns [`Error::RunActive`] while an engine run holds the run guard,
    /// [`Error::ActiveNotRemovable`] for the active version and
    /// [`Error::ManualNotRemovable`] for a manual engine.
    pub fn remove(&self, version: &str) -> Result<(), Error> {
        self.ensure_no_run()?;
        let mut state = self.installed()?;
        let engine = state
            .get(version)
            .cloned()
            .ok_or_else(|| Error::NotInstalled(version.to_owned()))?;
        if state.active.as_deref() == Some(version) {
            return Err(Error::ActiveNotRemovable(version.to_owned()));
        }
        if engine.is_manual() {
            return Err(Error::ManualNotRemovable(version.to_owned()));
        }

        let directory = self.dirs.engines.join(version);
        if !engine.path.starts_with(&directory) {
            return Err(Error::PathEscape(engine.path.clone()));
        }
        if directory.exists() {
            fs::remove_dir_all(&directory).map_err(Error::Io)?;
        }
        state.remove(version);
        state.save(&self.state_path()).map_err(Error::State)
    }

    /// Registers an existing executable as a manual engine (R8).
    ///
    /// It must be an absolute path to a file that passes the `-v` smoke test.
    /// It is recorded as manual and is never deleted by DDL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotAbsolute`]/[`Error::Io`] for a bad path,
    /// [`Error::Smoke`] when the smoke test fails, and
    /// [`Error::VersionConflict`] when that version is already installed.
    pub fn register_manual(&self, executable: &Path) -> Result<InstalledEngine, Error> {
        if !executable.is_absolute() {
            return Err(Error::NotAbsolute(executable.to_path_buf()));
        }
        let metadata = fs::metadata(executable).map_err(Error::Io)?;
        if !metadata.is_file() {
            return Err(Error::NotAbsolute(executable.to_path_buf()));
        }
        let version = smoke_test(executable).map_err(|error| Error::Smoke(Box::new(error)))?;

        let mut state = self.installed()?;
        if state.get(&version).is_some() {
            return Err(Error::VersionConflict(version));
        }
        let sha256 = sha256_file(executable).map_err(Error::Io)?;
        let engine = InstalledEngine {
            version: version.clone(),
            path: executable.to_path_buf(),
            mode: Mode::Manual,
            sha256: sha256.to_hex(),
            source_url: None,
            tag: None,
            reviewed: false,
            origin: Origin::Manual,
            installed_at: self.clock.now_unix_secs(),
        };
        state.insert(engine.clone());
        if state.active.is_none() {
            state.active = Some(version);
        }
        state.save(&self.state_path()).map_err(Error::State)?;
        Ok(engine)
    }

    /// The attribution data for `version` (R9).
    ///
    /// A reviewed version uses its reviewed URLs; an unreviewed installed
    /// version derives them from its tag; anything else exposes only the
    /// upstream licence.
    ///
    /// # Errors
    ///
    /// Returns [`Error::State`] when the state file is corrupt.
    pub fn attribution(&self, version: &str) -> Result<Attribution, Error> {
        if let Some(entry) = self.reviewed.get_by_version(version) {
            return Ok(Attribution {
                license: entry.license.clone(),
                release_url: Some(entry.release_url.clone()),
                source_url: Some(entry.source_url.clone()),
            });
        }
        let state = self.installed()?;
        let (release_url, source_url) =
            match state.get(version).and_then(|engine| engine.tag.as_deref()) {
                Some(tag) => (
                    Some(format!("{RELEASE_TAG_BASE}/{tag}")),
                    Some(format!("{SOURCE_TREE_BASE}/{tag}")),
                ),
                None => (None, None),
            };
        Ok(Attribution {
            license: ENGINE_LICENSE.to_owned(),
            release_url,
            source_url,
        })
    }

    /// Fails when an engine run currently holds the shared run guard (R7).
    fn ensure_no_run(&self) -> Result<(), Error> {
        if RunGuard::is_active(&self.dirs.guard).map_err(Error::from)? {
            let pid = recorded_pid(&self.dirs.guard).ok().flatten();
            return Err(Error::RunActive { pid });
        }
        Ok(())
    }

    /// A download pipeline for this manager's cache directory.
    fn fetcher(&self) -> Fetcher {
        #[allow(unused_mut)]
        let mut fetcher = Fetcher::new(self.dirs.cache.clone());
        #[cfg(any(test, feature = "test-support"))]
        {
            if self.allow_http_for_tests {
                fetcher = fetcher.allow_http_for_tests();
            }
        }
        fetcher
    }
}

/// The host of an absolute URL, lowercased.
fn host_of(url: &str) -> Option<String> {
    url::Url::parse(url).ok()?.host_str().map(str::to_owned)
}

/// Streams a file through SHA-256.
fn sha256_file(path: &Path) -> io::Result<Sha256> {
    let mut file = fs::File::open(path)?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(Sha256::finish(hasher))
}

/// Why an engine-manager operation failed (spec 004, R3–R9).
#[derive(Debug)]
pub enum Error {
    /// A filesystem operation failed.
    Io(io::Error),
    /// Installed state could not be read or written.
    State(StateError),
    /// Release discovery failed.
    Discovery(DiscoveryError),
    /// The install pipeline failed.
    Install(InstallError),
    /// An engine run is active, so the engine set must not change.
    RunActive {
        /// The PID recorded in the run lock, when it could be read.
        pid: Option<u32>,
    },
    /// No reviewed version (or installed entry) has that version.
    UnknownVersion(String),
    /// That version is not installed.
    NotInstalled(String),
    /// That version is already installed.
    AlreadyInstalled(String),
    /// Installing an unreviewed release needs an explicit opt-in.
    UnreviewedNeedsConsent,
    /// The release is reviewed or unsupported, so this path does not apply.
    NotUnreviewed(String),
    /// The release has no Linux AppImage.
    UnsupportedRelease(String),
    /// An unreviewed release has no published checksum (D-007).
    NoIntegrityValue {
        /// The upstream tag.
        tag: String,
    },
    /// A manual engine is never deleted by DDL.
    ManualNotRemovable(String),
    /// The active engine cannot be removed.
    ActiveNotRemovable(String),
    /// There is no previous known-good version to roll back to.
    NoPreviousVersion,
    /// The version is already registered by another engine.
    VersionConflict(String),
    /// A path that must be absolute is not.
    NotAbsolute(PathBuf),
    /// A download URL could not be parsed.
    InvalidSourceUrl(String),
    /// An installed path is outside its version directory.
    PathEscape(PathBuf),
    /// The `-v` smoke test failed.
    Smoke(Box<SmokeError>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::State(error) => write!(f, "{error}"),
            Self::Discovery(error) => write!(f, "{error}"),
            Self::Install(error) => write!(f, "{error}"),
            Self::RunActive { pid: Some(pid) } => {
                write!(f, "a DSDA-Doom run (PID {pid}) is active")
            }
            Self::RunActive { pid: None } => f.write_str("a DSDA-Doom run is active"),
            Self::UnknownVersion(version) => {
                write!(f, "no reviewed DSDA-Doom release matches `{version}`")
            }
            Self::NotInstalled(version) => write!(f, "DSDA-Doom {version} is not installed"),
            Self::AlreadyInstalled(version) => {
                write!(f, "DSDA-Doom {version} is already installed")
            }
            Self::UnreviewedNeedsConsent => f.write_str(
                "installing an unreviewed DSDA-Doom release needs explicit confirmation",
            ),
            Self::NotUnreviewed(tag) => {
                write!(f, "release {tag} is not an unreviewed release")
            }
            Self::UnsupportedRelease(tag) => {
                write!(f, "release {tag} has no Linux AppImage to install")
            }
            Self::NoIntegrityValue { tag } => write!(
                f,
                "DSDA-Doom release {tag} has no published checksum; DDL cannot verify it"
            ),
            Self::ManualNotRemovable(version) => write!(
                f,
                "DSDA-Doom {version} was added manually and is never deleted by DDL"
            ),
            Self::ActiveNotRemovable(version) => write!(
                f,
                "DSDA-Doom {version} is active; select another version before removing it"
            ),
            Self::NoPreviousVersion => {
                f.write_str("there is no previous known-good DSDA-Doom version to roll back to")
            }
            Self::VersionConflict(version) => write!(
                f,
                "another DSDA-Doom engine already reports version {version}"
            ),
            Self::NotAbsolute(path) => write!(
                f,
                "{} must be an existing absolute path to an executable",
                path.display()
            ),
            Self::InvalidSourceUrl(url) => write!(f, "invalid engine download URL: {url}"),
            Self::PathEscape(path) => write!(
                f,
                "refusing to delete {} because it is outside its version directory",
                path.display()
            ),
            Self::Smoke(error) => write!(f, "the engine smoke test failed: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::State(error) => Some(error),
            Self::Discovery(error) => Some(error),
            Self::Install(error) => Some(error),
            Self::Smoke(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<GuardError> for Error {
    fn from(error: GuardError) -> Self {
        match error {
            GuardError::Io(error) => Self::Io(error),
            GuardError::RunActive { pid } => Self::RunActive { pid },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloads::test_http::{Response, Server, TestDir};
    use crate::engines::dsda::discovery::CannedApi;

    /// A script that answers `-v` and `--appimage-extract` like DSDA-Doom.
    const HAPPY_APPIMAGE: &str = r#"#!/bin/sh
case "$1" in
  -v) echo "dsda-doom v0.30.1"; exit 0 ;;
  --appimage-extract)
    mkdir -p squashfs-root
    cat > squashfs-root/AppRun <<'RUN'
#!/bin/sh
echo "dsda-doom v0.30.1"
RUN
    chmod +x squashfs-root/AppRun
    exit 0 ;;
esac
exit 1
"#;

    /// A deterministic clock.
    struct FixedClock(u64);

    impl Clock for FixedClock {
        fn now_unix_secs(&self) -> u64 {
            self.0
        }
    }

    fn dirs(root: &Path) -> EngineDirs {
        EngineDirs {
            engines: root.join("data/engines/dsda-doom"),
            cache: root.join("cache"),
            guard: root.join("config/dsda"),
        }
    }

    fn manager(root: &Path) -> EngineManager {
        EngineManager::with_parts(
            dirs(root),
            ReviewedList::embedded(),
            Box::new(CannedApi::new()),
            Box::new(FixedClock(1000)),
        )
        .with_http_for_tests()
    }

    fn write_script(path: &Path, body: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
    }

    /// Serves `body` on a loopback HTTP server and returns the server plus the
    /// SHA-256 and length the caller must declare.
    fn serve(body: &str) -> (Server, Sha256, u64) {
        let bytes = body.as_bytes().to_vec();
        let length = bytes.len() as u64;
        let hash = Sha256::of(&bytes);
        let server = Server::start(move |_req| Response::ok(bytes.clone()));
        (server, hash, length)
    }

    #[test]
    fn installing_a_reviewed_version_downloads_verifies_and_activates() {
        let temp = TestDir::new("install-happy");
        let (server, hash, length) = serve(HAPPY_APPIMAGE);
        let manager = manager(temp.path()).with_download_base(server.url(""));
        // Override the reviewed asset size/hash with the script's own values by
        // installing through the unreviewed path is not possible (no consent);
        // instead call the pipeline with a reviewed-shaped request.
        let request = InstallRequest {
            version: "0.30.1".to_owned(),
            tag: Some("v0.30.1".to_owned()),
            source_url: server.url("/dsda-doom-0.30.1-linux-x86_64.appimage"),
            expected_size: length,
            expected_sha256: hash,
            reviewed: true,
        };
        let engine = manager
            .perform_install(request, &server.url("/x"))
            .expect("install");

        assert_eq!(engine.version, "0.30.1");
        assert_eq!(engine.mode, Mode::Appimage);
        assert_eq!(engine.origin, Origin::Downloaded);
        assert!(engine.reviewed);
        assert_eq!(engine.sha256, hash.to_hex());
        assert!(engine.path.is_file());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = fs::metadata(&engine.path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o755, "the AppImage is executable");
        }

        let state = manager.installed().expect("state");
        assert_eq!(state.active.as_deref(), Some("0.30.1"));
        assert_eq!(state.entries().len(), 1);
        assert_no_staging(temp.path());
    }

    #[test]
    fn a_reviewed_install_by_name_builds_the_upstream_url() {
        // A reviewed list whose size/hash match the fake script lets the whole
        // `install_reviewed` path run against the loopback server.
        let temp = TestDir::new("install-url");
        let (server, hash, length) = serve(HAPPY_APPIMAGE);
        let list = ReviewedList::parse(&format!(
            r#"
schema_version = 1

[[version]]
tag = "v0.30.1"
asset = "dsda-doom-0.30.1-linux-x86_64.appimage"
size = {length}
sha256 = "{hash}"
license = "GPL-2.0-or-later"
release_url = "https://example.invalid/r"
source_url = "https://example.invalid/s"
"#,
            hash = hash.to_hex()
        ))
        .expect("list");
        let manager = EngineManager::with_parts(
            dirs(temp.path()),
            list,
            Box::new(CannedApi::new()),
            Box::new(FixedClock(1)),
        )
        .with_http_for_tests()
        .with_download_base(server.url(""));

        let engine = manager.install_reviewed("0.30.1").expect("install");
        assert_eq!(engine.version, "0.30.1");
        assert!(engine.reviewed);
        assert_eq!(
            engine.source_url.as_deref(),
            Some(format!(
                "{}/v0.30.1/dsda-doom-0.30.1-linux-x86_64.appimage",
                server.url("")
            ))
            .as_deref()
        );
    }

    #[test]
    fn a_size_or_hash_mismatch_leaves_no_partial_install() {
        let temp = TestDir::new("install-bad");
        let (server, hash, length) = serve(HAPPY_APPIMAGE);
        let manager = manager(temp.path());

        // Wrong expected size.
        let request = InstallRequest {
            version: "0.30.1".to_owned(),
            tag: None,
            source_url: server.url("/f"),
            expected_size: length + 1,
            expected_sha256: hash,
            reviewed: true,
        };
        assert!(manager.perform_install(request, &server.url("/f")).is_err());

        // Wrong expected hash (right size).
        let request = InstallRequest {
            version: "0.30.2".to_owned(),
            tag: None,
            source_url: server.url("/f"),
            expected_size: length,
            expected_sha256: Sha256::of(b"different"),
            reviewed: true,
        };
        assert!(manager.perform_install(request, &server.url("/f")).is_err());

        assert!(!dirs(temp.path()).engines.join("0.30.1").exists());
        assert!(!dirs(temp.path()).engines.join("0.30.2").exists());
        assert!(manager.installed().unwrap().entries().is_empty());
        assert_no_staging(temp.path());
    }

    #[test]
    fn a_fuse_smoke_failure_falls_back_to_extraction() {
        const FUSE_APPIMAGE: &str = r#"#!/bin/sh
case "$1" in
  -v) echo "Cannot mount AppImage, please check your FUSE setup." 1>&2; exit 1 ;;
  --appimage-extract)
    mkdir -p squashfs-root
    cat > squashfs-root/AppRun <<'RUN'
#!/bin/sh
echo "dsda-doom v0.30.1"
RUN
    chmod +x squashfs-root/AppRun
    exit 0 ;;
esac
exit 1
"#;
        let temp = TestDir::new("install-fuse");
        let (server, hash, length) = serve(FUSE_APPIMAGE);
        let manager = manager(temp.path());
        let request = InstallRequest {
            version: "0.30.1".to_owned(),
            tag: None,
            source_url: server.url("/f"),
            expected_size: length,
            expected_sha256: hash,
            reviewed: false,
        };
        let engine = manager
            .perform_install(request, &server.url("/f"))
            .expect("fallback");

        assert_eq!(engine.mode, Mode::Extracted);
        assert!(
            engine.path.ends_with("squashfs-root/AppRun"),
            "{}",
            engine.path.display()
        );
        assert!(engine.path.is_file());
        // The whole extracted tree is kept.
        assert!(
            dirs(temp.path())
                .engines
                .join("0.30.1/squashfs-root")
                .is_dir()
        );
        assert_no_staging(temp.path());
    }

    #[test]
    fn a_non_fuse_smoke_failure_is_reported_and_leaves_nothing() {
        const BROKEN: &str = r#"#!/bin/sh
echo "I_Error: boom" 1>&2
exit 255
"#;
        let temp = TestDir::new("install-broken");
        let (server, hash, length) = serve(BROKEN);
        let manager = manager(temp.path());
        let request = InstallRequest {
            version: "0.30.1".to_owned(),
            tag: None,
            source_url: server.url("/f"),
            expected_size: length,
            expected_sha256: hash,
            reviewed: true,
        };
        let error = manager
            .perform_install(request, &server.url("/f"))
            .unwrap_err();
        assert!(matches!(error, Error::Install(_)), "{error}");
        assert!(!dirs(temp.path()).engines.join("0.30.1").exists());
        assert_no_staging(temp.path());
    }

    #[test]
    fn unreviewed_installs_require_consent_and_a_digest() {
        use crate::engines::dsda::discovery::{AssetInfo, Availability, DiscoveredRelease};

        const UNREVIEWED_APPIMAGE: &str = r#"#!/bin/sh
if [ "$1" = "-v" ]; then echo "dsda-doom v9.9.9"; exit 0; fi
exit 1
"#;
        let temp = TestDir::new("install-unreviewed");
        let (server, hash, length) = serve(UNREVIEWED_APPIMAGE);
        let manager = manager(temp.path());
        let release = DiscoveredRelease {
            tag: "v9.9.9".to_owned(),
            version: "9.9.9".to_owned(),
            release_url: None,
            published_at: None,
            asset: Some(AssetInfo {
                name: "dsda-doom-9.9.9-linux-x86_64.appimage".to_owned(),
                size: length,
                url: server.url("/f"),
                digest: Some(hash),
            }),
            availability: Availability::Unreviewed,
        };

        assert!(matches!(
            manager.install_unreviewed(&release, false),
            Err(Error::UnreviewedNeedsConsent)
        ));
        let engine = manager
            .install_unreviewed(&release, true)
            .expect("consented install");
        assert!(!engine.reviewed);
        assert_eq!(engine.origin, Origin::Downloaded);

        // A reviewed release is not installable through the unreviewed path.
        let reviewed = DiscoveredRelease {
            availability: Availability::Reviewed,
            ..release
        };
        assert!(matches!(
            manager.install_unreviewed(&reviewed, true),
            Err(Error::NotUnreviewed(_))
        ));
    }

    #[test]
    fn a_null_digest_unreviewed_release_is_refused() {
        use crate::engines::dsda::discovery::{AssetInfo, Availability, DiscoveredRelease};

        let temp = TestDir::new("install-nodigest");
        let manager = manager(temp.path());
        let release = DiscoveredRelease {
            tag: "v0.28.1".to_owned(),
            version: "0.28.1".to_owned(),
            release_url: None,
            published_at: None,
            asset: Some(AssetInfo {
                name: "dsda-doom-0.28.1-linux-x86_64.appimage".to_owned(),
                size: 10,
                url: "https://github.com/dsda-org/dsda-doom/releases/download/v0.28.1/x".to_owned(),
                digest: None,
            }),
            availability: Availability::Unreviewed,
        };
        let error = manager.install_unreviewed(&release, true).unwrap_err();
        assert!(matches!(error, Error::NoIntegrityValue { ref tag } if tag == "v0.28.1"));
        assert!(error.to_string().contains("no published checksum"));
    }

    /// Seeds two downloaded versions and their directories.
    fn seed_two(temp: &TestDir) -> EngineManager {
        let manager = manager(temp.path());
        let mut state = InstalledState::new();
        for (version, mode) in [("0.30.1", Mode::Appimage), ("0.30.0", Mode::Extracted)] {
            let dir = dirs(temp.path()).engines.join(version);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("dsda-doom.AppImage"), b"x").unwrap();
            state.insert(InstalledEngine {
                version: version.to_owned(),
                path: dir.join("dsda-doom.AppImage"),
                mode,
                sha256: "00".repeat(32),
                source_url: Some(format!("https://github.com/example/{version}")),
                tag: Some(format!("v{version}")),
                reviewed: true,
                origin: Origin::Downloaded,
                installed_at: 100,
            });
        }
        state.active = Some("0.30.1".to_owned());
        state.save(&manager.state_path()).unwrap();
        manager
    }

    #[test]
    fn select_rollback_and_remove_follow_the_rules() {
        let temp = TestDir::new("select");
        let manager = seed_two(&temp);

        // Select the other version; the old active becomes the rollback target.
        manager.select_engine("0.30.0").expect("select");
        let state = manager.installed().unwrap();
        assert_eq!(state.active.as_deref(), Some("0.30.0"));
        assert_eq!(state.previous_good.as_deref(), Some("0.30.1"));

        // The active version cannot be removed.
        assert!(matches!(
            manager.remove("0.30.0"),
            Err(Error::ActiveNotRemovable(_))
        ));

        // Rollback swaps back.
        assert_eq!(manager.rollback().expect("rollback"), "0.30.1");
        let state = manager.installed().unwrap();
        assert_eq!(state.active.as_deref(), Some("0.30.1"));
        assert_eq!(state.previous_good.as_deref(), Some("0.30.0"));

        // Remove the now-inactive version and its directory.
        manager.remove("0.30.0").expect("remove");
        assert!(!dirs(temp.path()).engines.join("0.30.0").exists());
        assert!(manager.installed().unwrap().get("0.30.0").is_none());

        assert!(matches!(
            manager.select_engine("nope"),
            Err(Error::NotInstalled(_))
        ));
        assert!(matches!(
            manager.remove("nope"),
            Err(Error::NotInstalled(_))
        ));
    }

    #[test]
    fn rollback_without_a_previous_version_is_a_typed_error() {
        let temp = TestDir::new("rollback-none");
        let manager = seed_two(&temp);
        manager.installed().unwrap(); // sanity
        // 0.30.1 is active with no previous_good yet.
        assert!(matches!(manager.rollback(), Err(Error::NoPreviousVersion)));
    }

    #[test]
    fn select_rollback_and_remove_are_refused_while_a_run_is_active() {
        let temp = TestDir::new("run-guard");
        let manager = seed_two(&temp);

        let guard = RunGuard::acquire(&dirs(temp.path()).guard).expect("acquire guard");
        assert!(matches!(
            manager.select_engine("0.30.0"),
            Err(Error::RunActive { .. })
        ));
        assert!(matches!(manager.rollback(), Err(Error::RunActive { .. })));
        assert!(matches!(
            manager.remove("0.30.0"),
            Err(Error::RunActive { .. })
        ));
        guard.end().expect("release");

        manager
            .select_engine("0.30.0")
            .expect("allowed after release");
    }

    #[test]
    fn register_manual_records_a_manual_engine_that_cannot_be_removed() {
        let temp = TestDir::new("manual");
        let executable = temp.path().join("my-dsda.sh");
        write_script(&executable, "#!/bin/sh\necho \"dsda-doom v0.31.0\"\n");

        let manager = manager(temp.path());
        let engine = manager.register_manual(&executable).expect("register");

        assert_eq!(engine.version, "0.31.0");
        assert_eq!(engine.mode, Mode::Manual);
        assert_eq!(engine.origin, Origin::Manual);
        assert!(!engine.reviewed);
        assert_eq!(
            engine.sha256,
            Sha256::of(&fs::read(&executable).unwrap()).to_hex()
        );
        assert_eq!(
            manager.installed().unwrap().active.as_deref(),
            Some("0.31.0")
        );

        assert!(matches!(
            manager.remove("0.31.0"),
            Err(Error::ActiveNotRemovable(_))
        ));
        // Even inactive, a manual engine is never deleted.
        let other = temp.path().join("other.sh");
        write_script(&other, "#!/bin/sh\necho \"dsda-doom v0.32.0\"\n");
        manager.register_manual(&other).expect("second");
        manager.select_engine("0.32.0").expect("select");
        assert!(matches!(
            manager.remove("0.31.0"),
            Err(Error::ManualNotRemovable(_))
        ));

        // A relative path and a duplicate version are rejected.
        assert!(matches!(
            manager.register_manual(Path::new("relative.sh")),
            Err(Error::NotAbsolute(_))
        ));
        assert!(matches!(
            manager.register_manual(&executable),
            Err(Error::VersionConflict(_))
        ));
    }

    #[test]
    fn attribution_uses_reviewed_then_derived_urls() {
        let temp = TestDir::new("attribution");
        let manager = seed_two(&temp);

        let reviewed = manager.attribution("0.30.1").expect("reviewed");
        assert_eq!(reviewed.license, "GPL-2.0-or-later");
        assert_eq!(
            reviewed.release_url.as_deref(),
            Some("https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.1")
        );
        assert_eq!(
            reviewed.source_url.as_deref(),
            Some("https://github.com/dsda-org/dsda-doom/tree/v0.30.1")
        );

        // Not reviewed, but installed with a tag: derive both URLs.
        let derived = manager.attribution("0.30.0").expect("derived");
        assert_eq!(derived.license, ENGINE_LICENSE);
        assert_eq!(
            derived.release_url.as_deref(),
            Some("https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.0")
        );
    }

    #[test]
    fn reviewed_install_uses_the_download_base_override() {
        let temp = TestDir::new("base-override");
        let (server, _, _) = serve(HAPPY_APPIMAGE);
        let manager = manager(temp.path()).with_download_base(server.url(""));
        assert_eq!(manager.download_base, server.url(""));
    }

    #[test]
    fn unknown_reviewed_version_is_rejected() {
        let temp = TestDir::new("unknown");
        let manager = manager(temp.path());
        assert!(matches!(
            manager.install_reviewed("1.2.3"),
            Err(Error::UnknownVersion(_))
        ));
    }

    #[test]
    fn state_path_and_fetcher_are_under_the_configured_dirs() {
        let temp = TestDir::new("paths");
        let manager = manager(temp.path());
        assert_eq!(
            manager.state_path(),
            dirs(temp.path()).engines.join(STATE_FILE)
        );
        assert_eq!(
            manager.discovery_cache_path(),
            dirs(temp.path()).cache.join(CACHE_DIR).join(CACHE_FILE)
        );
    }

    fn assert_no_staging(root: &Path) {
        let engines = dirs(root).engines;
        if let Ok(entries) = fs::read_dir(&engines) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                assert!(!name.starts_with(".staging-"), "leftover staging: {name}");
            }
        }
    }
}

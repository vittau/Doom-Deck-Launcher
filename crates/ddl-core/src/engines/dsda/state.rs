//! Installed-engine state: versions, the active one and the previous
//! known-good one (spec 004, R6).
//!
//! State is machine-written JSON with a versioned schema and atomic writes
//! (decision D-006). Each installed engine records the executable to run (the
//! AppImage file, the extracted `AppRun`, or a manual path), how it is run
//! ([`Mode`]), its SHA-256, where it came from, whether it was reviewed, and
//! when it was installed. [`InstalledState::active`] is the version DDL would
//! launch and [`InstalledState::previous_good`] is the rollback target.
//!
//! The type only stores and retrieves; the rules that need the shared run guard
//! (select / rollback / remove) live in [`super::manager`].

use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Schema version of the installed-engine state.
pub const SCHEMA_VERSION: u32 = 1;

/// File name of the installed-engine state inside the engine directory.
pub const STATE_FILE: &str = "installed.json";

/// How an installed engine is run (R4/R5/R8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// The AppImage is executed directly; FUSE mounts it on launch.
    Appimage,
    /// The AppImage was extracted once and `squashfs-root/AppRun` is executed.
    Extracted,
    /// A path the user registered directly.
    Manual,
}

/// Where an installed engine came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    /// Downloaded from upstream GitHub Releases.
    Downloaded,
    /// Registered by the user from an existing executable (R8).
    Manual,
}

/// One installed engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledEngine {
    /// Version string (the reviewed tag without the leading `v`, or whatever a
    /// manual engine reports from `-v`). Also the install directory name.
    pub version: String,
    /// Absolute path of the executable DDL runs.
    pub path: PathBuf,
    /// How the executable is run.
    pub mode: Mode,
    /// SHA-256 of the installed executable.
    pub sha256: String,
    /// The URL the executable was downloaded from, when it was downloaded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// The upstream git tag, when known (attribution and source URL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Whether this version was on the reviewed list when installed (R6).
    pub reviewed: bool,
    /// Where it came from.
    pub origin: Origin,
    /// When it was installed, in Unix seconds.
    pub installed_at: u64,
}

impl InstalledEngine {
    /// Whether this engine was registered manually (never deleted by DDL, R8).
    #[must_use]
    pub fn is_manual(&self) -> bool {
        self.origin == Origin::Manual
    }
}

/// The persisted set of installed engines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledState {
    /// Persisted schema version.
    pub schema_version: u32,
    /// Installed engines.
    #[serde(default)]
    pub installed: Vec<InstalledEngine>,
    /// The active version, or `None` before the first install.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<String>,
    /// The previous known-good version, used by [`super::manager::EngineManager::rollback`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_good: Option<String>,
}

impl Default for InstalledState {
    fn default() -> Self {
        Self::new()
    }
}

impl InstalledState {
    /// An empty state at the current schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            installed: Vec::new(),
            active: None,
            previous_good: None,
        }
    }

    /// The installed engines.
    #[must_use]
    pub fn entries(&self) -> &[InstalledEngine] {
        &self.installed
    }

    /// The entry for `version`, if any.
    #[must_use]
    pub fn get(&self, version: &str) -> Option<&InstalledEngine> {
        self.installed
            .iter()
            .find(|engine| engine.version == version)
    }

    /// The active entry, when its version is still installed.
    #[must_use]
    pub fn active_entry(&self) -> Option<&InstalledEngine> {
        self.active.as_deref().and_then(|version| self.get(version))
    }

    /// The previous known-good entry, when its version is still installed.
    #[must_use]
    pub fn previous_entry(&self) -> Option<&InstalledEngine> {
        self.previous_good
            .as_deref()
            .and_then(|version| self.get(version))
    }

    /// Inserts `engine`, replacing any entry with the same version.
    pub fn insert(&mut self, engine: InstalledEngine) {
        if let Some(existing) = self
            .installed
            .iter_mut()
            .find(|held| held.version == engine.version)
        {
            *existing = engine;
        } else {
            self.installed.push(engine);
        }
    }

    /// Removes the entry for `version`, returning it.
    pub fn remove(&mut self, version: &str) -> Option<InstalledEngine> {
        let index = self
            .installed
            .iter()
            .position(|engine| engine.version == version)?;
        let removed = self.installed.remove(index);
        if self.previous_good.as_deref() == Some(version) {
            self.previous_good = None;
        }
        Some(removed)
    }

    /// Loads state from `path`, or the empty state when the file does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`StateError`] when the file cannot be read, is not valid JSON,
    /// or declares an unsupported schema version.
    pub fn load(path: &Path) -> Result<Self, StateError> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::new()),
            Err(error) => return Err(StateError::Io(error)),
        };
        let state: Self =
            serde_json::from_str(&text).map_err(|error| StateError::Parse(error.to_string()))?;
        if state.schema_version != SCHEMA_VERSION {
            return Err(StateError::UnsupportedSchema(state.schema_version));
        }
        Ok(state)
    }

    /// Writes state to `path` atomically (temp file + rename).
    ///
    /// # Errors
    ///
    /// Returns [`StateError::Io`] when the file cannot be written.
    pub fn save(&self, path: &Path) -> Result<(), StateError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|error| StateError::Parse(error.to_string()))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(StateError::Io)?;
        }
        let temporary = temporary_path(path);
        let write = || -> io::Result<()> {
            let mut file = File::create(&temporary)?;
            file.write_all(json.as_bytes())?;
            file.write_all(b"\n")?;
            file.sync_all()
        };
        if let Err(error) = write() {
            let _ = fs::remove_file(&temporary);
            return Err(StateError::Io(error));
        }
        fs::rename(&temporary, path).map_err(StateError::Io)
    }
}

/// A unique sibling temporary path.
fn temporary_path(path: &Path) -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| STATE_FILE.to_owned());
    path.with_file_name(format!(".{name}.tmp.{}.{unique}", std::process::id()))
}

/// Why loading or saving state failed.
#[derive(Debug)]
pub enum StateError {
    /// Reading or writing the state file failed.
    Io(io::Error),
    /// The state file is not valid JSON.
    Parse(String),
    /// The state file declares an unsupported schema version.
    UnsupportedSchema(u32),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "installed-engine state i/o error: {error}"),
            Self::Parse(detail) => write!(f, "invalid installed-engine state: {detail}"),
            Self::UnsupportedSchema(version) => write!(
                f,
                "installed-engine state schema_version {version} is not supported (expected {SCHEMA_VERSION})"
            ),
        }
    }
}

impl std::error::Error for StateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine(version: &str, mode: Mode, origin: Origin) -> InstalledEngine {
        InstalledEngine {
            version: version.to_owned(),
            path: PathBuf::from(format!(
                "/data/engines/dsda-doom/{version}/dsda-doom.AppImage"
            )),
            mode,
            sha256: "00".repeat(32),
            source_url: Some(
                "https://github.com/dsda-org/dsda-doom/releases/download/v0.30.1/a".to_owned(),
            ),
            tag: Some(format!("v{version}")),
            reviewed: origin == Origin::Downloaded,
            origin,
            installed_at: 1000,
        }
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("engines/dsda-doom").join(STATE_FILE);

        let mut state = InstalledState::new();
        state.insert(engine("0.30.1", Mode::Appimage, Origin::Downloaded));
        state.insert(engine("0.30.0", Mode::Extracted, Origin::Downloaded));
        state.active = Some("0.30.1".to_owned());
        state.previous_good = Some("0.30.0".to_owned());

        state.save(&path).expect("save");
        let loaded = InstalledState::load(&path).expect("load");
        assert_eq!(loaded, state);
    }

    #[test]
    fn a_missing_file_is_the_empty_state() {
        let dir = tempfile::tempdir().unwrap();
        let state = InstalledState::load(&dir.path().join("nothing.json")).expect("empty");
        assert_eq!(state, InstalledState::new());
    }

    #[test]
    fn unsupported_schema_and_invalid_json_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(STATE_FILE);
        fs::write(&path, "not json").unwrap();
        assert!(matches!(
            InstalledState::load(&path),
            Err(StateError::Parse(_))
        ));
        fs::write(&path, r#"{"schema_version":9,"installed":[]}"#).unwrap();
        assert!(matches!(
            InstalledState::load(&path),
            Err(StateError::UnsupportedSchema(9))
        ));
    }

    #[test]
    fn insert_replaces_by_version_and_remove_clears_previous_good() {
        let mut state = InstalledState::new();
        state.insert(engine("0.30.1", Mode::Appimage, Origin::Downloaded));
        state.insert(engine("0.30.1", Mode::Extracted, Origin::Downloaded));
        assert_eq!(state.entries().len(), 1);
        assert_eq!(state.get("0.30.1").unwrap().mode, Mode::Extracted);

        state.previous_good = Some("0.30.1".to_owned());
        assert!(state.remove("0.30.1").is_some());
        assert!(state.previous_good.is_none());
    }
}

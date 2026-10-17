//! The IWAD registry: identified files, user choices, atomic JSON state
//! (spec 007, R5).
//!
//! [`Registry`] records each identified file's path, variant, SHA-256, size,
//! modification time and where it came from. Ambiguous files are stored with
//! their candidate list until the user picks one with [`Registry::choose`].
//! [`Registry::refresh`] re-checks existence and re-hashes only files whose
//! size or modification time changed. [`Registry::best_for`] answers a
//! recipe's request, preferring classic (`base/`) copies over KEX
//! (`rerelease/`) ones unless the caller names the variants it accepts.

use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::identify::{Identification, Outcome};
use super::table::{Game, IwadTable};

/// Registry schema version understood by this build.
pub const SCHEMA_VERSION: u32 = 1;

/// Where an identified file came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// A Steam library's known install directories.
    Steam,
    /// A folder the user chose.
    UserFolder,
    /// A file the user pointed DDL at directly.
    Manual,
    /// A GOG install recorded by Heroic Games Launcher (spec 021, R2).
    #[serde(rename = "gog_heroic")]
    GogHeroic,
    /// A GOG install directory from minigalaxy's setting (spec 021, R2).
    #[serde(rename = "gog_minigalaxy")]
    GogMinigalaxy,
    /// A GOG offline installer's default folder (spec 021, R2).
    #[serde(rename = "gog_offline")]
    GogOffline,
    /// A generic root, only when "Search standard Linux folders" is on
    /// (spec 021, R4).
    #[serde(rename = "standard_folder")]
    StandardFolder,
}

impl Source {
    /// The label shown in the IWAD list and `ddl iwad scan` (spec 021, R3).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Source::Steam => "Steam",
            Source::UserFolder => "User folder",
            Source::Manual => "Manual",
            Source::GogHeroic => "GOG via Heroic",
            Source::GogMinigalaxy => "GOG via minigalaxy",
            Source::GogOffline => "GOG offline installer",
            Source::StandardFolder => "Standard folder",
        }
    }
}

/// A file modification time, as whole seconds plus nanoseconds since the Unix
/// epoch. Stored in the registry so [`Registry::refresh`] can skip re-hashing
/// unchanged files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTime {
    /// Seconds since the Unix epoch.
    pub secs: i64,
    /// Sub-second nanoseconds.
    pub nanos: u32,
}

impl FileTime {
    /// Reads the modification time from file metadata.
    #[must_use]
    pub fn from_metadata(metadata: &fs::Metadata) -> Option<Self> {
        metadata.modified().ok().and_then(Self::from_system_time)
    }

    /// Converts a [`SystemTime`], or `None` if it predates the Unix epoch.
    #[must_use]
    pub fn from_system_time(time: SystemTime) -> Option<Self> {
        let duration = time.duration_since(UNIX_EPOCH).ok()?;
        Some(Self {
            secs: i64::try_from(duration.as_secs()).ok()?,
            nanos: duration.subsec_nanos(),
        })
    }
}

/// One identified (or user-unresolved) file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Absolute path to the user's file.
    pub path: PathBuf,
    /// The matched variant id, once known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// Candidate variant ids while the choice is ambiguous.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<String>,
    /// The variant's game, once known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game: Option<Game>,
    /// SHA-256 of the file, recorded when it was last identified.
    pub sha256: String,
    /// File size in bytes at that time.
    pub size: u64,
    /// Modification time at that time.
    pub mtime: FileTime,
    /// Where the file came from.
    pub source: Source,
}

/// What [`Registry::refresh`] did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RefreshSummary {
    /// Entries inspected.
    pub checked: usize,
    /// Entries whose contents were re-hashed (size or mtime changed).
    pub rehashed: usize,
    /// Entries whose recorded content changed.
    pub changed: usize,
    /// Entries dropped because the file is gone or is no longer a known IWAD.
    pub removed: usize,
}

/// The registry and its persisted form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registry {
    /// Persisted schema version.
    pub schema_version: u32,
    /// Identified files.
    #[serde(default)]
    pub entries: Vec<RegistryEntry>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// An empty registry at the current schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            entries: Vec::new(),
        }
    }

    /// The recorded entries.
    #[must_use]
    pub fn entries(&self) -> &[RegistryEntry] {
        &self.entries
    }

    /// The entry for `path`, if any.
    #[must_use]
    pub fn get(&self, path: &Path) -> Option<&RegistryEntry> {
        self.entries.iter().find(|entry| entry.path == path)
    }

    /// Inserts `entry`, replacing any entry with the same path.
    pub fn insert(&mut self, entry: RegistryEntry) {
        if let Some(existing) = self.entries.iter_mut().find(|held| held.path == entry.path) {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
    }

    /// Removes the entry for `path`, reporting whether it existed.
    pub fn remove(&mut self, path: &Path) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| entry.path != path);
        self.entries.len() != before
    }

    /// Records a file from its [`Identification`].
    ///
    /// Returns `false` (and stores nothing) when the file is not a recognised
    /// base game or its metadata cannot be read.
    pub fn record(&mut self, path: &Path, identification: &Identification, source: Source) -> bool {
        let Ok(metadata) = fs::metadata(path) else {
            return false;
        };
        let Some(mtime) = FileTime::from_metadata(&metadata) else {
            return false;
        };

        let (variant, candidates, game) = match &identification.outcome {
            Outcome::Known { id, game } => (Some(id.clone()), Vec::new(), Some(*game)),
            Outcome::Ambiguous { candidates, .. } => (None, candidates.clone(), None),
            Outcome::Unknown => return false,
        };

        self.insert(RegistryEntry {
            path: path.to_path_buf(),
            variant,
            candidates,
            game,
            sha256: identification.sha256.clone(),
            size: identification.size,
            mtime,
            source,
        });
        true
    }

    /// Resolves an ambiguous file to `variant_id`.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError::UnknownVariant`] when `variant_id` is not in
    /// `table`, or [`RegistryError::NotRegistered`] when `path` is unknown.
    pub fn choose(
        &mut self,
        path: &Path,
        variant_id: &str,
        table: &IwadTable,
    ) -> Result<(), RegistryError> {
        let variant = table
            .get(variant_id)
            .ok_or_else(|| RegistryError::UnknownVariant(variant_id.to_owned()))?;
        let entry = self
            .entries
            .iter_mut()
            .find(|entry| entry.path == path)
            .ok_or(RegistryError::NotRegistered)?;
        entry.variant = Some(variant.id.clone());
        entry.game = Some(variant.game);
        entry.candidates.clear();
        Ok(())
    }

    /// Re-checks every entry's existence and re-hashes changed files.
    ///
    /// A file is "changed" when its size or modification time differs from the
    /// record; only those are re-hashed. Changed files are re-identified, so a
    /// replaced file picks up its new variant (or leaves the registry when it
    /// is no longer a known base game). Missing files are dropped.
    pub fn refresh(&mut self, table: &IwadTable) -> RefreshSummary {
        let mut summary = RefreshSummary::default();
        let mut kept = Vec::with_capacity(self.entries.len());

        for mut entry in std::mem::take(&mut self.entries) {
            summary.checked += 1;
            let Ok(metadata) = fs::metadata(&entry.path) else {
                summary.removed += 1;
                continue;
            };
            if !metadata.is_file() {
                summary.removed += 1;
                continue;
            }

            let size = metadata.len();
            let mtime = FileTime::from_metadata(&metadata);
            let changed = size != entry.size || mtime != Some(entry.mtime);
            if !changed {
                kept.push(entry);
                continue;
            }

            summary.rehashed += 1;
            match super::identify::identify(&entry.path, table) {
                Ok(identification) => {
                    if identification.sha256 != entry.sha256 {
                        summary.changed += 1;
                    }
                    match identification.outcome {
                        Outcome::Known { id, game } => {
                            entry.variant = Some(id);
                            entry.game = Some(game);
                            entry.candidates.clear();
                            entry.sha256 = identification.sha256;
                            entry.size = identification.size;
                            entry.mtime = mtime.unwrap_or(entry.mtime);
                            kept.push(entry);
                        }
                        Outcome::Ambiguous { candidates, .. } => {
                            entry.variant = None;
                            entry.game = None;
                            entry.candidates = candidates;
                            entry.sha256 = identification.sha256;
                            entry.size = identification.size;
                            entry.mtime = mtime.unwrap_or(entry.mtime);
                            kept.push(entry);
                        }
                        Outcome::Unknown => summary.removed += 1,
                    }
                }
                Err(_) => {
                    summary.changed += 1;
                    summary.removed += 1;
                }
            }
        }

        self.entries = kept;
        summary
    }

    /// The best recorded IWAD for `game`.
    ///
    /// Only exactly identified entries qualify. When `accepted_variants` is
    /// non-empty, only those variant ids are considered. Among the rest,
    /// classic copies are preferred over KEX `rerelease/` copies, so vanilla
    /// content gets the uncensored 1.9 data unless a recipe asks otherwise.
    #[must_use]
    pub fn best_for(&self, game: Game, accepted_variants: &[String]) -> Option<&RegistryEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.variant.is_some() && entry.game == Some(game))
            .filter(|entry| {
                accepted_variants.is_empty()
                    || entry
                        .variant
                        .as_deref()
                        .is_some_and(|id| accepted_variants.iter().any(|accepted| accepted == id))
            })
            .min_by_key(|entry| preference(entry))
    }

    /// Loads a registry from `path`.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] when the file cannot be read, is not valid
    /// JSON, or declares an unsupported schema version.
    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        let text = fs::read_to_string(path).map_err(RegistryError::Io)?;
        let registry: Self =
            serde_json::from_str(&text).map_err(|error| RegistryError::Parse(error.to_string()))?;
        if registry.schema_version != SCHEMA_VERSION {
            return Err(RegistryError::UnsupportedSchema(registry.schema_version));
        }
        Ok(registry)
    }

    /// Writes a registry to `path` atomically (temp file + rename).
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError::Io`] when the file cannot be written.
    pub fn save(&self, path: &Path) -> Result<(), RegistryError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|error| RegistryError::Parse(error.to_string()))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(RegistryError::Io)?;
        }

        let file_name = path.file_name().map_or_else(
            || "registry.json".to_owned(),
            |name| name.to_string_lossy().into_owned(),
        );
        let temp = path.with_file_name(format!("{file_name}.{}.tmp", std::process::id()));

        let write = || -> io::Result<()> {
            let mut file = File::create(&temp)?;
            file.write_all(json.as_bytes())?;
            file.write_all(b"\n")?;
            file.sync_all()
        };
        if let Err(error) = write() {
            let _ = fs::remove_file(&temp);
            return Err(RegistryError::Io(error));
        }
        fs::rename(&temp, path).map_err(RegistryError::Io)
    }
}

/// Ranking for [`Registry::best_for`]: lower is preferred. Classic copies (not
/// KEX, and not under a `rerelease/` directory) sort first.
fn preference(entry: &RegistryEntry) -> (u8, u8) {
    let kex = u8::from(
        entry
            .variant
            .as_deref()
            .is_some_and(|id| id.contains("-kex")),
    );
    let rerelease = u8::from(entry.path.components().any(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case("rerelease")
    }));
    (kex, rerelease)
}

/// Why a registry operation failed.
#[derive(Debug)]
pub enum RegistryError {
    /// Reading or writing the state file failed.
    Io(io::Error),
    /// The state file is not valid JSON.
    Parse(String),
    /// The state file declares an unsupported schema version.
    UnsupportedSchema(u32),
    /// A variant id was not found in the table.
    UnknownVariant(String),
    /// No entry is registered for the given path.
    NotRegistered,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::Io(error) => write!(f, "I/O error: {error}"),
            RegistryError::Parse(message) => write!(f, "invalid registry JSON: {message}"),
            RegistryError::UnsupportedSchema(version) => write!(
                f,
                "registry schema_version {version} is not supported (expected {SCHEMA_VERSION})"
            ),
            RegistryError::UnknownVariant(id) => write!(f, "unknown IWAD variant `{id}`"),
            RegistryError::NotRegistered => f.write_str("no registered IWAD for that path"),
        }
    }
}

impl std::error::Error for RegistryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RegistryError::Io(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::iwads::identify::sha256_file;

    fn entry(path: &Path, variant: &str, game: Game) -> RegistryEntry {
        let metadata = fs::metadata(path).expect("metadata");
        RegistryEntry {
            path: path.to_path_buf(),
            variant: Some(variant.to_owned()),
            candidates: Vec::new(),
            game: Some(game),
            sha256: sha256_file(path).expect("hash"),
            size: metadata.len(),
            mtime: FileTime::from_metadata(&metadata).expect("mtime"),
            source: Source::UserFolder,
        }
    }

    #[test]
    fn store_sources_keep_stable_names_and_labels() {
        for (source, name, label) in [
            (Source::Steam, "steam", "Steam"),
            (Source::UserFolder, "userfolder", "User folder"),
            (Source::Manual, "manual", "Manual"),
            (Source::GogHeroic, "gog_heroic", "GOG via Heroic"),
            (
                Source::GogMinigalaxy,
                "gog_minigalaxy",
                "GOG via minigalaxy",
            ),
            (Source::GogOffline, "gog_offline", "GOG offline installer"),
            (Source::StandardFolder, "standard_folder", "Standard folder"),
        ] {
            let json = serde_json::to_string(&source).expect("serialize");
            assert_eq!(json, format!("\"{name}\""), "wire name for {source:?}");
            assert_eq!(serde_json::from_str::<Source>(&json).unwrap(), source);
            assert_eq!(source.label(), label);
        }
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = TempDir::new().expect("tempdir");
        let wad = dir.path().join("doom2.wad");
        fs::write(&wad, b"content").expect("write");

        let mut registry = Registry::new();
        registry.insert(entry(&wad, "doom2-1.9", Game::Doom2));
        let state = dir.path().join("state/registry.json");
        registry.save(&state).expect("save");

        let loaded = Registry::load(&state).expect("load");
        assert_eq!(loaded, registry);
    }

    #[test]
    fn load_rejects_an_unsupported_schema() {
        let dir = TempDir::new().expect("tempdir");
        let state = dir.path().join("registry.json");
        fs::write(&state, r#"{"schema_version":99,"entries":[]}"#).expect("write");
        let error = Registry::load(&state).expect_err("schema");
        assert!(matches!(error, RegistryError::UnsupportedSchema(99)));
    }

    #[test]
    fn load_rejects_invalid_json() {
        let dir = TempDir::new().expect("tempdir");
        let state = dir.path().join("registry.json");
        fs::write(&state, "not json").expect("write");
        assert!(matches!(
            Registry::load(&state),
            Err(RegistryError::Parse(_))
        ));
    }

    #[test]
    fn refresh_rehashes_only_changed_files() {
        let dir = TempDir::new().expect("tempdir");
        let stable = dir.path().join("stable.wad");
        let changing = dir.path().join("changing.wad");
        fs::write(&stable, b"stable contents").expect("write");
        fs::write(&changing, b"original contents").expect("write");

        let mut registry = Registry::new();
        registry.insert(entry(&stable, "doom2-1.9", Game::Doom2));
        registry.insert(entry(&changing, "doom2-1.9", Game::Doom2));

        // Nothing changed: no re-hash.
        let summary = registry.refresh(&IwadTable::embedded());
        assert_eq!(
            summary,
            RefreshSummary {
                checked: 2,
                rehashed: 0,
                changed: 0,
                removed: 0
            }
        );

        // Replace one file's contents: exactly one re-hash, one change. The
        // fixture is not a real WAD, so re-identification drops it.
        fs::write(&changing, b"completely different and longer contents").expect("rewrite");
        let summary = registry.refresh(&IwadTable::embedded());
        assert_eq!(summary.rehashed, 1);
        assert_eq!(summary.checked, 2);
        assert_eq!(summary.changed, 1);
        assert_eq!(
            summary.removed, 1,
            "the replaced file is no longer a known IWAD"
        );
        assert_eq!(registry.get(&stable).expect("stable").size, 15);
    }

    #[test]
    fn refresh_drops_missing_files() {
        let dir = TempDir::new().expect("tempdir");
        let wad = dir.path().join("gone.wad");
        fs::write(&wad, b"x").expect("write");

        let mut registry = Registry::new();
        registry.insert(entry(&wad, "doom2-1.9", Game::Doom2));
        fs::remove_file(&wad).expect("remove");

        let summary = registry.refresh(&IwadTable::embedded());
        assert_eq!(summary.removed, 1);
        assert!(registry.entries().is_empty());
    }

    #[test]
    fn best_for_prefers_classic_over_rerelease() {
        let dir = TempDir::new().expect("tempdir");
        let base = dir.path().join("base/doom2.wad");
        let rerelease = dir.path().join("rerelease/doom2.wad");
        fs::create_dir_all(base.parent().expect("parent")).expect("mkdir");
        fs::create_dir_all(rerelease.parent().expect("parent")).expect("mkdir");
        fs::write(&base, b"classic").expect("write");
        fs::write(&rerelease, b"kex").expect("write");

        let mut registry = Registry::new();
        // Insert the KEX copy first to prove ranking, not order, decides.
        registry.insert(entry(&rerelease, "doom2-1.9-kex-2024-10-01", Game::Doom2));
        registry.insert(entry(&base, "doom2-1.9", Game::Doom2));

        let best = registry.best_for(Game::Doom2, &[]).expect("a match");
        assert_eq!(best.variant.as_deref(), Some("doom2-1.9"));

        // A recipe that accepts only the KEX variant gets it.
        let accepted = vec!["doom2-1.9-kex-2024-10-01".to_owned()];
        let best = registry
            .best_for(Game::Doom2, &accepted)
            .expect("kex match");
        assert_eq!(best.variant.as_deref(), Some("doom2-1.9-kex-2024-10-01"));

        // No classic variant, no preference: the KEX copy still wins.
        let best = registry.best_for(Game::Doom2, &["nonexistent".to_owned()]);
        assert!(best.is_none());
    }

    #[test]
    fn choose_resolves_an_ambiguous_entry() {
        let table = IwadTable::embedded();
        let dir = TempDir::new().expect("tempdir");
        let wad = dir.path().join("maybe.wad");
        fs::write(&wad, b"x").expect("write");

        let mut registry = Registry::new();
        let identification = Identification {
            sha256: sha256_file(&wad).expect("hash"),
            size: 1,
            outcome: Outcome::Ambiguous {
                family: crate::iwads::identify::Family::Commercial,
                candidates: vec!["doom2-1.9".to_owned(), "tnt-1.9".to_owned()],
            },
        };
        assert!(registry.record(&wad, &identification, Source::Steam));

        registry
            .choose(&wad, "tnt-1.9", &table)
            .expect("choose succeeds");
        let stored = registry.get(&wad).expect("stored");
        assert_eq!(stored.variant.as_deref(), Some("tnt-1.9"));
        assert_eq!(stored.game, Some(Game::Tnt));
        assert!(stored.candidates.is_empty());

        assert!(matches!(
            registry.choose(&wad, "does-not-exist", &table),
            Err(RegistryError::UnknownVariant(_))
        ));
    }

    #[test]
    fn unknown_identifications_are_not_recorded() {
        let dir = TempDir::new().expect("tempdir");
        let wad = dir.path().join("mod.wad");
        fs::write(&wad, b"x").expect("write");

        let mut registry = Registry::new();
        let identification = Identification {
            sha256: sha256_file(&wad).expect("hash"),
            size: 1,
            outcome: Outcome::Unknown,
        };
        assert!(!registry.record(&wad, &identification, Source::Manual));
        assert!(registry.entries().is_empty());
    }
}

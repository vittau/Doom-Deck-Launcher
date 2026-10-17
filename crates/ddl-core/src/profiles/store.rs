//! Loading, saving and locating profiles (spec 009, R4).
//!
//! Each profile is one JSON document under `<config>/profiles/<id>.json`
//! (OVERVIEW §10), written atomically with an explicit schema version. Its
//! runtime directories live under `<data>/profiles/<id>/` (decision D-011):
//! `save`, `data` and `shots` map to DSDA's `-save`, `-data` and `-shotdir`,
//! and `run` is the working directory DDL starts the engine in.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::profile::{Profile, ProfileError, SCHEMA_VERSION};

/// Directory appended to DDL's config dir.
pub const CONFIG_DIR: &str = "profiles";
/// Directory appended to DDL's data dir.
pub const DATA_DIR: &str = "profiles";

/// Upper bound on a profile read.
const MAX_PROFILE_BYTES: u64 = 4 * 1024 * 1024;

/// The runtime directories one profile owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileDirs {
    /// `<data>/profiles/<id>`.
    pub root: PathBuf,
    /// DSDA `-save`.
    pub save: PathBuf,
    /// DSDA `-data`.
    pub data: PathBuf,
    /// DSDA `-shotdir`.
    pub shots: PathBuf,
    /// The working directory the engine starts in.
    pub run: PathBuf,
}

impl ProfileDirs {
    /// The runtime directories for `id` under `data_dir`.
    #[must_use]
    pub fn new(data_dir: &Path, id: &str) -> Self {
        let root = data_dir.join(DATA_DIR).join(id);
        Self {
            save: root.join("save"),
            data: root.join("data"),
            shots: root.join("shots"),
            run: root.join("run"),
            root,
        }
    }

    /// Every directory a launch needs, in creation order.
    #[must_use]
    pub fn all(&self) -> [&Path; 4] {
        [&self.save, &self.data, &self.shots, &self.run]
    }
}

/// Reads and writes profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileStore {
    config_dir: PathBuf,
    data_dir: PathBuf,
}

impl ProfileStore {
    /// Creates a store reading from `config_dir` and writing runtime dirs under
    /// `data_dir`.
    #[must_use]
    pub fn new(config_dir: impl Into<PathBuf>, data_dir: impl Into<PathBuf>) -> Self {
        Self {
            config_dir: config_dir.into(),
            data_dir: data_dir.into(),
        }
    }

    /// `<config>/profiles`.
    #[must_use]
    pub fn root(&self) -> PathBuf {
        self.config_dir.join(CONFIG_DIR)
    }

    /// `<data>/profiles/<id>` and the directories it owns.
    #[must_use]
    pub fn dirs(&self, id: &str) -> ProfileDirs {
        ProfileDirs::new(&self.data_dir, id)
    }

    /// `<config>/profiles/<id>.json`.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::UnsafeId`] when `id` is not a safe file name.
    pub fn path(&self, id: &str) -> Result<PathBuf, ProfileError> {
        if !safe_id(id) {
            return Err(ProfileError::UnsafeId(id.to_owned()));
        }
        Ok(self.root().join(format!("{id}.json")))
    }

    /// Loads the profile with `id`.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError`] when the file cannot be read or is malformed.
    pub fn load(&self, id: &str) -> Result<Profile, ProfileError> {
        let path = self.path(id)?;
        let file = fs::File::open(&path)?;
        let mut text = String::new();
        file.take(MAX_PROFILE_BYTES + 1).read_to_string(&mut text)?;
        if text.len() as u64 > MAX_PROFILE_BYTES {
            return Err(ProfileError::TooLarge(MAX_PROFILE_BYTES));
        }
        let profile: Profile =
            serde_json::from_str(&text).map_err(|error| ProfileError::Parse(error.to_string()))?;
        if profile.id != id {
            return Err(ProfileError::UnsafeId(profile.id));
        }
        if profile.schema_version != SCHEMA_VERSION {
            return Err(ProfileError::UnsupportedSchema(profile.schema_version));
        }
        Ok(profile)
    }

    /// Loads the profile with `id`, treating a missing file as `None`.
    ///
    /// # Errors
    ///
    /// As [`ProfileStore::load`], except a missing file is not an error.
    pub fn load_optional(&self, id: &str) -> Result<Option<Profile>, ProfileError> {
        match self.load(id) {
            Ok(profile) => Ok(Some(profile)),
            Err(ProfileError::Io(error)) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Writes `profile` atomically and creates its runtime directories.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError`] when the id is unsafe or a write fails.
    pub fn save(&self, profile: &Profile) -> Result<(), ProfileError> {
        let path = self.path(&profile.id)?;
        for dir in self.dirs(&profile.id).all() {
            fs::create_dir_all(dir)?;
        }
        let mut json = serde_json::to_string_pretty(profile)
            .map_err(|error| ProfileError::Parse(error.to_string()))?;
        json.push('\n');
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = temporary_path(&path);
        let write = || -> io::Result<()> {
            let mut file = fs::File::create(&temp)?;
            file.write_all(json.as_bytes())?;
            file.sync_all()
        };
        if let Err(error) = write() {
            let _ = fs::remove_file(&temp);
            return Err(ProfileError::Io(error));
        }
        fs::rename(&temp, &path)?;
        Ok(())
    }

    /// Every readable profile, in file-name order. Unreadable files are
    /// reported and skipped rather than failing the whole listing.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::Io`] when the directory cannot be read.
    pub fn list(&self) -> Result<Vec<Profile>, ProfileError> {
        let root = self.root();
        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(ProfileError::Io(error)),
        };
        let mut names: Vec<String> = Vec::new();
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if let Some(stem) = name.strip_suffix(".json") {
                names.push(stem.to_owned());
            }
        }
        names.sort();
        let mut profiles = Vec::with_capacity(names.len());
        for name in names {
            match self.load(&name) {
                Ok(profile) => profiles.push(profile),
                Err(error) => tracing::warn!(profile = name, error = %error, "skipping profile"),
            }
        }
        Ok(profiles)
    }

    /// Deletes a profile and its runtime directories, refusing the last profile
    /// for a recipe. The document is removed first: a failed document removal
    /// must never destroy saves. A later directory failure leaves only orphan data.
    ///
    /// # Errors
    /// Returns an error for the last profile or a failed filesystem operation.
    pub fn delete_with_data(&self, id: &str) -> Result<(), ProfileError> {
        let profile = self.load(id)?;
        if self
            .list()?
            .iter()
            .filter(|p| p.recipe_id == profile.recipe_id)
            .count()
            <= 1
        {
            return Err(ProfileError::Parse("cannot delete the last profile".into()));
        }
        self.delete(id)?;
        match fs::remove_dir_all(self.dirs(id).root) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Deletes the profile document for `id`, reporting whether it existed.
    /// Runtime directories are left in place so no save files are destroyed.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError`] when the id is unsafe or deletion fails.
    pub fn delete(&self, id: &str) -> Result<bool, ProfileError> {
        let path = self.path(id)?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(ProfileError::Io(error)),
        }
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "profile.json".to_owned());
    path.with_file_name(format!(".{name}.tmp.{}.{unique}", std::process::id()))
}

fn safe_id(id: &str) -> bool {
    !id.is_empty()
        && id != "."
        && id != ".."
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn save_load_list_and_delete_round_trip() {
        let config = TempDir::new().unwrap();
        let data = TempDir::new().unwrap();
        let store = ProfileStore::new(config.path(), data.path());

        let mut profile = Profile::new("p1", "First", "sigil-test");
        profile.set_component("compat", true);
        profile.overrides.insert("allow_freelook".to_owned(), 1);
        profile.touch(1234);
        store.save(&profile).expect("save");

        assert_eq!(store.load("p1").expect("load"), profile);
        assert_eq!(store.list().expect("list"), vec![profile.clone()]);
        assert_eq!(store.load_optional("missing").unwrap(), None);

        // The runtime directories exist once a profile is saved (D-011).
        let dirs = store.dirs("p1");
        for dir in dirs.all() {
            assert!(dir.is_dir(), "{dir:?} must exist");
        }

        assert!(store.delete("p1").expect("delete"));
        assert!(!store.delete("p1").expect("already gone"));
        assert!(store.list().expect("list").is_empty());
    }

    #[test]
    fn destructive_delete_guards_last_and_removes_only_its_data() {
        let temp = TempDir::new().unwrap();
        let store = ProfileStore::new(temp.path().join("config"), temp.path().join("data"));
        store.save(&Profile::new("p1", "Default", "map")).unwrap();
        fs::write(store.dirs("p1").save.join("save.dsg"), b"save").unwrap();
        assert!(store.delete_with_data("p1").is_err());
        assert!(store.dirs("p1").save.join("save.dsg").exists());
        store.save(&Profile::new("p2", "Casual", "map")).unwrap();
        store.delete_with_data("p1").unwrap();
        assert!(!store.path("p1").unwrap().exists());
        assert!(!store.dirs("p1").root.exists());
        assert!(store.dirs("p2").root.exists());
    }

    #[test]
    fn document_failure_preserves_data_and_directory_failure_leaves_no_document() {
        let temp = TempDir::new().unwrap();
        let store = ProfileStore::new(temp.path().join("config"), temp.path().join("data"));
        store.save(&Profile::new("p1", "Default", "map")).unwrap();
        store.save(&Profile::new("p2", "Casual", "map")).unwrap();
        // Missing document stops before touching data.
        store.delete("p1").unwrap();
        assert!(store.delete_with_data("p1").is_err());
        assert!(store.dirs("p1").root.exists());
        store.save(&Profile::new("p1", "Default", "map")).unwrap();
        // A file where a directory should be reliably fails even as root.
        fs::remove_dir_all(store.dirs("p1").root).unwrap();
        fs::write(store.dirs("p1").root, b"unremovable as directory").unwrap();
        assert!(store.delete_with_data("p1").is_err());
        assert!(!store.path("p1").unwrap().exists());
        assert!(store.load("p2").is_ok());
    }

    #[test]
    fn unsafe_ids_and_bad_schema_are_rejected() {
        let config = TempDir::new().unwrap();
        let data = TempDir::new().unwrap();
        let store = ProfileStore::new(config.path(), data.path());
        assert!(matches!(
            store.path("../escape"),
            Err(ProfileError::UnsafeId(_))
        ));

        fs::create_dir_all(store.root()).unwrap();
        fs::write(store.root().join("bad.json"), "not json").unwrap();
        assert!(matches!(store.load("bad"), Err(ProfileError::Parse(_))));

        let json = serde_json::to_string(&Profile::new("bad", "Bad", "sigil-test"))
            .unwrap()
            .replace("\"schema_version\":1", "\"schema_version\":99");
        fs::write(store.root().join("bad.json"), json).unwrap();
        assert!(matches!(
            store.load("bad"),
            Err(ProfileError::UnsupportedSchema(99))
        ));
    }
}

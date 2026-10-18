//! Loading, saving and locating custom setups (spec 026, R2/R6).
//!
//! Each setup is one JSON document under `<data>/custom/<id>.json`, written
//! atomically with an explicit schema version. Its runtime directories are the
//! selected profile's (`<data>/profiles/<profile id>/`), exactly as a catalog
//! mapset's are (decision D-011); a custom setup therefore reuses spec 018's
//! profile isolation rather than duplicating it.
//!
//! Deleting a setup removes DDL's own record — and, when the player asks, the
//! setup's profiles and their saves. The player's files, referenced in place
//! elsewhere, are never touched (spec 026, R6).

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::setup::{CustomSetup, SCHEMA_VERSION, SetupError, safe_id, slug};

/// Directory appended to DDL's data dir.
pub const DIR: &str = "custom";

/// Upper bound on a setup read, so a corrupt file cannot exhaust memory.
const MAX_SETUP_BYTES: u64 = 4 * 1024 * 1024;

/// Reads and writes custom setups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupStore {
    data_dir: PathBuf,
}

impl SetupStore {
    /// Creates a store rooted at DDL's data dir.
    #[must_use]
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    /// `<data>/custom`.
    #[must_use]
    pub fn root(&self) -> PathBuf {
        self.data_dir.join(DIR)
    }

    /// `<data>/custom/<id>.json`.
    ///
    /// # Errors
    ///
    /// Returns [`SetupError::UnsafeId`] when `id` is not a safe file name.
    pub fn path(&self, id: &str) -> Result<PathBuf, SetupError> {
        if !safe_id(id) {
            return Err(SetupError::UnsafeId(id.to_owned()));
        }
        Ok(self.root().join(format!("{id}.json")))
    }

    /// Whether a setup document exists for `id`.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.path(id).is_ok_and(|path| path.is_file())
    }

    /// A free id derived from `name`, suffixed `-2`, `-3`, … on collision.
    #[must_use]
    pub fn free_id(&self, name: &str) -> String {
        let base = slug(name);
        if !self.contains(&base) {
            return base;
        }
        let mut suffix = 2_u32;
        loop {
            let candidate = format!("{base}-{suffix}");
            if !self.contains(&candidate) {
                return candidate;
            }
            suffix += 1;
        }
    }

    /// Loads the setup with `id`.
    ///
    /// # Errors
    ///
    /// Returns a [`SetupError`] when the file cannot be read or is malformed.
    pub fn load(&self, id: &str) -> Result<CustomSetup, SetupError> {
        let path = self.path(id)?;
        let file = fs::File::open(&path)?;
        let mut text = String::new();
        file.take(MAX_SETUP_BYTES + 1).read_to_string(&mut text)?;
        if text.len() as u64 > MAX_SETUP_BYTES {
            return Err(SetupError::TooLarge(MAX_SETUP_BYTES));
        }
        let setup: CustomSetup =
            serde_json::from_str(&text).map_err(|error| SetupError::Parse(error.to_string()))?;
        if setup.id != id {
            return Err(SetupError::UnsafeId(setup.id));
        }
        if setup.schema_version != SCHEMA_VERSION {
            return Err(SetupError::UnsupportedSchema(setup.schema_version));
        }
        Ok(setup)
    }

    /// Loads the setup with `id`, treating a missing file as `None`.
    ///
    /// # Errors
    ///
    /// As [`SetupStore::load`], except a missing file is not an error.
    pub fn load_optional(&self, id: &str) -> Result<Option<CustomSetup>, SetupError> {
        match self.load(id) {
            Ok(setup) => Ok(Some(setup)),
            Err(SetupError::Io(error)) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Writes `setup` atomically, with DDL-only permissions.
    ///
    /// # Errors
    ///
    /// Returns a [`SetupError`] when the id is unsafe, the document is invalid
    /// or a write fails.
    pub fn save(&self, setup: &CustomSetup) -> Result<(), SetupError> {
        setup.validate()?;
        let path = self.path(&setup.id)?;
        let mut json = serde_json::to_string_pretty(setup)
            .map_err(|error| SetupError::Parse(error.to_string()))?;
        json.push('\n');
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = temporary_path(&path);
        let write = || -> io::Result<()> {
            let mut file = fs::File::create(&temp)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(fs::Permissions::from_mode(0o600))?;
            }
            file.write_all(json.as_bytes())?;
            file.sync_all()
        };
        if let Err(error) = write() {
            let _ = fs::remove_file(&temp);
            return Err(SetupError::Io(error));
        }
        fs::rename(&temp, &path)?;
        Ok(())
    }

    /// Every readable setup, in file-name order. Unreadable files are reported
    /// and skipped rather than failing the whole listing.
    ///
    /// # Errors
    ///
    /// Returns [`SetupError::Io`] when the directory cannot be read.
    pub fn list(&self) -> Result<Vec<CustomSetup>, SetupError> {
        let root = self.root();
        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(SetupError::Io(error)),
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
        let mut setups = Vec::with_capacity(names.len());
        for name in names {
            match self.load(&name) {
                Ok(setup) => setups.push(setup),
                Err(error) => tracing::warn!(setup = name, error = %error, "skipping setup"),
            }
        }
        Ok(setups)
    }

    /// Deletes DDL's record for `id`, reporting whether it existed. The player's
    /// files are never touched; profiles and saves are the caller's concern
    /// (spec 026, R6).
    ///
    /// # Errors
    ///
    /// Returns a [`SetupError`] when the id is unsafe or deletion fails.
    pub fn delete(&self, id: &str) -> Result<bool, SetupError> {
        let path = self.path(id)?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(SetupError::Io(error)),
        }
    }
}

/// True for a directory that holds at least one DSDA save slot (`.dsg`), used
/// to warn that changing a mapset's extras can strand an existing save
/// (spec 026, R5).
#[must_use]
pub fn has_saves(save_dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(save_dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry
            .path()
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("dsg"))
    })
}

fn temporary_path(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "setup.json".to_owned());
    path.with_file_name(format!(".{name}.tmp.{}.{unique}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::local::kind::{Kind, UserFile};
    use crate::local::setup::{Compatibility, CustomIwad, Group};

    use tempfile::TempDir;

    fn user_file(name: &str, kind: Kind) -> UserFile {
        UserFile {
            path: PathBuf::from(format!("/games/{name}")),
            size: 4,
            sha256: "b".repeat(64),
            kind,
        }
    }

    fn setup(id: &str) -> CustomSetup {
        let mut setup = CustomSetup::new(
            id,
            "My Setup",
            CustomIwad::Identified {
                variant: "doom2-1.9".to_owned(),
            },
            Compatibility::Level(crate::engines::dsda::Complevel::MBF21),
            1_700_000_000,
        );
        setup.add(user_file("maps.wad", Kind::Wad)).unwrap();
        setup
    }

    #[test]
    fn save_load_list_and_delete_round_trip() {
        let temp = TempDir::new().unwrap();
        let store = SetupStore::new(temp.path().join("data"));
        let setup = setup("my-setup");
        store.save(&setup).expect("save");
        assert_eq!(store.load("my-setup").unwrap(), setup);
        assert_eq!(store.list().unwrap(), vec![setup.clone()]);
        assert!(store.contains("my-setup"));
        assert_eq!(store.load_optional("missing").unwrap(), None);

        assert!(store.delete("my-setup").unwrap());
        assert!(!store.delete("my-setup").unwrap());
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn deleting_never_touches_a_referenced_user_file() {
        let temp = TempDir::new().unwrap();
        let store = SetupStore::new(temp.path().join("data"));
        let user_file = temp.path().join("player.wad");
        fs::write(&user_file, b"player data").unwrap();
        let mut setup = setup("my-setup");
        setup.files[0].path.clone_from(&user_file);
        store.save(&setup).unwrap();
        store.delete("my-setup").unwrap();
        assert_eq!(fs::read(&user_file).unwrap(), b"player data");
    }

    #[test]
    fn free_id_suffixes_collisions_and_rejects_unsafe_ids() {
        let temp = TempDir::new().unwrap();
        let store = SetupStore::new(temp.path());
        assert_eq!(store.free_id("My Setup"), "my-setup");
        store.save(&setup("my-setup")).unwrap();
        assert_eq!(store.free_id("My Setup"), "my-setup-2");
        store.save(&setup("my-setup-2")).unwrap();
        assert_eq!(store.free_id("My Setup"), "my-setup-3");
        assert!(matches!(
            store.path("../escape"),
            Err(SetupError::UnsafeId(_))
        ));
    }

    #[test]
    fn bad_schema_and_malformed_documents_are_rejected() {
        let temp = TempDir::new().unwrap();
        let store = SetupStore::new(temp.path());
        fs::create_dir_all(store.root()).unwrap();
        fs::write(store.root().join("bad.json"), "not json").unwrap();
        assert!(matches!(store.load("bad"), Err(SetupError::Parse(_))));

        let json = serde_json::to_string(&setup("bad"))
            .unwrap()
            .replace("\"schema_version\":1", "\"schema_version\":99");
        fs::write(store.root().join("bad.json"), json).unwrap();
        assert!(matches!(
            store.load("bad"),
            Err(SetupError::UnsupportedSchema(99))
        ));
    }

    #[test]
    fn has_saves_looks_for_dsg_slots() {
        let temp = TempDir::new().unwrap();
        assert!(!has_saves(temp.path()));
        fs::write(temp.path().join("slot0.dsg"), b"save").unwrap();
        assert!(has_saves(temp.path()));
    }

    #[test]
    fn group_parses_its_two_spellings() {
        assert_eq!(Group::parse("files"), Some(Group::Files));
        assert_eq!(Group::parse("patches"), Some(Group::Patches));
        assert_eq!(Group::parse("wads"), None);
        assert_eq!(Group::Files.as_str(), "files");
    }
}

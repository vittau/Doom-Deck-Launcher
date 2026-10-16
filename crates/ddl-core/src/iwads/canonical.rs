//! Canonical IWAD symlinks (spec 007, R6; decision D-017).
//!
//! DSDA-Doom picks the game *mission* from the IWAD file name, so DDL never
//! launches a user's file directly. [`CanonicalLinks::canonical_path`] creates
//! or refreshes `<data>/iwads/<variant-id>/<canonical filename>` as a symlink
//! to the user's file and returns the link path for the launch plan.
//!
//! Before returning, the target is verified against the registry: its size must
//! match, and when its modification time changed it is re-hashed and the
//! SHA-256 must still match. A drifted or missing target is an error, never a
//! silent launch.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::identify::sha256_file;
use super::registry::{FileTime, RegistryEntry};
use super::table::IwadTable;

/// Manages the `<data>/iwads/` symlink tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalLinks {
    root: PathBuf,
}

impl CanonicalLinks {
    /// Points the manager at `<data_dir>/iwads`.
    #[must_use]
    pub fn new(data_dir: &Path) -> Self {
        Self {
            root: data_dir.join("iwads"),
        }
    }

    /// The managed root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Creates or refreshes the canonical link for `entry`.
    ///
    /// Returns the link path (`<root>/<variant-id>/<canonical filename>`).
    ///
    /// # Errors
    ///
    /// Returns [`LinkError`] when the entry is unresolved, the variant is not in
    /// `table`, the target file is missing or has changed since it was recorded,
    /// an unrelated file occupies the link path, or the link cannot be created.
    pub fn canonical_path(
        &self,
        entry: &RegistryEntry,
        table: &IwadTable,
    ) -> Result<PathBuf, LinkError> {
        let variant_id = entry.variant.as_deref().ok_or(LinkError::Unresolved)?;
        let variant = table
            .get(variant_id)
            .ok_or_else(|| LinkError::UnknownVariant(variant_id.to_owned()))?;

        let resolved = verify_target(entry)?;

        let directory = self.root.join(variant_id);
        fs::create_dir_all(&directory).map_err(LinkError::Io)?;
        let link = directory.join(&variant.filename);
        replace_symlink(&link, &resolved)?;
        Ok(link)
    }
}

/// Confirms the target still matches the registry, returning its resolved path.
fn verify_target(entry: &RegistryEntry) -> Result<PathBuf, LinkError> {
    let metadata = fs::metadata(&entry.path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            LinkError::TargetMissing
        } else {
            LinkError::Io(error)
        }
    })?;
    if !metadata.is_file() {
        return Err(LinkError::TargetMissing);
    }
    if metadata.len() != entry.size {
        return Err(LinkError::TargetChanged);
    }
    if FileTime::from_metadata(&metadata) != Some(entry.mtime) {
        let sha256 = sha256_file(&entry.path).map_err(LinkError::Io)?;
        if !sha256.eq_ignore_ascii_case(&entry.sha256) {
            return Err(LinkError::TargetChanged);
        }
    }
    fs::canonicalize(&entry.path).map_err(LinkError::Io)
}

/// Atomically points `link` at `target`.
fn replace_symlink(link: &Path, target: &Path) -> Result<(), LinkError> {
    if let Ok(existing) = fs::read_link(link) {
        if existing == target {
            return Ok(());
        }
    } else if fs::symlink_metadata(link).is_ok() {
        // Something that is not a symlink occupies the DDL-managed path; refuse
        // to destroy it rather than deleting a possible user file.
        return Err(LinkError::Obstructed(link.to_path_buf()));
    }

    let file_name = link.file_name().map_or_else(
        || "iwad.wad".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let temp = link.with_file_name(format!(".{file_name}.{}.tmp", std::process::id()));
    let _ = fs::remove_file(&temp);

    #[cfg(unix)]
    std::os::unix::fs::symlink(target, &temp).map_err(LinkError::Io)?;
    #[cfg(not(unix))]
    {
        let _ = (target, &temp);
        return Err(LinkError::UnsupportedPlatform);
    }

    fs::rename(&temp, link).map_err(|error| {
        let _ = fs::remove_file(&temp);
        LinkError::Io(error)
    })
}

/// Why canonical linking failed.
#[derive(Debug)]
pub enum LinkError {
    /// The entry has no resolved variant (the user has not chosen yet).
    Unresolved,
    /// The variant id is not in the table.
    UnknownVariant(String),
    /// The target file no longer exists.
    TargetMissing,
    /// The target's size or SHA-256 no longer matches the record.
    TargetChanged,
    /// A non-symlink file occupies the link path.
    Obstructed(PathBuf),
    /// Creating the symlink failed.
    Io(io::Error),
    /// Symlinks are not available on this platform.
    UnsupportedPlatform,
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkError::Unresolved => {
                f.write_str("the IWAD is ambiguous and has not been resolved by the user")
            }
            LinkError::UnknownVariant(id) => write!(f, "unknown IWAD variant `{id}`"),
            LinkError::TargetMissing => f.write_str("the IWAD file no longer exists"),
            LinkError::TargetChanged => {
                f.write_str("the IWAD file changed since DDL recorded it; re-identify it")
            }
            LinkError::Obstructed(path) => write!(
                f,
                "{} already exists and is not a DDL symlink",
                path.display()
            ),
            LinkError::Io(error) => write!(f, "cannot create the canonical link: {error}"),
            LinkError::UnsupportedPlatform => {
                f.write_str("canonical IWAD links need a symlink-capable platform")
            }
        }
    }
}

impl std::error::Error for LinkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LinkError::Io(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use tempfile::TempDir;

    use super::*;
    use crate::iwads::registry::Source;
    use crate::iwads::table::{Game, IwadVariant};

    fn test_table(sha256: &str, size: u64) -> IwadTable {
        IwadTable {
            schema_version: 1,
            variants: vec![IwadVariant {
                id: "test-variant".to_owned(),
                game: Game::Doom2,
                title: "Test".to_owned(),
                filename: "doom2.wad".to_owned(),
                size,
                sha256: Some(sha256.to_owned()),
                md5: None,
                sha1: None,
                crc32: None,
                default_complevel: 2,
                playable: true,
                source: "https://example.invalid".to_owned(),
                hash_sources: BTreeMap::new(),
            }],
        }
    }

    #[test]
    fn canonical_path_creates_a_link_with_the_canonical_name() {
        let data = TempDir::new().expect("data");
        let store = TempDir::new().expect("store");
        let target = store.path().join("mine.wad");
        fs::write(&target, b"iwad bytes").expect("write");
        let metadata = fs::metadata(&target).expect("metadata");
        let entry = RegistryEntry {
            path: target.clone(),
            variant: Some("test-variant".to_owned()),
            candidates: Vec::new(),
            game: Some(Game::Doom2),
            sha256: sha256_file(&target).expect("hash"),
            size: metadata.len(),
            mtime: FileTime::from_metadata(&metadata).expect("mtime"),
            source: Source::Manual,
        };
        let table = test_table(&entry.sha256, entry.size);

        let links = CanonicalLinks::new(data.path());
        let link = links.canonical_path(&entry, &table).expect("link");
        assert_eq!(link, data.path().join("iwads/test-variant/doom2.wad"));
        assert_eq!(
            fs::read_link(&link).expect("read_link"),
            fs::canonicalize(&target).expect("canon")
        );
        assert!(fs::metadata(&link).expect("stat").is_file());

        // Refreshing is idempotent.
        let again = links.canonical_path(&entry, &table).expect("refresh");
        assert_eq!(again, link);
    }

    #[test]
    fn target_changes_are_detected() {
        let data = TempDir::new().expect("data");
        let store = TempDir::new().expect("store");
        let target = store.path().join("mine.wad");
        fs::write(&target, b"iwad bytes").expect("write");
        let metadata = fs::metadata(&target).expect("metadata");
        let entry = RegistryEntry {
            path: target.clone(),
            variant: Some("test-variant".to_owned()),
            candidates: Vec::new(),
            game: Some(Game::Doom2),
            sha256: sha256_file(&target).expect("hash"),
            size: metadata.len(),
            mtime: FileTime::from_metadata(&metadata).expect("mtime"),
            source: Source::Manual,
        };
        let table = test_table(&entry.sha256, entry.size);
        let links = CanonicalLinks::new(data.path());

        // Different size is caught immediately.
        fs::write(&target, b"different and longer bytes").expect("rewrite");
        assert!(matches!(
            links.canonical_path(&entry, &table),
            Err(LinkError::TargetChanged)
        ));
    }

    #[test]
    fn missing_target_is_reported() {
        let data = TempDir::new().expect("data");
        let store = TempDir::new().expect("store");
        let target = store.path().join("gone.wad");
        fs::write(&target, b"x").expect("write");
        let metadata = fs::metadata(&target).expect("metadata");
        let entry = RegistryEntry {
            path: target.clone(),
            variant: Some("test-variant".to_owned()),
            candidates: Vec::new(),
            game: Some(Game::Doom2),
            sha256: sha256_file(&target).expect("hash"),
            size: metadata.len(),
            mtime: FileTime::from_metadata(&metadata).expect("mtime"),
            source: Source::Manual,
        };
        let table = test_table(&entry.sha256, entry.size);
        fs::remove_file(&target).expect("remove");

        let links = CanonicalLinks::new(data.path());
        assert!(matches!(
            links.canonical_path(&entry, &table),
            Err(LinkError::TargetMissing)
        ));
    }

    #[test]
    fn unresolved_entries_are_refused() {
        let data = TempDir::new().expect("data");
        let store = TempDir::new().expect("store");
        let target = store.path().join("mine.wad");
        fs::write(&target, b"x").expect("write");
        let metadata = fs::metadata(&target).expect("metadata");
        let entry = RegistryEntry {
            path: target.clone(),
            variant: None,
            candidates: vec!["doom2-1.9".to_owned()],
            game: None,
            sha256: sha256_file(&target).expect("hash"),
            size: metadata.len(),
            mtime: FileTime::from_metadata(&metadata).expect("mtime"),
            source: Source::Manual,
        };
        let table = test_table(&entry.sha256, entry.size);

        let links = CanonicalLinks::new(data.path());
        assert!(matches!(
            links.canonical_path(&entry, &table),
            Err(LinkError::Unresolved)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn an_unrelated_file_at_the_link_path_is_not_destroyed() {
        let data = TempDir::new().expect("data");
        let store = TempDir::new().expect("store");
        let target = store.path().join("mine.wad");
        fs::write(&target, b"x").expect("write");
        let metadata = fs::metadata(&target).expect("metadata");
        let entry = RegistryEntry {
            path: target.clone(),
            variant: Some("test-variant".to_owned()),
            candidates: Vec::new(),
            game: Some(Game::Doom2),
            sha256: sha256_file(&target).expect("hash"),
            size: metadata.len(),
            mtime: FileTime::from_metadata(&metadata).expect("mtime"),
            source: Source::Manual,
        };
        let table = test_table(&entry.sha256, entry.size);

        let links = CanonicalLinks::new(data.path());
        let link = links.canonical_path(&entry, &table).expect("link");
        // Replace the symlink with a regular file; DDL must refuse.
        fs::remove_file(&link).expect("remove symlink");
        fs::write(&link, b"user data").expect("write file");

        assert!(matches!(
            links.canonical_path(&entry, &table),
            Err(LinkError::Obstructed(_))
        ));
    }
}

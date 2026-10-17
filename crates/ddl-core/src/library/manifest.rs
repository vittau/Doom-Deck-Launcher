//! The immutable install record (spec 009, R2).
//!
//! A mapset install is described by one small, versioned JSON document written
//! atomically inside `<data>/library/<recipe id>/<release>/install.json`. It
//! pins the recipe's content identity, every package DDL fetched and every
//! member (downloaded or user-imported) DDL placed, with its size, SHA-256 and
//! absolute path. Resolution reads this record rather than the live catalog, so
//! a later catalog update can never silently change what an installed mapset
//! launches (OVERVIEW §7).

use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

/// Schema version written into `install.json`.
pub const SCHEMA_VERSION: u32 = 1;

/// File name of the install record inside an install directory.
pub const MANIFEST_FILE: &str = "install.json";

/// Upper bound on a manifest read, so a corrupt file cannot exhaust memory.
const MAX_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;

/// One package DDL fetched for an install.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledPackage {
    /// Package id from the recipe.
    pub id: String,
    /// SHA-256 of the downloaded archive (spec 006, R3).
    pub sha256: String,
    /// Size of the downloaded archive in bytes.
    pub size: u64,
}

/// One member placed by an install.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledMember {
    /// Source package id, for a downloaded member.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Source member id, for a downloaded member.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<String>,
    /// Owning component id, for a user-supplied import (D-014).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// Absolute path of the installed file.
    pub path: PathBuf,
    /// Size in bytes at install time.
    pub size: u64,
    /// SHA-256 at install time.
    pub sha256: String,
    /// Whether the member is a permission/credit text kept unmodified.
    #[serde(default)]
    pub keep: bool,
}

/// The versioned install record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallManifest {
    /// Schema version for forward compatibility (decision D-006).
    pub schema_version: u32,
    /// Recipe id this install belongs to.
    pub recipe_id: String,
    /// Recipe release label.
    pub release: String,
    /// Content identity of the recipe at install time.
    pub recipe_sha256: String,
    /// Unix seconds when the install was first created.
    pub installed_at: u64,
    /// Packages fetched.
    #[serde(default)]
    pub packages: Vec<InstalledPackage>,
    /// Members placed, in install order.
    #[serde(default)]
    pub members: Vec<InstalledMember>,
}

impl InstallManifest {
    /// The installed member for a downloaded `(package, member)` pair.
    #[must_use]
    pub fn find_downloaded(&self, package: &str, member: &str) -> Option<&InstalledMember> {
        self.members.iter().find(|held| {
            held.package.as_deref() == Some(package) && held.member.as_deref() == Some(member)
        })
    }

    /// The user-supplied import recorded for `component`.
    #[must_use]
    pub fn find_imported(&self, component: &str) -> Option<&InstalledMember> {
        self.members
            .iter()
            .find(|held| held.package.is_none() && held.component.as_deref() == Some(component))
    }

    /// The size on disk of every file this install placed (spec 022, R2).
    #[must_use]
    pub fn installed_bytes(&self) -> u64 {
        self.members.iter().map(|member| member.size).sum()
    }

    /// Whether a user-imported component (D-014) is part of this install; an
    /// uninstall removes it too, so the confirm dialog says so (spec 022, R2).
    #[must_use]
    pub fn has_imports(&self) -> bool {
        self.members.iter().any(|member| member.package.is_none())
    }

    /// Loads a manifest from `path`.
    ///
    /// # Errors
    ///
    /// Returns [`ManifestError`] when the file cannot be read, is too large, is
    /// not valid JSON, or declares an unsupported schema version.
    pub fn load(path: &Path) -> Result<Self, ManifestError> {
        let file = fs::File::open(path).map_err(ManifestError::Io)?;
        let mut text = String::new();
        file.take(MAX_MANIFEST_BYTES + 1)
            .read_to_string(&mut text)
            .map_err(ManifestError::Io)?;
        if text.len() as u64 > MAX_MANIFEST_BYTES {
            return Err(ManifestError::TooLarge(MAX_MANIFEST_BYTES));
        }
        let manifest: Self =
            serde_json::from_str(&text).map_err(|error| ManifestError::Parse(error.to_string()))?;
        if manifest.schema_version != SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedSchema(manifest.schema_version));
        }
        Ok(manifest)
    }

    /// Loads a manifest, treating a missing file as `None`.
    ///
    /// # Errors
    ///
    /// As [`InstallManifest::load`], except a missing file is not an error.
    pub fn load_optional(path: &Path) -> Result<Option<Self>, ManifestError> {
        match Self::load(path) {
            Ok(manifest) => Ok(Some(manifest)),
            Err(ManifestError::Io(error)) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Writes the manifest to `path` atomically (temp file + rename).
    ///
    /// # Errors
    ///
    /// Returns [`ManifestError::Io`] when the file cannot be written.
    pub fn save(&self, path: &Path) -> Result<(), ManifestError> {
        let mut json = serde_json::to_string_pretty(self)
            .map_err(|error| ManifestError::Parse(error.to_string()))?;
        json.push('\n');
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(ManifestError::Io)?;
        }
        let temp = temporary_path(path);
        let write = || -> io::Result<()> {
            let mut file = fs::File::create(&temp)?;
            file.write_all(json.as_bytes())?;
            file.sync_all()
        };
        if let Err(error) = write() {
            let _ = fs::remove_file(&temp);
            return Err(ManifestError::Io(error));
        }
        fs::rename(&temp, path).map_err(ManifestError::Io)
    }
}

pub(super) fn temporary_path(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| MANIFEST_FILE.to_owned());
    path.with_file_name(format!(".{name}.tmp.{}.{unique}", std::process::id()))
}

/// Why an install record could not be read or written.
#[derive(Debug)]
pub enum ManifestError {
    /// A filesystem operation failed.
    Io(io::Error),
    /// The manifest is not valid JSON.
    Parse(String),
    /// The manifest exceeds the read bound.
    TooLarge(u64),
    /// The manifest declares an unsupported schema version.
    UnsupportedSchema(u32),
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "install manifest I/O error: {error}"),
            Self::Parse(message) => write!(f, "invalid install manifest: {message}"),
            Self::TooLarge(limit) => {
                write!(f, "install manifest exceeds the {limit} byte read limit")
            }
            Self::UnsupportedSchema(version) => write!(
                f,
                "install manifest schema_version {version} is not supported (expected {SCHEMA_VERSION})"
            ),
        }
    }
}

impl std::error::Error for ManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn manifest() -> InstallManifest {
        InstallManifest {
            schema_version: SCHEMA_VERSION,
            recipe_id: "sigil".to_owned(),
            release: "1.23".to_owned(),
            recipe_sha256: "a".repeat(64),
            installed_at: 1_700_000_000,
            packages: vec![InstalledPackage {
                id: "free".to_owned(),
                sha256: "b".repeat(64),
                size: 42,
            }],
            members: vec![InstalledMember {
                package: Some("free".to_owned()),
                member: Some("maps".to_owned()),
                component: None,
                path: PathBuf::from("/tmp/sigil/free/SIGIL_V1_23.wad"),
                size: 10,
                sha256: "c".repeat(64),
                keep: false,
            }],
        }
    }

    #[test]
    fn round_trips_and_looks_up_members() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("install.json");
        manifest().save(&path).expect("save");
        let loaded = InstallManifest::load(&path).expect("load");
        assert_eq!(loaded, manifest());
        assert!(loaded.find_downloaded("free", "maps").is_some());
        assert!(loaded.find_downloaded("free", "other").is_none());
        assert!(loaded.find_imported("soundtrack").is_none());
    }

    #[test]
    fn installed_size_and_imports_feed_the_uninstall_dialog() {
        let mut manifest = manifest();
        assert_eq!(manifest.installed_bytes(), 10);
        assert!(!manifest.has_imports());
        manifest.members.push(InstalledMember {
            package: None,
            member: None,
            component: Some("soundtrack".to_owned()),
            path: PathBuf::from("/tmp/sigil/imports/soundtrack/import.wad"),
            size: 32,
            sha256: "d".repeat(64),
            keep: false,
        });
        assert_eq!(manifest.installed_bytes(), 42);
        assert!(manifest.has_imports());
    }

    #[test]
    fn missing_file_is_none_and_bad_schema_is_rejected() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("install.json");
        assert_eq!(InstallManifest::load_optional(&path).unwrap(), None);

        fs::write(&path, r#"{"schema_version":99}"#).unwrap();
        assert!(matches!(
            InstallManifest::load(&path),
            Err(ManifestError::Parse(_))
        ));

        // A full document with an unsupported version is rejected as such.
        let json = serde_json::to_string(&manifest())
            .unwrap()
            .replace("\"schema_version\":1", "\"schema_version\":99");
        fs::write(&path, json).unwrap();
        assert!(matches!(
            InstallManifest::load(&path),
            Err(ManifestError::UnsupportedSchema(99))
        ));

        fs::write(&path, "not json").unwrap();
        assert!(matches!(
            InstallManifest::load(&path),
            Err(ManifestError::Parse(_))
        ));
    }
}

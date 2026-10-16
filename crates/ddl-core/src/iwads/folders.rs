//! Bounded user-folder scanning (spec 007, R4).
//!
//! [`scan_folder`] identifies every `*.wad` directly inside a user-chosen
//! folder plus exactly one level of subfolders, never recursively. It never
//! follows a symlink out of the chosen folder: symlinked directories are not
//! traversed, and a symlinked file whose target resolves outside the folder is
//! refused and recorded in [`FolderScan::skipped`].
//!
//! DDL never scans `$HOME` wholesale; the UI hands this function a folder the
//! user chose (OVERVIEW §6).

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// How many levels below the chosen folder are scanned (`0` = the folder, `1` =
/// its immediate subfolders).
pub const MAX_SUBFOLDER_DEPTH: usize = 1;

/// The outcome of scanning a folder.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FolderScan {
    /// `*.wad` files found, sorted, de-duplicated.
    pub wads: Vec<PathBuf>,
    /// Symlinks refused because their target escaped the chosen folder.
    pub skipped: Vec<PathBuf>,
}

/// Why a folder could not be scanned.
#[derive(Debug)]
pub enum FolderError {
    /// The folder does not exist.
    RootMissing,
    /// The path exists but is not a directory.
    RootNotDirectory,
    /// The folder could not be read.
    Io(io::Error),
}

impl fmt::Display for FolderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FolderError::RootMissing => f.write_str("the chosen folder does not exist"),
            FolderError::RootNotDirectory => f.write_str("the chosen path is not a folder"),
            FolderError::Io(error) => write!(f, "cannot read the chosen folder: {error}"),
        }
    }
}

impl std::error::Error for FolderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FolderError::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// Scans `root` for `*.wad` files (case-insensitive), one level deep.
///
/// # Errors
///
/// Returns [`FolderError`] when `root` is missing, is not a directory, or
/// cannot be read.
pub fn scan_folder(root: &Path) -> Result<FolderScan, FolderError> {
    let canonical = fs::canonicalize(root).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            FolderError::RootMissing
        } else {
            FolderError::Io(error)
        }
    })?;
    if !fs::metadata(&canonical).is_ok_and(|metadata| metadata.is_dir()) {
        return Err(FolderError::RootNotDirectory);
    }

    let mut scan = FolderScan::default();
    scan_level(&canonical, &canonical, 0, true, &mut scan)?;
    scan.wads.sort();
    scan.wads.dedup();
    scan.skipped.sort();
    scan.skipped.dedup();
    Ok(scan)
}

/// Scans one level, recursing into plain subdirectories up to the depth limit.
fn scan_level(
    directory: &Path,
    root: &Path,
    depth: usize,
    strict: bool,
    scan: &mut FolderScan,
) -> Result<(), FolderError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        // Only the chosen root failing to read is fatal; a subfolder that
        // cannot be read is tolerated, like a missing Steam install dir.
        Err(error) if strict => return Err(FolderError::Io(error)),
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_symlink() {
            handle_symlink(&path, root, scan);
        } else if file_type.is_dir() {
            if depth < MAX_SUBFOLDER_DEPTH {
                scan_level(&path, root, depth + 1, false, scan)?;
            }
        } else if file_type.is_file() && is_wad(&path) {
            scan.wads.push(path);
        }
    }
    Ok(())
}

/// Includes a symlinked file only when it stays inside `root`.
fn handle_symlink(path: &Path, root: &Path, scan: &mut FolderScan) {
    let Ok(resolved) = fs::canonicalize(path) else {
        scan.skipped.push(path.to_path_buf());
        return;
    };
    if !resolved.starts_with(root) {
        scan.skipped.push(path.to_path_buf());
        return;
    }
    if resolved.is_file() && is_wad(&resolved) {
        scan.wads.push(path.to_path_buf());
    }
    // Symlinked directories inside the root are never traversed, to avoid loops.
}

/// `true` when the path ends in `.wad`, whatever the case.
fn is_wad(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.to_string_lossy().eq_ignore_ascii_case("wad"))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// The scan canonicalises its root, so tests compare against the
    /// canonical path too (on macOS `/var` is a symlink to `/private/var`).
    fn canonical(dir: &tempfile::TempDir) -> PathBuf {
        fs::canonicalize(dir.path()).expect("canonicalize")
    }

    fn touch(path: &Path) {
        fs::write(path, b"fake wad").expect("write fixture");
    }

    #[test]
    fn scans_the_folder_and_one_level_of_subfolders() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = canonical(&dir);
        touch(&root.join("a.wad"));
        fs::create_dir_all(root.join("sub")).expect("mkdir");
        touch(&root.join("sub/b.wad"));
        fs::create_dir_all(root.join("sub/deep")).expect("mkdir");
        touch(&root.join("sub/deep/c.wad"));

        let scan = scan_folder(&root).expect("scan");
        assert_eq!(scan.wads, vec![root.join("a.wad"), root.join("sub/b.wad")]);
    }

    #[test]
    fn matches_the_wad_extension_case_insensitively() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = canonical(&dir);
        touch(&root.join("UPPER.WAD"));
        touch(&root.join("notawad.txt"));

        let scan = scan_folder(&root).expect("scan");
        assert_eq!(scan.wads, vec![root.join("UPPER.WAD")]);
    }

    #[test]
    fn missing_folder_is_an_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let error = scan_folder(&dir.path().join("nope")).expect_err("missing");
        assert!(matches!(error, FolderError::RootMissing));
    }

    #[test]
    fn a_file_is_not_a_folder() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let file = dir.path().join("file.wad");
        touch(&file);
        let error = scan_folder(&file).expect_err("not a dir");
        assert!(matches!(error, FolderError::RootNotDirectory));
    }

    #[cfg(unix)]
    mod unix {
        use std::os::unix::fs::symlink;

        use super::*;

        #[test]
        fn refuses_symlinks_that_escape_the_chosen_folder() {
            let root_dir = tempfile::TempDir::new().expect("root");
            let outside_dir = tempfile::TempDir::new().expect("outside");
            let outside_wad = outside_dir.path().join("secret.wad");
            touch(&outside_wad);

            let root = canonical(&root_dir);
            touch(&root.join("real.wad"));
            symlink(&outside_wad, root.join("escape.wad")).expect("symlink file");
            symlink(outside_dir.path(), root.join("escape-dir")).expect("symlink dir");

            let scan = scan_folder(&root).expect("scan");
            assert_eq!(scan.wads, vec![root.join("real.wad")]);
            assert!(scan.skipped.contains(&root.join("escape.wad")));
            assert!(scan.skipped.contains(&root.join("escape-dir")));
        }

        #[test]
        fn allows_symlinks_that_stay_inside_the_chosen_folder() {
            let dir = tempfile::TempDir::new().expect("tempdir");
            let root = canonical(&dir);
            touch(&root.join("real.wad"));
            symlink(root.join("real.wad"), root.join("link.wad")).expect("symlink");

            let scan = scan_folder(&root).expect("scan");
            assert!(scan.wads.contains(&root.join("link.wad")));
            assert!(scan.skipped.is_empty());
        }

        #[test]
        fn symlinked_directories_are_not_traversed() {
            let root_dir = tempfile::TempDir::new().expect("root");
            let outside_dir = tempfile::TempDir::new().expect("outside");
            touch(&outside_dir.path().join("inside.wad"));

            let root = canonical(&root_dir);
            symlink(outside_dir.path(), root.join("link-dir")).expect("symlink dir");

            let scan = scan_folder(&root).expect("scan");
            assert!(scan.wads.is_empty());
        }
    }
}

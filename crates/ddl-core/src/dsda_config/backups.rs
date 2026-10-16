//! Backups of the active configuration before a migration or reset
//! (spec 005, R5).
//!
//! Each backup is a byte-for-byte copy of the active file named
//! `<timestamp>-<template_version>.cfg`. Only the newest [`MAX_BACKUPS`] are
//! kept; older ones are removed.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// How many backups to retain.
pub const MAX_BACKUPS: usize = 10;

/// Directory (inside DDL's DSDA config directory) holding the backups.
pub const BACKUP_DIR: &str = "backups";

/// The file name for a backup taken at `timestamp` while applying
/// `template_version`.
#[must_use]
pub fn backup_name(timestamp: u64, template_version: &str) -> String {
    format!("{timestamp}-{template_version}.cfg")
}

/// Copies `active` into `dir` and rotates old backups. Returns the new path.
///
/// # Errors
/// Propagates any filesystem error from creating the directory, copying the
/// file or removing a rotated backup.
pub fn create(
    dir: &Path,
    active: &Path,
    timestamp: u64,
    template_version: &str,
) -> io::Result<PathBuf> {
    fs::create_dir_all(dir)?;
    let destination = dir.join(backup_name(timestamp, template_version));
    fs::copy(active, &destination)?;
    rotate(dir)?;
    Ok(destination)
}

/// Removes the oldest backups so at most [`MAX_BACKUPS`] remain. Files whose
/// names do not start with a timestamp are left alone.
///
/// # Errors
/// Propagates filesystem errors from reading the directory or removing a file.
pub fn rotate(dir: &Path) -> io::Result<()> {
    let mut backups: Vec<(u64, String)> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(timestamp) = leading_timestamp(&name) {
            backups.push((timestamp, name));
        }
    }
    backups.sort();
    let excess = backups.len().saturating_sub(MAX_BACKUPS);
    for (_, name) in backups.into_iter().take(excess) {
        fs::remove_file(dir.join(name))?;
    }
    Ok(())
}

fn leading_timestamp(name: &str) -> Option<u64> {
    name.split('-').next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_backup_is_a_byte_for_byte_copy() {
        let temp = tempfile::tempdir().expect("temp dir");
        let active = temp.path().join("active.cfg");
        fs::write(&active, "use_mouse 1\n").expect("write active");

        let backup = create(&temp.path().join(BACKUP_DIR), &active, 42, "1").expect("backup");

        assert_eq!(backup.file_name().unwrap(), "42-1.cfg");
        assert_eq!(fs::read_to_string(&backup).expect("read"), "use_mouse 1\n");
        assert_eq!(
            fs::read_to_string(&active).expect("read active"),
            "use_mouse 1\n",
            "the active file is not modified while being copied"
        );
    }

    #[test]
    fn rotation_keeps_only_the_newest_ten() {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = temp.path();
        for timestamp in 1..=13 {
            fs::write(dir.join(backup_name(timestamp, "1")), "x").expect("write");
        }
        // An unrelated file must not be rotated away.
        fs::write(dir.join("notes.txt"), "keep me").expect("write");

        rotate(dir).expect("rotate");

        let mut remaining: Vec<String> = fs::read_dir(dir)
            .expect("read dir")
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        remaining.sort();
        assert_eq!(remaining.len(), MAX_BACKUPS + 1, "10 backups + notes.txt");
        assert!(!remaining.contains(&backup_name(1, "1")));
        assert!(!remaining.contains(&backup_name(3, "1")));
        assert!(remaining.contains(&backup_name(4, "1")), "newest ten kept");
        assert!(remaining.contains(&"notes.txt".to_owned()));
    }
}

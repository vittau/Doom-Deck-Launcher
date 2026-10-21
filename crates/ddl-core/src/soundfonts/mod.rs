//! Optional reviewed MIDI fonts (spec 067). No font bytes ship with DDL.
//! Installed generations are immutable; an atomic pointer publishes all three
//! reviewed files together. HTML notices are retained as inert data only.
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256 as Digest256};

use crate::downloads::{
    CancellationToken, DownloadRequest, ExtractLimits, Extractor, Fetcher, HostAllowlist,
    InstallProgress, InstallStage, MemberRequest, Sha256,
};
use crate::platform::paths::Paths;

pub const SOURCE: &str =
    "https://www.dropbox.com/s/2rnpya9ecb9m4jh/arachno-soundfont-10-sf2.zip?dl=1";
pub const TERMS: &str = "https://www.arachnosoft.com/main/soundfont.php?documentation=";
pub const AUTHOR: &str = "Maxime Abbey";
pub const DOWNLOAD_BYTES: u64 = 143356181;
pub const FONT_FILE: &str = "Arachno SoundFont - Version 1.0.sf2";
/// Exact reviewed entry count and declared expansion of the author archive
/// (`docs/research/midi-soundfonts.md`). Both are enforced before extraction.
const ARCHIVE_ENTRIES: usize = 454;
const EXPANDED_BYTES: u64 = 173559660;
/// The reviewed archive's largest member: the SF2 itself.
const LARGEST_MEMBER_BYTES: u64 = 155405818;
/// The reviewed archive peaks at about 32:1. 64 leaves room for a future
/// reviewed repack without trusting an unbounded expansion ratio.
const MAX_COMPRESSION_RATIO: u64 = 64;
const ZIP_HASH: &str = "5c0c573abe6c5e1a3ccceba5d52a4229db24283620c9a3eb1f403018e2aa22df";
const FONT_HASH: &str = "9a57fb3b6714e69dda12390e351b087e81fc3b1eca15c6b4bbe172799f4cf3cd";
const MEMBERS: [(&str, u64, &str); 3] = [
    (FONT_FILE, 155405818, FONT_HASH),
    (
        "Read Me.txt",
        3388,
        "d7dab992c37c68e61decea3406d7b232b5382a5c4b7fd89af9ec09117e046873",
    ),
    (
        "Documentation/Arachno SoundFont Documentation - Version 1.2 - English.html",
        441987,
        "ed23308db66faa6dfee13019e0978dd1700d9a08a3d59672801613819426ca92",
    ),
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Choice {
    #[default]
    Default,
    Arachno,
}

/// None means no explicit Audio choice: preserve any custom engine settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preferences {
    pub schema_version: u32,
    pub choice: Option<Choice>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            schema_version: 1,
            choice: None,
        }
    }
}

#[derive(Debug)]
pub struct Error(String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}
impl From<crate::downloads::Error> for Error {
    fn from(e: crate::downloads::Error) -> Self {
        Self(e.to_string())
    }
}
fn invalid(message: &str) -> Error {
    Error(message.into())
}

#[derive(Debug, Clone)]
pub struct Soundfonts {
    root: PathBuf,
    preferences: PathBuf,
    cache: PathBuf,
}
#[derive(Debug, Serialize, Deserialize)]
struct Installed {
    schema_version: u32,
    generation: String,
}

/// Safe effective next-launch choice. A missing/corrupt managed font never blocks play.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effective {
    pub choice: Option<Choice>,
    pub path: Option<PathBuf>,
    pub warning: Option<String>,
}

impl Soundfonts {
    pub fn new(paths: &Paths) -> Self {
        Self {
            root: paths.data.join("soundfonts"),
            preferences: paths.config.join("audio.json"),
            cache: paths.cache.clone(),
        }
    }
    pub fn preferences(&self) -> Result<Preferences, Error> {
        if !self.preferences.exists() {
            return Ok(Preferences::default());
        }
        let prefs: Preferences = read_json(&self.preferences)?;
        if prefs.schema_version != 1 {
            return Err(invalid("Unsupported audio settings version"));
        }
        Ok(prefs)
    }
    pub fn select(&self, choice: Choice) -> Result<(), Error> {
        self.select_members(choice, &reviewed_members())
    }
    fn select_members(&self, choice: Choice, members: &[MemberRequest]) -> Result<(), Error> {
        if choice == Choice::Arachno {
            self.verify_members(members)?;
        }
        atomic_json(
            &self.preferences,
            &Preferences {
                schema_version: 1,
                choice: Some(choice),
            },
        )
    }
    pub fn effective(&self) -> Effective {
        self.effective_with(&reviewed_members())
    }
    fn effective_with(&self, members: &[MemberRequest]) -> Effective {
        let preferences = match self.preferences() {
            Ok(preferences) => preferences,
            // A damaged preferences file must never block a launch or make DDL
            // overwrite a configuration it cannot interpret. Leave the engine
            // alone and say so in Settings.
            Err(_) => {
                return Effective {
                    choice: None,
                    path: None,
                    warning: Some(
                        "Audio settings could not be read. Using the engine's current MIDI configuration."
                            .into(),
                    ),
                };
            }
        };
        let choice = preferences.choice;
        if choice != Some(Choice::Arachno) {
            return Effective {
                choice,
                path: None,
                warning: None,
            };
        }
        match self.verify_members(members) {
            Ok(path) => Effective { choice, path: Some(path), warning: None },
            Err(_) => Effective { choice: Some(Choice::Default), path: None,
                warning: Some("Arachno is missing or damaged. Using Default; reinstall Arachno in Audio settings.".into()) },
        }
    }
    /// Snapshot-only verification cache. Launches use `verified_font` to hash
    /// again on the services worker. Metadata changes invalidate this cache.
    pub fn cached_font(&self) -> Result<PathBuf, Error> {
        use std::collections::HashMap;
        use std::sync::{Mutex, OnceLock};
        static CACHE: OnceLock<Mutex<HashMap<PathBuf, (Vec<FileStamp>, PathBuf)>>> =
            OnceLock::new();
        let stamp = self.file_stamps()?;
        let cache = CACHE.get_or_init(Mutex::default);
        if let Some((previous, path)) = cache
            .lock()
            .map_err(|_| invalid("Soundfont cache unavailable"))?
            .get(&self.root)
            && *previous == stamp
        {
            return Ok(path.clone());
        }
        let path = self.verified_font()?;
        if self.file_stamps()? != stamp {
            return Err(invalid("Soundfont changed during verification"));
        }
        cache
            .lock()
            .map_err(|_| invalid("Soundfont cache unavailable"))?
            .insert(self.root.clone(), (stamp, path.clone()));
        Ok(path)
    }
    fn file_stamps(&self) -> Result<Vec<FileStamp>, Error> {
        self.file_stamps_for(&reviewed_members())
    }
    fn file_stamps_for(&self, members: &[MemberRequest]) -> Result<Vec<FileStamp>, Error> {
        let pointer = self.root.join("installed.json");
        let installed: Installed = read_json(&pointer)?;
        if installed.schema_version != 1
            || !installed.generation.starts_with("generation-")
            || !installed
                .generation
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        {
            return Err(invalid("Invalid soundfont installation record"));
        }
        let mut paths = vec![pointer];
        paths.extend(
            members
                .iter()
                .map(|m| self.root.join(&installed.generation).join(&m.path)),
        );
        paths
            .iter()
            .map(|path| {
                reject_symlinks(&self.root, path)?;
                let metadata = fs::metadata(path)?;
                Ok(FileStamp {
                    path: path.clone(),
                    len: metadata.len(),
                    modified: metadata.modified()?,
                    #[cfg(unix)]
                    identity: {
                        use std::os::unix::fs::MetadataExt;
                        (metadata.ino(), metadata.ctime(), metadata.ctime_nsec())
                    },
                })
            })
            .collect()
    }
    pub fn verified_font(&self) -> Result<PathBuf, Error> {
        self.verify_members(&reviewed_members())
    }
    fn verify_members(&self, members: &[MemberRequest]) -> Result<PathBuf, Error> {
        let installed: Installed = read_json(&self.root.join("installed.json"))?;
        if installed.schema_version != 1
            || !installed.generation.starts_with("generation-")
            || !installed
                .generation
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        {
            return Err(invalid("Invalid soundfont installation record"));
        }
        let dir = self.root.join(installed.generation);
        reject_symlinks(&self.root, &dir)?;
        for member in members {
            let path = dir.join(&member.path);
            reject_symlinks(&dir, &path)?;
            verify_file(
                &path,
                member.expected_size.unwrap(),
                member.expected_sha256.unwrap(),
                &CancellationToken::new(),
            )?;
        }
        let font = dir.join(&members[0].path);
        validate_sf2(&font)?;
        let font = fs::canonicalize(font)?;
        config_path(&font)?;
        Ok(font)
    }
    /// Cheap Settings-snapshot check: the pointer and every reviewed file exist
    /// at the reviewed sizes and no path component is a symlink. It does not
    /// hash; `select` and every launch use [`Soundfonts::verified_font`].
    pub fn install_present(&self) -> bool {
        self.install_present_with(&reviewed_members())
    }
    fn install_present_with(&self, members: &[MemberRequest]) -> bool {
        let Ok(stamps) = self.file_stamps_for(members) else {
            return false;
        };
        stamps.len() == members.len() + 1
            && stamps[1..]
                .iter()
                .zip(members)
                .all(|(stamp, member)| member.expected_size.is_none_or(|size| stamp.len == size))
    }
    /// Run on a worker thread. UI owns the single-download guard.
    pub fn install(
        &self,
        cancel: &CancellationToken,
        progress: &mut dyn FnMut(InstallProgress),
    ) -> Result<(), Error> {
        let fetcher = Fetcher::new(&self.cache).with_redirect_exceptions(Vec::<String>::new());
        let request = DownloadRequest::new(
            SOURCE,
            Sha256::parse_hex(ZIP_HASH).unwrap(),
            DOWNLOAD_BYTES,
            HostAllowlist::new(["www.dropbox.com", "dl.dropboxusercontent.com"]),
        )
        .with_size(DOWNLOAD_BYTES);
        let package = fetcher.download(&request, cancel, &mut |p| {
            progress(InstallProgress::transfer(p))
        })?;
        progress(InstallProgress::stage(InstallStage::Verify));
        self.install_archive(
            &package,
            DOWNLOAD_BYTES,
            Sha256::parse_hex(ZIP_HASH).unwrap(),
            &reviewed_members(),
            cancel,
            progress,
        )
    }
    fn install_archive(
        &self,
        package: &Path,
        size: u64,
        hash: Sha256,
        members: &[MemberRequest],
        cancel: &CancellationToken,
        progress: &mut dyn FnMut(InstallProgress),
    ) -> Result<(), Error> {
        verify_file(package, size, hash, cancel)?;
        inspect_archive(package, cancel)?;
        progress(InstallProgress::stage(InstallStage::Extract));
        let extraction = Extractor::new(self.root.join("staging"))
            .with_limits(limits())
            .with_cancellation(cancel.clone())
            .extract(package, members)?;
        validate_sf2(&extraction.staging_dir().join(&members[0].path))?;
        for member in members {
            File::open(extraction.staging_dir().join(&member.path))?.sync_all()?;
        }
        check_cancel(cancel)?;
        // No cancellation within the atomic publication boundary.
        progress(InstallProgress::stage(InstallStage::Commit));
        let generation = unique_name("generation");
        let destination = self.root.join(&generation);
        fs::rename(extraction.staging_dir(), &destination)?;
        // Dropping extraction now only attempts to remove the old staging path.
        atomic_json(
            &self.root.join("installed.json"),
            &Installed {
                schema_version: 1,
                generation,
            },
        )?;
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct FileStamp {
    path: PathBuf,
    len: u64,
    modified: std::time::SystemTime,
    #[cfg(unix)]
    identity: (u64, i64, i64),
}
fn reviewed_members() -> Vec<MemberRequest> {
    MEMBERS
        .iter()
        .map(|(path, size, hash)| {
            MemberRequest::new(*path)
                .with_size(*size)
                .with_sha256(Sha256::parse_hex(hash).unwrap())
        })
        .collect()
}
fn limits() -> ExtractLimits {
    ExtractLimits {
        max_total_uncompressed: EXPANDED_BYTES,
        max_member_uncompressed: LARGEST_MEMBER_BYTES,
        max_member_count: ARCHIVE_ENTRIES,
        max_compression_ratio: MAX_COMPRESSION_RATIO,
        max_nesting_depth: 0,
    }
}
fn check_cancel(cancel: &CancellationToken) -> Result<(), Error> {
    if cancel.is_cancelled() {
        Err(invalid("Download cancelled"))
    } else {
        Ok(())
    }
}
/// Validate every entry, including files we do not install.
fn inspect_archive(path: &Path, cancel: &CancellationToken) -> Result<(), Error> {
    let mut zip = zip::ZipArchive::new(File::open(path)?).map_err(|e| Error(e.to_string()))?;
    if zip.len() > limits().max_member_count || zip.len() != declared_entry_count(path)? {
        return Err(invalid("Too many ZIP entries"));
    }
    let mut names = HashSet::new();
    let mut total = 0u64;
    for index in 0..zip.len() {
        check_cancel(cancel)?;
        let entry = zip.by_index(index).map_err(|e| Error(e.to_string()))?;
        let name = entry.name().map_err(|e| Error(e.to_string()))?;
        if name.is_empty()
            || name.starts_with('/')
            || name.contains(['\\', ':', '\0'])
            || name.split('/').any(|p| p == ".." || p == ".")
            || !names.insert(name.trim_end_matches('/').to_owned())
            || entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000)
        {
            return Err(invalid("Unsafe or duplicate ZIP entry"));
        }
        total = total
            .checked_add(entry.size())
            .ok_or_else(|| invalid("ZIP size overflow"))?;
        if total > limits().max_total_uncompressed
            || entry.size() > limits().max_member_uncompressed
            || entry.size()
                > entry
                    .compressed_size()
                    .max(1)
                    .saturating_mul(limits().max_compression_ratio)
        {
            return Err(invalid("ZIP expansion limit exceeded"));
        }
    }
    Ok(())
}
// The ZIP crate indexes by name and collapses duplicate names. This reviewed
// classic ZIP has no ZIP64/multi-disk records: compare its bounded EOCD count
// with the unique index count so duplicates cannot disappear from the scan.
fn declared_entry_count(path: &Path) -> Result<usize, Error> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    let tail_size = size.min(65535 + 22) as usize;
    file.seek(SeekFrom::End(-(tail_size as i64)))?;
    let mut tail = vec![0; tail_size];
    file.read_exact(&mut tail)?;
    for index in (0..tail.len().saturating_sub(21)).rev() {
        if &tail[index..index + 4] != b"PK\x05\x06" {
            continue;
        }
        let word = |offset| u16::from_le_bytes([tail[index + offset], tail[index + offset + 1]]);
        if index + 22 + usize::from(word(20)) != tail.len() {
            continue;
        }
        let count = word(10);
        if word(4) != 0 || word(6) != 0 || word(8) != count || count == u16::MAX {
            return Err(invalid("Unsupported ZIP directory"));
        }
        return Ok(usize::from(count));
    }
    Err(invalid("Missing ZIP directory"))
}
fn verify_file(
    path: &Path,
    size: u64,
    hash: Sha256,
    cancel: &CancellationToken,
) -> Result<(), Error> {
    let mut file = File::open(path)?;
    if !file.metadata()?.is_file() || file.metadata()?.len() != size {
        return Err(invalid("Soundfont file size mismatch"));
    }
    let mut digest = Digest256::new();
    let mut buffer = [0u8; 65536];
    let mut total = 0;
    loop {
        check_cancel(cancel)?;
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total += read as u64;
        if total > size {
            return Err(invalid("Soundfont file grew during verification"));
        }
        digest.update(&buffer[..read]);
    }
    if total != size || Sha256::finish(digest) != hash {
        return Err(invalid("Soundfont SHA-256 mismatch"));
    }
    Ok(())
}
fn validate_sf2(path: &Path) -> Result<(), Error> {
    let mut file = File::open(path)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;
    if &header[..4] != b"RIFF"
        || &header[8..] != b"sfbk"
        || u64::from(u32::from_le_bytes(header[4..8].try_into().unwrap())) + 8
            != file.metadata()?.len()
    {
        return Err(invalid("Invalid RIFF/sfbk soundfont"));
    }
    Ok(())
}
/// DSDA v0.30.1 strings have no escape syntax. Spaces are supported.
pub fn config_path(path: &Path) -> Result<&str, Error> {
    let text = path
        .to_str()
        .ok_or_else(|| invalid("Soundfont path is not UTF-8"))?;
    if !path.is_absolute() || text.chars().any(|c| c == '"' || c.is_control()) {
        return Err(invalid(
            "Soundfont path cannot be represented in engine settings",
        ));
    }
    Ok(text)
}
fn reject_symlinks(root: &Path, path: &Path) -> Result<(), Error> {
    let mut current = root.to_path_buf();
    if fs::symlink_metadata(&current)?.file_type().is_symlink() {
        return Err(invalid("Soundfont symlink rejected"));
    }
    for component in path
        .strip_prefix(root)
        .map_err(|_| invalid("Soundfont path escapes storage"))?
        .components()
    {
        current.push(component);
        if fs::symlink_metadata(&current)?.file_type().is_symlink() {
            return Err(invalid("Soundfont symlink rejected"));
        }
    }
    Ok(())
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Error> {
    let file = File::open(path)?;
    if file.metadata()?.len() > 16384 {
        return Err(invalid("Soundfont state exceeds size limit"));
    }
    serde_json::from_reader(file.take(16385)).map_err(|e| Error(e.to_string()))
}
fn unique_name(prefix: &str) -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{prefix}-{}-{nanos}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}
fn atomic_json(path: &Path, value: &impl Serialize) -> Result<(), Error> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid("Missing state directory"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(unique_name("pending"));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&serde_json::to_vec_pretty(value).map_err(|e| Error(e.to_string()))?)?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    let _ = fs::remove_file(temporary);
    result
}

#[cfg(test)]
mod tests;

pub mod engine;

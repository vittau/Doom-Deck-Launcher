//! Per-mapset progress read from DSDA-Doom's own `stats.txt` (spec 116, D-045).
//!
//! DSDA-Doom writes `<-data>/dsda_doom_data/<iwad stem>/<pwad stems…>/stats.txt`
//! under the per-profile `-data` directory DDL passes
//! (`docs/research/dsda-stats.md` Q1/Q2). This module owns three read-only steps:
//!
//! 1. [`stats_path`] derives that exact path from a resolved launch's IWAD and
//!    ordered `-file` list, following the engine organizer's rules;
//! 2. [`read`] opens the file inside the profile's data directory, bounds the
//!    read and parses it strictly ([`parse`]);
//! 3. [`Request::read`] applies the engine-version gate and D-045's progress
//!    rule, producing a [`Progress`] that keeps *no stats yet* and *unavailable*
//!    distinct from *zero completed*.
//!
//! Nothing here writes, repairs or deletes an engine file, and no result is
//! cached: the caller reads again when it builds Library or Mapset detail, or
//! after the engine process has exited.

mod parse;

pub use parse::{
    MAX_BYTES, MAX_LINE_BYTES, MAX_MAP_ID_BYTES, MAX_ROWS, MAX_SKILL, ParseError, ROW_TOKENS, Row,
    STATS_VERSION, Stats, parse,
};

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use crate::engines::dsda::ReviewedList;

/// Directory DSDA-Doom's organizer appends to `-data`.
pub const DATA_DIR: &str = "dsda_doom_data";
/// The stats file name inside the organized directory.
pub const STATS_FILE: &str = "stats.txt";
/// The organizer uses the IWAD stem plus at most the first eight PWAD stems.
pub const MAX_PWADS: usize = 8;

/// One map's progress (spec 116, R4/R10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapProgress {
    /// The map lump name.
    pub id: String,
    /// Whether DSDA recorded a non-no-monsters skill (`best_skill > 0`).
    pub completed: bool,
    /// DSDA's recorded exits for the map.
    pub total_exits: u32,
    /// The highest recorded skill, or `None` when DSDA recorded none.
    pub best_skill: Option<u8>,
    /// D-045's UV-Max mark: UV or higher with all kills and secrets recorded.
    pub uv_max: bool,
    /// `floor(100 × best kills ÷ recorded total)`, or `None` when the total is
    /// unknown or zero (spec 116, R11). Never capped.
    pub kills_percent: Option<u64>,
    /// `floor(100 × best secrets ÷ recorded total)`, or `None` when the total is
    /// unknown or zero.
    pub secrets_percent: Option<u64>,
}

/// A readable stats file reduced to D-045's rule for one profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// Rows with `best_skill > 0`, after recipe exclusions.
    pub completed: usize,
    /// Parsed rows minus recipe exclusions present in the file.
    pub total: usize,
    /// Per-map rows in file order, after exclusions.
    pub maps: Vec<MapProgress>,
}

impl Stats {
    /// Applies D-045's progress rule: the denominator is every parsed row except
    /// the recipe's excluded map IDs; a map is completed only when `best_skill`
    /// is above zero. Excluded IDs that the file does not list are ignored.
    #[must_use]
    pub fn progress(&self, exclude_maps: &[String]) -> Recorded {
        let excluded: BTreeSet<&str> = exclude_maps.iter().map(String::as_str).collect();
        let maps: Vec<MapProgress> = self
            .rows
            .iter()
            .filter(|row| !excluded.contains(row.id.as_str()))
            .map(|row| MapProgress {
                id: row.id.clone(),
                completed: row.completed(),
                total_exits: row.total_exits,
                best_skill: (row.best_skill > 0).then_some(row.best_skill),
                uv_max: row.uv_max(),
                kills_percent: row.kills_percent(),
                secrets_percent: row.secrets_percent(),
            })
            .collect();
        Recorded {
            completed: maps.iter().filter(|map| map.completed).count(),
            total: maps.len(),
            maps,
        }
    }
}

/// The read-only progress state of one target for one profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    /// A valid file was read; the rule's counts are in [`Recorded`].
    Recorded(Recorded),
    /// DSDA has not written a stats file for this load yet. Never shown as `0`
    /// completed.
    NoStatsYet,
    /// Unreadable, invalid, unsupported engine or ambiguous layout.
    Unavailable(Unavailable),
}

impl Progress {
    /// Whether this state carries counts.
    #[must_use]
    pub const fn recorded(&self) -> Option<&Recorded> {
        match self {
            Self::Recorded(recorded) => Some(recorded),
            Self::NoStatsYet | Self::Unavailable(_) => None,
        }
    }
}

/// Why progress could not be shown (spec 116, R4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unavailable {
    /// The resolved load is one the research flags as unverified, so DDL will
    /// not guess the organizer path.
    AmbiguousLayout(LayoutError),
    /// The engine version is not audited for the stats format.
    UnsupportedEngine {
        /// The installed version DDL recorded.
        version: String,
    },
    /// No usable engine is installed, so DDL cannot confirm which format wrote
    /// the file (spec 116, R3).
    EngineMissing,
    /// The launch cannot be resolved (missing install, blocked choice), so the
    /// organizer path is unknown.
    Unresolved,
    /// The file is outside the profile's data directory, not a regular file, or
    /// could not be read.
    Read(ReadError),
    /// The file exists but violates the read contract.
    Invalid(ParseError),
}

impl fmt::Display for Unavailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AmbiguousLayout(error) => write!(f, "stats path unavailable: {error}"),
            Self::UnsupportedEngine { version } => {
                write!(f, "DSDA-Doom {version} is not audited for stats import")
            }
            Self::EngineMissing => f.write_str("no usable DSDA-Doom engine is installed"),
            Self::Unresolved => f.write_str("the launch cannot be resolved"),
            Self::Read(error) => write!(f, "stats file unavailable: {error}"),
            Self::Invalid(error) => write!(f, "stats file rejected: {error}"),
        }
    }
}

impl std::error::Error for Unavailable {}

/// Why a launch layout has no trustworthy organizer path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// The IWAD path has no usable UTF-8 file stem.
    IwadStem,
    /// A `-file` entry does not end in `.wad`. DSDA may load `.zip` archives
    /// differently at startup, which the research has not verified.
    UnverifiedFile(PathBuf),
    /// A path stem is not a usable organizer directory name.
    Stem(PathBuf),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IwadStem => f.write_str("the IWAD path has no usable file stem"),
            Self::UnverifiedFile(path) => write!(
                f,
                "`{}` is not a .wad file, so its engine load order cannot be verified",
                path.display()
            ),
            Self::Stem(path) => write!(
                f,
                "`{}` has no usable file stem for the organizer path",
                path.display()
            ),
        }
    }
}

impl std::error::Error for LayoutError {}

/// The launch inputs the organizer path depends on (spec 116, R1).
#[derive(Debug, Clone, Copy)]
pub struct Layout<'a> {
    /// The profile's `-data` directory (`ProfileDirs::data`).
    pub profile_data_dir: &'a Path,
    /// The IWAD path DSDA receives (DDL's canonical link).
    pub iwad: &'a Path,
    /// The ordered `-file` list the launch would pass.
    pub files: &'a [PathBuf],
    /// The ordered `-deh` list. Patches never contribute to the path; the field
    /// exists so the caller passes the whole launch instead of a filtered copy.
    pub patches: &'a [PathBuf],
}

/// DSDA-Doom's organized stats path for a resolved launch.
///
/// `dsda_doom_data`, the lowercase IWAD stem, then the stems of up to the first
/// eight `-file` entries that end in `.wad` (case-insensitively) in load order,
/// then `stats.txt`. Patches never contribute.
///
/// # Errors
///
/// Returns [`LayoutError`] when a stem is unusable or a `-file` entry is not a
/// `.wad`; the caller maps that to *unavailable* rather than guessing.
pub fn stats_path(layout: &Layout<'_>) -> Result<PathBuf, LayoutError> {
    let mut path = layout
        .profile_data_dir
        .join(DATA_DIR)
        .join(stem(layout.iwad).ok_or(LayoutError::IwadStem)?);
    for file in layout.files.iter().take(MAX_PWADS) {
        if file
            .extension()
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("wad"))
        {
            return Err(LayoutError::UnverifiedFile(file.clone()));
        }
        path.push(stem(file).ok_or_else(|| LayoutError::Stem(file.clone()))?);
    }
    // `patches` deliberately does not participate (spec 116, R1).
    let _ = layout.patches;
    Ok(path.join(STATS_FILE))
}

/// The lowercase stem of `path`, when it is a usable directory name.
fn stem(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    if stem.is_empty() || stem == "." || stem == ".." || stem.contains(['/', '\\', '\0']) {
        return None;
    }
    Some(stem.to_ascii_lowercase())
}

/// Why the stats file itself could not be used (spec 116, R2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadError {
    /// The path resolves outside the profile's data directory (a symlink
    /// escape), so DDL refuses it.
    Escape,
    /// The path is not a regular file (a directory, socket or device).
    NotAFile,
    /// The file is larger than [`MAX_BYTES`].
    Oversize {
        /// The accepted limit.
        limit: u64,
    },
    /// Opening, canonicalising or reading failed.
    Unreadable(String),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Escape => f.write_str("the path resolves outside the profile's data directory"),
            Self::NotAFile => f.write_str("the path is not a regular file"),
            Self::Oversize { limit } => write!(f, "the file is larger than {limit} bytes"),
            Self::Unreadable(detail) => write!(f, "{detail}"),
        }
    }
}

impl std::error::Error for ReadError {}

/// Reads `path` as untrusted bytes, contained in `profile_data_dir`.
///
/// `Ok(None)` means the file does not exist yet (DSDA never wrote it, so there
/// are no stats). Every other problem is a [`ReadError`].
///
/// # Errors
///
/// Returns [`ReadError`] for an escape, a non-regular file, an oversize file or
/// an I/O failure.
pub fn read(profile_data_dir: &Path, path: &Path) -> Result<Option<Vec<u8>>, ReadError> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ReadError::Unreadable(error.to_string())),
    };
    if !metadata.is_file() {
        return Err(ReadError::NotAFile);
    }
    // Anchor the open on the resolved target: the symlink chain is followed
    // once, then the canonical path must stay inside the profile's data root.
    let root = fs::canonicalize(profile_data_dir)
        .map_err(|error| ReadError::Unreadable(error.to_string()))?;
    let target =
        fs::canonicalize(path).map_err(|error| ReadError::Unreadable(error.to_string()))?;
    if !target.starts_with(&root) {
        return Err(ReadError::Escape);
    }
    if !fs::metadata(&target)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
    {
        return Err(ReadError::NotAFile);
    }
    if metadata.len() > MAX_BYTES {
        return Err(ReadError::Oversize { limit: MAX_BYTES });
    }
    let file = fs::File::open(&target).map_err(|error| ReadError::Unreadable(error.to_string()))?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ReadError::Unreadable(error.to_string()))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(ReadError::Oversize { limit: MAX_BYTES });
    }
    Ok(Some(bytes))
}

/// Everything one progress read needs (spec 116, R1–R4).
#[derive(Debug, Clone, Copy)]
pub struct Request<'a> {
    /// The profile's `-data` directory (`ProfileDirs::data`).
    pub profile_data_dir: &'a Path,
    /// The IWAD path DSDA receives.
    pub iwad: &'a Path,
    /// The ordered `-file` list.
    pub files: &'a [PathBuf],
    /// The ordered `-deh` list.
    pub patches: &'a [PathBuf],
    /// The installed engine version DDL recorded.
    pub engine_version: &'a str,
    /// The recipe's `progress.exclude_maps`.
    pub exclude_maps: &'a [String],
}

impl Request<'_> {
    /// Reads and evaluates progress for this launch.
    #[must_use]
    pub fn read(&self) -> Progress {
        if !ReviewedList::embedded().audits_stats(self.engine_version) {
            return Progress::Unavailable(Unavailable::UnsupportedEngine {
                version: self.engine_version.to_owned(),
            });
        }
        let layout = Layout {
            profile_data_dir: self.profile_data_dir,
            iwad: self.iwad,
            files: self.files,
            patches: self.patches,
        };
        let path = match stats_path(&layout) {
            Ok(path) => path,
            Err(error) => return Progress::Unavailable(Unavailable::AmbiguousLayout(error)),
        };
        match read(self.profile_data_dir, &path) {
            Ok(None) => Progress::NoStatsYet,
            Ok(Some(bytes)) => match parse(&bytes) {
                Ok(stats) => Progress::Recorded(stats.progress(self.exclude_maps)),
                Err(error) => Progress::Unavailable(Unavailable::Invalid(error)),
            },
            Err(error) => Progress::Unavailable(Unavailable::Read(error)),
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

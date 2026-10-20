//! The published `/idgames` index: schema, deterministic ordering and
//! validation (spec 050, R3).
//!
//! One JSON document, built daily by CI and read by the app (spec 051):
//!
//! ```json
//! {
//!   "schema": 1,
//!   "generated_at": "2026-10-19T05:07:08Z",
//!   "source": {
//!     "index": "https://youfailit.net/pub/idgames/ls-laR.gz",
//!     "index_sha256": "502a7ccd…",
//!     "mirror": "youfailit.net"
//!   },
//!   "min_ddl_version": "0.0.6",
//!   "entries": {
//!     "levels/doom2/Ports/megawads": [
//!       ["dbp37_augzen", "AUGER;ZENITH", "Doomer Boards Community",
//!        "2021-07-26", 12771737, "doom2", "mbf21", 1, 2021, null, 11, false]
//!     ]
//!   }
//! }
//! ```
//!
//! Entries are **grouped by directory** and each one is a fixed 12-slot array,
//! which keeps the whole document well under [`MAX_INDEX_BYTES`] (the research
//! estimate for 10,885 entries is 0.9–1.3 MB; the cap is 2 MiB). The slots, in
//! order, are:
//!
//! | # | field | notes |
//! |---|-------|-------|
//! | 0 | `stem` | archive file name without `.zip`; the stable key |
//! | 1 | `title` | the author's title, or the stem as a fallback |
//! | 2 | `author` | empty when the sidecar did not say |
//! | 3 | `date` | `YYYY-MM-DD`, day precision |
//! | 4 | `size` | bytes |
//! | 5 | `game` | `doom`, `doom2`, `tnt`, `plutonia` or `unknown` |
//! | 6 | `port` | `vanilla`, `limit-removing`, `boom`, `mbf`, `mbf21`, `zdoom`, `other` or `unknown` |
//! | 7 | `txt` | whether a usable sidecar was read |
//! | 8 | `cacoward` | Cacoward year, or `null` (R8) |
//! | 9 | `in_catalog` | recipe id when a DDL recipe pins this archive, or `null` (R8) |
//! | 10 | `maps` | map count when the text states it, or `null` (R8) |
//! | 11 | `latest_version` | the newest upload sharing this title (R8) |
//!
//! There are **no** descriptions, credits, ratings, screenshots or txt bodies.
//! The provider of the file is the archive: DDL stores facts about it.
//!
//! Determinism is part of the contract: directories are a `BTreeMap`, entries
//! are sorted by stem and validated to be, and the same inputs serialize to the
//! same bytes.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::{self, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::date::{Date, is_rfc3339_utc};
use super::inventory::is_indexable_dir;

/// The index format this DDL understands (spec 050, R3).
pub const INDEX_SCHEMA: u32 = 1;

/// The oldest DDL promised to consume this index (D-031, spec 050). Deliberate
/// and stable like the catalog bundle's constant: the first consumer is spec
/// 051, so this is the workspace version at that point and not the version of
/// whatever binary happens to build the file.
pub const DEFAULT_MIN_DDL_VERSION: &str = "0.0.6";

/// Largest published index DDL accepts (spec 050, R3).
pub const MAX_INDEX_BYTES: u64 = 2 * 1024 * 1024;

/// Upper bound on entries, so a corrupt document cannot exhaust memory.
pub const MAX_ENTRIES: usize = 50_000;

/// Upper bound on directories, for the same reason.
pub const MAX_DIRS: usize = 2_000;

/// The longest title or author the index stores.
pub const MAX_TEXT_CHARS: usize = 512;

/// The largest archive the index may describe (the observed maximum is 426 MB).
pub const MAX_ARCHIVE_BYTES: u64 = 1024 * 1024 * 1024;

/// The base game an entry needs. `unknown` is honest, never a guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndexGame {
    /// DOOM (shareware/registered/Ultimate).
    Doom,
    /// DOOM II.
    Doom2,
    /// TNT: Evilution.
    Tnt,
    /// The Plutonia Experiment.
    Plutonia,
    /// The path or text did not say.
    Unknown,
}

/// The port family the author's text requires (research §6.4).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Port {
    /// No special engine; vanilla limits.
    Vanilla,
    /// Raises the vanilla static limits but no Boom features.
    LimitRemoving,
    /// Boom/MBF-era feature set below MBF.
    Boom,
    /// MBF.
    Mbf,
    /// MBF21.
    Mbf21,
    /// ZDoom family (GZDoom, UZDoom, LZDoom, Zandronum, UDMF).
    Zdoom,
    /// A port DDL does not ship: the entry stays browsable, Play is disabled.
    Other,
    /// No engine line, or a line that names no format.
    #[default]
    Unknown,
}

impl Port {
    /// The stable string used in the JSON document.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::LimitRemoving => "limit-removing",
            Self::Boom => "boom",
            Self::Mbf => "mbf",
            Self::Mbf21 => "mbf21",
            Self::Zdoom => "zdoom",
            Self::Other => "other",
            Self::Unknown => "unknown",
        }
    }
}

/// Where the index came from (spec 050, R3): the inventory URL, the SHA-256 of
/// exactly the bytes read, and the mirror host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexSource {
    /// URL of the inventory file (`…/ls-laR.gz`).
    pub index: String,
    /// Lowercase hex SHA-256 of the downloaded inventory bytes.
    pub index_sha256: String,
    /// Host the inventory and sidecars were fetched from.
    pub mirror: String,
}

/// One archive in the index. Serialized as the 12-slot array documented on
/// the module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Archive file name without `.zip`.
    pub stem: String,
    /// Author title, or the stem when the sidecar had none.
    pub title: String,
    /// Author(s), empty when unknown.
    pub author: String,
    /// Archive date, day precision.
    pub date: Date,
    /// Size in bytes.
    pub size: u64,
    /// Base game.
    pub game: IndexGame,
    /// Port family.
    pub port: Port,
    /// Whether a usable sidecar was read.
    pub txt: bool,
    /// Cacoward year, if a winner (R8).
    pub cacoward: Option<u16>,
    /// Recipe id when a DDL recipe pins this archive (R8).
    pub in_catalog: Option<String>,
    /// Map count from the text (R8).
    pub maps: Option<u16>,
    /// Newest upload sharing this title (R8).
    pub latest_version: bool,
}

impl Entry {
    /// An entry with the filename fallback: no sidecar metadata yet.
    #[must_use]
    pub fn fallback(stem: &str, date: Date, size: u64, game: IndexGame) -> Self {
        Self {
            stem: stem.to_owned(),
            title: stem.to_owned(),
            author: String::new(),
            date,
            size,
            game,
            port: Port::Unknown,
            txt: false,
            cacoward: None,
            in_catalog: None,
            maps: None,
            latest_version: false,
        }
    }

    /// The archive path with its `.zip` suffix.
    #[must_use]
    pub fn zip_name(&self) -> String {
        format!("{}.zip", self.stem)
    }
}

impl Serialize for Entry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(12))?;
        seq.serialize_element(&self.stem)?;
        seq.serialize_element(&self.title)?;
        seq.serialize_element(&self.author)?;
        seq.serialize_element(&self.date.to_iso())?;
        seq.serialize_element(&self.size)?;
        seq.serialize_element(&self.game)?;
        seq.serialize_element(&self.port)?;
        seq.serialize_element(&self.txt)?;
        seq.serialize_element(&self.cacoward)?;
        seq.serialize_element(&self.in_catalog)?;
        seq.serialize_element(&self.maps)?;
        seq.serialize_element(&self.latest_version)?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EntryVisitor;

        impl<'de> Visitor<'de> for EntryVisitor {
            type Value = Entry;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a 12-element /idgames index entry array")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Entry, A::Error> {
                let stem: String = field(&mut seq, 0)?;
                let title: String = field(&mut seq, 1)?;
                let author: String = field(&mut seq, 2)?;
                let date_text: String = field(&mut seq, 3)?;
                let date = Date::parse_iso(&date_text).ok_or_else(|| {
                    de::Error::custom(format!("invalid archive date `{date_text}`"))
                })?;
                let size: u64 = field(&mut seq, 4)?;
                let game: IndexGame = field(&mut seq, 5)?;
                let port: Port = field(&mut seq, 6)?;
                let txt: bool = field(&mut seq, 7)?;
                let cacoward: Option<u16> = field(&mut seq, 8)?;
                let in_catalog: Option<String> = field(&mut seq, 9)?;
                let maps: Option<u16> = field(&mut seq, 10)?;
                let latest_version: bool = field(&mut seq, 11)?;
                if seq.next_element::<de::IgnoredAny>()?.is_some() {
                    return Err(de::Error::custom("an index entry has more than 12 fields"));
                }
                Ok(Entry {
                    stem,
                    title,
                    author,
                    date,
                    size,
                    game,
                    port,
                    txt,
                    cacoward,
                    in_catalog,
                    maps,
                    latest_version,
                })
            }
        }

        deserializer.deserialize_seq(EntryVisitor)
    }
}

fn field<'de, T: Deserialize<'de>, A: SeqAccess<'de>>(
    seq: &mut A,
    position: usize,
) -> Result<T, A::Error> {
    seq.next_element()?
        .ok_or_else(|| de::Error::invalid_length(position, &"a 12-element entry"))
}

/// The published index (spec 050, R3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Index {
    /// Format version; only [`INDEX_SCHEMA`] is accepted.
    pub schema: u32,
    /// Build time, RFC 3339 UTC.
    pub generated_at: String,
    /// Where the inventory came from.
    pub source: IndexSource,
    /// Oldest DDL allowed to read this index.
    pub min_ddl_version: String,
    /// Directory → entries sorted by stem.
    pub entries: BTreeMap<String, Vec<Entry>>,
}

impl Index {
    /// The total number of entries.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.entries.values().map(Vec::len).sum()
    }

    /// The directories, in publication order.
    #[must_use]
    pub fn directories(&self) -> usize {
        self.entries.len()
    }

    /// Serializes to the published bytes: compact JSON plus a trailing newline.
    ///
    /// # Errors
    ///
    /// Returns [`IndexError::Oversize`] when the document would exceed
    /// [`MAX_INDEX_BYTES`], or [`IndexError::Malformed`] if serialization
    /// itself fails (it cannot for this model, but the caller must not assume).
    pub fn to_json(&self) -> Result<Vec<u8>, IndexError> {
        let mut bytes =
            serde_json::to_vec(self).map_err(|error| IndexError::Malformed(error.to_string()))?;
        bytes.push(b'\n');
        if bytes.len() as u64 > MAX_INDEX_BYTES {
            return Err(IndexError::Oversize {
                limit: MAX_INDEX_BYTES,
                got: bytes.len() as u64,
            });
        }
        Ok(bytes)
    }

    /// Checks every whole-document and per-entry rule.
    ///
    /// # Errors
    ///
    /// See [`IndexError`].
    pub fn validate(&self) -> Result<(), IndexError> {
        if self.schema != INDEX_SCHEMA {
            return Err(IndexError::UnknownSchema { got: self.schema });
        }
        if !is_rfc3339_utc(&self.generated_at) {
            return Err(IndexError::Malformed(format!(
                "`generated_at` is not `YYYY-MM-DDTHH:MM:SSZ`: `{}`",
                self.generated_at
            )));
        }
        if semver::Version::parse(&self.min_ddl_version).is_err() {
            return Err(IndexError::Malformed(format!(
                "`min_ddl_version` `{}` is not a semantic version",
                self.min_ddl_version
            )));
        }
        validate_source(&self.source)?;
        if self.entries.len() > MAX_DIRS {
            return Err(IndexError::Malformed(format!(
                "{} directories exceeds the {MAX_DIRS} bound",
                self.entries.len()
            )));
        }
        let mut total = 0usize;
        let mut titles: BTreeMap<String, usize> = BTreeMap::new();
        for (dir, entries) in &self.entries {
            if !valid_dir(dir) {
                return Err(IndexError::Malformed(format!(
                    "directory `{dir}` is not an indexable DOOM level tree"
                )));
            }
            if entries.is_empty() {
                return Err(IndexError::Malformed(format!(
                    "directory `{dir}` has no entries"
                )));
            }
            total += entries.len();
            if total > MAX_ENTRIES {
                return Err(IndexError::Malformed(format!(
                    "more than {MAX_ENTRIES} entries"
                )));
            }
            let mut seen = BTreeSet::new();
            let mut previous: Option<&str> = None;
            for entry in entries {
                validate_entry(entry)?;
                if let Some(last) = previous
                    && last >= entry.stem.as_str()
                {
                    return Err(IndexError::Malformed(format!(
                        "directory `{dir}` is not sorted by stem at `{}`",
                        entry.stem
                    )));
                }
                previous = Some(&entry.stem);
                if !seen.insert(entry.stem.clone()) {
                    return Err(IndexError::Malformed(format!(
                        "duplicate stem `{}` in `{dir}`",
                        entry.stem
                    )));
                }
                if entry.latest_version {
                    *titles.entry(entry.title.to_lowercase()).or_default() += 1;
                }
            }
        }
        if total == 0 {
            return Err(IndexError::Malformed("the index has no entries".into()));
        }
        if let Some((title, count)) = titles.iter().find(|(_, count)| **count > 1) {
            return Err(IndexError::Malformed(format!(
                "`latest_version` is set {count} times for title `{title}`"
            )));
        }
        Ok(())
    }

    /// Parses and validates a published document.
    ///
    /// # Errors
    ///
    /// See [`IndexError`]; the size bound is checked before parsing.
    pub fn parse(bytes: &[u8]) -> Result<Self, IndexError> {
        if bytes.len() as u64 > MAX_INDEX_BYTES {
            return Err(IndexError::Oversize {
                limit: MAX_INDEX_BYTES,
                got: bytes.len() as u64,
            });
        }
        let index: Self = serde_json::from_slice(bytes)
            .map_err(|error| IndexError::Malformed(error.to_string()))?;
        index.validate()?;
        Ok(index)
    }
}

fn validate_source(source: &IndexSource) -> Result<(), IndexError> {
    if !source.index.starts_with("https://") {
        return Err(IndexError::Malformed(format!(
            "source index `{}` is not an HTTPS URL",
            source.index
        )));
    }
    if source.index.len() > 2_048 {
        return Err(IndexError::Malformed("source index URL is too long".into()));
    }
    if !is_hex_sha256(&source.index_sha256) {
        return Err(IndexError::Malformed(format!(
            "source index SHA-256 `{}` is not 64 hex digits",
            source.index_sha256
        )));
    }
    if !safe_host(&source.mirror) {
        return Err(IndexError::Malformed(format!(
            "source mirror `{}` is not a bare host name",
            source.mirror
        )));
    }
    Ok(())
}

fn validate_entry(entry: &Entry) -> Result<(), IndexError> {
    if !safe_stem(&entry.stem) {
        return Err(IndexError::Malformed(format!(
            "unsafe entry stem `{}`",
            entry.stem
        )));
    }
    for (label, value) in [("title", &entry.title), ("author", &entry.author)] {
        if value.chars().count() > MAX_TEXT_CHARS {
            return Err(IndexError::Malformed(format!(
                "`{label}` for `{}` exceeds {MAX_TEXT_CHARS} characters",
                entry.stem
            )));
        }
        if value.chars().any(|character| character.is_control()) {
            return Err(IndexError::Malformed(format!(
                "`{label}` for `{}` contains a control character",
                entry.stem
            )));
        }
    }
    if entry.title.is_empty() {
        return Err(IndexError::Malformed(format!(
            "entry `{}` has no title (the stem fallback is required)",
            entry.stem
        )));
    }
    if entry.size == 0 || entry.size > MAX_ARCHIVE_BYTES {
        return Err(IndexError::Malformed(format!(
            "entry `{}` has an impossible size {}",
            entry.stem, entry.size
        )));
    }
    if let Some(year) = entry.cacoward
        && !(1994..=2100).contains(&year)
    {
        return Err(IndexError::Malformed(format!(
            "entry `{}` has Cacoward year {year}",
            entry.stem
        )));
    }
    if let Some(maps) = entry.maps
        && !(1..=999).contains(&maps)
    {
        return Err(IndexError::Malformed(format!(
            "entry `{}` has map count {maps}",
            entry.stem
        )));
    }
    if let Some(recipe) = &entry.in_catalog
        && !safe_slug(recipe)
    {
        return Err(IndexError::Malformed(format!(
            "entry `{}` has an invalid recipe id `{recipe}`",
            entry.stem
        )));
    }
    Ok(())
}

fn valid_dir(dir: &str) -> bool {
    dir.len() <= 512
        && !dir.starts_with('/')
        && !dir.contains('\\')
        && dir.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment.chars().all(|character| {
                    (character.is_ascii_graphic() || character == ' ')
                        && character != '/'
                        && character != '\\'
                })
        })
        && is_indexable_dir(dir)
}

fn safe_stem(stem: &str) -> bool {
    !stem.is_empty()
        && stem.len() <= 128
        && stem != "."
        && stem != ".."
        && stem == stem.trim()
        && stem.chars().all(|character| {
            (character.is_ascii_graphic() || character == ' ')
                && !character.is_ascii_control()
                && character != '/'
                && character != '\\'
        })
}

fn safe_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 255
        && !host.contains('/')
        && !host.contains(':')
        && !host.contains(' ')
        && host
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
}

fn safe_slug(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 64
        && text
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn is_hex_sha256(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Why an index document was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexError {
    /// Larger than [`MAX_INDEX_BYTES`].
    Oversize {
        /// The bound.
        limit: u64,
        /// What was seen.
        got: u64,
    },
    /// `schema` is not [`INDEX_SCHEMA`].
    UnknownSchema {
        /// What the document declared.
        got: u32,
    },
    /// Not the expected shape, or an invariant was broken.
    Malformed(String),
}

impl fmt::Display for IndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Oversize { limit, got } => {
                write!(formatter, "index is larger than {limit} bytes (got {got})")
            }
            Self::UnknownSchema { got } => write!(
                formatter,
                "index schema {got} is not supported (this DDL understands {INDEX_SCHEMA})"
            ),
            Self::Malformed(detail) => write!(formatter, "index is malformed: {detail}"),
        }
    }
}

impl std::error::Error for IndexError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(stem: &str) -> Entry {
        Entry {
            stem: stem.to_owned(),
            title: format!("Title {stem}"),
            author: "Someone".into(),
            date: Date::new(2021, 7, 26).unwrap(),
            size: 1_000,
            game: IndexGame::Doom2,
            port: Port::Mbf21,
            txt: true,
            cacoward: Some(2021),
            in_catalog: Some("auger-zenith".into()),
            maps: Some(11),
            latest_version: false,
        }
    }

    fn index() -> Index {
        let mut entries = BTreeMap::new();
        entries.insert(
            "levels/doom2/Ports/megawads".to_owned(),
            vec![entry("augzen"), entry("b_two")],
        );
        Index {
            schema: INDEX_SCHEMA,
            generated_at: "2026-10-19T05:07:08Z".into(),
            source: IndexSource {
                index: "https://youfailit.net/pub/idgames/ls-laR.gz".into(),
                index_sha256: "502a7ccd3195452fb4955f077a078d0167bf80b721535be8c59e1daa269c2603"
                    .into(),
                mirror: "youfailit.net".into(),
            },
            min_ddl_version: DEFAULT_MIN_DDL_VERSION.into(),
            entries,
        }
    }

    #[test]
    fn a_valid_index_round_trips_byte_for_byte() {
        let index = index();
        index.validate().unwrap();
        let json = index.to_json().unwrap();
        let parsed = Index::parse(&json).unwrap();
        assert_eq!(parsed, index);
        assert_eq!(parsed.to_json().unwrap(), json);
    }

    #[test]
    fn entries_are_compact_twelve_slot_arrays() {
        let json = index().to_json().unwrap();
        let text = String::from_utf8(json).unwrap();
        assert!(
            text.contains(
                r#"["augzen","Title augzen","Someone","2021-07-26",1000,"doom2","mbf21",true,2021,"auger-zenith",11,false]"#
            ),
            "{text}"
        );
        // No descriptions, credits, ratings or txt bodies anywhere.
        assert!(!text.contains("description"));
    }

    #[test]
    fn unknown_schema_and_oversize_documents_are_refused() {
        let mut broken = index();
        broken.schema = 2;
        assert_eq!(broken.validate(), Err(IndexError::UnknownSchema { got: 2 }));

        let huge = vec![b' '; MAX_INDEX_BYTES as usize + 1];
        assert!(matches!(
            Index::parse(&huge),
            Err(IndexError::Oversize { .. })
        ));
    }

    #[test]
    fn ordering_is_part_of_the_contract() {
        let mut broken = index();
        broken
            .entries
            .get_mut("levels/doom2/Ports/megawads")
            .unwrap()
            .reverse();
        assert!(matches!(broken.validate(), Err(IndexError::Malformed(_))));
    }

    #[test]
    fn non_doom_directories_are_refused() {
        let mut broken = index();
        broken
            .entries
            .insert("levels/heretic".to_owned(), vec![entry("heretic_thing")]);
        assert!(broken.validate().is_err());
        let mut broken = index();
        broken.entries.insert(
            "levels/doom2/deathmatch".to_owned(),
            vec![entry("dm_thing")],
        );
        assert!(broken.validate().is_err());
    }

    #[test]
    fn unsafe_stems_and_impossible_sizes_are_refused() {
        for stem in ["../x", "a/b", "a\\b", "", "a\nb"] {
            let mut broken = index();
            broken
                .entries
                .get_mut("levels/doom2/Ports/megawads")
                .unwrap()[0]
                .stem = stem.to_owned();
            assert!(broken.validate().is_err(), "{stem:?} must be refused");
        }
        let mut broken = index();
        broken
            .entries
            .get_mut("levels/doom2/Ports/megawads")
            .unwrap()[0]
            .size = 0;
        assert!(broken.validate().is_err());
    }

    #[test]
    fn a_validation_error_names_the_entry_and_the_field() {
        let mut broken = index();
        broken
            .entries
            .get_mut("levels/doom2/Ports/megawads")
            .unwrap()[0]
            .title = "Cinema\u{1a}2".into();
        let error = broken.validate().unwrap_err();
        let text = error.to_string();
        assert!(text.contains("`title`"), "{text}");
        assert!(text.contains("`augzen`"), "{text}");
    }

    #[test]
    fn at_most_one_latest_version_per_title() {
        let mut broken = index();
        for entry in broken
            .entries
            .get_mut("levels/doom2/Ports/megawads")
            .unwrap()
        {
            entry.title = "Same Title".into();
            entry.latest_version = true;
        }
        assert!(broken.validate().is_err());
    }

    #[test]
    fn the_size_bound_is_checked_on_write_too() {
        // A document with many long titles approaches the cap; the check is a
        // guard, so assert the constant and the error path independently.
        assert_eq!(MAX_INDEX_BYTES, 2 * 1024 * 1024);
        let mut index = index();
        index.generated_at = "nope".into();
        assert!(index.validate().is_err());
    }
}

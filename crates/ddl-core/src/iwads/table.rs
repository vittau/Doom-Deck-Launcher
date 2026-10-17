//! The known-IWAD table (spec 007, R1).
//!
//! [`IwadTable`] is the parsed, validated form of `crates/ddl-core/data/iwads.toml`.
//! Every variant carries the game a recipe requires, a canonical filename, its
//! size, whichever identification hashes have a published source, a default
//! complevel (research §3) and the URL those values come from.
//!
//! Values are hand-authored data, so the type is deliberately strict: loading
//! rejects duplicate ids, unknown games, malformed or wrongly-sized hashes,
//! an id that is not a safe path component (it becomes a directory name under
//! `<data>/iwads/`), and any hash value without a source to cite.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// The embedded, reviewed table shipped with DDL.
pub const EMBEDDED_TABLE: &str = include_str!("../../data/iwads.toml");

/// Schema version this build understands.
pub const SCHEMA_VERSION: u32 = 1;

/// An abstract base game a recipe requires. `nerve` is the No Rest for the
/// Living expansion, which ships as a PWAD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    Doom,
    Doom2,
    Tnt,
    Plutonia,
    Nerve,
}

impl Game {
    /// Every game, in table order.
    pub const ALL: [Game; 5] = [
        Game::Doom,
        Game::Doom2,
        Game::Tnt,
        Game::Plutonia,
        Game::Nerve,
    ];

    /// The stable lowercase key used in TOML and in the JSON registry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Game::Doom => "doom",
            Game::Doom2 => "doom2",
            Game::Tnt => "tnt",
            Game::Plutonia => "plutonia",
            Game::Nerve => "nerve",
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An identification hash algorithm, ordered weakest to strongest.
///
/// SHA-256 is always computed locally; MD5/SHA-1/CRC-32 exist only to match
/// against the published identification tables (decision D-017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HashKind {
    Crc32,
    Md5,
    Sha1,
    Sha256,
}

impl HashKind {
    /// The TOML/`hash_sources` key for this algorithm.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            HashKind::Crc32 => "crc32",
            HashKind::Md5 => "md5",
            HashKind::Sha1 => "sha1",
            HashKind::Sha256 => "sha256",
        }
    }

    /// Expected number of hex digits.
    #[must_use]
    pub const fn hex_len(self) -> usize {
        match self {
            HashKind::Crc32 => 8,
            HashKind::Md5 => 32,
            HashKind::Sha1 => 40,
            HashKind::Sha256 => 64,
        }
    }
}

/// One recognised IWAD file variant.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct IwadVariant {
    /// Stable variant id; also used as a directory name under `<data>/iwads/`.
    pub id: String,
    /// Abstract game a recipe requires.
    pub game: Game,
    /// Human-readable title for the UI.
    pub title: String,
    /// Canonical basename DSDA-Doom needs for filename-driven mission choice.
    pub filename: String,
    /// Exact file size in bytes.
    pub size: u64,
    /// SHA-256, when a source publishes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// MD5, when a source publishes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
    /// SHA-1, when a source publishes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    /// CRC-32, when a source publishes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crc32: Option<String>,
    /// Default numeric complevel for this variant (research §3).
    pub default_complevel: u8,
    /// `false` for the shareware episode, which commercial content rejects.
    #[serde(default = "default_true")]
    pub playable: bool,
    /// The page this variant's values come from.
    pub source: String,
    /// Optional per-hash source overrides (e.g. the two SHA-256 rows).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hash_sources: BTreeMap<String, String>,
}

fn default_true() -> bool {
    true
}

impl IwadVariant {
    /// The published digest for `kind`, if any.
    #[must_use]
    pub fn hash(&self, kind: HashKind) -> Option<&str> {
        match kind {
            HashKind::Crc32 => self.crc32.as_deref(),
            HashKind::Md5 => self.md5.as_deref(),
            HashKind::Sha1 => self.sha1.as_deref(),
            HashKind::Sha256 => self.sha256.as_deref(),
        }
    }

    /// Every published digest, strongest first.
    #[must_use]
    pub fn hashes(&self) -> Vec<(HashKind, &str)> {
        let mut all = Vec::new();
        for kind in [
            HashKind::Sha256,
            HashKind::Sha1,
            HashKind::Md5,
            HashKind::Crc32,
        ] {
            if let Some(value) = self.hash(kind) {
                all.push((kind, value));
            }
        }
        all
    }

    /// The strongest published digest, which is the one identification matches.
    #[must_use]
    pub fn strongest_hash(&self) -> Option<(HashKind, &str)> {
        self.hashes().into_iter().next()
    }

    /// The URL that publishes `kind`'s value.
    #[must_use]
    pub fn source_for(&self, kind: HashKind) -> &str {
        self.hash_sources
            .get(kind.as_str())
            .map_or(self.source.as_str(), String::as_str)
    }
}

/// The parsed known-IWAD table.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct IwadTable {
    /// Schema version of the source document.
    pub schema_version: u32,
    /// One entry per file variant, in table order.
    #[serde(rename = "iwad", default)]
    pub variants: Vec<IwadVariant>,
}

/// Why loading or validating the table failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableError {
    /// The TOML could not be parsed.
    Parse(String),
    /// The document declares a schema version this build does not know.
    UnsupportedSchema(u32),
    /// The table contains no variants.
    Empty,
    /// Two variants share an id.
    DuplicateId(String),
    /// A field of a variant is missing or malformed.
    Invalid {
        /// The variant id (or a placeholder when the id itself is bad).
        id: String,
        /// The field name.
        field: &'static str,
        /// What is wrong.
        detail: String,
    },
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableError::Parse(message) => write!(f, "iwads.toml is not valid TOML: {message}"),
            TableError::UnsupportedSchema(version) => write!(
                f,
                "iwads.toml schema_version {version} is not supported (expected {SCHEMA_VERSION})"
            ),
            TableError::Empty => f.write_str("iwads.toml defines no IWAD variants"),
            TableError::DuplicateId(id) => write!(f, "iwads.toml defines the id `{id}` twice"),
            TableError::Invalid { id, field, detail } => {
                write!(f, "iwads.toml variant `{id}` field `{field}`: {detail}")
            }
        }
    }
}

impl std::error::Error for TableError {}

impl IwadTable {
    /// Parses and validates a table document.
    ///
    /// # Errors
    ///
    /// Returns [`TableError`] when the TOML is malformed or fails validation
    /// (see the module docs for the rules).
    pub fn parse(text: &str) -> Result<Self, TableError> {
        let table: Self =
            toml::from_str(text).map_err(|error| TableError::Parse(error.to_string()))?;
        table.validate()?;
        Ok(table)
    }

    /// Loads the reviewed table embedded in the binary.
    ///
    /// # Panics
    ///
    /// Panics only if the embedded data is invalid, which a unit test forbids.
    #[must_use]
    pub fn embedded() -> Self {
        Self::parse(EMBEDDED_TABLE).expect("embedded iwads.toml is valid")
    }

    /// Validates an already-deserialized table.
    ///
    /// # Errors
    ///
    /// Returns [`TableError`] describing the first problem found.
    pub fn validate(&self) -> Result<(), TableError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(TableError::UnsupportedSchema(self.schema_version));
        }
        if self.variants.is_empty() {
            return Err(TableError::Empty);
        }

        let mut seen: BTreeMap<&str, ()> = BTreeMap::new();
        for variant in &self.variants {
            if seen.insert(variant.id.as_str(), ()).is_some() {
                return Err(TableError::DuplicateId(variant.id.clone()));
            }
            variant.validate()?;
        }
        Ok(())
    }

    /// Looks up a variant by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&IwadVariant> {
        self.variants.iter().find(|variant| variant.id == id)
    }

    /// Every variant of `game`, in table order.
    pub fn variants_for(&self, game: Game) -> impl Iterator<Item = &IwadVariant> {
        self.variants
            .iter()
            .filter(move |variant| variant.game == game)
    }
}

impl IwadVariant {
    /// Validates a single variant.
    fn validate(&self) -> Result<(), TableError> {
        let invalid = |field: &'static str, detail: String| TableError::Invalid {
            id: self.id.clone(),
            field,
            detail,
        };

        if !is_safe_id(&self.id) {
            return Err(invalid(
                "id",
                "must be a non-empty path-safe token of ASCII letters, digits, '-', '_' or '.'"
                    .to_owned(),
            ));
        }
        if self.title.trim().is_empty() {
            return Err(invalid("title", "must not be empty".to_owned()));
        }
        if !is_bare_filename(&self.filename) {
            return Err(invalid(
                "filename",
                "must be a bare file name without path separators".to_owned(),
            ));
        }
        if self.size == 0 {
            return Err(invalid("size", "must be greater than zero".to_owned()));
        }
        if self.default_complevel > 21 {
            return Err(invalid(
                "default_complevel",
                format!("{} is not a known complevel", self.default_complevel),
            ));
        }
        if self.source.trim().is_empty() {
            return Err(invalid(
                "source",
                "must cite where the values come from".to_owned(),
            ));
        }
        if self.hashes().is_empty() {
            return Err(invalid(
                "hash",
                "at least one of sha256/sha1/md5/crc32 is required".to_owned(),
            ));
        }

        for (kind, value) in self.hashes() {
            if !is_lower_hex(value, kind.hex_len()) {
                return Err(invalid(
                    kind.as_str(),
                    format!("must be {} lowercase hex digits", kind.hex_len()),
                ));
            }
            if self.source_for(kind).trim().is_empty() {
                return Err(invalid(
                    kind.as_str(),
                    "has a value but no source to cite".to_owned(),
                ));
            }
        }

        for key in self.hash_sources.keys() {
            let known = [
                HashKind::Sha256,
                HashKind::Sha1,
                HashKind::Md5,
                HashKind::Crc32,
            ]
            .iter()
            .any(|kind| kind.as_str() == key);
            if !known {
                return Err(invalid(
                    "hash_sources",
                    format!("`{key}` is not a known hash algorithm"),
                ));
            }
            if self
                .hash_sources
                .get(key)
                .is_some_and(|value| value.trim().is_empty())
            {
                return Err(invalid(
                    "hash_sources",
                    format!("`{key}` must not be empty"),
                ));
            }
        }

        Ok(())
    }
}

/// `true` when `id` is a safe single path component.
fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && id != "."
        && id != ".."
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

/// `true` when `name` is a bare file name with no directory components.
fn is_bare_filename(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\'])
        && !name.contains('\0')
}

/// `true` when `value` is exactly `len` lowercase hex digits.
fn is_lower_hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_table_loads_and_validates() {
        let table = IwadTable::embedded();
        assert_eq!(table.schema_version, SCHEMA_VERSION);
        assert!(table.variants.len() >= 20, "expected a populated table");
        // The whole point of validation: every hash value has a source.
        for variant in &table.variants {
            for (kind, _) in variant.hashes() {
                assert!(
                    !variant.source_for(kind).trim().is_empty(),
                    "{} {} has no source",
                    variant.id,
                    kind.as_str()
                );
            }
            assert!(variant.strongest_hash().is_some());
        }
    }

    #[test]
    fn embedded_table_contains_the_reference_variants() {
        let table = IwadTable::embedded();

        let doom2 = table.get("doom2-1.9").expect("doom2 1.9 present");
        assert_eq!(doom2.size, 14_604_584);
        assert_eq!(doom2.default_complevel, 2);
        assert_eq!(doom2.game, Game::Doom2);
        assert_eq!(doom2.filename, "doom2.wad");

        let doomu = table.get("doom-1.9ud").expect("ultimate doom present");
        assert_eq!(doomu.filename, "doomu.wad");
        assert_eq!(doomu.default_complevel, 3);

        let plutonia = table.get("plutonia-1.9").expect("plutonia present");
        assert_eq!(plutonia.default_complevel, 4);
        assert_eq!(plutonia.game, Game::Plutonia);
    }

    #[test]
    fn embedded_table_contains_the_anthology_gog_revisions() {
        let table = IwadTable::embedded();

        // Values transcribed from research §1.3 / §1.4 (DoomWiki TNT.WAD /
        // PLUTONIA.WAD, read via the Wayback Machine). MD5/SHA-1/CRC-32 only.
        let tnt = table.get("tnt-anthology").expect("tnt anthology present");
        assert_eq!(tnt.game, Game::Tnt);
        assert_eq!(tnt.filename, "tnt.wad");
        assert_eq!(tnt.size, 18_654_796);
        assert_eq!(tnt.md5.as_deref(), Some("1d39e405bf6ee3df69a8d2646c8d5c49"));
        assert_eq!(
            tnt.sha1.as_deref(),
            Some("4a65c8b960225505187c36040b41a40b152f8f3e")
        );
        assert_eq!(tnt.crc32.as_deref(), Some("d4bb05c0"));
        assert_eq!(tnt.default_complevel, 4);
        assert!(tnt.playable);
        assert_eq!(tnt.source, "https://doomwiki.org/wiki/TNT.WAD");
        // The strongest published hash is the one identification matches on.
        assert_eq!(
            tnt.strongest_hash().map(|(kind, _)| kind),
            Some(HashKind::Sha1)
        );

        let plutonia = table
            .get("plutonia-anthology")
            .expect("plutonia anthology present");
        assert_eq!(plutonia.game, Game::Plutonia);
        assert_eq!(plutonia.filename, "plutonia.wad");
        assert_eq!(plutonia.size, 18_240_172);
        assert_eq!(
            plutonia.md5.as_deref(),
            Some("3493be7e1e2588bc9c8b31eab2587a04")
        );
        assert_eq!(
            plutonia.sha1.as_deref(),
            Some("f131cbe1946d7fddb3caec4aa258c83399c21e60")
        );
        assert_eq!(plutonia.crc32.as_deref(), Some("15cd1448"));
        assert_eq!(plutonia.default_complevel, 4);
        assert!(plutonia.playable);
        assert_eq!(plutonia.source, "https://doomwiki.org/wiki/PLUTONIA.WAD");
        assert_eq!(
            plutonia.strongest_hash().map(|(kind, _)| kind),
            Some(HashKind::Sha1)
        );
    }

    #[test]
    fn shareware_entry_is_not_playable() {
        let table = IwadTable::embedded();
        let shareware = table.get("doom-shareware-1.9").expect("shareware present");
        assert!(!shareware.playable);
        assert_eq!(shareware.filename, "doom1.wad");
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let text = r"
schema_version = 1
[[iwad]]
id = 'doom2-1.9'
game = 'doom2'
title = 'A'
filename = 'doom2.wad'
size = 1
sha1 = '0000000000000000000000000000000000000000'
default_complevel = 2
source = 'https://example.invalid/a'
[[iwad]]
id = 'doom2-1.9'
game = 'doom2'
title = 'B'
filename = 'doom2.wad'
size = 2
sha1 = '1111111111111111111111111111111111111111'
default_complevel = 2
source = 'https://example.invalid/b'
";
        assert_eq!(
            IwadTable::parse(text),
            Err(TableError::DuplicateId("doom2-1.9".to_owned()))
        );
    }

    #[test]
    fn hash_without_a_source_is_rejected() {
        let text = r#"
schema_version = 1
[[iwad]]
id = "x"
game = "doom"
title = "T"
filename = "doom.wad"
size = 10
sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
default_complevel = 3
source = ""
"#;
        let error = IwadTable::parse(text).expect_err("empty source must fail");
        assert!(matches!(
            error,
            TableError::Invalid {
                field: "source",
                ..
            }
        ));
    }

    #[test]
    fn bad_hash_length_is_rejected() {
        let text = r#"
schema_version = 1
[[iwad]]
id = "x"
game = "doom"
title = "T"
filename = "doom.wad"
size = 10
sha1 = "abc"
default_complevel = 3
source = "https://example.invalid"
"#;
        let error = IwadTable::parse(text).expect_err("short sha1 must fail");
        assert!(matches!(error, TableError::Invalid { field: "sha1", .. }));
    }

    #[test]
    fn unsafe_id_is_rejected() {
        let text = r#"
schema_version = 1
[[iwad]]
id = "../escape"
game = "doom"
title = "T"
filename = "doom.wad"
size = 10
md5 = "00000000000000000000000000000000"
default_complevel = 3
source = "https://example.invalid"
"#;
        let error = IwadTable::parse(text).expect_err("path traversal id must fail");
        assert!(matches!(error, TableError::Invalid { field: "id", .. }));
    }

    #[test]
    fn wrong_schema_version_is_rejected() {
        let error = IwadTable::parse("schema_version = 99\n").expect_err("schema must match");
        assert_eq!(error, TableError::UnsupportedSchema(99));
    }

    #[test]
    fn strongest_hash_prefers_sha256() {
        let table = IwadTable::embedded();
        let (kind, value) = table
            .get("doom2-1.9")
            .expect("doom2 present")
            .strongest_hash()
            .expect("has hashes");
        assert_eq!(kind, HashKind::Sha256);
        assert_eq!(value.len(), 64);

        let (kind, _) = table
            .get("tnt-1.9")
            .expect("tnt present")
            .strongest_hash()
            .expect("has hashes");
        assert_eq!(kind, HashKind::Sha1);
    }

    #[test]
    fn variants_for_game_filters() {
        let table = IwadTable::embedded();
        let doom2: Vec<_> = table
            .variants_for(Game::Doom2)
            .map(|v| v.id.as_str())
            .collect();
        assert!(doom2.contains(&"doom2-1.9"));
        assert!(doom2.iter().all(|id| !id.starts_with("tnt")));
    }
}

//! The custom setup model (spec 026, R2): a player-made launch target stored as
//! versioned JSON, referenced files and all.
//!
//! A setup is a name, a base IWAD, two ordered user-file groups and a
//! compatibility choice. The two groups are deliberate: DSDA-Doom processes
//! every `-deh` patch after every `-file` WAD, regardless of argv position, so
//! the model never pretends the two can interleave (spec 026, R3; research §1).
//!
//! Files are stored **in place**: DDL records their absolute path, size, SHA-256
//! and detected kind, and never copies, moves or executes them.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::engines::dsda::Complevel;

use super::kind::{Kind, UserFile};

/// Schema version written into each setup document.
pub const SCHEMA_VERSION: u32 = 1;

/// The base game a custom setup launches: an IWAD DDL identified (spec 004), or
/// a user file whose header is `IWAD` (spec 026, R2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case", deny_unknown_fields)]
pub enum CustomIwad {
    /// An identified IWAD from DDL's registry, launched through its canonical
    /// link so the engine picks the right mission (D-017).
    Identified {
        /// The identified variant id, e.g. `doom2-1.9`.
        variant: String,
    },
    /// A player file the engine accepts directly as an IWAD.
    File {
        /// The recorded user file; its header is `IWAD`.
        file: UserFile,
    },
}

/// The `-complevel` a custom setup asks for (spec 026, R2).
///
/// `Auto` emits no `-complevel` at all, so DSDA-Doom decides (from a `COMPLVL`
/// lump, else its configured default — research §1). `Level` pins one reviewed
/// value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Compatibility {
    /// No `-complevel`: the engine decides.
    #[default]
    Auto,
    /// One reviewed compatibility number.
    Level(Complevel),
}

impl Compatibility {
    /// The completeness level DSDA receives, or `None` for `Auto`.
    #[must_use]
    pub const fn level(self) -> Option<Complevel> {
        match self {
            Self::Auto => None,
            Self::Level(level) => Some(level),
        }
    }

    /// Parses `auto` or a reviewed compatibility number.
    ///
    /// # Errors
    ///
    /// Returns [`SetupError::Invalid`] for anything that is neither `auto` nor a
    /// reviewed number (`0`-`17` and `21`).
    pub fn parse(value: &str) -> Result<Self, SetupError> {
        if value.eq_ignore_ascii_case("auto") {
            return Ok(Self::Auto);
        }
        let number: u8 = value.parse().map_err(|_| {
            SetupError::Invalid(format!(
                "`{value}` is neither `auto` nor a reviewed compatibility number (0-17 or 21)"
            ))
        })?;
        Complevel::new(number)
            .map(Self::Level)
            .map_err(|error| SetupError::Invalid(error.to_string()))
    }
}

impl fmt::Display for Compatibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => f.write_str("auto"),
            Self::Level(level) => write!(f, "{}", level.value()),
        }
    }
}

impl Serialize for Compatibility {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::Level(level) => serializer.serialize_u8(level.value()),
        }
    }
}

impl<'de> Deserialize<'de> for Compatibility {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Word(String),
            Number(u8),
        }
        match Repr::deserialize(deserializer)? {
            Repr::Word(word) if word.eq_ignore_ascii_case("auto") => Ok(Self::Auto),
            Repr::Word(word) => Err(serde::de::Error::custom(format!(
                "compatibility `{word}` is neither `auto` nor a reviewed number"
            ))),
            Repr::Number(number) => Complevel::new(number)
                .map(Self::Level)
                .map_err(|error| serde::de::Error::custom(error.to_string())),
        }
    }
}

/// One custom setup (spec 026, R2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomSetup {
    /// Persisted schema version.
    pub schema_version: u32,
    /// Stable id, also the record file name and the runtime directory name.
    pub id: String,
    /// Friendly name for the UI.
    pub name: String,
    /// The base game.
    pub iwad: CustomIwad,
    /// Ordered WAD/ZIP files for the single `-file` flag.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<UserFile>,
    /// Ordered DEH/BEX patches for the single `-deh` flag.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patches: Vec<UserFile>,
    /// The compatibility choice.
    pub compatibility: Compatibility,
    /// Unix seconds when the setup was created.
    pub created_at: u64,
    /// Unix seconds of the last change.
    pub updated_at: u64,
}

impl CustomSetup {
    /// A new, empty setup at `now` (Unix seconds).
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        iwad: CustomIwad,
        compatibility: Compatibility,
        now: u64,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id: id.into(),
            name: name.into(),
            iwad,
            files: Vec::new(),
            patches: Vec::new(),
            compatibility,
            created_at: now,
            updated_at: now,
        }
    }

    /// Appends a file to the group its kind routes to, keeping player order and
    /// refusing a duplicate path (spec 026, R1/R3).
    ///
    /// # Errors
    ///
    /// Returns [`SetupError::Duplicate`] for a path already listed.
    pub fn add(&mut self, file: UserFile) -> Result<(), SetupError> {
        if self.contains(&file.path) {
            return Err(SetupError::Duplicate(file.path));
        }
        if file.kind.is_patch() {
            self.patches.push(file);
        } else {
            self.files.push(file);
        }
        Ok(())
    }

    /// Whether `path` is already in either group.
    #[must_use]
    pub fn contains(&self, path: &std::path::Path) -> bool {
        self.files.iter().any(|f| f.path == path) || self.patches.iter().any(|p| p.path == path)
    }

    /// Removes the entry at `index` in the requested group without touching the
    /// file on disk (spec 026, R6).
    ///
    /// # Errors
    ///
    /// Returns [`SetupError::OutOfRange`] when `index` is past the group.
    pub fn remove(&mut self, group: Group, index: usize) -> Result<UserFile, SetupError> {
        let list = match group {
            Group::Files => &mut self.files,
            Group::Patches => &mut self.patches,
        };
        if index >= list.len() {
            return Err(SetupError::OutOfRange {
                group,
                index,
                len: list.len(),
            });
        }
        Ok(list.remove(index))
    }

    /// Moves the entry at `from` to `to` within one group, keeping order
    /// (spec 026, R3/R7).
    ///
    /// # Errors
    ///
    /// Returns [`SetupError::OutOfRange`] when either index is past the group.
    pub fn move_entry(&mut self, group: Group, from: usize, to: usize) -> Result<(), SetupError> {
        let list = match group {
            Group::Files => &mut self.files,
            Group::Patches => &mut self.patches,
        };
        if from >= list.len() || to >= list.len() {
            return Err(SetupError::OutOfRange {
                group,
                index: from.max(to),
                len: list.len(),
            });
        }
        let entry = list.remove(from);
        list.insert(to, entry);
        Ok(())
    }

    /// The recorded user files in model order: the user-file IWAD first (when
    /// there is one), then the `-file` group, then the `-deh` group.
    #[must_use]
    pub fn all_user_files(&self) -> Vec<&UserFile> {
        let mut files: Vec<&UserFile> = Vec::new();
        if let CustomIwad::File { file } = &self.iwad {
            files.push(file);
        }
        files.extend(self.files.iter());
        files.extend(self.patches.iter());
        files
    }

    /// Checks the document's own invariants (not the files on disk).
    ///
    /// # Errors
    ///
    /// Returns a [`SetupError`] describing the first problem.
    pub fn validate(&self) -> Result<(), SetupError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(SetupError::UnsupportedSchema(self.schema_version));
        }
        if !safe_id(&self.id) {
            return Err(SetupError::UnsafeId(self.id.clone()));
        }
        if self.name.trim().is_empty() {
            return Err(SetupError::Invalid("the setup needs a name".to_owned()));
        }
        for file in &self.files {
            if !file.kind.is_load_file() {
                return Err(SetupError::WrongKind {
                    path: file.path.clone(),
                    kind: file.kind,
                });
            }
            if !file.path.is_absolute() {
                return Err(SetupError::Invalid(format!(
                    "`{}` is not an absolute path",
                    file.path.display()
                )));
            }
        }
        for patch in &self.patches {
            if !patch.kind.is_patch() {
                return Err(SetupError::WrongKind {
                    path: patch.path.clone(),
                    kind: patch.kind,
                });
            }
            if !patch.path.is_absolute() {
                return Err(SetupError::Invalid(format!(
                    "`{}` is not an absolute path",
                    patch.path.display()
                )));
            }
        }
        if let CustomIwad::File { file } = &self.iwad
            && file.kind != Kind::Wad
        {
            return Err(SetupError::WrongKind {
                path: file.path.clone(),
                kind: file.kind,
            });
        }
        Ok(())
    }
}

/// One of a setup's two ordered groups (spec 026, R3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    /// WAD/ZIP files, sent as the `-file` list.
    Files,
    /// DEH/BEX patches, sent as the `-deh` list.
    Patches,
}

impl Group {
    /// Parses `files` or `patches`.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "files" => Some(Self::Files),
            "patches" => Some(Self::Patches),
            _ => None,
        }
    }

    /// The lowercase label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Files => "files",
            Self::Patches => "patches",
        }
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The underlying indices of an extras list that belong to `group`, in list
/// order. A profile's extras are one ordered list (spec 026, R5); its two
/// engine groups are derived from the detected kinds.
#[must_use]
pub fn group_indices(extras: &[UserFile], group: Group) -> Vec<usize> {
    extras
        .iter()
        .enumerate()
        .filter(|(_, file)| match group {
            Group::Files => file.kind.is_load_file(),
            Group::Patches => file.kind.is_patch(),
        })
        .map(|(index, _)| index)
        .collect()
}

/// Whether `id` is safe as a file name and directory name.
#[must_use]
pub fn safe_id(id: &str) -> bool {
    !id.is_empty()
        && id != "."
        && id != ".."
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// A URL-style slug for a setup name, used as the default id.
#[must_use]
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = true;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("setup");
    }
    out
}

/// Why a setup document is invalid (spec 026, R2).
#[derive(Debug)]
pub enum SetupError {
    /// A filesystem operation failed.
    Io(std::io::Error),
    /// The document is not valid JSON.
    Parse(String),
    /// The document exceeds the read bound.
    TooLarge(u64),
    /// The document declares an unsupported schema version.
    UnsupportedSchema(u32),
    /// The id is not a safe file name.
    UnsafeId(String),
    /// A value is not acceptable.
    Invalid(String),
    /// A path is already listed.
    Duplicate(std::path::PathBuf),
    /// An entry has the wrong kind for its group.
    WrongKind {
        /// The offending path.
        path: std::path::PathBuf,
        /// The kind found.
        kind: Kind,
    },
    /// An index is past the group.
    OutOfRange {
        /// The group addressed.
        group: Group,
        /// The index given.
        index: usize,
        /// The group's length.
        len: usize,
    },
    /// Reading the referenced user file failed.
    File(super::kind::FileError),
}

impl fmt::Display for SetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "custom setup I/O error: {error}"),
            Self::Parse(message) => write!(f, "invalid custom setup: {message}"),
            Self::TooLarge(limit) => write!(f, "custom setup exceeds the {limit} byte read limit"),
            Self::UnsupportedSchema(version) => write!(
                f,
                "custom setup schema_version {version} is not supported (expected {SCHEMA_VERSION})"
            ),
            Self::UnsafeId(id) => write!(f, "`{id}` is not a safe setup id"),
            Self::Invalid(message) => write!(f, "invalid custom setup: {message}"),
            Self::Duplicate(path) => {
                write!(f, "`{}` is already in this setup", path.display())
            }
            Self::WrongKind { path, kind } => write!(
                f,
                "`{}` is a {kind} file and does not belong in that list",
                path.display()
            ),
            Self::OutOfRange { group, index, len } => write!(
                f,
                "index {index} is past the {group} list's {len} entr{}",
                if *len == 1 { "y" } else { "ies" }
            ),
            Self::File(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for SetupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::File(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SetupError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<super::kind::FileError> for SetupError {
    fn from(error: super::kind::FileError) -> Self {
        Self::File(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    fn file(name: &str, kind: Kind) -> UserFile {
        UserFile {
            path: PathBuf::from(format!("/games/{name}")),
            size: 10,
            sha256: "a".repeat(64),
            kind,
        }
    }

    fn setup() -> CustomSetup {
        CustomSetup::new(
            "my-setup",
            "My Setup",
            CustomIwad::Identified {
                variant: "doom2-1.9".to_owned(),
            },
            Compatibility::Auto,
            1_700_000_000,
        )
    }

    #[test]
    fn compatibility_parses_auto_and_reviewed_numbers_only() {
        assert_eq!(Compatibility::parse("AUTO").unwrap(), Compatibility::Auto);
        assert_eq!(
            Compatibility::parse("2").unwrap(),
            Compatibility::Level(Complevel::DOOM_1_9)
        );
        assert_eq!(
            Compatibility::parse("21").unwrap(),
            Compatibility::Level(Complevel::MBF21)
        );
        for bad in ["18", "19", "20", "22", "-1", "vanilla", "boom"] {
            assert!(Compatibility::parse(bad).is_err(), "{bad}");
        }
        assert!(
            Compatibility::parse("vanilla")
                .unwrap_err()
                .to_string()
                .contains("neither `auto` nor a reviewed"),
            "the message must explain the accepted forms"
        );
        assert_eq!(Compatibility::Auto.level(), None);
        assert_eq!(
            Compatibility::Level(Complevel::MBF)
                .level()
                .map(Complevel::value),
            Some(11)
        );
    }

    #[test]
    fn compatibility_round_trips_through_json_and_rejects_placeholders() {
        for value in [Compatibility::Auto, Compatibility::Level(Complevel::MBF21)] {
            let text = serde_json::to_string(&value).unwrap();
            assert_eq!(serde_json::from_str::<Compatibility>(&text).unwrap(), value);
        }
        assert!(serde_json::from_str::<Compatibility>("\"18\"").is_err());
        assert!(serde_json::from_str::<Compatibility>("18").is_err());
        assert!(serde_json::from_str::<Compatibility>("\"boom\"").is_err());
    }

    #[test]
    fn files_route_to_their_group_and_duplicates_are_refused() {
        let mut setup = setup();
        setup.add(file("maps.wad", Kind::Wad)).unwrap();
        setup.add(file("data.zip", Kind::Zip)).unwrap();
        setup.add(file("fix.deh", Kind::Deh)).unwrap();
        assert_eq!(setup.files.len(), 2);
        assert_eq!(setup.patches.len(), 1);
        assert!(matches!(
            setup.add(file("maps.wad", Kind::Wad)),
            Err(SetupError::Duplicate(_))
        ));
    }

    #[test]
    fn move_and_remove_only_reorder_or_drop_the_record() {
        let mut setup = setup();
        setup.add(file("a.wad", Kind::Wad)).unwrap();
        setup.add(file("b.wad", Kind::Wad)).unwrap();
        setup.add(file("c.wad", Kind::Wad)).unwrap();
        setup.move_entry(Group::Files, 0, 2).unwrap();
        let order: Vec<_> = setup
            .files
            .iter()
            .map(|f| f.path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(order, ["b.wad", "c.wad", "a.wad"]);
        assert!(matches!(
            setup.move_entry(Group::Files, 0, 9),
            Err(SetupError::OutOfRange { .. })
        ));
        let removed = setup.remove(Group::Files, 0).unwrap();
        assert_eq!(removed.path, PathBuf::from("/games/b.wad"));
        assert!(matches!(
            setup.remove(Group::Patches, 0),
            Err(SetupError::OutOfRange { .. })
        ));
    }

    #[test]
    fn validate_checks_schema_id_name_groups_and_absolute_paths() {
        assert!(setup().validate().is_ok());

        let mut bad = setup();
        bad.name = "  ".to_owned();
        assert!(matches!(bad.validate(), Err(SetupError::Invalid(_))));

        let mut bad = setup();
        bad.id = "../escape".to_owned();
        assert!(matches!(bad.validate(), Err(SetupError::UnsafeId(_))));

        let mut bad = setup();
        bad.patches.push(file("oops.wad", Kind::Wad));
        assert!(matches!(bad.validate(), Err(SetupError::WrongKind { .. })));

        let mut bad = setup();
        bad.schema_version = 9;
        assert!(matches!(
            bad.validate(),
            Err(SetupError::UnsupportedSchema(9))
        ));
    }

    #[test]
    fn ids_are_safe_lowercase_and_slugs_are_derived_from_names() {
        assert!(safe_id("my-setup-2"));
        assert!(!safe_id("My Setup"));
        assert!(!safe_id(""));
        assert!(!safe_id("a/b"));
        assert_eq!(slug("My First Setup!"), "my-first-setup");
        assert_eq!(slug("   "), "setup");
        assert_eq!(slug("SIGIL II"), "sigil-ii");
    }
}

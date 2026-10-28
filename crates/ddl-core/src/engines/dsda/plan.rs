//! The DSDA-Doom launch contract: a typed description of "what to play"
//! (decision D-011) and the argv it produces.
//!
//! [`LaunchPlan`] is the single choke point through which every game launch
//! passes. It is deliberately narrow:
//!
//! * values that the engine parser would accept but DDL does not review
//!   (compatibility placeholders `18`-`20`, the words `vanilla`/`boom`/`mbf`)
//!   are **not representable** ([`Complevel`], [`Skill`], [`Warp`]);
//! * there is exactly one ordered PWAD list, one ordered patch list and one
//!   `-assign` list, so DSDA's right-to-left "last occurrence wins" rule can
//!   never silently drop an entry (see `docs/research/dsda-engine.md` §1);
//! * [`LaunchPlan::validate`] rejects layouts DSDA would misinterpret, and
//!   [`LaunchPlan::build_command`] runs it before handing out a
//!   [`Command`].
//!
//! Every engine fact below is taken from `docs/research/dsda-engine.md`
//! (upstream tag `v0.30.1`); the module adds no CLI flag the research does not
//! document.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A DSDA-Doom compatibility level that DDL is willing to emit.
///
/// Only the reviewed numbers are representable: `0`-`17` and `21` (MBF21).
/// Upstream's parser also accepts `-1` and the placeholder values `18`-`20`,
/// which are not a compatibility family and so cannot reach a launch plan.
///
/// A level can carry the DSDA word that names it ([`ComplevelWord`], D-030):
/// the plan then sends the word to `-complevel` and keeps the resolved number
/// for display. A level without a word is sent as its number.
///
/// Source: `docs/research/dsda-engine.md` § "Compatibility values and
/// omission" (confirmed against upstream tag `v0.30.1`,
/// `prboom2/src/doomtype.h:113-140`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Complevel {
    value: u8,
    word: Option<ComplevelWord>,
}

/// The `-complevel` words DSDA-Doom v0.30.1 accepts, case-insensitively.
///
/// `boom` is 9, `mbf` is 11 and `mbf21` is 21; `vanilla` resolves to 4 for
/// TNT/Plutonia, 2 for other commercial Doom II games and otherwise 3.
///
/// Source: `docs/research/dsda-engine.md` §1 (confirmed against upstream tag
/// `v0.30.1`, `prboom2/src/dsda/settings.c:60-170`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComplevelWord {
    /// `vanilla`: 2, 3 or 4 depending on the IWAD.
    Vanilla,
    /// `boom`: 9.
    Boom,
    /// `mbf`: 11.
    Mbf,
    /// `mbf21`: 21.
    Mbf21,
}

impl ComplevelWord {
    /// The word as DSDA-Doom receives it on the command line.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::Boom => "boom",
            Self::Mbf => "mbf",
            Self::Mbf21 => "mbf21",
        }
    }

    /// The word as the UI spells it (`Vanilla`, `Boom`, `MBF`, `MBF21`).
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::Boom => "Boom",
            Self::Mbf => "MBF",
            Self::Mbf21 => "MBF21",
        }
    }
}

impl Complevel {
    /// Doom 1.2.
    pub const DOOM_1_2: Self = Self::from_reviewed(0);
    /// Doom 1.666.
    pub const DOOM_1_666: Self = Self::from_reviewed(1);
    /// Doom / Doom II 1.9.
    pub const DOOM_1_9: Self = Self::from_reviewed(2);
    /// Ultimate Doom / Doom95.
    pub const ULTIMATE_DOOM: Self = Self::from_reviewed(3);
    /// Final Doom.
    pub const FINAL_DOOM: Self = Self::from_reviewed(4);
    /// Boom compatibility mode.
    pub const BOOM: Self = Self::from_reviewed(7);
    /// Boom 2.02 (the Boom 2.02 release level, distinct from 2.01).
    pub const BOOM_2_02: Self = Self::from_reviewed(9);
    /// MBF.
    pub const MBF: Self = Self::from_reviewed(11);
    /// Latest PrBoom.
    pub const PRBOOM_LATEST: Self = Self::from_reviewed(17);
    /// MBF21.
    pub const MBF21: Self = Self::from_reviewed(21);

    /// Builds a compatibility level from a reviewed number.
    ///
    /// # Errors
    ///
    /// Returns [`ValueRangeError::Complevel`] for anything outside `0`-`17` and
    /// `21`, including the placeholders `18`-`20`.
    pub fn new(value: u8) -> Result<Self, ValueRangeError> {
        match value {
            0..=17 | 21 => Ok(Self::from_reviewed(value)),
            other => Err(ValueRangeError::Complevel(other)),
        }
    }

    /// Const constructor used by the named constants; callers must pass a
    /// reviewed value (the compiler cannot check that, the constants above and
    /// [`Complevel::new`] are the only call sites).
    const fn from_reviewed(value: u8) -> Self {
        Self { value, word: None }
    }

    /// Builds the level a DSDA word names. `vanilla_value` is what `vanilla`
    /// resolves to for the identified IWAD (2, 3 or 4); the other words have
    /// one fixed number.
    ///
    /// # Errors
    ///
    /// Returns [`ValueRangeError::Complevel`] when the resolved number is not a
    /// reviewed value.
    pub fn from_word(word: ComplevelWord, vanilla_value: u8) -> Result<Self, ValueRangeError> {
        let value = match word {
            ComplevelWord::Vanilla => vanilla_value,
            ComplevelWord::Boom => 9,
            ComplevelWord::Mbf => 11,
            ComplevelWord::Mbf21 => 21,
        };
        Self::new(value).map(|level| Self {
            word: Some(word),
            ..level
        })
    }

    /// The number this level resolves to.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.value
    }

    /// The DSDA word that names this level, when it was built from one.
    #[must_use]
    pub const fn word(self) -> Option<ComplevelWord> {
        self.word
    }

    /// What DSDA-Doom receives after `-complevel`: the word when there is one,
    /// otherwise the number.
    #[must_use]
    pub fn argument(self) -> String {
        match self.word {
            Some(word) => word.as_str().to_owned(),
            None => self.value.to_string(),
        }
    }

    /// The player-facing label: `Vanilla (2)`, `Boom (9)`, `MBF21 (21)`, or the
    /// bare number when no word names the level (D-030).
    #[must_use]
    pub fn label(self) -> String {
        match self.word {
            Some(word) => format!("{} ({})", word.title(), self.value),
            None => self.value.to_string(),
        }
    }

    /// The compatibility family this number selects.
    ///
    /// Transcribed from the enum table in `docs/research/dsda-engine.md` §
    /// "Compatibility values and omission".
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self.value {
            0 => "Doom 1.2",
            1 => "Doom 1.666",
            2 => "Doom/Doom II 1.9",
            3 => "Ultimate Doom/Doom95",
            4 => "Final Doom",
            5 => "DosDoom 0.47",
            6 => "TASDoom",
            7 => "Boom compatibility mode",
            8 => "Boom 2.01",
            9 => "Boom 2.02",
            10 => "LxDoom 1.3.2+",
            11 => "MBF",
            12 => "PrBoom 2.03beta",
            13 => "PrBoom 2.1.0-2.1.1",
            14 => "PrBoom 2.2.x",
            15 => "PrBoom 2.3.x",
            16 => "PrBoom 2.4.0",
            17 => "latest PrBoom",
            21 => "MBF21",
            // Unreachable: `Complevel` can only hold a reviewed value.
            _ => "unreviewed compatibility level",
        }
    }
}

impl TryFrom<u8> for Complevel {
    type Error = ValueRangeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Display for Complevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.word {
            Some(word) => write!(f, "{} ({})", word.as_str(), self.value),
            None => write!(f, "{} ({})", self.value, self.name()),
        }
    }
}

/// A DSDA-Doom starting skill.
///
/// Upstream's parser accepts `1`-`255`, but the built-in skill inventory has
/// five entries and setup clamps to the highest built-in skill, so DDL only
/// represents `1`-`5` (skill 1 = `-skill 1` … skill 5 = Nightmare).
///
/// Source: `docs/research/dsda-engine.md` §1 `-skill n` (confirmed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Skill(u8);

impl Skill {
    /// Builds a skill from the engine's accepted range.
    ///
    /// # Errors
    ///
    /// Returns [`ValueRangeError::Skill`] for anything outside `1`-`5`.
    pub fn new(value: u8) -> Result<Self, ValueRangeError> {
        match value {
            1..=5 => Ok(Self(value)),
            other => Err(ValueRangeError::Skill(other)),
        }
    }

    /// The number DSDA-Doom receives on the command line.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Skill {
    type Error = ValueRangeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The `-warp` target of a launch.
///
/// DSDA-Doom accepts zero to two integers, each `0`-`99`, and autostarts the
/// game at that point. Which of the two numbers means what depends on the game:
///
/// * **Doom II** (and other commercial games): the first number is the **map**
///   and a second number is ignored — use [`Warp::Single`].
/// * **Doom / Ultimate Doom**: the first number is the **episode** and the
///   second the map, which defaults to episode map 1 — use [`Warp::Single`] for
///   "episode with its default map", [`Warp::EpisodeMap`] to choose the map too.
///
/// Source: `docs/research/dsda-engine.md` §1 `-warp [n [m]]` (confirmed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Warp {
    /// `-warp n`: a Doom II map, or a Doom episode with its default map.
    Single(u8),
    /// `-warp episode map`: a Doom / Ultimate Doom episode and map.
    EpisodeMap(u8, u8),
}

impl Warp {
    /// A single `-warp n` value.
    ///
    /// # Errors
    ///
    /// Returns [`ValueRangeError::Warp`] when `value` is above `99`.
    pub fn single(value: u8) -> Result<Self, ValueRangeError> {
        check_warp(value)?;
        Ok(Self::Single(value))
    }

    /// A `-warp episode map` pair.
    ///
    /// # Errors
    ///
    /// Returns [`ValueRangeError::Warp`] when either value is above `99`.
    pub fn episode_map(episode: u8, map: u8) -> Result<Self, ValueRangeError> {
        check_warp(episode)?;
        check_warp(map)?;
        Ok(Self::EpisodeMap(episode, map))
    }
}

fn check_warp(value: u8) -> Result<(), ValueRangeError> {
    if value <= 99 {
        Ok(())
    } else {
        Err(ValueRangeError::Warp(value))
    }
}

impl fmt::Display for Warp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(value) => write!(f, "{value}"),
            Self::EpisodeMap(episode, map) => write!(f, "{episode} {map}"),
        }
    }
}

/// A transient screen resolution for one launch (`-width` / `-height`).
///
/// DDL passes the drawable size of its own fullscreen window, so DSDA-Doom
/// renders for the actual display (the Deck panel or an external TV) instead of
/// a fixed size. Both flags are ordinary integers with the upstream ranges
/// `-width` ≥ 320 and `-height` ≥ 200
/// ([`args.c:536-545`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L536-L545));
/// they are read in `I_InitScreenResolution` and only change the current
/// session
/// ([`i_video.c:1050-1062`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_video.c#L1050-L1062)).
///
/// The constructor keeps the value bounded, so an absurd window size cannot
/// reach the engine. A display whose drawable size is unavailable simply keeps
/// the config-file fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenSize {
    width: u32,
    height: u32,
}

impl ScreenSize {
    /// The smallest width DSDA-Doom accepts for `-width`.
    pub const MIN_WIDTH: u32 = 320;
    /// The smallest height DSDA-Doom accepts for `-height`.
    pub const MIN_HEIGHT: u32 = 200;
    /// DDL's ceiling for a plausible display dimension.
    pub const MAX_DIMENSION: u32 = 16_384;

    /// Builds a bounded screen size.
    ///
    /// # Errors
    ///
    /// Returns [`ValueRangeError::Width`] or [`ValueRangeError::Height`] when a
    /// dimension is outside DSDA-Doom's accepted range or DDL's sane ceiling.
    pub fn new(width: u32, height: u32) -> Result<Self, ValueRangeError> {
        if !(Self::MIN_WIDTH..=Self::MAX_DIMENSION).contains(&width) {
            return Err(ValueRangeError::Width(width));
        }
        if !(Self::MIN_HEIGHT..=Self::MAX_DIMENSION).contains(&height) {
            return Err(ValueRangeError::Height(height));
        }
        Ok(Self { width, height })
    }

    /// The value sent to `-width`.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// The value sent to `-height`.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }
}

impl fmt::Display for ScreenSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

/// A value outside the range DDL accepts for an engine option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueRangeError {
    /// `-complevel`: reviewed values are `0`-`17` and `21`.
    Complevel(u8),
    /// `-skill`: DDL accepts `1`-`5`.
    Skill(u8),
    /// `-warp`: each value is `0`-`99`.
    Warp(u8),
    /// `-width`: DSDA-Doom accepts at least `320`; DDL bounds the top too.
    Width(u32),
    /// `-height`: DSDA-Doom accepts at least `200`; DDL bounds the top too.
    Height(u32),
}

impl fmt::Display for ValueRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complevel(value) => write!(
                f,
                "compatibility level {value} is not a reviewed DSDA-Doom compatibility value \
                 (0-17 or 21)"
            ),
            Self::Skill(value) => {
                write!(f, "skill {value} is outside DDL's accepted range 1-5")
            }
            Self::Warp(value) => {
                write!(
                    f,
                    "warp value {value} is outside DSDA-Doom's accepted range 0-99"
                )
            }
            Self::Width(value) => write!(
                f,
                "screen width {value} is outside DDL's accepted range {}-{}",
                ScreenSize::MIN_WIDTH,
                ScreenSize::MAX_DIMENSION
            ),
            Self::Height(value) => write!(
                f,
                "screen height {value} is outside DDL's accepted range {}-{}",
                ScreenSize::MIN_HEIGHT,
                ScreenSize::MAX_DIMENSION
            ),
        }
    }
}

impl std::error::Error for ValueRangeError {}

/// One transient `-assign key=value` config override (decision D-012).
///
/// The key must match `[a-z0-9_]+` and appear in the allowlist the caller
/// passes to [`LaunchPlan::validate`]; neither the key nor the value may start
/// with `-`, which the engine parser would read as the next option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignOverride {
    /// The DSDA-Doom config key, e.g. `dsda_fps`.
    pub key: String,
    /// The value as it appears after `=`.
    pub value: String,
}

impl AssignOverride {
    /// Builds an override. Validation happens in [`LaunchPlan::validate`],
    /// because the allowed keys are caller-supplied.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl fmt::Display for AssignOverride {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

/// The set of `-assign` keys the caller considers safe.
///
/// DDL only ever forwards owned, reviewed per-profile scalars (decision D-012),
/// so an unknown key is a bug or an injection attempt rather than a feature.
/// The engine itself rejects unknown CLI assignments, but by then the process
/// has already started.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AssignAllowlist {
    keys: BTreeSet<String>,
}

impl AssignAllowlist {
    /// An empty allowlist, which rejects every override.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// An allowlist built from `keys`.
    pub fn with_keys<I, S>(keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            keys: keys.into_iter().map(Into::into).collect(),
        }
    }

    /// Adds one key.
    pub fn insert(&mut self, key: impl Into<String>) {
        self.keys.insert(key.into());
    }

    /// Whether `key` is allowed.
    #[must_use]
    pub fn contains(&self, key: &str) -> bool {
        self.keys.contains(key)
    }
}

impl fmt::Display for AssignAllowlist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.keys.iter().cloned().collect::<Vec<_>>().join(", ")
        )
    }
}

/// Which field of a [`LaunchPlan`] a path problem is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathField {
    /// The engine executable.
    Executable,
    /// The IWAD (normally DDL's canonical-name symlink, decision D-017).
    Iwad,
    /// A PWAD/ZIP in the `-file` list.
    File,
    /// A DEH/BEX patch in the `-deh` list.
    Patch,
    /// The demo recording path, or its parent directory.
    Record,
    /// The config file path, or its parent directory.
    Config,
    /// The per-profile save directory.
    SaveDir,
    /// The per-profile data directory.
    DataDir,
    /// The per-profile screenshot directory.
    ShotDir,
    /// The working directory the engine starts in.
    WorkingDir,
}

impl fmt::Display for PathField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Executable => "engine executable",
            Self::Iwad => "IWAD",
            Self::File => "PWAD/ZIP file",
            Self::Patch => "DEH/BEX patch",
            Self::Record => "demo recording path or its parent directory",
            Self::Config => "config file path or its parent directory",
            Self::SaveDir => "save directory",
            Self::DataDir => "data directory",
            Self::ShotDir => "screenshot directory",
            Self::WorkingDir => "working directory",
        })
    }
}

/// Why a [`LaunchPlan`] cannot be turned into a command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A path looks like another option (`-…`), so the engine parser would
    /// stop reading the current value and treat it as a flag.
    PathLooksLikeOption {
        /// The field the offending path belongs to.
        field: PathField,
        /// The offending path.
        path: PathBuf,
    },
    /// A path is not absolute. DSDA-Doom resolves relative paths through its
    /// own search policy; DDL never relies on that.
    PathNotAbsolute {
        /// The field the offending path belongs to.
        field: PathField,
        /// The offending path.
        path: PathBuf,
    },
    /// An input file does not exist.
    MissingFile {
        /// The field the missing file belongs to.
        field: PathField,
        /// The missing file.
        path: PathBuf,
    },
    /// An input path exists but is not a regular file (a directory, socket or
    /// device).
    NotRegularFile {
        /// The field the offending path belongs to.
        field: PathField,
        /// The offending path.
        path: PathBuf,
    },
    /// A required directory does not exist; call [`LaunchPlan::prepare_dirs`].
    /// DSDA-Doom does not create `-save`, `-data`, `-shotdir` or the parent of
    /// an arbitrary `-config`.
    DirectoryMissing {
        /// The field the missing directory belongs to.
        field: PathField,
        /// The missing directory.
        path: PathBuf,
    },
    /// A required directory path exists but is not a directory.
    NotADirectory {
        /// The field the offending path belongs to.
        field: PathField,
        /// The offending path.
        path: PathBuf,
    },
    /// An `-assign` key contains characters outside `[a-z0-9_]`.
    AssignKeyInvalid {
        /// The offending key.
        key: String,
    },
    /// An `-assign` key is not in the caller-supplied allowlist.
    AssignKeyNotAllowed {
        /// The offending key.
        key: String,
    },
    /// An `-assign` value looks like another option (`-…`).
    AssignValueLooksLikeOption {
        /// The key the value belongs to.
        key: String,
        /// The offending value.
        value: String,
    },
    /// DSDA-Doom cannot record a demo and reset to a pistol start on every
    /// map: recording rejects `-pistolstart` outright.
    PistolStartWithRecord,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathLooksLikeOption { field, path } => write!(
                f,
                "{field} path \"{}\" starts with '-' and would be parsed as an option",
                path.display()
            ),
            Self::PathNotAbsolute { field, path } => {
                write!(f, "{field} path \"{}\" is not absolute", path.display())
            }
            Self::MissingFile { field, path } => {
                write!(f, "{field} \"{}\" does not exist", path.display())
            }
            Self::NotRegularFile { field, path } => {
                write!(f, "{field} \"{}\" is not a regular file", path.display())
            }
            Self::DirectoryMissing { field, path } => write!(
                f,
                "{field} \"{}\" does not exist; create it with prepare_dirs() first",
                path.display()
            ),
            Self::NotADirectory { field, path } => {
                write!(f, "{field} \"{}\" is not a directory", path.display())
            }
            Self::AssignKeyInvalid { key } => {
                write!(f, "assign key \"{key}\" does not match [a-z0-9_]+")
            }
            Self::AssignKeyNotAllowed { key } => {
                write!(f, "assign key \"{key}\" is not in the caller's allowlist")
            }
            Self::AssignValueLooksLikeOption { key, value } => write!(
                f,
                "value \"{value}\" of assign \"{key}\" starts with '-' and would be parsed \
                 as an option"
            ),
            Self::PistolStartWithRecord => f.write_str(
                "pistol start (-pistolstart) cannot be combined with demo recording (-record)",
            ),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Everything DSDA-Doom needs to start a reviewed game.
///
/// Paths are absolute; see [`LaunchPlan::validate`] for the checks
/// [`LaunchPlan::build_command`] enforces before the engine starts.
///
/// The list fields are the *whole* list for their option. There is no API that
/// appends a second `-file`, `-deh`, `-iwad` or `-assign` flag, which is what
/// makes "DSDA keeps only the last occurrence" unreachable through this type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    /// The engine executable (the AppImage, or `AppRun` after extraction).
    pub executable: PathBuf,
    /// The IWAD, normally DDL's canonical-name symlink (decision D-017).
    pub iwad: PathBuf,
    /// Ordered PWAD/ZIP files for the single `-file` flag.
    pub files: Vec<PathBuf>,
    /// Ordered DEH/BEX patches for the single `-deh` flag.
    pub patches: Vec<PathBuf>,
    /// The compatibility level sent as `-complevel`.
    ///
    /// `None` emits no `-complevel` at all, so the engine decides (a custom
    /// setup's **Auto**; spec 026, R2/R3). Reviewed catalog mapsets always pin
    /// one.
    pub complevel: Option<Complevel>,
    /// Optional explicit skill for recording; ordinary launches leave this unset.
    pub skill: Option<Skill>,
    /// Optional `-warp` target.
    pub warp: Option<Warp>,
    /// Whether to reset to a pistol start on every map (`-pistolstart`).
    pub pistol_start: bool,
    /// Optional `-record` demo output path.
    pub record: Option<PathBuf>,
    /// The DDL-owned config **file** sent as `-config` (decision D-012).
    pub config: PathBuf,
    /// Per-profile save directory (`-save`).
    pub save_dir: PathBuf,
    /// Per-profile data directory (`-data`).
    pub data_dir: PathBuf,
    /// Per-profile screenshot directory (`-shotdir`).
    pub shot_dir: PathBuf,
    /// The working directory the engine starts in.
    pub working_dir: PathBuf,
    /// The transient drawable resolution (`-width` / `-height`, spec 064 R6).
    ///
    /// `None` keeps DSDA's config-file `screen_resolution`, which is the
    /// fallback for a windowed UI or an unavailable measurement.
    pub screen: Option<ScreenSize>,
    /// Ordered transient config overrides for the single `-assign` flag.
    pub assigns: Vec<AssignOverride>,
}

impl LaunchPlan {
    /// The argv DSDA-Doom receives, in the order fixed by decision D-011:
    /// `-iwad`, `-file`, `-deh`, `-complevel`, `-skill`, `-warp`, `-pistolstart`,
    /// `-record`, `-noautoload`, `-config`, `-save`, `-data`, `-shotdir`,
    /// `-width`, `-height`, `-assign`.
    ///
    /// Each value is its own token. `-file`, `-deh` and `-assign` appear at
    /// most once, followed by all of their values, because DSDA's parser keeps
    /// only the last occurrence of a repeated flag.
    ///
    /// This method does not validate. Use [`LaunchPlan::build_command`] to get a
    /// command that is guaranteed valid, or call [`LaunchPlan::validate`] first.
    #[must_use]
    pub fn to_command(&self) -> Vec<OsString> {
        let mut argv = Vec::with_capacity(6 + self.files.len() + self.patches.len());
        argv.push(OsString::from("-iwad"));
        argv.push(self.iwad.clone().into_os_string());

        if !self.files.is_empty() {
            argv.push(OsString::from("-file"));
            argv.extend(self.files.iter().map(|path| path.clone().into_os_string()));
        }
        if !self.patches.is_empty() {
            argv.push(OsString::from("-deh"));
            argv.extend(
                self.patches
                    .iter()
                    .map(|path| path.clone().into_os_string()),
            );
        }

        if let Some(complevel) = self.complevel {
            argv.push(OsString::from("-complevel"));
            argv.push(OsString::from(complevel.argument()));
        }
        if let Some(skill) = self.skill {
            argv.push(OsString::from("-skill"));
            argv.push(OsString::from(skill.value().to_string()));
        }

        if let Some(warp) = self.warp {
            argv.push(OsString::from("-warp"));
            match warp {
                Warp::Single(value) => argv.push(OsString::from(value.to_string())),
                Warp::EpisodeMap(episode, map) => {
                    argv.push(OsString::from(episode.to_string()));
                    argv.push(OsString::from(map.to_string()));
                }
            }
        }
        if self.pistol_start {
            argv.push(OsString::from("-pistolstart"));
        }
        if let Some(record) = &self.record {
            argv.push(OsString::from("-record"));
            argv.push(record.clone().into_os_string());
        }

        // Always emitted: reviewed recipes must not depend on directory
        // autoload or a COMPLVL lump (decision D-011, research §1).
        argv.push(OsString::from("-noautoload"));
        argv.push(OsString::from("-config"));
        argv.push(self.config.clone().into_os_string());
        argv.push(OsString::from("-save"));
        argv.push(self.save_dir.clone().into_os_string());
        argv.push(OsString::from("-data"));
        argv.push(self.data_dir.clone().into_os_string());
        argv.push(OsString::from("-shotdir"));
        argv.push(self.shot_dir.clone().into_os_string());

        // Spec 064 R6: the transient drawable resolution of DDL's fullscreen
        // window. DSDA reads both flags in I_InitScreenResolution, so the
        // persisted `screen_resolution` remains the fallback.
        if let Some(screen) = self.screen {
            argv.push(OsString::from("-width"));
            argv.push(OsString::from(screen.width().to_string()));
            argv.push(OsString::from("-height"));
            argv.push(OsString::from(screen.height().to_string()));
        }

        if !self.assigns.is_empty() {
            argv.push(OsString::from("-assign"));
            argv.extend(
                self.assigns
                    .iter()
                    .map(|assign| OsString::from(assign.to_string())),
            );
        }

        argv
    }

    /// Validates the plan against the filesystem and `assign_allowlist`, then
    /// returns a [`Command`] with the program, arguments and working directory
    /// set.
    ///
    /// # Errors
    ///
    /// Returns the [`ValidationError`] [`LaunchPlan::validate`] would return.
    pub fn build_command(
        &self,
        assign_allowlist: &AssignAllowlist,
    ) -> Result<Command, ValidationError> {
        self.validate(assign_allowlist)?;
        let mut command = Command::new(&self.executable);
        command.args(self.to_command());
        command.current_dir(&self.working_dir);
        Ok(command)
    }

    /// Checks everything that can make a launch fail before the process starts.
    ///
    /// Cheap checks run first: pistol start plus recording, then the `-assign`
    /// list, then the paths. Directories must already exist; call
    /// [`LaunchPlan::prepare_dirs`] first. The config file and the demo
    /// recording file are outputs and need not exist, but their parent
    /// directories must (the engine does not create them).
    ///
    /// # Errors
    ///
    /// Returns the first [`ValidationError`] found.
    pub fn validate(&self, assign_allowlist: &AssignAllowlist) -> Result<(), ValidationError> {
        if self.pistol_start && self.record.is_some() {
            return Err(ValidationError::PistolStartWithRecord);
        }
        for assign in &self.assigns {
            validate_assign(assign, assign_allowlist)?;
        }

        check_regular_file(PathField::Executable, &self.executable)?;
        check_regular_file(PathField::Iwad, &self.iwad)?;
        for file in &self.files {
            check_regular_file(PathField::File, file)?;
        }
        for patch in &self.patches {
            check_regular_file(PathField::Patch, patch)?;
        }
        if let Some(record) = &self.record {
            check_token(PathField::Record, record)?;
            check_directory(PathField::Record, parent_or_self(record))?;
        }
        check_token(PathField::Config, &self.config)?;
        check_directory(PathField::Config, parent_or_self(&self.config))?;

        check_directory(PathField::SaveDir, &self.save_dir)?;
        check_directory(PathField::DataDir, &self.data_dir)?;
        check_directory(PathField::ShotDir, &self.shot_dir)?;
        check_directory(PathField::WorkingDir, &self.working_dir)?;
        Ok(())
    }

    /// Creates the directories DSDA-Doom needs but does not create itself: the
    /// per-profile working, save, data and screenshot directories, the config
    /// file's parent, and the demo recording's parent.
    ///
    /// Idempotent, and safe to call before [`LaunchPlan::validate`].
    ///
    /// # Errors
    ///
    /// Returns the first [`io::Error`] from `create_dir_all`, annotated with
    /// the directory that failed.
    pub fn prepare_dirs(&self) -> io::Result<()> {
        let mut dirs: Vec<&Path> = vec![
            &self.working_dir,
            &self.save_dir,
            &self.data_dir,
            &self.shot_dir,
        ];
        dirs.push(parent_or_self(&self.config));
        if let Some(record) = &self.record {
            dirs.push(parent_or_self(record));
        }
        for dir in dirs {
            fs::create_dir_all(dir)
                .map_err(|err| io::Error::new(err.kind(), format!("{}: {err}", dir.display())))?;
        }
        Ok(())
    }

    /// A human-readable summary for the UI's "Will launch" preview and for
    /// diagnostics.
    ///
    /// Paths under `home` are shortened to `~/…`, so a summary exported to a
    /// log does not leak the user's home directory. `home` is a parameter
    /// because [`crate::platform::paths`] deliberately does not expose `HOME`.
    #[must_use]
    pub fn describe(&self, home: &Path) -> String {
        let mut out = String::from("Launch plan\n");
        out.push_str(&format!(
            "  executable: {}\n",
            shorten(&self.executable, home)
        ));
        out.push_str(&format!("  IWAD: {}\n", shorten(&self.iwad, home)));
        push_path_list(&mut out, "files (load order)", &self.files, home);
        push_path_list(&mut out, "patches", &self.patches, home);
        match self.complevel {
            Some(complevel) => out.push_str(&format!("  complevel: {complevel}\n")),
            None => out.push_str("  complevel: (engine default)\n"),
        }
        if let Some(skill) = self.skill {
            out.push_str(&format!("  skill: {skill}\n"));
        }
        match self.warp {
            Some(warp) => out.push_str(&format!("  warp: {warp}\n")),
            None => out.push_str("  warp: (none)\n"),
        }
        out.push_str(&format!(
            "  pistol start: {}\n",
            if self.pistol_start { "yes" } else { "no" }
        ));
        match &self.record {
            Some(record) => out.push_str(&format!("  record: {}\n", shorten(record, home))),
            None => out.push_str("  record: (none)\n"),
        }
        out.push_str(&format!("  config: {}\n", shorten(&self.config, home)));
        out.push_str(&format!("  save: {}\n", shorten(&self.save_dir, home)));
        out.push_str(&format!("  data: {}\n", shorten(&self.data_dir, home)));
        out.push_str(&format!("  shotdir: {}\n", shorten(&self.shot_dir, home)));
        out.push_str(&format!(
            "  screen: {}\n",
            self.screen.map_or_else(
                || String::from("(config fallback)"),
                |screen| screen.to_string()
            )
        ));
        out.push_str(&format!(
            "  working dir: {}\n",
            shorten(&self.working_dir, home)
        ));
        if self.assigns.is_empty() {
            out.push_str("  assign: (none)");
        } else {
            out.push_str("  assign:");
            for assign in &self.assigns {
                out.push_str(&format!("\n    {assign}"));
            }
        }
        out
    }
}

/// Returns `path` itself when it has no parent (e.g. `/`), so a directory check
/// reports the path that was actually given.
fn parent_or_self(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => path,
    }
}

fn push_path_list(out: &mut String, label: &str, paths: &[PathBuf], home: &Path) {
    if paths.is_empty() {
        out.push_str(&format!("  {label}: (none)\n"));
    } else {
        out.push_str(&format!("  {label}:"));
        for path in paths {
            out.push_str(&format!("\n    {}", shorten(path, home)));
        }
        out.push('\n');
    }
}

/// Shortens `path` to `~`/`~/…` when it is `home` or below it.
fn shorten(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => String::from("~"),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

/// A value must not look like another option: DSDA's parser stops reading the
/// current option's values at the next token starting with `-`.
fn looks_like_option(path: &Path) -> bool {
    path.as_os_str().to_string_lossy().starts_with('-')
}

fn check_token(field: PathField, path: &Path) -> Result<(), ValidationError> {
    if looks_like_option(path) {
        return Err(ValidationError::PathLooksLikeOption {
            field,
            path: path.to_path_buf(),
        });
    }
    if !path.is_absolute() {
        return Err(ValidationError::PathNotAbsolute {
            field,
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn check_regular_file(field: PathField, path: &Path) -> Result<(), ValidationError> {
    check_token(field, path)?;
    match fs::metadata(path) {
        Ok(meta) if meta.is_file() => Ok(()),
        Ok(_) => Err(ValidationError::NotRegularFile {
            field,
            path: path.to_path_buf(),
        }),
        Err(_) => Err(ValidationError::MissingFile {
            field,
            path: path.to_path_buf(),
        }),
    }
}

fn check_directory(field: PathField, path: &Path) -> Result<(), ValidationError> {
    check_token(field, path)?;
    match fs::metadata(path) {
        Ok(meta) if meta.is_dir() => Ok(()),
        Ok(_) => Err(ValidationError::NotADirectory {
            field,
            path: path.to_path_buf(),
        }),
        Err(_) => Err(ValidationError::DirectoryMissing {
            field,
            path: path.to_path_buf(),
        }),
    }
}

fn validate_assign(
    assign: &AssignOverride,
    allowlist: &AssignAllowlist,
) -> Result<(), ValidationError> {
    let valid_chars = !assign.key.is_empty()
        && assign
            .key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !valid_chars {
        return Err(ValidationError::AssignKeyInvalid {
            key: assign.key.clone(),
        });
    }
    if !allowlist.contains(&assign.key) {
        return Err(ValidationError::AssignKeyNotAllowed {
            key: assign.key.clone(),
        });
    }
    if assign.value.starts_with('-') {
        return Err(ValidationError::AssignValueLooksLikeOption {
            key: assign.key.clone(),
            value: assign.value.clone(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The engine values and paths of a fully specified plan. The paths need
    /// not exist: argv and `describe` never touch the filesystem.
    fn sample_plan() -> LaunchPlan {
        LaunchPlan {
            executable: PathBuf::from("/opt/ddl/engines/dsda-doom.appimage"),
            iwad: PathBuf::from("/home/deck/.local/share/ddl/iwads/doom2/doom2.wad"),
            files: vec![
                PathBuf::from("/home/deck/.local/share/ddl/content/dbp37/DBP37_AUGZEN.wad"),
                PathBuf::from(
                    "/home/deck/.local/share/ddl/content/dbp37/addons/DBP37_DeckardBlaster.wad",
                ),
            ],
            patches: vec![PathBuf::from(
                "/home/deck/.local/share/ddl/content/dbp37/augzen.deh",
            )],
            complevel: Some(Complevel::DOOM_1_9),
            skill: Some(Skill::new(3).expect("3 is a valid skill")),
            warp: Some(Warp::single(1).expect("1 is a valid warp")),
            pistol_start: true,
            record: None,
            config: PathBuf::from("/home/deck/.config/ddl/dsda/doom2.cfg"),
            save_dir: PathBuf::from("/home/deck/.local/share/ddl/profiles/p1/save"),
            data_dir: PathBuf::from("/home/deck/.local/share/ddl/profiles/p1/data"),
            shot_dir: PathBuf::from("/home/deck/.local/share/ddl/profiles/p1/shots"),
            working_dir: PathBuf::from("/home/deck/.local/share/ddl/profiles/p1/run"),
            screen: None,
            assigns: Vec::new(),
        }
    }

    /// A minimal plan: single IWAD, no files, patches, warp or options.
    fn minimal_plan() -> LaunchPlan {
        LaunchPlan {
            files: Vec::new(),
            patches: Vec::new(),
            warp: None,
            pistol_start: false,
            ..sample_plan()
        }
    }

    /// Renders argv tokens as strings for readable snapshots.
    fn tokens(argv: &[OsString]) -> Vec<String> {
        argv.iter()
            .map(|token| token.to_string_lossy().into_owned())
            .collect()
    }

    fn count(tokens: &[String], flag: &str) -> usize {
        tokens.iter().filter(|token| token.as_str() == flag).count()
    }

    const HOME: &str = "/home/deck";

    #[test]
    fn minimal_plan_argv_order_is_exact() {
        let argv = tokens(&minimal_plan().to_command());
        assert_eq!(
            argv,
            [
                "-iwad",
                "/home/deck/.local/share/ddl/iwads/doom2/doom2.wad",
                "-complevel",
                "2",
                "-skill",
                "3",
                "-noautoload",
                "-config",
                "/home/deck/.config/ddl/dsda/doom2.cfg",
                "-save",
                "/home/deck/.local/share/ddl/profiles/p1/save",
                "-data",
                "/home/deck/.local/share/ddl/profiles/p1/data",
                "-shotdir",
                "/home/deck/.local/share/ddl/profiles/p1/shots",
            ]
        );
    }

    /// R6 (spec 064): a detected drawable size becomes two transient flags,
    /// after `-shotdir` and before `-assign`, never a single `WxH` string.
    #[test]
    fn a_detected_screen_emits_width_and_height_after_shotdir() {
        let plan = LaunchPlan {
            screen: Some(ScreenSize::new(1920, 1080).expect("a display size")),
            assigns: vec![AssignOverride::new("dsda_fps", "35")],
            ..minimal_plan()
        };
        let argv = tokens(&plan.to_command());
        let shotdir = argv
            .iter()
            .position(|token| token == "-shotdir")
            .expect("shotdir");
        assert_eq!(
            &argv[shotdir + 2..shotdir + 6],
            ["-width", "1920", "-height", "1080"]
        );
        assert_eq!(count(&argv, "-width"), 1);
        assert_eq!(count(&argv, "-height"), 1);
        assert_eq!(argv.last().map(String::as_str), Some("dsda_fps=35"));
        assert!(
            plan.describe(Path::new(HOME))
                .contains("  screen: 1920x1080\n")
        );
    }

    #[test]
    fn a_config_fallback_plan_emits_no_width_or_height() {
        let argv = tokens(&minimal_plan().to_command());
        assert!(!argv.iter().any(|token| token == "-width"), "{argv:?}");
        assert!(!argv.iter().any(|token| token == "-height"), "{argv:?}");
        assert!(
            minimal_plan()
                .describe(Path::new(HOME))
                .contains("  screen: (config fallback)\n")
        );
    }

    /// R6 (spec 064): sizes outside the engine's minimum, or DDL's ceiling, are
    /// rejected before they can reach argv.
    #[test]
    fn screen_sizes_are_bounded() {
        assert!(ScreenSize::new(320, 200).is_ok());
        assert!(ScreenSize::new(16_384, 16_384).is_ok());
        for (width, height) in [(319, 200), (320, 199), (0, 0), (16_385, 200), (320, 16_385)] {
            assert!(
                ScreenSize::new(width, height).is_err(),
                "{width}x{height} must be rejected"
            );
        }
        assert_eq!(
            ScreenSize::new(319, 200)
                .expect_err("too narrow")
                .to_string(),
            "screen width 319 is outside DDL's accepted range 320-16384"
        );
        assert_eq!(
            ScreenSize::new(320, 199)
                .expect_err("too short")
                .to_string(),
            "screen height 199 is outside DDL's accepted range 200-16384"
        );
    }

    #[test]
    fn multi_file_plan_argv_order_is_exact() {
        let argv = tokens(&sample_plan().to_command());
        assert_eq!(
            argv,
            [
                "-iwad",
                "/home/deck/.local/share/ddl/iwads/doom2/doom2.wad",
                "-file",
                "/home/deck/.local/share/ddl/content/dbp37/DBP37_AUGZEN.wad",
                "/home/deck/.local/share/ddl/content/dbp37/addons/DBP37_DeckardBlaster.wad",
                "-deh",
                "/home/deck/.local/share/ddl/content/dbp37/augzen.deh",
                "-complevel",
                "2",
                "-skill",
                "3",
                "-warp",
                "1",
                "-pistolstart",
                "-noautoload",
                "-config",
                "/home/deck/.config/ddl/dsda/doom2.cfg",
                "-save",
                "/home/deck/.local/share/ddl/profiles/p1/save",
                "-data",
                "/home/deck/.local/share/ddl/profiles/p1/data",
                "-shotdir",
                "/home/deck/.local/share/ddl/profiles/p1/shots",
            ]
        );
    }

    #[test]
    fn episode_and_map_warp_emits_two_tokens() {
        let plan = LaunchPlan {
            warp: Some(Warp::episode_map(1, 3).expect("values are in range")),
            ..minimal_plan()
        };
        let argv = tokens(&plan.to_command());
        let warp = argv
            .iter()
            .position(|token| token == "-warp")
            .expect("warp is present");
        assert_eq!(&argv[warp..warp + 3], ["-warp", "1", "3"]);
    }

    #[test]
    fn recording_plan_argv_order_is_exact() {
        let plan = LaunchPlan {
            pistol_start: false,
            record: Some(PathBuf::from(
                "/home/deck/.local/share/ddl/profiles/p1/demos/run1.lmp",
            )),
            ..minimal_plan()
        };
        let argv = tokens(&plan.to_command());
        let record = argv
            .iter()
            .position(|token| token == "-record")
            .expect("record is present");
        assert_eq!(
            &argv[record..record + 2],
            [
                "-record",
                "/home/deck/.local/share/ddl/profiles/p1/demos/run1.lmp"
            ]
        );
        assert!(!argv.iter().any(|token| token == "-pistolstart"));
    }

    #[test]
    fn assign_overrides_use_one_flag_followed_by_all_values() {
        let plan = LaunchPlan {
            assigns: vec![
                AssignOverride::new("dsda_fps", "35"),
                AssignOverride::new("autorun", "1"),
            ],
            ..minimal_plan()
        };
        let argv = tokens(&plan.to_command());
        let assign = argv
            .iter()
            .position(|token| token == "-assign")
            .expect("assign is present");
        assert_eq!(argv.len(), assign + 3);
        assert_eq!(&argv[assign..], ["-assign", "dsda_fps=35", "autorun=1"]);
    }

    #[test]
    fn repeated_lists_emit_each_flag_exactly_once() {
        // DSDA keeps only the last occurrence of a repeated flag, so the
        // impossibility of a second list is a correctness property, not a
        // style choice.
        let plan = LaunchPlan {
            assigns: vec![
                AssignOverride::new("dsda_fps", "35"),
                AssignOverride::new("autorun", "1"),
            ],
            ..sample_plan()
        };
        let argv = tokens(&plan.to_command());
        assert_eq!(count(&argv, "-iwad"), 1);
        assert_eq!(count(&argv, "-file"), 1);
        assert_eq!(count(&argv, "-deh"), 1);
        assert_eq!(count(&argv, "-complevel"), 1);
        assert_eq!(count(&argv, "-skill"), 1);
        assert_eq!(count(&argv, "-warp"), 1);
        assert_eq!(count(&argv, "-pistolstart"), 1);
        assert_eq!(count(&argv, "-noautoload"), 1);
        assert_eq!(count(&argv, "-config"), 1);
        assert_eq!(count(&argv, "-save"), 1);
        assert_eq!(count(&argv, "-data"), 1);
        assert_eq!(count(&argv, "-shotdir"), 1);
        assert_eq!(count(&argv, "-assign"), 1);
    }

    #[test]
    fn empty_lists_omit_file_deh_and_assign_entirely() {
        let argv = tokens(&minimal_plan().to_command());
        assert!(!argv.iter().any(|token| token == "-file"));
        assert!(!argv.iter().any(|token| token == "-deh"));
        assert!(!argv.iter().any(|token| token == "-assign"));
    }

    #[test]
    fn an_auto_complevel_omits_the_flag_and_says_so_in_the_summary() {
        // Spec 026, R2: a custom setup's Auto means "no -complevel".
        let plan = LaunchPlan {
            complevel: None,
            ..minimal_plan()
        };
        let argv = tokens(&plan.to_command());
        assert!(!argv.iter().any(|token| token == "-complevel"), "{argv:?}");
        assert!(
            plan.describe(Path::new(HOME))
                .contains("  complevel: (engine default)\n"),
            "{}",
            plan.describe(Path::new(HOME))
        );
    }

    #[test]
    fn reviewed_complevels_are_accepted_and_placeholders_are_not() {
        for value in 0..=17u8 {
            assert!(Complevel::new(value).is_ok(), "{value} should be reviewed");
        }
        assert!(Complevel::new(21).is_ok());
        for value in [18u8, 19, 20] {
            assert!(
                Complevel::new(value).is_err(),
                "{value} is a placeholder and must be rejected"
            );
        }
        assert_eq!(Complevel::MBF21.value(), 21);
        assert_eq!(Complevel::FINAL_DOOM.value(), 4);
        assert_eq!(
            Complevel::new(2).expect("reviewed").name(),
            "Doom/Doom II 1.9"
        );
        assert_eq!(
            Complevel::new(18).expect_err("placeholder").to_string(),
            "compatibility level 18 is not a reviewed DSDA-Doom compatibility value (0-17 or 21)"
        );
    }

    #[test]
    fn a_word_names_its_level_and_keeps_the_resolved_number() {
        let vanilla = Complevel::from_word(ComplevelWord::Vanilla, 2).expect("reviewed");
        assert_eq!(vanilla.value(), 2);
        assert_eq!(vanilla.word(), Some(ComplevelWord::Vanilla));
        assert_eq!(vanilla.argument(), "vanilla");
        assert_eq!(vanilla.label(), "Vanilla (2)");
        assert_eq!(vanilla.to_string(), "vanilla (2)");
        for (word, value, argument, label) in [
            (ComplevelWord::Boom, 9, "boom", "Boom (9)"),
            (ComplevelWord::Mbf, 11, "mbf", "MBF (11)"),
            (ComplevelWord::Mbf21, 21, "mbf21", "MBF21 (21)"),
        ] {
            let level = Complevel::from_word(word, 3).expect("reviewed");
            assert_eq!(level.value(), value);
            assert_eq!(level.argument(), argument);
            assert_eq!(level.label(), label);
        }
        let number = Complevel::new(7).expect("reviewed");
        assert_eq!(number.word(), None);
        assert_eq!(number.argument(), "7");
        assert_eq!(number.label(), "7");
        assert!(Complevel::from_word(ComplevelWord::Vanilla, 18).is_err());
    }

    #[test]
    fn an_alias_is_sent_as_a_word_and_a_number_as_a_number() {
        let after_flag = |plan: &LaunchPlan| {
            let argv = plan.to_command();
            let at = argv.iter().position(|t| t == "-complevel").expect("flag");
            argv[at + 1].to_string_lossy().into_owned()
        };
        let aliased = LaunchPlan {
            complevel: Some(Complevel::from_word(ComplevelWord::Vanilla, 2).unwrap()),
            ..sample_plan()
        };
        assert_eq!(after_flag(&aliased), "vanilla");
        assert!(
            aliased
                .describe(Path::new("/home/deck"))
                .contains("  complevel: vanilla (2)\n")
        );
        let boom = LaunchPlan {
            complevel: Some(Complevel::from_word(ComplevelWord::Boom, 2).unwrap()),
            ..sample_plan()
        };
        assert_eq!(after_flag(&boom), "boom");
        let numeric = LaunchPlan {
            complevel: Some(Complevel::new(7).unwrap()),
            ..sample_plan()
        };
        assert_eq!(after_flag(&numeric), "7");
        assert!(
            numeric
                .describe(Path::new("/home/deck"))
                .contains("  complevel: 7 (Boom compatibility mode)\n")
        );
    }

    #[test]
    fn skills_outside_one_to_five_are_rejected() {
        assert!(Skill::new(0).is_err());
        assert!(Skill::new(6).is_err());
        for value in 1..=5u8 {
            assert_eq!(Skill::new(value).expect("in range").value(), value);
        }
    }

    #[test]
    fn warp_values_above_ninety_nine_are_rejected() {
        assert!(Warp::single(100).is_err());
        assert!(Warp::single(0).is_ok(), "0 is accepted by the parser");
        assert!(Warp::episode_map(1, 100).is_err());
        assert_eq!(
            Warp::episode_map(1, 3).expect("in range").to_string(),
            "1 3"
        );
    }

    // ---- validation: one test per error variant (spec 003, A2) ------------

    /// An existing regular file, used wherever an input file is required.
    fn existing_file() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
    }

    /// An existing directory, used wherever a directory is required.
    fn existing_dir() -> PathBuf {
        std::env::temp_dir()
    }

    /// A plan that passes validation, so each test can break exactly one field.
    ///
    /// The config file itself need not exist: it is an output of the engine.
    fn valid_plan() -> LaunchPlan {
        LaunchPlan {
            executable: existing_file(),
            iwad: existing_file(),
            files: Vec::new(),
            patches: Vec::new(),
            complevel: Some(Complevel::DOOM_1_9),
            skill: Some(Skill::new(3).expect("3 is a valid skill")),
            warp: None,
            pistol_start: false,
            record: None,
            config: existing_dir().join("dsda-doom.cfg"),
            save_dir: existing_dir(),
            data_dir: existing_dir(),
            shot_dir: existing_dir(),
            working_dir: existing_dir(),
            screen: None,
            assigns: Vec::new(),
        }
    }

    fn allowlist() -> AssignAllowlist {
        AssignAllowlist::with_keys(["dsda_fps", "autorun"])
    }

    #[test]
    fn a_well_formed_plan_validates() {
        assert_eq!(valid_plan().validate(&allowlist()), Ok(()));
    }

    #[test]
    fn build_command_validates_first_and_keeps_the_plan_argv() {
        let invalid = LaunchPlan {
            iwad: PathBuf::from("relative/doom2.wad"),
            ..valid_plan()
        };
        assert!(
            invalid.build_command(&allowlist()).is_err(),
            "build_command must not hand out a command for an invalid plan"
        );

        let plan = valid_plan();
        let command = plan.build_command(&allowlist()).expect("the plan is valid");
        assert_eq!(command.get_program(), plan.executable.as_os_str());
        assert_eq!(command.get_current_dir(), Some(plan.working_dir.as_path()));
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            tokens(&plan.to_command())
        );
    }

    #[test]
    fn path_that_looks_like_an_option_is_rejected() {
        let plan = LaunchPlan {
            iwad: PathBuf::from("-evil.wad"),
            ..valid_plan()
        };
        assert_eq!(
            plan.validate(&allowlist()),
            Err(ValidationError::PathLooksLikeOption {
                field: PathField::Iwad,
                path: PathBuf::from("-evil.wad"),
            })
        );
    }

    #[test]
    fn relative_path_is_rejected() {
        let plan = LaunchPlan {
            iwad: PathBuf::from("relative/doom2.wad"),
            ..valid_plan()
        };
        assert_eq!(
            plan.validate(&allowlist()),
            Err(ValidationError::PathNotAbsolute {
                field: PathField::Iwad,
                path: PathBuf::from("relative/doom2.wad"),
            })
        );
    }

    #[test]
    fn missing_input_file_is_rejected() {
        let missing = std::env::temp_dir().join("ddl-spec-003-missing-dir/doom2.wad");
        let plan = LaunchPlan {
            iwad: missing.clone(),
            ..valid_plan()
        };
        assert_eq!(
            plan.validate(&allowlist()),
            Err(ValidationError::MissingFile {
                field: PathField::Iwad,
                path: missing,
            })
        );
    }

    #[test]
    fn directory_where_a_file_is_expected_is_rejected() {
        let directory = existing_dir();
        let plan = LaunchPlan {
            iwad: directory.clone(),
            ..valid_plan()
        };
        assert_eq!(
            plan.validate(&allowlist()),
            Err(ValidationError::NotRegularFile {
                field: PathField::Iwad,
                path: directory,
            })
        );
    }

    #[test]
    fn missing_directory_is_rejected() {
        let missing = std::env::temp_dir().join("ddl-spec-003-missing-save-dir");
        let plan = LaunchPlan {
            save_dir: missing.clone(),
            ..valid_plan()
        };
        assert_eq!(
            plan.validate(&allowlist()),
            Err(ValidationError::DirectoryMissing {
                field: PathField::SaveDir,
                path: missing,
            })
        );
    }

    #[test]
    fn file_where_a_directory_is_expected_is_rejected() {
        let file = existing_file();
        let plan = LaunchPlan {
            working_dir: file.clone(),
            ..valid_plan()
        };
        assert_eq!(
            plan.validate(&allowlist()),
            Err(ValidationError::NotADirectory {
                field: PathField::WorkingDir,
                path: file,
            })
        );
    }

    #[test]
    fn a_missing_config_parent_directory_is_rejected() {
        let missing = std::env::temp_dir().join("ddl-spec-003-missing-config/config.cfg");
        let plan = LaunchPlan {
            config: missing.clone(),
            ..valid_plan()
        };
        assert_eq!(
            plan.validate(&allowlist()),
            Err(ValidationError::DirectoryMissing {
                field: PathField::Config,
                path: std::env::temp_dir().join("ddl-spec-003-missing-config"),
            })
        );
    }

    #[test]
    fn assign_key_with_invalid_characters_is_rejected() {
        let plan = LaunchPlan {
            assigns: vec![AssignOverride::new("dsda-fps", "35")],
            ..valid_plan()
        };
        assert_eq!(
            plan.validate(&allowlist()),
            Err(ValidationError::AssignKeyInvalid {
                key: String::from("dsda-fps"),
            })
        );
    }

    #[test]
    fn assign_key_outside_the_allowlist_is_rejected() {
        let plan = LaunchPlan {
            assigns: vec![AssignOverride::new("dsda_fps", "35")],
            ..valid_plan()
        };
        assert_eq!(
            plan.validate(&AssignAllowlist::with_keys(["autorun"])),
            Err(ValidationError::AssignKeyNotAllowed {
                key: String::from("dsda_fps"),
            })
        );
    }

    #[test]
    fn assign_value_that_looks_like_an_option_is_rejected() {
        let plan = LaunchPlan {
            assigns: vec![AssignOverride::new("dsda_fps", "-1")],
            ..valid_plan()
        };
        assert_eq!(
            plan.validate(&allowlist()),
            Err(ValidationError::AssignValueLooksLikeOption {
                key: String::from("dsda_fps"),
                value: String::from("-1"),
            })
        );
    }

    #[test]
    fn pistol_start_with_recording_is_rejected() {
        let plan = LaunchPlan {
            pistol_start: true,
            record: Some(PathBuf::from(
                "/home/deck/.local/share/ddl/profiles/p1/demos/run1.lmp",
            )),
            ..valid_plan()
        };
        assert_eq!(
            plan.validate(&allowlist()),
            Err(ValidationError::PistolStartWithRecord)
        );
    }

    // ---- describe (spec 003, A5) ------------------------------------------

    #[test]
    fn describe_snapshot_for_a_full_plan() {
        let expected = "\
Launch plan
  executable: /opt/ddl/engines/dsda-doom.appimage
  IWAD: ~/.local/share/ddl/iwads/doom2/doom2.wad
  files (load order):
    ~/.local/share/ddl/content/dbp37/DBP37_AUGZEN.wad
    ~/.local/share/ddl/content/dbp37/addons/DBP37_DeckardBlaster.wad
  patches:
    ~/.local/share/ddl/content/dbp37/augzen.deh
  complevel: 2 (Doom/Doom II 1.9)
  skill: 3
  warp: 1
  pistol start: yes
  record: (none)
  config: ~/.config/ddl/dsda/doom2.cfg
  save: ~/.local/share/ddl/profiles/p1/save
  data: ~/.local/share/ddl/profiles/p1/data
  shotdir: ~/.local/share/ddl/profiles/p1/shots
  screen: (config fallback)
  working dir: ~/.local/share/ddl/profiles/p1/run
  assign: (none)";
        assert_eq!(sample_plan().describe(Path::new(HOME)), expected);
    }

    #[test]
    fn describe_lists_options_or_none_and_keeps_paths_outside_home_absolute() {
        let plan = LaunchPlan {
            assigns: vec![AssignOverride::new("dsda_fps", "35")],
            ..minimal_plan()
        };
        let text = plan.describe(Path::new(HOME));
        assert!(text.contains("  files (load order): (none)\n"), "{text}");
        assert!(text.contains("  patches: (none)\n"), "{text}");
        assert!(text.contains("  warp: (none)\n"), "{text}");
        assert!(text.contains("  pistol start: no\n"), "{text}");
        assert!(text.contains("  record: (none)\n"), "{text}");
        assert!(
            text.contains("  executable: /opt/ddl/engines/dsda-doom.appimage\n"),
            "{text}"
        );
        assert!(text.ends_with("  assign:\n    dsda_fps=35"), "{text}");
    }

    #[test]
    fn describe_shortens_the_home_directory_itself() {
        let plan = LaunchPlan {
            executable: PathBuf::from(HOME),
            ..minimal_plan()
        };
        assert!(
            plan.describe(Path::new(HOME)).contains("  executable: ~\n"),
            "the home directory itself is rendered as ~"
        );
    }

    #[test]
    fn describe_does_not_shorten_a_sibling_of_home() {
        let plan = LaunchPlan {
            executable: PathBuf::from("/home/deckard/dsda-doom"),
            ..minimal_plan()
        };
        assert!(
            plan.describe(Path::new(HOME))
                .contains("  executable: /home/deckard/dsda-doom\n"),
            "a path that merely shares a prefix with home is left alone"
        );
    }
}

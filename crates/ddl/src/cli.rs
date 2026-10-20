//! Command-line parsing for the `ddl` binary (spec 001 R3, spec 014 R1).
//!
//! A small hand-written parser in the same style as `ddl catalog lint`: each
//! subcommand is parsed positionally, unknown options fail, and nothing here
//! ever initialises a window. `--help` is available globally and per
//! subcommand.

use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

/// Executable name as it appears in `--version` and usage text.
pub const BIN: &str = env!("CARGO_PKG_NAME");

/// Version reported by `--version`, taken from the workspace `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Options that modify a normal run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Options {
    /// `--windowed`: run in a 1280x800 window instead of fullscreen.
    pub windowed: bool,
}

/// Which help text to print.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTopic {
    /// `ddl --help`.
    Global,
    /// `ddl engine [--help]`.
    Engine,
    /// `ddl engine status --help`.
    EngineStatus,
    /// `ddl engine install --help`.
    EngineInstall,
    /// `ddl iwad [--help]`.
    Iwad,
    /// `ddl iwad scan --help`.
    IwadScan,
    /// `ddl mapset [--help]`.
    Mapset,
    /// `ddl mapset list --help`.
    MapsetList,
    /// `ddl mapset install --help`.
    MapsetInstall,
    MapsetStatus,
    MapsetUpdate,
    MapsetUninstall,
    /// `ddl launch --help`.
    Launch,
    /// `ddl doctor --help`.
    Doctor,
    /// `ddl catalog [--help]`.
    Catalog,
    /// `ddl catalog bundle --help`.
    CatalogBundle,
    /// `ddl catalog check-links --help`.
    CatalogCheckLinks,
    /// `ddl catalog e2e-sample --help`.
    CatalogE2eSample,
    /// `ddl catalog status --help`.
    CatalogStatus,
    /// `ddl catalog update --help`.
    CatalogUpdate,
    /// `ddl custom --help`.
    Custom,
    /// `ddl custom list --help`.
    CustomList,
    /// `ddl custom show --help`.
    CustomShow,
    /// `ddl custom create --help`.
    CustomCreate,
    /// `ddl custom add-file --help`.
    CustomAddFile,
    /// `ddl custom move --help`.
    CustomMove,
    /// `ddl custom remove-file --help`.
    CustomRemoveFile,
    /// `ddl custom delete --help`.
    CustomDelete,
    /// `ddl profile --help`.
    Profile,
    /// `ddl profile extras --help`.
    ProfileExtras,
}

/// Which of a setup's two ordered groups a CLI command addresses (spec 026, R3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupArg {
    /// WAD/ZIP files, sent as the `-file` list.
    Files,
    /// DEH/BEX patches, sent as the `-deh` list.
    Patches,
}

impl GroupArg {
    /// Parses `files` or `patches`.
    fn parse(value: &str) -> Option<Self> {
        match value {
            "files" => Some(Self::Files),
            "patches" => Some(Self::Patches),
            _ => None,
        }
    }
}

/// `ddl custom …` (spec 026, R7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomCommand {
    /// `ddl custom list`.
    List,
    /// `ddl custom show <id>`.
    Show {
        /// Setup id.
        id: String,
    },
    /// `ddl custom create <name> --iwad <variant> | --iwad-file <path>`.
    Create {
        /// Friendly name.
        name: String,
        /// Explicit id (`--id`).
        id: Option<String>,
        /// Identified IWAD variant (`--iwad`).
        iwad: Option<String>,
        /// User file whose header is `IWAD` (`--iwad-file`).
        iwad_file: Option<std::path::PathBuf>,
        /// `auto` or a reviewed number (`--complevel`).
        compatibility: Option<String>,
    },
    /// `ddl custom add-file <id> <path...>`.
    AddFile {
        /// Setup id.
        id: String,
        /// Files to append, routed by content.
        paths: Vec<std::path::PathBuf>,
    },
    /// `ddl custom move <id> <files|patches> <from> <to>`.
    Move {
        /// Setup id.
        id: String,
        /// Group addressed.
        group: GroupArg,
        /// Source index.
        from: usize,
        /// Destination index.
        to: usize,
    },
    /// `ddl custom remove-file <id> <files|patches> <index>`.
    RemoveFile {
        /// Setup id.
        id: String,
        /// Group addressed.
        group: GroupArg,
        /// Index to drop.
        index: usize,
    },
    /// `ddl custom delete <id> [--delete-profiles]`.
    Delete {
        /// Setup id.
        id: String,
        /// Also delete its profiles and saves (D-022 checkbox).
        delete_profiles: bool,
    },
}

/// `ddl profile extras …` (spec 026, R5/R7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileExtrasCommand {
    /// List the selected profile's extras.
    List {
        /// Recipe id.
        recipe: String,
        /// Profile name or id (`--profile`).
        profile: Option<String>,
    },
    /// Append user files to the selected profile.
    Add {
        /// Recipe id.
        recipe: String,
        /// Profile name or id (`--profile`).
        profile: Option<String>,
        /// Files to append.
        paths: Vec<std::path::PathBuf>,
    },
    /// Remove one extra from a group.
    Remove {
        /// Recipe id.
        recipe: String,
        /// Profile name or id (`--profile`).
        profile: Option<String>,
        /// Group addressed.
        group: GroupArg,
        /// Index within the group.
        index: usize,
    },
    /// Reorder one extra inside a group.
    Move {
        /// Recipe id.
        recipe: String,
        /// Profile name or id (`--profile`).
        profile: Option<String>,
        /// Group addressed.
        group: GroupArg,
        /// Source index within the group.
        from: usize,
        /// Destination index within the group.
        to: usize,
    },
    /// Drop every extra from the selected profile.
    Clear {
        /// Recipe id.
        recipe: String,
        /// Profile name or id (`--profile`).
        profile: Option<String>,
    },
}

/// What the user asked `ddl` to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Start the launcher with the given options.
    Run(Options),
    /// Offline by default; --fetch explicitly verifies upstream package bytes.
    CatalogLint {
        paths: Vec<PathBuf>,
        fetch: bool,
    },
    /// `ddl catalog bundle`: build one JSON catalog bundle (spec 019, R1).
    CatalogBundle {
        /// Where to write the bundle (`--out`).
        out: PathBuf,
        /// The catalog directory to bundle (`--catalog`, default `catalog`).
        catalog: PathBuf,
        /// The source-commit serial in UTC Unix seconds (`--serial`, default
        /// `DDL_CATALOG_SERIAL`/`SOURCE_DATE_EPOCH`).
        serial: Option<u64>,
        /// The source commit (`--commit`, default `DDL_SOURCE_COMMIT`/`GITHUB_SHA`).
        commit: Option<String>,
        /// The oldest DDL allowed to consume the bundle (`--min-ddl-version`,
        /// default `DEFAULT_MIN_DDL_VERSION`, a deliberate constant).
        min_ddl_version: Option<String>,
    },
    /// `ddl catalog check-links`: header-only link health of every recipe
    /// source (spec 044, R1–R2).
    CatalogCheckLinks {
        /// The catalog directory (default `catalog`).
        dir: PathBuf,
        /// Print the machine-readable report instead of the Markdown table.
        json: bool,
        /// List only the rows that are not `ok`.
        problems_only: bool,
    },
    /// `ddl catalog e2e-sample`: the lean install set of the real-network e2e
    /// (spec 044, R4).
    CatalogE2eSample {
        /// The catalog directory (default `catalog`).
        dir: PathBuf,
        /// Cap on one recipe's total download, in MiB (`--max-mib`).
        max_mib: u64,
    },
    /// `ddl catalog status`: the effective catalog's serial and source.
    CatalogStatus,
    /// `ddl catalog update`: fetch and accept a newer bundle now (spec 019, R5).
    CatalogUpdate,
    /// `ddl engine status`: installed versions, the active one and its mode.
    EngineStatus,
    /// `ddl engine install`: the reviewed install path, optionally forcing the
    /// extraction fallback.
    EngineInstall {
        /// Reviewed version or tag; `None` means the newest reviewed release.
        version: Option<String>,
        /// Take the extraction fallback even when the AppImage would run.
        force_extract: bool,
    },
    /// `ddl iwad scan`: Steam discovery plus the given folders.
    IwadScan {
        paths: Vec<PathBuf>,
    },
    /// `ddl mapset list`.
    MapsetList,
    MapsetStatus {
        id: Option<String>,
    },
    MapsetUpdate {
        id: String,
    },
    /// `ddl mapset uninstall`: remove an installed mapset (spec 022, R5).
    MapsetUninstall {
        /// Recipe id.
        id: String,
        /// Also delete the mapset's profiles and their saves (`--delete-profiles`).
        delete_profiles: bool,
    },
    /// `ddl mapset install`: install a recipe, optionally enabling components.
    MapsetInstall {
        /// Recipe id.
        id: String,
        /// Optional component ids to enable (`--with`).
        components: Vec<String>,
    },
    /// `ddl launch`: resolve a mapset, and either print the plan or run it.
    Launch {
        /// Recipe id.
        id: String,
        /// Print the argv and working directory instead of launching.
        dry_run: bool,
        /// Profile name or stable id.
        profile: Option<String>,
        /// Optional component ids to enable (`--with`).
        components: Vec<String>,
        /// Starting skill override (`--skill`).
        skill: Option<u8>,
        /// Reset to a pistol start on every map (`--pistol-start`).
        pistol_start: bool,
        /// Re-record changed user files instead of blocking (spec 026, R4).
        accept_changes: bool,
    },
    /// `ddl custom …`: player-made setups (spec 026, R7).
    Custom(CustomCommand),
    /// `ddl profile extras …`: extra files on a catalog profile (spec 026, R7).
    ProfileExtras(ProfileExtrasCommand),
    /// `ddl doctor`: one paste-friendly diagnostic report.
    Doctor,
    /// Print [`version_line`] and exit successfully.
    Version,
    /// Print the topic's help and exit successfully.
    Help(HelpTopic),
}

/// A rejected command line. Rendered by the binary as
/// `ddl: <message>` plus a `--help` hint, exiting with status 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    fn unknown(argument: &str) -> Self {
        let message = if argument.starts_with('-') {
            format!("unknown option `{argument}`")
        } else {
            format!("unexpected argument `{argument}`")
        };
        Self { message }
    }

    fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}

/// `ddl <version>` — the exact first line of `ddl --version`.
pub fn version_line() -> String {
    format!("{BIN} {VERSION}")
}

/// The full `ddl --help` text. Kept for the parser tests; the binary uses
/// [`help_for`].
#[cfg(test)]
pub fn help_text() -> String {
    help_for(HelpTopic::Global)
}

/// The help text for one topic (spec 014, R1: `--help` for each subcommand).
pub fn help_for(topic: HelpTopic) -> String {
    match topic {
        HelpTopic::Global => format!(
            "\
{BIN}: DoomDeckLauncher, a native DOOM mapset launcher for Steam Deck

Usage: {BIN} [OPTIONS]
       {BIN} catalog lint [--fetch] [PATHS...]
       {BIN} catalog bundle --out <file> [--catalog <dir>] [--serial <unix>] [--commit <sha>] [--min-ddl-version <v>]
       {BIN} catalog check-links [DIR] [--json] [--problems-only]
       {BIN} catalog e2e-sample [DIR] [--max-mib <n>]
       {BIN} catalog status
       {BIN} catalog update
       {BIN} engine status
       {BIN} engine install [--version <V>] [--force-extract]
       {BIN} iwad scan [PATH...]
       {BIN} mapset list
       {BIN} mapset status [<id>]
       {BIN} mapset update <id>
       {BIN} mapset uninstall <id> [--delete-profiles]
       {BIN} mapset install <id> [--with <component>...]
       {BIN} launch <id> [--dry-run] [--profile <name|id>] [--with <component>...] [--skill <1-5>] [--pistol-start] [--accept-changes]
       {BIN} custom list
       {BIN} custom show <id>
       {BIN} custom create <name> [--id <id>] (--iwad <variant> | --iwad-file <path>) [--complevel auto|<n>]
       {BIN} custom add-file <id> <path>...
       {BIN} custom move <id> <files|patches> <from> <to>
       {BIN} custom remove-file <id> <files|patches> <index>
       {BIN} custom delete <id> [--delete-profiles]
       {BIN} profile extras <recipe> [--profile <name|id>] list
       {BIN} profile extras <recipe> [--profile <name|id>] add <path>...
       {BIN} profile extras <recipe> [--profile <name|id>] move <files|patches> <from> <to>
       {BIN} profile extras <recipe> [--profile <name|id>] remove <files|patches> <index>
       {BIN} profile extras <recipe> [--profile <name|id>] clear
       {BIN} doctor

Commands:
  catalog       Lint recipes, build a bundle or fetch a catalog update
  engine        Install or inspect the DSDA-Doom engine
  iwad scan     Find and identify your IWAD files
  mapset        List, inspect, install or update mapsets
  launch        Resolve a mapset or custom setup; without --dry-run, run it
  custom        Create and edit player-made setups from local files
  profile       Edit a mapset profile's extra files
  doctor        Print a paste-friendly diagnostic report

Options:
      --windowed  Run in a 1280x800 window instead of fullscreen
  -h, --help      Print this help and exit
  -V, --version   Print version information and exit

Environment:
  DDL_LOG  Log level for stderr diagnostics: error, warn, info, debug or trace
           (default: info)
"
        ),
        HelpTopic::Engine => format!(
            "\
{BIN} engine: manage the DSDA-Doom engine

Usage: {BIN} engine status
       {BIN} engine install [--version <V>] [--force-extract]

Commands:
  status   Show installed versions, the active one and how it runs
  install  Download, verify and install a reviewed DSDA-Doom release

Options for `engine install`:
      --version <V>     Reviewed version (0.30.1) or tag (v0.30.1)
      --force-extract   Take the extraction fallback even when the AppImage
                        would run, to validate it on purpose (D-013)
"
        ),
        HelpTopic::EngineStatus => format!(
            "\
{BIN} engine status

Show every installed DSDA-Doom version, the active one and how it runs
(appimage, extracted or manual).

Usage: {BIN} engine status
"
        ),
        HelpTopic::EngineInstall => format!(
            "\
{BIN} engine install [--version <V>] [--force-extract]

Download a reviewed DSDA-Doom AppImage, verify its size and SHA-256,
smoke-test it with `-v` and install it. Progress goes to stderr.

Usage: {BIN} engine install [--version <V>] [--force-extract]

Options:
      --version <V>     Reviewed version (0.30.1) or tag (v0.30.1);
                        default: the newest reviewed release
      --force-extract   Extract the AppImage even when it would run, so the
                        D-013 fallback can be validated on purpose
"
        ),
        HelpTopic::Iwad => format!(
            "\
{BIN} iwad: find and identify IWAD files

Usage: {BIN} iwad scan [PATH...]

Commands:
  scan  Discover and identify base-game IWADs
"
        ),
        HelpTopic::IwadScan => format!(
            "\
{BIN} iwad scan [PATH...]

Discover IWADs in the Steam libraries, in GOG installs recorded by Heroic or
minigalaxy, and in each PATH (which is added to DDL's folders), then print
every identified file with its source, variant, game and SHA-256. Generic
DOOM folders are only searched when enabled in Settings. Ambiguous files are
listed with their candidates and are never chosen automatically. DDL never
downloads IWADs.

Usage: {BIN} iwad scan [PATH...]
"
        ),
        HelpTopic::Mapset => format!(
            "\
{BIN} mapset: inspect and manage mapsets

Usage: {BIN} mapset list
       {BIN} mapset status [<id>]
       {BIN} mapset update <id>
       {BIN} mapset uninstall <id> [--delete-profiles]
       {BIN} mapset install <id> [--with <component>...]

Commands:
  list     List every catalog recipe with its install state
  install  Install a mapset, optionally enabling components
  status   Classify changes to installed recipes
  update   Apply a recipe update safely
  uninstall  Remove an installed mapset and its files
"
        ),
        HelpTopic::MapsetList => format!(
            "\
{BIN} mapset list

List every catalog recipe with its id, title, base game and install state.

Usage: {BIN} mapset list
"
        ),
        HelpTopic::MapsetStatus => format!("{BIN} mapset status [<id>]\n\nPrint installed recipe change classifications and summaries.\n\nUsage: {BIN} mapset status [<id>]\n"),
        HelpTopic::MapsetUpdate => format!("{BIN} mapset update <id>\n\nStage, verify and atomically activate a catalog update; preserve profiles and saves.\nProgress goes to stderr.\n\nUsage: {BIN} mapset update <id>\n"),
        HelpTopic::MapsetUninstall => format!("{BIN} mapset uninstall <id> [--delete-profiles]\n\nRemove an installed mapset. By default only its install files are deleted; --delete-profiles\nalso deletes the mapset's profiles and their saves, screenshots and data.\n\nUsage: {BIN} mapset uninstall <id> [--delete-profiles]\n\nOptions:\n      --delete-profiles  Also delete the mapset's profiles and saves\n"),
        HelpTopic::MapsetInstall => format!(
            "\
{BIN} mapset install <id> [--with <component>...]

Install a mapset from the embedded catalog, downloading and verifying its
packages. Progress goes to stderr. `--with` enables an optional component
and is validated against the recipe.

Usage: {BIN} mapset install <id> [--with <component>...]
"
        ),
        HelpTopic::Launch => format!(
            "\
{BIN} launch <id> [--dry-run] [--profile <name|id>] [--with <component>...] [--skill <1-5>] [--pistol-start] [--accept-changes]

Resolve a mapset — or a custom setup — exactly as the UI's Play does. With
--dry-run it prints the argv (one argument per line) and the working directory,
or the typed blocker with its explanation. Without --dry-run it starts
DSDA-Doom through the same run guard as the UI.

Usage: {BIN} launch <id> [--dry-run] [--profile <name|id>] [--with <component>...] [--skill <1-5>] [--pistol-start] [--accept-changes]

Options:
      --dry-run        Print the launch plan instead of starting the engine
      --with <C>       Enable an optional component (repeatable)
      --skill <1-5>    Starting skill
      --profile <name|id>  Select a saved profile
      --pistol-start   Reset to a pistol start on every map
      --accept-changes Re-record user files that changed since DDL read them
"
        ),
        HelpTopic::Doctor => format!(
            "\
{BIN} doctor

Print a paste-friendly report: DDL's paths, engine state, IWAD registry,
installed mapsets, DSDA config state and log location.

Usage: {BIN} doctor
"
        ),
        HelpTopic::Catalog => format!(
            "\
{BIN} catalog: the recipe catalog

Usage: {BIN} catalog lint [--fetch] [PATHS...]
       {BIN} catalog bundle --out <file> [--catalog <dir>] [--serial <unix>] [--commit <sha>] [--min-ddl-version <v>]
       {BIN} catalog check-links [DIR] [--json] [--problems-only]
       {BIN} catalog e2e-sample [DIR] [--max-mib <n>]
       {BIN} catalog status
       {BIN} catalog update

Commands:
  lint         Validate catalog recipes (offline; --fetch verifies upstream bytes)
  bundle       Lint, then write one JSON bundle for the CI catalog branch
  check-links  Check every recipe mirror's headers (no downloads); fails when a recipe has no good source
  e2e-sample   Print the lean set of installs the real-network e2e runs
  status  Show the catalog DDL is using (built-in or updated) and its date
  update  Fetch and accept a newer catalog bundle now
"
        ),
        HelpTopic::CatalogBundle => format!(
            "\
{BIN} catalog bundle --out <file> [--catalog <dir>] [--serial <unix>] [--commit <sha>] [--min-ddl-version <v>]

Lint the catalog directory and write one JSON bundle. It refuses to write on
any lint error. The serial is the UTC Unix time of the source commit, so a
release built from the same commit and the published bundle agree.

Usage: {BIN} catalog bundle --out <file> [--catalog <dir>] [--serial <unix>] [--commit <sha>] [--min-ddl-version <v>]

Options:
      --out <file>           Where to write the bundle (required)
      --catalog <dir>        Catalog directory to bundle (default: catalog)
      --serial <unix>        Source-commit time in UTC seconds
                             (default: DDL_CATALOG_SERIAL, then SOURCE_DATE_EPOCH)
      --commit <sha>         Source commit (default: DDL_SOURCE_COMMIT, then GITHUB_SHA)
      --min-ddl-version <v>  Oldest DDL allowed to consume the bundle
                             (default: a deliberate constant, currently 0.0.4;
                             raise it only when the bundle needs a newer client)
"
        ),
        HelpTopic::CatalogCheckLinks => format!(
            "\
{BIN} catalog check-links [DIR] [--json] [--problems-only]

Ask every source URL of every recipe in DIR (default: catalog) whether the file
is still there and still the pinned size, without downloading it: a HEAD request
(or GET with Range: bytes=0-0 when a host refuses HEAD) through the downloader's
allowlist and redirect rules. At most 4 requests run at once and one per host.

Each URL is ok, size-changed, gone (404/410), host-refused (a redirect left the
download allowlist) or error. The report is a Markdown table on stdout. The exit
status is non-zero when any recipe has no ok source left; a recipe with one dead
mirror and one good one is only a warning.

Usage: {BIN} catalog check-links [DIR] [--json] [--problems-only]

Options:
      --json           Print the report as JSON instead of a Markdown table
      --problems-only  List only the rows that are not ok
"
        ),
        HelpTopic::CatalogE2eSample => format!(
            "\
{BIN} catalog e2e-sample [DIR] [--max-mib <n>]

Print the installs the real-network e2e should run, one per line as
recipe<TAB>component<TAB>reason (the component is - for a plain install):
the smallest recipe per first-source download host, plus every recipe with
optional components, each free component once. Each recipe's total download
stays under --max-mib (default 64) unless no recipe for its host fits.

Usage: {BIN} catalog e2e-sample [DIR] [--max-mib <n>]
"
        ),
        HelpTopic::CatalogStatus => format!(
            "\
{BIN} catalog status

Show the catalog DDL is using: the embedded catalog or a cached update, its
serial date and the source commit. Never touches the network.

Usage: {BIN} catalog status
"
        ),
        HelpTopic::CatalogUpdate => format!(
            "\
{BIN} catalog update

Fetch the published catalog bundle and accept it when it is newer than the one
DDL has and every recipe passes the same lint CI runs. Progress goes to stderr.

Usage: {BIN} catalog update
"
        ),
        HelpTopic::Custom => format!(
            "\
{BIN} custom: player-made setups from local files

Usage: {BIN} custom list
       {BIN} custom show <id>
       {BIN} custom create <name> [--id <id>] (--iwad <variant> | --iwad-file <path>) [--complevel auto|<n>]
       {BIN} custom add-file <id> <path>...
       {BIN} custom move <id> <files|patches> <from> <to>
       {BIN} custom remove-file <id> <files|patches> <index>
       {BIN} custom delete <id> [--delete-profiles]

Commands:
  list         List every custom setup
  show         Show one setup and check that its files are still usable
  create       Create a setup (an identified IWAD or a file whose header is IWAD)
  add-file     Append files, routed by content to the WAD/ZIP or DEH/BEX list
  move         Reorder one entry inside a group
  remove-file  Drop one entry; the file on disk is never deleted
  delete       Remove DDL's record, optionally with its profiles and saves

Custom setups are not verified by DDL and are labelled `Custom -- not verified`.
"
        ),
        HelpTopic::CustomList => format!(
            "\
{BIN} custom list

List every custom setup with its id, name, IWAD and file counts.

Usage: {BIN} custom list
"
        ),
        HelpTopic::CustomShow => format!(
            "\
{BIN} custom show <id>

Show one custom setup and check each user file against its recorded size and
SHA-256, naming any file that is missing or changed.

Usage: {BIN} custom show <id>
"
        ),
        HelpTopic::CustomCreate => format!(
            "\
{BIN} custom create <name> [--id <id>] (--iwad <variant> | --iwad-file <path>) [--complevel auto|<n>]

Create a custom setup. --iwad names an identified IWAD variant from `ddl iwad
scan`; --iwad-file names a file whose header is IWAD. --complevel defaults to
auto (no -complevel, DSDA-Doom decides).

Usage: {BIN} custom create <name> [--id <id>] (--iwad <variant> | --iwad-file <path>) [--complevel auto|<n>]
"
        ),
        HelpTopic::CustomAddFile => format!(
            "\
{BIN} custom add-file <id> <path>...

Append one or more user files to a setup. Each file's kind is detected from its
content; WAD/ZIP go to the -file list, DEH/BEX to the -deh list, in the order
given. Accepted extensions are .wad, .lmp, .zip, .deh and .bex.

Usage: {BIN} custom add-file <id> <path>...
"
        ),
        HelpTopic::CustomMove => format!(
            "\
{BIN} custom move <id> <files|patches> <from> <to>

Move one entry within a setup's -file or -deh list, keeping the rest in order.

Usage: {BIN} custom move <id> <files|patches> <from> <to>
"
        ),
        HelpTopic::CustomRemoveFile => format!(
            "\
{BIN} custom remove-file <id> <files|patches> <index>

Remove one entry from a setup's -file or -deh list. The file on disk is never
deleted.

Usage: {BIN} custom remove-file <id> <files|patches> <index>
"
        ),
        HelpTopic::CustomDelete => format!(
            "\
{BIN} custom delete <id> [--delete-profiles]

Remove a custom setup's record. --delete-profiles also deletes the setup's
profiles and their saves. Your own files are never touched.

Usage: {BIN} custom delete <id> [--delete-profiles]

Options:
      --delete-profiles  Also delete the setup's profiles and saves
"
        ),
        HelpTopic::Profile => format!(
            "\
{BIN} profile: edit a mapset profile's extra files

Usage: {BIN} profile extras <recipe> [--profile <name|id>] list
       {BIN} profile extras <recipe> [--profile <name|id>] add <path>...
       {BIN} profile extras <recipe> [--profile <name|id>] move <files|patches> <from> <to>
       {BIN} profile extras <recipe> [--profile <name|id>] remove <files|patches> <index>
       {BIN} profile extras <recipe> [--profile <name|id>] clear
"
        ),
        HelpTopic::ProfileExtras => format!(
            "\
{BIN} profile extras <recipe> [--profile <name|id>] <list|add|move|remove|clear>

Extra files are layered after the recipe's own verified files: recipe files in
recipe order, then the extras. A profile with extras is labelled `Custom -- not
verified`. Changing extras on a profile that has saves can make those saves
unloadable, so DDL warns.

Usage: {BIN} profile extras <recipe> [--profile <name|id>] list
       {BIN} profile extras <recipe> [--profile <name|id>] add <path>...
       {BIN} profile extras <recipe> [--profile <name|id>] move <files|patches> <from> <to>
       {BIN} profile extras <recipe> [--profile <name|id>] remove <files|patches> <index>
       {BIN} profile extras <recipe> [--profile <name|id>] clear

Options:
      --profile <name|id>  Select a saved profile (default: the active one)
"
        ),
    }
}

/// Parses the arguments after the program name.
///
/// `--help` and `--version` win wherever they appear at the top level, as they
/// do in most CLIs. When the first argument names a subcommand, that
/// subcommand's parser owns the rest (so `engine install --version 0.30.1` is
/// the engine's version, not DDL's).
///
/// # Errors
///
/// Returns [`ParseError`] for any unknown option or stray positional argument.
pub fn parse<I, S>(args: I) -> Result<Command, ParseError>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();

    match args.first().and_then(|arg| arg.to_str()) {
        Some("catalog") => parse_catalog(&args[1..]),
        Some("engine") => parse_engine(&args[1..]),
        Some("iwad") => parse_iwad(&args[1..]),
        Some("mapset") => parse_mapset(&args[1..]),
        Some("launch") => parse_launch(&args[1..]),
        Some("custom") => parse_custom(&args[1..]),
        Some("profile") => parse_profile(&args[1..]),
        Some("doctor") => parse_doctor(&args[1..]),
        _ => parse_global(&args),
    }
}

fn parse_global(args: &[OsString]) -> Result<Command, ParseError> {
    if wants_help(args) {
        return Ok(Command::Help(HelpTopic::Global));
    }
    if args.iter().any(|arg| is_one_of(arg, &["-V", "--version"])) {
        return Ok(Command::Version);
    }
    let mut options = Options::default();
    for arg in args {
        if is_one_of(arg, &["--windowed"]) {
            options.windowed = true;
        } else {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        }
    }
    Ok(Command::Run(options))
}

fn parse_catalog(args: &[OsString]) -> Result<Command, ParseError> {
    match args.first().and_then(|arg| arg.to_str()) {
        Some("lint") => parse_catalog_lint(&args[1..]),
        Some("bundle") => parse_catalog_bundle(&args[1..]),
        Some("check-links") => parse_catalog_check_links(&args[1..]),
        Some("e2e-sample") => parse_catalog_e2e_sample(&args[1..]),
        Some("status") => {
            if wants_help(&args[1..]) {
                return Ok(Command::Help(HelpTopic::CatalogStatus));
            }
            for arg in &args[1..] {
                if arg != "--" {
                    return Err(ParseError::unknown(&arg.to_string_lossy()));
                }
            }
            Ok(Command::CatalogStatus)
        }
        Some("update") => {
            if wants_help(&args[1..]) {
                return Ok(Command::Help(HelpTopic::CatalogUpdate));
            }
            for arg in &args[1..] {
                if arg != "--" {
                    return Err(ParseError::unknown(&arg.to_string_lossy()));
                }
            }
            Ok(Command::CatalogUpdate)
        }
        _ if wants_help(args) => Ok(Command::Help(HelpTopic::Catalog)),
        _ => Err(ParseError::message(
            "expected `catalog lint [--fetch] [paths]`, `catalog bundle --out <file>`, `catalog check-links`, `catalog e2e-sample`, `catalog status` or `catalog update`",
        )),
    }
}

fn parse_catalog_lint(args: &[OsString]) -> Result<Command, ParseError> {
    if wants_help(args) {
        return Ok(Command::Help(HelpTopic::Catalog));
    }
    let mut paths = Vec::new();
    let mut fetch = false;
    let mut positional = false;
    for arg in args {
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && arg == "--fetch" {
            fetch = true;
        } else if !positional && arg.to_string_lossy().starts_with('-') {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        } else {
            paths.push(PathBuf::from(arg));
        }
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("catalog"));
    }
    Ok(Command::CatalogLint { paths, fetch })
}

fn parse_catalog_bundle(args: &[OsString]) -> Result<Command, ParseError> {
    let mut out = None;
    let mut catalog = None;
    let mut serial = None;
    let mut commit = None;
    let mut min_ddl_version = None;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if is_one_of(arg, &["-h", "--help"]) {
            return Ok(Command::Help(HelpTopic::CatalogBundle));
        } else if arg == "--out" {
            out = Some(PathBuf::from(option_value(args, index, "--out")?));
            index += 1;
        } else if arg == "--catalog" {
            catalog = Some(PathBuf::from(option_value(args, index, "--catalog")?));
            index += 1;
        } else if arg == "--serial" {
            let raw = option_value(args, index, "--serial")?;
            serial = Some(
                raw.parse()
                    .map_err(|_| ParseError::message("`--serial` must be a UTC Unix time"))?,
            );
            index += 1;
        } else if arg == "--commit" {
            commit = Some(option_value(args, index, "--commit")?.to_owned());
            index += 1;
        } else if arg == "--min-ddl-version" {
            min_ddl_version = Some(option_value(args, index, "--min-ddl-version")?.to_owned());
            index += 1;
        } else {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        }
        index += 1;
    }
    Ok(Command::CatalogBundle {
        out: out.ok_or_else(|| ParseError::message("`catalog bundle` needs `--out <file>`"))?,
        catalog: catalog.unwrap_or_else(|| PathBuf::from("catalog")),
        serial,
        commit,
        min_ddl_version,
    })
}

fn parse_catalog_check_links(args: &[OsString]) -> Result<Command, ParseError> {
    if wants_help(args) {
        return Ok(Command::Help(HelpTopic::CatalogCheckLinks));
    }
    let mut dir = None;
    let mut json = false;
    let mut problems_only = false;
    let mut positional = false;
    for arg in args {
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && arg == "--json" {
            json = true;
        } else if !positional && arg == "--problems-only" {
            problems_only = true;
        } else if !positional && arg.to_string_lossy().starts_with('-') {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        } else if dir.replace(PathBuf::from(arg)).is_some() {
            return Err(ParseError::message(
                "`catalog check-links` takes at most one directory",
            ));
        }
    }
    Ok(Command::CatalogCheckLinks {
        dir: dir.unwrap_or_else(|| PathBuf::from("catalog")),
        json,
        problems_only,
    })
}

fn parse_catalog_e2e_sample(args: &[OsString]) -> Result<Command, ParseError> {
    if wants_help(args) {
        return Ok(Command::Help(HelpTopic::CatalogE2eSample));
    }
    let mut dir = None;
    let mut max_mib = ddl_core::catalog::DEFAULT_MAX_ARCHIVE_BYTES / (1024 * 1024);
    let mut positional = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && arg == "--max-mib" {
            max_mib = option_value(args, index, "--max-mib")?
                .parse()
                .map_err(|_| ParseError::message("`--max-mib` must be a whole number of MiB"))?;
            index += 1;
        } else if !positional && arg.to_string_lossy().starts_with('-') {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        } else if dir.replace(PathBuf::from(arg)).is_some() {
            return Err(ParseError::message(
                "`catalog e2e-sample` takes at most one directory",
            ));
        }
        index += 1;
    }
    Ok(Command::CatalogE2eSample {
        dir: dir.unwrap_or_else(|| PathBuf::from("catalog")),
        max_mib,
    })
}

/// The UTF-8 value following `--flag` at `index`.
fn option_value<'a>(args: &'a [OsString], index: usize, name: &str) -> Result<&'a str, ParseError> {
    args.get(index + 1)
        .and_then(|value| value.to_str())
        .ok_or_else(|| ParseError::message(format!("`{name}` needs a value")))
}

fn parse_engine(args: &[OsString]) -> Result<Command, ParseError> {
    match args.first().and_then(|arg| arg.to_str()) {
        Some("status") => {
            if wants_help(&args[1..]) {
                return Ok(Command::Help(HelpTopic::EngineStatus));
            }
            for arg in &args[1..] {
                if arg != "--" {
                    return Err(ParseError::unknown(&arg.to_string_lossy()));
                }
            }
            Ok(Command::EngineStatus)
        }
        Some("install") => parse_engine_install(&args[1..]),
        _ if wants_help(args) => Ok(Command::Help(HelpTopic::Engine)),
        _ => Err(ParseError::message(
            "expected `engine status` or `engine install [--version <V>] [--force-extract]`",
        )),
    }
}

fn parse_engine_install(args: &[OsString]) -> Result<Command, ParseError> {
    let mut version = None;
    let mut force_extract = false;
    let mut positional = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && is_one_of(arg, &["-h", "--help"]) {
            return Ok(Command::Help(HelpTopic::EngineInstall));
        } else if !positional && arg == "--force-extract" {
            force_extract = true;
        } else if !positional && arg == "--version" {
            let value = args.get(index + 1).ok_or_else(|| {
                ParseError::message("`--version` needs a reviewed version or tag")
            })?;
            let value = value
                .to_str()
                .ok_or_else(|| ParseError::message("`--version` must be valid UTF-8"))?;
            version = Some(value.to_owned());
            index += 1;
        } else {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        }
        index += 1;
    }
    Ok(Command::EngineInstall {
        version,
        force_extract,
    })
}

fn parse_iwad(args: &[OsString]) -> Result<Command, ParseError> {
    match args.first().and_then(|arg| arg.to_str()) {
        Some("scan") => {
            let mut paths = Vec::new();
            let mut positional = false;
            for arg in &args[1..] {
                if !positional && arg == "--" {
                    positional = true;
                } else if !positional && is_one_of(arg, &["-h", "--help"]) {
                    return Ok(Command::Help(HelpTopic::IwadScan));
                } else if !positional && arg.to_string_lossy().starts_with('-') {
                    return Err(ParseError::unknown(&arg.to_string_lossy()));
                } else {
                    paths.push(PathBuf::from(arg));
                }
            }
            Ok(Command::IwadScan { paths })
        }
        _ if wants_help(args) => Ok(Command::Help(HelpTopic::Iwad)),
        _ => Err(ParseError::message("expected `iwad scan [PATH...]`")),
    }
}

fn parse_mapset(args: &[OsString]) -> Result<Command, ParseError> {
    match args.first().and_then(|arg| arg.to_str()) {
        Some("list") => {
            if wants_help(&args[1..]) {
                return Ok(Command::Help(HelpTopic::MapsetList));
            }
            for arg in &args[1..] {
                if arg != "--" {
                    return Err(ParseError::unknown(&arg.to_string_lossy()));
                }
            }
            Ok(Command::MapsetList)
        }
        Some("status" | "update") => {
            let update = args[0] == "update";
            if wants_help(&args[1..]) {
                return Ok(Command::Help(if update {
                    HelpTopic::MapsetUpdate
                } else {
                    HelpTopic::MapsetStatus
                }));
            }
            if args.len() > 2 {
                return Err(ParseError::unknown(&args[2].to_string_lossy()));
            }
            let id = args.get(1).map(|s| s.to_string_lossy().into_owned());
            if let Some(id) = &id
                && id.starts_with('-')
            {
                return Err(ParseError::unknown(id));
            }
            if update {
                Ok(Command::MapsetUpdate {
                    id: id
                        .ok_or_else(|| ParseError::message("`mapset update` needs a recipe id"))?,
                })
            } else {
                Ok(Command::MapsetStatus { id })
            }
        }
        Some("install") => parse_mapset_install(&args[1..]),
        Some("uninstall") => parse_mapset_uninstall(&args[1..]),
        _ if wants_help(args) => Ok(Command::Help(HelpTopic::Mapset)),
        _ => Err(ParseError::message(
            "expected `mapset list`, `mapset status [<id>]`, `mapset install <id>`, `mapset update <id>` or `mapset uninstall <id>`",
        )),
    }
}

fn parse_mapset_uninstall(args: &[OsString]) -> Result<Command, ParseError> {
    let mut id = None;
    let mut delete_profiles = false;
    let mut positional = false;
    for arg in args {
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && is_one_of(arg, &["-h", "--help"]) {
            return Ok(Command::Help(HelpTopic::MapsetUninstall));
        } else if !positional && arg == "--delete-profiles" {
            delete_profiles = true;
        } else if !positional && arg.to_string_lossy().starts_with('-') {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        } else if id.is_none() {
            id = Some(arg.to_string_lossy().into_owned());
        } else {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        }
    }
    let id = id.ok_or_else(|| ParseError::message("`mapset uninstall` needs a recipe id"))?;
    Ok(Command::MapsetUninstall {
        id,
        delete_profiles,
    })
}

fn parse_mapset_install(args: &[OsString]) -> Result<Command, ParseError> {
    let mut id = None;
    let mut components = Vec::new();
    let mut positional = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && is_one_of(arg, &["-h", "--help"]) {
            return Ok(Command::Help(HelpTopic::MapsetInstall));
        } else if !positional && arg == "--with" {
            let value = args
                .get(index + 1)
                .ok_or_else(|| ParseError::message("`--with` needs a component id"))?;
            let value = value
                .to_str()
                .ok_or_else(|| ParseError::message("the component id must be valid UTF-8"))?;
            components.push(value.to_owned());
            index += 1;
        } else if !positional && arg.to_string_lossy().starts_with('-') {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        } else if id.is_none() {
            id = Some(arg.to_string_lossy().into_owned());
        } else {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        }
        index += 1;
    }
    let id = id.ok_or_else(|| ParseError::message("`mapset install` needs a recipe id"))?;
    Ok(Command::MapsetInstall { id, components })
}

fn parse_launch(args: &[OsString]) -> Result<Command, ParseError> {
    let mut id = None;
    let mut dry_run = false;
    let mut profile = None;
    let mut components = Vec::new();
    let mut skill = None;
    let mut pistol_start = false;
    let mut accept_changes = false;
    let mut positional = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && is_one_of(arg, &["-h", "--help"]) {
            return Ok(Command::Help(HelpTopic::Launch));
        } else if !positional && arg == "--dry-run" {
            dry_run = true;
        } else if !positional && arg == "--accept-changes" {
            accept_changes = true;
        } else if !positional && arg == "--pistol-start" {
            pistol_start = true;
        } else if !positional && arg == "--profile" {
            if profile.is_some() {
                return Err(ParseError::message("`--profile` may appear only once"));
            }
            let value = args
                .get(index + 1)
                .and_then(|v| v.to_str())
                .filter(|v| !v.is_empty() && !v.starts_with('-'))
                .ok_or_else(|| ParseError::message("`--profile` needs a name or id"))?;
            profile = Some(value.to_owned());
            index += 1;
        } else if !positional && arg == "--with" {
            let value = args
                .get(index + 1)
                .ok_or_else(|| ParseError::message("`--with` needs a component id"))?;
            let value = value
                .to_str()
                .ok_or_else(|| ParseError::message("the component id must be valid UTF-8"))?;
            components.push(value.to_owned());
            index += 1;
        } else if !positional && arg == "--skill" {
            let value = args
                .get(index + 1)
                .ok_or_else(|| ParseError::message("`--skill` needs a number from 1 to 5"))?;
            let value = value
                .to_str()
                .ok_or_else(|| ParseError::message("`--skill` must be a number from 1 to 5"))?;
            let parsed: u8 = value
                .parse()
                .map_err(|_| ParseError::message("`--skill` must be a number from 1 to 5"))?;
            if !(1..=5).contains(&parsed) {
                return Err(ParseError::message(
                    "`--skill` must be a number from 1 to 5",
                ));
            }
            skill = Some(parsed);
            index += 1;
        } else if !positional && arg.to_string_lossy().starts_with('-') {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        } else if id.is_none() {
            id = Some(arg.to_string_lossy().into_owned());
        } else {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        }
        index += 1;
    }
    let id = id.ok_or_else(|| ParseError::message("`launch` needs a recipe id"))?;
    Ok(Command::Launch {
        id,
        dry_run,
        profile,
        components,
        skill,
        pistol_start,
        accept_changes,
    })
}

fn parse_custom(args: &[OsString]) -> Result<Command, ParseError> {
    match args.first().and_then(|arg| arg.to_str()) {
        Some("list") => {
            if wants_help(&args[1..]) {
                return Ok(Command::Help(HelpTopic::CustomList));
            }
            reject_extra_positionals(&args[1..])?;
            Ok(Command::Custom(CustomCommand::List))
        }
        Some("show") => {
            if wants_help(&args[1..]) {
                return Ok(Command::Help(HelpTopic::CustomShow));
            }
            Ok(Command::Custom(CustomCommand::Show {
                id: one_id(&args[1..], "`custom show <id>`")?,
            }))
        }
        Some("create") => parse_custom_create(&args[1..]),
        Some("add-file") => {
            if wants_help(&args[1..]) {
                return Ok(Command::Help(HelpTopic::CustomAddFile));
            }
            let words = positionals(&args[1..])?;
            let (id, paths) = words
                .split_first()
                .ok_or_else(|| ParseError::message("`custom add-file` needs a setup id"))?;
            if paths.is_empty() {
                return Err(ParseError::message(
                    "`custom add-file` needs at least one file path",
                ));
            }
            Ok(Command::Custom(CustomCommand::AddFile {
                id: id.clone(),
                paths: paths.iter().map(PathBuf::from).collect(),
            }))
        }
        Some("move") => {
            if wants_help(&args[1..]) {
                return Ok(Command::Help(HelpTopic::CustomMove));
            }
            let words = positionals(&args[1..])?;
            let [id, group, from, to] = words.as_slice() else {
                return Err(ParseError::message(
                    "`custom move` needs `<id> <files|patches> <from> <to>`",
                ));
            };
            Ok(Command::Custom(CustomCommand::Move {
                id: id.clone(),
                group: parse_group(group)?,
                from: parse_index(from)?,
                to: parse_index(to)?,
            }))
        }
        Some("remove-file") => {
            if wants_help(&args[1..]) {
                return Ok(Command::Help(HelpTopic::CustomRemoveFile));
            }
            let words = positionals(&args[1..])?;
            let [id, group, index] = words.as_slice() else {
                return Err(ParseError::message(
                    "`custom remove-file` needs `<id> <files|patches> <index>`",
                ));
            };
            Ok(Command::Custom(CustomCommand::RemoveFile {
                id: id.clone(),
                group: parse_group(group)?,
                index: parse_index(index)?,
            }))
        }
        Some("delete") => {
            let mut id = None;
            let mut delete_profiles = false;
            let mut positional = false;
            for arg in &args[1..] {
                if !positional && arg == "--" {
                    positional = true;
                } else if !positional && is_one_of(arg, &["-h", "--help"]) {
                    return Ok(Command::Help(HelpTopic::CustomDelete));
                } else if !positional && arg == "--delete-profiles" {
                    delete_profiles = true;
                } else if !positional && arg.to_string_lossy().starts_with('-') {
                    return Err(ParseError::unknown(&arg.to_string_lossy()));
                } else if id.is_none() {
                    id = Some(arg.to_string_lossy().into_owned());
                } else {
                    return Err(ParseError::unknown(&arg.to_string_lossy()));
                }
            }
            Ok(Command::Custom(CustomCommand::Delete {
                id: id.ok_or_else(|| ParseError::message("`custom delete` needs a setup id"))?,
                delete_profiles,
            }))
        }
        _ if wants_help(args) => Ok(Command::Help(HelpTopic::Custom)),
        _ => Err(ParseError::message(
            "expected `custom list`, `custom show <id>`, `custom create <name>`, `custom add-file <id> <path>`, `custom move <id> <group> <from> <to>`, `custom remove-file <id> <group> <index>` or `custom delete <id>`",
        )),
    }
}

fn parse_custom_create(args: &[OsString]) -> Result<Command, ParseError> {
    let mut name = None;
    let mut id = None;
    let mut iwad = None;
    let mut iwad_file = None;
    let mut compatibility = None;
    let mut positional = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && is_one_of(arg, &["-h", "--help"]) {
            return Ok(Command::Help(HelpTopic::CustomCreate));
        } else if !positional && arg == "--id" {
            id = Some(option_value(args, index, "--id")?.to_owned());
            index += 1;
        } else if !positional && arg == "--iwad" {
            iwad = Some(option_value(args, index, "--iwad")?.to_owned());
            index += 1;
        } else if !positional && arg == "--iwad-file" {
            iwad_file = Some(PathBuf::from(option_value(args, index, "--iwad-file")?));
            index += 1;
        } else if !positional && arg == "--complevel" {
            compatibility = Some(option_value(args, index, "--complevel")?.to_owned());
            index += 1;
        } else if !positional && arg.to_string_lossy().starts_with('-') {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        } else if name.is_none() {
            name = Some(arg.to_string_lossy().into_owned());
        } else {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        }
        index += 1;
    }
    let name = name.ok_or_else(|| ParseError::message("`custom create` needs a name"))?;
    match (&iwad, &iwad_file) {
        (None, None) => {
            return Err(ParseError::message(
                "`custom create` needs `--iwad <variant>` or `--iwad-file <path>`",
            ));
        }
        (Some(_), Some(_)) => {
            return Err(ParseError::message(
                "`custom create` takes `--iwad` or `--iwad-file`, not both",
            ));
        }
        _ => {}
    }
    Ok(Command::Custom(CustomCommand::Create {
        name,
        id,
        iwad,
        iwad_file,
        compatibility,
    }))
}

fn parse_profile(args: &[OsString]) -> Result<Command, ParseError> {
    match args.first().and_then(|arg| arg.to_str()) {
        Some("extras") => parse_profile_extras(&args[1..]),
        _ if wants_help(args) => Ok(Command::Help(HelpTopic::Profile)),
        _ => Err(ParseError::message(
            "expected `profile extras <recipe> ...`",
        )),
    }
}

fn parse_profile_extras(args: &[OsString]) -> Result<Command, ParseError> {
    let mut profile = None;
    let mut rest: Vec<String> = Vec::new();
    let mut positional = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && is_one_of(arg, &["-h", "--help"]) {
            return Ok(Command::Help(HelpTopic::ProfileExtras));
        } else if !positional && arg == "--profile" {
            profile = Some(option_value(args, index, "--profile")?.to_owned());
            index += 1;
        } else if !positional && arg.to_string_lossy().starts_with('-') {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        } else {
            rest.push(arg.to_string_lossy().into_owned());
        }
        index += 1;
    }
    let (recipe, after_recipe) = rest
        .split_first()
        .ok_or_else(|| ParseError::message("`profile extras` needs a recipe id"))?;
    let (action, action_args) = after_recipe
        .split_first()
        .ok_or_else(|| ParseError::message("`profile extras` needs an action"))?;
    let command = match action.as_str() {
        "list" if action_args.is_empty() => ProfileExtrasCommand::List {
            recipe: recipe.clone(),
            profile,
        },
        "clear" if action_args.is_empty() => ProfileExtrasCommand::Clear {
            recipe: recipe.clone(),
            profile,
        },
        "add" => {
            if action_args.is_empty() {
                return Err(ParseError::message(
                    "`profile extras add` needs at least one file path",
                ));
            }
            ProfileExtrasCommand::Add {
                recipe: recipe.clone(),
                profile,
                paths: action_args.iter().map(PathBuf::from).collect(),
            }
        }
        "move" => {
            let [group, from, to] = action_args else {
                return Err(ParseError::message(
                    "`profile extras move` needs `<files|patches> <from> <to>`",
                ));
            };
            ProfileExtrasCommand::Move {
                recipe: recipe.clone(),
                profile,
                group: parse_group(group)?,
                from: parse_index(from)?,
                to: parse_index(to)?,
            }
        }
        "remove" => {
            let [group, index] = action_args else {
                return Err(ParseError::message(
                    "`profile extras remove` needs `<files|patches> <index>`",
                ));
            };
            ProfileExtrasCommand::Remove {
                recipe: recipe.clone(),
                profile,
                group: parse_group(group)?,
                index: parse_index(index)?,
            }
        }
        _ => {
            return Err(ParseError::message(
                "expected `profile extras <recipe> [--profile <name|id>] <list|add|move|remove|clear>`",
            ));
        }
    };
    Ok(Command::ProfileExtras(command))
}

/// Parses `files` or `patches`.
fn parse_group(value: &str) -> Result<GroupArg, ParseError> {
    GroupArg::parse(value).ok_or_else(|| {
        ParseError::message(format!(
            "`{value}` is not a group; use `files` or `patches`"
        ))
    })
}

/// Parses a non-negative index.
fn parse_index(value: &str) -> Result<usize, ParseError> {
    value
        .parse()
        .map_err(|_| ParseError::message(format!("`{value}` is not a valid index")))
}

/// Every non-option argument, honouring a `--` separator.
fn positionals(args: &[OsString]) -> Result<Vec<String>, ParseError> {
    let mut positional = false;
    let mut words = Vec::new();
    for arg in args {
        if !positional && arg == "--" {
            positional = true;
        } else if !positional && arg.to_string_lossy().starts_with('-') {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        } else {
            words.push(arg.to_string_lossy().into_owned());
        }
    }
    Ok(words)
}

/// Rejects any positional argument at all.
fn reject_extra_positionals(args: &[OsString]) -> Result<(), ParseError> {
    for arg in args {
        if arg != "--" {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        }
    }
    Ok(())
}

/// Exactly one positional id.
fn one_id(args: &[OsString], usage: &str) -> Result<String, ParseError> {
    let words = positionals(args)?;
    match words.as_slice() {
        [id] => Ok(id.clone()),
        _ => Err(ParseError::message(format!("{usage} takes exactly one id"))),
    }
}

fn parse_doctor(args: &[OsString]) -> Result<Command, ParseError> {
    if wants_help(args) {
        return Ok(Command::Help(HelpTopic::Doctor));
    }
    for arg in args {
        if arg != "--" {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        }
    }
    Ok(Command::Doctor)
}

/// True when `-h`/`--help` appears anywhere in `args`.
fn wants_help(args: &[OsString]) -> bool {
    args.iter().any(|arg| is_one_of(arg, &["-h", "--help"]))
}

/// True when `arg` is valid UTF-8 and exactly matches one of `names`.
fn is_one_of(arg: &OsString, names: &[&str]) -> bool {
    arg.to_str().is_some_and(|value| names.contains(&value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(args: &[&str]) -> Result<Command, ParseError> {
        parse(args.iter().map(OsString::from))
    }

    #[test]
    fn catalog_lint_parses_defaults_fetch_paths_and_separator() {
        assert_eq!(
            parse_str(&["catalog", "lint"]),
            Ok(Command::CatalogLint {
                paths: vec!["catalog".into()],
                fetch: false
            })
        );
        assert_eq!(
            parse_str(&["catalog", "lint", "a.toml", "--fetch", "b.toml"]),
            Ok(Command::CatalogLint {
                paths: vec!["a.toml".into(), "b.toml".into()],
                fetch: true
            })
        );
        assert_eq!(
            parse_str(&["catalog", "lint", "--", "-literal.toml"]),
            Ok(Command::CatalogLint {
                paths: vec!["-literal.toml".into()],
                fetch: false
            })
        );
        for args in [
            &["catalog"][..],
            &["catalog", "fetch"],
            &["catalog", "lint", "--unknown"],
        ] {
            assert!(parse_str(args).is_err());
        }
    }

    #[test]
    fn catalog_check_links_parses_its_options() {
        assert_eq!(
            parse_str(&["catalog", "check-links"]),
            Ok(Command::CatalogCheckLinks {
                dir: "catalog".into(),
                json: false,
                problems_only: false
            })
        );
        assert_eq!(
            parse_str(&[
                "catalog",
                "check-links",
                "other",
                "--json",
                "--problems-only"
            ]),
            Ok(Command::CatalogCheckLinks {
                dir: "other".into(),
                json: true,
                problems_only: true
            })
        );
        assert_eq!(
            parse_str(&["catalog", "check-links", "--help"]),
            Ok(Command::Help(HelpTopic::CatalogCheckLinks))
        );
        for args in [
            &["catalog", "check-links", "--unknown"][..],
            &["catalog", "check-links", "a", "b"],
        ] {
            assert!(parse_str(args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn catalog_e2e_sample_parses_its_options() {
        assert_eq!(
            parse_str(&["catalog", "e2e-sample"]),
            Ok(Command::CatalogE2eSample {
                dir: "catalog".into(),
                max_mib: 64
            })
        );
        assert_eq!(
            parse_str(&["catalog", "e2e-sample", "--max-mib", "32", "other"]),
            Ok(Command::CatalogE2eSample {
                dir: "other".into(),
                max_mib: 32
            })
        );
        for args in [
            &["catalog", "e2e-sample", "--max-mib"][..],
            &["catalog", "e2e-sample", "--max-mib", "big"],
            &["catalog", "e2e-sample", "--nope"],
        ] {
            assert!(parse_str(args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn catalog_bundle_parses_its_options_and_requires_out() {
        assert_eq!(
            parse_str(&["catalog", "bundle", "--out", "bundle.json"]),
            Ok(Command::CatalogBundle {
                out: "bundle.json".into(),
                catalog: "catalog".into(),
                serial: None,
                commit: None,
                min_ddl_version: None
            })
        );
        assert_eq!(
            parse_str(&[
                "catalog",
                "bundle",
                "--out",
                "/tmp/b.json",
                "--catalog",
                "c",
                "--serial",
                "1700000000",
                "--commit",
                "abc1234",
                "--min-ddl-version",
                "0.0.4"
            ]),
            Ok(Command::CatalogBundle {
                out: "/tmp/b.json".into(),
                catalog: "c".into(),
                serial: Some(1_700_000_000),
                commit: Some("abc1234".into()),
                min_ddl_version: Some("0.0.4".into())
            })
        );
        assert_eq!(
            parse_str(&["catalog", "bundle", "--help"]),
            Ok(Command::Help(HelpTopic::CatalogBundle))
        );
        for args in [
            vec!["catalog", "bundle"],
            vec!["catalog", "bundle", "--out"],
            vec!["catalog", "bundle", "--out", "b", "--serial", "soon"],
            vec!["catalog", "bundle", "--out", "b", "--nope"],
        ] {
            assert!(parse_str(&args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn catalog_status_and_update_parse() {
        assert_eq!(
            parse_str(&["catalog", "status"]),
            Ok(Command::CatalogStatus)
        );
        assert_eq!(
            parse_str(&["catalog", "update"]),
            Ok(Command::CatalogUpdate)
        );
        assert_eq!(
            parse_str(&["catalog", "status", "--help"]),
            Ok(Command::Help(HelpTopic::CatalogStatus))
        );
        assert_eq!(
            parse_str(&["catalog", "update", "--help"]),
            Ok(Command::Help(HelpTopic::CatalogUpdate))
        );
        assert_eq!(
            parse_str(&["catalog", "--help"]),
            Ok(Command::Help(HelpTopic::Catalog))
        );
        for args in [
            vec!["catalog", "status", "extra"],
            vec!["catalog", "update", "--bad"],
        ] {
            assert!(parse_str(&args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn no_arguments_runs_fullscreen() {
        assert_eq!(
            parse_str(&[]),
            Ok(Command::Run(Options { windowed: false }))
        );
    }

    #[test]
    fn windowed_flag_is_accepted() {
        assert_eq!(
            parse_str(&["--windowed"]),
            Ok(Command::Run(Options { windowed: true }))
        );
    }

    #[test]
    fn repeated_windowed_flag_is_harmless() {
        assert_eq!(
            parse_str(&["--windowed", "--windowed"]),
            Ok(Command::Run(Options { windowed: true }))
        );
    }

    #[test]
    fn version_flags_are_recognised() {
        assert_eq!(parse_str(&["--version"]), Ok(Command::Version));
        assert_eq!(parse_str(&["-V"]), Ok(Command::Version));
    }

    #[test]
    fn help_flags_are_recognised() {
        assert_eq!(parse_str(&["--help"]), Ok(Command::Help(HelpTopic::Global)));
        assert_eq!(parse_str(&["-h"]), Ok(Command::Help(HelpTopic::Global)));
    }

    #[test]
    fn help_wins_over_other_arguments() {
        assert_eq!(
            parse_str(&["--windowed", "--help"]),
            Ok(Command::Help(HelpTopic::Global))
        );
        assert_eq!(
            parse_str(&["--nope", "--help"]),
            Ok(Command::Help(HelpTopic::Global))
        );
    }

    #[test]
    fn version_wins_over_run_options_but_not_help() {
        assert_eq!(
            parse_str(&["--windowed", "--version"]),
            Ok(Command::Version)
        );
        assert_eq!(
            parse_str(&["--version", "--help"]),
            Ok(Command::Help(HelpTopic::Global))
        );
    }

    #[test]
    fn unknown_long_option_is_rejected_with_a_message() {
        let error = parse_str(&["--nope"]).expect_err("unknown option");
        assert_eq!(error.to_string(), "unknown option `--nope`");
    }

    #[test]
    fn unknown_short_option_is_rejected() {
        let error = parse_str(&["-x"]).expect_err("unknown option");
        assert_eq!(error.to_string(), "unknown option `-x`");
    }

    #[test]
    fn typo_of_a_known_option_is_rejected() {
        let error = parse_str(&["--windwoed"]).expect_err("typo");
        assert!(error.to_string().starts_with("unknown option"));
    }

    #[test]
    fn positional_arguments_are_rejected() {
        let error = parse_str(&["play"]).expect_err("stray argument");
        assert_eq!(error.to_string(), "unexpected argument `play`");
    }

    #[test]
    fn engine_status_and_install_parse() {
        assert_eq!(parse_str(&["engine", "status"]), Ok(Command::EngineStatus));
        assert_eq!(
            parse_str(&["engine", "install"]),
            Ok(Command::EngineInstall {
                version: None,
                force_extract: false
            })
        );
        assert_eq!(
            parse_str(&[
                "engine",
                "install",
                "--version",
                "0.30.1",
                "--force-extract"
            ]),
            Ok(Command::EngineInstall {
                version: Some("0.30.1".into()),
                force_extract: true
            })
        );
        // `--version` here is the engine's, not DDL's.
        assert_ne!(
            parse_str(&["engine", "install", "--version", "0.30.1"]),
            Ok(Command::Version)
        );
        assert!(parse_str(&["engine", "install", "--version"]).is_err());
        assert!(parse_str(&["engine", "frobnicate"]).is_err());
        assert!(parse_str(&["engine"]).is_err());
    }

    #[test]
    fn engine_help_topics_are_distinct() {
        assert_eq!(
            parse_str(&["engine", "--help"]),
            Ok(Command::Help(HelpTopic::Engine))
        );
        assert_eq!(
            parse_str(&["engine", "status", "--help"]),
            Ok(Command::Help(HelpTopic::EngineStatus))
        );
        assert_eq!(
            parse_str(&["engine", "install", "--help"]),
            Ok(Command::Help(HelpTopic::EngineInstall))
        );
    }

    #[test]
    fn iwad_scan_parses_paths_and_separator() {
        assert_eq!(
            parse_str(&["iwad", "scan"]),
            Ok(Command::IwadScan { paths: vec![] })
        );
        assert_eq!(
            parse_str(&["iwad", "scan", "/a", "b dir"]),
            Ok(Command::IwadScan {
                paths: vec!["/a".into(), "b dir".into()]
            })
        );
        assert_eq!(
            parse_str(&["iwad", "scan", "--", "-weird"]),
            Ok(Command::IwadScan {
                paths: vec!["-weird".into()]
            })
        );
        assert_eq!(
            parse_str(&["iwad", "--help"]),
            Ok(Command::Help(HelpTopic::Iwad))
        );
        assert_eq!(
            parse_str(&["iwad", "scan", "--help"]),
            Ok(Command::Help(HelpTopic::IwadScan))
        );
        assert!(parse_str(&["iwad"]).is_err());
    }

    #[test]
    fn mapset_list_and_install_parse() {
        assert_eq!(parse_str(&["mapset", "list"]), Ok(Command::MapsetList));
        assert_eq!(
            parse_str(&["mapset", "install", "sigil"]),
            Ok(Command::MapsetInstall {
                id: "sigil".into(),
                components: vec![]
            })
        );
        assert_eq!(
            parse_str(&["mapset", "install", "sigil", "--with", "compat"]),
            Ok(Command::MapsetInstall {
                id: "sigil".into(),
                components: vec!["compat".into()]
            })
        );
        assert!(parse_str(&["mapset", "install"]).is_err());
        assert!(parse_str(&["mapset", "install", "sigil", "extra"]).is_err());
        assert!(parse_str(&["mapset", "install", "sigil", "--with"]).is_err());
        assert_eq!(
            parse_str(&["mapset", "list", "--help"]),
            Ok(Command::Help(HelpTopic::MapsetList))
        );
    }

    #[test]
    fn launch_parses_flags_in_any_order() {
        assert_eq!(
            parse_str(&["launch", "sigil", "--dry-run"]),
            Ok(Command::Launch {
                id: "sigil".into(),
                dry_run: true,
                profile: None,
                components: vec![],
                skill: None,
                pistol_start: false,
                accept_changes: false
            })
        );
        assert_eq!(
            parse_str(&[
                "launch",
                "--skill",
                "4",
                "auger-zenith",
                "--with",
                "alternate-blaster",
                "--pistol-start"
            ]),
            Ok(Command::Launch {
                id: "auger-zenith".into(),
                dry_run: false,
                profile: None,
                components: vec!["alternate-blaster".into()],
                skill: Some(4),
                pistol_start: true,
                accept_changes: false
            })
        );
        assert!(parse_str(&["launch"]).is_err());
        assert!(parse_str(&["launch", "sigil", "--skill", "9"]).is_err());
        assert!(parse_str(&["launch", "sigil", "--skill"]).is_err());
        assert_eq!(
            parse_str(&["launch", "--help"]),
            Ok(Command::Help(HelpTopic::Launch))
        );
    }

    #[test]
    fn launch_profile_accepts_names_and_ids_and_rejects_missing_or_repeated_values() {
        for selector in ["Casual", "Profile 2", "sigil-default"] {
            for args in [
                vec!["launch", "sigil", "--profile", selector],
                vec!["launch", "--profile", selector, "sigil"],
            ] {
                assert!(
                    matches!(parse_str(&args), Ok(Command::Launch { profile: Some(p), .. }) if p == selector)
                );
            }
        }
        for args in [
            vec!["launch", "sigil", "--profile"],
            vec!["launch", "sigil", "--profile", ""],
            vec!["launch", "sigil", "--profile", "--dry-run"],
            vec!["launch", "sigil", "--profile", "UV", "--profile", "Casual"],
        ] {
            assert!(parse_str(&args).is_err());
        }
        assert!(help_for(HelpTopic::Launch).contains("--profile"));
    }

    #[test]
    fn doctor_parses() {
        assert_eq!(parse_str(&["doctor"]), Ok(Command::Doctor));
        assert_eq!(
            parse_str(&["doctor", "--help"]),
            Ok(Command::Help(HelpTopic::Doctor))
        );
        assert!(parse_str(&["doctor", "extra"]).is_err());
    }

    #[test]
    fn version_line_starts_with_the_binary_name_and_version() {
        assert_eq!(version_line(), format!("ddl {VERSION}"));
    }

    #[test]
    fn help_text_lists_every_option() {
        let help = help_text();
        assert!(help.contains("Usage: ddl [OPTIONS]"));
        assert!(help.contains("--windowed"));
        assert!(help.contains("--help"));
        assert!(help.contains("--version"));
        assert!(help.contains("DDL_LOG"));
        assert!(help.contains("catalog lint [--fetch]"));
        assert!(help.contains("engine install"));
        assert!(help.contains("iwad scan"));
        assert!(help.contains("mapset install"));
        assert!(help.contains("launch <id>"));
        assert!(help.contains("doctor"));
    }

    #[test]
    fn per_topic_help_mentions_its_own_command() {
        assert!(help_for(HelpTopic::EngineInstall).contains("--force-extract"));
        assert!(help_for(HelpTopic::IwadScan).contains("iwad scan"));
        assert!(help_for(HelpTopic::Launch).contains("--dry-run"));
        for topic in [
            HelpTopic::Global,
            HelpTopic::Engine,
            HelpTopic::EngineStatus,
            HelpTopic::EngineInstall,
            HelpTopic::Iwad,
            HelpTopic::IwadScan,
            HelpTopic::Mapset,
            HelpTopic::MapsetList,
            HelpTopic::MapsetInstall,
            HelpTopic::Launch,
            HelpTopic::Doctor,
            HelpTopic::Catalog,
            HelpTopic::CatalogBundle,
            HelpTopic::CatalogCheckLinks,
            HelpTopic::CatalogE2eSample,
            HelpTopic::CatalogStatus,
            HelpTopic::CatalogUpdate,
            HelpTopic::Custom,
            HelpTopic::CustomList,
            HelpTopic::CustomShow,
            HelpTopic::CustomCreate,
            HelpTopic::CustomAddFile,
            HelpTopic::CustomMove,
            HelpTopic::CustomRemoveFile,
            HelpTopic::CustomDelete,
            HelpTopic::Profile,
            HelpTopic::ProfileExtras,
        ] {
            let text = help_for(topic);
            assert!(text.starts_with("ddl"), "{topic:?}: {text}");
            assert!(text.contains("Usage:"), "{topic:?}");
        }
    }
}

#[cfg(test)]
mod recipe_change_tests {
    use super::*;
    fn parse_args(args: &[&str]) -> Result<Command, ParseError> {
        parse(args.iter().map(OsString::from))
    }
    #[test]
    fn mapset_status_and_update_follow_positional_style() {
        assert_eq!(
            parse_args(&["mapset", "status"]),
            Ok(Command::MapsetStatus { id: None })
        );
        assert_eq!(
            parse_args(&["mapset", "status", "sigil"]),
            Ok(Command::MapsetStatus {
                id: Some("sigil".into())
            })
        );
        assert_eq!(
            parse_args(&["mapset", "update", "sigil"]),
            Ok(Command::MapsetUpdate { id: "sigil".into() })
        );
        assert_eq!(
            parse_args(&["mapset", "status", "--help"]),
            Ok(Command::Help(HelpTopic::MapsetStatus))
        );
        assert_eq!(
            parse_args(&["mapset", "update", "--help"]),
            Ok(Command::Help(HelpTopic::MapsetUpdate))
        );
        for args in [
            vec!["mapset", "update"],
            vec!["mapset", "update", "sigil", "extra"],
            vec!["mapset", "status", "--bad"],
        ] {
            assert!(parse_args(&args).is_err());
        }
    }

    #[test]
    fn mapset_uninstall_parses_the_optional_profile_flag() {
        assert_eq!(
            parse_args(&["mapset", "uninstall", "sigil"]),
            Ok(Command::MapsetUninstall {
                id: "sigil".into(),
                delete_profiles: false
            })
        );
        assert_eq!(
            parse_args(&["mapset", "uninstall", "--delete-profiles", "sigil"]),
            Ok(Command::MapsetUninstall {
                id: "sigil".into(),
                delete_profiles: true
            })
        );
        assert_eq!(
            parse_args(&["mapset", "uninstall", "sigil", "--help"]),
            Ok(Command::Help(HelpTopic::MapsetUninstall))
        );
        assert!(help_for(HelpTopic::MapsetUninstall).contains("--delete-profiles"));
        for args in [
            vec!["mapset", "uninstall"],
            vec!["mapset", "uninstall", "sigil", "extra"],
            vec!["mapset", "uninstall", "--bad", "sigil"],
            vec!["mapset", "uninstall", "sigil", "--with", "compat"],
        ] {
            assert!(parse_args(&args).is_err(), "{args:?}");
        }
    }
}

#[cfg(test)]
mod local_content_tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Result<Command, ParseError> {
        parse(args.iter().map(OsString::from))
    }

    #[test]
    fn custom_subcommands_parse_into_typed_commands() {
        assert_eq!(
            parse_args(&["custom", "list"]),
            Ok(Command::Custom(CustomCommand::List))
        );
        assert_eq!(
            parse_args(&["custom", "show", "my-setup"]),
            Ok(Command::Custom(CustomCommand::Show {
                id: "my-setup".into()
            }))
        );
        assert_eq!(
            parse_args(&[
                "custom",
                "create",
                "My Setup",
                "--iwad",
                "doom2-1.9",
                "--complevel",
                "auto"
            ]),
            Ok(Command::Custom(CustomCommand::Create {
                name: "My Setup".into(),
                id: None,
                iwad: Some("doom2-1.9".into()),
                iwad_file: None,
                compatibility: Some("auto".into())
            }))
        );
        assert_eq!(
            parse_args(&[
                "custom",
                "create",
                "Mine",
                "--id",
                "mine",
                "--iwad-file",
                "/games/DOOM2.WAD",
                "--complevel",
                "21"
            ]),
            Ok(Command::Custom(CustomCommand::Create {
                name: "Mine".into(),
                id: Some("mine".into()),
                iwad: None,
                iwad_file: Some("/games/DOOM2.WAD".into()),
                compatibility: Some("21".into())
            }))
        );
        assert_eq!(
            parse_args(&["custom", "add-file", "mine", "a.wad", "b.deh"]),
            Ok(Command::Custom(CustomCommand::AddFile {
                id: "mine".into(),
                paths: vec!["a.wad".into(), "b.deh".into()]
            }))
        );
        assert_eq!(
            parse_args(&["custom", "move", "mine", "files", "0", "2"]),
            Ok(Command::Custom(CustomCommand::Move {
                id: "mine".into(),
                group: GroupArg::Files,
                from: 0,
                to: 2
            }))
        );
        assert_eq!(
            parse_args(&["custom", "remove-file", "mine", "patches", "1"]),
            Ok(Command::Custom(CustomCommand::RemoveFile {
                id: "mine".into(),
                group: GroupArg::Patches,
                index: 1
            }))
        );
        assert_eq!(
            parse_args(&["custom", "delete", "--delete-profiles", "mine"]),
            Ok(Command::Custom(CustomCommand::Delete {
                id: "mine".into(),
                delete_profiles: true
            }))
        );
    }

    #[test]
    fn custom_help_topics_and_rejections() {
        assert_eq!(
            parse_args(&["custom", "--help"]),
            Ok(Command::Help(HelpTopic::Custom))
        );
        assert_eq!(
            parse_args(&["custom", "create", "--help"]),
            Ok(Command::Help(HelpTopic::CustomCreate))
        );
        for args in [
            vec!["custom"],
            vec!["custom", "frobnicate"],
            vec!["custom", "show"],
            vec!["custom", "show", "a", "b"],
            vec!["custom", "create", "name"],
            vec![
                "custom",
                "create",
                "name",
                "--iwad",
                "v",
                "--iwad-file",
                "p",
            ],
            vec!["custom", "add-file", "mine"],
            vec!["custom", "move", "mine", "files", "0"],
            vec!["custom", "move", "mine", "wads", "0", "1"],
            vec!["custom", "remove-file", "mine", "files", "x"],
            vec!["custom", "delete"],
            vec!["custom", "delete", "mine", "--bad"],
        ] {
            assert!(parse_args(&args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn profile_extras_subcommands_parse_into_typed_commands() {
        assert_eq!(
            parse_args(&["profile", "extras", "sigil", "list"]),
            Ok(Command::ProfileExtras(ProfileExtrasCommand::List {
                recipe: "sigil".into(),
                profile: None
            }))
        );
        assert_eq!(
            parse_args(&[
                "profile",
                "extras",
                "sigil",
                "--profile",
                "UV",
                "add",
                "a.wad"
            ]),
            Ok(Command::ProfileExtras(ProfileExtrasCommand::Add {
                recipe: "sigil".into(),
                profile: Some("UV".into()),
                paths: vec!["a.wad".into()]
            }))
        );
        assert_eq!(
            parse_args(&[
                "profile",
                "extras",
                "--profile",
                "UV",
                "sigil",
                "move",
                "files",
                "0",
                "1"
            ]),
            Ok(Command::ProfileExtras(ProfileExtrasCommand::Move {
                recipe: "sigil".into(),
                profile: Some("UV".into()),
                group: GroupArg::Files,
                from: 0,
                to: 1
            }))
        );
        assert_eq!(
            parse_args(&["profile", "extras", "sigil", "remove", "patches", "0"]),
            Ok(Command::ProfileExtras(ProfileExtrasCommand::Remove {
                recipe: "sigil".into(),
                profile: None,
                group: GroupArg::Patches,
                index: 0
            }))
        );
        assert_eq!(
            parse_args(&["profile", "extras", "sigil", "clear"]),
            Ok(Command::ProfileExtras(ProfileExtrasCommand::Clear {
                recipe: "sigil".into(),
                profile: None
            }))
        );
        assert_eq!(
            parse_args(&["profile", "extras", "--help"]),
            Ok(Command::Help(HelpTopic::ProfileExtras))
        );
        for args in [
            vec!["profile"],
            vec!["profile", "extras"],
            vec!["profile", "extras", "sigil"],
            vec!["profile", "extras", "sigil", "add"],
            vec!["profile", "extras", "sigil", "move", "files", "0"],
            vec!["profile", "extras", "sigil", "remove", "patches"],
            vec!["profile", "extras", "sigil", "clear", "extra"],
            vec!["profile", "extras", "sigil", "--profile"],
        ] {
            assert!(parse_args(&args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn launch_accepts_accept_changes() {
        assert_eq!(
            parse_args(&["launch", "mine", "--accept-changes"]),
            Ok(Command::Launch {
                id: "mine".into(),
                dry_run: false,
                profile: None,
                components: vec![],
                skill: None,
                pistol_start: false,
                accept_changes: true
            })
        );
    }
}

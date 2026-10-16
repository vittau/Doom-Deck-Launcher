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
    /// `ddl launch --help`.
    Launch,
    /// `ddl doctor --help`.
    Doctor,
}

/// What the user asked `ddl` to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Start the launcher with the given options.
    Run(Options),
    /// Offline by default; --fetch explicitly verifies upstream package bytes.
    CatalogLint { paths: Vec<PathBuf>, fetch: bool },
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
    IwadScan { paths: Vec<PathBuf> },
    /// `ddl mapset list`.
    MapsetList,
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
        /// Optional component ids to enable (`--with`).
        components: Vec<String>,
        /// Starting skill override (`--skill`).
        skill: Option<u8>,
        /// Reset to a pistol start on every map (`--pistol-start`).
        pistol_start: bool,
    },
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
       {BIN} engine status
       {BIN} engine install [--version <V>] [--force-extract]
       {BIN} iwad scan [PATH...]
       {BIN} mapset list
       {BIN} mapset install <id> [--with <component>...]
       {BIN} launch <id> [--dry-run] [--with <component>...] [--skill <1-5>] [--pistol-start]
       {BIN} doctor

Commands:
  catalog lint  Validate catalog recipes (offline; --fetch verifies upstream bytes)
  engine        Install or inspect the DSDA-Doom engine
  iwad scan     Find and identify your IWAD files
  mapset        List or install catalog mapsets
  launch        Resolve a mapset; without --dry-run, run it
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

Discover IWADs in the Steam libraries and in each PATH (which is added to
DDL's folders), then print every identified file with its variant, game and
SHA-256. Ambiguous files are listed with their candidates and are never
chosen automatically. DDL never downloads IWADs.

Usage: {BIN} iwad scan [PATH...]
"
        ),
        HelpTopic::Mapset => format!(
            "\
{BIN} mapset: list and install catalog mapsets

Usage: {BIN} mapset list
       {BIN} mapset install <id> [--with <component>...]

Commands:
  list     List every catalog recipe with its install state
  install  Install a mapset, optionally enabling components
"
        ),
        HelpTopic::MapsetList => format!(
            "\
{BIN} mapset list

List every catalog recipe with its id, title, base game and install state.

Usage: {BIN} mapset list
"
        ),
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
{BIN} launch <id> [--dry-run] [--with <component>...] [--skill <1-5>] [--pistol-start]

Resolve a mapset exactly as the UI's Play does. With --dry-run it prints the
argv (one argument per line) and the working directory, or the typed blocker
with its explanation. Without --dry-run it starts DSDA-Doom through the same
run guard as the UI.

Usage: {BIN} launch <id> [--dry-run] [--with <component>...] [--skill <1-5>] [--pistol-start]

Options:
      --dry-run        Print the launch plan instead of starting the engine
      --with <C>       Enable an optional component (repeatable)
      --skill <1-5>    Starting skill
      --pistol-start   Reset to a pistol start on every map
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
    if !args.first().is_some_and(|arg| arg == "lint") {
        return Err(ParseError::message(
            "expected `catalog lint [--fetch] [paths]`",
        ));
    }
    if wants_help(&args[1..]) {
        return Ok(Command::Help(HelpTopic::Global));
    }
    let mut paths = Vec::new();
    let mut fetch = false;
    let mut positional = false;
    for arg in &args[1..] {
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
        Some("install") => parse_mapset_install(&args[1..]),
        _ if wants_help(args) => Ok(Command::Help(HelpTopic::Mapset)),
        _ => Err(ParseError::message(
            "expected `mapset list` or `mapset install <id> [--with <component>...]`",
        )),
    }
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
    let mut components = Vec::new();
    let mut skill = None;
    let mut pistol_start = false;
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
        } else if !positional && arg == "--pistol-start" {
            pistol_start = true;
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
                return Err(ParseError::message("`--skill` must be a number from 1 to 5"));
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
        components,
        skill,
        pistol_start,
    })
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
            parse_str(&["engine", "install", "--version", "0.30.1", "--force-extract"]),
            Ok(Command::EngineInstall {
                version: Some("0.30.1".into()),
                force_extract: true
            })
        );
        // `--version` here is the engine's, not DDL's.
        assert_ne!(parse_str(&["engine", "install", "--version", "0.30.1"]), Ok(Command::Version));
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
                components: vec![],
                skill: None,
                pistol_start: false
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
                components: vec!["alternate-blaster".into()],
                skill: Some(4),
                pistol_start: true
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
        ] {
            let text = help_for(topic);
            assert!(text.starts_with("ddl"), "{topic:?}: {text}");
            assert!(text.contains("Usage:"), "{topic:?}");
        }
    }
}

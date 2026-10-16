//! Command-line parsing for the `ddl` binary (spec 001, R3).
//!
//! DDL's surface is three flags wide, so this is a hand-written parser instead
//! of a general argument-parsing dependency: fewer dependencies to audit, and
//! the exact required behaviour (`--version`, `--help`, `--windowed`, unknown
//! options fail) is directly unit-tested in this module.

use std::ffi::OsString;
use std::fmt;

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

/// What the user asked `ddl` to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Start the launcher with the given options.
    Run(Options),
    /// Print [`version_line`] and exit successfully.
    Version,
    /// Print [`help_text`] and exit successfully.
    Help,
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

/// The full `ddl --help` text.
pub fn help_text() -> String {
    format!(
        "\
{BIN}: DoomDeckLauncher, a native DOOM mapset launcher for Steam Deck

Usage: {BIN} [OPTIONS]

Options:
      --windowed  Run in a 1280x800 window instead of fullscreen
  -h, --help      Print this help and exit
  -V, --version   Print version information and exit

Environment:
  DDL_LOG  Log level for stderr diagnostics: error, warn, info, debug or trace
           (default: info)
"
    )
}

/// Parses the arguments after the program name.
///
/// `--help` and `--version` win wherever they appear, as they do in most CLIs;
/// other options are accepted in any order and may be repeated.
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

    if args.iter().any(|arg| is_one_of(arg, &["-h", "--help"])) {
        return Ok(Command::Help);
    }
    if args.iter().any(|arg| is_one_of(arg, &["-V", "--version"])) {
        return Ok(Command::Version);
    }

    let mut options = Options::default();
    for arg in &args {
        if is_one_of(arg, &["--windowed"]) {
            options.windowed = true;
        } else {
            return Err(ParseError::unknown(&arg.to_string_lossy()));
        }
    }
    Ok(Command::Run(options))
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
        assert_eq!(parse_str(&["--help"]), Ok(Command::Help));
        assert_eq!(parse_str(&["-h"]), Ok(Command::Help));
    }

    #[test]
    fn help_wins_over_other_arguments() {
        assert_eq!(parse_str(&["--windowed", "--help"]), Ok(Command::Help));
        assert_eq!(parse_str(&["--nope", "--help"]), Ok(Command::Help));
    }

    #[test]
    fn version_wins_over_run_options_but_not_help() {
        assert_eq!(
            parse_str(&["--windowed", "--version"]),
            Ok(Command::Version)
        );
        assert_eq!(parse_str(&["--version", "--help"]), Ok(Command::Help));
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
    }
}

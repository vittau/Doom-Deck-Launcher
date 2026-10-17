//! Real-binary regression tests for the headless CLI help (spec 014, R1).
//!
//! Every `--help` exits 0 and prints usage; `SLINT_BACKEND` is set to an
//! invalid value so a subcommand that tried to create a window would fail
//! instead of opening one.
use std::{path::Path, process::Command};

fn ddl() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ddl"));
    command.current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    command.env("SLINT_BACKEND", "invalid-backend-proves-headless");
    command
}

#[test]
fn every_subcommand_help_exits_zero_and_prints_usage() {
    let cases: &[(&[&str], &str)] = &[
        (&["--help"], "Usage: ddl [OPTIONS]"),
        (
            &["engine", "--help"],
            "ddl engine: manage the DSDA-Doom engine",
        ),
        (&["engine", "status", "--help"], "ddl engine status"),
        (
            &["engine", "install", "--help"],
            "ddl engine install [--version <V>] [--force-extract]",
        ),
        (
            &["iwad", "--help"],
            "ddl iwad: find and identify IWAD files",
        ),
        (&["iwad", "scan", "--help"], "ddl iwad scan [PATH...]"),
        (
            &["mapset", "--help"],
            "ddl mapset: inspect and manage mapsets",
        ),
        (&["mapset", "list", "--help"], "ddl mapset list"),
        (&["mapset", "status", "--help"], "ddl mapset status [<id>]"),
        (&["mapset", "update", "--help"], "ddl mapset update <id>"),
        (
            &["mapset", "install", "--help"],
            "ddl mapset install <id> [--with <component>...]",
        ),
        (&["launch", "--help"], "ddl launch <id> [--dry-run]"),
        (&["doctor", "--help"], "ddl doctor"),
    ];
    for (args, expected) in cases {
        let output = ddl().args(*args).output().unwrap();
        assert!(
            output.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(expected),
            "{args:?} missing {expected:?}: {stdout}"
        );
    }
}

#[test]
fn unknown_subcommands_are_usage_errors() {
    for args in [
        &["engine"][..],
        &["engine", "frobnicate"],
        &["iwad"],
        &["mapset", "install"],
        &["mapset", "update"],
        &["mapset", "status", "extra", "extra"],
        &["launch"],
        &["doctor", "extra"],
    ] {
        let output = ddl().args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("--help"),
            "{args:?}"
        );
    }
}

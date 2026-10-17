//! Real CLI regression tests; no network and no UI backend/window initialization.
use std::{path::Path, process::Command};

fn ddl() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ddl"));
    command.current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    command.env("SLINT_BACKEND", "invalid-backend-proves-headless");
    command
}

#[test]
fn catalog_lint_defaults_to_all_recipes_without_starting_ui() {
    let output = ddl().args(["catalog", "lint"]).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Catalog lint passed (offline)"));
}

#[test]
fn catalog_lint_failure_has_file_line_hint_and_nonzero_exit() {
    let output = ddl()
        .args([
            "catalog",
            "lint",
            "crates/ddl-core/tests/fixtures/catalog/invalid-hash/sigil.toml",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("sigil.toml:42:"), "{stderr}");
    assert!(stderr.contains("malformed package SHA-256"));
    assert!(stderr.contains("hint:"));
}

#[test]
fn catalog_bundle_and_status_run_headless() {
    let temp = tempfile::tempdir().unwrap();
    let bundle = temp.path().join("catalog.json");
    let output = ddl()
        .env("HOME", temp.path())
        .args([
            "catalog",
            "bundle",
            "--out",
            bundle.to_str().unwrap(),
            "--serial",
            "1700000000",
            "--commit",
            "deadbeef",
            "--min-ddl-version",
            "0.0.1",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Wrote "),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(bundle.is_file());

    // `status` reads only local state, so it works with an empty XDG root.
    let status = ddl()
        .env("HOME", temp.path())
        .args(["catalog", "status"])
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    let stdout = String::from_utf8_lossy(&status.stdout);
    assert!(stdout.contains("Catalog: built-in"), "{stdout}");
    assert!(stdout.contains("Automatic checks: on"), "{stdout}");
}

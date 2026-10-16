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

//! Integration tests for spec 026 R1 against hand-built fixture files.
//!
//! `tests/fixtures/local/` holds a minimal PWAD, a stored ZIP, a DEH patch and a
//! `.pk3`-named copy of the ZIP. None of them is real game data. The test proves
//! DDL detects each kind from content and refuses the copy whose suffix the
//! engine's `-file` has no branch for.

use std::path::{Path, PathBuf};

use ddl_core::local::{FileError, Kind, UserFile};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/local")
        .join(name)
}

#[test]
fn fixture_kinds_are_detected_from_content() {
    assert_eq!(
        UserFile::read(&fixture("tiny.wad")).expect("wad").kind,
        Kind::Wad
    );
    assert_eq!(
        UserFile::read(&fixture("tiny.zip")).expect("zip").kind,
        Kind::Zip
    );
    assert_eq!(
        UserFile::read(&fixture("tiny.deh")).expect("deh").kind,
        Kind::Deh
    );

    // The `.pk3` holds the same ZIP bytes, but the engine's `-file` routing has
    // no `.pk3` branch, so DDL refuses it (spec 026, R1).
    let error = UserFile::read(&fixture("tiny.pk3")).expect_err("pk3 is refused");
    match error {
        FileError::UnsupportedSuffix { suffix, .. } => assert_eq!(suffix, "pk3"),
        other => panic!("expected UnsupportedSuffix, got {other}"),
    }
}

#[test]
fn the_zip_fixture_is_a_real_single_entry_archive() {
    let file = std::fs::File::open(fixture("tiny.zip")).expect("open zip");
    let mut archive = zip::ZipArchive::new(file).expect("valid zip");
    assert_eq!(archive.len(), 1);
    let entry = archive.by_index(0).expect("entry");
    assert_eq!(entry.name().expect("entry name"), "MAP01.lmp");
}

//! Spec 116, R8: fixtures for the path derivation, the safe parser and the rule.

use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::*;

/// The 18-row Abandon fixture: MAP01–MAP12 recorded a skill, MAP13–MAP18 did not.
const ABANDON: &[u8] = include_bytes!("../../tests/fixtures/stats/abandon-18-rows.txt");

/// A valid one-row document with the given map ID, skill and exits.
fn row(id: &str, skill: u8, exits: u32) -> String {
    format!("{id} 1 1 {skill} -1 -1 -1 {exits} 0 0 0 0 -1 -1 -1\n")
}

/// A valid row with explicit kill/secret bests and totals (spec 116, R10).
fn detail_row(
    id: &str,
    skill: u8,
    best_kills: u32,
    max_kills: i32,
    best_secrets: u32,
    max_secrets: i32,
) -> String {
    format!(
        "{id} 1 1 {skill} -1 -1 -1 1 {best_kills} {best_kills} 0 {best_secrets} {max_kills} -1 {max_secrets}\n"
    )
}

/// A document with the two header lines and `rows`.
fn document(rows: &[String]) -> Vec<u8> {
    let mut text = String::from("1\n0\n");
    for row in rows {
        text.push_str(row);
    }
    text.into_bytes()
}

fn data_dir() -> TempDir {
    TempDir::new().expect("temp dir")
}

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

fn layout<'a>(
    data: &'a Path,
    iwad: &'a Path,
    files: &'a [PathBuf],
    patches: &'a [PathBuf],
) -> Layout<'a> {
    Layout {
        profile_data_dir: data,
        iwad,
        files,
        patches,
    }
}

#[test]
fn the_abandon_fixture_yields_twelve_of_seventeen() {
    let stats = parse(ABANDON).expect("the reviewed fixture is valid");
    assert_eq!(stats.rows.len(), 18);
    assert_eq!(stats.total_kills, 7350);

    let recorded = stats.progress(&["MAP18".to_owned()]);
    assert_eq!(recorded.total, 17, "18 detected rows minus MAP18");
    assert_eq!(recorded.completed, 12);
    assert_eq!(recorded.maps.len(), 17);
    assert_eq!(
        recorded.maps[0],
        MapProgress {
            id: "MAP01".to_owned(),
            completed: true,
            total_exits: 2,
            best_skill: Some(4),
            uv_max: true,
            kills_percent: Some(100),
            secrets_percent: Some(100),
        }
    );
    // A no-monsters exit is counted as exits but never as completed.
    let map13 = recorded.maps.iter().find(|map| map.id == "MAP13").unwrap();
    assert_eq!(map13.total_exits, 3);
    assert!(!map13.completed);
    assert_eq!(map13.best_skill, None);
    assert!(!map13.uv_max, "a map with no recorded skill has no trophy");
    assert_eq!(map13.kills_percent, Some(0), "0 of 61 kills is 0%");
    assert_eq!(map13.secrets_percent, None, "an unknown total is not 0%");
    assert!(!recorded.maps.iter().any(|map| map.id == "MAP18"));
}

#[test]
fn the_uv_max_mark_follows_the_documented_rule() {
    // (skill, best kills, max kills, best secrets, max secrets, expected)
    for (skill, best_kills, max_kills, best_secrets, max_secrets, expected) in [
        (4, 100, 100, 5, 5, true),
        (5, 80, 80, 3, 3, true),
        (3, 100, 100, 5, 5, false),
        (2, 100, 100, 5, 5, false),
        (4, 99, 100, 5, 5, false),
        (4, 100, 100, 4, 5, false),
        (4, 100, -1, 5, 5, false),
        (4, 100, 100, 5, -1, false),
        (4, 0, 0, 0, 0, true),
        // Independently accumulated bests may exceed a later denominator.
        (4, 120, 100, 6, 5, true),
    ] {
        let stats = parse(&document(&[detail_row(
            "MAP01",
            skill,
            best_kills,
            max_kills,
            best_secrets,
            max_secrets,
        )]))
        .expect("the generated row is valid");
        let recorded = stats.progress(&[]);
        assert_eq!(
            recorded.maps[0].uv_max, expected,
            "skill {skill}, kills {best_kills}/{max_kills}, secrets {best_secrets}/{max_secrets}"
        );
        assert_eq!(stats.rows[0].uv_max(), expected);
    }
}

#[test]
fn kills_and_secrets_percentages_follow_the_rule() {
    // (best, recorded total, expected floor percentage)
    for (best, max, expected) in [
        (199, 200, 99),
        (200, 200, 100),
        (0, 61, 0),
        (1, 3, 33),
        // Resurrected or spawned monsters can push kills above the total.
        (120, 100, 120),
    ] {
        let stats = parse(&document(&[detail_row("MAP01", 4, best, max, best, max)]))
            .expect("the generated row is valid");
        let map = &stats.progress(&[]).maps[0];
        assert_eq!(map.kills_percent, Some(expected), "kills {best}/{max}");
        assert_eq!(map.secrets_percent, Some(expected), "secrets {best}/{max}");
    }
    // An unknown (-1) or zero total has no percentage at all.
    for max in [-1, 0] {
        let stats = parse(&document(&[detail_row("MAP01", 4, 5, max, 5, max)])).unwrap();
        let map = &stats.progress(&[]).maps[0];
        assert_eq!(map.kills_percent, None, "kills total {max}");
        assert_eq!(map.secrets_percent, None, "secrets total {max}");
    }
    // The two columns are independent: kills known, secrets unknown.
    let stats = parse(&document(&[detail_row("MAP01", 4, 50, 100, 2, -1)])).unwrap();
    let map = &stats.progress(&[]).maps[0];
    assert_eq!(map.kills_percent, Some(50));
    assert_eq!(map.secrets_percent, None);
}

#[test]
fn exclusions_absent_from_the_file_are_ignored() {
    let stats = parse(ABANDON).unwrap();
    let recorded = stats.progress(&["MAP99".to_owned()]);
    assert_eq!(recorded.total, 18);
    assert_eq!(recorded.completed, 12);
}

#[test]
fn zero_and_unknown_values_are_preserved() {
    let stats = parse(&document(&[row("MAP01", 0, 2)])).unwrap();
    let parsed = &stats.rows[0];
    assert_eq!(parsed.best_skill, 0);
    assert_eq!(parsed.best_time, -1);
    assert_eq!(parsed.best_max_time, -1);
    assert_eq!(parsed.max_kills, -1);
    assert_eq!(parsed.total_exits, 2, "an exit survives a zero skill");
    assert!(!parsed.completed());

    let recorded = stats.progress(&[]);
    assert_eq!(recorded.completed, 0, "zero completed is a real value");
    assert_eq!(recorded.total, 1);
}

#[test]
fn lf_and_crlf_parse_identically() {
    let lf = document(&[row("MAP01", 4, 1), row("MAP02", 0, 0)]);
    let crlf: Vec<u8> = String::from_utf8(lf.clone())
        .unwrap()
        .replace('\n', "\r\n")
        .into_bytes();
    assert_eq!(parse(&lf).unwrap(), parse(&crlf).unwrap());
}

#[test]
fn short_and_long_rows_reject_the_whole_file() {
    let short = b"1\n0\nMAP01 1 1 4 10 10 -1 1 5 5 1 0 5 1\n".to_vec();
    assert!(matches!(
        parse(&short),
        Err(ParseError::WrongTokenCount { line: 3, got: 14 })
    ));
    let long = b"1\n0\nMAP01 1 1 4 10 10 -1 1 5 5 1 0 5 1 0 7\n".to_vec();
    assert!(matches!(
        parse(&long),
        Err(ParseError::WrongTokenCount { line: 3, got: 16 })
    ));
    // A failure before a valid prefix still rejects everything: the caller has
    // no `Stats` to read a partial row list from.
    let valid_then_bad = document(&[row("MAP01", 4, 1), "MAP02 1\n".to_owned()]);
    assert!(parse(&valid_then_bad).is_err());
}

#[test]
fn malformed_and_out_of_range_values_are_rejected() {
    assert!(matches!(
        parse(b"1\n0\nMAP01 1 1 4 10 10 -1 1 5 5 1 0 5 1 x\n"),
        Err(ParseError::BadInteger { line: 3, .. })
    ));
    assert!(matches!(
        parse(b"1\n0\nMAP01 1 1 4 10 10 -1 1 5 5 1 0 5 1 99999999999\n"),
        Err(ParseError::BadInteger { line: 3, .. })
    ));
    assert!(matches!(
        parse(b"1\n0\nMAP01 1 1 6 10 10 -1 1 5 5 1 0 5 1 0\n"),
        Err(ParseError::OutOfRange {
            field: "best_skill",
            ..
        })
    ));
    assert!(matches!(
        parse(b"1\n0\nMAP01 1 1 4 10 10 -1 -2 5 5 1 0 5 1 0\n"),
        Err(ParseError::OutOfRange {
            field: "total_exits",
            ..
        })
    ));
    // Times may be unknown (-1) but never another negative value.
    assert!(matches!(
        parse(b"1\n0\nMAP01 1 1 4 -5 10 -1 1 5 5 1 0 5 1 0\n"),
        Err(ParseError::OutOfRange {
            field: "best_time",
            ..
        })
    ));
}

#[test]
fn version_and_header_shape_are_strict() {
    assert!(matches!(
        parse(b"2\n0\nMAP01 1 1 4 10 10 -1 1 5 5 1 0 5 1 0\n"),
        Err(ParseError::UnsupportedVersion { .. })
    ));
    assert!(matches!(
        parse(b"0\n0\nMAP01 1 1 4 10 10 -1 1 5 5 1 0 5 1 0\n"),
        Err(ParseError::UnsupportedVersion { .. })
    ));
    assert!(matches!(parse(b"1\n"), Err(ParseError::MissingHeaders)));
    assert!(matches!(parse(b""), Err(ParseError::MissingHeaders)));
    assert!(matches!(
        parse(b"1\n-1\n"),
        Err(ParseError::OutOfRange {
            field: "total_kills",
            ..
        })
    ));
    // The final row must be terminated, and no blank line may hide a row.
    assert!(matches!(
        parse(b"1\n0\nMAP01 1 1 4 10 10 -1 1 5 5 1 0 5 1 0"),
        Err(ParseError::MissingFinalNewline)
    ));
    assert!(parse(b"1\n0\n\n").is_err());
}

#[test]
fn duplicate_and_malformed_map_ids_are_rejected() {
    let duplicate = document(&[row("MAP01", 4, 1), row("MAP01", 0, 0)]);
    assert!(matches!(
        parse(&duplicate),
        Err(ParseError::DuplicateMapId { line: 4, .. })
    ));
    let long = document(&[row("MAP012345", 4, 1)]);
    assert!(matches!(
        parse(&long),
        Err(ParseError::BadMapId { line: 3 })
    ));
    let non_ascii = b"1\n0\nMAP\xc3\x8901 1 1 4 10 10 -1 1 5 5 1 0 5 1 0\n".to_vec();
    assert!(matches!(
        parse(&non_ascii),
        Err(ParseError::BadMapId { line: 3 })
    ));
}

#[test]
fn row_and_byte_limits_reject_the_file() {
    let rows: Vec<String> = (0..=MAX_ROWS).map(|_| row("MAP01", 0, 0)).collect();
    // Duplicate IDs would fire first for identical rows; use distinct IDs that
    // still fit eight bytes.
    let rows: Vec<String> = rows
        .iter()
        .enumerate()
        .map(|(index, _)| row(&format!("M{index:05}"), 0, 0))
        .collect();
    assert!(matches!(
        parse(&document(&rows)),
        Err(ParseError::TooManyRows { limit: MAX_ROWS })
    ));

    let mut huge = document(&[row("MAP01", 4, 1)]);
    huge.resize(MAX_BYTES as usize + 1, b'\n');
    assert!(matches!(parse(&huge), Err(ParseError::Oversize { .. })));

    let long_line = vec![b'1'; MAX_LINE_BYTES + 1];
    assert!(matches!(
        parse(&[b"1\n0\n", long_line.as_slice(), b"\n"].concat()),
        Err(ParseError::LineTooLong { line: 3 })
    ));
}

#[test]
fn the_path_follows_the_engine_organizer() {
    let data = PathBuf::from("/data/profiles/one/data");
    let iwad = PathBuf::from("/data/iwads/doom2-1.9/doom2.wad");

    // Catalog mapset: lowercase IWAD stem, then the PWAD stem.
    let files = paths(&["/library/Abandon.wad"]);
    assert_eq!(
        stats_path(&layout(&data, &iwad, &files, &[])).unwrap(),
        data.join("dsda_doom_data/doom2/abandon/stats.txt")
    );

    // Catalog plus an extra WAD: profile extras append in load order.
    let with_extra = paths(&["/library/Abandon.wad", "/extras/Weapons.wad"]);
    assert_eq!(
        stats_path(&layout(&data, &iwad, &with_extra, &[])).unwrap(),
        data.join("dsda_doom_data/doom2/abandon/weapons/stats.txt")
    );

    // Base game: no `-file` at all.
    assert_eq!(
        stats_path(&layout(&data, &iwad, &[], &[])).unwrap(),
        data.join("dsda_doom_data/doom2/stats.txt")
    );

    // Custom setup: the IWAD stem, then every ordered file.
    let custom = paths(&["/wads/A.wad", "/wads/B.wad"]);
    assert_eq!(
        stats_path(&layout(&data, Path::new("/iwads/doom.wad"), &custom, &[])).unwrap(),
        data.join("dsda_doom_data/doom/a/b/stats.txt")
    );

    // Mixed case is lowercased.
    let mixed = paths(&["/wads/Example.WAD"]);
    assert_eq!(
        stats_path(&layout(&data, Path::new("/iwads/DOOM2.WAD"), &mixed, &[])).unwrap(),
        data.join("dsda_doom_data/doom2/example/stats.txt")
    );
}

#[test]
fn only_the_first_eight_wads_and_no_patch_contribute() {
    let data = PathBuf::from("/data/profiles/one/data");
    let iwad = PathBuf::from("/iwads/doom2.wad");
    let files = paths(&[
        "/w/1.wad", "/w/2.wad", "/w/3.wad", "/w/4.wad", "/w/5.wad", "/w/6.wad", "/w/7.wad",
        "/w/8.wad", "/w/9.wad",
    ]);
    assert_eq!(
        stats_path(&layout(&data, &iwad, &files, &[])).unwrap(),
        data.join("dsda_doom_data/doom2/1/2/3/4/5/6/7/8/stats.txt")
    );
    // A patch never participates, so adding it leaves the path identical.
    let patches = paths(&["/w/fix.deh", "/w/fix.bex"]);
    assert_eq!(
        stats_path(&layout(&data, &iwad, &files, &patches)).unwrap(),
        stats_path(&layout(&data, &iwad, &files, &[])).unwrap()
    );
}

#[test]
fn an_unverified_file_yields_no_path() {
    let data = PathBuf::from("/data/profiles/one/data");
    let iwad = PathBuf::from("/iwads/doom2.wad");
    let files = paths(&["/w/pack.zip"]);
    assert_eq!(
        stats_path(&layout(&data, &iwad, &files, &[])),
        Err(LayoutError::UnverifiedFile(PathBuf::from("/w/pack.zip")))
    );
    assert_eq!(
        stats_path(&layout(&data, Path::new("/"), &[], &[])),
        Err(LayoutError::IwadStem)
    );
}

#[test]
fn a_missing_file_is_no_stats_yet_not_zero_completed() {
    let temp = data_dir();
    let profile_data = temp.path().join("profiles/one/data");
    std::fs::create_dir_all(&profile_data).unwrap();
    let iwad = PathBuf::from("/iwads/doom2.wad");
    let files = paths(&["/w/map.wad"]);
    let request = Request {
        profile_data_dir: &profile_data,
        iwad: &iwad,
        files: &files,
        patches: &[],
        engine_version: "0.30.1",
        exclude_maps: &[],
    };
    assert_eq!(request.read(), Progress::NoStatsYet);
}

#[test]
fn an_unreadable_or_invalid_file_is_unavailable() {
    let temp = data_dir();
    let profile_data = temp.path().join("profiles/one/data");
    let organized = profile_data.join("dsda_doom_data/doom2");
    std::fs::create_dir_all(&organized).unwrap();
    let stats = organized.join("stats.txt");
    std::fs::write(&stats, b"1\n0\nnot a row\n").unwrap();
    let iwad = PathBuf::from("/iwads/doom2.wad");
    let files: Vec<PathBuf> = Vec::new();
    let request = Request {
        profile_data_dir: &profile_data,
        iwad: &iwad,
        files: &files,
        patches: &[],
        engine_version: "0.30.1",
        exclude_maps: &[],
    };
    assert!(matches!(
        request.read(),
        Progress::Unavailable(Unavailable::Invalid(ParseError::WrongTokenCount { .. }))
    ));

    // A newer, unaudited engine never imports, whatever the file says.
    std::fs::write(&stats, ABANDON).unwrap();
    let mut newer = request;
    newer.engine_version = "0.31.0";
    assert!(matches!(
        newer.read(),
        Progress::Unavailable(Unavailable::UnsupportedEngine { .. })
    ));

    // The same file with the audited engine is read and reduced by the rule.
    let mut recorded = request;
    let excluded = ["MAP18".to_owned()];
    recorded.exclude_maps = &excluded;
    assert_eq!(
        recorded.read(),
        Progress::Recorded(Recorded {
            completed: 12,
            total: 17,
            maps: parse(ABANDON).unwrap().progress(&["MAP18".to_owned()]).maps,
        })
    );
}

#[test]
fn an_oversize_file_is_unavailable_before_reading() {
    let temp = data_dir();
    let profile_data = temp.path().join("profiles/one/data");
    let organized = profile_data.join("dsda_doom_data/doom2");
    std::fs::create_dir_all(&organized).unwrap();
    std::fs::write(
        organized.join("stats.txt"),
        vec![b'0'; MAX_BYTES as usize + 1],
    )
    .unwrap();
    assert_eq!(
        read(&profile_data, &organized.join("stats.txt")),
        Err(ReadError::Oversize { limit: MAX_BYTES })
    );
}

#[test]
fn a_directory_instead_of_a_file_is_unavailable() {
    let temp = data_dir();
    let profile_data = temp.path().join("profiles/one/data");
    let organized = profile_data.join("dsda_doom_data/doom2");
    std::fs::create_dir_all(organized.join("stats.txt")).unwrap();
    assert_eq!(
        read(&profile_data, &organized.join("stats.txt")),
        Err(ReadError::NotAFile)
    );
}

#[cfg(unix)]
#[test]
fn a_symlink_escaping_the_profile_data_directory_is_refused() {
    let temp = data_dir();
    let profile_data = temp.path().join("profiles/one/data");
    let outside = temp.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("stats.txt"), ABANDON).unwrap();
    let organized = profile_data.join("dsda_doom_data");
    std::fs::create_dir_all(&organized).unwrap();
    std::os::unix::fs::symlink(&outside, organized.join("doom2")).unwrap();
    assert_eq!(
        read(&profile_data, &organized.join("doom2/stats.txt")),
        Err(ReadError::Escape)
    );
}

#[cfg(unix)]
#[test]
fn a_symlink_inside_the_data_directory_is_still_read() {
    let temp = data_dir();
    let profile_data = temp.path().join("profiles/one/data");
    let real = profile_data.join("real");
    std::fs::create_dir_all(&real).unwrap();
    std::fs::write(real.join("stats.txt"), ABANDON).unwrap();
    let organized = profile_data.join("dsda_doom_data");
    std::fs::create_dir_all(&organized).unwrap();
    std::os::unix::fs::symlink(&real, organized.join("doom2")).unwrap();
    let bytes = read(&profile_data, &organized.join("doom2/stats.txt"))
        .unwrap()
        .expect("the contained target is readable");
    assert_eq!(bytes, ABANDON);
}

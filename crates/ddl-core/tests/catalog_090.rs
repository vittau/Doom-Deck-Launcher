//! Spec 090: release chronology for PWAD browsing and the focused Community
//! Projects tab.
//!
//! Everything here is offline. Recipe fixtures are synthetic TOML; the embedded
//! catalog is read through its public API only.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ddl_core::catalog::{
    BUNDLE_SCHEMA, Bundle, COMMUNITY_PROJECTS, Catalog, DOOMER_BOARDS, ReleaseDate, accept_bundle,
    compare_browsing_order, load_cache, parse_bundle, shows_in_category, store_cache, validate,
};

fn catalog_file(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../catalog")
        .join(name)
}

fn hosts_text() -> String {
    std::fs::read_to_string(catalog_file("hosts.toml")).expect("catalog hosts")
}

fn tags_text() -> String {
    std::fs::read_to_string(catalog_file("tags.toml")).expect("catalog tags")
}

/// A minimal valid recipe. `date` is an optional `original_release` line, so
/// callers can exercise absent, partial, full and malformed values.
fn recipe(id: &str, title: &str, release: &str, date: Option<&str>) -> String {
    let sha = "0".repeat(64);
    format!(
        r#"
schema_version = 1
id = "{id}"
title = "{title}"
authors = ["Someone"]
release = "{release}"
{date}page = "https://www.doomworld.com/idgames/levels/doom2/a"
description = "A mapset."
permission = "Free to redistribute."
[base_game]
game = "doom2"
[compatibility]
family = "boom"
complevel = "boom"
[compatibility.engine]
range = "=0.30.1"
status = "inspected"
[verification]
status = "inspected"
tester = "test"
date = "2026-10-22"
engine_version = "0.30.1"
inspected = ["fixture"]
notes = "fixture"
limitations = []
[[packages]]
id = "free"
page = "https://www.doomworld.com/idgames/levels/doom2/a"
urls = ["https://www.gamers.org/pub/idgames/a.zip"]
size = 1
sha256 = "{sha}"
kind = "zip"
[[packages.members]]
id = "maps"
path = "a.wad"
size = 1
sha256 = "{sha}"
[[components]]
id = "main"
label = "Main"
description = "Main."
optional = false
default = true
position = 1
[[components.members]]
package = "free"
member = "maps"
role = "maps"
"#,
        date = date.map(|line| format!("{line}\n")).unwrap_or_default(),
    )
}

fn catalog_of(files: &[(&str, &str)]) -> Catalog {
    Catalog::from_sources(&hosts_text(), &tags_text(), files)
}

fn date(year: u16, month: Option<u8>, day: Option<u8>) -> ReleaseDate {
    ReleaseDate { year, month, day }
}

#[test]
fn every_embedded_recipe_has_absent_or_valid_release_metadata() {
    let catalog = Catalog::embedded();
    assert!(catalog.skipped.is_empty(), "{:?}", catalog.skipped);
    assert!(!catalog.recipes.is_empty());
    for held in &catalog.recipes {
        let violations = validate(&held.recipe, Path::new(&held.file), &catalog.hosts);
        assert!(
            violations.is_empty(),
            "{}: {:?}",
            held.recipe.id,
            violations
        );
    }
}

#[test]
fn the_embedded_catalog_orders_by_date_and_hides_db_entries_from_community_projects() {
    let catalog = Catalog::embedded();
    let mut recipes: Vec<_> = catalog.recipes.iter().map(|held| &held.recipe).collect();
    recipes.sort_by(|a, b| compare_browsing_order(a, b));

    let mut seen_unknown = false;
    let mut previous: Option<(u16, u8, u8)> = None;
    for recipe in &recipes {
        match recipe.original_release {
            Some(date) => {
                assert!(
                    !seen_unknown,
                    "{}: a dated recipe sorts after an unknown date",
                    recipe.id
                );
                let key = date.ordering_key();
                if let Some(previous) = previous {
                    assert!(
                        previous <= key,
                        "{}: {:?} sorts before {:?}",
                        recipe.id,
                        key,
                        previous
                    );
                }
                previous = Some(key);
            }
            None => seen_unknown = true,
        }
    }

    let mut dating = 0;
    let mut dbps = 0;
    for held in &catalog.recipes {
        if held.recipe.original_release.is_some() {
            dating += 1;
        }
        if held.recipe.tags.iter().any(|tag| tag == DOOMER_BOARDS) {
            dbps += 1;
            assert!(
                !shows_in_category(&held.recipe, COMMUNITY_PROJECTS),
                "{}: a Doomer Boards recipe is in the Community Projects tab",
                held.recipe.id
            );
            assert!(
                shows_in_category(&held.recipe, DOOMER_BOARDS),
                "{}: the dedicated Doomer Boards tab keeps it",
                held.recipe.id
            );
        }
    }
    assert!(dbps > 0, "the catalog still ships Doomer Boards recipes");
    assert!(
        dating >= 90,
        "only {dating} embedded recipes carry a release date"
    );
}

#[test]
fn community_projects_keeps_ordinary_community_recipes() {
    let catalog = Catalog::embedded();
    let shown: Vec<&str> = catalog
        .recipes
        .iter()
        .map(|held| &held.recipe)
        .filter(|recipe| shows_in_category(recipe, COMMUNITY_PROJECTS))
        .map(|recipe| recipe.id.as_str())
        .collect();
    assert!(
        !shown.is_empty(),
        "ordinary community projects stay visible"
    );
    assert!(
        shown
            .iter()
            .any(|id| *id == "eviternity" || *id == "back-to-saturn-x-e1"),
        "a known community project is still listed: {shown:?}"
    );
}

#[test]
fn version_labels_do_not_decide_the_order() {
    // The lexicographic labels sort 10.0 before 9.0, while the dates are the
    // other way round: chronology must win (R3 forbids parsing the label).
    let catalog = catalog_of(&[
        (
            "newer.toml",
            &recipe(
                "newer",
                "Newer",
                "10.0",
                Some("original_release = { year = 2001, month = 3 }"),
            ),
        ),
        (
            "older.toml",
            &recipe(
                "older",
                "Older",
                "9.0",
                Some("original_release = { year = 1998, month = 12, day = 4 }"),
            ),
        ),
    ]);
    assert!(catalog.skipped.is_empty(), "{:?}", catalog.skipped);
    let older = catalog.recipe("older").unwrap();
    let newer = catalog.recipe("newer").unwrap();
    assert!(
        compare_browsing_order(older, newer) == std::cmp::Ordering::Less,
        "1998 must sort before 2001 regardless of the version labels"
    );
}

#[test]
fn equal_and_unknown_dates_tie_deterministically() {
    let same = Some("original_release = { year = 2005, month = 9, day = 30 }");
    let catalog = catalog_of(&[
        ("bravo.toml", &recipe("bravo", "Alpha", "1.0", same)),
        ("alpha.toml", &recipe("alpha", "Alpha", "2.0", same)),
        ("unknown.toml", &recipe("unknown", "Aardvark", "0.1", None)),
    ]);
    assert!(catalog.skipped.is_empty(), "{:?}", catalog.skipped);
    let bravo = catalog.recipe("bravo").unwrap();
    let alpha = catalog.recipe("alpha").unwrap();
    let unknown = catalog.recipe("unknown").unwrap();
    // Same title and date: the id breaks the tie, not catalog order.
    assert!(compare_browsing_order(alpha, bravo) == std::cmp::Ordering::Less);
    // An unknown date sorts after any known date, even with an earlier title.
    assert!(compare_browsing_order(bravo, unknown) == std::cmp::Ordering::Less);
    let mut sorted = vec![unknown, bravo, alpha];
    sorted.sort_by(|a, b| compare_browsing_order(a, b));
    assert_eq!(
        sorted.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
        ["alpha", "bravo", "unknown"]
    );
}

#[test]
fn partial_precision_never_claims_january_or_the_first_of_the_month() {
    let catalog = catalog_of(&[
        (
            "year.toml",
            &recipe(
                "year",
                "Year only",
                "1.0",
                Some("original_release = { year = 1997 }"),
            ),
        ),
        (
            "december.toml",
            &recipe(
                "december",
                "December",
                "1.0",
                Some("original_release = { year = 1997, month = 12, day = 31 }"),
            ),
        ),
        (
            "may.toml",
            &recipe(
                "may",
                "May",
                "1.0",
                Some("original_release = { year = 1997, month = 5 }"),
            ),
        ),
        (
            "may-day.toml",
            &recipe(
                "may-day",
                "May day",
                "1.0",
                Some("original_release = { year = 1997, month = 5, day = 20 }"),
            ),
        ),
    ]);
    assert!(catalog.skipped.is_empty(), "{:?}", catalog.skipped);
    let by_id = |id: &str| catalog.recipe(id).unwrap();
    let mut sorted = vec![
        by_id("year"),
        by_id("december"),
        by_id("may"),
        by_id("may-day"),
    ];
    sorted.sort_by(|a, b| compare_browsing_order(a, b));
    assert_eq!(
        sorted.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
        ["may-day", "may", "december", "year"],
        "a month-only date follows its known days; a year-only date follows the year"
    );
    assert_eq!(date(1997, Some(5), None).iso(), "1997-05");
    assert_eq!(date(1997, None, None).iso(), "1997");
}

#[test]
fn legacy_recipes_without_the_field_still_load_and_serialize() {
    let catalog = catalog_of(&[("legacy.toml", &recipe("legacy", "Legacy", "1.0", None))]);
    assert!(catalog.skipped.is_empty(), "{:?}", catalog.skipped);
    let legacy = catalog.recipe("legacy").unwrap();
    assert_eq!(legacy.original_release, None);
    // The optional field is absent from JSON too, so an old cache/manifest shape
    // round-trips unchanged.
    let json = serde_json::to_value(legacy).unwrap();
    assert!(json.get("original_release").is_none());
    let back: ddl_core::catalog::Recipe = serde_json::from_value(json).unwrap();
    assert_eq!(back.original_release, None);
}

#[test]
fn present_release_dates_are_validated() {
    let hosts = Catalog::embedded().hosts;
    let check = |file: &str, text: &str| -> Vec<String> {
        let recipe: ddl_core::catalog::Recipe = toml::from_str(text).expect("fixture parses");
        validate(&recipe, Path::new(file), &hosts)
            .into_iter()
            .map(|violation| violation.key)
            .collect()
    };

    // A real date, a year and a year-month are all accepted.
    for date in [
        "original_release = { year = 1993, month = 12, day = 10 }",
        "original_release = { year = 1993 }",
        "original_release = { year = 1993, month = 12 }",
    ] {
        let keys = check("ok.toml", &recipe("ok", "OK", "1.0", Some(date)));
        assert!(keys.is_empty(), "{date}: {keys:?}");
    }

    // Impossible months, days and a day without a month are rejected.
    for (file, date, key) in [
        (
            "month.toml",
            "original_release = { year = 1997, month = 13 }",
            "original_release.month",
        ),
        (
            "day.toml",
            "original_release = { year = 1997, month = 2, day = 30 }",
            "original_release.day",
        ),
        (
            "orphan.toml",
            "original_release = { year = 1997, day = 4 }",
            "original_release.day",
        ),
        (
            "year.toml",
            "original_release = { year = 1800 }",
            "original_release.year",
        ),
    ] {
        let id = file.trim_end_matches(".toml");
        let keys = check(file, &recipe(id, "Bad", "1.0", Some(date)));
        assert!(keys.contains(&key.to_string()), "{date}: {keys:?}");
    }

    // The loader applies the same rules, so a bad field is a skipped recipe.
    let skipped = catalog_of(&[(
        "bad.toml",
        &recipe(
            "bad",
            "Bad",
            "1.0",
            Some("original_release = { year = 1997, month = 13 }"),
        ),
    )]);
    assert_eq!(skipped.recipes.len(), 0);
    assert_eq!(skipped.skipped.len(), 1);
    assert!(
        skipped.skipped[0].reason.contains("original release"),
        "{}",
        skipped.skipped[0].reason
    );
}

#[test]
fn legacy_bundles_and_caches_missing_the_field_load_but_old_clients_are_gated() {
    let mut files = BTreeMap::new();
    files.insert("hosts.toml".to_owned(), hosts_text());
    files.insert("tags.toml".to_owned(), tags_text());
    files.insert(
        "legacy.toml".to_owned(),
        recipe("legacy", "Legacy", "1.0", None),
    );
    let bundle = Bundle {
        bundle_schema: BUNDLE_SCHEMA,
        catalog_serial: 9_000,
        source_commit: "abc1234".to_owned(),
        min_ddl_version: "0.0.12".to_owned(),
        files,
    };

    // A bundle written before the field existed parses and keeps its recipe.
    let bytes = serde_json::to_vec(&bundle).unwrap();
    let parsed = parse_bundle(&bytes).expect("legacy bundle parses");
    let accepted = accept_bundle(&parsed, &hosts_text(), 100, "0.0.12").expect("accepted");
    assert_eq!(accepted.catalog.recipes.len(), 1);
    assert!(accepted.dropped().is_empty(), "{:?}", accepted.dropped());
    assert_eq!(
        accepted.catalog.recipe("legacy").unwrap().original_release,
        None
    );

    // The capability gate refuses the same document on a pre-0.0.12 client
    // instead of letting it drop every dated recipe.
    assert!(accept_bundle(&parsed, &hosts_text(), 100, "0.0.11").is_err());

    // A cache written from that bundle loads through the same path.
    let dir = tempfile::tempdir().unwrap();
    store_cache(dir.path(), &accepted.bundle, 42).unwrap();
    let cached = load_cache(dir.path()).expect("cache loads");
    assert_eq!(cached.fetched_at, 42);
    assert_eq!(cached.catalog.recipes.len(), 1);
    assert_eq!(
        cached.catalog.recipe("legacy").unwrap().original_release,
        None
    );
}

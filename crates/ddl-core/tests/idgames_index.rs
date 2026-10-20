//! Spec 050 acceptance (A1/A2): fixtures, exclusion rules, sidecar parsing,
//! schema round-trip, determinism, the size bound and incremental builds.
//!
//! Every request goes through [`FixtureSource`], which reads
//! `tests/fixtures/idgames/txt/<stem>.txt` from disk. **No test touches the
//! network** (`AGENTS.md`); the production HTTP client is exercised nowhere
//! here by construction, and the crawl's real requests are a CI concern.

use std::collections::BTreeMap;
use std::path::PathBuf;

use ddl_core::idgames::build::{build_index, BuildSettings};
use ddl_core::idgames::cacowards::parse_cacowards;
use ddl_core::idgames::catalog_map::catalog_paths;
use ddl_core::idgames::fetch::{mirror_for, NoPacer, TxtResponse, TxtSource};
use ddl_core::idgames::index::{Entry, Index, IndexGame, MAX_INDEX_BYTES, Port};
use ddl_core::idgames::inventory::{Exclusion, parse_inventory, parse_inventory_gzip};
use ddl_core::idgames::state::CrawlState;
use ddl_core::idgames::Date;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/idgames");

/// The day the fixtures were "generated": recent `ls` dates belong to 2026.
fn today() -> Date {
    Date::new(2026, 10, 20).unwrap()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(FIXTURES).join(name)
}

fn inventory_text() -> String {
    std::fs::read_to_string(fixture("ls-laR.txt")).unwrap()
}

/// Answers from the fixture directory by URL file name, and records every call.
#[derive(Default)]
struct FixtureSource {
    calls: Vec<(String, Option<String>, Option<String>)>,
}

impl TxtSource for FixtureSource {
    fn get(
        &mut self,
        url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> TxtResponse {
        self.calls.push((
            url.to_owned(),
            etag.map(ToOwned::to_owned),
            last_modified.map(ToOwned::to_owned),
        ));
        let name = url.rsplit('/').next().unwrap_or_default();
        match std::fs::read(fixture("txt").join(name)) {
            Ok(bytes) => TxtResponse::Fetched {
                bytes,
                etag: Some(format!("\"{name}\"")),
                last_modified: None,
            },
            Err(_) => TxtResponse::Missing,
        }
    }
}

fn settings() -> BuildSettings {
    let markdown = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/research/cacowards.md"
    ))
    .unwrap();
    let catalog = catalog_paths(&PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog"
    )));
    let mut settings = BuildSettings::new(
        *mirror_for("youfailit.net").unwrap(),
        "https://youfailit.net/pub/idgames/ls-laR.gz",
        "502a7ccd3195452fb4955f077a078d0167bf80b721535be8c59e1daa269c2603",
        1_792_386_428,
    );
    settings.budget = 50;
    settings.cacowards = parse_cacowards(&markdown);
    settings.catalog = catalog;
    settings
}

fn entry<'a>(index: &'a Index, dir: &str, stem: &str) -> &'a Entry {
    index.entries[dir]
        .iter()
        .find(|entry| entry.stem == stem)
        .unwrap_or_else(|| panic!("{dir}/{stem} is in the index"))
}

#[test]
fn the_gzip_inventory_matches_its_plain_text_source() {
    let bytes = std::fs::read(fixture("ls-laR.gz")).unwrap();
    let from_gzip = parse_inventory_gzip(&bytes, today()).unwrap();
    let plain = parse_inventory(&inventory_text(), today());
    assert_eq!(from_gzip.entries, plain.entries);
    assert_eq!(from_gzip.report.excluded, plain.report.excluded);
}

#[test]
fn exclusion_rules_keep_only_the_two_doom_level_trees() {
    let inventory = parse_inventory(&inventory_text(), today());
    let paths: Vec<String> = inventory
        .entries
        .iter()
        .map(ddl_core::idgames::inventory::InventoryEntry::zip_path)
        .collect();
    assert_eq!(paths.len(), 8, "{paths:#?}");
    assert!(paths.iter().all(|path| {
        (path.starts_with("levels/doom/") || path.starts_with("levels/doom2/"))
            && !path.contains("/deathmatch/")
    }));
    assert_eq!(inventory.report.excluded_count(Exclusion::Deathmatch), 1);
    assert_eq!(inventory.report.excluded_count(Exclusion::TransientTree), 3);
    assert_eq!(inventory.report.excluded_count(Exclusion::OtherGame), 6);
    assert_eq!(inventory.report.excluded_count(Exclusion::NotAZip), 2);
}

#[test]
fn fullsort_is_an_equivalent_alternative_inventory() {
    let text = std::fs::read_to_string(fixture("fullsort.txt")).unwrap();
    let fullsort = parse_inventory(&text, today());
    let ls = parse_inventory(&inventory_text(), today());
    let mut left: Vec<(String, u64, String)> = fullsort
        .entries
        .iter()
        .map(|entry| (entry.zip_path(), entry.size, entry.date.to_iso()))
        .collect();
    let mut right: Vec<(String, u64, String)> = ls
        .entries
        .iter()
        .map(|entry| (entry.zip_path(), entry.size, entry.date.to_iso()))
        .collect();
    left.sort();
    right.sort();
    assert_eq!(left, right);
}

#[test]
fn the_built_index_has_the_fields_the_spec_requires() {
    let mut state = CrawlState::new(1_792_386_428);
    let mut source = FixtureSource::default();
    let outcome = build_index(
        &inventory_text(),
        &settings(),
        None,
        &mut state,
        &mut source,
        &mut NoPacer,
    )
    .unwrap();
    assert_eq!(outcome.report.requests, 8);
    assert_eq!(outcome.report.fetched, 6);
    assert_eq!(outcome.report.missing_txt, 2, "nocase.zip and old.zip have no sidecar");
    assert_eq!(outcome.report.bad_txt, 1, "freeform.txt has no labelled field");
    assert_eq!(outcome.report.stopped, None);

    let index = &outcome.index;
    assert_eq!(index.schema, ddl_core::idgames::INDEX_SCHEMA);
    assert_eq!(index.source.mirror, "youfailit.net");
    assert!(index.source.index.ends_with("/ls-laR.gz"));
    assert!(semver::Version::parse(&index.min_ddl_version).is_ok());

    let augzen = entry(index, "levels/doom2/Ports/megawads", "dbp37_augzen");
    assert_eq!(augzen.title, "AUGER;ZENITH");
    assert_eq!(augzen.author, "Doomer Boards Community");
    assert_eq!(augzen.game, IndexGame::Doom2);
    assert_eq!(augzen.port, Port::Mbf21);
    assert_eq!(augzen.maps, Some(11));
    assert!(augzen.txt);
    assert_eq!(augzen.date.to_iso(), "2021-07-26");
    assert_eq!(augzen.size, 12_771_737);
    assert_eq!(augzen.cacoward, Some(2021));
    assert_eq!(augzen.in_catalog.as_deref(), Some("auger-zenith"));

    let tistartan = entry(index, "levels/doom2/Ports/megawads", "tistartan");
    assert_eq!(tistartan.date.to_iso(), "2026-10-18", "recent date year inferred");
    assert_eq!(tistartan.maps, Some(32));

    let latin1 = entry(index, "levels/doom2/Ports/megawads", "latin1");
    assert_eq!(latin1.title, "München", "Windows-1252 fallback");
    assert_eq!(latin1.author, "Jörg");
    assert_eq!(latin1.port, Port::Boom);

    let zdoom = entry(index, "levels/doom2/Ports/a-c", "zdoom_only");
    assert_eq!(
        zdoom.port,
        Port::Zdoom,
        "the ZDoom family is recorded, not flattened into `other`"
    );
    assert_eq!(zdoom.game, IndexGame::Doom2);

    let sstruggle = entry(index, "levels/doom/Ports/megawads", "sstruggle");
    assert_eq!(sstruggle.game, IndexGame::Doom);
    assert_eq!(sstruggle.port, Port::Vanilla);
    assert_eq!(sstruggle.maps, None, "E1M1-E4M9 does not state a map count");
    assert_eq!(sstruggle.cacoward, Some(2022));

    // A missing sidecar keeps the entry with a filename fallback.
    let nocase = entry(index, "levels/doom2/Ports/megawads", "nocase");
    assert!(!nocase.txt);
    assert_eq!(nocase.title, "nocase");
    assert_eq!(nocase.author, "");
    assert_eq!(nocase.port, Port::Unknown);

    // A free-form sidecar is present but unusable: still the fallback.
    let freeform = entry(index, "levels/doom2/Ports/megawads", "freeform");
    assert_eq!(freeform.title, "freeform");
    assert!(freeform.txt);
}

#[test]
fn the_document_round_trips_is_deterministic_and_bounded() {
    let mut state = CrawlState::new(1_792_386_428);
    let mut source = FixtureSource::default();
    let first = build_index(
        &inventory_text(),
        &settings(),
        None,
        &mut state,
        &mut source,
        &mut NoPacer,
    )
    .unwrap();
    let mut source = FixtureSource::default();
    let second = build_index(
        &inventory_text(),
        &settings(),
        None,
        &mut state,
        &mut source,
        &mut NoPacer,
    )
    .unwrap();
    let json = first.index.to_json().unwrap();
    assert_eq!(json, second.index.to_json().unwrap());
    assert!(json.len() as u64 <= MAX_INDEX_BYTES);
    let parsed = Index::parse(&json).unwrap();
    assert_eq!(parsed, first.index);
    assert_eq!(parsed.to_json().unwrap(), json);
    // No descriptions, credits, ratings or txt bodies.
    let text = String::from_utf8(json).unwrap();
    for forbidden in [
        "description",
        "Doomer Boards Community's credits",
        "This is just prose",
        "Tested With",
    ] {
        assert!(!text.contains(forbidden), "{forbidden} must not be published");
    }
}

#[test]
fn incremental_builds_fetch_only_added_or_changed_entries() {
    let mut state = CrawlState::new(1_792_386_428);
    let mut first_source = FixtureSource::default();
    let first = build_index(
        &inventory_text(),
        &settings(),
        None,
        &mut state,
        &mut first_source,
        &mut NoPacer,
    )
    .unwrap();
    assert_eq!(first_source.calls.len(), 8);

    let mut second_source = FixtureSource::default();
    let second = build_index(
        &inventory_text(),
        &settings(),
        Some(&first.index),
        &mut state,
        &mut second_source,
        &mut NoPacer,
    )
    .unwrap();
    assert_eq!(second_source.calls.len(), 0);
    assert_eq!(second.report.reused, 6, "entries with a usable sidecar are reused");

    // One upload changed size: exactly one request, for that entry.
    let changed = inventory_text().replace(
        "  10370840 Oct 18 11:48 tistartan.zip",
        "  10370841 Oct 18 11:48 tistartan.zip",
    );
    let mut third_source = FixtureSource::default();
    let third = build_index(
        &changed,
        &settings(),
        Some(&second.index),
        &mut state,
        &mut third_source,
        &mut NoPacer,
    )
    .unwrap();
    assert_eq!(third_source.calls.len(), 1);
    assert!(third_source.calls[0].0.ends_with("/tistartan.txt"));
    assert_eq!(
        entry(&third.index, "levels/doom2/Ports/megawads", "tistartan").size,
        10_370_841
    );
    assert_eq!(
        entry(&third.index, "levels/doom2/Ports/megawads", "tistartan").title,
        "Tistartan"
    );
}

#[test]
fn the_crawl_order_is_by_relevance() {
    let mut state = CrawlState::new(1_792_386_428);
    let mut source = FixtureSource::default();
    let settings = settings();
    build_index(
        &inventory_text(),
        &settings,
        None,
        &mut state,
        &mut source,
        &mut NoPacer,
    )
    .unwrap();
    let urls: Vec<&str> = source.calls.iter().map(|(url, _, _)| url.as_str()).collect();
    // dbp37_augzen (2021) and sstruggle (2022) are Cacoward winners, so they
    // are the first two requests; the newest of the two wins the tie.
    assert!(
        urls[0].ends_with("/levels/doom2/Ports/megawads/dbp37_augzen.txt"),
        "{urls:#?}"
    );
    assert!(
        urls[1].ends_with("/levels/doom/Ports/megawads/sstruggle.txt"),
        "{urls:#?}"
    );
    assert!(urls[2].contains("/Ports/megawads/"), "{urls:#?}");
    assert!(urls[3].contains("/Ports/megawads/"), "{urls:#?}");
    assert!(
        urls.iter().position(|url| url.contains("latin1")).unwrap()
            < urls.iter().position(|url| url.contains("zdoom_only")).unwrap(),
        "{urls:#?}"
    );
}

#[test]
fn cacoward_years_are_read_from_the_real_research_document() {
    let markdown = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/research/cacowards.md"
    ))
    .unwrap();
    let winners = parse_cacowards(&markdown);
    let years: BTreeMap<&str, u16> = BTreeMap::from([
        ("levels/doom2/ports/megawads/dbp37_augzen.zip", 2021),
        ("levels/doom2/ports/megawads/eviternityii.zip", 2024),
        ("levels/doom/ports/megawads/sstruggle.zip", 2022),
    ]);
    for (path, year) in years {
        assert_eq!(winners.get(path), Some(&year), "{path}");
    }
}

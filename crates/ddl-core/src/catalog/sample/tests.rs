//! Spec 044 A4: the e2e sample over synthetic recipes and the real `catalog/`.

use std::collections::BTreeSet;
use std::path::Path;

use super::*;
use crate::catalog::recipe_fixture::recipe;
use crate::catalog::{Catalog, load_catalog_dir};

const MIB: u64 = 1024 * 1024;
const CAP: u64 = 64 * MIB;

fn urls(host: &str) -> Vec<String> {
    vec![
        format!("https://{host}/a.zip"),
        "https://mirror.example/a.zip".into(),
    ]
}

fn simple(id: &str, host: &str, mib: u64) -> Recipe {
    recipe(id, &[("free", urls(host), mib * MIB)], &[])
}

fn ids(sample: &Sample) -> Vec<&str> {
    sample.recipe_ids()
}

#[test]
fn takes_the_smallest_recipe_per_first_source_host() {
    let recipes = [
        simple("big-a", "a.example", 30),
        simple("small-a", "a.example", 2),
        simple("only-b", "b.example", 9),
        simple("mid-a", "a.example", 5),
    ];
    let refs: Vec<&Recipe> = recipes.iter().collect();
    let sample = e2e_sample(&refs, CAP);
    assert_eq!(ids(&sample), ["only-b", "small-a"]);
    assert!(sample.entries.iter().all(|e| e.component.is_none()));
    assert_eq!(
        sample.entries[1].reason,
        "smallest recipe for host a.example"
    );
}

#[test]
fn a_mirror_host_does_not_count_as_a_first_source() {
    // `mirror.example` is only ever a second URL, so it needs no recipe.
    let recipes = [simple("one", "a.example", 1)];
    let refs: Vec<&Recipe> = recipes.iter().collect();
    assert_eq!(ids(&e2e_sample(&refs, CAP)), ["one"]);
}

#[test]
fn a_recipe_is_chosen_for_each_host_of_its_packages() {
    let two_hosts = recipe(
        "two-hosts",
        &[("a", urls("a.example"), MIB), ("b", urls("b.example"), MIB)],
        &[],
    );
    let other = simple("other-b", "b.example", 3);
    let refs = [&two_hosts, &other];
    let sample = e2e_sample(&refs, CAP);
    assert_eq!(ids(&sample), ["two-hosts"]);
    assert_eq!(
        sample.entries[0].reason,
        "smallest recipe for host a.example; smallest recipe for host b.example"
    );
}

#[test]
fn the_cap_skips_big_recipes_unless_none_fits_the_host() {
    let recipes = [
        simple("huge", "slow.example", 150),
        simple("small", "fast.example", 3),
        simple("big-fast", "fast.example", 100),
        simple("less-huge", "slow.example", 90),
    ];
    let refs: Vec<&Recipe> = recipes.iter().collect();
    let sample = e2e_sample(&refs, CAP);
    // `fast.example` has a small recipe; `slow.example` has none under the
    // cap, so its smallest one stands in and the host stays covered.
    assert_eq!(ids(&sample), ["less-huge", "small"]);
    let over = sample
        .entries
        .iter()
        .find(|e| e.recipe == "less-huge")
        .unwrap();
    assert!(over.reason.contains("over the size cap"), "{over:?}");
}

#[test]
fn every_free_optional_component_is_installed_once() {
    let with = recipe(
        "with",
        &[("free", urls("a.example"), 10 * MIB)],
        &[
            ("main", false, false),
            ("addon", true, false),
            ("music", true, true),
            ("hud", true, false),
        ],
    );
    let small = simple("small", "a.example", 1);
    let refs = [&with, &small];
    let sample = e2e_sample(&refs, CAP);
    let pairs: Vec<(&str, Option<&str>)> = sample
        .entries
        .iter()
        .map(|e| (e.recipe.as_str(), e.component.as_deref()))
        .collect();
    // `music` is user-supplied and cannot be installed; `main` is required.
    assert_eq!(
        pairs,
        [
            ("small", None),
            ("with", Some("addon")),
            ("with", Some("hud"))
        ]
    );
    // A component recipe that is not the host pick needs no plain install:
    // `--with` installs the base packages too.
    assert!(!pairs.contains(&("with", None)));
}

#[test]
fn a_component_recipe_over_the_cap_is_skipped_and_reported() {
    let big = recipe(
        "big",
        &[("free", urls("a.example"), 200 * MIB)],
        &[("addon", true, false)],
    );
    let small = simple("small", "a.example", 1);
    let refs = [&big, &small];
    let sample = e2e_sample(&refs, CAP);
    assert_eq!(ids(&sample), ["small"]);
    assert_eq!(sample.skipped.len(), 1);
    assert_eq!(sample.skipped[0].0, "big");
}

#[test]
fn the_sample_is_deterministic_and_input_order_does_not_matter() {
    let recipes = [
        simple("b", "a.example", 4),
        simple("a", "a.example", 4),
        simple("c", "c.example", 1),
    ];
    let forward: Vec<&Recipe> = recipes.iter().collect();
    let backward: Vec<&Recipe> = recipes.iter().rev().collect();
    let one = e2e_sample(&forward, CAP);
    assert_eq!(one, e2e_sample(&backward, CAP));
    // A size tie is broken by id.
    assert_eq!(ids(&one), ["a", "c"]);
}

#[test]
fn the_tsv_has_recipe_component_and_reason_columns() {
    let with = recipe(
        "with",
        &[("free", urls("a.example"), MIB)],
        &[("addon", true, false)],
    );
    let refs = [&with];
    let tsv = e2e_sample(&refs, CAP).to_tsv();
    assert_eq!(
        tsv,
        "with\t-\tsmallest recipe for host a.example\nwith\taddon\toptional component `addon`\n"
    );
}

fn real_catalog() -> Catalog {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog");
    let catalog = load_catalog_dir(&dir).expect("catalog/ loads");
    assert!(catalog.skipped.is_empty(), "{:?}", catalog.skipped);
    catalog
}

/// Spec 044 A4: over the real catalog every host is covered, every component
/// and status-bar recipe is in, and the size cap holds.
#[test]
fn the_real_catalog_sample_covers_hosts_components_and_the_cap() {
    let catalog = real_catalog();
    let recipes: Vec<&Recipe> = catalog.recipes.iter().map(|r| &r.recipe).collect();
    assert!(recipes.len() > 50, "the real catalog is loaded");
    let sample = e2e_sample(&recipes, DEFAULT_MAX_ARCHIVE_BYTES);
    assert!(sample.skipped.is_empty(), "{:?}", sample.skipped);

    let find = |id: &str| recipes.iter().find(|r| r.id == id).copied().unwrap();
    let sampled: BTreeSet<&str> = sample.recipe_ids().into_iter().collect();

    // Every first-source host has a sampled recipe fetching from it.
    let all_hosts: BTreeSet<String> = recipes.iter().flat_map(|r| first_hosts(r)).collect();
    assert!(all_hosts.len() >= 5, "{all_hosts:?}");
    let covered: BTreeSet<String> = sampled
        .iter()
        .flat_map(|id| first_hosts(find(id)))
        .collect();
    assert_eq!(covered, all_hosts, "a host has no sampled recipe");

    // Every recipe with free optional components is in with each of them, once.
    for recipe in &recipes {
        for component in free_components(recipe) {
            let count = sample
                .entries
                .iter()
                .filter(|e| e.recipe == recipe.id && e.component.as_deref() == Some(component))
                .count();
            assert_eq!(count, 1, "{}:{component}", recipe.id);
        }
    }

    // Every widescreen status-bar case is in (spec 017, R7).
    let status_bar: Vec<(&str, &str)> = recipes
        .iter()
        .flat_map(|r| {
            r.components
                .iter()
                .filter(|c| c.label.to_ascii_lowercase().contains("status bar"))
                .map(|c| (r.id.as_str(), c.id.as_str()))
        })
        .collect();
    assert!(status_bar.len() >= 9, "{status_bar:?}");
    for (recipe, component) in status_bar {
        assert!(
            sample
                .entries
                .iter()
                .any(|e| e.recipe == recipe && e.component.as_deref() == Some(component)),
            "{recipe}:{component} is not in the sample"
        );
    }

    // The cap holds for every sampled recipe, except a host's only fit.
    for id in &sampled {
        let recipe = find(id);
        if download_bytes(recipe) <= DEFAULT_MAX_ARCHIVE_BYTES {
            continue;
        }
        let reasons: Vec<&str> = sample
            .entries
            .iter()
            .filter(|e| e.recipe == *id)
            .map(|e| e.reason.as_str())
            .collect();
        assert!(
            reasons.iter().any(|r| r.contains("over the size cap")),
            "{id} is over the cap without being a host's only fit"
        );
    }

    // And it is lean: far fewer installs than recipes, and the 156 MB file
    // that sank the 2026-10-19 run is out.
    assert!(
        sampled.len() * 3 < recipes.len(),
        "{} of {}",
        sampled.len(),
        recipes.len()
    );
    assert!(!sampled.contains("avpvtvjvvhvss"), "{sampled:?}");
}

#[test]
fn the_real_catalog_sample_is_stable() {
    let catalog = real_catalog();
    let recipes: Vec<&Recipe> = catalog.recipes.iter().map(|r| &r.recipe).collect();
    let reversed: Vec<&Recipe> = recipes.iter().rev().copied().collect();
    assert_eq!(
        e2e_sample(&recipes, DEFAULT_MAX_ARCHIVE_BYTES),
        e2e_sample(&reversed, DEFAULT_MAX_ARCHIVE_BYTES)
    );
}

//! Worker B coverage for spec 091: the reviewed `gimmick` vocabulary entry, its
//! Discover ordering and recipe-level lint behaviour.
//!
//! These tests are offline. They read `catalog/tags.toml` through the public
//! catalog model and use the embedded catalog for a real recipe shape, without
//! editing or depending on another worker's recipe files.

use std::path::{Path, PathBuf};

use ddl_core::catalog::{
    Catalog, Recipe, TagVocabulary, validate, validate_tags, validate_vocabulary,
};

const CATALOG: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../catalog");

fn vocabulary() -> TagVocabulary {
    let text = std::fs::read_to_string(PathBuf::from(CATALOG).join("tags.toml"))
        .expect("catalog/tags.toml is readable");
    toml::from_str(&text).expect("catalog/tags.toml parses as the tag vocabulary")
}

fn embedded() -> Catalog {
    Catalog::embedded()
}

#[test]
fn gimmick_is_reviewed_with_the_required_label_and_order() {
    let vocab = vocabulary();
    assert_eq!(vocab.schema_version, 1);

    let gimmick = vocab
        .get("gimmick")
        .expect("gimmick is reviewed vocabulary");
    assert_eq!(gimmick.label, "Gimmick");
    assert_eq!(gimmick.order, 75);
    assert!(
        gimmick.description.contains("constraint") && gimmick.description.contains("concept"),
        "the description names the design constraint/central concept rule: {}",
        gimmick.description
    );

    // Exactly one definition, and every order is unique so the Discover strip is
    // deterministic.
    assert_eq!(
        vocab.tags.iter().filter(|tag| tag.id == "gimmick").count(),
        1
    );
    let mut orders: Vec<u32> = vocab.tags.iter().map(|tag| tag.order).collect();
    orders.sort_unstable();
    orders.dedup();
    assert_eq!(orders.len(), vocab.tags.len(), "tag orders are unique");
    assert!(
        validate_vocabulary(&vocab).is_empty(),
        "the reviewed vocabulary validates: {:?}",
        validate_vocabulary(&vocab)
    );
}

#[test]
fn gimmick_sits_after_challenging_and_before_metadata_only_tags() {
    let vocab = vocabulary();
    let ordered: Vec<&str> = vocab.ordered().iter().map(|tag| tag.id.as_str()).collect();

    let position = |id: &str| {
        ordered
            .iter()
            .position(|tag| *tag == id)
            .unwrap_or_else(|| panic!("{id} is in the ordered vocabulary"))
    };
    assert_eq!(
        position("gimmick"),
        position("challenging") + 1,
        "Gimmick follows Challenging in the Discover sequence"
    );
    assert!(
        position("gimmick") < position("featured"),
        "Gimmick precedes the metadata-only Featured tag"
    );
    assert!(
        position("gimmick") < position("beginner-friendly"),
        "Gimmick precedes the metadata-only Beginner Friendly tag"
    );
}

#[test]
fn embedded_catalog_accepts_the_gimmick_vocabulary() {
    let catalog = embedded();
    assert!(
        catalog
            .skipped
            .iter()
            .all(|skipped| skipped.file != "tags.toml"),
        "the embedded tag vocabulary is accepted: {:?}",
        catalog.skipped
    );
    let gimmick = catalog
        .tags
        .get("gimmick")
        .expect("the embedded catalog carries gimmick");
    assert_eq!(gimmick.label, "Gimmick");
    assert_eq!(gimmick.order, 75);
}

#[test]
fn a_recipe_may_carry_gimmick_and_unknown_or_repeated_tags_still_fail() {
    let catalog = embedded();
    let vocab = &catalog.tags;
    let hosts = &catalog.hosts;
    let loaded = catalog
        .recipes
        .iter()
        .find(|loaded| loaded.recipe.id == "no-end-in-sight")
        .expect("an embedded recipe gives the tag test a real recipe shape");
    let path = Path::new(CATALOG).join("no-end-in-sight.toml");

    let tagged: Recipe = Recipe {
        tags: vec!["gimmick".to_owned()],
        ..loaded.recipe.clone()
    };
    assert!(
        validate_tags(&tagged, vocab).is_empty(),
        "a recipe with the gimmick tag passes tag validation"
    );
    assert!(
        validate(&tagged, &path, hosts).is_empty(),
        "a recipe with the gimmick tag passes full catalog validation: {:?}",
        validate(&tagged, &path, hosts)
    );

    let duplicate: Recipe = Recipe {
        tags: vec!["gimmick".to_owned(), "gimmick".to_owned()],
        ..loaded.recipe.clone()
    };
    assert!(
        validate_tags(&duplicate, vocab)
            .iter()
            .any(|error| error.message.contains("duplicate tag `gimmick`")),
        "the vocabulary still rejects a repeated tag"
    );

    let unknown: Recipe = Recipe {
        tags: vec!["not-a-reviewed-tag".to_owned()],
        ..loaded.recipe.clone()
    };
    assert!(
        validate_tags(&unknown, vocab)
            .iter()
            .any(|error| error.message.contains("unknown tag `not-a-reviewed-tag`")),
        "the vocabulary still rejects an unknown tag"
    );
}

#[test]
fn the_10x10_project_carries_gimmick_when_its_recipe_is_embedded() {
    // Worker A owns catalog/the-10x10-project.toml and its tag assignment. In the
    // worker branch that file may not exist yet; once integrated, this asserts the
    // requested assignment rather than freezing another worker's bytes here.
    let catalog = embedded();
    if let Some(loaded) = catalog
        .recipes
        .iter()
        .find(|loaded| loaded.recipe.id == "the-10x10-project")
    {
        assert!(
            loaded.recipe.tags.iter().any(|tag| tag == "gimmick"),
            "The 10x10 Project carries the gimmick tag"
        );
    }
}

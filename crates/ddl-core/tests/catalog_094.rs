//! Spec 094 worker A: offline evidence for the player-supplied No Rest for the
//! Living expansion. The test reads only the embedded catalog; it never imports
//! or executes NERVE.WAD and never touches the network.

use ddl_core::catalog::{Catalog, Role, VerificationStatus};

const ID: &str = "no-rest-for-the-living";
const BFG_SHA256: &str = "2d234de8700ee403375fda278e50ea17739b36703b3316e0429b05aca4458fa9";

fn recipe<'a>(catalog: &'a Catalog) -> &'a ddl_core::catalog::Recipe {
    catalog
        .recipe(ID)
        .unwrap_or_else(|| panic!("{ID} must be embedded"))
}

/// The metadata-only recipe must embed and must not declare any download.
#[test]
fn player_supplied_recipe_embeds_without_packages_or_skips() {
    let catalog = Catalog::embedded();
    assert!(
        catalog.skipped.is_empty(),
        "the shipped catalog must be clean: {:?}",
        catalog.skipped
    );
    let recipe = recipe(&catalog);
    assert_eq!(recipe.id, ID);
    assert_eq!(recipe.title, "No Rest for the Living");
    assert_eq!(recipe.authors, ["Nerve Software"]);
    assert!(
        recipe.packages.is_empty(),
        "the expansion is player-supplied and has no package"
    );
    assert!(
        recipe.release.contains("2010-05-26"),
        "the release text carries the first-public date"
    );
    assert!(
        recipe.permission.contains("No redistribution permission"),
        "the recipe claims no redistribution permission"
    );
}

/// Doom II must be the IWAD; NERVE.WAD is the imported expansion PWAD.
#[test]
fn expansion_requires_doom2_and_accepts_classic_and_kex_variants() {
    let catalog = Catalog::embedded();
    let recipe = recipe(&catalog);
    assert_eq!(recipe.base_game.game, ddl_core::catalog::Game::Doom2);
    assert_eq!(
        recipe.base_game.variants,
        [
            "doom2-1.9",
            "doom2-1.9-kex-2024-08-06",
            "doom2-1.9-kex-2024-10-01"
        ],
        "classic 1.9 and both reviewed Steam/KEX Doom II variants"
    );
    assert!(
        !recipe
            .base_game
            .variants
            .iter()
            .any(|variant| variant.contains("nerve")),
        "the expansion PWAD is never modelled as an IWAD"
    );
}

/// The required main is exactly one imported maps member, with the inspected
/// BFG SHA-256 and no package/member reference.
#[test]
fn required_main_is_a_single_user_supplied_map_component() {
    let catalog = Catalog::embedded();
    let recipe = recipe(&catalog);
    assert_eq!(recipe.components.len(), 1);
    let main = &recipe.components[0];
    assert_eq!(main.id, "main");
    assert!(!main.optional, "the imported expansion is required");
    assert!(main.default, "the required main is on by default");
    assert_eq!(main.position, 1);
    assert!(
        main.user_supplied,
        "DDL never downloads the commercial file"
    );
    assert_eq!(main.members.len(), 1);
    assert_eq!(main.members[0].role, Role::Maps);
    assert_eq!(main.members[0].package, None);
    assert_eq!(main.members[0].member, None);
    assert_eq!(main.known_sha256, [BFG_SHA256]);
    let obtain = main.obtain.as_deref().expect("import instructions");
    assert!(
        obtain.to_ascii_lowercase().contains("nerve.wad"),
        "the instructions preserve the filename used by DSDA"
    );

    let order = recipe
        .load_order(&[])
        .expect("the required main must load by default");
    assert_eq!(order.files.len(), 1);
    assert_eq!(order.files[0].role, Role::Maps);
    assert_eq!(order.files[0].package, None);
    assert_eq!(order.files[0].member, None);
    assert!(order.deh.is_empty());
}

/// Evidence and categories: inspected status, an explicit unknown-hash caveat,
/// and the 2010 Cacowards/episode/classics memberships.
#[test]
fn evidence_keeps_the_bfg_hash_kex_caveat_and_earned_tags() {
    let catalog = Catalog::embedded();
    let recipe = recipe(&catalog);
    assert_eq!(recipe.verification.status, VerificationStatus::Inspected);
    assert_eq!(recipe.verification.engine_version, "0.30.1");
    assert_eq!(recipe.compatibility.engine.range, "=0.30.1");
    assert!(
        recipe
            .verification
            .notes
            .contains("KEX SHA-256 is not published or inspected"),
        "the KEX hash limitation is explicit"
    );
    assert!(
        recipe
            .verification
            .notes
            .contains("reported as an unrecognized hash"),
        "an unrecognized KEX import is not labelled verified"
    );
    assert!(
        recipe.verification.notes.contains("cacowards/2010"),
        "the Cacowards citation is present"
    );
    assert!(
        recipe
            .verification
            .limitations
            .iter()
            .any(|line| line.contains("Needs Deck Validation")),
        "gameplay remains unverified"
    );

    let mut tags = recipe.tags.clone();
    tags.sort_unstable();
    assert_eq!(tags, ["cacowards", "classics", "episode"]);
    for tag in &recipe.tags {
        assert!(
            catalog.tags.get(tag).is_some(),
            "the recipe uses reviewed vocabulary only"
        );
    }
    let description = recipe.description.to_ascii_lowercase();
    for term in [
        "dehacked",
        "mapinfo",
        "umapinfo",
        "complevel",
        "lump",
        "boom",
    ] {
        assert!(
            !description.contains(term),
            "product copy must not leak `{term}`"
        );
    }
}

/// The supplied BFG bytes are not fixture data: this test only pins the imported
/// file's identity through the recipe. A comment-level KEX fingerprint is checked
/// as text so the published metadata cannot be replaced by an invented SHA-256.
#[test]
fn kex_metadata_is_documented_without_an_invented_sha256() {
    let catalog = Catalog::embedded();
    let recipe = recipe(&catalog);
    let main = &recipe.components[0];
    assert_eq!(main.known_sha256.len(), 1);
    assert_eq!(main.known_sha256[0], BFG_SHA256);
    assert!(
        !main
            .known_sha256
            .iter()
            .any(|hash| hash == "23422eb42833ac7b0dd59c0c7ae18a6f"),
        "the KEX MD5 is not a SHA-256"
    );
    assert!(
        recipe
            .verification
            .inspected
            .iter()
            .any(|line| line.contains("Owner-supplied NERVE.WAD")),
        "the evidence states what was actually inspected"
    );
}

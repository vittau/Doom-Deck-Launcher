//! Spec 093: Haste.
//!
//! Offline tests for the one recipe this worker owns: source identity,
//! composition, base game/complevel, tag evidence, product-copy rules, dates and
//! the bundled screenshot. Values come from `docs/research/haste.md`. The
//! coordinator owns the combined recipe-count assertion.

use ddl_core::catalog::{Catalog, Complevel, ComplevelAlias, Game, PortFamily, Recipe, Role};

const RECIPE: &str = "haste";

fn recipe() -> Recipe {
    Catalog::embedded()
        .recipe(RECIPE)
        .unwrap_or_else(|| panic!("{RECIPE} must load from the embedded catalog"))
        .clone()
}

/// The recipe embeds and is never skipped.
#[test]
fn recipe_embeds_without_skips() {
    let catalog = Catalog::embedded();
    assert!(
        catalog.get(RECIPE).is_some(),
        "{RECIPE} must embed; skipped: {:?}",
        catalog.skipped
    );
    assert!(
        !catalog
            .skipped
            .iter()
            .any(|skipped| skipped.file == "haste.toml"),
        "the recipe must not be skipped: {:?}",
        catalog.skipped
    );
}

/// Package and case-exact member identities, pinned from the bytes.
#[test]
fn package_and_members_keep_their_inspected_identity() {
    let r = recipe();
    assert_eq!(r.packages.len(), 1);
    let package = &r.packages[0];
    assert_eq!(package.id, "free");
    assert_eq!(package.size, 5_519_763);
    assert_eq!(
        package.sha256,
        "f71eb1623da6c5fc02c998499dc123b45db5c8f56ad3d8a5f92bbc800f033f12"
    );
    assert_eq!(
        package.idgames_path.as_deref(),
        Some("levels/doom2/Ports/megawads/haste.zip")
    );
    assert_eq!(package.urls.len(), 3, "three verified HTTPS mirrors");
    assert!(
        package.urls.iter().all(|url| url.starts_with("https://")),
        "all mirrors are HTTPS"
    );

    assert_eq!(package.members.len(), 2, "maps and credits");
    let maps = &package.members[0];
    assert_eq!(maps.id, "maps");
    assert_eq!(maps.path, "Haste.wad", "case-exact map WAD");
    assert_eq!(maps.size, 16_931_409);
    assert_eq!(
        maps.sha256,
        "fac36c642771ccd7336814bbff443e093ddf09469b156017a5bf2d7312d01818"
    );
    assert!(!maps.keep, "the map WAD is loaded, not kept");

    let text = &package.members[1];
    assert_eq!(text.id, "credits");
    assert_eq!(text.path, "Haste.txt", "case-exact credits file");
    assert_eq!(text.size, 4_629);
    assert_eq!(
        text.sha256,
        "8bdd5194682b362b4f3884d74ae65564a05e27d2abd8d6c8a4f2918ca966368d"
    );
    assert!(text.keep, "the unmodified credits file is kept");
}

/// One mandatory component, one loaded file, DEHACKED embedded in the WAD.
#[test]
fn load_order_is_the_single_map_wad_with_embedded_deh() {
    let r = recipe();
    assert_eq!(r.components.len(), 1);
    let main = &r.components[0];
    assert_eq!(main.id, "main");
    assert!(!main.optional);
    assert!(main.default);
    assert_eq!(main.position, 1);
    assert!(!main.user_supplied);
    assert_eq!(main.members.len(), 1);
    assert_eq!(main.members[0].role, Role::Maps);
    assert_eq!(main.members[0].package.as_deref(), Some("free"));
    assert_eq!(main.members[0].member.as_deref(), Some("maps"));

    let order = r.load_order(&[]).expect("the default selection is valid");
    assert_eq!(order.files.len(), 1);
    assert_eq!(order.files[0].member.as_deref(), Some("maps"));
    assert!(order.deh.is_empty(), "no separate -deh member");
    assert!(r.default_components().is_empty(), "no default options");
}

/// DOOM II, the author-stated family and the boom alias.
#[test]
fn base_game_and_complevel_match_the_author_text() {
    let r = recipe();
    assert_eq!(r.base_game.game, Game::Doom2);
    assert!(r.base_game.variants.is_empty());
    assert_eq!(r.compatibility.family, PortFamily::Boom);
    assert_eq!(
        r.compatibility.complevel,
        Complevel::Alias(ComplevelAlias::Boom)
    );
    assert_eq!(r.compatibility.engine.range, "=0.30.1");
}

/// Source-backed tags and the counted slots: eighteen, so never `megawad`;
/// the Cacowards write-up calls it approachable, so never `challenging`.
#[test]
fn tags_follow_the_evidence_and_map_count() {
    let catalog = Catalog::embedded();
    let r = recipe();
    for tag in &r.tags {
        assert!(
            catalog.tags.get(tag).is_some(),
            "reviewed tag id expected: {tag}"
        );
    }
    for tag in ["cacowards", "community-projects", "classics"] {
        assert!(r.tags.iter().any(|held| held == tag), "missing {tag}");
    }
    for tag in ["megawad", "episode", "challenging", "gimmick"] {
        assert!(
            !r.tags.iter().any(|held| held == tag),
            "tag {tag} is not evidenced for eighteen approachable maps"
        );
    }
    assert!(
        r.verification
            .notes
            .contains("https://www.doomworld.com/cacowards/2021/best4/"),
        "the exact Cacowards page is cited"
    );
    assert!(
        r.verification.notes.contains("winner"),
        "the Cacowards role is stated"
    );
    assert!(
        r.verification
            .notes
            .contains("hardcoded Doom II next-map table"),
        "progression evidence stays in verification"
    );
}

/// Product copy stays about the experience: no port-family names, map-slot
/// codes, lumps, patches or engine names in root/component descriptions.
#[test]
fn descriptions_stay_product_copy() {
    let r = recipe();
    assert!(
        r.description.starts_with("DOOM II:"),
        "the description opens with the game: {}",
        r.description
    );
    for jargon in [
        "Boom", "boom", "MAPINFO", "DEHACKED", "COMPLVL", "UMAPINFO", "lump", "DSDA", "MAP0",
        "MAP31", "PrBoom",
    ] {
        assert!(
            !r.description.contains(jargon),
            "root description must not carry {jargon}: {}",
            r.description
        );
        for component in &r.components {
            assert!(
                !component.description.contains(jargon),
                "component `{}` must not carry {jargon}: {}",
                component.id,
                component.description
            );
        }
    }
    assert!(
        !r.verification.limitations.is_empty(),
        "technical limitations are recorded honestly"
    );
}

/// The readme date is day-first and the archive upload is a later identity.
#[test]
fn dates_record_the_original_release_and_the_archive_identity() {
    let r = recipe();
    assert!(
        r.release.contains("2021-09-22"),
        "the readme release date is 2021-09-22: {}",
        r.release
    );
    assert!(
        r.release.contains("2022-01-08"),
        "the idgames upload is dated 2022-01-08: {}",
        r.release
    );
}

/// The bundled screenshot is referenced with the project leads' credit.
#[test]
fn bundled_screenshot_is_referenced() {
    let r = recipe();
    assert_eq!(r.screenshots.len(), 1);
    assert_eq!(r.screenshots[0].asset, "haste/01.jpg");
    assert_eq!(r.screenshots[0].credit, "Insane_Gazebo & Scotty");
}

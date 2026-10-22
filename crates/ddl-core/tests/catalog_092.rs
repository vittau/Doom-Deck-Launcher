//! Spec 092: Swim With The Whales.
//!
//! These tests freeze the inspected source identity, composition, base game,
//! complevel, date decision, tag claims and product-copy rules for the one
//! recipe this worker owns. They are offline; every value comes from the bytes
//! recorded in `docs/research/swim-with-the-whales.md`. The coordinator owns
//! the combined recipe-count assertion.

use ddl_core::catalog::{Catalog, Complevel, ComplevelAlias, Game, PortFamily, Recipe, Role};

const RECIPE: &str = "swim-with-the-whales";

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
            .any(|skipped| skipped.file == "swim-with-the-whales.toml"),
        "the recipe must not be skipped: {:?}",
        catalog.skipped
    );
}

/// Package and case-exact nested member identities, pinned from the bytes.
#[test]
fn package_and_members_keep_their_inspected_identity() {
    let r = recipe();
    assert_eq!(r.packages.len(), 1, "one package");
    let package = &r.packages[0];
    assert_eq!(package.id, "free");
    assert_eq!(package.size, 2_719_261);
    assert_eq!(
        package.sha256,
        "ebd2317a3006614ce4afbd1f8c8294f34949e872860cbc2a17ed8de59d73aef7"
    );
    assert_eq!(
        package.idgames_path.as_deref(),
        Some("levels/doom2/Ports/s-u/swtw.zip")
    );
    assert_eq!(package.urls.len(), 3, "three verified HTTPS mirrors");
    assert!(
        package.urls.iter().all(|url| url.starts_with("https://")),
        "all mirrors are HTTPS"
    );

    assert_eq!(package.members.len(), 2, "maps and credits");
    let maps = &package.members[0];
    assert_eq!(maps.id, "maps");
    assert_eq!(maps.path, "swtw/swtw.wad", "case-exact nested map WAD");
    assert_eq!(maps.size, 8_868_645);
    assert_eq!(
        maps.sha256,
        "7f081d591f2fce923d08fe1f86ed52cc5b8bd10061c2c96711506ea6286390d7"
    );
    assert!(!maps.keep, "the map WAD is loaded, not kept");

    let text = &package.members[1];
    assert_eq!(text.id, "credits");
    assert_eq!(text.path, "swtw/swtw.txt", "case-exact nested credits file");
    assert_eq!(text.size, 8_328);
    assert_eq!(
        text.sha256,
        "521cb605249c52bbe3015cd62c1419c047bfcd2d4409cc17cfda5a106cf6dba0"
    );
    assert!(text.keep, "the unmodified credits file is kept");
}

/// One mandatory component, one loaded file, and the DEHACKED patch embedded
/// in the WAD rather than declared as a separate `-deh` member.
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
    assert!(
        order.deh.is_empty(),
        "the DEHACKED patch is embedded in the WAD, not a separate -deh member"
    );
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

/// Source-backed tags, counted slots and the owner's gimmick boundary.
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
    for tag in ["cacowards", "classics", "episode", "challenging"] {
        assert!(r.tags.iter().any(|held| held == tag), "missing {tag}");
    }
    assert!(
        !r.tags.iter().any(|tag| tag == "megawad"),
        "four slots are not a megawad"
    );
    assert!(
        !r.tags.iter().any(|tag| tag == "gimmick"),
        "the owner's gimmick tag covers The 10x10 Project and Linear Doom only"
    );
    assert!(
        r.verification
            .notes
            .contains("https://www.doomworld.com/cacowards/2013/"),
        "the Cacowards year page is cited"
    );
    assert!(
        r.verification.notes.contains("runner-up"),
        "the Cacowards role is stated"
    );
    assert!(
        r.verification
            .notes
            .contains("hardcoded Doom II next-map table"),
        "progression evidence stays in verification"
    );
    assert!(
        r.verification.notes.contains("base IWAD"),
        "the post-set fallback is documented"
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
        "MAP31", "Glboom", "PrBoom",
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

/// Original public date and precision, kept separate from the archive upload.
#[test]
fn dates_record_the_original_release_and_the_archive_identity() {
    let r = recipe();
    assert!(
        r.release.contains("2013-12-02"),
        "the readme date is 2013-12-02: {}",
        r.release
    );
    assert!(
        r.release.contains("2013-11-20"),
        "the first public release is 2013-11-20: {}",
        r.release
    );
    assert!(
        r.release.contains("2013-12-21"),
        "the idgames upload is dated 2013-12-21: {}",
        r.release
    );
    assert!(
        !r.release.contains("2013-02-12"),
        "the readme date is not February: {}",
        r.release
    );
}

/// The bundled screenshot is referenced with the author's credit.
#[test]
fn bundled_screenshot_is_referenced() {
    let r = recipe();
    assert_eq!(r.screenshots.len(), 1);
    assert_eq!(r.screenshots[0].asset, "swim-with-the-whales/01.jpg");
    assert_eq!(r.screenshots[0].credit, "Ribbiks");
}

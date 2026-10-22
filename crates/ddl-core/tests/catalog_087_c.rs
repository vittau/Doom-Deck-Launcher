//! Spec 087 worker C, extended by specs 088 and 089: Arrival, Vanguard, Lunatic,
//! Overboard, Headless Chicken and Micro-Slaughter Community Project.
//!
//! These tests freeze the source identity, composition, load order, base game,
//! complevel, metadata claims and tag claims of the six recipes this worker owns.
//! They are offline: every value comes from the inspected archives recorded in
//! `docs/research/reference-087-c.md`. The coordinator owns the combined
//! recipe-count assertion.

use ddl_core::catalog::{
    Catalog, Complevel, ComplevelAlias, Game, PortFamily, Recipe, Role, VerificationStatus,
};

/// Every recipe this worker owns, with the values inspected from upstream bytes.
const ASSIGNED: [&str; 6] = [
    "arrival",
    "vanguard",
    "lunatic",
    "overboard",
    "headless-chicken",
    "micro-slaughter-community-project",
];

fn recipe(id: &str) -> Recipe {
    Catalog::embedded()
        .recipe(id)
        .unwrap_or_else(|| panic!("{id} must load from the embedded catalog"))
        .clone()
}

/// No assigned recipe may be skipped by the embedded loader.
#[test]
fn assigned_recipes_embed_without_skips() {
    let catalog = Catalog::embedded();
    for id in ASSIGNED {
        assert!(
            catalog.get(id).is_some(),
            "{id} must embed; skipped: {:?}",
            catalog.skipped
        );
        let file = format!("{id}.toml");
        assert!(
            !catalog.skipped.iter().any(|skipped| skipped.file == file),
            "{file} must not be skipped: {:?}",
            catalog.skipped
        );
    }
}

/// Package, member and kept-credit identities, pinned from the downloaded bytes.
#[test]
fn packages_and_members_keep_their_inspected_identity() {
    // id, package size, package sha256, wad path, wad size, wad sha256, text path, text size, text sha256
    let expected = [
        (
            "arrival",
            2_552_025_u64,
            "3907dbe4dfb39129ef2ff3b957640ddea7a7cad4e7d00e4a7b04adaa431992aa",
            "arrival.wad",
            7_820_717_u64,
            "6e499ff4c0ca7afdd94bf9696be9aaa7594cc767331c56dd8f24d0c9f66acda9",
            "arrival.txt",
            4_063_u64,
            "a343d200cde1e70669df39a5b528ae18ece39f832c3c12d5bfd6f8350a66870d",
        ),
        (
            "vanguard",
            9_910_540_u64,
            "d37b5bd01b7c9902be97f1d42c1c1f68cf4aaeb9ac97ab8cce88fcd1b459dbef",
            "Vanguard.wad",
            34_897_578_u64,
            "52807ec27756d23dedc388c8c39b953797fa2da1c8339303de63141a0cebc465",
            "Vanguard.txt",
            4_223_u64,
            "ceab9fcf5532157aa65fa5f3a69143d2a8711c86ef5bd6dd066340eca047bdf2",
        ),
        (
            "lunatic",
            4_058_779_u64,
            "01e393282b23af6e854d8925f958ad7fb769f004cdfaa97b815bb0d9b999e616",
            "Lunatic.wad",
            18_160_632_u64,
            "4ef88cf44d7d15effdf634811e3ae21fceaee21336870879c95af75b08783c9f",
            "Lunatic.txt",
            3_579_u64,
            "93f0b9a52f1006c49fef41c341d86c5d3d83992418cafdd12fb00beb290f73cf",
        ),
        (
            "overboard",
            7_728_312_u64,
            "9dddda5bb754192c97507646c65f214a378206015d5fa1995bf75269bb02356a",
            "overboard.wad",
            25_265_537_u64,
            "608218886e8a61086040d28045d81644801c33d1265f343cd6cf62d36f7b3e95",
            "overboard.txt",
            3_135_u64,
            "b4cf202073939a044f361fa1180e23b789becf6b9c82f42996fd96a5d37f3ce2",
        ),
        (
            "headless-chicken",
            38_085_270_u64,
            "de8926c8d3c22dc739718eb06c18da994f3a9fcd12c0f404cbc4c342ec5c4659",
            "HDLSCHKN.wad",
            126_358_308_u64,
            "5b7620307c6c8f0203508c1e380a563dda0d62172e65f32406166c3e427380ab",
            "hdlschkn.txt",
            27_158_u64,
            "1a09ef102856ba673bc4b5dcce954b630be97dca086259ef0490e7ee0a1d3ca3",
        ),
        (
            "micro-slaughter-community-project",
            25_051_502_u64,
            "0e708e0e871a583d522b09c45ba2f00466a66c957f408aad5f710e19fb1a09cc",
            "MSCP_v1b.wad",
            86_441_134_u64,
            "c28e7d90342abd2bab37a243f1078b44dc8bce22e2ee6490f8009d5502f02cf6",
            "MSCP_v1b.txt",
            5_455_u64,
            "57048764375bd4e8129c6a4cd2609854d17498535186c4e1b861fe7ef6073a4e",
        ),
    ];
    for (id, pkg_size, pkg_sha, maps_path, maps_size, maps_sha, text_path, text_size, text_sha) in
        expected
    {
        let r = recipe(id);
        assert_eq!(r.packages.len(), 1, "{id} has one package");
        let package = &r.packages[0];
        assert_eq!(package.id, "free", "{id}");
        assert_eq!(package.size, pkg_size, "{id} package size");
        assert_eq!(package.sha256, pkg_sha, "{id} package hash");
        let idgames_path = idgames_path(id);
        assert_eq!(
            package.idgames_path.as_deref(),
            Some(idgames_path),
            "{id} idgames path"
        );
        assert_eq!(package.urls.len(), 3, "{id} lists three verified mirrors");
        assert!(
            package.urls.iter().all(|url| url.starts_with("https://")),
            "{id} mirrors are HTTPS"
        );

        assert_eq!(package.members.len(), 2, "{id} declares maps and credits");
        let maps = &package.members[0];
        assert_eq!(maps.id, "maps", "{id}");
        assert_eq!(maps.path, maps_path, "{id} case-exact map WAD");
        assert_eq!(maps.size, maps_size, "{id} WAD size");
        assert_eq!(maps.sha256, maps_sha, "{id} WAD hash");
        assert!(!maps.keep, "{id} map WAD is loaded, not kept");

        let text = &package.members[1];
        assert_eq!(text.id, "credits", "{id}");
        assert_eq!(text.path, text_path, "{id} case-exact credits file");
        assert_eq!(text.size, text_size, "{id} credits size");
        assert_eq!(text.sha256, text_sha, "{id} credits hash");
        assert!(text.keep, "{id} keeps the unmodified credits file");
    }
}

/// The archive filename is not always the recipe id, so every path is pinned.
fn idgames_path(id: &str) -> &'static str {
    match id {
        "arrival" => "levels/doom2/Ports/a-c/arrival.zip",
        "vanguard" => "levels/doom2/Ports/v-z/vanguard.zip",
        "lunatic" => "levels/doom2/Ports/j-l/lunatic.zip",
        "overboard" => "levels/doom2/Ports/megawads/overboard.zip",
        "headless-chicken" => "levels/doom2/Ports/megawads/hdlschkn.zip",
        "micro-slaughter-community-project" => "levels/doom2/Ports/megawads/mscp_v1b.zip",
        other => panic!("unexpected recipe id {other}"),
    }
}

/// One mandatory component, the map WAD in position 1, and no external patch:
/// the DEHACKED patch of every release is embedded in the WAD itself, so its
/// bytes are covered by the pinned WAD hash rather than a separate member.
#[test]
fn load_order_is_the_single_map_wad_and_deh_is_embedded() {
    for id in ASSIGNED {
        let r = recipe(id);
        assert_eq!(r.components.len(), 1, "{id} has only the main component");
        let main = &r.components[0];
        assert_eq!(main.id, "main", "{id}");
        assert!(!main.optional, "{id} main is required");
        assert!(main.default, "{id} main is on by default");
        assert_eq!(main.position, 1, "{id} main loads first");
        assert!(main.replaces.is_none(), "{id}");
        assert!(!main.user_supplied, "{id}");
        assert_eq!(main.members.len(), 1, "{id} loads exactly one file");
        assert_eq!(main.members[0].role, Role::Maps, "{id}");
        assert_eq!(main.members[0].package.as_deref(), Some("free"), "{id}");
        assert_eq!(main.members[0].member.as_deref(), Some("maps"), "{id}");

        let order = r.load_order(&[]).expect("the default selection is valid");
        assert_eq!(order.files.len(), 1, "{id}");
        assert_eq!(order.files[0].member.as_deref(), Some("maps"), "{id}");
        assert!(
            order.deh.is_empty(),
            "{id}: the DEHACKED patch is embedded in the WAD, not a separate -deh member"
        );
        assert!(
            r.default_components().is_empty(),
            "{id} has no default options"
        );
    }
}

/// DOOM II, the inspected port family and the alias the author's own text names.
#[test]
fn base_game_and_complevel_match_the_author_text() {
    for id in ASSIGNED {
        let r = recipe(id);
        assert_eq!(r.base_game.game, Game::Doom2, "{id} needs DOOM II");
        assert!(
            r.base_game.variants.is_empty(),
            "{id} accepts any DOOM II IWAD"
        );
        assert_eq!(r.compatibility.engine.status, VerificationStatus::Inspected);
        assert_eq!(r.compatibility.engine.range, "=0.30.1");
    }

    let arrival = recipe("arrival");
    assert_eq!(arrival.compatibility.family, PortFamily::LimitRemoving);
    assert_eq!(
        arrival.compatibility.complevel,
        Complevel::Alias(ComplevelAlias::Vanilla)
    );

    for id in [
        "vanguard",
        "lunatic",
        "overboard",
        "headless-chicken",
        "micro-slaughter-community-project",
    ] {
        let r = recipe(id);
        assert_eq!(r.compatibility.family, PortFamily::Boom, "{id} is Boom");
        assert_eq!(
            r.compatibility.complevel,
            Complevel::Alias(ComplevelAlias::Boom),
            "{id} uses the boom alias"
        );
    }

    assert_eq!(
        recipe("micro-slaughter-community-project")
            .compatibility
            .runtime
            .pistol_start,
        Some(true),
        "the author strongly recommends pistol starts"
    );
}

/// Product copy stays plain: DEHACKED/MAPINFO/EMAPINFO/UMAPINFO/COMPLVL and
/// engine-source talk belongs in verification, not in descriptions. The same
/// test keeps the ZDoom-MAPINFO correction and the hardcoded progression
/// evidence in place.
#[test]
fn descriptions_are_product_copy_and_technical_evidence_stays_in_verification() {
    for id in ASSIGNED {
        let r = recipe(id);
        assert!(
            r.description.starts_with("DOOM II:"),
            "{id} description should open with the game and read as product copy: {}",
            r.description
        );
        for jargon in [
            "DEHACKED", "MAPINFO", "EMAPINFO", "UMAPINFO", "COMPLVL", "lump", "DSDA", "legacy.c",
        ] {
            assert!(
                !r.description.contains(jargon),
                "{id} root description must not carry {jargon}: {}",
                r.description
            );
            for component in &r.components {
                assert!(
                    !component.description.contains(jargon),
                    "{id} component `{}` must not carry {jargon}: {}",
                    component.id,
                    component.description
                );
            }
        }
        assert!(
            !r.verification.notes.trim().is_empty(),
            "{id} keeps the technical evidence in verification.notes"
        );
    }

    for id in ["vanguard", "lunatic"] {
        let r = recipe(id);
        assert!(
            r.verification.notes.contains("ZDoom syntax"),
            "{id} names the ZDoom syntax in its evidence"
        );
        assert!(
            r.verification
                .notes
                .contains("hardcoded Doom II next-map table"),
            "{id} documents the hardcoded progression"
        );
        assert!(
            r.verification.notes.contains("base IWAD"),
            "{id} documents where progression continues after the last custom map"
        );
        assert!(
            r.verification
                .limitations
                .iter()
                .any(|limit| limit.contains("base IWAD")),
            "{id} records the fallback as a limitation"
        );
    }
    let lunatic = recipe("lunatic");
    assert!(
        lunatic.verification.notes.contains("EMAPINFO")
            && lunatic.verification.notes.contains("Eternity"),
        "Lunatic's EMAPINFO is described as Eternity metadata"
    );
}

/// The two releases without UMAPINFO document the same stock-map fallback.
#[test]
fn releases_without_umapinfo_document_the_stock_map_fallback() {
    for id in ["arrival", "overboard"] {
        let r = recipe(id);
        assert!(
            r.verification
                .limitations
                .iter()
                .any(|limit| limit.contains("base IWAD")),
            "{id} documents the post-run fallback"
        );
    }
    for id in ["headless-chicken", "micro-slaughter-community-project"] {
        assert!(
            recipe(id).verification.notes.contains("UMAPINFO"),
            "{id} documents its UMAPINFO-driven progression"
        );
    }
}

/// The two release labels that were normalised wrongly are fixed, with the
/// evidence kept in verification (spec 090 chronology is worker D's field).
#[test]
fn corrected_release_labels_hold_their_sourced_dates() {
    let overboard = recipe("overboard");
    assert!(
        overboard.release.contains("2022-08-04"),
        "Overboard is 4 August 2022 (readme 04/08/22, day/month): {}",
        overboard.release
    );
    assert!(
        !overboard.release.contains("2022-04-08"),
        "the April reading was a normalisation error: {}",
        overboard.release
    );

    let chicken = recipe("headless-chicken");
    assert!(
        chicken.release.contains("2023-12-09"),
        "Headless Chicken RC1 is 9 December 2023 (readme 9/12/2023, day/month): {}",
        chicken.release
    );
    assert!(
        !chicken.release.contains("2023-09-12"),
        "the September reading was a normalisation error: {}",
        chicken.release
    );

    for (id, date) in [
        ("arrival", "2021-04-16"),
        ("vanguard", "2011-05-11"),
        ("lunatic", "2011-08-11"),
        ("micro-slaughter-community-project", "2021-01-27"),
    ] {
        let r = recipe(id);
        assert!(
            r.release.contains(date),
            "{id} keeps its sourced original date {date}: {}",
            r.release
        );
    }
}

/// Tags follow their vocabulary definitions: counted slots and credited teams.
#[test]
fn tags_follow_the_counted_map_slots_and_credits() {
    let catalog = Catalog::embedded();
    for id in ASSIGNED {
        let r = recipe(id);
        for tag in &r.tags {
            assert!(
                catalog.tags.get(tag).is_some(),
                "{id} uses a reviewed tag id: {tag}"
            );
        }
    }

    for id in ["arrival", "vanguard", "lunatic", "overboard"] {
        let r = recipe(id);
        assert!(r.tags.iter().any(|tag| tag == "cacowards"), "{id}");
        assert!(
            !r.tags.iter().any(|tag| tag == "megawad"),
            "{id} has fewer than thirty maps"
        );
    }
    assert!(recipe("lunatic").tags.iter().any(|tag| tag == "episode"));
    for id in ["arrival", "vanguard", "overboard"] {
        assert!(
            !recipe(id).tags.iter().any(|tag| tag == "episode"),
            "{id} has more than nine map slots"
        );
    }

    let chicken = recipe("headless-chicken");
    assert!(
        chicken.tags.iter().any(|tag| tag == "megawad"),
        "33 map slots earn the megawad tag"
    );
    assert!(
        !chicken.tags.iter().any(|tag| tag == "cacowards"),
        "no Cacowards entry was found for Headless Chicken"
    );

    let mscp = recipe("micro-slaughter-community-project");
    assert!(
        mscp.tags.iter().any(|tag| tag == "community-projects"),
        "fifteen credited mappers"
    );
    assert!(
        !mscp
            .tags
            .iter()
            .any(|tag| tag == "megawad" || tag == "episode"),
        "19 slots are neither a megawad nor an episode"
    );
    assert_eq!(mscp.authors.len(), 15, "every credited mapper is listed");
}

/// Each `cacowards` tag carries the year page the lint requires, and the
/// 2011 edition still uses its legacy `/18years/` paths.
#[test]
fn cacowards_evidence_names_the_year_page_and_role() {
    for (id, page) in [
        ("arrival", "https://www.doomworld.com/cacowards/2021/best3/"),
        ("vanguard", "https://www.doomworld.com/cacowards/2011/"),
        ("lunatic", "https://www.doomworld.com/cacowards/2011/"),
        (
            "overboard",
            "https://www.doomworld.com/cacowards/2022/best3/",
        ),
    ] {
        let r = recipe(id);
        assert!(r.verification.notes.contains(page), "{id} cites {page}");
        assert!(
            r.verification.notes.contains("winner"),
            "{id} states the Cacowards role"
        );
    }
    let vanguard = recipe("vanguard");
    assert!(vanguard.verification.notes.contains("18years/best2.php"));
}

/// Every assigned recipe that ships a bundled in-game screenshot links it.
#[test]
fn bundled_screenshots_are_referenced() {
    for (id, credit) in [
        (
            "arrival",
            "Walker \"Pavera\" Wright & Brayden \"AD_79\" Hart",
        ),
        ("overboard", "Cyriak Harris (mouldy)"),
    ] {
        let r = recipe(id);
        assert_eq!(r.screenshots.len(), 1, "{id}");
        assert_eq!(r.screenshots[0].asset, format!("{id}/01.jpg"), "{id}");
        assert_eq!(r.screenshots[0].credit, credit, "{id}");
    }
    for id in [
        "vanguard",
        "lunatic",
        "headless-chicken",
        "micro-slaughter-community-project",
    ] {
        assert!(
            recipe(id).screenshots.is_empty(),
            "{id} uses the painted fallback art: no obtainable source was found"
        );
    }
}

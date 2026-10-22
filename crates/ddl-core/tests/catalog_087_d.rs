//! Spec 087 worker D and its spec 088/089 extensions: the UAC Ultra, Japanese
//! Community Project, Stardate 20X6, Ozonia and Abandon recipes.
//!
//! Offline only: every fact here comes from `Catalog::embedded()`, i.e. the recipes
//! committed under `catalog/`, whose package/member bytes were verified separately
//! with `ddl catalog lint --fetch` (recorded in `docs/reports/087-d.md`). These tests
//! pin the inspected identities so a later edit cannot silently swap a release,
//! reorder the load list or drop a check needed for a correct launch.

use ddl_core::catalog::{
    Catalog, Complevel, ComplevelAlias, Game, PackageKind, PortFamily, Recipe, Role,
    VerificationStatus,
};

/// The ids this worker owns: spec 087 R1 (three), spec 088 R1 (Ozonia) and
/// spec 089 R1 (Abandon).
const ASSIGNED: [&str; 5] = [
    "uac-ultra",
    "japanese-community-project",
    "stardate-20x6",
    "ozonia",
    "abandon",
];

fn recipe(id: &str) -> Recipe {
    Catalog::embedded()
        .recipe(id)
        .unwrap_or_else(|| panic!("`{id}` is not in the embedded catalog"))
        .clone()
}

#[test]
fn assigned_recipes_embed_without_being_skipped() {
    let catalog = Catalog::embedded();
    for id in ASSIGNED {
        let file = format!("{id}.toml");
        assert!(
            catalog.skipped.iter().all(|skipped| skipped.file != file),
            "`{file}` was skipped: {:?}",
            catalog
                .skipped
                .iter()
                .find(|skipped| skipped.file == file)
                .map(|skipped| skipped.reason.as_str())
        );
        let loaded = catalog
            .get(id)
            .unwrap_or_else(|| panic!("`{id}` is not in the embedded catalog"));
        assert_eq!(loaded.file, file);
        assert_eq!(loaded.recipe.id, id);
        assert_eq!(
            loaded.recipe.verification.status,
            VerificationStatus::Inspected,
            "`{id}` must stay inspected until a Deck gameplay test exists"
        );
        assert_eq!(
            loaded.recipe.compatibility.engine.status,
            VerificationStatus::Inspected
        );
        assert!(!loaded.recipe.authors.is_empty());
        assert!(!loaded.recipe.verification.limitations.is_empty());
        assert_eq!(
            loaded.recipe.screenshots.len(),
            1,
            "`{id}` keeps one sourced Cacowards screenshot"
        );
        assert_eq!(loaded.recipe.screenshots[0].asset, format!("{id}/01.jpg"));
        assert!(!loaded.recipe.screenshots[0].credit.trim().is_empty());
    }
}

#[test]
fn package_and_member_bytes_match_the_inspected_archives() {
    // (member path, size, SHA-256, keep)
    type MemberFixture = (&'static str, u64, &'static str, bool);
    // (recipe id, package size, package SHA-256, idgames path, members)
    type PackageFixture = (
        &'static str,
        u64,
        &'static str,
        &'static str,
        &'static [MemberFixture],
    );
    let expected: [PackageFixture; 5] = [
        (
            "uac-ultra",
            2_309_100,
            "58ebd0fa2fc0ef26fbcd4908a5cb08dbccacc637613db7cb73cec682a863a5a5",
            "levels/doom2/Ports/s-u/uacultra.zip",
            &[
                (
                    "uacultra.wad",
                    7_473_347,
                    "db52da533223c74c01d8dd0eb2d81b39fc6a344249be3a7fe92a34a8a0926798",
                    false,
                ),
                (
                    "uacultra.txt",
                    4_135,
                    "ee2372b5bc80393a9bb28a238d7b03f55ca248bd402c9a068bba2019a395ac8d",
                    true,
                ),
            ],
        ),
        (
            "japanese-community-project",
            13_575_267,
            "2d9d54a4092252031cc038197d6bddc25cd692f4071eec76726cdca4bb70ce63",
            "levels/doom2/Ports/megawads/jpcp.zip",
            &[
                (
                    "JPCP.wad",
                    45_520_073,
                    "b994fbbab8a78f7133fc792679b95571b30ba2491503e35179eb49d77c29c4dd",
                    false,
                ),
                (
                    "JPCP_HUD.wad",
                    33_600,
                    "beff29573bb665b597ad9b0671e6f95b08bba45cdae57304fc4091444d7a2009",
                    false,
                ),
                (
                    "JPCP.txt",
                    13_271,
                    "362a529fc38dc13ce6744472f3cbb1f41e2ee0cd8ed7bbb2975d15e9e1c5a349",
                    true,
                ),
            ],
        ),
        (
            "stardate-20x6",
            4_995_637,
            "4e2a73c60b1eab37bbb35a78706cb557020346f6532c293f45e1ec9c9ac3a0d4",
            "levels/doom2/Ports/s-u/stardate.zip",
            &[
                (
                    "stardate20x6/stardate20x6.wad",
                    14_650_225,
                    "633d06668414de9fd20683b26924808e7b11f65a7e7b4546df68af691019b9ed",
                    false,
                ),
                (
                    "stardate20x6/stardate20x6.txt",
                    7_262,
                    "d7ba39b2a625f4b15fe08676cd56cfe529eaea128497051a522ad09e641fd8a7",
                    true,
                ),
            ],
        ),
        (
            "ozonia",
            17_180_670,
            "f5bbc36ecf8d79652275a98a91672ee6ed6062132ac727069471a28c8f007228",
            "levels/doom2/Ports/megawads/ozonia.zip",
            &[
                (
                    "ozonia.wad",
                    55_559_897,
                    "18c8ae66abafba79ed03d4eaa382d41af8c1d862a6906cba27d071fbe4508fb8",
                    false,
                ),
                (
                    "ozonia.txt",
                    6_595,
                    "fb38dcf0da17da4b25fde66a03413a6454a583b650cb3221a547237946590275",
                    true,
                ),
            ],
        ),
        (
            "abandon",
            46_212_925,
            "9ef49a28f7852b074470abf21b02de895f2cf9a2a6eb8300c65b12c833dfbd27",
            "levels/doom2/Ports/megawads/abandon.zip",
            &[
                (
                    "Abandon.wad",
                    177_444_166,
                    "ce48df1db341d8e3ef02250a0d3852032a8b15813c0734495d70ee489e534bea",
                    false,
                ),
                (
                    "Abandon.txt",
                    4_997,
                    "b2e9c6423db89c132ccd4f9d95074ccac69fa035a76b1e2e6ed211f377e410a4",
                    true,
                ),
            ],
        ),
    ];

    for (id, size, sha256, idgames_path, members) in expected {
        let recipe = recipe(id);
        assert_eq!(recipe.packages.len(), 1, "`{id}` has exactly one package");
        let package = &recipe.packages[0];
        assert_eq!(package.id, "free");
        assert_eq!(package.kind, PackageKind::Zip);
        assert_eq!(package.size, size, "`{id}` package size");
        assert_eq!(package.sha256, sha256, "`{id}` package SHA-256");
        assert_eq!(
            package.idgames_path.as_deref(),
            Some(idgames_path),
            "`{id}` canonical archive path"
        );
        // Primary first: the canonical gamers.org mirror, then reviewed byte-identical mirrors.
        assert!(
            package.urls[0].starts_with("https://www.gamers.org/pub/idgames/"),
            "`{id}` primary URL: {:?}",
            package.urls
        );
        assert_eq!(package.urls.len(), 3, "`{id}` mirror count");

        let declared: Vec<(&str, u64, &str, bool)> = package
            .members
            .iter()
            .map(|member| {
                (
                    member.path.as_str(),
                    member.size,
                    member.sha256.as_str(),
                    member.keep,
                )
            })
            .collect();
        assert_eq!(declared, members.to_vec(), "`{id}` members");
        // Every member is a nonempty, case-exact path inside the archive and credits are kept.
        assert!(
            package
                .members
                .iter()
                .filter(|member| member.keep)
                .all(|member| member.path.ends_with(".txt"))
        );
    }
}

#[test]
fn base_games_and_complevels_follow_the_authors_own_words() {
    for id in ASSIGNED {
        let recipe = recipe(id);
        assert_eq!(recipe.base_game.game, Game::Doom2, "`{id}` base game");
        assert!(recipe.base_game.variants.is_empty());
    }

    // Boom-compatible sets name DSDA-Doom's `boom` alias (complevel 9); the
    // limit-removing megawad names `vanilla`, which is 2 on Doom II.
    let expected = [
        ("uac-ultra", PortFamily::Boom, ComplevelAlias::Boom),
        (
            "japanese-community-project",
            PortFamily::LimitRemoving,
            ComplevelAlias::Vanilla,
        ),
        ("stardate-20x6", PortFamily::Boom, ComplevelAlias::Boom),
        ("ozonia", PortFamily::Boom, ComplevelAlias::Boom),
        ("abandon", PortFamily::Boom, ComplevelAlias::Boom),
    ];
    for (id, family, alias) in expected {
        let recipe = recipe(id);
        assert_eq!(recipe.compatibility.family, family, "`{id}` family");
        assert_eq!(
            recipe.compatibility.complevel,
            Complevel::Alias(alias),
            "`{id}` complevel"
        );
        // D-030: the alias the recipe writes is the one that names its number on Doom II.
        let number = match alias {
            ComplevelAlias::Vanilla => 2,
            ComplevelAlias::Boom => 9,
            ComplevelAlias::Mbf => 11,
            ComplevelAlias::Mbf21 => 21,
        };
        assert_eq!(
            ComplevelAlias::naming(number, recipe.base_game.game),
            Some(alias)
        );
    }

    // The three continuous sets share one -file entry; Ozonia's UMAPINFO supplies its
    // episodes, so none of the four needs a -warp default.
    for id in ASSIGNED {
        assert!(
            recipe(id).compatibility.runtime.warp.is_none(),
            "`{id}` must not prescribe a warp"
        );
    }
}

#[test]
fn the_required_load_order_is_maps_only_and_the_japanese_hud_loads_last() {
    for id in ASSIGNED {
        let recipe = recipe(id);
        // Only `main` is on by default: the DEHACKED patches and all music live inside
        // the map WAD, so exactly one -file entry and no -deh entry are needed.
        assert_eq!(
            recipe.default_components(),
            Vec::<String>::new(),
            "`{id}` default add-ons"
        );
        let order = recipe.load_order(&[]).expect("main-only order is legal");
        let files: Vec<(Option<&str>, Role)> = order
            .files
            .iter()
            .map(|member| (member.member.as_deref(), member.role))
            .collect();
        let maps = recipe
            .packages
            .iter()
            .flat_map(|package| package.members.iter())
            .find(|member| member.id == "maps")
            .expect("main maps member");
        assert_eq!(
            files,
            vec![(Some(maps.id.as_str()), Role::Maps)],
            "`{id}` loads the map WAD first and alone"
        );
        assert!(
            order.deh.is_empty(),
            "`{id}` must not add a separate -deh entry; its patch is embedded"
        );
    }

    let jpcp = recipe("japanese-community-project");
    let order = jpcp
        .load_order(&["japanese-hud"])
        .expect("the Japanese HUD is optional and legal");
    let files: Vec<(Option<&str>, Role)> = order
        .files
        .iter()
        .map(|member| (member.member.as_deref(), member.role))
        .collect();
    assert_eq!(
        files,
        vec![(Some("maps"), Role::Maps), (Some("hud"), Role::Graphics)],
        "the map WAD must load before the optional HUD graphics"
    );
    assert!(order.deh.is_empty());
    let hud = jpcp
        .components
        .iter()
        .find(|component| component.id == "japanese-hud")
        .expect("declared Japanese HUD component");
    assert!(hud.optional && !hud.default, "the HUD stays opt-in");
    assert!(hud.position > 1, "the HUD loads after main");
}

#[test]
fn tags_carry_their_cited_evidence() {
    let expected: [(&str, &[&str], &str); 5] = [
        (
            "uac-ultra",
            &["classics", "cacowards"],
            "https://www.doomworld.com/cacowards/2010/",
        ),
        (
            "japanese-community-project",
            &["classics", "cacowards", "community-projects", "megawad"],
            "https://www.doomworld.com/cacowards/2016/",
        ),
        (
            "stardate-20x6",
            &["classics", "cacowards", "episode", "challenging"],
            "https://www.doomworld.com/cacowards/2013/",
        ),
        (
            "ozonia",
            &["cacowards", "megawad"],
            "https://www.doomworld.com/cacowards/2022/best1/",
        ),
        (
            "abandon",
            &["cacowards", "challenging"],
            "https://www.doomworld.com/cacowards/2020/best1/",
        ),
    ];

    for (id, tags, citation) in expected {
        let recipe = recipe(id);
        assert_eq!(recipe.tags, tags, "`{id}` tags");
        let evidence = std::iter::once(&recipe.verification.notes)
            .chain(&recipe.verification.inspected)
            .any(|text| text.contains(citation));
        assert!(evidence, "`{id}` must cite {citation} in its verification");
    }
}

#[test]
fn release_labels_use_the_normalized_readme_dates() {
    // Numeric readme dates ambiguous between day/month and month/day are resolved by the
    // ZIP member timestamps (docs/research/reference-087-d.md, Method): Stardate 20X6 is
    // 2013-06-02 and Abandon is 2023-05-07, not February 6 / July 5.
    let expected = [
        ("uac-ultra", "v1.2 (2010-03-29)"),
        ("japanese-community-project", "v2.3 final (2016-07-24)"),
        ("stardate-20x6", "2013-06-02"),
        ("ozonia", "2021-12-23"),
        ("abandon", "2023-05-07 update (original 2020 release)"),
    ];
    for (id, release) in expected {
        assert_eq!(recipe(id).release, release, "`{id}` release label");
    }
}

//! Spec 087 worker A and spec 088 worker A: offline identity tests for the
//! curated Scythe II, Plutonia 2 and TNT: Revilution recipes.
//!
//! Every expected byte count and SHA-256 here came from the archives inspected
//! in `docs/research/reference-087-a.md`. The tests only read the embedded
//! catalog and never touch the network (`AGENTS.md`).

use ddl_core::catalog::{
    Catalog, Complevel, ComplevelAlias, Game, PackageKind, PortFamily, Recipe, Role,
    VerificationStatus,
};

const ASSIGNED: [&str; 3] = ["plutonia-2", "scythe-ii", "tnt-revilution"];

fn recipe<'a>(catalog: &'a Catalog, id: &str) -> &'a Recipe {
    catalog
        .recipe(id)
        .unwrap_or_else(|| panic!("{id} must be embedded"))
}

/// A1/A3 for this worker: the three assigned recipes load from the embedded
/// catalog with no skipped entry and no duplicate id.
#[test]
fn assigned_recipes_embed_without_skips() {
    let catalog = Catalog::embedded();
    assert!(
        catalog.skipped.is_empty(),
        "the shipped catalog must be clean: {:?}",
        catalog.skipped
    );

    for id in ASSIGNED {
        let matches = catalog
            .recipes
            .iter()
            .filter(|held| held.recipe.id == id)
            .count();
        assert_eq!(matches, 1, "{id} must appear exactly once");
    }
}

/// R2: package and member identities are frozen from the inspected bytes.
#[test]
fn assigned_packages_and_members_keep_their_inspected_bytes() {
    struct Expected {
        id: &'static str,
        package_size: u64,
        package_sha256: &'static str,
        idgames_path: &'static str,
        urls: [&'static str; 3],
        members: [(&'static str, &'static str, u64, &'static str, bool); 3],
        member_count: usize,
    }

    let expected = [
        Expected {
            id: "scythe-ii",
            package_size: 10_584_383,
            package_sha256: "0d5c1ab9d0152f5f1cf7a0d4ca2f41f5a6ac2986fa6ee850504d1d49e020cd8d",
            idgames_path: "levels/doom2/Ports/megawads/scythe2.zip",
            urls: [
                "https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/scythe2.zip",
                "https://mirrors.lug.mtu.edu/idgames/levels/doom2/Ports/megawads/scythe2.zip",
                "https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/Ports/megawads/scythe2.zip",
            ],
            members: [
                (
                    "maps",
                    "scythe2.wad",
                    31_285_404,
                    "157fa27eed11a872c41263ff24b5677aeb123514a6352f040f7873435ad09ee1",
                    false,
                ),
                (
                    "credits",
                    "scythe2.txt",
                    9_052,
                    "1902b55fb824fde4d9dffe8e15b29c7d8bebec62450636f0a4cd25a721c9835f",
                    true,
                ),
                ("", "", 0, "", false),
            ],
            member_count: 2,
        },
        Expected {
            id: "plutonia-2",
            package_size: 5_375_859,
            package_sha256: "bb925d13119e6d5dfea69a5ffe17d8954cdc3f2da0a6128a58f6ff6a366a0c66",
            idgames_path: "levels/doom2/megawads/pl2.zip",
            urls: [
                "https://www.gamers.org/pub/idgames/levels/doom2/megawads/pl2.zip",
                "https://mirrors.lug.mtu.edu/idgames/levels/doom2/megawads/pl2.zip",
                "https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/megawads/pl2.zip",
            ],
            members: [
                (
                    "maps",
                    "PL2.WAD",
                    18_131_932,
                    "9495a54ecfd5ab2dde4fed22eb4a1dd3e70296899df835f5aa74d8a8cc6f253e",
                    false,
                ),
                (
                    "credits",
                    "PL2.TXT",
                    3_915,
                    "c19728b15488d00ca1f86973ad4508506c99a996617e7c66b1ac164a298f218d",
                    true,
                ),
                (
                    "manual",
                    "PL2INFO.TXT",
                    66_591,
                    "e79112f2be34943f249a68e960bb6cf577253d670e1d7d28773b5b423ba4028c",
                    true,
                ),
            ],
            member_count: 3,
        },
        Expected {
            id: "tnt-revilution",
            package_size: 8_619_240,
            package_sha256: "aa64d4f3e713e4c7349c86d02731428c19e1f403f87e7d589d22e663a446cba8",
            idgames_path: "levels/doom2/megawads/tntr.zip",
            urls: [
                "https://www.gamers.org/pub/idgames/levels/doom2/megawads/tntr.zip",
                "https://mirrors.lug.mtu.edu/idgames/levels/doom2/megawads/tntr.zip",
                "https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/megawads/tntr.zip",
            ],
            members: [
                (
                    "maps",
                    "TNTR.wad",
                    28_776_797,
                    "3e842f3172fac2d2008cbf0dfe40b89af2ea34ece07c0df75f8992359a79af86",
                    false,
                ),
                (
                    "credits",
                    "TNTR.txt",
                    10_182,
                    "52f724f2e9562a7d68bb3635fff758bf1ee5e1d268c151a59959be5c06fae6a3",
                    true,
                ),
                ("", "", 0, "", false),
            ],
            member_count: 2,
        },
    ];

    let catalog = Catalog::embedded();
    for expected in expected {
        let recipe = recipe(&catalog, expected.id);
        assert_eq!(
            recipe.packages.len(),
            1,
            "{} uses one canonical package",
            expected.id
        );
        let package = &recipe.packages[0];
        assert_eq!(package.id, "free");
        assert_eq!(package.kind, PackageKind::Zip);
        assert_eq!(package.size, expected.package_size, "{}", expected.id);
        assert_eq!(package.sha256, expected.package_sha256, "{}", expected.id);
        assert_eq!(
            package.idgames_path.as_deref(),
            Some(expected.idgames_path),
            "{}",
            expected.id
        );
        assert_eq!(
            package.urls, expected.urls,
            "{} lists only verified byte-identical mirrors",
            expected.id
        );
        assert_eq!(
            package.members.len(),
            expected.member_count,
            "{}",
            expected.id
        );
        for (member, (id, path, size, sha256, keep)) in package.members.iter().zip(expected.members)
        {
            assert_eq!(member.id, id, "{} member id", expected.id);
            assert_eq!(member.path, path, "{} case-exact member path", expected.id);
            assert_eq!(member.size, size, "{} member size", expected.id);
            assert_eq!(member.sha256, sha256, "{} member SHA-256", expected.id);
            assert_eq!(member.keep, keep, "{} keep flag", expected.id);
        }
        assert!(
            package
                .members
                .iter()
                .all(|m| !m.path.to_ascii_lowercase().ends_with(".deh")),
            "{} loads DEHACKED only from the WAD's embedded lump",
            expected.id
        );
    }
}

/// R1/R2: the actual base game and complevel come from the release text, not
/// the /idgames directory. Plutonia 2 and TNT: Revilution must not be inferred
/// as Doom II.
#[test]
fn assigned_recipes_pin_base_game_and_complevel() {
    let catalog = Catalog::embedded();
    let cases = [
        ("scythe-ii", Game::Doom2, PortFamily::LimitRemoving, 2),
        ("plutonia-2", Game::Plutonia, PortFamily::Vanilla, 4),
        ("tnt-revilution", Game::Tnt, PortFamily::Vanilla, 4),
    ];

    for (id, game, family, resolved_complevel) in cases {
        let recipe = recipe(&catalog, id);
        assert_eq!(recipe.base_game.game, game, "{id} base game");
        assert!(
            recipe.base_game.variants.is_empty(),
            "{id} targets any variant of its game"
        );
        assert_eq!(recipe.compatibility.family, family, "{id} port family");
        assert_eq!(
            recipe.compatibility.complevel,
            Complevel::Alias(ComplevelAlias::Vanilla),
            "{id} names DSDA-Doom's vanilla alias"
        );
        assert_eq!(
            game.vanilla_complevel(),
            resolved_complevel,
            "{id} resolves the alias on this IWAD"
        );
        assert_eq!(
            recipe.compatibility.engine.range, "=0.30.1",
            "{id} pins the inspected engine"
        );
        assert_eq!(
            recipe.compatibility.engine.status,
            VerificationStatus::Inspected,
            "{id} does not overclaim engine support"
        );
        assert_eq!(recipe.verification.status, VerificationStatus::Inspected);
        assert!(!recipe.verification.inspected.is_empty(), "{id}");
        assert!(!recipe.verification.limitations.is_empty(), "{id}");
    }
}

/// R2: main is the only component and loads exactly one WAD, in order; the
/// embedded DEHACKED lump is never a separate `dehacked` member.
#[test]
fn assigned_recipes_load_one_main_wad_in_order() {
    let catalog = Catalog::embedded();
    for id in ASSIGNED {
        let recipe = recipe(&catalog, id);
        assert_eq!(recipe.components.len(), 1, "{id}");
        let main = &recipe.components[0];
        assert_eq!(main.id, "main");
        assert!(!main.optional && main.default, "{id} main is required");
        assert_eq!(main.position, 1, "{id}");
        assert_eq!(main.members.len(), 1, "{id}");
        assert_eq!(main.members[0].package.as_deref(), Some("free"));
        assert_eq!(main.members[0].member.as_deref(), Some("maps"));
        assert_eq!(main.members[0].role, Role::Maps);

        let order = recipe
            .load_order(&[])
            .unwrap_or_else(|error| panic!("{id} default order: {error}"));
        assert_eq!(order.files.len(), 1, "{id}");
        assert_eq!(order.files[0].role, Role::Maps, "{id}");
        assert_eq!(
            order.files[0].package.as_deref(),
            Some("free"),
            "{id} package order"
        );
        assert_eq!(order.files[0].member.as_deref(), Some("maps"), "{id}");
        assert!(
            order.deh.is_empty(),
            "{id} must not load an external DEHACKED patch twice"
        );
    }
}

/// R3: tags are evidence-backed memberships with the cited Cacowards year.
#[test]
fn assigned_recipes_keep_their_reviewed_tags() {
    let catalog = Catalog::embedded();
    let cases = [
        (
            "scythe-ii",
            ["cacowards", "challenging", "classics", "megawad"],
            "cacowards/2005",
        ),
        (
            "plutonia-2",
            ["cacowards", "classics", "community-projects", "megawad"],
            "cacowards/2009",
        ),
        (
            "tnt-revilution",
            ["cacowards", "classics", "community-projects", "megawad"],
            "cacowards/2017",
        ),
    ];

    for (id, expected, year) in cases {
        let recipe = recipe(&catalog, id);
        let mut tags = recipe.tags.clone();
        tags.sort_unstable();
        assert_eq!(tags, expected, "{id} tags");
        assert!(
            recipe.verification.notes.contains(year),
            "{id} cites its Cacowards page ({year})"
        );
        for tag in &recipe.tags {
            assert!(
                catalog.tags.get(tag).is_some(),
                "{id} uses reviewed vocabulary only"
            );
        }
    }
}

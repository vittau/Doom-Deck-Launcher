//! Worker B coverage for specs 087, 088 and 089: Doom the Way id Did, No End in
//! Sight, Doom II the Way id Did, Deathless, Sunder and REKKR.
//!
//! The tests are offline. They freeze the release identities inspected from the
//! author/mirror bytes (archive and member sizes and SHA-256), the required IWAD
//! and complevel, the declared file order and the evidence-backed tags, and they
//! prove that the same recipes load from the embedded catalog without skips.

use std::path::PathBuf;

use ddl_core::catalog::{
    Catalog, Complevel, ComplevelAlias, Game, PortFamily, Recipe, Role, VerificationStatus,
    lint_paths,
};

const CATALOG: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../catalog");

struct Member {
    id: &'static str,
    path: &'static str,
    size: u64,
    sha256: &'static str,
    keep: bool,
}

struct Expected {
    id: &'static str,
    title: &'static str,
    game: Game,
    variants: &'static [&'static str],
    family: PortFamily,
    complevel: ComplevelAlias,
    tags: &'static [&'static str],
    package_size: u64,
    package_sha256: &'static str,
    package_urls: usize,
    idgames_path: Option<&'static str>,
    members: &'static [Member],
}

const ASSIGNED: [Expected; 6] = [
    Expected {
        id: "doom-the-way-id-did",
        title: "Doom the Way id Did",
        game: Game::Doom,
        variants: &[
            "doom-1.9",
            "doom-1.9ud",
            "doom-1.9ud-kex-2024-08-06",
            "doom-1.9ud-kex-2024-10-01",
        ],
        family: PortFamily::Vanilla,
        complevel: ComplevelAlias::Vanilla,
        tags: &["classics", "cacowards", "community-projects"],
        package_size: 2_005_606,
        package_sha256: "122b50713026a05423776e5c1330770f74a371d4236c63497f0f7f64bc9a62cb",
        package_urls: 2,
        idgames_path: Some("levels/doom/megawads/dtwid.zip"),
        members: &[
            Member {
                id: "maps",
                path: "DTWID.wad",
                size: 5_073_423,
                sha256: "5fcec0759a85aa3860ade5cb745cd2930c825eecaea3d823dc9f63840c5ceab5",
                keep: false,
            },
            Member {
                id: "readme",
                path: "DTWID.txt",
                size: 8_448,
                sha256: "d58e57291240e31a6df14d2323e15a70c18bc7d2376669fcbec22440e1849dee",
                keep: true,
            },
        ],
    },
    Expected {
        id: "no-end-in-sight",
        title: "No End in Sight",
        game: Game::Doom,
        variants: &[
            "doom-1.9ud",
            "doom-1.9ud-kex-2024-08-06",
            "doom-1.9ud-kex-2024-10-01",
        ],
        family: PortFamily::LimitRemoving,
        complevel: ComplevelAlias::Vanilla,
        tags: &["classics", "cacowards", "megawad"],
        package_size: 6_431_036,
        package_sha256: "2cbc625a7bb5f0796046fc42b478f5b375060899366d652432375f26f50da66a",
        package_urls: 2,
        idgames_path: Some("levels/doom/Ports/megawads/neis.zip"),
        members: &[
            Member {
                id: "maps",
                path: "NEIS.wad",
                size: 19_830_307,
                sha256: "f4a7f69f07d72ac5fa284e71c8fb66fb9d38256bb4000f183c46bb92fe35e069",
                keep: false,
            },
            Member {
                id: "readme",
                path: "NEIS.txt",
                size: 6_847,
                sha256: "683ebdcedd268af10524e29b55ad4169fd85aaef86cd5e70838591b57f2a3e82",
                keep: true,
            },
        ],
    },
    Expected {
        id: "doom-ii-the-way-id-did",
        title: "Doom II the Way id Did",
        game: Game::Doom2,
        variants: &[
            "doom2-1.9",
            "doom2-1.9-kex-2024-08-06",
            "doom2-1.9-kex-2024-10-01",
        ],
        family: PortFamily::Vanilla,
        complevel: ComplevelAlias::Vanilla,
        tags: &["classics", "cacowards", "community-projects", "megawad"],
        package_size: 1_964_344,
        package_sha256: "e5ef240143640e537ab5b47239967ec9e70ba84a314ee40d3a2acb66757aed32",
        package_urls: 2,
        idgames_path: Some("levels/doom2/megawads/d2twid.zip"),
        members: &[
            Member {
                id: "maps",
                path: "D2TWID.wad",
                size: 6_753_456,
                sha256: "128a613717fee36b08b72ff5e54cf0e689136b25ab6fe7460f6ab634226f0e7f",
                keep: false,
            },
            Member {
                id: "readme",
                path: "D2TWID.txt",
                size: 5_938,
                sha256: "019df270d2b644cacec330729bcebfe322396d40b0e81fd005ef0010f3481c77",
                keep: true,
            },
        ],
    },
    Expected {
        id: "deathless",
        title: "Deathless",
        game: Game::Doom,
        variants: &[
            "doom-1.9ud",
            "doom-1.9ud-kex-2024-08-06",
            "doom-1.9ud-kex-2024-10-01",
        ],
        family: PortFamily::LimitRemoving,
        complevel: ComplevelAlias::Vanilla,
        tags: &["classics", "megawad"],
        package_size: 1_743_823,
        package_sha256: "47481ee568d7fada97ffb15ffd308dd3d9085f974ca3687e154c6b9f06e7c9ee",
        package_urls: 2,
        idgames_path: Some("levels/doom/Ports/megawads/deathless.zip"),
        members: &[
            Member {
                id: "maps",
                path: "deathless.wad",
                size: 5_212_998,
                sha256: "295efe40a40e5f93a913b762e1c6052220dab3ed8d409a0e758c1abac49600c3",
                keep: false,
            },
            Member {
                id: "readme",
                path: "deathless1.1.txt",
                size: 6_950,
                sha256: "e0540ba517e70a9b29abaab7b4fe39c315c4fea7b8bbe6ffc1ca0584db897495",
                keep: true,
            },
        ],
    },
    Expected {
        id: "sunder",
        title: "Sunder",
        game: Game::Doom2,
        variants: &[],
        family: PortFamily::Boom,
        complevel: ComplevelAlias::Boom,
        tags: &["classics", "challenging"],
        package_size: 49_972_580,
        package_sha256: "c11bb7bb6f601b0badbed58f23d04e8b8f352795ebee66074645329d3f17fd2e",
        package_urls: 1,
        idgames_path: None,
        members: &[
            Member {
                id: "maps",
                path: "Sunder 2407.wad",
                size: 167_039_763,
                sha256: "b58787ea6390da29a5f2c2fd3488065a4979a30dc684d67e5a6d7ae26834005c",
                keep: false,
            },
            Member {
                id: "readme",
                path: "SunderReadMeV2.TXT",
                size: 34_421,
                sha256: "4c67a6f5c798840953dc2d20344e3760627016f926c26d2f1cfbc5e9948b0e40",
                keep: true,
            },
        ],
    },
    Expected {
        id: "rekkr",
        title: "REKKR",
        game: Game::Doom,
        variants: &[
            "doom-1.9ud",
            "doom-1.9ud-kex-2024-08-06",
            "doom-1.9ud-kex-2024-10-01",
        ],
        family: PortFamily::Vanilla,
        complevel: ComplevelAlias::Vanilla,
        tags: &["classics", "cacowards", "community-projects", "megawad"],
        package_size: 14_152_590,
        package_sha256: "c28a32699a58c976c135f8d5bcc270f2801a88019fc8a1d7b4805a54cb0b8429",
        package_urls: 2,
        idgames_path: Some("levels/doom/megawads/rekkr.zip"),
        members: &[
            Member {
                id: "maps",
                path: "REKKR.wad",
                size: 41_960_202,
                sha256: "fadf0b0ddb492c56e19ba5f56c8ac2e77413f287e8436cbcab31f2d41d7e9b0f",
                keep: false,
            },
            Member {
                id: "readme",
                path: "REKKR.txt",
                size: 3_188,
                sha256: "ad736135c000c80d348cb1dc5b12eed69980dd0e45dfa10207ebf77b7c377cae",
                keep: true,
            },
            Member {
                id: "credits",
                path: "REK_CREDS.txt",
                size: 4_848,
                sha256: "e8e8b89970f957996cf44013c0df9939c6f1c130dfe1bf8aeb2579d6d0b640e8",
                keep: true,
            },
        ],
    },
];

fn embedded() -> Catalog {
    Catalog::embedded()
}

fn recipe<'a>(catalog: &'a Catalog, id: &str) -> &'a Recipe {
    catalog
        .recipes
        .iter()
        .find(|loaded| loaded.recipe.id == id)
        .map(|loaded| &loaded.recipe)
        .unwrap_or_else(|| panic!("{id} is embedded and valid"))
}

#[test]
fn assigned_recipes_are_embedded_without_skips() {
    let catalog = embedded();
    assert!(
        catalog.skipped.is_empty(),
        "embedded catalog must not skip recipes: {:?}",
        catalog.skipped
    );
    for expected in &ASSIGNED {
        let file = format!("{}.toml", expected.id);
        let loaded = catalog
            .recipes
            .iter()
            .find(|loaded| loaded.recipe.id == expected.id)
            .unwrap_or_else(|| panic!("{file} must be embedded"));
        assert_eq!(loaded.file, file);
        assert_eq!(loaded.sha256.len(), 64, "recipe identity is pinned");
        assert_eq!(
            loaded.recipe.verification.status,
            VerificationStatus::Inspected
        );
    }
}

#[test]
fn assigned_recipes_pin_sources_and_members() {
    let catalog = embedded();
    for expected in &ASSIGNED {
        let r = recipe(&catalog, expected.id);
        assert_eq!(r.schema_version, 1);
        assert_eq!(r.title, expected.title);
        assert!(!r.authors.is_empty(), "{} credits its authors", expected.id);
        assert!(!r.permission.trim().is_empty());
        assert!(r.page.starts_with("https://"));

        assert_eq!(r.base_game.game, expected.game);
        let variants: Vec<&str> = r.base_game.variants.iter().map(String::as_str).collect();
        assert_eq!(
            variants, expected.variants,
            "{} accepted IWADs",
            expected.id
        );

        assert_eq!(r.compatibility.family, expected.family);
        assert_eq!(
            r.compatibility.complevel,
            Complevel::Alias(expected.complevel),
            "{} complevel alias",
            expected.id
        );
        assert_eq!(r.compatibility.engine.range, "=0.30.1");
        assert_eq!(r.compatibility.engine.status, VerificationStatus::Inspected);

        assert_eq!(r.packages.len(), 1, "{} has one package", expected.id);
        let package = &r.packages[0];
        assert_eq!(package.size, expected.package_size);
        assert_eq!(package.sha256, expected.package_sha256);
        assert_eq!(package.urls.len(), expected.package_urls);
        assert_eq!(package.idgames_path.as_deref(), expected.idgames_path);
        for url in &package.urls {
            assert!(url.starts_with("https://"), "{url} must be HTTPS");
        }

        let members: Vec<(&str, &str, u64, &str, bool)> = package
            .members
            .iter()
            .map(|m| {
                (
                    m.id.as_str(),
                    m.path.as_str(),
                    m.size,
                    m.sha256.as_str(),
                    m.keep,
                )
            })
            .collect();
        let want: Vec<(&str, &str, u64, &str, bool)> = expected
            .members
            .iter()
            .map(|m| (m.id, m.path, m.size, m.sha256, m.keep))
            .collect();
        assert_eq!(members, want, "{} member identity and order", expected.id);
        for member in &package.members {
            assert_eq!(member.sha256.len(), 64);
            // The declared path must be the byte-exact archive name (including
            // Sunder's space), which the table above spells out.
            assert!(
                expected.members.iter().any(|m| m.path == member.path),
                "{} member path {:?} is case-exact",
                expected.id,
                member.path
            );
        }
    }
}

#[test]
fn assigned_recipes_declare_one_required_maps_component() {
    let catalog = embedded();
    for expected in &ASSIGNED {
        let r = recipe(&catalog, expected.id);
        assert_eq!(r.components.len(), 1, "{} has one component", expected.id);
        let main = &r.components[0];
        assert_eq!(main.id, "main");
        assert!(!main.optional);
        assert!(main.default);
        assert_eq!(main.position, 1);
        assert_eq!(main.replaces, None);
        assert_eq!(main.exclusive_group, None);
        assert!(!main.user_supplied);
        assert_eq!(
            main.members.len(),
            1,
            "{} loads only its map WAD",
            expected.id
        );
        let member = &main.members[0];
        assert_eq!(member.role, Role::Maps);
        assert_eq!(member.package.as_deref(), Some(r.packages[0].id.as_str()));
        assert_eq!(member.member.as_deref(), Some("maps"));

        // Every inspected archive supplies its level names (and, for Sunder and
        // Deathless, monster changes) through a WAD-embedded DEHACKED lump, which
        // DSDA-Doom applies automatically. Loading an older external .deh on top
        // would duplicate or override it, so no recipe declares one.
        assert!(
            r.components
                .iter()
                .flat_map(|component| &component.members)
                .all(|member| member.role != Role::Dehacked),
            "{} must not add an external DeHackEd load",
            expected.id
        );
        let order = r.load_order(&[]).expect("default selection is valid");
        assert_eq!(order.files.len(), 1);
        assert_eq!(order.deh.len(), 0);
        assert_eq!(r.default_components(), Vec::<String>::new());
    }
}

#[test]
fn assigned_recipes_keep_only_evidence_backed_tags() {
    let catalog = embedded();
    for expected in &ASSIGNED {
        let r = recipe(&catalog, expected.id);
        assert_eq!(r.tags, expected.tags, "{} tags", expected.id);
        for tag in &r.tags {
            assert!(
                catalog.tags.get(tag).is_some(),
                "{} tag {tag} exists in tags.toml",
                expected.id
            );
        }
        // Doom the Way id Did is 27 maps: it must not claim the 30-map megawad
        // tag, and neither it nor any multi-episode release claims `episode`.
        if expected.id == "doom-the-way-id-did" {
            assert!(!r.tags.iter().any(|tag| tag == "megawad"));
            assert!(!r.tags.iter().any(|tag| tag == "episode"));
        }
        // Sunder's pinned 22-map build is not a megawad.
        if expected.id == "sunder" {
            assert!(!r.tags.iter().any(|tag| tag == "megawad"));
        }
        assert!(!r.verification.limitations.is_empty());
        assert!(!r.verification.notes.trim().is_empty());
    }
}

#[test]
fn assigned_recipes_pass_offline_lint_including_images() {
    let catalog = embedded();
    let hosts = PathBuf::from(CATALOG).join("hosts.toml");
    for expected in &ASSIGNED {
        let path = PathBuf::from(CATALOG).join(format!("{}.toml", expected.id));
        let diagnostics = lint_paths(&[path], &hosts, None);
        assert!(diagnostics.is_empty(), "{:?}", diagnostics);
    }
    // Doom the Way id Did references two committed screenshots, so image lint ran.
    assert_eq!(recipe(&catalog, "doom-the-way-id-did").screenshots.len(), 2);
    for id in [
        "no-end-in-sight",
        "doom-ii-the-way-id-did",
        "deathless",
        "sunder",
        "rekkr",
    ] {
        assert!(recipe(&catalog, id).screenshots.is_empty());
    }
}

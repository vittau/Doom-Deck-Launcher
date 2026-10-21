use super::*;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

const CATALOG: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../catalog");
const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/catalog");
fn hosts() -> Hosts {
    toml::from_str(&fs::read_to_string(format!("{CATALOG}/hosts.toml")).unwrap()).unwrap()
}
fn recipe(id: &str) -> Recipe {
    toml::from_str(&fs::read_to_string(format!("{CATALOG}/{id}.toml")).unwrap()).unwrap()
}
fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn reference_recipes_round_trip_and_lint_offline() {
    for id in [
        "alien-vendetta",
        "ancient-aliens",
        "auger-zenith",
        "back-to-saturn-x-e1",
        "back-to-saturn-x-e2",
        "eviternity",
        "eviternity-ii",
        "going-down",
        "memento-mori",
        "memento-mori-ii",
        "scythe",
        "sigil",
        "sigil-ii",
        "sunlust",
        "valiant",
    ] {
        let r = recipe(id);
        assert_eq!(
            toml::from_str::<Recipe>(&toml::to_string_pretty(&r).unwrap()).unwrap(),
            r
        );
        assert_eq!(
            validate(&r, Path::new(&format!("{id}.toml")), &hosts()),
            vec![]
        );
        assert_eq!(r.verification.status, VerificationStatus::Inspected);
    }
    assert!(
        lint_paths(
            &[CATALOG.into()],
            &Path::new(CATALOG).join("hosts.toml"),
            None
        )
        .is_empty()
    );
}

#[test]
fn recipes_with_a_status_bar_default_it_on() {
    for id in [
        "ancient-aliens",
        "back-to-saturn-x-e1",
        "back-to-saturn-x-e2",
        "eviternity",
        "valiant",
        "going-down",
        "scythe",
        "sigil",
        "sigil-ii",
    ] {
        let r = recipe(id);
        assert_eq!(
            r.default_components(),
            vec!["wide-status-bar".to_owned()],
            "{id}"
        );
        // The default combination is legal and loads the bar last.
        let order = r.load_order(&["wide-status-bar"]).unwrap();
        assert_eq!(
            order.files.last().unwrap().member.as_deref(),
            Some("statusbar")
        );
    }
    for id in ["alien-vendetta", "auger-zenith", "eviternity-ii", "sunlust"] {
        assert_eq!(
            recipe(id).default_components(),
            Vec::<String>::new(),
            "{id}"
        );
    }
}

#[test]
fn reference_combinations_have_explicit_order_and_replacements() {
    let auger = recipe("auger-zenith");
    assert_eq!(auger.load_order(&[]).unwrap().files.len(), 1);
    for addon in ["alternate-blaster", "deckard-blaster"] {
        let order = auger.load_order(&[addon]).unwrap();
        assert_eq!(order.files.len(), 2);
        assert_eq!(order.files[0].member.as_deref(), Some("maps"));
        assert!(order.deh.is_empty()); // WAD-embedded DEHACKED is automatic.
    }
    assert!(
        auger
            .load_order(&["alternate-blaster", "deckard-blaster"])
            .unwrap_err()
            .contains("group")
    );
    for id in ["sigil", "sigil-ii"] {
        let r = recipe(id);
        let order = r.load_order(&["soundtrack"]).unwrap();
        assert_eq!(order.files.len(), 1);
        assert!(order.files[0].package.is_none());
        assert_eq!(r.base_game.variants, ["doom-1.9ud"]);
    }
    let sigil = recipe("sigil");
    assert_eq!(
        sigil.load_order(&["compat"]).unwrap().files[0]
            .member
            .as_deref(),
        Some("compat")
    );
    assert!(sigil.load_order(&["compat", "soundtrack"]).is_err());
    assert!(sigil.load_order(&["unknown"]).is_err());
    assert!(sigil.load_order(&["compat", "compat"]).is_err());
}

#[test]
fn purchased_sigil_editions_are_known_complete_replacements() {
    // Read-only identities from the owner's supplied media (spec 068); these
    // are metadata only and stand in for the paid binary content.
    const SIGIL_REG_SHA256: &str =
        "46fcc22fba47fff3872d9d294ee07bdbab4b0ec9feaab277f83836f5aed80bd0";
    const SIGIL_II_MP3_SHA256: &str =
        "8a803f5425d4bf31ddfe537b659d794acdff0631316f51b9ed45311b5ed059dc";
    const SIGIL_SHREDS_COMPAT_SHA256: &str =
        "e5592bc76951a63d8647c5dc7004378ca4d7fcecfda3ab73acc6c220f1193ab7";

    let sigil = recipe("sigil");
    let soundtrack = sigil
        .components
        .iter()
        .find(|component| component.id == "soundtrack")
        .unwrap();
    assert!(soundtrack.user_supplied);
    assert_eq!(soundtrack.known_sha256, [SIGIL_REG_SHA256]);
    assert_eq!(soundtrack.replaces.as_deref(), Some("main"));
    assert_eq!(soundtrack.exclusive_group.as_deref(), Some("edition"));
    assert!(
        soundtrack
            .obtain
            .as_deref()
            .unwrap()
            .contains("SIGIL_V1_23_REG.wad")
    );

    // The paid REG WAD is a complete replacement: exactly one imported file in
    // the main position, not a music add-on after the free WAD.
    let order = sigil.load_order(&["soundtrack"]).unwrap();
    assert_eq!(order.files.len(), 1);
    assert!(order.files[0].package.is_none());
    assert!(order.files[0].member.is_none());
    assert_eq!(order.files[0].role, Role::Maps);

    // The free MIDI WAD has a different identity, so importing it for the paid
    // component is an unrecognized mismatch under D-014.
    let free_maps = sigil
        .packages
        .iter()
        .find(|package| package.id == "free")
        .unwrap()
        .members
        .iter()
        .find(|member| member.id == "maps")
        .unwrap();
    assert!(
        !soundtrack
            .known_sha256
            .iter()
            .any(|known| known == &free_maps.sha256)
    );

    // SIGIL_SHREDS_COMPAT.wad is a music-only paid add-on that requires the
    // free E3 map component. It is never a native E5 replacement.
    let compat_soundtrack = sigil
        .components
        .iter()
        .find(|component| component.id == "compat-soundtrack")
        .unwrap();
    assert!(compat_soundtrack.user_supplied);
    assert_eq!(compat_soundtrack.known_sha256, [SIGIL_SHREDS_COMPAT_SHA256]);
    assert_eq!(compat_soundtrack.replaces, None);
    assert_eq!(compat_soundtrack.requires.as_deref(), Some("compat"));
    assert!(
        compat_soundtrack
            .obtain
            .as_deref()
            .unwrap()
            .contains("SIGIL_SHREDS_COMPAT.wad")
    );
    let order = sigil.load_order(&["compat", "compat-soundtrack"]).unwrap();
    assert_eq!(order.files.len(), 2);
    assert_eq!(order.files[0].member.as_deref(), Some("compat"));
    assert_eq!(order.files[1].role, Role::Music);

    // The dependent music never loads alone or with a native replacement.
    assert!(sigil.load_order(&["compat-soundtrack"]).is_err());
    assert!(
        sigil
            .load_order(&["soundtrack", "compat-soundtrack"])
            .is_err()
    );

    let sigil_ii = recipe("sigil-ii");
    let soundtrack_ii = sigil_ii
        .components
        .iter()
        .find(|component| component.id == "soundtrack")
        .unwrap();
    assert!(soundtrack_ii.user_supplied);
    assert_eq!(soundtrack_ii.known_sha256, [SIGIL_II_MP3_SHA256]);
    assert_eq!(soundtrack_ii.replaces.as_deref(), Some("main"));
    assert_eq!(soundtrack_ii.exclusive_group.as_deref(), Some("edition"));
    assert!(
        soundtrack_ii
            .obtain
            .as_deref()
            .unwrap()
            .contains("SIGIL_II_MP3_V1_0.WAD")
    );

    let order = sigil_ii.load_order(&["soundtrack"]).unwrap();
    assert_eq!(order.files.len(), 1);
    assert!(order.files[0].package.is_none());
    assert!(order.files[0].member.is_none());
    assert_eq!(order.files[0].role, Role::Maps);

    let free_maps = sigil_ii
        .packages
        .iter()
        .find(|package| package.id == "free")
        .unwrap()
        .members
        .iter()
        .find(|member| member.id == "maps")
        .unwrap();
    assert!(
        !soundtrack_ii
            .known_sha256
            .iter()
            .any(|known| known == &free_maps.sha256)
    );
}

#[test]
fn compatibility_music_activates_maps_and_switching_keeps_the_selection_legal() {
    let sigil = recipe("sigil");
    let apply = |selected: &[String], id: &str, enabled: bool| {
        sigil.apply_component_choice(selected, id, enabled).unwrap()
    };
    let mut selected = apply(&[], "compat-soundtrack", true);
    assert_eq!(selected, ["compat", "compat-soundtrack"]);
    assert_eq!(
        sigil
            .load_order(&["compat", "compat-soundtrack"])
            .unwrap()
            .files
            .len(),
        2
    );

    // Switching to the native paid replacement removes the COMPAT maps and
    // their dependent music instead of leaving an orphan.
    selected = apply(&selected, "soundtrack", true);
    assert_eq!(selected, ["soundtrack"]);

    // Choosing the free/native path also clears the COMPAT pair.
    selected = apply(&selected, "compat", true);
    assert_eq!(selected, ["compat"]);
    selected = apply(&selected, "compat-soundtrack", true);
    assert_eq!(selected, ["compat", "compat-soundtrack"]);
    selected = apply(&selected, "compat", false);
    assert!(selected.is_empty());

    // Explicit CLI-style selections reject contradictions independently of order.
    for requested in [
        vec!["compat".to_owned(), "soundtrack".to_owned()],
        vec!["soundtrack".to_owned(), "compat".to_owned()],
        vec!["soundtrack".to_owned(), "compat-soundtrack".to_owned()],
        vec!["compat-soundtrack".to_owned(), "soundtrack".to_owned()],
    ] {
        let error = sigil.explicit_selection(&requested).unwrap_err();
        assert!(
            error.contains("replacement") || error.contains("requires"),
            "{requested:?}: {error}"
        );
    }
    assert_eq!(
        sigil
            .explicit_selection(&["compat-soundtrack".to_owned()])
            .unwrap(),
        ["compat", "compat-soundtrack"]
    );

    let mut profile = crate::profiles::Profile::new("p1", "Play", "sigil");
    profile.components = vec!["compat-soundtrack".to_owned()];
    assert!(profile.validate(&sigil).is_err());
    profile.components = vec!["compat".to_owned(), "compat-soundtrack".to_owned()];
    assert!(profile.validate(&sigil).is_ok());

    // Runtime validation rejects a bad hand-built recipe even when the broken
    // dependency is not selected.
    let mut broken = sigil.clone();
    broken.components[1].requires = Some("compat-soundtrack".to_owned());
    assert!(broken.active_components(&[]).is_err());
}

#[test]
fn every_semantic_rule_rejects_a_bad_recipe() {
    type Mutation = fn(&mut Recipe);
    let cases: &[(&str, Mutation)] = &[
        ("schema_version", |r| r.schema_version = 2),
        ("invalid recipe id", |r| r.id = "SIGIL".into()),
        ("file name", |r| r.id = "different".into()),
        ("identity/permission", |r| r.permission.clear()),
        ("invalid release label", |r| {
            r.release = "Final/version".into()
        }),
        ("invalid release label", |r| r.release = r"WIP\build".into()),
        ("missing authors", |r| r.authors.clear()),
        ("canonical page", |r| {
            r.page = "http://romero.com/sigil".into()
        }),
        ("unsafe image", |r| {
            r.images.push(Image {
                path: "../x.png".into(),
                permission: "yes".into(),
            })
        }),
        ("image permission", |r| {
            r.images.push(Image {
                path: "x.png".into(),
                permission: "".into(),
            })
        }),
        ("IWAD variant", |r| {
            r.base_game.variants = vec!["bad/name".into()]
        }),
        ("IWAD variant", |r| {
            r.base_game.variants.push(r.base_game.variants[0].clone())
        }),
        ("no packages", |r| r.packages.clear()),
        ("duplicate package", |r| {
            r.packages.push(r.packages[0].clone())
        }),
        ("package page", |r| {
            r.packages[0].page = "https://unlisted.invalid".into()
        }),
        ("download URLs", |r| r.packages[0].urls.clear()),
        ("download URLs", |r| {
            r.packages[0].urls = vec!["https://www.doomworld.com/file.zip".into()]
        }),
        ("package size", |r| r.packages[0].size = 0),
        ("package size", |r| r.packages[0].size = u64::MAX),
        ("package SHA", |r| r.packages[0].sha256 = "0".repeat(63)),
        ("idgames path", |r| {
            r.packages[0].idgames_path = Some("../x".into())
        }),
        ("no named members", |r| r.packages[0].members.clear()),
        ("duplicate member id", |r| {
            let m = r.packages[0].members[0].clone();
            r.packages[0].members.push(m);
        }),
        ("duplicate member path", |r| {
            r.packages[0].members[0].path = "../evil.wad".into()
        }),
        ("member size", |r| r.packages[0].members[0].size = 0),
        ("member SHA", |r| {
            r.packages[0].members[0].sha256 = "g".repeat(64)
        }),
        ("non-ZIP", |r| {
            r.packages[0].members[0].path = "x.wad!/x.wad".into()
        }),
        ("bare WAD", |r| r.packages[0].kind = PackageKind::Wad),
        ("missing main", |r| r.components[0].id = "other".into()),
        ("duplicate component", |r| {
            r.components.push(r.components[0].clone())
        }),
        ("component text", |r| r.components[0].label.clear()),
        ("required/optional", |r| r.components[0].optional = true),
        ("load position", |r| r.components[0].position = 0),
        ("replacement target", |r| {
            r.components[1].replaces = Some("other".into())
        }),
        ("exclusive group", |r| {
            r.components[1].exclusive_group = Some("?".into())
        }),
        ("invalid component dependency", |r| {
            r.components[3].requires = Some("?".into())
        }),
        ("unknown or self-referential component dependency", |r| {
            r.components[3].requires = Some("missing".into())
        }),
        ("unknown or self-referential component dependency", |r| {
            r.components[3].requires = Some("compat-soundtrack".into())
        }),
        ("unsupported dependency chain", |r| {
            r.components[1].requires = Some("compat-soundtrack".into())
        }),
        ("dependency loads at the wrong position", |r| {
            r.components[3].position = 1
        }),
        ("no members", |r| r.components[0].members.clear()),
        ("user-supplied source", |r| r.components[2].obtain = None),
        ("known SHA", |r| {
            r.components[2].known_sha256 = vec!["bad".into()]
        }),
        ("references a download", |r| {
            r.components[2].members[0].package = Some("free".into())
        }),
        ("unknown package/member", |r| {
            r.components[0].members[0].member = Some("missing".into())
        }),
        ("duplicate component member", |r| {
            let m = r.components[0].members[0].clone();
            r.components[0].members.push(m);
        }),
        ("role inconsistent", |r| {
            r.components[0].members[0].role = Role::Dehacked
        }),
        ("role inconsistent", |r| {
            r.components[0].members[0].member = Some("credits".into())
        }),
        ("add-on must load after", |r| {
            r.components[1].replaces = None;
            r.components[1].position = 1;
        }),
        ("ambiguous load order", |r| {
            r.components[1].replaces = None;
            r.components[1].position = 2;
            r.components[2].replaces = None;
            r.components[2].position = 2;
            r.components[2].exclusive_group = None;
        }),
        ("loaded twice", |r| {
            r.components[1].replaces = None;
            r.components[1].position = 2;
            r.components[1].members[0].member = Some("maps".into());
        }),
        ("default component", |r| {
            r.components[1].default = true;
            r.components[2].default = true;
        }),
        ("unreviewed complevel", |r| {
            r.compatibility.complevel = Complevel::Number(18)
        }),
        ("engine version range", |r| {
            r.compatibility.engine.range = "invalid".into()
        }),
        ("engine version range", |r| {
            r.compatibility.engine.range = "<0.30.1".into()
        }),
        ("invalid engine version", |r| {
            r.verification.engine_version = "not a version".into()
        }),
        ("invalid warp", |r| {
            r.compatibility.runtime.warp = Some(vec![100])
        }),
        ("verification date", |r| {
            r.verification.date = "2026-02-30".into()
        }),
        ("verification evidence", |r| r.verification.tester.clear()),
        ("inspection evidence", |r| r.verification.inspected.clear()),
        ("inspection mislabeled", |r| {
            r.compatibility.engine.status = VerificationStatus::Verified
        }),
        ("screenshot asset", |r| {
            r.screenshots.push(Screenshot {
                asset: "../escape.jpg".into(),
                credit: "Someone".into(),
            })
        }),
        ("screenshot asset", |r| {
            r.screenshots.push(Screenshot {
                asset: "card.gif".into(),
                credit: "Someone".into(),
            })
        }),
        ("screenshot asset", |r| {
            let shot = r.screenshots[0].clone();
            r.screenshots.push(shot);
        }),
        ("screenshot credit", |r| {
            r.screenshots[0].credit.clear();
        }),
    ];
    for (message, mutation) in cases {
        let mut r = recipe("sigil");
        mutation(&mut r);
        let errors = validate(&r, Path::new("sigil.toml"), &hosts());
        assert!(
            errors.iter().any(|e| e.message.contains(message)),
            "expected {message}, got {errors:?}"
        );
    }
}

#[test]
fn schema_rejects_unknown_fields_at_every_level_and_wrong_enums() {
    let source = fs::read_to_string(format!("{CATALOG}/sigil.toml")).unwrap();
    for header in [
        "",
        "[base_game]",
        "[compatibility]",
        "[compatibility.engine]",
        "[compatibility.runtime]",
        "[verification]",
        "[[packages]]",
        "[[packages.members]]",
        "[[components]]",
        "[[components.members]]",
    ] {
        let text = if header.is_empty() {
            format!("unknown = true\n{source}")
        } else {
            source.replacen(header, &format!("{header}\nunknown = true"), 1)
        };
        assert!(
            toml::from_str::<Recipe>(&text)
                .unwrap_err()
                .message()
                .contains("unknown field"),
            "{header}"
        );
    }
    for (old, new) in [
        ("game = \"doom\"", "game = \"shareware\""),
        ("family = \"limit-removing\"", "family = \"gzdoom\""),
        ("status = \"inspected\"", "status = \"untested\""),
        ("kind = \"zip\"", "kind = \"exe\""),
        ("role = \"maps\"", "role = \"script\""),
        ("complevel = \"vanilla\"", "complevel = \"auto\""),
    ] {
        assert!(toml::from_str::<Recipe>(&source.replacen(old, new, 1)).is_err());
    }
}

#[test]
fn reviewed_complevels_runtime_defaults_and_dates() {
    let mut r = recipe("sigil");
    for n in 0..=21 {
        r.compatibility.complevel = Complevel::Number(n);
        // 3 is what `vanilla` means on Doom, 9/11/21 are named by an alias: the
        // lint asks for the alias there (D-030), so only the others stay numbers.
        assert_eq!(
            validate(&r, Path::new("sigil.toml"), &hosts()).is_empty(),
            (n <= 17 || n == 21) && ![3, 9, 11, 21].contains(&n),
            "{n}"
        );
    }
    for alias in [
        ComplevelAlias::Vanilla,
        ComplevelAlias::Boom,
        ComplevelAlias::Mbf,
        ComplevelAlias::Mbf21,
    ] {
        r.compatibility.complevel = Complevel::Alias(alias);
        r.compatibility.runtime.warp = Some(vec![5, 1]);
        r.compatibility.runtime.pistol_start = Some(true);
        r.verification.date = "2024-02-29".into();
        assert!(validate(&r, Path::new("sigil.toml"), &hosts()).is_empty());
    }
}

/// Lints a mutated copy of a catalog recipe and returns the complevel violations.
fn complevel_violations(id: &str, change: impl FnOnce(&mut Recipe)) -> Vec<lint::Violation> {
    let mut r = recipe(id);
    change(&mut r);
    validate(&r, Path::new(&format!("{id}.toml")), &hosts())
        .into_iter()
        .filter(|v| v.key == "compatibility.complevel")
        .collect()
}

#[test]
fn a_number_an_alias_names_is_rejected_with_the_alias_as_the_hint() {
    // D-030: 2 on Doom II, 3 on Doom, 4 on TNT/Plutonia are `vanilla`.
    for (id, game, number) in [
        ("auger-zenith", Game::Doom2, 2),
        ("sigil", Game::Doom, 3),
        ("auger-zenith", Game::Tnt, 4),
        ("auger-zenith", Game::Plutonia, 4),
    ] {
        let found = complevel_violations(id, |r| {
            r.base_game.game = game;
            r.compatibility.complevel = Complevel::Number(number);
        });
        assert_eq!(found.len(), 1, "{game:?} {number}: {found:?}");
        assert!(found[0].message.contains("`vanilla`"), "{found:?}");
        assert!(
            found[0].hint.contains("complevel = \"vanilla\""),
            "{found:?}"
        );
    }
    for (number, alias) in [(9, "boom"), (11, "mbf"), (21, "mbf21")] {
        let found = complevel_violations("auger-zenith", |r| {
            r.compatibility.complevel = Complevel::Number(number);
        });
        assert_eq!(found.len(), 1, "{number}: {found:?}");
        assert!(
            found[0].hint.contains(&format!("complevel = \"{alias}\"")),
            "{found:?}"
        );
    }
}

#[test]
fn an_alias_or_a_number_no_alias_names_is_accepted() {
    for alias in [
        ComplevelAlias::Vanilla,
        ComplevelAlias::Boom,
        ComplevelAlias::Mbf,
        ComplevelAlias::Mbf21,
    ] {
        let found = complevel_violations("auger-zenith", |r| {
            r.compatibility.complevel = Complevel::Alias(alias);
        });
        assert_eq!(found, vec![], "{alias:?}");
    }
    // 2 is not `vanilla` on Ultimate Doom, and 4 is not on Doom II.
    for (game, number) in [(Game::Doom, 2), (Game::Doom2, 4), (Game::Doom, 7)] {
        let found = complevel_violations("sigil", |r| {
            r.base_game.game = game;
            r.compatibility.complevel = Complevel::Number(number);
        });
        assert_eq!(found, vec![], "{game:?} {number}");
    }
}

#[test]
fn every_catalog_recipe_uses_an_alias_whenever_one_names_its_level() {
    let mut checked = 0;
    for entry in fs::read_dir(CATALOG).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if name == "hosts.toml"
            || name == "tags.toml"
            || path.extension().is_none_or(|x| x != "toml")
        {
            continue;
        }
        let r: Recipe = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        if let Complevel::Number(n) = r.compatibility.complevel {
            assert_eq!(ComplevelAlias::naming(n, r.base_game.game), None, "{name}");
        }
        checked += 1;
    }
    assert!(checked >= 80, "{checked} recipes");
}

#[test]
fn the_removed_complevel_status_key_fails_lint_with_a_clear_message() {
    let source = fs::read_to_string(format!("{CATALOG}/sigil.toml")).unwrap();
    let with_status = source.replacen(
        "complevel = \"vanilla\"",
        "complevel = \"vanilla\"\ncomplevel_status = \"proposed\"",
        1,
    );
    assert_ne!(with_status, source);

    let dir = Temp::new();
    let path = dir.0.join("sigil.toml");
    fs::write(&path, &with_status).unwrap();
    let diagnostics = lint_paths(
        std::slice::from_ref(&path),
        &Path::new(CATALOG).join("hosts.toml"),
        None,
    );
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert!(
        diagnostics[0].message.contains("`complevel_status`")
            && diagnostics[0].message.contains("no longer part"),
        "{diagnostics:?}"
    );
    assert!(diagnostics[0].hint.contains("delete"), "{diagnostics:?}");
    assert_eq!(diagnostics[0].line, 29, "{diagnostics:?}");

    // The embedded loader skips such a recipe with the same reason.
    let hosts_text = fs::read_to_string(format!("{CATALOG}/hosts.toml")).unwrap();
    let loaded = Catalog::from_sources(&hosts_text, "", &[("sigil.toml", with_status.as_str())]);
    assert!(loaded.recipes.is_empty());
    assert!(
        loaded.skipped[0].reason.contains("complevel_status"),
        "{:?}",
        loaded.skipped
    );
}

#[test]
fn https_hosts_paths_and_hashes_are_strict() {
    for url in [
        "http://romero.com/x",
        "https://romero.com.evil/x",
        "https://evil@romero.com/x",
        "https://romero.com:444/x",
        "https://romero.com/x#fragment",
        "file:///etc/passwd",
        "https://www.doomworld.com/x",
    ] {
        assert!(lint::checked_url(url, &hosts().downloads).is_err(), "{url}");
    }
    for path in [
        "/x",
        "../x",
        "x/../y",
        "C:/x",
        "x\\y",
        "x//y",
        "./x",
        "x.zip!/../y",
        "x.zip!/y.zip!/z.zip!/a.zip!/b",
        "-x.wad",
    ] {
        assert!(!lint::safe_path(path), "{path}");
    }
    assert!(lint::safe_path(
        "DBP37_Addons.zip!/DBP37_AlternateBlaster.wad"
    ));
    assert!(lint::hash(&"A".repeat(64)));
}

#[test]
fn failing_fixtures_have_precise_lines_messages_and_hints() {
    for (folder, message, line) in [
        ("invalid-hash", "malformed package SHA-256", 40),
        ("unknown-key", "unknown field", 1),
    ] {
        let errors = lint_paths(
            &[Path::new(FIXTURES).join(folder).join("sigil.toml")],
            &Path::new(CATALOG).join("hosts.toml"),
            None,
        );
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].message.contains(message), "{errors:?}");
        assert_eq!(errors[0].line, line, "{errors:?}");
        assert!(!errors[0].hint.is_empty());
        assert!(
            errors[0]
                .to_string()
                .contains(&format!("sigil.toml:{line}:"))
        );
    }
}

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ddl-catalog-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn lint_reports_duplicate_recipe_ids_io_empty_and_bad_hosts() {
    let temp = Temp::new();
    for name in ["sigil.toml", "duplicate.toml"] {
        fs::write(
            temp.0.join(name),
            toml::to_string(&recipe("sigil")).unwrap(),
        )
        .unwrap();
    }
    let errors = lint_paths(
        std::slice::from_ref(&temp.0),
        &Path::new(CATALOG).join("hosts.toml"),
        None,
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("duplicate recipe id"))
    );
    assert!(
        !lint_paths(
            &[temp.0.join("missing")],
            &Path::new(CATALOG).join("hosts.toml"),
            None
        )
        .is_empty()
    );
    let empty = Temp::new();
    assert!(
        lint_paths(
            std::slice::from_ref(&empty.0),
            &Path::new(CATALOG).join("hosts.toml"),
            None
        )[0]
        .message
        .contains("no recipe")
    );
    fs::write(
        empty.0.join("hosts.toml"),
        "schema_version=2\npages=[]\ndownloads=[]",
    )
    .unwrap();
    assert!(
        lint_paths(
            std::slice::from_ref(&empty.0),
            &empty.0.join("hosts.toml"),
            None
        )[0]
        .message
        .contains("invalid host")
    );
    fs::write(
        empty.0.join("large.toml"),
        " ".repeat(lint::MAX_RECIPE_BYTES as usize + 1),
    )
    .unwrap();
    assert!(
        lint_paths(
            &[empty.0.join("large.toml")],
            &Path::new(CATALOG).join("hosts.toml"),
            None
        )[0]
        .message
        .contains("exceeds 1 MiB")
    );
}

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        writer
            .start_file(
                *name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )
            .unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
fn package(bytes: &[u8], path: &str, member: &[u8]) -> Package {
    Package {
        id: "fixture".into(),
        page: "https://romero.com/sigil".into(),
        urls: vec!["https://romero.com/one.zip".into()],
        size: bytes.len() as u64,
        sha256: sha(bytes),
        kind: PackageKind::Zip,
        idgames_path: None,
        members: vec![Member {
            id: "main".into(),
            path: path.into(),
            size: member.len() as u64,
            sha256: sha(member),
            keep: false,
        }],
    }
}

#[test]
fn fetch_checks_nested_zip_exact_case_size_and_hash_without_network() {
    let inner = zip_bytes(&[("Main.WAD", b"fixture")]);
    let outer = zip_bytes(&[("inner.zip", &inner)]);
    let p = package(&outer, "inner.zip!/Main.WAD", b"fixture");
    assert!(check_package(&p, &outer).is_ok());
    for (expected, mutation) in [
        ("size mismatch", 0),
        ("SHA-256 mismatch", 1),
        ("size mismatch", 2),
        ("SHA-256 mismatch", 3),
        ("missing exact member", 4),
    ] {
        let mut bad = p.clone();
        match mutation {
            0 => bad.size += 1,
            1 => bad.sha256 = "0".repeat(64),
            2 => bad.members[0].size += 1,
            3 => bad.members[0].sha256 = "0".repeat(64),
            _ => bad.members[0].path = "inner.zip!/main.wad".into(),
        }
        assert!(check_package(&bad, &outer).unwrap_err().contains(expected));
    }
    let mut bare = package(b"wad", "main.wad", b"wad");
    bare.kind = PackageKind::Wad;
    assert!(check_package(&bare, b"wad").is_ok());
    let corrupt = package(b"not a zip", "main.wad", b"wad");
    assert!(
        check_package(&corrupt, b"not a zip")
            .unwrap_err()
            .contains("invalid ZIP")
    );
}

#[test]
fn fetch_rejects_archive_traversal_symlinks_and_bombs() {
    for name in ["../evil.wad", "/evil.wad", "C:/evil.wad"] {
        let bytes = zip_bytes(&[(name, b"evil"), ("main.wad", b"good")]);
        assert!(
            check_package(&package(&bytes, "main.wad", b"good"), &bytes)
                .unwrap_err()
                .contains("unsafe ZIP entry")
        );
    }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .add_symlink(
            "main.wad",
            "/outside",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    assert!(
        check_package(&package(&bytes, "main.wad", b"good"), &bytes)
            .unwrap_err()
            .contains("symlink")
    );
    // Patch only the central-directory uncompressed size; avoid allocating a bomb.
    let mut bytes = zip_bytes(&[("main.wad", b"good")]);
    let central = bytes.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
    bytes[central + 24..central + 28]
        .copy_from_slice(&((lint::MAX_BYTES + 1) as u32).to_le_bytes());
    assert!(
        check_package(&package(&bytes, "main.wad", b"good"), &bytes)
            .unwrap_err()
            .contains("budget")
    );
}

#[test]
fn deh_order_is_separate_and_members_are_never_implicitly_loaded() {
    let mut r = recipe("sigil");
    let mut patch = r.packages[0].members[0].clone();
    patch.id = "patch".into();
    patch.path = "patch.bex".into();
    r.packages[0].members.push(patch);
    r.components[0].members.push(ComponentMember {
        package: Some("free".into()),
        member: Some("patch".into()),
        role: Role::Dehacked,
    });
    assert!(validate(&r, Path::new("sigil.toml"), &hosts()).is_empty());
    let order = r.load_order(&[]).unwrap();
    assert_eq!(order.files.len(), 1);
    assert_eq!(order.deh.len(), 1);
    assert_eq!(order.deh[0].member.as_deref(), Some("patch"));
}

#[test]
fn injected_fetch_verifies_declared_bytes_and_reports_failures() {
    struct Fake {
        bytes: Vec<u8>,
        calls: usize,
    }
    impl Fetch for Fake {
        fn download(&mut self, _package: &Package, _hosts: &Hosts) -> Result<Vec<u8>, String> {
            self.calls += 1;
            Ok(self.bytes.clone())
        }
    }

    let bytes = zip_bytes(&[("main.wad", b"good")]);
    let package = package(&bytes, "main.wad", b"good");
    let mut fake = Fake {
        bytes: bytes.clone(),
        calls: 0,
    };
    assert!(fetch::fetch_package(&package, &hosts(), &mut fake, &[]).is_ok());
    assert_eq!(fake.calls, 1);

    // A fetch that returns the wrong bytes is reported with the package id.
    fake.bytes = vec![];
    fake.calls = 0;
    let error = fetch::fetch_package(&package, &hosts(), &mut fake, &[]).unwrap_err();
    assert!(error.contains("size mismatch"), "{error}");
    assert!(error.contains("package `fixture`"), "{error}");
    assert_eq!(fake.calls, 1);
}

#[test]
fn lint_fetch_calls_the_injected_boundary_only_when_requested() {
    struct Fake {
        bytes: Vec<u8>,
        calls: usize,
    }
    impl Fetch for Fake {
        fn download(&mut self, _package: &Package, _hosts: &Hosts) -> Result<Vec<u8>, String> {
            self.calls += 1;
            Ok(self.bytes.clone())
        }
    }

    let bytes = zip_bytes(&[("main.wad", b"good")]);
    let temp = Temp::new();
    let mut recipe = recipe("sigil");
    recipe.packages = vec![package(&bytes, "main.wad", b"good")];
    recipe.packages[0].id = "free".into();
    recipe.packages[0].members[0].id = "maps".into();
    recipe.components.truncate(1);
    fs::write(temp.0.join("sigil.toml"), toml::to_string(&recipe).unwrap()).unwrap();

    let mut fake = Fake { bytes, calls: 0 };
    // Offline lint (None) never touches the boundary.
    assert!(
        lint_paths(
            std::slice::from_ref(&temp.0),
            &Path::new(CATALOG).join("hosts.toml"),
            None
        )
        .is_empty()
    );
    assert_eq!(fake.calls, 0);
    // --fetch invokes it once per package.
    assert!(
        lint_paths(
            std::slice::from_ref(&temp.0),
            &Path::new(CATALOG).join("hosts.toml"),
            Some(&mut fake)
        )
        .is_empty()
    );
    assert_eq!(fake.calls, 1);
}

/// Builds the smallest valid in-memory WAD whose directory names are `names`.
/// One zero byte of payload per lump keeps fixtures tiny (spec 017, R4).
fn wad_bytes(magic: &[u8; 4], names: &[&str]) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(magic);
    data.extend_from_slice(&(names.len() as i32).to_le_bytes());
    data.extend_from_slice(&0_i32.to_le_bytes()); // infotableofs, patched below
    let mut offsets = Vec::new();
    for _ in names {
        offsets.push((data.len() as i32, 1_i32));
        data.push(0);
    }
    let infotableofs = data.len() as i32;
    data[8..12].copy_from_slice(&infotableofs.to_le_bytes());
    for (name, (offset, size)) in names.iter().zip(offsets) {
        data.extend_from_slice(&offset.to_le_bytes());
        data.extend_from_slice(&size.to_le_bytes());
        let mut field = [0_u8; 8];
        let end = name.len().min(8);
        field[..end].copy_from_slice(&name.as_bytes()[..end]);
        data.extend_from_slice(&field);
    }
    data
}

fn bare_wad_package(id: &str, member_id: &str, bytes: &[u8], path: &str) -> Package {
    Package {
        id: id.into(),
        page: "https://github.com/NightFright2k19/doom_widehud".into(),
        urls: vec![format!(
            "https://raw.githubusercontent.com/NightFright2k19/doom_widehud/eb3773ce9e4563c69a092386bf2e4f8d740dfd4d/pwad/{path}"
        )],
        size: bytes.len() as u64,
        sha256: sha(bytes),
        kind: PackageKind::Wad,
        idgames_path: None,
        members: vec![Member {
            id: member_id.into(),
            path: path.into(),
            size: bytes.len() as u64,
            sha256: sha(bytes),
            keep: false,
        }],
    }
}

#[test]
fn wad_directory_reads_and_bounds_lump_names() {
    assert_eq!(
        wad_lump_names(&wad_bytes(b"PWAD", &["STBAR"])).unwrap(),
        ["STBAR"]
    );
    assert_eq!(
        wad_lump_names(&wad_bytes(b"IWAD", &["STBAR", "MAP01"])).unwrap(),
        ["STBAR", "MAP01"]
    );
    assert!(wad_lump_names(b"not a wad").is_err());
    assert!(wad_lump_names(&[]).is_err());
    let mut truncated = wad_bytes(b"IWAD", &["STBAR", "MAP01"]);
    truncated.truncate(truncated.len() - 1);
    assert!(wad_lump_names(&truncated).is_err());
}

#[test]
fn cosmetic_only_lumps_allow_status_bar_families_and_reject_the_rest() {
    let allowed = [
        "STBAR", "STARMS", "STFST00", "STFOUCH0", "STFTL10", "STFGOD0", "STTNUM0", "STTMINUS",
        "STTPRCNT", "STGNUM3", "STYSNUM4", "STKEYS0", "STCFN033", "STDISK", "STCDROM", "BRDR_TL",
    ];
    let names: Vec<String> = allowed.iter().map(|s| (*s).to_string()).collect();
    assert!(check_status_bar_lumps(&names).is_ok(), "{names:?}");
    let mut lumpy = names.clone();
    for bad in [
        "DEHACKED", "MAPINFO", "UMAPINFO", "MAP01", "D_RUNNIN", "DSPISTOL", "TROOA1", "TEXTURE1",
        "PNAMES", "BOSSBACK", "CWILV00", "WINUM0", "STEEL1", "STEP1",
    ] {
        lumpy.push(bad.to_string());
        assert!(
            check_status_bar_lumps(&lumpy).unwrap_err().contains(bad),
            "{bad}"
        );
        lumpy.pop();
    }
}

#[test]
fn fetch_package_rejects_a_non_cosmetic_status_bar_member() {
    struct Fake(Vec<u8>);
    impl Fetch for Fake {
        fn download(&mut self, _package: &Package, _hosts: &Hosts) -> Result<Vec<u8>, String> {
            Ok(self.0.clone())
        }
    }
    let clean = wad_bytes(b"PWAD", &["STBAR"]);
    let package = bare_wad_package("widehud", "statusbar", &clean, "x.wad");
    assert!(fetch::fetch_package(&package, &hosts(), &mut Fake(clean.clone()), &["x.wad"]).is_ok());
    let dirty = wad_bytes(b"PWAD", &["DEHACKED", "MAPINFO", "STBAR"]);
    let package = bare_wad_package("widehud", "statusbar", &dirty, "x.wad");
    let error =
        fetch::fetch_package(&package, &hosts(), &mut Fake(dirty.clone()), &["x.wad"]).unwrap_err();
    assert!(error.contains("non-cosmetic lump `DEHACKED`"), "{error}");
    // A package no status-bar component references is verified, not lump-checked.
    assert!(fetch::fetch_package(&package, &hosts(), &mut Fake(dirty.clone()), &[]).is_ok());
}

#[test]
fn lint_fetch_rejects_a_status_bar_component_that_is_not_cosmetic() {
    struct Fake {
        main: Vec<u8>,
        bar: Vec<u8>,
    }
    impl Fetch for Fake {
        fn download(&mut self, package: &Package, _hosts: &Hosts) -> Result<Vec<u8>, String> {
            Ok(if package.id == "widehud" {
                self.bar.clone()
            } else {
                self.main.clone()
            })
        }
    }
    let main = wad_bytes(b"PWAD", &["MAP01"]);
    let dirty = wad_bytes(b"PWAD", &["DEHACKED", "STBAR"]);
    let build = |bar: &[u8]| {
        let mut r = recipe("eviternity");
        r.packages = vec![
            bare_wad_package("free", "maps", &main, "main.wad"),
            bare_wad_package("widehud", "statusbar", bar, "x.wad"),
        ];
        r.components = vec![
            Component {
                id: "main".into(),
                label: "Eviternity".into(),
                description: "main".into(),
                optional: false,
                default: true,
                position: 1,
                exclusive_group: None,
                replaces: None,
                requires: None,
                user_supplied: false,
                obtain: None,
                known_sha256: vec![],
                members: vec![ComponentMember {
                    package: Some("free".into()),
                    member: Some("maps".into()),
                    role: Role::Maps,
                }],
            },
            Component {
                id: "wide-status-bar".into(),
                label: "Widescreen status bar".into(),
                description: "bar".into(),
                optional: true,
                default: true,
                position: 2,
                exclusive_group: None,
                replaces: None,
                requires: None,
                user_supplied: false,
                obtain: None,
                known_sha256: vec![],
                members: vec![ComponentMember {
                    package: Some("widehud".into()),
                    member: Some("statusbar".into()),
                    role: Role::Graphics,
                }],
            },
        ];
        r
    };
    let temp = Temp::new();
    let file = temp.0.join("eviternity.toml");
    let hosts_path = Path::new(CATALOG).join("hosts.toml");
    let notes = |r: &Recipe| toml::to_string_pretty(r).unwrap();
    fs::write(&file, notes(&build(&dirty))).unwrap();
    let errors = lint_paths(
        std::slice::from_ref(&file),
        &hosts_path,
        Some(&mut Fake {
            main: main.clone(),
            bar: dirty.clone(),
        }),
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].message.contains("non-cosmetic lump `DEHACKED`"));
    let clean = wad_bytes(b"PWAD", &["STBAR"]);
    fs::write(&file, notes(&build(&clean))).unwrap();
    assert_eq!(
        lint_paths(
            std::slice::from_ref(&file),
            &hosts_path,
            Some(&mut Fake {
                main: main.clone(),
                bar: clean.clone(),
            }),
        ),
        vec![]
    );
}

#[test]
fn https_fetch_downloads_through_the_shared_006_pipeline() {
    use crate::downloads::test_http::{Response, Server, TestDir};

    let body = zip_bytes(&[("main.wad", b"good")]);
    let served = body.clone();
    let server = Server::start(move |_req| Response::ok(served.clone()));
    let dir = TestDir::new("catalog-fetch");

    let mut package = package(&body, "main.wad", b"good");
    package.urls = vec![server.url("/pkg.zip")];
    let hosts = Hosts {
        schema_version: 1,
        pages: vec!["127.0.0.1".into()],
        downloads: vec!["127.0.0.1".into()],
    };

    let mut fetch = HttpsFetch::new(dir.path()).allow_http_for_tests();
    fetch
        .download(&package, &hosts)
        .expect("the shared fetcher downloads and verifies the package");

    // A wrong declared hash is a typed fetcher failure, not a silent success.
    package.sha256 = "0".repeat(64);
    let error = fetch.download(&package, &hosts).unwrap_err();
    assert!(error.contains("SHA-256"), "{error}");
}

#[test]
fn embedded_catalog_loads_the_reference_recipes_without_skips() {
    let catalog = Catalog::embedded();
    assert!(
        catalog.skipped.is_empty(),
        "the shipped catalog must be clean: {:?}",
        catalog.skipped
    );
    let mut ids: Vec<&str> = catalog.ids().collect();
    ids.sort_unstable();
    assert_eq!(
        ids,
        [
            "a-doomer-boards-christmas-carol",
            "after-the-fall",
            "alien-bastards",
            "alien-vendetta",
            "alone",
            "ancient-aliens",
            "antaresian-reliquary",
            "aquatic-wonder",
            "auger-zenith",
            "augustland-barrens",
            "avpvtvjvvhvss",
            "back-to-saturn-x-e1",
            "back-to-saturn-x-e2",
            "biotech-is-godzilla",
            "carnage-oasis",
            "caustic-industry",
            "christmas-in-inferno",
            "chronicles-of-ghost-town",
            "circle-of-caina",
            "coffin-curse",
            "cybs-freaky-colonoscopy",
            "dead-but-dreaming",
            "deadly-ritual",
            "dreamcatcher-apparatus",
            "dungeons-and-demons",
            "earth-ends",
            "eldritch-voyage",
            "emerald-city",
            "evil-egypt",
            "eviternity",
            "eviternity-ii",
            "fear-and-loathing",
            "forest-swords",
            "forgotten-psychedelics",
            "fuck-global-warming-this-is-hell",
            "funnelcake-apparitions",
            "going-down",
            "grief-arbiter",
            "haunting-hollow",
            "havoc-in-creation",
            "hell-revealed-3-pt-1",
            "immortal-warfare",
            "into-the-storm",
            "japanese-valentines",
            "jolly-junkyard",
            "legend-of-the-hidden-tech",
            "lilywhite-lilith",
            "luminous-gloom",
            "lunar-strain",
            "marble-hill",
            "mausoleum-nefarium",
            "memento-mori",
            "memento-mori-ii",
            "mindblood-genesis",
            "monuments-of-mars",
            "monuments-of-mars-2",
            "morbid-autumn",
            "narcoleptic-strata",
            "occult-secrets-of-the-third-reich-on-mars",
            "outposts-of-the-wasteland",
            "patagonian-pilgrimage",
            "phantom-processing",
            "plutonian-sunrise",
            "rainy-days-in-casablanca",
            "sanguine-holy-land",
            "santas-outback-bender",
            "scythe",
            "shatter-realm",
            "sigil",
            "sigil-ii",
            "slime-on-earth",
            "spaceballs-the-quest-for-more-dbp-maps",
            "stagnant-weather",
            "steamy-bathhouse",
            "stroggmans-tundra",
            "sunlust",
            "tempest-enterprise",
            "ten-day-vacation",
            "terminal-voltage",
            "the-city-of-damned-children",
            "the-economy-of-the-dead",
            "the-magnificent-five",
            "the-merry-christmassy-doom-project",
            "the-vast-silence",
            "tomatomania",
            "umbral-platinum",
            "valiant",
            "vicarious-reality",
            "vrack-botanicals",
            "wretched-morrow",
            "xenomorph-base",
            "zeppelin-armada",
        ]
    );
    for id in [
        "alien-vendetta",
        "ancient-aliens",
        "auger-zenith",
        "back-to-saturn-x-e1",
        "back-to-saturn-x-e2",
        "eviternity",
        "eviternity-ii",
        "going-down",
        "memento-mori",
        "memento-mori-ii",
        "scythe",
        "sigil",
        "sigil-ii",
        "sunlust",
        "valiant",
    ] {
        let loaded = catalog.get(id).expect("recipe present");
        assert!(loaded.file.ends_with(&format!("{id}.toml")));
        assert_eq!(loaded.sha256.len(), 64);
        assert_eq!(
            loaded.sha256,
            sha(fs::read_to_string(format!("{CATALOG}/{}", loaded.file))
                .unwrap()
                .as_bytes())
        );
    }
    assert!(!catalog.hosts.downloads.is_empty());
}

#[test]
fn invalid_recipes_are_skipped_with_a_logged_reason_never_crashing() {
    let hosts_text = fs::read_to_string(format!("{CATALOG}/hosts.toml")).unwrap();
    let tags_text = fs::read_to_string(format!("{CATALOG}/tags.toml")).unwrap();
    let good = fs::read_to_string(format!("{CATALOG}/sigil.toml")).unwrap();
    // A recipe whose package hash is malformed must be skipped, not accepted.
    let broken = good.replace(
        "57c35a6bcd722433fc97344314e52c06c91fdfde33b18c6d1c14a4d29e229105",
        "not-a-hash",
    );
    // A file that is not even TOML.
    let garbage = "this is not toml".to_owned();

    let catalog = Catalog::from_sources(
        &hosts_text,
        &tags_text,
        &[
            ("sigil.toml", good.as_str()),
            ("broken.toml", broken.as_str()),
            ("garbage.toml", garbage.as_str()),
        ],
    );
    assert_eq!(catalog.ids().collect::<Vec<_>>(), ["sigil"]);
    assert_eq!(catalog.skipped.len(), 2, "{:?}", catalog.skipped);
    let broken = catalog
        .skipped
        .iter()
        .find(|entry| entry.file == "broken.toml")
        .expect("broken recipe reported");
    assert!(broken.reason.contains("SHA-256"), "{broken:?}");
    let garbage = catalog
        .skipped
        .iter()
        .find(|entry| entry.file == "garbage.toml")
        .expect("garbage recipe reported");
    assert!(!garbage.reason.is_empty());
}

#[test]
fn duplicate_recipe_ids_keep_the_first_file() {
    let hosts_text = fs::read_to_string(format!("{CATALOG}/hosts.toml")).unwrap();
    let tags_text = fs::read_to_string(format!("{CATALOG}/tags.toml")).unwrap();
    let good = fs::read_to_string(format!("{CATALOG}/sigil.toml")).unwrap();
    let catalog = Catalog::from_sources(
        &hosts_text,
        &tags_text,
        &[("sigil.toml", good.as_str()), ("sigil.toml", good.as_str())],
    );
    assert_eq!(catalog.recipes.len(), 1);
    assert_eq!(catalog.recipes[0].file, "sigil.toml");
    assert_eq!(catalog.skipped.len(), 1);
    assert!(catalog.skipped[0].reason.contains("duplicate recipe id"));
}

#[test]
fn semantic_diagnostics_locate_repeated_nested_keys() {
    let temp = Temp::new();
    let mut r = recipe("auger-zenith");
    r.packages[0].members[3].sha256 = "bad".into();
    let text = toml::to_string_pretty(&r).unwrap();
    let expected = text
        .lines()
        .position(|line| line.contains("\"bad\""))
        .unwrap()
        + 1;
    fs::write(temp.0.join("auger-zenith.toml"), &text).unwrap();
    let errors = lint_paths(
        std::slice::from_ref(&temp.0),
        &Path::new(CATALOG).join("hosts.toml"),
        None,
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].line, expected, "{errors:?}");
}

fn vocabulary() -> TagVocabulary {
    toml::from_str(&fs::read_to_string(format!("{CATALOG}/tags.toml")).unwrap()).unwrap()
}

#[test]
fn shipped_vocabulary_is_valid_and_has_the_spec_015_tags() {
    let vocabulary = vocabulary();
    assert_eq!(validate_vocabulary(&vocabulary), vec![]);
    let ids: Vec<_> = vocabulary.ordered().iter().map(|t| t.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "cacowards",
            "megawad",
            "community-projects",
            "doomer-boards",
            "classics",
            "episode",
            "challenging",
            "featured",
            "beginner-friendly"
        ]
    );
    assert_eq!(Catalog::embedded().tags, vocabulary);
}

/// Spec 073, R5: the owner's Discover sequence is the vocabulary's prefix, and
/// Featured/Beginner Friendly keep their vocabulary outside it.
#[test]
fn requested_discover_sequence_leads_the_vocabulary() {
    let vocabulary = vocabulary();
    let ordered: Vec<_> = vocabulary.ordered().iter().map(|t| t.id.as_str()).collect();
    assert_eq!(
        &ordered[..7],
        [
            "cacowards",
            "megawad",
            "community-projects",
            "doomer-boards",
            "classics",
            "episode",
            "challenging"
        ],
        "the requested category sequence, in order"
    );
    let outside = &ordered[7..];
    assert!(
        outside.contains(&"featured") && outside.contains(&"beginner-friendly"),
        "metadata tags stay in the vocabulary but out of the sequence: {outside:?}"
    );
    let doomer_boards = vocabulary.get("doomer-boards").expect("new tag");
    assert_eq!(doomer_boards.label, "Doomer Boards");
    assert!(!doomer_boards.description.trim().is_empty());
}

/// Spec 073, R5: every recipe named by the verified DBP inventory
/// (`docs/research/doomer-boards-projects.md`) carries `doomer-boards`, and it
/// keeps its general `community-projects` membership.
#[test]
fn doomer_boards_covers_every_researched_dbp_recipe() {
    let catalog = Catalog::embedded();
    let mut tagged = Vec::new();
    for held in &catalog.recipes {
        if held.recipe.tags.iter().any(|t| t == "doomer-boards") {
            tagged.push(held.recipe.id.as_str());
        }
    }
    tagged.sort_unstable();
    let mut expected = dbp_recipe_ids();
    expected.sort_unstable();
    assert_eq!(tagged, expected, "every DBP recipe, and nothing else");
    assert_eq!(expected.len(), 77, "DBP 01–77 in the inventory");
    for id in expected {
        let recipe = catalog.recipe(id).expect(id);
        assert!(
            recipe.tags.iter().any(|t| t == "community-projects"),
            "{id} keeps its community-projects membership"
        );
    }
}

/// Spec 073, R5: multi-author projects that are not Doomer Boards Projects keep
/// `community-projects` and must never be relabelled.
#[test]
fn non_dbp_community_projects_stay_out_of_doomer_boards() {
    let catalog = Catalog::embedded();
    for id in [
        "alien-vendetta",
        "back-to-saturn-x-e1",
        "back-to-saturn-x-e2",
        "eviternity",
        "eviternity-ii",
        "memento-mori",
        "memento-mori-ii",
    ] {
        let recipe = catalog.recipe(id).expect(id);
        assert!(
            recipe.tags.iter().any(|t| t == "community-projects"),
            "{id} is a multi-author project"
        );
        assert!(
            !recipe.tags.iter().any(|t| t == "doomer-boards"),
            "{id} is not a Doomer Boards Project"
        );
    }
}

/// Spec 073, R1/R2: the two new Classics freeze their verified source identity,
/// composition and category reachability, plus Scythe's existing membership.
#[test]
fn memento_mori_recipes_keep_their_source_identity() {
    let catalog = Catalog::embedded();
    for (id, package_sha, size, maps, music, readme) in [
        (
            "memento-mori",
            "c8607a7a32d4f6547d492e0f734324320b955e2b11691b02b107de09ac2e7bfb",
            2_950_178,
            (
                "3e34522f61e40e508fbbcf1f32c3de3f9dc9ec5749e2a411bbe646d7ae35a22e",
                7_825_282,
            ),
            (
                "3e1618f79d6e045ce18f7f72f8a329595e475b1d61b19264bc6b73d615345e73",
                885_387,
            ),
            (
                "33f67e460161ae8153fe0c06735d9c6d827736b716ace24a3519db0f0da45bee",
                2_893,
            ),
        ),
        (
            "memento-mori-ii",
            "f4e7a10b035b123647f631348d63fc46392d8d8e21fab8a5158cd9ef35027451",
            4_061_135,
            (
                "810c3eb9eb38cd86a5b6ba0bc032d8a9a7a29a58072acbb68d4bb824a8de859b",
                11_453_000,
            ),
            (
                "d09350612e953c1dc7055dbcb05228c698f7267bdf38d6cb1de11966c7b922d5",
                1_075_404,
            ),
            (
                "8706785377fb8be6e832c745952c8ab9ecec58736d57667d717329e5b1c354e5",
                42_580,
            ),
        ),
    ] {
        let r = catalog.recipe(id).expect(id);
        assert!(
            r.tags.iter().any(|t| t == "classics"),
            "{id} under Classics"
        );
        assert!(r.tags.iter().any(|t| t == "megawad"), "{id}: 32 maps");
        assert_eq!(r.base_game.game, Game::Doom2);
        assert_eq!(r.compatibility.family, PortFamily::Vanilla);
        assert_eq!(
            r.compatibility.complevel,
            Complevel::Alias(ComplevelAlias::Vanilla)
        );
        assert_eq!(r.packages.len(), 1);
        let package = &r.packages[0];
        assert_eq!((package.size, package.sha256.as_str()), (size, package_sha));
        assert_eq!(package.kind, PackageKind::Zip);
        // One required component: the maps, then the separate music WAD, in the
        // order the release's own launch line uses.
        assert_eq!(
            r.components.len(),
            1,
            "{id}: only the required main component"
        );
        let main = r.components.iter().find(|c| c.id == "main").expect("main");
        assert!(!main.optional && main.default);
        let roles: Vec<_> = main.members.iter().map(|m| m.role).collect();
        assert_eq!(roles, [Role::Maps, Role::Music], "{id}: maps then music");
        let order = r.load_order(&[]).expect("a valid default selection");
        assert_eq!(
            order
                .files
                .iter()
                .filter_map(|member| member.member.as_ref())
                .map(String::as_str)
                .collect::<Vec<_>>(),
            [maps_member(id), music_member(id)],
            "{id}: load order"
        );
        let loaded: Vec<_> = r
            .components
            .iter()
            .flat_map(|c| c.members.iter())
            .map(|m| {
                let member = r.packages[0]
                    .members
                    .iter()
                    .find(|candidate| Some(&candidate.id) == m.member.as_ref())
                    .expect("declared member");
                (
                    member.path.as_str(),
                    m.role,
                    member.size,
                    member.sha256.as_str(),
                )
            })
            .collect();
        let by_path = |wanted: &str| {
            loaded
                .iter()
                .find(|(path, ..)| *path == wanted)
                .copied()
                .unwrap_or_else(|| panic!("{id}: {wanted} loaded"))
        };
        assert_eq!(
            by_path(maps_path(id)),
            (maps_path(id), Role::Maps, maps.1, maps.0)
        );
        assert_eq!(
            by_path(music_path(id)),
            (music_path(id), Role::Music, music.1, music.0)
        );
        let readme_member = r.packages[0]
            .members
            .iter()
            .find(|m| m.path == readme_path(id))
            .expect("readme member");
        assert_eq!(
            (readme_member.size, readme_member.sha256.as_str()),
            (readme.1, readme.0)
        );
        assert!(readme_member.keep, "{id}: the credits text is kept");
    }
    // R2: Scythe already exists once and is already a Classic.
    let scythe: Vec<_> = catalog
        .recipes
        .iter()
        .filter(|held| held.recipe.id == "scythe")
        .collect();
    assert_eq!(scythe.len(), 1, "no duplicate Scythe recipe");
    let scythe = &scythe[0].recipe;
    assert!(scythe.tags.iter().any(|t| t == "classics"));
    assert!(!scythe.tags.iter().any(|t| t == "doomer-boards"));
    assert_eq!(scythe.packages[0].size, 2_086_863);
}

fn maps_path(id: &str) -> &'static str {
    if id == "memento-mori" {
        "MM.WAD"
    } else {
        "MM2.WAD"
    }
}

fn maps_member(id: &str) -> &'static str {
    if id == "memento-mori" {
        "mm-maps"
    } else {
        "mm2-maps"
    }
}

fn music_member(id: &str) -> &'static str {
    if id == "memento-mori" {
        "mm-music"
    } else {
        "mm2-music"
    }
}

fn music_path(id: &str) -> &'static str {
    if id == "memento-mori" {
        "MMMUS.WAD"
    } else {
        "MM2MUS.WAD"
    }
}

fn readme_path(id: &str) -> &'static str {
    if id == "memento-mori" {
        "MM.TXT"
    } else {
        "MM2.TXT"
    }
}

/// The recipe ids of DBP 01–77, from the verified inventory table in
/// `docs/research/doomer-boards-projects.md` (one row per published project).
fn dbp_recipe_ids() -> Vec<&'static str> {
    [
        "monuments-of-mars",
        "circle-of-caina",
        "forest-swords",
        "xenomorph-base",
        "coffin-curse",
        "vicarious-reality",
        "the-merry-christmassy-doom-project",
        "mindblood-genesis",
        "legend-of-the-hidden-tech",
        "earth-ends",
        "lilywhite-lilith",
        "into-the-storm",
        "alien-bastards",
        "after-the-fall",
        "fuck-global-warming-this-is-hell",
        "cybs-freaky-colonoscopy",
        "alone",
        "umbral-platinum",
        "a-doomer-boards-christmas-carol",
        "dungeons-and-demons",
        "occult-secrets-of-the-third-reich-on-mars",
        "biotech-is-godzilla",
        "evil-egypt",
        "spaceballs-the-quest-for-more-dbp-maps",
        "dead-but-dreaming",
        "the-city-of-damned-children",
        "ten-day-vacation",
        "fear-and-loathing",
        "morbid-autumn",
        "the-magnificent-five",
        "santas-outback-bender",
        "tomatomania",
        "rainy-days-in-casablanca",
        "luminous-gloom",
        "stroggmans-tundra",
        "aquatic-wonder",
        "auger-zenith",
        "chronicles-of-ghost-town",
        "carnage-oasis",
        "funnelcake-apparitions",
        "hell-revealed-3-pt-1",
        "slime-on-earth",
        "christmas-in-inferno",
        "japanese-valentines",
        "vrack-botanicals",
        "monuments-of-mars-2",
        "dreamcatcher-apparatus",
        "avpvtvjvvhvss",
        "mausoleum-nefarium",
        "emerald-city",
        "deadly-ritual",
        "havoc-in-creation",
        "plutonian-sunrise",
        "steamy-bathhouse",
        "jolly-junkyard",
        "augustland-barrens",
        "shatter-realm",
        "immortal-warfare",
        "zeppelin-armada",
        "eldritch-voyage",
        "tempest-enterprise",
        "haunting-hollow",
        "terminal-voltage",
        "the-vast-silence",
        "sanguine-holy-land",
        "lunar-strain",
        "marble-hill",
        "outposts-of-the-wasteland",
        "caustic-industry",
        "forgotten-psychedelics",
        "grief-arbiter",
        "wretched-morrow",
        "narcoleptic-strata",
        "phantom-processing",
        "patagonian-pilgrimage",
        "stagnant-weather",
        "the-economy-of-the-dead",
    ]
    .into_iter()
    .collect()
}

#[test]
fn vocabulary_rules_reject_bad_ids_text_and_order() {
    let mut v = vocabulary();
    v.tags[1].id = v.tags[0].id.clone();
    v.tags[2].id = "Bad Id".into();
    v.tags[3].label = " ".into();
    v.tags[4].order = v.tags[5].order;
    v.schema_version = 2;
    let keys: Vec<_> = validate_vocabulary(&v).into_iter().map(|e| e.key).collect();
    for key in [
        "schema_version",
        "tags.1.id",
        "tags.2.id",
        "tags.3.label",
        "tags.5.order",
    ] {
        assert!(keys.iter().any(|k| k == key), "{key} missing in {keys:?}");
    }
}

#[test]
fn unknown_and_duplicate_tags_are_lint_errors() {
    let mut r = recipe("sigil");
    r.tags = vec!["episode".into(), "no-such-tag".into(), "episode".into()];
    let errors = validate_tags(&r, &vocabulary());
    assert!(
        errors
            .iter()
            .any(|e| e.key == "tags.1" && e.message.contains("unknown tag `no-such-tag`")),
        "{errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|e| e.key == "tags.2" && e.message.contains("duplicate tag")),
        "{errors:?}"
    );
    r.tags = vec!["episode".into()];
    assert_eq!(validate_tags(&r, &vocabulary()), vec![]);
    r.tags.clear();
    assert_eq!(validate_tags(&r, &vocabulary()), vec![], "absent = none");
}

#[test]
fn cacowards_tag_needs_a_year_page_citation_in_notes_or_inspected() {
    let mut r = recipe("sigil");
    r.tags = vec!["cacowards".into()];
    r.verification.notes.clear();
    r.verification.inspected.clear();
    let missing = |r: &Recipe| {
        validate_tags(r, &vocabulary())
            .iter()
            .any(|e| e.message.contains("Cacowards citation"))
    };
    assert!(missing(&r));
    // A bare mention, another host or a page without a year is not a citation.
    r.verification.notes = "Cacoward winner".into();
    r.verification.inspected = vec![
        "https://example.com/cacowards/2019/".into(),
        "https://www.doomworld.com/cacowards/".into(),
    ];
    assert!(missing(&r));
    r.verification
        .inspected
        .push("Cacowards 2019: https://www.doomworld.com/cacowards/2019/".into());
    assert!(!missing(&r));
    r.verification.inspected.pop();
    r.verification.notes = "Listed in https://www.doomworld.com/cacowards/2021/best1/".into();
    assert!(!missing(&r));
    // Other tags never require it.
    r.verification.notes = "n".into();
    r.tags = vec!["episode".into()];
    assert!(!missing(&r));
}

#[test]
fn lint_paths_reports_unknown_tags_with_file_and_line() {
    let temp = Temp::new();
    for name in ["hosts.toml", "tags.toml"] {
        fs::copy(Path::new(CATALOG).join(name), temp.0.join(name)).unwrap();
    }
    let mut r = recipe("sigil");
    r.tags = vec!["episode".into(), "bogus".into()];
    let text = toml::to_string_pretty(&r).unwrap();
    let expected = text.lines().position(|l| l.contains("\"bogus\"")).unwrap() + 1;
    fs::write(temp.0.join("sigil.toml"), &text).unwrap();
    let errors = lint_paths(
        std::slice::from_ref(&temp.0),
        &temp.0.join("hosts.toml"),
        None,
    );
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].message.contains("unknown tag `bogus`"));
    assert_eq!(errors[0].line, expected, "{errors:?}");
    assert!(errors[0].hint.contains("tags.toml"));
    // tags.toml beside the recipes is never linted as a recipe.
    fs::write(
        temp.0.join("sigil.toml"),
        toml::to_string_pretty(&recipe("sigil")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        lint_paths(
            std::slice::from_ref(&temp.0),
            &temp.0.join("hosts.toml"),
            None
        ),
        vec![]
    );
}

#[test]
fn tags_round_trip_and_are_optional_in_schema_one() {
    let mut r = recipe("sigil");
    r.tags = vec!["episode".into(), "challenging".into()];
    let text = toml::to_string_pretty(&r).unwrap();
    assert!(text.contains("tags = ["), "{text}");
    assert_eq!(toml::from_str::<Recipe>(&text).unwrap(), r);
    r.tags.clear();
    assert!(!toml::to_string_pretty(&r).unwrap().contains("tags"));
    // JSON state written before tags existed still loads (installed snapshots).
    let json = serde_json::to_string(&r).unwrap();
    assert!(!json.contains("tags"));
    assert_eq!(serde_json::from_str::<Recipe>(&json).unwrap(), r);
}

#[test]
fn loader_skips_recipes_with_unknown_tags_and_keeps_known_ones() {
    let hosts_text = fs::read_to_string(format!("{CATALOG}/hosts.toml")).unwrap();
    let tags_text = fs::read_to_string(format!("{CATALOG}/tags.toml")).unwrap();
    let good = fs::read_to_string(format!("{CATALOG}/sigil.toml")).unwrap();
    let with = |tags: &str, file: &str| {
        let line = good.lines().find(|l| l.starts_with("tags = ")).unwrap();
        (
            file.to_owned(),
            good.replace(line, &format!("tags = {tags}")),
        )
    };
    let (f1, tagged) = with("[\"episode\"]", "sigil.toml");
    let (f2, unknown) = with("[\"nope\"]", "sigil-ii.toml");
    let unknown = unknown.replace("id = \"sigil\"", "id = \"sigil-ii\"");
    let catalog = Catalog::from_sources(
        &hosts_text,
        &tags_text,
        &[
            (f1.as_str(), tagged.as_str()),
            (f2.as_str(), unknown.as_str()),
        ],
    );
    assert_eq!(catalog.recipe("sigil").unwrap().tags, ["episode"]);
    assert!(catalog.recipe("sigil-ii").is_none());
    assert!(
        catalog.skipped[0].reason.contains("unknown tag `nope`"),
        "{:?}",
        catalog.skipped
    );
    // A broken vocabulary is reported and leaves tagged recipes skipped.
    let broken =
        Catalog::from_sources(&hosts_text, "not toml [", &[(f1.as_str(), tagged.as_str())]);
    assert!(broken.skipped.iter().any(|s| s.file == "tags.toml"));
    assert!(broken.recipe("sigil").is_none());
}

/// A header-only JPEG good enough for `image::image_info`; DDL never decodes
/// pixels in lint, so a fabricated SOF0 segment is a valid fixture.
fn jpeg_header(width: u16, height: u16) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xD8];
    bytes.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x11, 0x08]);
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&[3, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1, 0xFF, 0xD9]);
    bytes
}

fn sources_toml(credit: &str) -> String {
    format!(
        "schema_version = 1\n\
         [[screenshot]]\n\
         file = \"01.jpg\"\n\
         source_url = \"https://example.invalid/01.jpg\"\n\
         source_page = \"https://example.invalid/\"\n\
         credit = \"{credit}\"\n\
         original_size = \"640x360\"\n\
         retrieved = \"2026-10-17\"\n\
         permission = \"none found\"\n"
    )
}

/// Lays out `<repo>/catalog/sigil.toml` plus the screenshots tree and returns
/// the repository root.
fn screenshot_repo(sources: Option<&str>) -> Temp {
    let temp = Temp::new();
    let mut r = recipe("sigil");
    r.screenshots = vec![Screenshot {
        asset: "sigil/01.jpg".into(),
        credit: "Test author".into(),
    }];
    fs::create_dir_all(temp.0.join("catalog")).unwrap();
    fs::write(
        temp.0.join("catalog/sigil.toml"),
        toml::to_string(&r).unwrap(),
    )
    .unwrap();
    let dir = temp.0.join("crates/ddl/ui/assets/screenshots/sigil");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("01.jpg"), jpeg_header(640, 360)).unwrap();
    if let Some(sources) = sources {
        fs::write(dir.join("SOURCES.toml"), sources).unwrap();
    }
    temp
}

fn lint_screenshot_repo(temp: &Temp) -> Vec<Diagnostic> {
    lint_paths(
        &[temp.0.join("catalog")],
        &Path::new(CATALOG).join("hosts.toml"),
        None,
    )
}

#[test]
fn screenshots_lint_accepts_a_carded_asset_with_sources() {
    let temp = screenshot_repo(Some(&sources_toml("Test author")));
    assert_eq!(lint_screenshot_repo(&temp), vec![]);
}

/// A header-only PNG: signature, IHDR length, `IHDR`, width and height.
fn png_header(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    bytes.extend_from_slice(&13u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 2, 0, 0, 0, 0, 0, 0, 0]);
    bytes
}

/// Lints a repository whose single screenshot has the given bytes.
fn lint_screenshot_bytes(bytes: Vec<u8>) -> Vec<Diagnostic> {
    let temp = screenshot_repo(Some(&sources_toml("Test author")));
    fs::write(
        temp.0.join("crates/ddl/ui/assets/screenshots/sigil/01.jpg"),
        bytes,
    )
    .unwrap();
    lint_screenshot_repo(&temp)
}

#[test]
fn screenshots_lint_enforces_jpeg_1280x800_and_the_byte_cap() {
    // Both edges of the dimension bound and the byte cap pass.
    assert_eq!(lint_screenshot_bytes(jpeg_header(1280, 800)), vec![]);
    let mut at_cap = jpeg_header(1280, 800);
    at_cap.resize(192 * 1024, 0);
    assert_eq!(lint_screenshot_bytes(at_cap), vec![]);

    // A PNG is rejected, even at an accepted size, under a `.jpg` name.
    let errors = lint_screenshot_bytes(png_header(640, 360));
    assert!(
        errors.iter().any(|e| e.message.contains("must be JPEG")),
        "{errors:?}"
    );

    // An oversize JPEG is rejected on either axis.
    for (width, height) in [(1281, 800), (1280, 801), (1920, 1080)] {
        let errors = lint_screenshot_bytes(jpeg_header(width, height));
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("larger than 1280x800")),
            "{width}x{height}: {errors:?}"
        );
    }

    // A JPEG one byte over the cap is rejected.
    let mut over = jpeg_header(1280, 800);
    over.resize(192 * 1024 + 1, 0);
    let errors = lint_screenshot_bytes(over);
    assert!(
        errors.iter().any(|e| e.message.contains("over the")),
        "{errors:?}"
    );
}

#[test]
fn screenshots_lint_rejects_non_jpeg_asset_names() {
    for name in ["sigil/01.png", "sigil/01.webp"] {
        let mut r = recipe("sigil");
        r.screenshots = vec![Screenshot {
            asset: name.into(),
            credit: "Test author".into(),
        }];
        let errors = validate(&r, Path::new("sigil.toml"), &hosts());
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("invalid or duplicate screenshot asset")),
            "{name}: {errors:?}"
        );
    }
}

#[test]
fn screenshots_lint_reports_missing_oversize_small_and_uncredited_assets() {
    // A referenced asset that is not committed.
    let temp = screenshot_repo(None);
    fs::remove_file(temp.0.join("crates/ddl/ui/assets/screenshots/sigil/01.jpg")).unwrap();
    let errors = lint_screenshot_repo(&temp);
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("missing screenshot")),
        "{errors:?}"
    );

    // Over the 192 KiB hard bound.
    let temp = screenshot_repo(Some(&sources_toml("Test author")));
    let mut big = jpeg_header(640, 360);
    big.resize(300 * 1024, 0);
    fs::write(
        temp.0.join("crates/ddl/ui/assets/screenshots/sigil/01.jpg"),
        big,
    )
    .unwrap();
    let errors = lint_screenshot_repo(&temp);
    assert!(
        errors.iter().any(|e| e.message.contains("over the")),
        "{errors:?}"
    );

    // Below the 640x360 card floor.
    let temp = screenshot_repo(Some(&sources_toml("Test author")));
    fs::write(
        temp.0.join("crates/ddl/ui/assets/screenshots/sigil/01.jpg"),
        jpeg_header(320, 200),
    )
    .unwrap();
    let errors = lint_screenshot_repo(&temp);
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("smaller than 640x360")),
        "{errors:?}"
    );

    // A committed image with no SOURCES.toml credit entry.
    let temp = screenshot_repo(None);
    let errors = lint_screenshot_repo(&temp);
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("no SOURCES.toml entry")),
        "{errors:?}"
    );

    // A SOURCES.toml entry with an empty credit is rejected too.
    let temp = screenshot_repo(Some(&sources_toml("")));
    let errors = lint_screenshot_repo(&temp);
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("no SOURCES.toml entry")),
        "{errors:?}"
    );
}

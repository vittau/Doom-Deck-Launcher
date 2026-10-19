//! Resolver tests (spec 009, A3/A4) against the three reference recipes.
//!
//! No engine runs and no network is touched: installed members are small local
//! fixtures whose recorded size and SHA-256 the resolver verifies, and the IWAD
//! registry points at local files.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::*;
use crate::catalog::{Catalog, Complevel as RecipeComplevel, ComplevelAlias, Game, Recipe};
use crate::downloads::Sha256;
use crate::dsda_config::DsdaConfigManager;
use crate::engines::dsda::{AssignOverride, Warp};
use crate::iwads::{
    FileTime, Game as IwadsGame, Registry, RegistryEntry, Source, identify::sha256_file,
};
use crate::library::{InstallManifest, InstalledMember, SCHEMA_VERSION};
use crate::profiles::Profile;

fn embedded(id: &str) -> Recipe {
    Catalog::embedded()
        .recipe(id)
        .expect("reference recipe")
        .clone()
}

fn registry_entry(path: &Path, variant: &str, game: IwadsGame) -> RegistryEntry {
    let metadata = fs::metadata(path).expect("metadata");
    RegistryEntry {
        path: path.to_path_buf(),
        variant: Some(variant.to_owned()),
        candidates: Vec::new(),
        game: Some(game),
        sha256: sha256_file(path).expect("hash"),
        size: metadata.len(),
        mtime: FileTime::from_metadata(&metadata).expect("mtime"),
        source: Source::Manual,
    }
}

/// Writes a local IWAD fixture and returns a registry holding it.
fn iwad_registry(root: &Path, variant: &str, game: IwadsGame, bytes: &[u8]) -> Registry {
    let path = root.join(format!("{variant}.wad"));
    fs::write(&path, bytes).expect("write iwad");
    let mut registry = Registry::new();
    registry.insert(registry_entry(&path, variant, game));
    registry
}

fn engine(root: &Path) -> (Engine, PathBuf) {
    let path = root.join("dsda-doom");
    fs::write(&path, b"#!/bin/sh\n").expect("write engine");
    (
        Engine {
            executable: path.clone(),
            version: "0.30.1".to_owned(),
        },
        path,
    )
}

fn empty_manifest(recipe: &Recipe) -> InstallManifest {
    InstallManifest {
        schema_version: SCHEMA_VERSION,
        recipe_id: recipe.id.clone(),
        release: recipe.release.clone(),
        recipe_sha256: "a".repeat(64),
        installed_at: 0,
        packages: Vec::new(),
        members: Vec::new(),
    }
}

/// Places a small downloaded member and records it in the manifest.
fn add_member(
    data: &Path,
    manifest: &mut InstallManifest,
    recipe: &Recipe,
    package: &str,
    member: &str,
    bytes: &[u8],
) -> PathBuf {
    let path = data
        .join("library")
        .join(&recipe.id)
        .join(&recipe.release)
        .join(package)
        .join(format!("{member}.wad"));
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(&path, bytes).expect("write member");
    manifest.members.push(InstalledMember {
        package: Some(package.to_owned()),
        member: Some(member.to_owned()),
        component: None,
        path: path.clone(),
        size: bytes.len() as u64,
        sha256: Sha256::of(bytes).to_hex(),
        keep: false,
    });
    path
}

/// Records a user-supplied import.
fn add_import(
    data: &Path,
    manifest: &mut InstallManifest,
    recipe: &Recipe,
    component: &str,
    bytes: &[u8],
) -> PathBuf {
    let path = data
        .join("library")
        .join(&recipe.id)
        .join(&recipe.release)
        .join("imports")
        .join(component)
        .join(format!("{component}.wad"));
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(&path, bytes).expect("write import");
    manifest.members.push(InstalledMember {
        package: None,
        member: None,
        component: Some(component.to_owned()),
        path: path.clone(),
        size: bytes.len() as u64,
        sha256: Sha256::of(bytes).to_hex(),
        keep: false,
    });
    path
}

fn config(root: &Path) -> DsdaConfigManager {
    DsdaConfigManager::new(root.join("config")).expect("config manager")
}

/// Replaces the temporary root in a rendered summary with `<data>` so snapshots
/// are stable and readable.
fn normalize(text: &str, root: &Path) -> String {
    text.replace(&root.display().to_string(), "<data>")
}

#[test]
fn auger_zenith_order_is_main_then_addon_and_exclusive_groups_are_enforced() {
    let dir = TempDir::new().unwrap();
    let data = dir.path();
    let recipe = embedded("auger-zenith");
    let registry = iwad_registry(data, "doom2-1.9", IwadsGame::Doom2, b"doom2 bytes");
    let (engine, _) = engine(data);
    let config = config(data);

    let mut manifest = empty_manifest(&recipe);
    let maps = add_member(data, &mut manifest, &recipe, "free", "maps", b"main maps");
    let alternate = add_member(
        data,
        &mut manifest,
        &recipe,
        "free",
        "alternate",
        b"alternate blaster",
    );
    let deckard = add_member(
        data,
        &mut manifest,
        &recipe,
        "free",
        "deckard",
        b"deckard blaster",
    );

    // Main only.
    let profile = Profile::new("p1", "Play", "auger-zenith");
    let resolution = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve main");
    assert_eq!(resolution.plan().files, vec![maps.clone()]);
    assert_eq!(resolution.summary().content.len(), 1);
    assert_eq!(resolution.summary().content[0].position, 1);

    // With each add-on: main first, then the add-on.
    for (component, addon) in [
        ("alternate-blaster", &alternate),
        ("deckard-blaster", &deckard),
    ] {
        let profile = Profile {
            components: vec![component.to_owned()],
            ..Profile::new("p1", "Play", "auger-zenith")
        };
        let resolution = resolve(
            &recipe,
            &profile,
            &Iwads::new(data, &registry),
            &manifest,
            Some(&engine),
            &config,
        )
        .expect("resolve addon");
        assert_eq!(resolution.plan().files, vec![maps.clone(), addon.clone()]);
        assert_eq!(resolution.summary().content[1].position, 2);
    }

    // Two members of the exclusive blaster group cannot be enabled together.
    let profile = Profile {
        components: vec!["alternate-blaster".to_owned(), "deckard-blaster".to_owned()],
        ..Profile::new("p1", "Play", "auger-zenith")
    };
    let blocker = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect_err("exclusive group");
    assert!(
        matches!(blocker, Blocker::IncompatibleChoice { .. }),
        "{blocker}"
    );
    assert!(blocker.to_string().contains("group"), "{blocker}");
}

#[test]
fn widescreen_status_bar_loads_last_after_the_mapset() {
    let dir = TempDir::new().unwrap();
    let data = dir.path();
    let recipe = embedded("eviternity");
    let registry = iwad_registry(data, "doom2-1.9", IwadsGame::Doom2, b"doom2 bytes");
    let (engine, _) = engine(data);
    let config = config(data);

    let mut manifest = empty_manifest(&recipe);
    let maps = add_member(
        data,
        &mut manifest,
        &recipe,
        "free",
        "maps",
        b"eviternity maps",
    );
    let bar = add_member(
        data,
        &mut manifest,
        &recipe,
        "widehud",
        "statusbar",
        b"widescreen bar",
    );

    let profile = Profile {
        components: vec!["wide-status-bar".to_owned()],
        ..Profile::new("p1", "Play", "eviternity")
    };
    let resolution = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve with the wide bar");
    // The single `-file` list is main first, the wide bar last (spec 017, R1).
    assert_eq!(resolution.plan().files, vec![maps.clone(), bar.clone()]);
    let argv = resolution.plan().to_command();
    assert_eq!(argv.iter().filter(|arg| *arg == "-file").count(), 1);
    let files = argv.iter().position(|arg| arg == "-file").unwrap();
    assert_eq!(argv[files + 1], maps.as_os_str());
    assert_eq!(argv[files + 2], bar.as_os_str());
    assert_eq!(argv[files + 3], "-complevel");
    let last = resolution.summary().content.last().expect("content");
    assert_eq!(last.position, 2);
    assert_eq!(last.label, "Widescreen status bar");
    assert_eq!(last.path, bar);
}

#[test]
fn iwad_status_bar_coexists_with_main_and_the_edition_replacement() {
    let dir = TempDir::new().unwrap();
    let data = dir.path();
    let recipe = embedded("sigil");
    let registry = iwad_registry(data, "doom-1.9ud", IwadsGame::Doom, b"doomu bytes");
    let (engine, _) = engine(data);
    let config = config(data);

    let mut manifest = empty_manifest(&recipe);
    let maps = add_member(data, &mut manifest, &recipe, "free", "maps", b"sigil main");
    let compat = add_member(
        data,
        &mut manifest,
        &recipe,
        "free",
        "compat",
        b"sigil compat",
    );
    let bar = add_member(
        data,
        &mut manifest,
        &recipe,
        "widehud",
        "statusbar",
        b"widescreen bar",
    );

    // Main then the base-game IWAD bar.
    let profile = Profile {
        components: vec!["wide-status-bar".to_owned()],
        ..Profile::new("p1", "Play", "sigil")
    };
    let resolution = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve with the IWAD bar");
    assert_eq!(resolution.plan().files, vec![maps.clone(), bar.clone()]);

    // The compat edition replaces main; the IWAD bar still loads last.
    let profile = Profile {
        components: vec!["compat".to_owned(), "wide-status-bar".to_owned()],
        ..Profile::new("p1", "Play", "sigil")
    };
    let resolution = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve compat + IWAD bar");
    assert_eq!(resolution.plan().files, vec![compat, bar]);
    assert!(!resolution.plan().files.contains(&maps));
}

#[test]
fn sigil_compat_replaces_main_and_the_missing_soundtrack_blocks_only_when_enabled() {
    let dir = TempDir::new().unwrap();
    let data = dir.path();
    let recipe = embedded("sigil");
    let registry = iwad_registry(data, "doom-1.9ud", IwadsGame::Doom, b"doomu bytes");
    let (engine, _) = engine(data);
    let config = config(data);

    let mut manifest = empty_manifest(&recipe);
    let maps = add_member(data, &mut manifest, &recipe, "free", "maps", b"sigil main");
    let compat = add_member(
        data,
        &mut manifest,
        &recipe,
        "free",
        "compat",
        b"sigil compat",
    );

    // Main only: the free WAD loads.
    let profile = Profile::new("p1", "Play", "sigil");
    let resolution = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve main");
    assert_eq!(resolution.plan().files, vec![maps.clone()]);

    // Compat replaces main, never loads beside it.
    let profile = Profile {
        components: vec!["compat".to_owned()],
        ..Profile::new("p1", "Play", "sigil")
    };
    let resolution = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve compat");
    assert_eq!(resolution.plan().files, vec![compat.clone()]);
    assert!(!resolution.plan().files.contains(&maps));

    // The paid soundtrack is user-supplied: only blocking when enabled.
    let profile = Profile::new("p1", "Play", "sigil");
    assert!(
        resolve(
            &recipe,
            &profile,
            &Iwads::new(data, &registry),
            &manifest,
            Some(&engine),
            &config
        )
        .is_ok(),
        "a disabled user-supplied component is not a blocker"
    );

    let profile = Profile {
        components: vec!["soundtrack".to_owned()],
        ..Profile::new("p1", "Play", "sigil")
    };
    let blocker = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect_err("soundtrack missing");
    match blocker {
        Blocker::ComponentNotProvided { component, .. } => assert_eq!(component, "soundtrack"),
        other => panic!("expected ComponentNotProvided, got {other}"),
    }

    // Importing it clears the blocker.
    let profile = Profile {
        components: vec!["soundtrack".to_owned()],
        ..Profile::new("p1", "Play", "sigil")
    };
    add_import(
        data,
        &mut manifest,
        &recipe,
        "soundtrack",
        b"paid soundtrack",
    );
    let resolution = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve with soundtrack");
    assert_eq!(resolution.plan().files.len(), 1);
    assert!(
        resolution.summary().content[0]
            .path
            .ends_with("soundtrack.wad")
    );
}

#[test]
fn missing_iwad_names_the_game_and_vanilla_resolves_per_family() {
    let dir = TempDir::new().unwrap();
    let data = dir.path();
    let (engine, _) = engine(data);
    let config = config(data);

    // No IWAD at all.
    let recipe = embedded("auger-zenith");
    let registry = Registry::new();
    let mut manifest = empty_manifest(&recipe);
    add_member(data, &mut manifest, &recipe, "free", "maps", b"maps");
    let profile = Profile::new("p1", "Play", "auger-zenith");
    let blocker = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect_err("no iwad");
    assert_eq!(
        blocker,
        Blocker::MissingIwad {
            game: Game::Doom2,
            accepted: Vec::new(),
        }
    );
    assert!(blocker.to_string().contains("doom2"), "{blocker}");

    // `vanilla` is resolved from the identified family (D-017).
    let cases = [
        ("doom2-1.9", IwadsGame::Doom2, Game::Doom2, 2u8),
        ("doom-1.9ud", IwadsGame::Doom, Game::Doom, 3u8),
        ("tnt-1.9", IwadsGame::Tnt, Game::Tnt, 4u8),
    ];
    for (variant, iwad_game, recipe_game, expected) in cases {
        let mut recipe = embedded("auger-zenith");
        recipe.base_game.game = recipe_game;
        recipe.base_game.variants.clear();
        recipe.compatibility.complevel = RecipeComplevel::Alias(ComplevelAlias::Vanilla);

        let root = TempDir::new().unwrap();
        let registry = iwad_registry(root.path(), variant, iwad_game, b"iwad");
        let mut manifest = empty_manifest(&recipe);
        add_member(root.path(), &mut manifest, &recipe, "free", "maps", b"maps");
        let profile = Profile::new("p1", "Play", "auger-zenith");
        let resolution = resolve(
            &recipe,
            &profile,
            &Iwads::new(root.path(), &registry),
            &manifest,
            Some(&engine),
            &config,
        )
        .expect("resolve");
        assert_eq!(resolution.summary().complevel, expected, "{variant}");
        assert_eq!(
            resolution
                .plan()
                .complevel
                .expect("a catalog launch pins a complevel")
                .value(),
            expected
        );
    }
}

#[test]
fn blockers_cover_not_installed_engine_missing_integrity_and_bad_overrides() {
    let dir = TempDir::new().unwrap();
    let data = dir.path();
    let recipe = embedded("auger-zenith");
    let registry = iwad_registry(data, "doom2-1.9", IwadsGame::Doom2, b"doom2 bytes");
    let (engine, _) = engine(data);
    let config = config(data);
    let profile = Profile::new("p1", "Play", "auger-zenith");
    let iwads = Iwads::new(data, &registry);

    // Not installed: the main member is absent from the manifest.
    let manifest = empty_manifest(&recipe);
    assert!(matches!(
        resolve(&recipe, &profile, &iwads, &manifest, Some(&engine), &config),
        Err(Blocker::NotInstalled)
    ));

    // Engine missing.
    let mut manifest = empty_manifest(&recipe);
    add_member(data, &mut manifest, &recipe, "free", "maps", b"maps");
    assert!(matches!(
        resolve(&recipe, &profile, &iwads, &manifest, None, &config),
        Err(Blocker::EngineMissing)
    ));

    // Integrity failed: the recorded file no longer exists.
    let mut manifest = empty_manifest(&recipe);
    let path = add_member(data, &mut manifest, &recipe, "free", "maps", b"maps");
    fs::remove_file(&path).unwrap();
    assert!(matches!(
        resolve(&recipe, &profile, &iwads, &manifest, Some(&engine), &config),
        Err(Blocker::IntegrityFailed { .. })
    ));

    // A tampered file fails its hash check.
    let mut manifest = empty_manifest(&recipe);
    let path = add_member(data, &mut manifest, &recipe, "free", "maps", b"maps");
    fs::write(&path, b"tampered but same length!!").unwrap();
    assert!(matches!(
        resolve(&recipe, &profile, &iwads, &manifest, Some(&engine), &config),
        Err(Blocker::IntegrityFailed { .. })
    ));

    // An override key outside spec 005's allowlist is an incompatible choice.
    let mut manifest = empty_manifest(&recipe);
    add_member(data, &mut manifest, &recipe, "free", "maps", b"maps");
    let mut profile = Profile::new("p1", "Play", "auger-zenith");
    profile.overrides.insert("input_fire".to_owned(), 1);
    assert!(matches!(
        resolve(&recipe, &profile, &iwads, &manifest, Some(&engine), &config),
        Err(Blocker::IncompatibleChoice { .. })
    ));
}

#[test]
fn warp_pistol_start_and_assign_overrides_reach_the_plan() {
    let dir = TempDir::new().unwrap();
    let data = dir.path();
    let mut recipe = embedded("auger-zenith");
    recipe.compatibility.runtime.warp = Some(vec![5, 1]);
    let registry = iwad_registry(data, "doom2-1.9", IwadsGame::Doom2, b"doom2 bytes");
    let (engine, _) = engine(data);
    let config = config(data);
    let mut manifest = empty_manifest(&recipe);
    add_member(data, &mut manifest, &recipe, "free", "maps", b"maps");

    let mut profile = Profile::new("p1", "Play", "auger-zenith");
    profile.pistol_start = true;
    profile.overrides.insert("allow_freelook".to_owned(), 1);
    let resolution = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve");
    let plan = resolution.plan();
    assert_eq!(plan.warp, Some(Warp::EpisodeMap(5, 1)));
    assert!(plan.pistol_start);
    assert_eq!(
        plan.assigns,
        vec![AssignOverride::new("allow_freelook", "1")]
    );
}

#[test]
fn prepare_creates_directories_and_the_canonical_link_and_then_validates() {
    let dir = TempDir::new().unwrap();
    let data = dir.path();
    let recipe = embedded("auger-zenith");
    let registry = iwad_registry(data, "doom2-1.9", IwadsGame::Doom2, b"doom2 bytes");
    let (engine, _) = engine(data);
    let config = config(data);
    let mut manifest = empty_manifest(&recipe);
    add_member(data, &mut manifest, &recipe, "free", "maps", b"maps");

    let profile = Profile::new("p1", "Play", "auger-zenith");
    let resolution = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve");

    // Resolution is pure: nothing was written yet.
    assert!(!resolution.plan().iwad.exists());
    assert!(!resolution.plan().save_dir.exists());

    resolution.prepare().expect("prepare");
    assert_eq!(
        resolution.plan().iwad,
        data.join("iwads/doom2-1.9/doom2.wad")
    );
    assert!(resolution.plan().save_dir.is_dir());
    assert!(resolution.plan().data_dir.is_dir());
    assert!(resolution.plan().shot_dir.is_dir());
    assert!(resolution.plan().working_dir.is_dir());
    resolution.validate().expect("valid plan");

    // Preparing again is idempotent.
    resolution.prepare().expect("prepare again");
}

#[test]
fn resolver_prefers_the_classic_steam_copy_over_the_kex_copy() {
    // Spec 028, R3/A2: with both the Deck's `base/` and `rerelease/` copies
    // registered, SIGIL and AUGER;ZENITH launch from `base/`.
    let dir = TempDir::new().unwrap();
    let data = dir.path();
    let install = data.join("steam/steamapps/common/Ultimate Doom");
    let base_doom = install.join("base/DOOM.WAD");
    let kex_doom = install.join("rerelease/doom.wad");
    let base_doom2 = install.join("base/doom2/DOOM2.WAD");
    let kex_doom2 = install.join("rerelease/doom2.wad");
    for path in [&base_doom, &kex_doom, &base_doom2, &kex_doom2] {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, path.to_string_lossy().as_bytes()).unwrap();
    }

    let mut registry = Registry::new();
    // KEX first: ranking, not insertion order, must decide.
    registry.insert(registry_entry(
        &kex_doom2,
        "doom2-1.9-kex-2024-10-01",
        IwadsGame::Doom2,
    ));
    registry.insert(registry_entry(
        &kex_doom,
        "doom-1.9ud-kex-2024-10-01",
        IwadsGame::Doom,
    ));
    registry.insert(registry_entry(&base_doom2, "doom2-1.9", IwadsGame::Doom2));
    registry.insert(registry_entry(&base_doom, "doom-1.9ud", IwadsGame::Doom));
    let (engine, _) = engine(data);
    let config = config(data);

    let sigil = embedded("sigil");
    let mut sigil_manifest = empty_manifest(&sigil);
    add_member(
        data,
        &mut sigil_manifest,
        &sigil,
        "free",
        "maps",
        b"sigil main",
    );
    let resolution = resolve(
        &sigil,
        &Profile::new("p1", "Play", "sigil"),
        &Iwads::new(data, &registry),
        &sigil_manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve SIGIL");
    resolution.prepare().expect("prepare SIGIL");
    assert_eq!(
        fs::canonicalize(&resolution.plan().iwad).unwrap(),
        fs::canonicalize(&base_doom).unwrap(),
        "SIGIL must launch from base/DOOM.WAD"
    );

    let auger = embedded("auger-zenith");
    let mut auger_manifest = empty_manifest(&auger);
    add_member(
        data,
        &mut auger_manifest,
        &auger,
        "free",
        "maps",
        b"auger main",
    );
    let resolution = resolve(
        &auger,
        &Profile::new("p1", "Play", "auger-zenith"),
        &Iwads::new(data, &registry),
        &auger_manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve AUGER;ZENITH");
    resolution.prepare().expect("prepare AUGER;ZENITH");
    assert_eq!(
        fs::canonicalize(&resolution.plan().iwad).unwrap(),
        fs::canonicalize(&base_doom2).unwrap(),
        "AUGER;ZENITH must launch from base/doom2/DOOM2.WAD"
    );
}

#[test]
fn summary_snapshot_for_the_three_reference_recipes() {
    // Each case: recipe id, iwad variant + game, the main member path.
    let cases = [
        (
            "auger-zenith",
            "doom2-1.9",
            IwadsGame::Doom2,
            "doom2.wad",
            "AUGER;ZENITH (Doomer Boards Project 37)",
        ),
        ("sigil", "doom-1.9ud", IwadsGame::Doom, "doomu.wad", "SIGIL"),
        (
            "sigil-ii",
            "doom-1.9ud",
            IwadsGame::Doom,
            "doomu.wad",
            "SIGIL II",
        ),
    ];
    for (id, variant, iwad_game, filename, label) in cases {
        let dir = TempDir::new().unwrap();
        let data = dir.path();
        let recipe = embedded(id);
        let registry = iwad_registry(data, variant, iwad_game, b"iwad bytes");
        let (engine, _) = engine(data);
        let config = config(data);
        let mut manifest = empty_manifest(&recipe);
        add_member(data, &mut manifest, &recipe, "free", "maps", b"maps");

        let profile = Profile::new("p1", "Play", id);
        let resolution = resolve(
            &recipe,
            &profile,
            &Iwads::new(data, &registry),
            &manifest,
            Some(&engine),
            &config,
        )
        .expect("resolve");
        let text = normalize(&resolution.summary().to_string(), data);
        let expected = format!(
            "Will launch\n  base game: {} (present)\n  IWAD: <data>/iwads/{variant}/{filename} ({variant})\n  content (load order):\n    1. {label}\n       <data>/library/{id}/{release}/free/maps.wad\n  complevel: {complevel} (proposed)\n  skill: 3\n  pistol start: no\n  engine: 0.30.1",
            crate::resolver::game_label(recipe.base_game.game),
            release = recipe.release,
            complevel = match &recipe.compatibility.complevel {
                RecipeComplevel::Number(number) => *number,
                _ => 0,
            },
        );
        assert_eq!(text, expected, "{id}");
    }
}

/// Reads a small user file fixture of the given content kind.
fn user_file(dir: &Path, name: &str, bytes: &[u8]) -> crate::local::UserFile {
    let path = dir.join(name);
    fs::write(&path, bytes).expect("write user file");
    crate::local::UserFile::read(&path).expect("accepted user file")
}

/// A minimal valid PWAD with the given lump names; no real game data.
fn pwad(lumps: &[&str]) -> Vec<u8> {
    let mut directory = Vec::new();
    for name in lumps {
        directory.extend_from_slice(&0_i32.to_le_bytes());
        directory.extend_from_slice(&0_i32.to_le_bytes());
        let mut padded = [0_u8; 8];
        let len = name.len().min(8);
        padded[..len].copy_from_slice(&name.as_bytes()[..len]);
        directory.extend_from_slice(&padded);
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"PWAD");
    out.extend_from_slice(&(lumps.len() as i32).to_le_bytes());
    out.extend_from_slice(&12_i32.to_le_bytes());
    out.extend_from_slice(&directory);
    out
}

#[test]
fn profile_extras_load_after_the_recipe_files_and_patches() {
    // Spec 026, A2/R3/R5: the recipe's verified order is never changed; extras
    // go after it, WAD/ZIP in list order into `-file`, DEH/BEX into `-deh`.
    let dir = TempDir::new().unwrap();
    let data = dir.path();
    let recipe = embedded("alien-vendetta");
    let registry = iwad_registry(data, "doom2-1.9", IwadsGame::Doom2, b"doom2 bytes");
    let (engine, _) = engine(data);
    let config = config(data);

    let mut manifest = empty_manifest(&recipe);
    let maps = add_member(data, &mut manifest, &recipe, "free", "maps", b"main maps");
    let patch = add_member(
        data,
        &mut manifest,
        &recipe,
        "free",
        "patch",
        b"recipe patch",
    );

    let extra_wad = user_file(data, "extra.wad", &pwad(&["MAP01"]));
    let extra_deh = user_file(
        data,
        "extra.deh",
        b"Patch File for DeHackEd v3.0\nThing 1\nHit points = 30\n",
    );

    let profile = Profile {
        extras: vec![extra_wad.clone(), extra_deh.clone()],
        ..Profile::new("p1", "Play", "alien-vendetta")
    };
    let resolution = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect("resolve with extras");
    assert_eq!(
        resolution.plan().files,
        vec![maps.clone(), extra_wad.path.clone()],
        "recipe files come first, extras after"
    );
    assert_eq!(
        resolution.plan().patches,
        vec![patch.clone(), extra_deh.path.clone()],
        "recipe patches come first, extra patches after"
    );
    assert_eq!(
        resolution.summary().content.len(),
        2,
        "the summary lists the extra file too"
    );
    assert_eq!(resolution.summary().content[1].label, "Extra file");

    // A missing extra blocks the launch and is named (R4).
    fs::remove_file(&extra_wad.path).unwrap();
    let never = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect_err("missing extra");
    match never {
        Blocker::UserFileMissing { path } => assert_eq!(path, extra_wad.path),
        other => panic!("expected UserFileMissing, got {other}"),
    }

    // A changed extra blocks until the player confirms (R4).
    let changed = user_file(data, "changed.wad", &pwad(&["MAP01"]));
    fs::write(&changed.path, pwad(&["MAP02", "MAP03"])).unwrap();
    let profile = Profile {
        extras: vec![changed.clone()],
        ..Profile::new("p1", "Play", "alien-vendetta")
    };
    let blocked = resolve(
        &recipe,
        &profile,
        &Iwads::new(data, &registry),
        &manifest,
        Some(&engine),
        &config,
    )
    .expect_err("changed extra");
    assert!(
        matches!(blocked, Blocker::UserFileChanged { .. }),
        "{blocked}"
    );
    assert!(blocked.to_string().contains("confirm"), "{blocked}");

    // Re-recording the change lets the same launch through.
    let mut extras = profile.extras.clone();
    crate::local::accept_changed(&mut extras).expect("accept the change");
    let profile = Profile {
        extras,
        ..Profile::new("p1", "Play", "alien-vendetta")
    };
    assert!(
        resolve(
            &recipe,
            &profile,
            &Iwads::new(data, &registry),
            &manifest,
            Some(&engine),
            &config,
        )
        .is_ok()
    );
}

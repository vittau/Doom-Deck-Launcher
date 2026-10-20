//! Focused spec 026 journeys: each owns scratch state and an off-screen renderer.
use super::*;
use crate::headless::{render, setup, write_png};
use ddl_core::{downloads::CancellationToken, profiles::ProfileStore};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

// Slint's platform is process-global. Separate named tests re-execute only their
// own case in a headless subprocess with a private artifact directory (spec 027).
fn isolated(name: &str) -> bool {
    crate::headless::isolated(module_path!(), name)
}
fn environment(home: &Path) -> Environment {
    fs::create_dir_all(home).unwrap();
    let paths = ddl_core::platform::paths::resolve(&|key: &str| {
        (key == "HOME").then(|| home.display().to_string())
    })
    .unwrap();
    Environment {
        paths,
        home: home.into(),
        test: None,
    }
    .without_catalog_auto_updates()
}
fn fixtures(home: &Path) -> Vec<PathBuf> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ddl-core/tests/fixtures/local");
    let mut paths = vec![];
    for name in ["tiny.wad", "tiny.zip", "tiny.deh"] {
        let path = home.join(name);
        fs::copy(source.join(name), &path).unwrap();
        paths.push(path);
    }
    fs::copy(source.join("tiny.pk3"), home.join("unsupported.pk3")).unwrap();
    fs::write(home.join("false.wad"), b"not a WAD").unwrap();
    let base = home.join("base.wad");
    fs::write(&base, include_bytes!("../tests/fixtures/local/base.wad")).unwrap();
    paths
}
fn services(env: &Environment) -> Services {
    let mut services = Services::open(env.clone()).unwrap();
    services
        .execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
        .unwrap();
    services
}
fn custom(env: &Environment) -> (Services, Vec<PathBuf>) {
    let files = fixtures(&env.home);
    let mut services = services(env);
    services
        .custom_create(
            "Night maps",
            Some("night-maps"),
            NewIwad::File(env.home.join("base.wad")),
            Compatibility::Auto,
        )
        .unwrap();
    services.custom_add_file("night-maps", &files).unwrap();
    (services, files)
}
fn wait(runtime: &mut Runtime, ui: &AppWindow) {
    let end = Instant::now() + Duration::from_secs(10);
    while runtime.busy {
        runtime.drain(ui);
        assert!(Instant::now() < end, "services worker timed out");
        std::thread::sleep(Duration::from_millis(2));
    }
    runtime.sync(ui);
}
fn act(runtime: &mut Runtime, ui: &AppWindow, action: Action) {
    runtime.action(ui, action);
    wait(runtime, ui);
}
fn click(runtime: &mut Runtime, ui: &AppWindow, id: i32) {
    runtime.click(ui, id);
    wait(runtime, ui);
}
fn runtime(env: &Environment, services: &Services, ui: &AppWindow) -> Runtime {
    let mut runtime = Runtime::new(env.clone());
    runtime.snapshot = Some(services.snapshot().unwrap());
    runtime.screen = Screen::Library;
    runtime.sync(ui);
    runtime
}

#[test]
fn local_new_setup_iwad_compatibility_and_content_filtered_multiselect() {
    if isolated("local_new_setup_iwad_compatibility_and_content_filtered_multiselect") {
        return;
    }
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let env = environment(&temp.path().join("home"));
    let files = fixtures(&env.home);
    fs::create_dir_all(env.home.join("Downloads")).unwrap();
    fs::create_dir_all(env.home.join("Documents")).unwrap();
    let s = services(&env);
    let mut r = runtime(&env, &s, &ui);
    act(&mut r, &ui, Action::Y);
    assert_eq!(r.local.view, Some(View::New));
    write_png("026-new-custom.png", &render(&window));
    click(&mut r, &ui, 1); // IWAD picker
    assert_eq!(r.local.view, Some(View::Iwad));
    write_png("026-iwad-picker.png", &render(&window));
    click(&mut r, &ui, 1); // local IWAD file
    assert_eq!(r.directory.as_ref().unwrap().path, env.home);
    let d = r.directory.as_ref().unwrap();
    assert!(d.shortcuts.contains(&env.home.join("Downloads")));
    assert!(d.shortcuts.contains(&env.home.join("Documents")));
    assert!(
        !d.entries
            .iter()
            .any(|(p, _)| p.ends_with("false.wad") || p.ends_with("unsupported.pk3"))
    );
    assert!(
        !r.nodes.iter().any(|n| n.label.contains("tiny.wad")),
        "IWAD-only picker hides PWADs"
    );
    let index = d
        .entries
        .iter()
        .position(|(p, _)| p.ends_with("base.wad"))
        .unwrap();
    click(&mut r, &ui, PICK + index as i32);
    assert!(r.local.iwad.is_some());
    assert_eq!(r.local.view, Some(View::New));
    click(&mut r, &ui, 2); // compatibility
    assert_eq!(r.local.view, Some(View::Compat));
    assert_eq!(compatibility_values().len(), 20);
    write_png("026-compatibility.png", &render(&window));
    click(&mut r, &ui, 10); // Auto
    click(&mut r, &ui, 3); // Create
    assert_eq!(r.local.view, Some(View::Detail));
    assert_eq!(r.snapshot.as_ref().unwrap().custom.len(), 1);
    click(&mut r, &ui, 2); // Order
    act(&mut r, &ui, Action::Y);
    r.focus = PICK;
    r.sync(&ui);
    for path in &files[..2] {
        let index = r
            .directory
            .as_ref()
            .unwrap()
            .entries
            .iter()
            .position(|(p, _)| p == path)
            .unwrap();
        while r.focus < PICK + index as i32 {
            act(&mut r, &ui, Action::Down);
        }
        assert_eq!(r.focus, PICK + index as i32);
        act(&mut r, &ui, Action::Confirm);
    }
    assert_eq!(r.local.selected.len(), 2);
    assert_eq!(ui.get_scroll_offset(), 0.);
    let frames: Vec<_> = r.nodes.iter().filter(|n| n.kind == 1).collect();
    assert_eq!(
        frames.len(),
        2,
        "each picker column has its own carved plate"
    );
    for row in r.nodes.iter().filter(|n| n.kind == 3) {
        assert!(
            frames.iter().any(|frame| row.x >= frame.x + 16.
                && row.x + row.w <= frame.x + frame.w - 16.
                && row.y >= frame.y + 16.
                && row.y + row.h <= frame.y + frame.h - 16.),
            "complete row stays inside its viewport"
        );
    }
    // Vertical navigation traverses entries beyond the visible page.
    let ids: Vec<_> = r
        .directory
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .enumerate()
        .map(|(index, _)| PICK + index as i32)
        .collect();
    r.focus = ids[0];
    r.sync(&ui);
    for id in ids.iter().skip(1) {
        act(&mut r, &ui, Action::Down);
        assert_eq!(r.focus, *id);
        assert!(r.nodes.iter().any(|n| n.id == *id));
    }
    // Return focus to the selected ZIP for the multi-select evidence frame.
    let zip = r
        .directory
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .position(|(p, _)| p == &files[1])
        .unwrap();
    while r.focus > ids[0] {
        act(&mut r, &ui, Action::Up);
    }
    while r.focus < PICK + zip as i32 {
        act(&mut r, &ui, Action::Down);
    }
    write_png("026-file-picker-multiselect.png", &render(&window));
    act(&mut r, &ui, Action::Menu); // Start confirms
    assert_eq!(r.local.view, Some(View::Order));
    assert_eq!(r.local_setup().unwrap().files.len(), 2);
    // B cancels a later picker without adding anything and restores the editor.
    act(&mut r, &ui, Action::Y);
    act(&mut r, &ui, Action::Back);
    assert_eq!(r.local.view, Some(View::Order));
    assert!(files.iter().all(|p| p.is_file()));
    act(&mut r, &ui, Action::Back); // custom detail
    click(&mut r, &ui, 4);
    click(&mut r, &ui, 29); // documented level 21
    assert_eq!(
        r.local_setup()
            .unwrap()
            .compatibility
            .level()
            .unwrap()
            .value(),
        21
    );
    click(&mut r, &ui, 4);
    click(&mut r, &ui, 10); // restore Auto
    assert_eq!(r.local_setup().unwrap().compatibility, Compatibility::Auto);
    // Use the existing explicit fake-registry seam; no game data is needed.
    let mut refreshed = Services::open(env.clone()).unwrap();
    refreshed
        .register_fake_iwads(&temp.path().join("identified"))
        .unwrap();
    r.snapshot = Some(refreshed.snapshot().unwrap());
    r.sync(&ui);
    let variant = r
        .snapshot
        .as_ref()
        .unwrap()
        .iwads
        .iter()
        .find_map(|e| e.variant.clone())
        .unwrap();
    click(&mut r, &ui, 3);
    write_png("026-iwad-picker.png", &render(&window));
    click(&mut r, &ui, 10);
    assert!(
        matches!(&r.local_setup().unwrap().iwad, CustomIwad::Identified { variant: selected } if selected == &variant)
    );
}

#[test]
fn local_order_moves_with_dpad_stays_in_group_and_removes_without_deleting() {
    if isolated("local_order_moves_with_dpad_stays_in_group_and_removes_without_deleting") {
        return;
    }
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let env = environment(&temp.path().join("home"));
    let (s, files) = custom(&env);
    let mut r = runtime(&env, &s, &ui);
    click(&mut r, &ui, CUSTOM);
    click(&mut r, &ui, 2);
    assert_eq!(r.focus, 1);
    act(&mut r, &ui, Action::Down);
    assert_eq!(r.focus, ENTRY);
    act(&mut r, &ui, Action::Confirm);
    act(&mut r, &ui, Action::Down);
    assert_eq!(r.local.moving, Some((Group::Files, 1)));
    assert_eq!(r.local_setup().unwrap().files[1].path, files[0]);
    write_png("026-order-mid-move.png", &render(&window));
    act(&mut r, &ui, Action::Down); // cannot cross into patches
    assert_eq!(r.local.moving, Some((Group::Files, 1)));
    act(&mut r, &ui, Action::Confirm);
    act(&mut r, &ui, Action::X);
    assert_eq!(r.local_setup().unwrap().files.len(), 1);
    assert!(files[0].is_file());
    assert_eq!(r.local_setup().unwrap().patches.len(), 1);
}

#[test]
fn local_custom_library_home_missing_changed_and_delete_dialog() {
    if isolated("local_custom_library_home_missing_changed_and_delete_dialog") {
        return;
    }
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let env = environment(&temp.path().join("home"));
    let (s, files) = custom(&env);
    let store = ProfileStore::new(&env.paths.config, &env.paths.data);
    let mut profile = store.load_optional("night-maps-default").unwrap().unwrap();
    profile.touch(1000);
    store.save(&profile).unwrap();
    let mut r = runtime(&env, &s, &ui);
    act(&mut r, &ui, Action::Down);
    assert_eq!(r.focus, CUSTOM);
    act(&mut r, &ui, Action::Up);
    assert_eq!(r.focus, NEW);
    write_png("026-library-custom.png", &render(&window));
    act(&mut r, &ui, Action::Tab(0));
    assert_eq!(r.focus, CUSTOM);
    assert_eq!(ui.get_band_art(), 4);
    assert!(ui.get_band_screenshot().is_empty());
    assert!(r.nodes.iter().any(|n| n.id == CUSTOM));
    write_png("026-home-custom.png", &render(&window));
    click(&mut r, &ui, CUSTOM);
    write_png("026-custom-detail.png", &render(&window));
    fs::remove_file(&files[0]).unwrap();
    r.snapshot = Some(Services::open(env.clone()).unwrap().snapshot().unwrap());
    r.sync(&ui);
    click(&mut r, &ui, 1);
    assert!(matches!(r.local.dialog, Some(Dialog::Issue(_, false))));
    assert!(
        ui.get_sheet_nodes()
            .iter()
            .any(|n| n.label.contains("tiny.wad"))
    );
    let sheet: Vec<_> = ui.get_sheet_nodes().iter().collect();
    assert!(sheet.iter().any(|n| n.kind == 20 && n.label == "tiny.wad"));
    assert!(
        sheet
            .iter()
            .any(|n| n.kind == 21 && n.label.starts_with('/') && !n.label.contains('`'))
    );
    assert!(
        sheet
            .iter()
            .any(|n| n.label == ui.global::<LocalStrings>().get_file_missing())
    );
    write_png("026-missing-file.png", &render(&window));
    act(&mut r, &ui, Action::Back);
    fs::write(
        &files[0],
        include_bytes!("../../ddl-core/tests/fixtures/local/tiny.wad"),
    )
    .unwrap();
    // A same-kind changed WAD, still valid: change a directory name byte.
    let mut changed = fs::read(&files[0]).unwrap();
    let last = changed.len() - 1;
    changed[last] ^= 1;
    fs::write(&files[0], changed).unwrap();
    r.snapshot = Some(Services::open(env.clone()).unwrap().snapshot().unwrap());
    r.sync(&ui);
    click(&mut r, &ui, 1);
    assert!(matches!(r.local.dialog, Some(Dialog::Issue(_, true))));
    assert_eq!(r.focus, 900);
    assert!(
        ui.get_sheet_nodes()
            .iter()
            .any(|n| n.label == ui.global::<LocalStrings>().get_file_changed())
    );
    write_png("026-changed-file.png", &render(&window));
    act(&mut r, &ui, Action::Right);
    assert_eq!(r.focus, 901);
    act(&mut r, &ui, Action::Confirm);
    assert!(r.local_issue().is_none());
    // A changed player-file IWAD follows the same explicit acceptance path.
    let base = env.home.join("base.wad");
    let mut changed_base = fs::read(&base).unwrap();
    changed_base.extend_from_slice(b"new");
    fs::write(&base, changed_base).unwrap();
    r.snapshot = Some(Services::open(env.clone()).unwrap().snapshot().unwrap());
    r.sync(&ui);
    click(&mut r, &ui, 1);
    assert!(matches!(r.local.dialog, Some(Dialog::Issue(_, true))));
    click(&mut r, &ui, 901);
    assert!(r.local_issue().is_none());
    click(&mut r, &ui, 5);
    assert_eq!(r.focus, 900);
    assert!(!r.local.delete_profiles);
    write_png("026-delete-custom.png", &render(&window));
    act(&mut r, &ui, Action::Up);
    assert_eq!(r.focus, 902);
    act(&mut r, &ui, Action::Confirm);
    assert!(r.local.delete_profiles);
    act(&mut r, &ui, Action::Back); // checkbox resets each time
    click(&mut r, &ui, 5);
    assert!(!r.local.delete_profiles);
    act(&mut r, &ui, Action::Right);
    act(&mut r, &ui, Action::Confirm);
    assert!(
        Services::open(env.clone())
            .unwrap()
            .custom_list()
            .unwrap()
            .is_empty()
    );
    assert!(store.load_optional("night-maps-default").unwrap().is_some());
    assert!(files.iter().all(|p| p.is_file()));
}

#[test]
fn local_catalog_extras_lock_recipe_members_warn_for_saves_and_keep_profile_scope() {
    if isolated("local_catalog_extras_lock_recipe_members_warn_for_saves_and_keep_profile_scope") {
        return;
    }
    let (window, ui) = setup();
    ui.show().unwrap();
    let (_temp, _server, env, _hits) = crate::services::recipe_change_tests::setup();
    let files = fixtures(&env.home);
    let mut s = crate::services::recipe_change_tests::installed(&env);
    s.execute(
        Command::NewProfile("auger-zenith".into(), false),
        &CancellationToken::new(),
        &mut |_| {},
    )
    .unwrap();
    s.extras_add("auger-zenith", None, &files).unwrap();
    let profile = s.extras_list("auger-zenith", None).unwrap();
    let store = ProfileStore::new(&env.paths.config, &env.paths.data);
    let save = store.dirs(&profile.id).save;
    fs::create_dir_all(&save).unwrap();
    fs::write(save.join("save0.dsg"), b"save").unwrap();
    let mut r = runtime(&env, &s, &ui);
    r.screen = Screen::Setup;
    r.selected = 0;
    r.focus = 60;
    r.sync(&ui);
    assert!(r.nodes.iter().any(|n| n.id == EXTRAS));
    r.focus = EXTRAS;
    r.sync(&ui);
    write_png("026-play-setup-extras.png", &render(&window));
    click(&mut r, &ui, EXTRAS);
    assert!(matches!(r.local.dialog, Some(Dialog::Saves)));
    write_png("026-extras-saves-warning.png", &render(&window));
    act(&mut r, &ui, Action::Back);
    let locked = r
        .nodes
        .iter()
        .find(|n| n.detail == ui.global::<LocalStrings>().get_locked())
        .unwrap();
    assert!(!locked.actionable);
    assert_eq!(locked.tone, 1);
    assert_eq!(
        locked.kind, 16,
        "locked rows use the dimmed plate and lock icon renderer"
    );
    assert_eq!(
        locked.label, "main.wad",
        "recipe rows have no position number"
    );
    assert!(locked.y < r.nodes.iter().find(|n| n.id == ENTRY).unwrap().y);
    write_png("026-extras-order.png", &render(&window));
    r.focus = 1;
    act(&mut r, &ui, Action::Down);
    assert_eq!(r.focus, ENTRY);
    act(&mut r, &ui, Action::Confirm);
    act(&mut r, &ui, Action::Down);
    act(&mut r, &ui, Action::Confirm);
    assert!(
        store
            .load_optional("auger-zenith-default")
            .unwrap()
            .unwrap()
            .extras
            .is_empty()
    );
    let saved = Services::open(env)
        .unwrap()
        .extras_list("auger-zenith", Some(&profile.id))
        .unwrap();
    assert_eq!(saved.extras[1].path, files[0]);
}

#[test]
fn local_delete_checked_removes_only_owned_profiles_and_saves() {
    if isolated("local_delete_checked_removes_only_owned_profiles_and_saves") {
        return;
    }
    let (_window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let env = environment(&temp.path().join("home"));
    let (s, files) = custom(&env);
    let store = ProfileStore::new(&env.paths.config, &env.paths.data);
    let dirs = store.dirs("night-maps-default");
    fs::create_dir_all(&dirs.save).unwrap();
    fs::write(dirs.save.join("save0.dsg"), b"save").unwrap();
    let mut r = runtime(&env, &s, &ui);
    click(&mut r, &ui, CUSTOM);
    click(&mut r, &ui, 5);
    click(&mut r, &ui, 902);
    assert!(r.local.delete_profiles);
    click(&mut r, &ui, 901);
    assert!(store.load_optional("night-maps-default").unwrap().is_none());
    assert!(!dirs.root.exists());
    assert!(files.iter().all(|p| p.is_file()));
    assert!(env.home.join("base.wad").is_file());
}

#[test]
fn local_path_elision_preserves_both_ends_and_unicode() {
    assert_eq!(elide_path_middle("/short/path.wad", 64), "/short/path.wad");
    let path = format!("/{}{}", "é/".repeat(100), "last.wad");
    let display = elide_path_middle(&path, 64);
    assert_eq!(display.chars().count(), 64);
    assert!(display.starts_with("/é/"));
    assert!(display.ends_with("last.wad"));
    assert!(display.contains('…'));
}

#[test]
fn base_games_library_page_setup_sources_and_recent_history() {
    if isolated("base_games_library_page_setup_sources_and_recent_history") {
        return;
    }
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let env = environment(&temp.path().join("home"));
    fixtures(&env.home);
    let _ = services(&env);
    let mut registry = ddl_core::iwads::Registry::new();
    for (file, variant, game) in [
        ("doom.wad", "doom-1.9ud", ddl_core::iwads::Game::Doom),
        ("doom2.wad", "doom2-1.9", ddl_core::iwads::Game::Doom2),
        ("tnt.wad", "tnt-1.9", ddl_core::iwads::Game::Tnt),
        (
            "plutonia.wad",
            "plutonia-1.9",
            ddl_core::iwads::Game::Plutonia,
        ),
    ] {
        let path = env
            .home
            .join(".local/share/Steam/steamapps/common/Ultimate Doom/base")
            .join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::copy(env.home.join("base.wad"), &path).unwrap();
        let meta = fs::metadata(&path).unwrap();
        registry.insert(ddl_core::iwads::RegistryEntry {
            path: path.clone(),
            variant: Some(variant.into()),
            candidates: vec![],
            game: Some(game),
            sha256: ddl_core::iwads::identify::sha256_file(&path).unwrap(),
            size: meta.len(),
            mtime: ddl_core::iwads::FileTime::from_metadata(&meta).unwrap(),
            source: ddl_core::iwads::Source::Steam,
        });
    }
    let mut alternate = registry
        .entries()
        .iter()
        .find(|e| e.game == Some(ddl_core::iwads::Game::Doom2))
        .unwrap()
        .clone();
    alternate.path = alternate
        .path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("rerelease/doom2.wad");
    fs::create_dir_all(alternate.path.parent().unwrap()).unwrap();
    fs::copy(env.home.join("base.wad"), &alternate.path).unwrap();
    let alternate_path = alternate.path.clone();
    alternate.mtime =
        ddl_core::iwads::FileTime::from_metadata(&fs::metadata(&alternate.path).unwrap()).unwrap();
    registry.insert(alternate);
    registry.save(&env.paths.config.join("iwads.json")).unwrap();
    // Use the production snapshot; no download/install record is created.
    let s = Services::open(env.clone()).unwrap();
    let mut snapshot = s.snapshot().unwrap();
    assert_eq!(snapshot.base_games.len(), 4);
    snapshot.folders.push(env.home.join("My IWADs"));
    let mut r = runtime(&env, &s, &ui);
    r.snapshot = Some(snapshot);
    r.sync(&ui);
    assert!(r.nodes.iter().any(|n| n.label == "Base games"));
    write_png("032-library-base-games.png", &render(&window));
    click(&mut r, &ui, BASE + 1);
    assert_eq!(r.local.view, Some(View::Base));
    assert!(!ui.get_root_screen());
    // Spec 033 R5: no "Original game" caption, and home paths show with `~`.
    assert!(!r.nodes.iter().any(|n| n.label.contains("Original game")));
    assert!(r.nodes.iter().any(|n| n.label.starts_with("~/")));
    write_png("032-base-game.png", &render(&window));
    click(&mut r, &ui, 2);
    assert_eq!(r.local.view, Some(View::BaseSetup));
    write_png("032-base-play-setup.png", &render(&window));
    click(&mut r, &ui, 4);
    assert_eq!(r.base_item().unwrap().profile.skill, 5);
    click(&mut r, &ui, 5);
    assert!(r.base_item().unwrap().profile.pistol_start);
    click(&mut r, &ui, 2);
    assert_eq!(r.local.view, Some(View::BaseCopies));
    click(&mut r, &ui, 11);
    assert_eq!(
        r.base_item().unwrap().profile.iwad_path.as_ref(),
        Some(&alternate_path)
    );
    fs::remove_file(&alternate_path).unwrap();
    r.snapshot = Some(Services::open(env.clone()).unwrap().snapshot().unwrap());
    r.sync(&ui);
    assert!(
        r.nodes
            .iter()
            .any(|n| n.detail.contains("Selected copy unavailable"))
    );
    write_png("032-base-copy-fallback.png", &render(&window));
    click(&mut r, &ui, 3);
    click(&mut r, &ui, 1);
    assert_eq!(r.base_item().unwrap().profiles.len(), 2);
    let profiles = &r.base_item().unwrap().profiles;
    assert_ne!(profiles[0].id, profiles[1].id);
    assert!(matches!(r.sheet, Some(Sheet::ProfileNames)));
    ui.set_profile_name("Weekend Doom II".into());
    click(&mut r, &ui, 917);
    assert_eq!(r.base_item().unwrap().profile.name, "Weekend Doom II");
    click(&mut r, &ui, 3);
    click(&mut r, &ui, 4);
    assert!(matches!(r.sheet, Some(Sheet::ProfileNames)));
    ui.set_profile_name("default".into());
    click(&mut r, &ui, 917);
    assert!(!ui.get_profile_name_error().is_empty());
    write_png("032-base-profile-name.png", &render(&window));
    ui.set_profile_name("My Doom II run".into());
    click(&mut r, &ui, 917);
    assert_eq!(r.base_item().unwrap().profile.name, "My Doom II run");
    click(&mut r, &ui, 10);
    assert_eq!(r.local.view, Some(View::BaseSetup));
    click(&mut r, &ui, 6);
    assert_eq!(r.local.view, Some(View::Order));
    act(&mut r, &ui, Action::Back);
    assert_eq!(r.local.view, Some(View::BaseSetup));
    // Automatic source actions show information and never offer removal.
    r.local = LocalUi::default();
    r.screen = Screen::Settings;
    r.settings_section = 1;
    r.focus = 70;
    r.snapshot
        .as_mut()
        .unwrap()
        .folders
        .push(env.home.join("My IWADs"));
    r.sync(&ui);
    assert!(
        r.nodes
            .iter()
            .any(|n| n.label.starts_with("Steam ·") && n.id >= 1000)
    );
    assert_eq!(
        r.nodes
            .iter()
            .filter(|n| n.label == text(&ui, T::Remove) && n.actionable)
            .count(),
        1
    );
    click(&mut r, &ui, 1000);
    assert!(r.sheet.is_none(), "automatic sources never open Remove");
    assert_eq!(r.snapshot.as_ref().unwrap().automatic_folders.len(), 2);
    r.focus = 100;
    r.sync(&ui);
    write_png("032-iwad-sources.png", &render(&window));
    // Saved profile history feeds Home across restarts.
    let mut profile = s.base_item("base:doom2", None).unwrap().profile;
    profile.touch(1234);
    ProfileStore::new(&env.paths.config, &env.paths.data)
        .save(&profile)
        .unwrap();
    r.snapshot = Some(Services::open(env.clone()).unwrap().snapshot().unwrap());
    r.screen = Screen::Home;
    r.focus = 1;
    r.sync(&ui);
    assert!(r.nodes.iter().any(|n| n.label == "Doom II"));
    assert_eq!(r.focus, BASE + 1);
    write_png("032-home-base-game.png", &render(&window));
    r.snapshot.as_mut().unwrap().items[0].last_played = Some(5678);
    r.focus = 1;
    r.sync(&ui);
    assert!(
        r.nodes.iter().any(|n| n.label == "Doom II" && n.y == 400.),
        "base games remain in mixed recent history"
    );
}

/// Spec 057 A2: walk real production rows in both directions, including both
/// columns of multi-item rows. Only directional input is used during each walk.
fn walk_rows(r: &mut Runtime, ui: &AppWindow, rows: &[&[i32]]) {
    for column in 0..rows.iter().map(|row| row.len()).max().unwrap() {
        for (direction, order) in [
            (Action::Down, (0..rows.len()).collect::<Vec<_>>()),
            (Action::Up, (0..rows.len()).rev().collect::<Vec<_>>()),
        ] {
            r.focus = rows[order[0]][column.min(rows[order[0]].len() - 1)];
            r.sync(ui);
            for &row in &order[1..] {
                let source = r.nodes.iter().find(|n| n.id == r.focus).unwrap().clone();
                r.action(ui, direction);
                assert!(
                    rows[row].contains(&r.focus),
                    "{direction:?}: expected row {:?}, got {}",
                    rows[row],
                    r.focus
                );
                // Spec 060 R1: the landing item is the leftmost row item the
                // source spans, so a source over one column keeps that column.
                let landing = rows[row]
                    .iter()
                    .filter_map(|id| r.nodes.iter().find(|n| n.id == *id))
                    .filter(|n| n.x < source.x + source.w && n.x + n.w > source.x)
                    .min_by(|a, b| a.x.total_cmp(&b.x));
                if let Some(landing) = landing {
                    assert_eq!(
                        r.focus, landing.id,
                        "{direction:?} from {} into {:?}",
                        source.id, rows[row]
                    );
                }
            }
        }
    }
}

#[test]
fn controller_057_focus_walks_every_multi_item_screen() {
    if isolated("controller_057_focus_walks_every_multi_item_screen") {
        return;
    }
    let (_window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let env = environment(&temp.path().join("home"));
    let (mut s, _) = custom(&env);
    s.register_fake_iwads(&temp.path().join("identified"))
        .unwrap();
    let mut r = runtime(&env, &s, &ui);
    assert_eq!(r.snapshot.as_ref().unwrap().custom_profiles[0].skill, 4);
    r.selected = r
        .snapshot
        .as_ref()
        .unwrap()
        .items
        .iter()
        .position(|i| i.recipe.id == "sigil")
        .unwrap();
    r.screen = Screen::Setup;
    r.focus = 60;
    r.sync(&ui);
    // Advanced → Skill/Pistol start → Recipe load order, and the exact reverse.
    walk_rows(&mut r, &ui, &[&[42], &[40, 41], &[43]]);
    // Include every preceding option row, even if a recipe adds components.
    let mut options: Vec<_> = r
        .nodes
        .iter()
        .filter(|n| n.actionable && n.x < 700.)
        .collect();
    options.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
    let mut rows: Vec<Vec<i32>> = vec![];
    let mut y = f32::NEG_INFINITY;
    for n in options {
        if (n.y - y).abs() > 1. {
            rows.push(vec![]);
            y = n.y;
        }
        rows.last_mut().unwrap().push(n.id);
    }
    let refs: Vec<_> = rows.iter().map(Vec::as_slice).collect();
    walk_rows(&mut r, &ui, &refs);
    r.screen = Screen::Detail;
    r.sync(&ui);
    walk_rows(&mut r, &ui, &[&[1], &[3]]);
    r.screen = Screen::Settings;
    r.settings_section = 1;
    r.sync(&ui);
    walk_rows(&mut r, &ui, &[&[1], &[2], &[3], &[4]]);
    r.settings_section = 1;
    walk_rows(&mut r, &ui, &[&[72], &[70, 71]]);
    // Folder actions return to the preceding full-width preference row.
    for id in [70, 71] {
        r.focus = id;
        r.action(&ui, Action::Up);
        assert_eq!(r.focus, 72);
    }
    r.screen = Screen::Library;
    r.local = LocalUi::default();
    r.sync(&ui);
    click(&mut r, &ui, BASE);
    assert_eq!(r.local.view, Some(View::Base));
    // The base-game page has one action row; each column is a boundary.
    for id in [1, 2] {
        r.focus = id;
        r.action(&ui, Action::Up);
        assert_eq!(r.focus, id);
        r.action(&ui, Action::Down);
        assert_eq!(r.focus, id);
    }
    click(&mut r, &ui, 2);
    walk_rows(&mut r, &ui, &[&[1], &[2], &[3], &[4, 5], &[6]]);
    r.local = LocalUi::default();
    r.sync(&ui);
    click(&mut r, &ui, CUSTOM);
    assert_eq!(r.local.view, Some(View::Detail));
    walk_rows(&mut r, &ui, &[&[1, 2], &[3], &[4], &[5]]);
    r.local.creating = Some("New setup".into());
    r.local.iwad = Some(NewIwad::File(env.home.join("base.wad")));
    r.local.view = Some(View::New);
    r.sync(&ui);
    walk_rows(&mut r, &ui, &[&[1], &[2], &[3]]);
    // Settings opened from a child screen retains that Back level, including
    // the recovery → Diagnostics route introduced by spec 055.
    r.local = LocalUi::default();
    r.screen = Screen::Detail;
    r.history.clear();
    r.enter(Screen::Settings, 4);
    r.sync(&ui);
    assert!(!ui.get_quit_hint());
    r.action(&ui, Action::Back);
    assert_eq!(r.screen, Screen::Detail);
    assert!(!ui.get_sheet_open());
}

/// Spec 060 A1: rows are entered at their leftmost item.
#[test]
fn controller_060_rows_are_entered_at_their_leftmost_item() {
    if isolated("controller_060_rows_are_entered_at_their_leftmost_item") {
        return;
    }
    let (_window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let env = environment(&temp.path().join("home"));
    let (mut s, _) = custom(&env);
    s.register_fake_iwads(&temp.path().join("identified"))
        .unwrap();
    let mut r = runtime(&env, &s, &ui);
    r.selected = r
        .snapshot
        .as_ref()
        .unwrap()
        .items
        .iter()
        .position(|i| i.recipe.id == "sigil")
        .unwrap();
    r.screen = Screen::Setup;
    r.focus = 60;
    r.sync(&ui);
    // Down from the full-width Advanced card lands on Skill, not Pistol start.
    r.focus = 42;
    r.action(&ui, Action::Down);
    assert_eq!(r.focus, 40, "Advanced → Skill");
    // Up from the full-width Recipe load order lands on Skill as well.
    r.focus = 43;
    r.action(&ui, Action::Up);
    assert_eq!(r.focus, 40, "Recipe load order → Skill");
    // Pistol start stays one Right away, and Left returns.
    r.action(&ui, Action::Right);
    assert_eq!(r.focus, 41, "Skill → Pistol start");
    r.action(&ui, Action::Left);
    assert_eq!(r.focus, 40);
    // A source over exactly one column keeps it: Pistol start → Load order →
    // back up lands on the leftmost of the row it spans.
    r.focus = 41;
    r.action(&ui, Action::Down);
    assert_eq!(r.focus, 43);
    r.action(&ui, Action::Up);
    assert_eq!(r.focus, 40);
}

#[test]
fn controller_057_right_stick_scrolls_planes_without_focus_changes() {
    if isolated("controller_057_right_stick_scrolls_planes_without_focus_changes") {
        return;
    }
    use crate::input::{CONFIG, Mapper, Signal};
    use gilrs::Axis;
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let env = environment(&temp.path().join("home"));
    let (s, _) = custom(&env);
    let mut r = runtime(&env, &s, &ui);
    let mut mapper = Mapper::default();
    assert_eq!(
        mapper.event(0, Signal::Axis(Axis::RightStickY, -1.), Duration::ZERO),
        None
    );
    let scroll = mapper.scroll(CONFIG.poll);
    assert!(scroll.1 > 0.);
    // Real scrollable screens all use the bounded content plane.
    for screen in [
        Screen::Home,
        Screen::Discover,
        Screen::Library,
        Screen::Detail,
        Screen::Settings,
        Screen::Setup,
    ] {
        r.screen = screen;
        r.focus = 1;
        r.sync(&ui);
        // Give each real screen an overflowing content extent to exercise the
        // shared plane even when this small fixture fits on a Deck viewport.
        ui.set_content_height(1200.);
        ui.set_scroll_offset(0.);
        let focus = ui.get_focused();
        ui.invoke_scroll(scroll.0, scroll.1);
        assert!(
            ui.get_scroll_offset() > 0.,
            "{screen:?}: height {}, size {:?}",
            ui.get_content_height(),
            window.size()
        );
        assert_eq!(ui.get_focused(), focus);
        if screen == Screen::Settings {
            r.sync(&ui);
            ui.set_scroll_offset(0.);
            ui.invoke_scroll(scroll.0, scroll.1);
            let offset = ui.get_scroll_offset();
            assert!(offset > 0., "the real Engine page overflows");
            r.sync(&ui);
            assert_eq!(
                ui.get_scroll_offset(),
                offset,
                "service repaint preserves manual scrolling"
            );
        }
    }
    // The fixed setup summary has its own plane, chosen by the logical target.
    r.screen = Screen::Setup;
    r.focus = 50;
    r.sync(&ui);
    let summary_focus = ui.get_focused();
    ui.set_setup_content_height(1000.);
    ui.set_setup_scroll_offset(0.);
    ui.invoke_scroll(0., scroll.1);
    assert!(ui.get_setup_scroll_offset() > 0.);
    assert_eq!(ui.get_focused(), summary_focus);
    r.local.view = Some(View::Compat);
    r.screen = Screen::Library;
    r.focus = 10;
    r.sync(&ui);
    ui.set_scroll_offset(0.);
    ui.invoke_scroll(0., scroll.1);
    assert!(ui.get_scroll_offset() > 0.);
    assert_eq!(ui.get_focused(), 10);
    r.local_error("Long diagnostic detail.\n".repeat(100));
    r.sync(&ui);
    ui.invoke_scroll(0., scroll.1);
    assert!(ui.get_error_scroll_offset() > 0.);
    assert_eq!(ui.get_focused(), 900);
    write_png("057-long-error-scroll.png", &render(&window));
    r.local.dialog = None;
    r.local.view = None;
    r.show_sheet(Sheet::Quit);
    r.sync(&ui);
    ui.set_sheet_height(1000.);
    ui.invoke_scroll(0., scroll.1);
    assert!(ui.get_sheet_scroll_offset() > 0.);
    assert_eq!(ui.get_focused(), 900);
    // Spec 055 recovery lists every unreadable file on the shared plane; the
    // content grows with the list, so no fixture extent is forced here.
    r.sheet = None;
    r.unreadable = (0..12)
        .map(|n| crate::services::UnreadableFile {
            path: format!("/config/profiles/unreadable-{n}.json").into(),
            reason: "expected value at line 1 column 1; ".repeat(4),
        })
        .collect();
    r.screen = Screen::Recovery;
    r.focus = 800;
    r.sync(&ui);
    let recovery_focus = ui.get_focused();
    assert!(ui.get_quit_hint(), "B opens Quit on the recovery screen");
    assert!(ui.get_content_height() > ui.window().size().height as f32 - 294.);
    ui.set_scroll_offset(0.);
    ui.invoke_scroll(0., scroll.1);
    assert!(ui.get_scroll_offset() > 0., "long Recovery list scrolls");
    assert_eq!(ui.get_focused(), recovery_focus);
    r.unreadable.clear();
    // Horizontal overflow is bounded by the same plane (e.g. a narrow window).
    r.screen = Screen::Library;
    r.focus = 1;
    r.sync(&ui);
    ui.set_content_width(1800.);
    mapper.event(0, Signal::Axis(Axis::RightStickX, 1.), Duration::ZERO);
    let (x, y) = mapper.scroll(CONFIG.poll);
    ui.invoke_scroll(x, y);
    assert!(ui.get_scroll_x_offset() > 0.);
    let focus = ui.get_focused();
    mapper.event(0, Signal::Axis(Axis::RightStickX, 0.1), Duration::ZERO);
    mapper.event(0, Signal::Axis(Axis::RightStickY, 0.1), Duration::ZERO);
    assert_eq!(mapper.scroll(CONFIG.poll), (0., 0.));
    assert_eq!(ui.get_focused(), focus);
}

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
    assert!(r.nodes.iter().any(|n| n.label == "Original game"));
    write_png("032-base-game.png", &render(&window));
    click(&mut r, &ui, 2);
    assert_eq!(r.local.view, Some(View::BaseSetup));
    write_png("032-base-play-setup.png", &render(&window));
    click(&mut r, &ui, 4);
    assert_eq!(r.base_item().unwrap().profile.skill, 4);
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

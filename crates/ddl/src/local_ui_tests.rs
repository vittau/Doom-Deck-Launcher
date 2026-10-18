//! Focused spec 026 journeys: each owns scratch state and an off-screen renderer.
use super::*;
use crate::headless::{render, setup, write_png};
use ddl_core::{downloads::CancellationToken, profiles::ProfileStore};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

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
    for path in &files[..2] {
        let index = r
            .directory
            .as_ref()
            .unwrap()
            .entries
            .iter()
            .position(|(p, _)| p == path)
            .unwrap();
        click(&mut r, &ui, PICK + index as i32);
    }
    assert_eq!(r.local.selected.len(), 2);
    write_png("026-file-picker-multiselect.png", &render(&window));
    act(&mut r, &ui, Action::Menu); // Start confirms
    assert_eq!(r.local.view, Some(View::Order));
    assert_eq!(r.local_setup().unwrap().files.len(), 2);
    // B cancels a later picker without adding anything and restores the editor.
    act(&mut r, &ui, Action::Y);
    act(&mut r, &ui, Action::Back);
    assert_eq!(r.local.view, Some(View::Order));
    assert!(files.iter().all(|p| p.is_file()));
}

#[test]
fn local_order_moves_with_dpad_stays_in_group_and_removes_without_deleting() {
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
    let (window, ui) = setup();
    ui.show().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let env = environment(&temp.path().join("home"));
    let (s, files) = custom(&env);
    let store = ProfileStore::new(&env.paths.data);
    let mut profile = store.load_optional("night-maps-default").unwrap().unwrap();
    profile.touch(1000);
    store.save(&profile).unwrap();
    let mut r = runtime(&env, &s, &ui);
    write_png("026-library-custom.png", &render(&window));
    r.screen = Screen::Home;
    r.sync(&ui);
    assert!(r.nodes.iter().any(|n| n.id == CUSTOM));
    write_png("026-home-custom.png", &render(&window));
    click(&mut r, &ui, CUSTOM);
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
    write_png("026-changed-file.png", &render(&window));
    act(&mut r, &ui, Action::Right);
    assert_eq!(r.focus, 901);
    act(&mut r, &ui, Action::Confirm);
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
    let (window, ui) = setup();
    ui.show().unwrap();
    let (_temp, _server, env, _hits) = crate::services::recipe_change_tests::setup();
    let files = fixtures(&env.home);
    let mut s = crate::services::recipe_change_tests::installed(&env);
    s.extras_add("auger-zenith", None, &files).unwrap();
    let profile = s.extras_list("auger-zenith", None).unwrap();
    let store = ProfileStore::new(&env.paths.data);
    let save = store.dirs(&profile.id).save;
    fs::create_dir_all(&save).unwrap();
    fs::write(save.join("save0.dsg"), b"save").unwrap();
    let mut r = runtime(&env, &s, &ui);
    r.screen = Screen::Setup;
    r.selected = 0;
    r.focus = 60;
    r.sync(&ui);
    assert!(r.nodes.iter().any(|n| n.id == EXTRAS));
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
    assert!(locked.y < r.nodes.iter().find(|n| n.id == ENTRY).unwrap().y);
    write_png("026-extras-order.png", &render(&window));
    r.focus = 1;
    act(&mut r, &ui, Action::Down);
    assert_eq!(r.focus, ENTRY);
    act(&mut r, &ui, Action::Confirm);
    act(&mut r, &ui, Action::Down);
    act(&mut r, &ui, Action::Confirm);
    let saved = Services::open(env)
        .unwrap()
        .extras_list("auger-zenith", Some(&profile.id))
        .unwrap();
    assert_eq!(saved.extras[1].path, files[0]);
}

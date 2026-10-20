//! Spec 059: the import offer and the sandbox line, driven by real key events
//! and rendered off-screen with Slint's software renderer.
use super::*;
use crate::{
    runtime::{Runtime, Screen},
    services::{
        Command, Environment, TestEnvironment,
        flatpak_tests::{flatpak, install_fixture, sandbox_env, steam_with_hidden_sd_card},
    },
};
use ddl_core::{
    catalog::Catalog,
    engines::dsda,
    platform::{legacy_import, sandbox::Sandbox},
};
use std::{
    cell::RefCell,
    fs,
    sync::Arc,
    time::{Duration, Instant},
};

fn settle(state: &Rc<RefCell<Runtime>>) {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        pump_events();
        if !state.borrow().busy {
            break;
        }
        assert!(Instant::now() < until, "worker stalled");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn labels(ui: &AppWindow) -> String {
    ui.get_nodes()
        .iter()
        .map(|n| n.label.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The sandbox environment with canned engine/catalog fixtures, so the first
/// start never reaches the network.
fn environment(root: &std::path::Path) -> Environment {
    let mut env = sandbox_env(root);
    env.test = Some(Arc::new(TestEnvironment {
        api: Arc::new(dsda::CannedApi::new()),
        catalog: Catalog::embedded(),
        engine_base: String::new(),
        catalog_url: String::new(),
        reviewed: dsda::ReviewedList::embedded(),
        runner: Arc::new(dsda::FakeEngineRunner::new(dsda::LaunchExit::Clean)),
        engine_response_timeout: None,
    }));
    env
}

struct Scene {
    window: Rc<slint::platform::software_renderer::MinimalSoftwareWindow>,
    ui: AppWindow,
    state: Rc<RefCell<Runtime>>,
    env: Environment,
    _temp: tempfile::TempDir,
}

fn scene(sandbox: Sandbox, tarball_install: bool) -> Scene {
    scene_with(sandbox, tarball_install, false)
}

fn scene_with(sandbox: Sandbox, tarball_install: bool, hidden_sd_card: bool) -> Scene {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    let (window, ui) = setup();
    let temp = tempfile::tempdir().unwrap();
    let env = environment(temp.path());
    if tarball_install {
        install_fixture("v0.0.7-deck", &env.home);
    }
    if hidden_sd_card {
        steam_with_hidden_sd_card(&env.home, temp.path());
    }
    let state = crate::runtime::attach_with_sandbox(&ui, env.clone(), sandbox);
    settle(&state);
    Scene {
        window,
        ui,
        state,
        env,
        _temp: temp,
    }
}

#[test]
fn import_offer_can_be_postponed_then_declined_for_good() {
    if isolated(
        module_path!(),
        "import_offer_can_be_postponed_then_declined_for_good",
    ) {
        return;
    }
    let Scene {
        window,
        ui,
        state,
        env,
        _temp,
    } = scene(flatpak(), true);

    // The offer opens over the first screen, controller-first and legible.
    assert!(ui.get_sheet_open());
    assert!(ui.get_sheet_title().contains("Import your existing setup?"));
    assert_eq!(ui.get_focused(), 900, "the safe choice has focus");
    write_png("059-import-offer.png", &render(&window));

    // B closes it without answering: nothing is remembered.
    press(&window, Key::Escape);
    assert!(!ui.get_sheet_open());
    assert_eq!(legacy_import::recorded_decision(&env.paths), None);

    // The next start asks again; "No, don't ask again" is remembered and
    // touches nothing of the player's.
    state.borrow_mut().start(&ui, Command::Startup);
    settle(&state);
    assert!(ui.get_sheet_open());
    press(&window, Key::Return);
    assert!(!ui.get_sheet_open());
    assert_eq!(
        legacy_import::recorded_decision(&env.paths),
        Some(legacy_import::Decision::Declined)
    );
    assert!(
        !env.paths
            .config
            .join("profiles/fixture-sigil.json")
            .exists(),
        "nothing imported"
    );
    assert!(
        legacy_import::LegacyInstall::under(&env.home)
            .config
            .join("app.json")
            .is_file()
    );

    state.borrow_mut().start(&ui, Command::Startup);
    settle(&state);
    assert!(!ui.get_sheet_open(), "a declined offer never comes back");
    assert_eq!(legacy_import::find(&flatpak(), &env.home, &env.paths), None);
}

#[test]
fn import_offer_accepted_brings_the_old_setup_across() {
    if isolated(
        module_path!(),
        "import_offer_accepted_brings_the_old_setup_across",
    ) {
        return;
    }
    let Scene {
        window,
        ui,
        state,
        env,
        _temp,
    } = scene(flatpak(), true);
    assert!(ui.get_sheet_open());

    // Right to "Import", A confirms.
    press(&window, Key::RightArrow);
    assert_eq!(ui.get_focused(), 901);
    write_png("059-import-offer-import-focused.png", &render(&window));
    press(&window, Key::Return);
    settle(&state);

    assert!(!ui.get_sheet_open());
    assert!(
        state.borrow().problem.is_none(),
        "{:?}",
        state.borrow().problem
    );
    let state_ref = state.borrow();
    let snapshot = state_ref
        .snapshot
        .as_ref()
        .expect("the imported state loaded");
    assert_eq!(snapshot.engine.as_ref().unwrap().version, "0.30.1");
    assert_eq!(snapshot.folders.len(), 2, "IWAD folders came across");
    assert_eq!(
        snapshot
            .items
            .iter()
            .filter(|i| i.manifest.is_some())
            .count(),
        2,
        "both installed mapsets came across"
    );
    assert_eq!(state_ref.screen, Screen::Home);
    drop(state_ref);
    assert!(env.paths.config.join("app.json").is_file());
    write_png("059-after-import.png", &render(&window));
}

#[test]
fn diagnostics_show_the_flatpak_sandbox_line_and_the_update_source() {
    if isolated(
        module_path!(),
        "diagnostics_show_the_flatpak_sandbox_line_and_the_update_source",
    ) {
        return;
    }
    let Scene {
        window,
        ui,
        state,
        env: _,
        _temp,
    } = scene(flatpak(), false);
    assert!(!ui.get_sheet_open(), "no old install, no offer");

    state.borrow_mut().screen = Screen::Settings;
    state.borrow_mut().focus = 4;
    state.borrow_mut().sync(&ui);
    ui.invoke_clicked(4);
    let shown = labels(&ui);
    assert!(
        shown.contains("Flatpak sandbox: yes (io.github.vittau.Doom-Deck-Launcher)"),
        "{shown}"
    );
    assert!(shown.contains("App version"), "{shown}");
    assert!(
        shown.contains("DSDA-Doom ") || shown.contains("No engine"),
        "the engine version is its own line: {shown}"
    );
    assert!(
        shown.contains("DDL updates come from Discover (Flathub)"),
        "{shown}"
    );
    write_png("059-diagnostics-flatpak.png", &render(&window));
}

#[test]
fn diagnostics_outside_the_sandbox_say_no_and_hide_the_store_note() {
    if isolated(
        module_path!(),
        "diagnostics_outside_the_sandbox_say_no_and_hide_the_store_note",
    ) {
        return;
    }
    let Scene {
        window,
        ui,
        state,
        env,
        _temp,
    } = scene(Sandbox::none(), true);
    // Not sandboxed: a tarball install next to us is nobody's business.
    assert!(!ui.get_sheet_open());
    assert!(
        !env.paths
            .config
            .join("profiles/fixture-sigil.json")
            .exists()
    );

    state.borrow_mut().screen = Screen::Settings;
    state.borrow_mut().focus = 4;
    state.borrow_mut().sync(&ui);
    ui.invoke_clicked(4);
    let shown = labels(&ui);
    assert!(shown.contains("Flatpak sandbox: no"), "{shown}");
    assert!(!shown.contains("Discover (Flathub)"), "{shown}");
    write_png("059-diagnostics-plain.png", &render(&window));
    let _ = fs::metadata(&env.home);
}

#[test]
fn settings_name_a_library_the_flatpak_cannot_see() {
    if isolated(
        module_path!(),
        "settings_name_a_library_the_flatpak_cannot_see",
    ) {
        return;
    }
    let Scene {
        window,
        ui,
        state,
        env,
        _temp,
    } = scene_with(flatpak(), false, true);

    state.borrow_mut().screen = Screen::Settings;
    state.borrow_mut().focus = 2;
    state.borrow_mut().sync(&ui);
    ui.invoke_clicked(2);
    let shown = labels(&ui);
    let sd_card = _temp.path().join("run/media/deck/sd-card/SteamLibrary");
    assert!(
        shown.contains(&format!(
            "{} is not visible to the Flatpak — see the guide",
            sd_card.display()
        )),
        "{shown}"
    );
    write_png("059-folders-not-visible.png", &render(&window));
    let _ = env;
}

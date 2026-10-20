//! Spec 027: the headless production journey split into focused tests.
//!
//! Every test below runs alone in a headless subprocess (one Slint platform per
//! process) with its own scratch XDG root and its own artifact directory, so
//! `cargo test -p ddl <name>` reproduces just that screen or flow. A later test
//! replays its predecessors with a silent capture sink to reach the same state;
//! the predecessor's own test is what asserts and writes those frames.
use super::{isolated, production, setup};

/// One stateful journey segment in production order.
type Step = for<'a, 'b> fn(&'a production::Scene, production::Captures<'b>);

const STEPS: [Step; 11] = [
    production::first_run,
    production::iwad_review,
    production::discover_and_detail,
    production::install_and_setup,
    production::play_setup_options,
    production::library_home_and_favorite,
    production::blocked_setup_and_settings,
    production::settings_rows,
    production::engine_updates_and_dialogs,
    production::install_error,
    production::auger_setup,
];

/// Replays the first `through` segments with no rendering, so a later segment
/// starts from exactly the state its own test reaches interactively.
fn replay(scene: &production::Scene, through: usize) {
    let silent = |_: &str| {};
    for step in &STEPS[..through] {
        step(scene, &silent);
    }
}

#[test]
fn first_run_and_engine_setup() {
    if isolated(module_path!(), "first_run_and_engine_setup") {
        return;
    }
    let scene = production::scene();
    let capture = |name: &str| scene.capture(name);
    production::first_run(&scene, &capture);
}

#[test]
fn iwad_review() {
    if isolated(module_path!(), "iwad_review") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 1);
    let capture = |name: &str| scene.capture(name);
    production::iwad_review(&scene, &capture);
}

#[test]
fn discover_and_detail() {
    if isolated(module_path!(), "discover_and_detail") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 2);
    let capture = |name: &str| scene.capture(name);
    production::discover_and_detail(&scene, &capture);
}

#[test]
fn title_band_fades_from_panel() {
    if isolated(module_path!(), "title_band_fades_from_panel") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 2);
    // Production Discover exercises screenshot and painted fallback sources;
    // the final capture is the production Mapset detail screen.
    let capture = |name: &str| {
        assert!(scene.ui.get_art_band());
        if name == "013-discover-fallback.png" {
            assert!(scene.ui.get_band_screenshot().is_empty());
            assert_eq!(scene.ui.get_band_art(), 4);
        } else {
            assert!(!scene.ui.get_band_screenshot().is_empty());
        }
        let frame = super::render(&scene.window);
        let pixel = |x: usize, y: usize| frame.as_slice()[y * 1280 + x];
        for y in 120..208 {
            let panel = pixel(839, y);
            assert_eq!((panel.r, panel.g, panel.b), (0x12, 0x0c, 0x0d));
            for x in 840..960 {
                let art = pixel(x, y);
                // Even a white source must emerge gradually over 120 px.
                // At the joint the allowed one-level rounding error rules out
                // the old abrupt edge, regardless of the source image texture.
                let bound = ((x - 840) * 255).div_ceil(120) as u8 + 2;
                for (actual, deep) in [(art.r, panel.r), (art.g, panel.g), (art.b, panel.b)] {
                    assert!(
                        actual.abs_diff(deep) <= bound,
                        "{name}: joint at {x},{y}: {actual} vs {deep}, bound {bound}"
                    );
                }
            }
        }
        assert!(
            (120..208).any(|y| {
                let p = pixel(960, y);
                p.r.abs_diff(0x12) > 20 || p.g.abs_diff(0x0c) > 20 || p.b.abs_diff(0x0d) > 20
            }),
            "{name}: the image must emerge from the fade"
        );
        super::write_png(&format!("049-{name}"), &frame);
        // The heading and status retain their original placement and pixels.
        scene.ui.set_art_band(false);
        let plain = super::render(&scene.window);
        for y in 120..208 {
            let row = y * 1280;
            assert_eq!(
                &frame.as_slice()[row + 40..row + 824],
                &plain.as_slice()[row + 40..row + 824]
            );
        }
        scene.ui.set_art_band(true);
    };
    production::discover_and_detail(&scene, &capture);
}

#[test]
fn install_progress_and_play_setup() {
    if isolated(module_path!(), "install_progress_and_play_setup") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 3);
    let capture = |name: &str| scene.capture(name);
    production::install_and_setup(&scene, &capture);
}

#[test]
fn play_setup_options_and_engine_exit() {
    if isolated(module_path!(), "play_setup_options_and_engine_exit") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 4);
    let capture = |name: &str| scene.capture(name);
    production::play_setup_options(&scene, &capture);
}

#[test]
fn library_favorite_and_home() {
    if isolated(module_path!(), "library_favorite_and_home") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 5);
    let capture = |name: &str| scene.capture(name);
    production::library_home_and_favorite(&scene, &capture);
}

#[test]
fn blocked_setup_and_settings() {
    if isolated(module_path!(), "blocked_setup_and_settings") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 6);
    let capture = |name: &str| scene.capture(name);
    production::blocked_setup_and_settings(&scene, &capture);
}

#[test]
fn settings_rows_and_about() {
    if isolated(module_path!(), "settings_rows_and_about") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 7);
    let capture = |name: &str| scene.capture(name);
    production::settings_rows(&scene, &capture);
}

#[test]
fn engine_updates_reset_and_remove_folder() {
    if isolated(module_path!(), "engine_updates_reset_and_remove_folder") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 8);
    let capture = |name: &str| scene.capture(name);
    production::engine_updates_and_dialogs(&scene, &capture);
}

#[test]
fn install_error_keeps_previous() {
    if isolated(module_path!(), "install_error_keeps_previous") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 9);
    let capture = |name: &str| scene.capture(name);
    production::install_error(&scene, &capture);
}

#[test]
fn auger_options_and_components() {
    if isolated(module_path!(), "auger_options_and_components") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 10);
    let capture = |name: &str| scene.capture(name);
    production::auger_setup(&scene, &capture);
}

#[test]
fn profiles_isolation_and_continue() {
    if isolated(module_path!(), "profiles_isolation_and_continue") {
        return;
    }
    let scene = production::scene();
    replay(&scene, 11);
    let capture = |name: &str| scene.capture(name);
    production::profiles(&scene, &capture);
}

/// Spec 015: the Discover category strip, favorites and the screenshot credits.
#[test]
fn categories_favorites_and_screenshots() {
    if isolated(module_path!(), "categories_favorites_and_screenshots") {
        return;
    }
    let (window, ui) = setup();
    production::categories_journey(&window, &ui);
}

/// Spec 020: an installed recipe change, Keep current and a removed recipe.
#[test]
fn recipe_changes_update_keep_and_removed() {
    if isolated(module_path!(), "recipe_changes_update_keep_and_removed") {
        return;
    }
    let (window, ui) = setup();
    production::recipe_changes_journey(&window, &ui);
}

/// Spec 022: the uninstall confirm dialog and the keep-profiles path.
#[test]
fn uninstall_confirm_and_keep_profiles() {
    if isolated(module_path!(), "uninstall_confirm_and_keep_profiles") {
        return;
    }
    let (window, ui) = setup();
    production::uninstall_journey(&window, &ui);
}

/// Spec 025: every production title/author slot fits. Each screen group is its
/// own test so no single render pass runs long. `DDL_TITLE_RECIPE` keeps one
/// render fixture; `DDL_TITLE_AUDIT` writes the measurement audit.
fn title_fit(
    module: &str,
    name: &str,
    screens: &'static [&'static str],
    components: bool,
    sheets: &'static [&'static str],
    recipe_group: Option<usize>,
) {
    if isolated(module, name) {
        return;
    }
    let (window, ui) = setup();
    production::title_fit_journey(&window, &ui, screens, components, sheets, recipe_group);
}

#[test]
fn title_fit_home() {
    title_fit(
        module_path!(),
        "title_fit_home",
        &["home"],
        false,
        &[],
        None,
    );
}

#[test]
fn title_fit_discover() {
    title_fit(
        module_path!(),
        "title_fit_discover",
        &["discover"],
        false,
        &[],
        None,
    );
}

#[test]
fn title_fit_library() {
    title_fit(
        module_path!(),
        "title_fit_library",
        &["library"],
        false,
        &[],
        None,
    );
}

#[test]
fn title_fit_detail() {
    title_fit(
        module_path!(),
        "title_fit_detail",
        &["detail"],
        false,
        &[],
        None,
    );
}

#[test]
fn title_fit_progress() {
    title_fit(
        module_path!(),
        "title_fit_progress",
        &["progress"],
        false,
        &[],
        None,
    );
}

#[test]
fn title_fit_setup_and_components_first() {
    title_fit(
        module_path!(),
        "title_fit_setup_and_components_first",
        &["setup"],
        true,
        &[],
        Some(0),
    );
}

#[test]
fn title_fit_setup_and_components_second() {
    title_fit(
        module_path!(),
        "title_fit_setup_and_components_second",
        &["setup"],
        true,
        &[],
        Some(1),
    );
}

#[test]
fn title_fit_profiles() {
    title_fit(
        module_path!(),
        "title_fit_profiles",
        &[],
        false,
        &["profiles"],
        None,
    );
}

#[test]
fn title_fit_uninstall() {
    title_fit(
        module_path!(),
        "title_fit_uninstall",
        &[],
        false,
        &["uninstall"],
        None,
    );
}

/// Spec 030: production navigation, modal hints, spacing and timed feedback.
#[test]
fn navigation_feedback() {
    use crate::{input::Action, runtime::Screen};
    use slint::Model;
    if isolated(module_path!(), "navigation_feedback") {
        return;
    }
    let scene = production::scene();
    production::first_run(&scene, &|_| {});
    let ui = &scene.ui;
    let mut r = scene.state.borrow_mut();
    r.screen = Screen::FirstRun;
    let installed_engine = r.snapshot.as_mut().unwrap().engine.take();
    r.focus = 1;
    r.sync(ui);
    assert!(ui.get_install_pulse());
    assert!(!ui.get_tabs_enabled());
    for action in [Action::PreviousTab, Action::NextTab, Action::Up] {
        r.action(ui, action);
        assert_eq!(r.screen, Screen::FirstRun);
        assert!(r.focus >= 0);
    }
    r.focus = 1;
    r.sync(ui);
    scene.capture("030-first-run.png");
    r.snapshot.as_mut().unwrap().engine = installed_engine;
    r.sync(ui);
    assert!(!ui.get_install_pulse(), "installed engine stops the pulse");
    r.snapshot.as_mut().unwrap().engine = None;
    r.playing = true;
    r.sync(ui);
    assert!(!ui.get_install_pulse(), "gameplay never pulses");
    r.playing = false;
    for screen in [
        Screen::Home,
        Screen::Discover,
        Screen::Library,
        Screen::Settings,
    ] {
        r.screen = screen;
        r.focus = 1;
        r.sync(ui);
        assert!(ui.get_tabs_enabled());
        assert!(!ui.get_install_pulse());
        r.action(ui, Action::NextTab);
        assert_ne!(r.screen, screen);
    }
    r.screen = Screen::Home;
    r.focus = 1;
    r.sync(ui);
    let assert_hint_pixels = || {
        let frame = super::render(&scene.window);
        let hint_pixels = (754..786)
            .flat_map(|y| (40..240).map(move |x| y * 1280 + x))
            .filter(|&i| {
                let p = frame.as_slice()[i];
                (p.r, p.g, p.b) != (0x12, 0x0c, 0x0d)
            })
            .count();
        assert!(
            hint_pixels > 100,
            "dialog A/B hints must be visible above the scrim"
        );
    };
    assert_hint_pixels();
    scene.capture("030-home.png");
    r.action(ui, Action::Menu);
    assert!(ui.get_sheet_open());
    assert!(!ui.get_tabs_enabled());
    assert_hint_pixels();
    scene.capture("030-dialog.png");
    r.action(ui, Action::Back);
    r.screen = Screen::Discover;
    let sample = r.snapshot.as_ref().unwrap().items[0].clone();
    for i in 1..9 {
        let mut item = sample.clone();
        item.recipe.id = format!("scroll-fixture-{i}");
        r.snapshot.as_mut().unwrap().items.push(item);
    }
    r.focus = 10;
    r.sync(ui);
    let last = ui
        .get_nodes()
        .iter()
        .filter(|n| n.actionable && (10..100).contains(&n.id))
        .last()
        .unwrap()
        .id;
    r.focus = last;
    r.sync(ui);
    assert!(ui.get_scroll_offset() > 0.);
    scene.capture("030-discover-scrolled.png");
    r.screen = Screen::Detail;
    r.focus = 1;
    r.sync(ui);
    for action in [Action::PreviousTab, Action::NextTab, Action::Up] {
        r.action(ui, action);
        assert_eq!(r.screen, Screen::Detail);
        assert!(r.focus >= 0);
    }
    assert!(!ui.get_tabs_enabled());
    scene.capture("030-detail.png");
    ui.invoke_show_toast("Old toast".into());
    super::advance_time(2000);
    assert_eq!(ui.get_toast(), "Old toast");
    ui.invoke_show_toast("New toast".into());
    super::advance_time(2000);
    ui.invoke_show_toast("New toast".into());
    super::advance_time(2000);
    assert_eq!(ui.get_toast(), "New toast");
    assert!(ui.get_toast_opacity() > 0.);
    super::advance_time(1100);
    assert!(ui.get_toast().is_empty());
    let _ = ui.get_toast_opacity();
    super::advance_time(150);
    assert!(
        ui.get_toast_opacity() > 0.0 && ui.get_toast_opacity() < 1.0,
        "toast fades rather than disappearing abruptly"
    );
    super::advance_time(200);
    assert_eq!(ui.get_toast_opacity(), 0.);
    r.screen = Screen::Home;
    r.sync(ui);
    ui.invoke_show_toast("Stale toast".into());
    r.action(ui, Action::NextTab);
    assert!(ui.get_toast().is_empty());
}

/// Spec 040 R6: a download row resumes from the bytes on disk and says
/// "Resuming…" while the range request is in flight.
#[test]
fn resuming_download_row() {
    if isolated(module_path!(), "resuming_download_row") {
        return;
    }
    use crate::runtime::{Screen, download_row};
    use ddl_core::downloads::{InstallProgress, Progress};
    use slint::Model as _;

    let scene = production::scene();
    let state = scene.state.clone();
    let ui = &scene.ui;
    {
        let mut r = state.borrow_mut();
        r.screen = Screen::Progress;
        r.busy = true;
        r.progress = Some(InstallProgress::transfer(Progress {
            downloaded: 61 * 1024 * 1024,
            total: Some(121 * 1024 * 1024),
            resuming: true,
        }));
        r.sync(ui);
        let row = download_row(r.progress);
        assert_eq!(row.fraction, Some(504), "61 of 121 MiB");
        assert!(row.resuming);
    }
    assert!(
        ui.get_nodes().iter().any(|n| n.label.contains("Resuming")),
        "the download row must say Resuming…"
    );
    assert!(
        ui.get_nodes().iter().any(|n| n.art == 504),
        "the bar starts at the resumed byte count"
    );
    scene.capture("040-resuming-download.png");
}

#[test]
fn controller_057_tabs_skills_and_root_quit() {
    if isolated(module_path!(), "controller_057_tabs_skills_and_root_quit") {
        return;
    }
    use crate::runtime::Screen;
    use slint::platform::Key;
    use slint::{ComponentHandle, Model};
    let scene = production::scene();
    replay(&scene, 4);
    let ui = &scene.ui;
    assert!(ui.get_nodes().iter().any(|n| n.label == "Skill 4 (UV)"));
    scene.state.borrow_mut().focus = 40;
    scene.state.borrow_mut().sync(ui);
    scene.capture("057-skill-picker-uv.png");
    ui.invoke_clicked(40);
    // Save completion updates the production skill control.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while scene.state.borrow().busy {
        super::pump_events();
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "Skill 5 (Nightmare)")
    );
    scene.capture("057-skill-picker-nightmare.png");
    for label in ["Skill 4 (UV)", "Skill 5 (Nightmare)"] {
        ui.set_fit_label(label.into());
        ui.set_fit_width(272.);
        ui.set_fit_height(36.);
        ui.set_fit_size(22.);
        ui.set_fit_smaller(22.);
        ui.set_fit_display(false);
        ui.set_fit_strong(true);
        ui.set_fit_multiline(false);
        assert!(ui.get_fit_base_width() <= 272.);
        assert!(ui.get_fit_base_height() <= 36.);
    }
    let quits = std::rc::Rc::new(std::cell::Cell::new(0));
    let q = quits.clone();
    ui.on_quit_requested(move || q.set(q.get() + 1));
    scene.state.borrow_mut().screen = Screen::Home;
    scene.state.borrow_mut().sync(ui);
    for (tab, screen) in [
        Screen::Home,
        Screen::Library,
        Screen::Discover,
        Screen::Settings,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(scene.state.borrow().screen, screen);
        assert_eq!(ui.get_active_tab(), tab as i32);
        super::press(&scene.window, Key::Escape);
        assert!(ui.get_sheet_open());
        assert_eq!(ui.get_focused(), 900);
        let buttons: Vec<_> = ui.get_sheet_nodes().iter().collect();
        assert_eq!(buttons.iter().find(|n| n.id == 900).unwrap().label, "Quit");
        assert_eq!(
            buttons.iter().find(|n| n.id == 901).unwrap().label,
            "Keep playing"
        );
        if tab == 0 {
            scene.capture("057-quit-default.png");
        }
        super::press(&scene.window, Key::Escape);
        assert!(!ui.get_sheet_open());
        assert_eq!(quits.get(), 0);
        scene.capture(&format!("057-tab-{tab}.png"));
        super::press(&scene.window, Key::Tab);
    }
    assert_eq!(scene.state.borrow().screen, Screen::Home);
    super::press(&scene.window, Key::Escape);
    super::press(&scene.window, Key::Return);
    assert_eq!(quits.get(), 1);
    // Reverse shoulders and direct tab activation share the same index mapping.
    scene
        .state
        .borrow_mut()
        .action(ui, crate::input::Action::PreviousTab);
    assert_eq!(scene.state.borrow().screen, Screen::Settings);
    for (index, expected) in [
        Screen::Home,
        Screen::Library,
        Screen::Discover,
        Screen::Settings,
    ]
    .into_iter()
    .enumerate()
    {
        ui.invoke_action(20 + index as i32);
        assert_eq!(scene.state.borrow().screen, expected);
    }
    let _ = ui.window();
}

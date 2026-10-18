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

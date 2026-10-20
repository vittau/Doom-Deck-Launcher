//! Spec 055: recovery through real keyboard events and off-screen rendering.
use super::*;
use crate::{
    runtime::{Runtime, Screen},
    services::Services,
};
use std::{
    cell::RefCell,
    fs,
    time::{Duration, Instant},
};

fn settle(state: &Rc<RefCell<Runtime>>) {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        pump_events();
        if !state.borrow().busy {
            break;
        }
        assert!(Instant::now() < until, "recovery worker stalled");
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

#[test]
fn saved_state_recovery_with_controller_keys() {
    if isolated(module_path!(), "saved_state_recovery_with_controller_keys") {
        return;
    }
    let scene = super::production::scene();
    let (window, ui, state) = (&scene.window, &scene.ui, &scene.state);
    let env = scene.env.clone();
    let paths = &scene.paths;
    settle(state);
    fs::create_dir_all(&paths.config).unwrap();
    let broken = paths.config.join("app.json");
    fs::write(&broken, b"{broken json").unwrap();
    state
        .borrow_mut()
        .start(ui, crate::services::Command::Startup);
    settle(state);
    assert_eq!(state.borrow().screen, Screen::Recovery);
    assert!(labels(ui).contains("app.json"));
    assert!(!labels(ui).contains("Install DSDA-Doom"));
    write_png("055-recovery.png", &render(window));

    // A retries; its second failure is visibly different.
    press(window, Key::Return);
    settle(state);
    assert!(labels(ui).contains("Retry failed"));
    write_png("055-retry-still-unreadable.png", &render(window));

    // D-pad to Diagnostics, A opens it even though no snapshot exists.
    press(window, Key::RightArrow);
    press(window, Key::RightArrow);
    press(window, Key::Return);
    assert_eq!(state.borrow().screen, Screen::Settings);
    let diagnostics = labels(ui);
    for wanted in [
        crate::cli::VERSION,
        "logs/ddl.log",
        "Config:",
        "Data:",
        "app.json",
    ] {
        assert!(
            diagnostics.contains(wanted),
            "missing {wanted}: {diagnostics}"
        );
    }
    write_png("055-diagnostics-unreadable.png", &render(window));
    // Every state-dependent Settings panel explains itself.
    for key in [Key::UpArrow, Key::UpArrow, Key::UpArrow] {
        press(window, key);
        assert!(labels(ui).contains("Saved setup is unavailable"));
    }
    press(window, Key::Escape);
    assert_eq!(state.borrow().screen, Screen::Recovery);
    // Focus restored to Diagnostics; left reaches Start fresh.
    press(window, Key::LeftArrow);
    press(window, Key::Return);
    assert!(ui.get_sheet_open());
    write_png("055-start-fresh-confirm.png", &render(window));
    press(window, Key::Escape);
    assert!(!ui.get_sheet_open());
    assert!(broken.exists(), "B must cancel without renaming");
    press(window, Key::Return);
    press(window, Key::RightArrow);
    press(window, Key::Return);
    settle(state);
    assert_eq!(state.borrow().screen, Screen::FirstRun);
    assert!(state.borrow().problem.is_none());
    let backups = fs::read_dir(&paths.config)
        .unwrap()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("app.json.unreadable-")
        })
        .collect::<Vec<_>>();
    assert_eq!(backups.len(), 1);
    assert_eq!(fs::read(backups[0].path()).unwrap(), b"{broken json");
    assert!(Services::open(env).is_ok());
}

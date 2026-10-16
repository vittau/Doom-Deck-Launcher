//! End-to-end production services with loopback HTTP, scratch XDG roots, fake engine runner.
use super::*;
use crate::{
    runtime::{Runtime, Screen},
    services::{Command, Environment, Problem, Services, TestEnvironment},
};
use ddl_core::{
    catalog::Catalog,
    downloads::{
        Sha256,
        test_http::{Response, Server},
    },
    engines::dsda::{self, EngineRunner},
};
use std::{
    cell::RefCell,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};

struct Gate {
    open: Mutex<bool>,
    wake: Condvar,
}
impl Gate {
    fn new() -> Self {
        Self {
            open: Mutex::new(false),
            wake: Condvar::new(),
        }
    }
    fn wait(&self) {
        let guard = self.open.lock().unwrap();
        drop(
            self.wake
                .wait_timeout_while(guard, Duration::from_secs(20), |open| !*open)
                .unwrap(),
        );
    }
    fn release(&self) {
        *self.open.lock().unwrap() = true;
        self.wake.notify_all();
    }
}
struct GatedRunner {
    gate: Arc<Gate>,
    fake: Arc<dsda::FakeEngineRunner>,
    exit: Arc<std::sync::atomic::AtomicI32>,
}
impl EngineRunner for GatedRunner {
    fn launch(
        &self,
        p: &dsda::LaunchPlan,
        a: &dsda::AssignAllowlist,
    ) -> Result<dsda::LaunchOutcome, dsda::LaunchError> {
        self.gate.wait();
        let mut outcome = self.fake.launch(p, a)?;
        let exit = self.exit.load(std::sync::atomic::Ordering::SeqCst);
        outcome.exit_status = if exit < 0 {
            dsda::LaunchExit::Signalled { signal: -exit }
        } else if exit == 0 {
            dsda::LaunchExit::Clean
        } else {
            dsda::LaunchExit::Failed { code: exit }
        };
        Ok(outcome)
    }
}
fn wait(state: &Rc<RefCell<Runtime>>, test: impl Fn(&Runtime) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        pump_events();
        if test(&state.borrow()) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "worker stalled: screen {:?}, problem {:?}",
            state.borrow().screen,
            state.borrow().problem
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn idle(state: &Rc<RefCell<Runtime>>) {
    wait(state, |r| !r.busy);
    assert!(
        state.borrow().problem.is_none(),
        "{:?}",
        state.borrow().problem
    );
}
fn capture(window: &MinimalSoftwareWindow, name: &str) {
    let buffer = render(window);
    write_png(name, &buffer);
    let reference = match name {
        "010-first-run.png" => "first-run.png",
        "010-first-run-engine-install.png"
        | "010-install-progress.png"
        | "010-install-error.png" => "download-install.png",
        "010-mapset-detail.png" => "mapset-detail.png",
        "010-play-setup-ready.png" | "010-engine-failed-toast.png" => "play-setup-sigil.png",
        "010-play-setup-blocked.png" => "play-blocked-iwad.png",
        "010-settings.png" => "settings.png",
        _ => name,
    };
    comparison(reference, &buffer);
}
fn wad() -> Vec<u8> {
    let mut bytes = b"IWAD".to_vec();
    bytes.extend_from_slice(&36i32.to_le_bytes());
    bytes.extend_from_slice(&12i32.to_le_bytes());
    for episode in 1..=4 {
        for map in 1..=9 {
            bytes.extend_from_slice(&[0; 8]);
            let mut name = [0; 8];
            let label = format!("E{episode}M{map}");
            name[..label.len()].copy_from_slice(label.as_bytes());
            bytes.extend_from_slice(&name);
        }
    }
    bytes
}

pub(super) fn journey(window: &MinimalSoftwareWindow, ui: &AppWindow) {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    let owned = home.join("owned");
    std::fs::create_dir_all(&owned).unwrap();
    std::fs::write(owned.join("renamed.wad"), wad()).unwrap();
    let paths = ddl_core::platform::paths::resolve(&|key: &str| {
        (key == "HOME").then(|| home.display().to_string())
    })
    .unwrap();
    let engine_bytes = b"#!/bin/sh\nprintf 'dsda-doom v0.30.1\\n'\n".to_vec();
    let mut catalog = Catalog::embedded();
    catalog.recipes.retain(|r| r.recipe.id == "sigil");
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    use std::io::Write;
    for member in &mut catalog.recipes[0].recipe.packages[0].members {
        let bytes = format!("Synthetic test member: {}", member.id).into_bytes();
        member.size = bytes.len() as u64;
        member.sha256 = Sha256::of(&bytes).to_hex();
        zip.start_file(&member.path, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&bytes).unwrap();
    }
    let package = zip.finish().unwrap().into_inner();
    let mut broken = catalog.recipes[0].clone();
    broken.recipe.id = "zz-test-error".into();
    broken.recipe.title = "Checksum error test".into();
    broken.recipe.packages[0].sha256 = "0".repeat(64);
    catalog.recipes.push(broken);
    let package_gate = Arc::new(Gate::new());
    let pg = package_gate.clone();
    let engine_gate = Arc::new(Gate::new());
    let g = engine_gate.clone();
    let engine_body = engine_bytes.clone();
    let package_body = package.clone();
    let server = Server::start(move |request| {
        if request.target.contains("AppImage") {
            g.wait();
            Response::ok(engine_body.clone())
        } else {
            pg.wait();
            Response::ok(package_body.clone())
        }
    });
    let url = server.url("/sigil.zip");
    catalog.recipes[0].recipe.packages[0].urls = vec![url];
    catalog.recipes[0].recipe.packages[0].size = package.len() as u64;
    catalog.recipes[0].recipe.packages[0].sha256 = Sha256::of(&package).to_hex();
    catalog.hosts.downloads.push("127.0.0.1".into());
    catalog.recipes[1].recipe.packages[0].urls = vec![server.url("/bad.zip")];
    catalog.recipes[1].recipe.packages[0].size = package.len() as u64;
    let reviewed=dsda::ReviewedList::parse(&format!("schema_version = 1\n[[version]]\ntag = 'v0.30.1'\nasset = 'test-linux.AppImage'\nsize = {}\nsha256 = '{}'\nlicense = 'GPL-2.0-or-later'\nrelease_url = 'https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.1'\nsource_url = 'https://github.com/dsda-org/dsda-doom/tree/v0.30.1'\n",engine_bytes.len(),Sha256::of(&engine_bytes).to_hex())).unwrap();
    let play_gate = Arc::new(Gate::new());
    let fake = Arc::new(
        dsda::FakeEngineRunner::new(dsda::LaunchExit::Failed { code: 255 })
            .with_log_path(paths.state.join("logs/fake-engine.log")),
    );
    let exit = Arc::new(std::sync::atomic::AtomicI32::new(255));
    let test = Arc::new(TestEnvironment {
        catalog,
        reviewed,
        engine_base: server.url(""),
        runner: Arc::new(GatedRunner {
            gate: play_gate.clone(),
            fake: fake.clone(),
            exit: exit.clone(),
        }),
    });
    let env = Environment {
        paths: paths.clone(),
        home: home.clone(),
        test: Some(test),
    };
    let state = crate::runtime::attach_environment(ui, env.clone());
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::FirstRun);
    assert!(paths.config.join("dsda/active.cfg").is_file());
    capture(window, "010-first-run.png");
    // A requests real reviewed install on a worker; gate HTTP while rendering progress.
    press(window, Key::Return);
    wait(&state, |r| r.progress.is_some());
    capture(window, "010-first-run-engine-install.png");
    assert_eq!(state.borrow().screen, Screen::Progress);
    press(window, Key::Return);
    assert_eq!(ui.get_focused(), 900);
    press(window, Key::Return);
    assert!(
        state.borrow().busy,
        "safe Cancel default keeps download running"
    );
    press(window, Key::Return);
    press(window, Key::RightArrow);
    press(window, Key::Return);
    engine_gate.release();
    wait(&state, |r| !r.busy);
    assert_eq!(state.borrow().problem, Some(Problem::Cancelled));
    assert!(
        Services::open(env.clone())
            .unwrap()
            .snapshot()
            .unwrap()
            .engine
            .is_none()
    );
    ui.invoke_clicked(800);
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::FirstRun);
    assert!(state.borrow().snapshot.as_ref().unwrap().engine.is_some());
    // Controller browser, bounded scan and explicit ambiguity choice.
    ui.invoke_clicked(2);
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::Browser);
    capture(window, "010-directory-browser.png");
    press(window, Key::Escape);
    assert_eq!(state.borrow().screen, Screen::FirstRun);
    ui.invoke_clicked(2);
    idle(&state);
    ui.invoke_clicked(1);
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::Review);
    capture(window, "010-iwad-ambiguous.png");
    let candidate = state.borrow().snapshot.as_ref().unwrap().iwads[0]
        .candidates
        .iter()
        .position(|v| v == "doom-1.9ud")
        .unwrap();
    ui.invoke_clicked(10 + candidate as i32);
    idle(&state);
    capture(window, "010-iwad-found.png");
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().iwads[0]
            .variant
            .as_deref(),
        Some("doom-1.9ud")
    );
    ui.invoke_clicked(1);
    assert_eq!(state.borrow().screen, Screen::FirstRun);
    ui.invoke_clicked(4);
    ui.invoke_clicked(10);
    assert_eq!(state.borrow().screen, Screen::Detail);
    capture(window, "010-mapset-detail.png");
    ui.invoke_clicked(1);
    wait(&state, |r| r.progress.is_some() || !r.busy);
    assert_eq!(state.borrow().screen, Screen::Progress);
    capture(window, "010-install-progress.png");
    package_gate.release();
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::Setup);
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .launch
            .is_ok()
    );
    capture(window, "010-play-setup-ready.png");
    // Profile values persist; advanced replacement never combines with main.
    ui.invoke_clicked(40);
    idle(&state);
    ui.invoke_clicked(41);
    idle(&state);
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .skill,
        4
    );
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .pistol_start
    );
    ui.invoke_clicked(42);
    ui.invoke_clicked(101);
    assert!(ui.get_sheet_open());
    press(window, Key::Escape);
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .components
            .is_empty()
    );
    ui.invoke_clicked(101);
    press(window, Key::RightArrow);
    press(window, Key::Return);
    idle(&state);
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .components,
        vec!["compat"]
    );
    ui.invoke_clicked(2);
    idle(&state);
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .launch
            .as_ref()
            .unwrap()
            .content[0]
            .path
            .ends_with("SIGIL_COMPAT_V1_23.wad")
    );
    ui.invoke_clicked(101);
    idle(&state);
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .components
            .is_empty()
    );
    // The directory browser exposes local WAD imports, and an unknown paid combination stays off.
    std::fs::write(
        home.join("purchased.wad"),
        b"Unrecognized synthetic soundtrack",
    )
    .unwrap();
    ui.invoke_clicked(102);
    idle(&state);
    let row = state
        .borrow()
        .directory
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .position(|(p, _)| p == &home.join("purchased.wad"))
        .unwrap();
    ui.invoke_clicked(100 + row as i32);
    wait(&state, |r| !r.busy);
    assert_eq!(state.borrow().problem, Some(Problem::UnknownFile));
    assert!(
        Services::open(env.clone())
            .unwrap()
            .snapshot()
            .unwrap()
            .items[0]
            .profile
            .components
            .is_empty()
    );
    press(window, Key::Escape);
    assert_eq!(state.borrow().screen, Screen::Setup);
    // The full focused load-order row is visible, including its ring, while the summary stays fixed.
    ui.invoke_clicked(43);
    let nodes = ui.get_nodes();
    let focused = nodes.iter().find(|n| n.id == 43).unwrap();
    let offset = ui.get_scroll_offset();
    assert!(focused.y - offset >= 12.);
    assert!(focused.y + focused.h - offset <= 494.);
    capture(window, "010-play-setup-options-scroll.png");
    // Playing ignores controller and pointer actions, retains run guard and restores Play.
    ui.invoke_clicked(50);
    wait(&state, |r| r.playing);
    capture(window, "010-playing.png");
    assert!(ddl_core::dsda_config::RunGuard::is_active(&paths.config.join("dsda")).unwrap());
    let focused = ui.get_focused();
    press(window, Key::Tab);
    press(window, Key::Menu);
    ui.invoke_clicked(40);
    assert_eq!(ui.get_focused(), focused);
    assert!(!ui.get_sheet_open());
    play_gate.release();
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::Setup);
    assert_eq!(ui.get_focused(), 50);
    assert_eq!(fake.launch_count(), 1);
    assert!(ui.get_toast().contains("fake-engine.log"));
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .last_played
            .is_some()
    );
    capture(window, "010-engine-failed-toast.png");
    assert!(!ddl_core::dsda_config::RunGuard::is_active(&paths.config.join("dsda")).unwrap());
    state.borrow_mut().neutral_poll(true);
    for status in [-6, 0] {
        exit.store(status, std::sync::atomic::Ordering::SeqCst);
        ui.invoke_clicked(50);
        idle(&state);
        assert_eq!(ui.get_focused(), 50);
        assert_eq!(ui.get_toast().is_empty(), status == 0);
        state.borrow_mut().neutral_poll(true);
    }
    assert_eq!(fake.launch_count(), 3);
    press(window, Key::Escape);
    assert_eq!(state.borrow().screen, Screen::Detail);
    press(window, Key::Escape);
    assert_eq!(state.borrow().screen, Screen::Discover);
    press(window, Key::Tab);
    assert_eq!(state.borrow().screen, Screen::Library);
    assert!(ui.get_nodes().iter().any(|n| n.id == 10));
    assert!(!ui.get_nodes().iter().any(|n| n.id == 11));
    capture(window, "010-library.png");
    press(window, Key::F1);
    assert_eq!(state.borrow().screen, Screen::Home);
    capture(window, "010-home-continue.png");
    ui.invoke_clicked(1);
    assert_eq!(state.borrow().screen, Screen::Setup);
    // Missing IWAD rechecks and gates the real resolver without changing the install.
    std::fs::remove_file(owned.join("renamed.wad")).unwrap();
    state.borrow_mut().start(ui, Command::Scan(None));
    idle(&state);
    state.borrow_mut().screen = Screen::Setup;
    state.borrow_mut().focus = 50;
    state.borrow_mut().sync(ui);
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .launch
            .is_err()
    );
    capture(window, "010-play-setup-blocked.png");
    ui.invoke_clicked(50);
    assert_eq!(fake.launch_count(), 3);
    // Reset is confirmed and preserves a backup. Folder remove never deletes user data.
    state.borrow_mut().screen = Screen::Settings;
    state.borrow_mut().focus = 1;
    state.borrow_mut().sync(ui);
    capture(window, "010-settings.png");
    ui.invoke_clicked(3);
    ui.invoke_clicked(70);
    press(window, Key::Return);
    assert!(!state.borrow().busy);
    ui.invoke_clicked(70);
    press(window, Key::RightArrow);
    press(window, Key::Return);
    idle(&state);
    assert!(
        std::fs::read_dir(paths.config.join("dsda/backups"))
            .unwrap()
            .next()
            .is_some()
    );
    ui.invoke_clicked(2);
    ui.invoke_clicked(100);
    assert!(ui.get_sheet_open());
    press(window, Key::Escape);
    assert!(!state.borrow().snapshot.as_ref().unwrap().folders.is_empty());
    ui.invoke_clicked(100);
    press(window, Key::RightArrow);
    press(window, Key::Return);
    idle(&state);
    assert!(state.borrow().snapshot.as_ref().unwrap().folders.is_empty());
    assert!(home.join("purchased.wad").is_file());
    // Error is generated by a real checksum rejection, with the prior install retained.
    state.borrow_mut().screen = Screen::Detail;
    state.borrow_mut().selected = 1;
    state.borrow_mut().focus = 1;
    state.borrow_mut().sync(ui);
    ui.invoke_clicked(1);
    wait(&state, |r| !r.busy);
    assert_eq!(state.borrow().problem, Some(Problem::Checksum));
    capture(window, "010-install-error.png");
    assert!(
        ui.get_nodes()
            .iter()
            .any(|node| node.label.contains("checksum"))
    );
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .manifest
            .is_some()
    );
    // Restart uses stored manifest/recipe and profile, with no discovery download.
    let restarted = Services::open(env).unwrap().snapshot().unwrap();
    assert_eq!(restarted.items[0].profile.skill, 4);
    assert!(restarted.items[0].manifest.is_some());
    assert!(restarted.items[0].profile.last_played.is_some());
}

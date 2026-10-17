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
                .wait_timeout_while(guard, Duration::from_secs(60), |open| !*open)
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
    let started = Instant::now();
    let buffer = render(window);
    let elapsed = started.elapsed();
    if matches!(name, "010-install-progress.png" | "010-home-continue.png") {
        let source =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/assets/ember/ornament-top.png");
        let mut reader = png::Decoder::new(std::io::BufReader::new(
            std::fs::File::open(source).unwrap(),
        ))
        .read_info()
        .unwrap();
        let mut expected = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut expected).unwrap();
        assert_eq!((info.width, info.height), (96, 28));
        let x = if name == "010-install-progress.png" {
            592
        } else {
            384
        };
        for row in 0..28 {
            for col in 0..96 {
                let pixel = buffer.as_slice()[(222 + row) * WIDTH as usize + x + col];
                let offset = (row * 96 + col) * 3;
                assert_eq!(
                    [pixel.r, pixel.g, pixel.b],
                    expected[offset..offset + 3],
                    "ornament must render at native size on {name}"
                );
            }
        }
    }
    if name == "010-discover.png" {
        for y in 294..318 {
            for x in 32..432 {
                assert_eq!(
                    buffer.as_slice()[y * WIDTH as usize + x],
                    buffer.as_slice()[y * WIDTH as usize + x + 408],
                    "focused card retains the same carved top rail"
                );
            }
        }
    }
    assert!(
        !window.draw_if_needed(|_| panic!("idle UI unexpectedly repainted")),
        "no continuous animation at idle"
    );
    let path = write_png(name, &buffer);
    use std::io::Write;
    if name == "010-loading.png" {
        std::fs::write(path.parent().unwrap().join("012-render-metrics.tsv"), "").unwrap();
    }
    let mut metrics = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.parent().unwrap().join("012-render-metrics.tsv"))
        .unwrap();
    writeln!(metrics, "{name}\t{:.2}", elapsed.as_secs_f64() * 1000.).unwrap();
    if name == "010-home-continue.png" {
        comparison("ember-home.png", &buffer);
    }
    if name == "010-play-setup-auger.png" {
        comparison("ember-play-setup-auger.png", &buffer);
    }
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
/// A minimal valid PWAD whose only lump is `STBAR` (spec 017): the synthetic
/// widescreen status bar the journey installs, never the real collection file.
fn status_bar_wad() -> Vec<u8> {
    let mut bytes = b"PWAD".to_vec();
    bytes.extend_from_slice(&1_i32.to_le_bytes()); // one lump
    bytes.extend_from_slice(&12_i32.to_le_bytes()); // directory follows the header
    bytes.extend_from_slice(&12_i32.to_le_bytes()); // lump offset (unused)
    bytes.extend_from_slice(&0_i32.to_le_bytes()); // lump size
    let mut name = [0_u8; 8];
    name[..5].copy_from_slice(b"STBAR");
    bytes.extend_from_slice(&name);
    bytes
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
    let mut auger = catalog
        .recipes
        .iter()
        .find(|r| r.recipe.id == "auger-zenith")
        .unwrap()
        .clone();
    let auger_package = synthetic_package(&mut auger.recipe);
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
    // Spec 017: the SIGIL recipe now carries an optional widescreen status bar as
    // a second, bare-WAD package, default-on. Serve a synthetic WAD so the
    // journey stays offline while the toggle and the launch list render.
    let wide_bytes = status_bar_wad();
    if let Some(wide) = catalog.recipes[0]
        .recipe
        .packages
        .iter_mut()
        .find(|p| p.id == "widehud")
    {
        wide.size = wide_bytes.len() as u64;
        wide.sha256 = Sha256::of(&wide_bytes).to_hex();
        wide.members[0].size = wide_bytes.len() as u64;
        wide.members[0].sha256 = Sha256::of(&wide_bytes).to_hex();
    }
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
        } else if request.target == "/auger.zip" {
            Response::ok(auger_package.clone())
        } else if request.target == "/wide.wad" {
            Response::ok(wide_bytes.clone())
        } else {
            pg.wait();
            Response::ok(package_body.clone())
        }
    });
    let url = server.url("/sigil.zip");
    catalog.recipes[0].recipe.packages[0].urls = vec![url];
    catalog.recipes[0].recipe.packages[0].size = package.len() as u64;
    catalog.recipes[0].recipe.packages[0].sha256 = Sha256::of(&package).to_hex();
    if let Some(wide) = catalog.recipes[0]
        .recipe
        .packages
        .iter_mut()
        .find(|p| p.id == "widehud")
    {
        wide.urls = vec![server.url("/wide.wad")];
    }
    catalog.hosts.downloads.push("127.0.0.1".into());
    auger.recipe.packages[0].urls = vec![server.url("/auger.zip")];
    catalog.recipes.push(auger);
    catalog.recipes[1].recipe.packages[0].urls = vec![server.url("/bad.zip")];
    catalog.recipes[1].recipe.packages[0].size = package.len() as u64;
    let reviewed=dsda::ReviewedList::parse(&format!("schema_version = 1\n[[version]]\ntag = 'v0.30.1'\nasset = 'test-linux.AppImage'\nsize = {}\nsha256 = '{}'\nlicense = 'GPL-2.0-or-later'\nrelease_url = 'https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.1'\nsource_url = 'https://github.com/dsda-org/dsda-doom/tree/v0.30.1'\n",engine_bytes.len(),Sha256::of(&engine_bytes).to_hex())).unwrap();
    let play_gate = Arc::new(Gate::new());
    let fake = Arc::new(
        dsda::FakeEngineRunner::new(dsda::LaunchExit::Failed { code: 255 })
            .with_log_path(paths.state.join("logs/fake-engine.log")),
    );
    let exit = Arc::new(std::sync::atomic::AtomicI32::new(255));
    let api = Arc::new(dsda::CannedApi::new());
    let test = Arc::new(TestEnvironment {
        api: api.clone(),
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
    assert!(ui.get_nodes().iter().any(|n| n.kind == 11));
    capture(window, "010-loading.png");
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::FirstRun);
    assert!(paths.config.join("dsda/active.cfg").is_file());
    capture(window, "010-first-run.png");
    for (screen, name) in [
        (Screen::Home, "010-home-empty.png"),
        (Screen::Library, "010-library-empty.png"),
    ] {
        state.borrow_mut().screen = screen;
        state.borrow_mut().sync(ui);
        capture(window, name);
    }
    state.borrow_mut().screen = Screen::FirstRun;
    state.borrow_mut().focus = 1;
    state.borrow_mut().sync(ui);
    ui.invoke_action(10);
    capture(window, "010-quit-dialog.png");
    press(window, Key::Escape);
    // A requests real reviewed install on a worker; gate HTTP while rendering progress.
    press(window, Key::Return);
    wait(&state, |r| r.progress.is_some());
    capture(window, "010-first-run-engine-install.png");
    assert_eq!(state.borrow().screen, Screen::Progress);
    press(window, Key::Return);
    assert_eq!(ui.get_focused(), 900);
    capture(window, "010-cancel-dialog.png");
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
    let entries = state.borrow().directory.as_ref().unwrap().entries.clone();
    let first_hidden = entries
        .iter()
        .position(|(p, _)| crate::services::hidden_path(p))
        .unwrap();
    assert!(
        entries[..first_hidden]
            .iter()
            .all(|(p, _)| !crate::services::hidden_path(p))
    );
    assert!(
        entries[first_hidden..]
            .iter()
            .all(|(p, _)| crate::services::hidden_path(p))
    );
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label.starts_with('.') && n.tone == 1)
    );
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
    {
        let mut r = state.borrow_mut();
        let saved = r.snapshot.clone();
        let snapshot = r.snapshot.as_mut().unwrap();
        let template = snapshot.items[1].clone();
        snapshot.items = Catalog::embedded()
            .recipes
            .into_iter()
            .map(|recipe| {
                let mut item = template.clone();
                item.recipe = recipe.recipe;
                item
            })
            .collect();
        r.sync(ui);
        assert!(
            r.snapshot
                .as_ref()
                .unwrap()
                .items
                .iter()
                .any(|i| i.recipe.id == "eviternity")
        );
        assert!(ui.get_nodes().iter().any(|n| n.kind == 4 && n.art == 3));
        capture(window, "010-discover.png");
        // Render-only community entry: no catalog or install-state mutation.
        let item = &mut r.snapshot.as_mut().unwrap().items[0];
        item.recipe.id = "community-without-art".into();
        item.recipe.title = "Community map (render fixture)".into();
        item.recipe.authors = vec!["Render fixture — not catalog metadata".into()];
        r.sync(ui);
        assert!(ui.get_nodes().iter().any(|n| n.kind == 4 && n.art == 4));
        assert_eq!(ui.get_band_art(), 4);
        capture(window, "013-discover-fallback.png");
        r.snapshot.as_mut().unwrap().items[0].recipe.id = "eviternity".into();
        r.snapshot.as_mut().unwrap().items[0].recipe.title = "Eviternity (render fixture)".into();
        r.sync(ui);
        assert!(ui.get_nodes().iter().any(|n| n.kind == 4 && n.art == 3));
        assert_eq!(ui.get_band_art(), 3);
        capture(window, "013-discover-eviternity.png");
        r.snapshot = saved;
        r.sync(ui);
    }
    ui.invoke_clicked(11);
    assert_eq!(state.borrow().screen, Screen::Detail);
    assert_eq!(ui.get_heading(), "Details / SIGIL");
    capture(window, "010-mapset-detail.png");
    ui.invoke_clicked(1);
    wait(&state, |r| r.progress.is_some() || !r.busy);
    assert_eq!(state.borrow().screen, Screen::Progress);
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label.contains("Download → Extract → Verify → Install"))
    );
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label.contains(" B") || n.label.contains("KiB"))
    );
    capture(window, "010-install-progress.png");
    // Fast local stages are represented deterministically while HTTP remains gated.
    let saved_progress = state.borrow().progress;
    for (stage, name) in [
        (
            ddl_core::downloads::InstallStage::Verify,
            "010-install-verify.png",
        ),
        (
            ddl_core::downloads::InstallStage::Extract,
            "010-install-extract.png",
        ),
        (
            ddl_core::downloads::InstallStage::Commit,
            "010-install-commit.png",
        ),
    ] {
        state.borrow_mut().progress = Some(ddl_core::downloads::InstallProgress::stage(stage));
        state.borrow_mut().sync(ui);
        capture(window, name);
    }
    state.borrow_mut().progress = saved_progress;
    state.borrow_mut().sync(ui);
    package_gate.release();
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::Setup);
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[1]
            .launch
            .is_ok()
    );
    assert_eq!(ui.get_heading(), "Play setup / SIGIL");
    // Spec 017: the widescreen status bar is default-on, so SIGIL launches two
    // content files, the mapset then the bar.
    assert!(ui.get_nodes().iter().any(|n| n.label == "2 content files"));
    let bar_toggle = ui
        .get_nodes()
        .iter()
        .find(|n| n.label == "Widescreen status bar")
        .expect("the optional bar toggle appears on Play setup");
    assert_eq!(bar_toggle.detail, "On");
    assert!(bar_toggle.actionable);
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label.starts_with("2  doom_uwide.wad")),
        "the wide bar is the last -file entry"
    );
    let title = ui
        .get_nodes()
        .iter()
        .find(|n| n.label == "Will launch")
        .unwrap();
    assert_eq!(title.y, 36.);
    capture(window, "010-play-setup-ready.png");
    // Profile values persist; advanced replacement never combines with main.
    ui.invoke_clicked(40);
    idle(&state);
    ui.invoke_clicked(41);
    idle(&state);
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[1]
            .profile
            .skill,
        4
    );
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[1]
            .profile
            .pistol_start
    );
    ui.invoke_clicked(42);
    ui.invoke_clicked(101);
    assert!(ui.get_sheet_open());
    capture(window, "010-compat-dialog.png");
    press(window, Key::Escape);
    // The default-on wide bar survives a cancelled compat confirmation.
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[1]
            .profile
            .components,
        vec!["wide-status-bar"]
    );
    ui.invoke_clicked(101);
    press(window, Key::RightArrow);
    press(window, Key::Return);
    idle(&state);
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[1]
            .profile
            .components,
        vec!["wide-status-bar", "compat"]
    );
    ui.invoke_clicked(2);
    idle(&state);
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[1]
            .launch
            .as_ref()
            .unwrap()
            .content[0]
            .path
            .ends_with("SIGIL_COMPAT_V1_23.wad")
    );
    ui.invoke_clicked(101);
    idle(&state);
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[1]
            .profile
            .components,
        vec!["wide-status-bar"]
    );
    // The directory browser exposes local WAD imports, and an unknown paid combination stays off.
    std::fs::write(
        home.join("purchased.wad"),
        b"Unrecognized synthetic soundtrack",
    )
    .unwrap();
    ui.invoke_clicked(102);
    idle(&state);
    capture(window, "010-import-browser.png");
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
    assert_eq!(
        Services::open(env.clone())
            .unwrap()
            .snapshot()
            .unwrap()
            .items[1]
            .profile
            .components,
        vec!["wide-status-bar"]
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
    let panel = ui
        .get_nodes()
        .iter()
        .find(|n| n.kind == 1 && n.x == 12.)
        .unwrap();
    assert!(
        panel.y + panel.h >= focused.y + focused.h + 24.,
        "the carved bottom rail stays below the complete options list"
    );
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
        state.borrow().snapshot.as_ref().unwrap().items[1]
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
    assert!(ui.get_nodes().iter().any(|n| n.id == 11));
    assert!(!ui.get_nodes().iter().any(|n| n.id == 12));
    capture(window, "010-library.png");
    // Spec 015, R5: X marks the focused installed mapset; Library shows the star and sorts it first.
    let marks = |ui: &AppWindow| ui.get_nodes().iter().filter(|n| n.kind == 12).count();
    assert_eq!(marks(ui), 0);
    press_char(window, "x");
    idle(&state);
    assert_eq!(marks(ui), 1);
    assert!(ui.get_favorite_hint());
    assert_eq!(ui.get_favorite_label(), "Unfavorite");
    capture(window, "015-library-favorite.png");
    press_char(window, "x");
    idle(&state);
    assert_eq!(marks(ui), 0);
    press(window, Key::F1);
    assert_eq!(state.borrow().screen, Screen::Home);
    assert!(ui.get_nodes().iter().any(|n| n.label == "Recently played"));
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "Ultimate DOOM found")
    );
    assert!(ui.get_nodes().iter().any(|n| n.label == "DOOM II missing"));
    let checks: Vec<_> = ui.get_nodes().iter().filter(|n| n.kind == 7).collect();
    assert_eq!(
        checks.iter().map(|n| n.y).collect::<Vec<_>>(),
        [100., 172., 244., 316.]
    );
    assert_eq!(
        checks.iter().map(|n| n.tone).collect::<Vec<_>>(),
        [2, 2, 4, 3]
    );
    assert!(ui.get_nodes().iter().any(|n| n.id == 2 && n.y == 402.));
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
        state.borrow().snapshot.as_ref().unwrap().items[1]
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
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "Check when you are ready. No update check has run.")
    );
    capture(window, "010-settings.png");
    for (focus, name) in [
        (2, "010-settings-folders.png"),
        (3, "010-settings-config.png"),
        (4, "010-settings-diagnostics.png"),
    ] {
        state.borrow_mut().focus = focus;
        state.borrow_mut().sync(ui);
        capture(window, name);
        if focus == 4 {
            assert!(ui.get_nodes().iter().any(|n| n.kind == 10));
            let credit = ui
                .get_nodes()
                .iter()
                .find(|n| n.label.starts_with("Widescreen status bars:"))
                .expect("About credits the status-bar collection");
            assert!(
                credit
                    .label
                    .contains("NightFright and contributors (GPL-3.0)")
            );
            assert!(
                credit
                    .label
                    .contains("https://github.com/NightFright2k19/doom_widehud")
            );
            let panel = ui
                .get_nodes()
                .iter()
                .find(|n| n.kind == 1 && n.x == 348.)
                .unwrap();
            assert!(credit.y + credit.h + 24. <= panel.y + panel.h);
            capture(window, "013-about.png");
        }
    }
    // Spec 021 A3: the opt-in Settings row renders (existing Ember row) and
    // persists, defaulting off.
    state.borrow_mut().focus = 2;
    state.borrow_mut().sync(ui);
    let row = ui
        .get_nodes()
        .iter()
        .find(|n| n.id == 72)
        .expect("the standard-folders row renders");
    assert_eq!(row.label, "Search standard Linux folders");
    assert_eq!(row.detail, "Off");
    assert!(
        !state
            .borrow()
            .snapshot
            .as_ref()
            .unwrap()
            .search_standard_folders
    );
    ui.invoke_clicked(72);
    idle(&state);
    assert!(
        state
            .borrow()
            .snapshot
            .as_ref()
            .unwrap()
            .search_standard_folders
    );
    assert_eq!(
        ui.get_nodes().iter().find(|n| n.id == 72).unwrap().detail,
        "On"
    );
    ui.invoke_clicked(72);
    idle(&state);
    assert!(
        !state
            .borrow()
            .snapshot
            .as_ref()
            .unwrap()
            .search_standard_folders
    );
    state.borrow_mut().focus = 1;
    state.borrow_mut().sync(ui);
    assert!(
        api.requests().is_empty(),
        "startup and local actions never check online releases"
    );
    api.push_error(dsda::ApiError::new("synthetic offline"));
    ui.invoke_clicked(70);
    wait(&state, |r| !r.busy);
    assert_eq!(state.borrow().problem, Some(Problem::Update));
    assert!(state.borrow().snapshot.as_ref().unwrap().engine.is_some());
    press(window, Key::Escape);
    api.push(dsda::ApiResponse {
        status: 200,
        etag: Some("test".into()),
        body: "[]".into(),
    });
    ui.invoke_clicked(70);
    idle(&state);
    assert_eq!(api.requests().len(), 2);
    let mut installed = dsda::InstalledState::load(
        &dsda::EngineDirs::for_paths(&paths)
            .engines
            .join(dsda::STATE_FILE),
    )
    .unwrap();
    let mut previous = installed.active_entry().unwrap().clone();
    previous.version = "0.30.0".into();
    previous.tag = Some("v0.30.0".into());
    installed.insert(previous);
    installed.previous_good = Some("0.30.0".into());
    installed
        .save(
            &dsda::EngineDirs::for_paths(&paths)
                .engines
                .join(dsda::STATE_FILE),
        )
        .unwrap();
    let profile = state.borrow().snapshot.as_ref().unwrap().items[1]
        .profile
        .clone();
    state.borrow_mut().start(ui, Command::Save(profile));
    idle(&state);
    ui.invoke_clicked(71);
    idle(&state);
    assert_eq!(
        state
            .borrow()
            .snapshot
            .as_ref()
            .unwrap()
            .engine
            .as_ref()
            .unwrap()
            .version,
        "0.30.0"
    );
    ui.invoke_clicked(71);
    idle(&state);
    assert_eq!(
        state
            .borrow()
            .snapshot
            .as_ref()
            .unwrap()
            .engine
            .as_ref()
            .unwrap()
            .version,
        "0.30.1"
    );

    ui.invoke_clicked(3);
    ui.invoke_clicked(70);
    capture(window, "010-reset-dialog.png");
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
    capture(window, "010-remove-folder-dialog.png");
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
    state.borrow_mut().selected = 2;
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
        state.borrow().snapshot.as_ref().unwrap().items[1]
            .manifest
            .is_some()
    );
    // Restart uses stored manifest/recipe and profile, with no discovery download.
    let restarted = Services::open(env.clone()).unwrap().snapshot().unwrap();
    assert_eq!(restarted.items[1].profile.skill, 4);
    assert!(restarted.items[1].manifest.is_some());
    assert!(restarted.items[1].profile.last_played.is_some());
    // Actual AUGER resolver/install/profile journey: no borrowed ready summary.
    press(window, Key::Escape);
    let doom2 = home.join("synthetic-doom2.wad");
    std::fs::write(&doom2, commercial_wad()).unwrap();
    state
        .borrow_mut()
        .start(ui, Command::Scan(Some(home.clone())));
    idle(&state);
    let doom2_entry = state
        .borrow()
        .snapshot
        .as_ref()
        .unwrap()
        .iwads
        .iter()
        .find(|e| e.path.ends_with("synthetic-doom2.wad"))
        .cloned()
        .unwrap();
    assert!(doom2_entry.candidates.iter().any(|id| id == "doom2-1.9"));
    state
        .borrow_mut()
        .start(ui, Command::Choose(doom2_entry.path, "doom2-1.9".into()));
    idle(&state);
    state.borrow_mut().screen = Screen::Detail;
    state.borrow_mut().selected = 0;
    state.borrow_mut().focus = 1;
    state.borrow_mut().sync(ui);
    ui.invoke_clicked(1);
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::Setup);
    assert_eq!(ui.get_heading(), "Play setup / AUGER;ZENITH");
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.kind == 9 && n.label == "Add-on — choose one")
    );
    assert!(ui.get_nodes().iter().any(|n| n.id == 301 && n.primary));
    assert_eq!(
        ui.get_nodes()
            .iter()
            .filter(|n| n.kind == 8 && n.primary)
            .count(),
        1
    );
    capture(window, "010-play-setup-auger.png");
    // Left/right operates on the three choices as one compact group.
    state.borrow_mut().focus = 301;
    state.borrow_mut().sync(ui);
    press(window, Key::RightArrow);
    assert_eq!(ui.get_focused(), 101);
    press(window, Key::Return);
    idle(&state);
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .components,
        ["alternate-blaster"]
    );
    if state.borrow().snapshot.as_ref().unwrap().items[0]
        .launch
        .is_err()
    {
        ui.invoke_clicked(2);
        idle(&state);
    }
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "2  DBP37_AlternateBlaster.wad")
    );
    assert!(ui.get_nodes().iter().any(|n| n.label == "2 content files"));
    let summary = state.borrow().snapshot.as_ref().unwrap().items[0]
        .launch
        .clone()
        .unwrap();
    assert!(summary.content[0].path.ends_with("DBP37_AUGZEN.wad"));
    assert!(
        summary.content[1]
            .path
            .ends_with("DBP37_AlternateBlaster.wad")
    );
    assert_eq!(summary.content[1].position, 2);
    state.borrow_mut().focus = 101;
    state.borrow_mut().sync(ui);
    assert_eq!(
        ui.get_nodes()
            .iter()
            .filter(|n| n.kind == 8 && n.primary)
            .map(|n| n.id)
            .collect::<Vec<_>>(),
        [101]
    );
    capture(window, "010-play-setup-auger-alternate.png");
    // A on the selected choice is idempotent, and a different choice replaces it.
    press(window, Key::Return);
    idle(&state);
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .components,
        ["alternate-blaster"]
    );
    press(window, Key::RightArrow);
    assert_eq!(ui.get_focused(), 102);
    press(window, Key::Return);
    idle(&state);
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .components,
        ["deckard-blaster"]
    );
    if state.borrow().snapshot.as_ref().unwrap().items[0]
        .launch
        .is_err()
    {
        ui.invoke_clicked(2);
        idle(&state);
    }
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "2  DBP37_DeckardBlaster.wad")
    );
    state.borrow_mut().focus = 102;
    state.borrow_mut().sync(ui);
    assert_eq!(
        ui.get_nodes()
            .iter()
            .filter(|n| n.kind == 8 && n.primary)
            .map(|n| n.id)
            .collect::<Vec<_>>(),
        [102]
    );
    capture(window, "010-play-setup-auger-deckard.png");
    ui.invoke_clicked(301);
    idle(&state);
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .components
            .is_empty()
    );
    assert!(ui.get_nodes().iter().any(|n| n.label == "1 content file"));
    assert_eq!(
        Services::open(env.clone())
            .unwrap()
            .snapshot()
            .unwrap()
            .items[0]
            .launch
            .as_ref()
            .unwrap()
            .content
            .len(),
        1
    );
    profiles_journey(window, ui, &state, &env, &fake);
}

/// Spec 018: real keys, production services, independent save identity and Continue.
fn profiles_journey(
    window: &MinimalSoftwareWindow,
    ui: &AppWindow,
    state: &Rc<RefCell<Runtime>>,
    env: &Environment,
    fake: &dsda::FakeEngineRunner,
) {
    // Current mapset is installed AUGER;ZENITH, with all game prerequisites ready.
    state.borrow_mut().focus = 60;
    state.borrow_mut().sync(ui);
    press(window, Key::Return);
    idle(state);
    assert!(ui.get_sheet_open());
    assert_eq!(ui.get_focused(), 912);
    assert!(
        !ui.get_sheet_nodes()
            .iter()
            .find(|n| n.id == 915)
            .unwrap()
            .actionable
    );
    press(window, Key::Return); // New — copy settings
    idle(state);
    assert_eq!(ui.get_focused(), 60);
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "Active profile: Profile 2")
    );
    let copied = state.borrow().snapshot.as_ref().unwrap().items[0]
        .profile
        .clone();
    assert_eq!(copied.name, "Profile 2");
    press(window, Key::Return);
    idle(state);
    capture(window, "018-profile-picker.png");
    // At the last profile, focus begins on copy; Up reaches Previous.
    press(window, Key::UpArrow);
    assert_eq!(ui.get_focused(), 910);
    press(window, Key::Return);
    idle(state);
    assert!(ui.get_sheet_title().contains("Default"));
    assert_eq!(ui.get_focused(), 911); // Disabled Previous cannot strand focus.
    press(window, Key::Return);
    idle(state);
    assert!(ui.get_sheet_title().contains("Profile 2"));
    press(window, Key::Escape);
    assert_eq!(ui.get_focused(), 60);
    // Rename via a D-pad preset, without text entry.
    press(window, Key::Return);
    idle(state);
    press(window, Key::DownArrow); // copy -> rename
    assert_eq!(ui.get_focused(), 914);
    press(window, Key::Return);
    assert_eq!(ui.get_focused(), 910);
    capture(window, "018-profile-names.png");
    press(window, Key::RightArrow);
    press(window, Key::Return); // Casual
    idle(state);
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "Active profile: Casual")
    );
    // Confirm names the profile and its data; B preserves both and restores focus.
    press(window, Key::Return);
    idle(state);
    press(window, Key::DownArrow);
    press(window, Key::RightArrow);
    assert_eq!(ui.get_focused(), 915);
    press(window, Key::Return);
    assert_eq!(ui.get_focused(), 900);
    assert!(ui.get_sheet_title().contains("Casual"));
    assert!(ui.get_sheet_title().contains("saves and screenshots"));
    capture(window, "018-delete-confirm.png");
    press(window, Key::Escape);
    assert_eq!(ui.get_focused(), 60);
    let store = ddl_core::profiles::ProfileStore::new(&env.paths.config, &env.paths.data);
    assert!(store.load(&copied.id).is_ok());
    // Play and Home Continue both use Casual's actual isolated runtime directories.
    let before_play = fake.launches().len();
    ui.invoke_clicked(50);
    idle(state);
    state.borrow_mut().neutral_poll(true);
    assert_eq!(fake.launches().len(), before_play + 1);
    assert_profile_dirs(fake, &store.dirs(&copied.id));
    {
        let mut r = state.borrow_mut();
        r.screen = Screen::Home;
        r.focus = 1;
        r.sync(ui);
    }
    assert!(ui.get_nodes().iter().any(|n| n.label.contains("Casual")));
    capture(window, "018-home-continue.png");
    let before_continue = fake.launches().len();
    press(window, Key::Return); // Continue opens setup, preserving the established flow.
    assert_eq!(state.borrow().screen, Screen::Setup);
    assert_eq!(ui.get_focused(), 50);
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .id,
        copied.id
    );
    assert_eq!(fake.launches().len(), before_continue);
    press(window, Key::Return); // Play the remembered profile.
    idle(state);
    state.borrow_mut().neutral_poll(true);
    assert_eq!(fake.launches().len(), before_continue + 1);
    assert_profile_dirs(fake, &store.dirs(&copied.id));
    // Library shows the count using the same snapshot.
    {
        let mut r = state.borrow_mut();
        r.screen = Screen::Library;
        r.focus = 10;
        r.sync(ui);
    }
    assert!(ui.get_nodes().iter().any(|n| n.label == "2 profiles"));
    capture(window, "018-library-profiles.png");
    // Delete explicitly, and ensure the other profile's dirs survive.
    {
        let mut r = state.borrow_mut();
        r.screen = Screen::Setup;
        r.focus = 60;
        r.sync(ui);
    }
    press(window, Key::Return);
    idle(state);
    press(window, Key::DownArrow);
    press(window, Key::RightArrow);
    press(window, Key::Return);
    press(window, Key::RightArrow);
    press(window, Key::Return);
    idle(state);
    assert!(store.load_optional(&copied.id).unwrap().is_none());
    assert!(!store.dirs(&copied.id).root.exists());
    assert!(store.dirs("auger-zenith-default").root.exists());
}

fn assert_profile_dirs(fake: &dsda::FakeEngineRunner, dirs: &ddl_core::profiles::ProfileDirs) {
    let launches = fake.launches();
    let args = &launches.last().unwrap().argv;
    for (flag, path) in [
        ("-save", &dirs.save),
        ("-data", &dirs.data),
        ("-shotdir", &dirs.shots),
    ] {
        assert!(
            args.windows(2)
                .any(|pair| pair[0] == flag && pair[1] == path.as_os_str()),
            "{flag}: {args:?}"
        );
    }
}

fn press_char(window: &MinimalSoftwareWindow, text: &str) {
    window.dispatch_event(WindowEvent::KeyPressed { text: text.into() });
}
fn card_ids(ui: &AppWindow) -> Vec<i32> {
    ui.get_nodes()
        .iter()
        .filter(|n| n.kind == 1 && n.actionable)
        .map(|n| n.id)
        .collect()
}
fn chip_labels(ui: &AppWindow) -> Vec<String> {
    ui.get_nodes()
        .iter()
        .filter(|n| n.kind == 14 && (200..230).contains(&n.id))
        .map(|n| n.label.to_string())
        .collect()
}
fn selected_chip(ui: &AppWindow) -> String {
    ui.get_nodes()
        .iter()
        .find(|n| n.kind == 14 && (200..230).contains(&n.id) && n.primary)
        .map(|n| n.label.to_string())
        .unwrap()
}

/// Spec 015 (R4–R6): category strip, favorites, tag chips; embedded catalog, no network.
pub(super) fn categories_journey(window: &MinimalSoftwareWindow, ui: &AppWindow) {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let paths = ddl_core::platform::paths::resolve(&|key: &str| {
        (key == "HOME").then(|| home.display().to_string())
    })
    .unwrap();
    let env = Environment {
        paths,
        home,
        test: None,
    };
    let state = crate::runtime::attach_environment(ui, env.clone());
    idle(&state);
    let catalog = Catalog::embedded();
    let used: Vec<_> = catalog
        .tags
        .ordered()
        .into_iter()
        .filter(|tag| {
            catalog
                .recipes
                .iter()
                .any(|r| r.recipe.tags.contains(&tag.id))
        })
        .collect();
    assert!(used.len() >= 2, "the shipped recipes use several tags");
    {
        let mut r = state.borrow_mut();
        r.screen = Screen::Discover;
        r.focus = 10;
        r.sync(ui);
    }
    let all = card_ids(ui);
    assert_eq!(all.len(), catalog.recipes.len());
    let mut expected = vec!["All".to_owned()];
    expected.extend(used.iter().map(|t| t.label.clone()));
    expected.push("Favorites".into());
    // The strip scrolls when it is wider than the grid: the visible chips are a window
    // of the full list, and arrows mark what is hidden.
    let visible = chip_labels(ui);
    assert!(
        visible.len() >= 6 && expected.starts_with(&visible),
        "{visible:?}"
    );
    let arrow = |glyph: &str| ui.get_nodes().iter().any(|n| n.label == glyph);
    assert_eq!(arrow("›"), visible.len() < expected.len());
    assert!(!arrow("‹"));
    assert_eq!(selected_chip(ui), "All");
    assert!(ui.get_category_hint() && ui.get_favorite_hint());
    capture(window, "015-discover-all.png");

    // Up from the grid reaches the strip; Left/Right switch the category as focus moves.
    press(window, Key::UpArrow);
    assert_eq!(ui.get_focused(), 200);
    assert!(!ui.get_favorite_hint(), "X has no target on the strip");
    press(window, Key::LeftArrow);
    assert_eq!(ui.get_focused(), 200, "no edge wrap");
    let tag_cards = |id: &str| {
        let cards = card_ids(ui);
        let items = state.borrow().snapshot.as_ref().unwrap().items.clone();
        let holding: Vec<i32> = items
            .iter()
            .enumerate()
            .filter(|(_, i)| i.recipe.tags.iter().any(|t| t == id))
            .map(|(i, _)| 10 + i as i32)
            .collect();
        (cards, holding)
    };
    for (n, tag) in used.iter().enumerate() {
        press(window, Key::RightArrow);
        assert_eq!(ui.get_focused(), 201 + n as i32);
        assert_eq!(selected_chip(ui), tag.label);
        let (mut cards, mut holding) = tag_cards(&tag.id);
        cards.sort_unstable();
        holding.sort_unstable();
        assert_eq!(cards, holding, "category {}", tag.id);
        assert!(!cards.is_empty());
        if tag.id == "cacowards" {
            capture(window, "015-discover-category.png");
        }
    }
    // Triggers (L2/R2) step the same list without wrapping, from the grid too.
    ui.invoke_action(11);
    assert_eq!(selected_chip(ui), used[used.len() - 2].label);
    ui.invoke_action(12);
    assert_eq!(selected_chip(ui), used[used.len() - 1].label);
    press(window, Key::DownArrow);
    assert!(card_ids(ui).contains(&ui.get_focused()));
    ui.invoke_action(12);
    assert_eq!(selected_chip(ui), "Favorites");
    ui.invoke_action(12);
    assert_eq!(selected_chip(ui), "Favorites", "no wrap at the end");
    assert_eq!(
        chip_labels(ui).last().map(String::as_str),
        Some("Favorites")
    );
    assert_eq!(arrow("‹"), chip_labels(ui).len() < expected.len());
    assert!(!arrow("›"));
    // Empty Favorites: friendly explanation and a way forward.
    assert!(card_ids(ui).is_empty());
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label.contains("press X to keep it here"))
    );
    assert_eq!(ui.get_focused(), 200 + used.len() as i32 + 1);
    capture(window, "015-discover-favorites-empty.png");
    press(window, Key::DownArrow);
    assert_eq!(ui.get_focused(), 230);
    press(window, Key::Return);
    assert_eq!(selected_chip(ui), "All");
    assert_eq!(ui.get_focused(), card_ids(ui)[0]);

    // X on the second card: persisted, starred, and sorted first.
    press(window, Key::RightArrow);
    let second = ui.get_focused();
    assert_ne!(second, card_ids(ui)[0]);
    press_char(window, "x");
    idle(&state);
    let marked = state.borrow().snapshot.as_ref().unwrap().items[(second - 10) as usize]
        .recipe
        .id
        .clone();
    let saved = Services::open(env.clone()).unwrap().snapshot().unwrap();
    assert_eq!(
        saved
            .items
            .iter()
            .filter(|i| i.favorite)
            .map(|i| i.recipe.id.as_str())
            .collect::<Vec<_>>(),
        [marked.as_str()]
    );
    let first_card = ui
        .get_nodes()
        .iter()
        .find(|n| n.id == second)
        .map(|n| (n.x, n.y))
        .unwrap();
    assert_eq!(first_card, (12., 84.), "favorites sort first");
    assert_eq!(ui.get_nodes().iter().filter(|n| n.kind == 12).count(), 1);
    assert_eq!(ui.get_favorite_label(), "Unfavorite");
    capture(window, "015-discover-all-favorite.png");
    // The Favorites category now shows exactly that mapset.
    ui.invoke_action(12);
    assert_eq!(selected_chip(ui), used[0].label);
    for _ in 0..used.len() {
        ui.invoke_action(12);
    }
    assert_eq!(selected_chip(ui), "Favorites");
    assert_eq!(card_ids(ui), [second]);
    capture(window, "015-discover-favorites.png");

    // Detail: all tag chips, favorite mark, X and the Favorite button.
    press(window, Key::Return);
    assert_eq!(state.borrow().screen, Screen::Detail);
    let recipe = &catalog.recipe(&marked).unwrap();
    let chips: Vec<_> = ui
        .get_nodes()
        .iter()
        .filter(|n| n.kind == 13)
        .map(|n| n.label.to_string())
        .collect();
    let want: Vec<_> = catalog
        .tags
        .ordered()
        .into_iter()
        .filter(|t| recipe.tags.contains(&t.id))
        .map(|t| t.label.clone())
        .collect();
    assert_eq!(chips, want);
    assert!(chips.len() >= 2);
    assert!(ui.get_nodes().iter().any(|n| n.kind == 12));
    assert!(ui.get_favorite_hint());
    capture(window, "015-detail-tags.png");
    press_char(window, "x");
    idle(&state);
    assert!(!ui.get_nodes().iter().any(|n| n.kind == 12));
    assert_eq!(ui.get_favorite_label(), "Favorite");
    ui.invoke_clicked(3);
    idle(&state);
    assert!(ui.get_nodes().iter().any(|n| n.kind == 12));
    ui.invoke_clicked(3);
    idle(&state);
    assert!(!ui.get_nodes().iter().any(|n| n.kind == 12));
    assert!(
        Services::open(env.clone())
            .unwrap()
            .snapshot()
            .unwrap()
            .items
            .iter()
            .all(|i| !i.favorite)
    );

    // Back keeps the category; leaving Discover and returning does too.
    press(window, Key::Escape);
    assert_eq!(state.borrow().screen, Screen::Discover);
    assert_eq!(selected_chip(ui), "Favorites");
    assert!(card_ids(ui).is_empty());
    assert!(card_ids(ui).iter().all(|id| *id != ui.get_focused()));
    press(window, Key::F3);
    assert_eq!(state.borrow().screen, Screen::Library);
    press(window, Key::F2);
    assert_eq!(state.borrow().screen, Screen::Discover);
    assert_eq!(
        selected_chip(ui),
        "Favorites",
        "last category kept for the session"
    );
}

fn commercial_wad() -> Vec<u8> {
    let mut bytes = b"IWAD".to_vec();
    bytes.extend_from_slice(&32i32.to_le_bytes());
    bytes.extend_from_slice(&12i32.to_le_bytes());
    for map in 1..=32 {
        bytes.extend_from_slice(&[0; 8]);
        let mut lump = [0; 8];
        let name = format!("MAP{map:02}");
        lump[..name.len()].copy_from_slice(name.as_bytes());
        bytes.extend_from_slice(&lump);
    }
    bytes
}
/// Rebuild reviewed member paths with tiny synthetic bytes, including the nested add-on ZIP.
fn synthetic_package(recipe: &mut ddl_core::catalog::Recipe) -> Vec<u8> {
    use std::collections::BTreeMap;
    let mut outer = vec![];
    let mut nested: BTreeMap<String, Vec<(String, Vec<u8>)>> = BTreeMap::new();
    for member in &mut recipe.packages[0].members {
        let bytes = format!("Synthetic fixture member: {}", member.id).into_bytes();
        member.size = bytes.len() as u64;
        member.sha256 = Sha256::of(&bytes).to_hex();
        if let Some((archive, path)) = member.path.split_once("!/") {
            nested
                .entry(archive.into())
                .or_default()
                .push((path.into(), bytes));
        } else {
            outer.push((member.path.clone(), bytes));
        }
    }
    for (path, entries) in nested {
        outer.push((path, zip_entries(entries)));
    }
    let bytes = zip_entries(outer);
    recipe.packages[0].size = bytes.len() as u64;
    recipe.packages[0].sha256 = Sha256::of(&bytes).to_hex();
    bytes
}
fn zip_entries(entries: Vec<(String, Vec<u8>)>) -> Vec<u8> {
    use std::io::Write;
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&bytes).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

pub(super) fn recipe_changes_journey(window: &MinimalSoftwareWindow, ui: &AppWindow) {
    use crate::services::recipe_change_tests::{catalog_env, installed, setup};
    let (_temp, _server, env, _hits) = setup();
    let mut services = installed(&env);
    let snapshot = services.snapshot().unwrap();
    let mut profile = snapshot.items[0].profile.clone();
    profile
        .components
        .push(snapshot.items[0].recipe.components[1].id.clone());
    services
        .execute(
            Command::Save(profile),
            &ddl_core::downloads::CancellationToken::new(),
            &mut |_| {},
        )
        .unwrap();
    let changed = catalog_env(&env, |c| {
        let recipe = &mut c.recipes[0].recipe;
        recipe.release = "1.1".into();
        recipe.compatibility.complevel = ddl_core::catalog::Complevel::Number(11);
        recipe.components[1].id = "new-addon".into();
    });
    let state = crate::runtime::attach_environment(ui, changed.clone());
    idle(&state);
    {
        let mut r = state.borrow_mut();
        r.screen = Screen::Library;
        r.focus = 10;
        r.sync(ui);
    }
    assert!(ui.get_nodes().iter().any(|n| n.label == "Update available"));
    capture(window, "020-library-update.png");
    press(window, Key::Return);
    assert_eq!(state.borrow().screen, Screen::Detail);
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label.contains("New release 1.1") && n.label.contains("complevel"))
    );
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "Old saves may not load in the new release.")
    );
    capture(window, "020-detail-update.png");
    press(window, Key::DownArrow);
    assert_eq!(ui.get_focused(), 2);
    press(window, Key::DownArrow);
    assert_eq!(ui.get_focused(), 3);
    press(window, Key::Return);
    idle(&state);
    assert!(state.borrow().snapshot.as_ref().unwrap().items[0].update_dismissed);
    // An explicit Update remains available in detail after Keep current.
    press(window, Key::UpArrow);
    assert_eq!(ui.get_focused(), 2);
    press(window, Key::Return);
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::Setup);
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[0].recipe_status,
        ddl_core::library::RecipeStatus::Current
    );
    assert!(ui.get_toast().contains("removed add-on"));
    capture(window, "020-update-notice.png");
    let removed = catalog_env(&changed, |c| c.recipes.clear());
    let state = crate::runtime::attach_environment(ui, removed);
    idle(&state);
    {
        let mut r = state.borrow_mut();
        r.screen = Screen::Library;
        r.focus = 10;
        r.sync(ui);
    }
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "No longer in the catalog")
    );
    capture(window, "020-library-removed.png");
    press(window, Key::Return);
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "No longer in the catalog")
    );
    capture(window, "020-detail-removed.png");
    // Spec 022 R1: an install that is no longer in the catalog can still be
    // uninstalled from its detail screen.
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.id == 5 && n.label == "Uninstall"),
        "removed-catalog installs keep the Uninstall action"
    );
}

/// Spec 022 R1/R2: the detail action, the confirm dialog unchecked then checked
/// (driven by real keys), the safe Cancel default, and the unchecked path that
/// keeps profiles while removing the install files.
pub(super) fn uninstall_journey(window: &MinimalSoftwareWindow, ui: &AppWindow) {
    use crate::services::recipe_change_tests::{installed, setup};
    let (_temp, _server, env, _hits) = setup();
    let mut services = installed(&env);
    services
        .execute(
            Command::NewProfile("auger-zenith".into(), false),
            &ddl_core::downloads::CancellationToken::new(),
            &mut |_| {},
        )
        .unwrap();
    let store = ddl_core::profiles::ProfileStore::new(&env.paths.config, &env.paths.data);
    let profiles = store.list().unwrap();
    assert_eq!(profiles.len(), 2);
    for profile in &profiles {
        std::fs::write(store.dirs(&profile.id).save.join("save.dsg"), b"save").unwrap();
    }
    drop(services);

    let state = crate::runtime::attach_environment(ui, env.clone());
    idle(&state);
    let index = state
        .borrow()
        .snapshot
        .as_ref()
        .unwrap()
        .items
        .iter()
        .position(|i| i.recipe.id == "auger-zenith")
        .unwrap();
    {
        let mut r = state.borrow_mut();
        r.screen = Screen::Detail;
        r.selected = index;
        r.focus = 5;
        r.sync(ui);
    }
    let action = ui
        .get_nodes()
        .iter()
        .find(|n| n.id == 5)
        .expect("an installed mapset offers Uninstall on detail");
    assert_eq!(action.label, "Uninstall");
    capture(window, "022-detail-uninstall.png");

    press(window, Key::Return);
    assert!(ui.get_sheet_open());
    // R2: focus starts on Cancel and the checkbox is unchecked by default.
    assert_eq!(ui.get_focused(), 900);
    assert!(ui.get_sheet_title().contains("Uninstall"));
    let checkbox = |ui: &AppWindow| {
        ui.get_sheet_nodes()
            .iter()
            .find(|n| n.id == 902)
            .expect("the checkbox renders")
    };
    assert_eq!(checkbox(ui).art, 0);
    assert!(
        checkbox(ui).label.contains("2 profiles"),
        "{}",
        checkbox(ui).label
    );
    assert!(
        ui.get_sheet_nodes()
            .iter()
            .any(|n| n.label.starts_with("Installed size:")),
        "the dialog names the installed size"
    );
    assert!(
        ui.get_sheet_nodes()
            .iter()
            .any(|n| n.label.contains("Profiles and saves are kept")),
        "the unchecked dialog says what is kept"
    );
    capture(window, "022-uninstall-confirm.png");

    // Up reaches the checkbox; A toggles it without any text entry.
    press(window, Key::UpArrow);
    assert_eq!(ui.get_focused(), 902);
    press(window, Key::Return);
    assert_eq!(checkbox(ui).art, 1);
    assert!(ui.get_sheet_open(), "toggling never closes the dialog");
    assert!(
        ui.get_sheet_nodes()
            .iter()
            .any(|n| n.label.contains("will be deleted too")),
        "the checked dialog names the consequence"
    );
    capture(window, "022-uninstall-delete-profiles.png");

    // B cancels; nothing changes.
    press(window, Key::Escape);
    assert!(!ui.get_sheet_open());
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[index]
            .manifest
            .is_some()
    );

    // Confirm with the checkbox unchecked: only the install files go.
    press(window, Key::Return);
    assert_eq!(ui.get_focused(), 900);
    press(window, Key::RightArrow);
    assert_eq!(ui.get_focused(), 901);
    press(window, Key::Return);
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::Library);
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[index]
            .manifest
            .is_none(),
        "the mapset is no longer installed"
    );
    assert_eq!(store.list().unwrap().len(), 2, "profiles are kept");
    for profile in &profiles {
        assert_eq!(
            std::fs::read(store.dirs(&profile.id).save.join("save.dsg")).unwrap(),
            b"save"
        );
    }
}

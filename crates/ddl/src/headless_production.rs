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
    platform::paths::Paths,
};
use std::{
    cell::RefCell,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};

pub(super) struct Gate {
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
        let expected =
            slint::Image::load_from_svg_data(include_bytes!("../ui/assets/ember/ornament-top.svg"))
                .unwrap()
                .to_rgba8()
                .unwrap();
        assert_eq!((expected.width(), expected.height()), (96, 28));
        let mut opaque = 0;
        let x = if name == "010-install-progress.png" {
            592
        } else {
            384
        };
        for row in 0..28 {
            for col in 0..96 {
                let pixel = buffer.as_slice()[(222 + row) * WIDTH as usize + x + col];
                let reference = expected.as_slice()[row * 96 + col];
                // Transparent engraving margins expose the shared rail below.
                // Fully opaque crest pixels still verify the exact native size.
                if reference.a == 255 {
                    opaque += 1;
                    assert_eq!(
                        [pixel.r, pixel.g, pixel.b],
                        [reference.r, reference.g, reference.b],
                        "vector ornament must render at native size on {name}"
                    );
                }
            }
        }
        assert!(opaque > 300, "the crest must contain visible carved detail");
    }
    if name == "010-discover.png" {
        // Spec 030 moves the viewport down 20px and scrolls this first row
        // up 16px to reserve its full focus ring above the footer.
        for y in 298..322 {
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

/// Signature of a segment's capture sink: the real renderer/PNG writer, or a
/// no-op for the silent prefix a later segment replays to reach its state.
pub(super) type Captures<'a> = &'a dyn Fn(&str);

/// One focused headless scenario: its own scratch XDG root, its own loopback
/// HTTP fixture, its own gated engine/package/play servers and its own
/// off-screen component. No two tests share a mutable path (spec 027, R1/R3).
pub(super) struct Scene {
    _temp: tempfile::TempDir,
    _server: Server,
    pub(super) paths: Paths,
    pub(super) home: PathBuf,
    pub(super) env: Environment,
    pub(super) window: Rc<MinimalSoftwareWindow>,
    pub(super) ui: AppWindow,
    pub(super) state: Rc<RefCell<Runtime>>,
    pub(super) fake: Arc<dsda::FakeEngineRunner>,
    pub(super) api: Arc<dsda::CannedApi>,
    pub(super) exit: Arc<std::sync::atomic::AtomicI32>,
    pub(super) engine_gate: Arc<Gate>,
    pub(super) package_gate: Arc<Gate>,
    pub(super) play_gate: Arc<Gate>,
}

impl Scene {
    /// The real capture: render, assert the idle frame, write the PNG and the
    /// reference comparison.
    pub(super) fn capture(&self, name: &str) {
        capture(&self.window, name);
    }
}

/// Builds the shared scenario fixture. `setup()` installs the one Slint platform
/// this child process may have.
pub(super) fn scene() -> Scene {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    let (window, ui) = setup();
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
        } else if request.target == "/catalog.json" {
            // The journey never triggers a catalog check; a 404 keeps it honest.
            Response::status(404, "no catalog in this fixture")
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
        catalog_url: server.url("/catalog.json"),
        runner: Arc::new(GatedRunner {
            gate: play_gate.clone(),
            fake: fake.clone(),
            exit: exit.clone(),
        }),
        // The engine server withholds the response headers until `engine_gate`
        // is released, so the production 15 s header timeout could fire while
        // the test thread is starved under load (spec 024, R4). A longer
        // test-only timeout makes the gate — not the timer — decide the outcome.
        engine_response_timeout: Some(Duration::from_secs(120)),
    });
    let env = Environment {
        paths: paths.clone(),
        home: home.clone(),
        test: Some(test),
    };
    let state = crate::runtime::attach_environment(&ui, env.clone());
    Scene {
        _temp: temp,
        _server: server,
        paths,
        home,
        env,
        window,
        ui,
        state,
        fake,
        api,
        exit,
        engine_gate,
        package_gate,
        play_gate,
    }
}

/// Specs 010/024: loading, empty Home/Library, the quit dialog and the engine
/// install → cancel → retry path.
pub(super) fn first_run(scene: &Scene, capture: Captures) {
    let Scene {
        paths,
        env,
        engine_gate,
        ..
    } = scene;
    let window = scene.window.as_ref();
    let ui = &scene.ui;
    let state = scene.state.clone();
    assert!(ui.get_nodes().iter().any(|n| n.kind == 11));
    capture("010-loading.png");
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::FirstRun);
    assert!(paths.config.join("dsda/active.cfg").is_file());
    capture("010-first-run.png");
    for (screen, name) in [
        (Screen::Home, "010-home-empty.png"),
        (Screen::Library, "010-library-empty.png"),
    ] {
        state.borrow_mut().screen = screen;
        state.borrow_mut().sync(ui);
        capture(name);
    }
    state.borrow_mut().screen = Screen::FirstRun;
    state.borrow_mut().focus = 1;
    state.borrow_mut().sync(ui);
    ui.invoke_action(10);
    capture("010-quit-dialog.png");
    press(window, Key::Escape);
    // A requests real reviewed install on a worker; gate HTTP while rendering progress.
    press(window, Key::Return);
    wait(&state, |r| r.progress.is_some());
    capture("010-first-run-engine-install.png");
    assert_eq!(state.borrow().screen, Screen::Progress);
    press(window, Key::Return);
    assert_eq!(ui.get_focused(), 900);
    capture("010-cancel-dialog.png");
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
}

/// Spec 010: the controller browser, bounded scan and explicit ambiguity choice.
pub(super) fn iwad_review(scene: &Scene, capture: Captures) {
    let window = scene.window.as_ref();
    let ui = &scene.ui;
    let state = scene.state.clone();
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
    capture("010-directory-browser.png");
    press(window, Key::Escape);
    assert_eq!(state.borrow().screen, Screen::FirstRun);
    ui.invoke_clicked(2);
    idle(&state);
    ui.invoke_clicked(1);
    idle(&state);
    assert_eq!(state.borrow().screen, Screen::Review);
    capture("010-iwad-ambiguous.png");
    let candidate = state.borrow().snapshot.as_ref().unwrap().iwads[0]
        .candidates
        .iter()
        .position(|v| v == "doom-1.9ud")
        .unwrap();
    ui.invoke_clicked(10 + candidate as i32);
    idle(&state);
    capture("010-iwad-found.png");
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().iwads[0]
            .variant
            .as_deref(),
        Some("doom-1.9ud")
    );
    ui.invoke_clicked(1);
    assert_eq!(state.borrow().screen, Screen::FirstRun);
}

/// Specs 010/013: Discover cards, the render-only fallback and Eviternity
/// fixtures, and the SIGIL mapset detail.
pub(super) fn discover_and_detail(scene: &Scene, capture: Captures) {
    let ui = &scene.ui;
    let state = scene.state.clone();
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
        capture("010-discover.png");
        // Render-only community entry: no catalog or install-state mutation.
        let item = &mut r.snapshot.as_mut().unwrap().items[0];
        item.recipe.id = "community-without-art".into();
        item.recipe.screenshots.clear();
        item.recipe.title = "Community map (render fixture)".into();
        item.recipe.authors = vec!["Render fixture — not catalog metadata".into()];
        r.sync(ui);
        assert!(ui.get_nodes().iter().any(|n| n.kind == 4 && n.art == 4));
        assert_eq!(ui.get_band_art(), 4);
        capture("013-discover-fallback.png");
        r.snapshot.as_mut().unwrap().items[0].recipe =
            Catalog::embedded().recipe("eviternity").unwrap().clone();
        r.snapshot.as_mut().unwrap().items[0].recipe.title = "Eviternity (render fixture)".into();
        r.sync(ui);
        assert!(ui.get_nodes().iter().any(|n| n.kind == 4 && n.art == 3));
        assert_eq!(ui.get_band_art(), 3);
        capture("013-discover-eviternity.png");
        r.snapshot = saved;
        r.sync(ui);
    }
    ui.invoke_clicked(11);
    assert_eq!(state.borrow().screen, Screen::Detail);
    assert_eq!(ui.get_heading(), "Details / SIGIL");
    capture("010-mapset-detail.png");
}

/// Specs 010/017/024: SIGIL install progress with the gated package, the staged
/// frames and the ready Play setup.
pub(super) fn install_and_setup(scene: &Scene, capture: Captures) {
    let Scene { package_gate, .. } = scene;
    let ui = &scene.ui;
    let state = scene.state.clone();
    ui.invoke_clicked(1);
    wait(&state, |r| r.progress.is_some() || !r.busy);
    // Spec 024 R4: slow PNG captures must not race the real 15 s header timeout.
    package_gate.release();
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
    capture("010-install-progress.png");
    // Fast local stages are represented deterministically while worker events stay queued.
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
        capture(name);
    }
    state.borrow_mut().progress = saved_progress;
    state.borrow_mut().sync(ui);
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
    // Spec 046, R4: the alias with its resolved number, no verification wording.
    let complevel = ui
        .get_nodes()
        .iter()
        .find(|n| n.label.starts_with("Complevel"))
        .map(|n| n.label.to_string())
        .expect("Play setup shows the complevel");
    assert_eq!(complevel, "Complevel: Vanilla (3)");
    capture("010-play-setup-ready.png");
    // Spec 060 A2/A3: Play unfocused (primary, not amber), focused (ring), and a
    // focused Skill; the locked required component stays visibly disabled.
    let focus = state.borrow().focus;
    for (name, id) in [
        ("060-play-ready-unfocused.png", 42),
        ("060-play-ready-focused.png", 50),
        ("060-play-ready-skill-focused.png", 40),
    ] {
        state.borrow_mut().focus = id;
        state.borrow_mut().sync(ui);
        capture(name);
    }
    state.borrow_mut().focus = focus;
    state.borrow_mut().sync(ui);
}

/// Specs 010/017/025: advanced-component exclusivity, the compatibility dialog,
/// imports, scrolling, playing and the engine-exit toast.
pub(super) fn play_setup_options(scene: &Scene, capture: Captures) {
    let Scene {
        home,
        env,
        paths,
        fake,
        play_gate,
        exit,
        ..
    } = scene;
    let window = scene.window.as_ref();
    let ui = &scene.ui;
    let state = scene.state.clone();
    // Profile values persist; advanced replacement never combines with main.
    ui.invoke_clicked(40);
    idle(&state);
    ui.invoke_clicked(41);
    idle(&state);
    assert_eq!(
        state.borrow().snapshot.as_ref().unwrap().items[1]
            .profile
            .skill,
        5
    );
    assert!(
        state.borrow().snapshot.as_ref().unwrap().items[1]
            .profile
            .pistol_start
    );
    ui.invoke_clicked(42);
    ui.invoke_clicked(101);
    assert!(ui.get_sheet_open());
    capture("010-compat-dialog.png");
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
    capture("010-import-browser.png");
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
    capture("010-play-setup-options-scroll.png");
    // Playing ignores controller and pointer actions, retains run guard and restores Play.
    ui.invoke_clicked(50);
    wait(&state, |r| r.playing);
    // A silent prefix replay renders no frame between the click and the assert,
    // so wait for the worker to take the run guard before checking it.
    wait(&state, |_| {
        ddl_core::dsda_config::RunGuard::is_active(&paths.config.join("dsda")).unwrap_or(false)
    });
    capture("010-playing.png");
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
    capture("010-engine-failed-toast.png");
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
}

/// Specs 010/015: Library, the favorite star, Home Continue and its credits.
pub(super) fn library_home_and_favorite(scene: &Scene, capture: Captures) {
    let window = scene.window.as_ref();
    let ui = &scene.ui;
    let state = scene.state.clone();
    press(window, Key::Escape);
    assert_eq!(state.borrow().screen, Screen::Detail);
    press(window, Key::Escape);
    assert_eq!(state.borrow().screen, Screen::Discover);
    state
        .borrow_mut()
        .action(ui, crate::input::Action::PreviousTab);
    assert_eq!(state.borrow().screen, Screen::Library);
    assert!(ui.get_nodes().iter().any(|n| n.id == 11));
    assert!(!ui.get_nodes().iter().any(|n| n.id == 12));
    assert!(
        ui.get_nodes()
            .iter()
            .filter(|n| n.kind == 4 && n.w >= 160.)
            .all(|n| !n.screenshot.is_empty())
    );
    capture("010-library.png");
    // Spec 015, R5: X marks the focused installed mapset; Library shows the star and sorts it first.
    let marks = |ui: &AppWindow| ui.get_nodes().iter().filter(|n| n.kind == 12).count();
    assert_eq!(marks(ui), 0);
    press_char(window, "x");
    idle(&state);
    assert_eq!(marks(ui), 1);
    assert!(ui.get_favorite_hint());
    assert_eq!(ui.get_favorite_label(), "Unfavorite");
    capture("015-library-favorite.png");
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
        [100., 172., 244.]
    );
    assert_eq!(checks.iter().map(|n| n.tone).collect::<Vec<_>>(), [2, 2, 4]);
    assert!(ui.get_nodes().iter().any(|n| n.id == 2 && n.y == 330.));
    assert!(
        ui.get_nodes()
            .iter()
            .filter(|n| n.kind == 4 && n.w >= 160.)
            .all(|n| !n.screenshot.is_empty())
    );
    assert!(!ui.get_nodes().iter().any(|n| n.kind == 16));
    assert!(ui.get_nodes().iter().all(|n| {
        let label = n.label.to_lowercase();
        !label.contains("verification pending") && !label.contains("needs deck validation")
    }));
    capture("010-home-continue.png");
}

/// Specs 010/021: the missing-IWAD blocked Play setup and the Settings summary.
pub(super) fn blocked_setup_and_settings(scene: &Scene, capture: Captures) {
    let Scene { fake, .. } = scene;
    let ui = &scene.ui;
    let state = scene.state.clone();
    let owned = scene.home.join("owned");
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
    capture("010-play-setup-blocked.png");
    ui.invoke_clicked(50);
    assert_eq!(fake.launch_count(), 3);
    // Reset is confirmed and preserves a backup. Folder remove never deletes user data.
    state.borrow_mut().screen = Screen::Settings;
    state.borrow_mut().focus = 1;
    state.borrow_mut().sync(ui);
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "No update check has run.")
    );
    capture("010-settings.png");
}

/// Specs 019/021: catalog update rows and the folded Settings sections.
pub(super) fn settings_rows(scene: &Scene, capture: Captures) {
    let Scene { api, .. } = scene;
    let ui = &scene.ui;
    let state = scene.state.clone();
    // Spec 019 R5/A3: the catalog rows render headlessly and the automatic
    // toggle persists (default on).
    let catalog_row = ui
        .get_nodes()
        .iter()
        .find(|n| n.id == 74)
        .expect("the automatic-catalog-updates row renders");
    assert_eq!(catalog_row.label, "Automatic catalog updates");
    assert_eq!(catalog_row.detail, "On");
    let catalog_summary = ui
        .get_nodes()
        .iter()
        .find(|n| n.label.starts_with("Catalog:"))
        .expect("the catalog summary renders");
    assert!(
        catalog_summary.label.contains("built-in"),
        "{}",
        catalog_summary.label
    );
    assert_eq!(
        ui.get_nodes()
            .iter()
            .find(|n| n.id == 73)
            .expect("the manual catalog check renders")
            .label,
        "Check for catalog updates"
    );
    ui.invoke_clicked(74);
    idle(&state);
    assert!(
        !state
            .borrow()
            .snapshot
            .as_ref()
            .unwrap()
            .catalog
            .auto_update
    );
    assert_eq!(
        ui.get_nodes().iter().find(|n| n.id == 74).unwrap().detail,
        "Off"
    );
    ui.invoke_clicked(74);
    idle(&state);
    assert!(
        state
            .borrow()
            .snapshot
            .as_ref()
            .unwrap()
            .catalog
            .auto_update
    );
    assert_eq!(
        ui.get_nodes().iter().find(|n| n.id == 74).unwrap().detail,
        "On"
    );
    for (focus, name) in [
        (2, "010-settings-folders.png"),
        (3, "010-settings-config.png"),
        (4, "010-settings-diagnostics.png"),
    ] {
        state.borrow_mut().focus = focus;
        state.borrow_mut().sync(ui);
        capture(name);
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
            capture("013-about.png");
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
}

/// Specs 010/019/021: engine update/error/rollback, DSDA reset and folder removal.
pub(super) fn engine_updates_and_dialogs(scene: &Scene, capture: Captures) {
    let Scene {
        api, paths, home, ..
    } = scene;
    let window = scene.window.as_ref();
    let ui = &scene.ui;
    let state = scene.state.clone();
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
        &dsda::EngineDirs::for_paths(paths)
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
            &dsda::EngineDirs::for_paths(paths)
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
    capture("010-reset-dialog.png");
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
    capture("010-remove-folder-dialog.png");
    press(window, Key::Escape);
    assert!(!state.borrow().snapshot.as_ref().unwrap().folders.is_empty());
    ui.invoke_clicked(100);
    press(window, Key::RightArrow);
    press(window, Key::Return);
    idle(&state);
    assert!(state.borrow().snapshot.as_ref().unwrap().folders.is_empty());
    assert!(home.join("purchased.wad").is_file());
}

/// Spec 010: a real checksum rejection keeps the previous install.
pub(super) fn install_error(scene: &Scene, capture: Captures) {
    let Scene { env, .. } = scene;
    let ui = &scene.ui;
    let state = scene.state.clone();
    // Error is generated by a real checksum rejection, with the prior install retained.
    state.borrow_mut().screen = Screen::Detail;
    state.borrow_mut().selected = 2;
    state.borrow_mut().focus = 1;
    state.borrow_mut().sync(ui);
    ui.invoke_clicked(1);
    wait(&state, |r| !r.busy);
    assert_eq!(state.borrow().problem, Some(Problem::Checksum));
    capture("010-install-error.png");
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
    assert_eq!(restarted.items[1].profile.skill, 5);
    assert!(restarted.items[1].manifest.is_some());
    assert!(restarted.items[1].profile.last_played.is_some());
}

/// Spec 010: the actual AUGER;ZENITH resolver/install and exclusive add-on chooser.
pub(super) fn auger_setup(scene: &Scene, capture: Captures) {
    let Scene { home, env, .. } = scene;
    let window = scene.window.as_ref();
    let ui = &scene.ui;
    let state = scene.state.clone();
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
    capture("010-play-setup-auger.png");
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
    capture("010-play-setup-auger-alternate.png");
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
    capture("010-play-setup-auger-deckard.png");
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
}

/// Spec 018: real keys, production services, independent save identity and Continue.
pub(super) fn profiles(scene: &Scene, capture: Captures) {
    let Scene { env, fake, .. } = scene;
    let window = scene.window.as_ref();
    let ui = &scene.ui;
    let state = &scene.state;
    // R1: the summary is visible and Edit is reachable by spatial navigation.
    {
        let mut r = state.borrow_mut();
        r.screen = Screen::Detail;
        r.focus = 3;
        r.sync(ui);
    }
    assert!(!ui.get_nodes().iter().any(|n| n.id == 4));
    press(window, Key::RightArrow);
    assert_eq!(ui.get_focused(), 6);
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label.contains("Skill 4 · Pistol start"))
    );
    capture("031-detail-profile.png");
    press(window, Key::Return);
    assert_eq!(state.borrow().screen, Screen::Setup);
    assert_eq!(ui.get_focused(), 60);
    assert!(!ui.get_tabs_enabled());
    // Fake launcher sees exactly one request on the real field's focus gain.
    #[derive(Default)]
    struct Fake(std::cell::RefCell<Vec<(String, Vec<String>)>>);
    impl crate::runtime::osk::Launcher for Fake {
        fn launch(&self, program: &str, args: &[&str]) -> std::io::Result<()> {
            self.0
                .borrow_mut()
                .push((program.into(), args.iter().map(|s| (*s).into()).collect()));
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "fake unavailable",
            ))
        }
    }
    let launcher = Rc::new(Fake::default());
    let fake_launcher = launcher.clone();
    ui.on_profile_name_focused(move || {
        let _ = crate::runtime::osk::focused(fake_launcher.as_ref());
    });
    press(window, Key::Return);
    idle(state);
    assert!(ui.get_sheet_open());
    assert!(
        !ui.get_sheet_nodes()
            .iter()
            .find(|n| n.id == 915)
            .unwrap()
            .actionable
    );
    ui.invoke_clicked(912); // Copy settings; numbered default immediately opens input.
    idle(state);
    assert!(ui.get_profile_name_open());
    assert_eq!(ui.get_profile_name(), "Profile 2");
    render(window); // Apply focus callbacks before synthesizing keyboard events.
    assert_eq!(
        &*launcher.0.borrow(),
        &[("steam".into(), vec!["steam://open/keyboard".into()])]
    );
    let copied = state.borrow().snapshot.as_ref().unwrap().items[0]
        .profile
        .clone();
    // Real TextInput accepts virtual keyboard events, including root shortcut letters.
    window.dispatch_event(WindowEvent::KeyPressed {
        text: "  Casual xy[]  ".into(),
    });
    assert_eq!(ui.get_profile_name(), "  Casual xy[]  ");
    capture("031-name-input.png");
    press(window, Key::Return);
    idle(state);
    assert!(!ui.get_profile_name_open());
    assert!(
        ui.get_nodes()
            .iter()
            .any(|n| n.label == "Active profile: Casual xy[]")
    );
    // Rename to the existing default name: stays open and shows the reason inline.
    ui.invoke_clicked(60);
    idle(state);
    ui.invoke_clicked(914);
    render(window);
    window.dispatch_event(WindowEvent::KeyPressed {
        text: "default".into(),
    });
    press(window, Key::Return);
    assert!(ui.get_profile_name_open());
    assert!(ui.get_profile_name_error().contains("already exists"));
    capture("031-name-invalid.png");
    ui.set_profile_name("Casual".into());
    ui.invoke_clicked(917);
    idle(state);
    // Selecting rows with A activates them; L1/R1 wrap within the sheet.
    ui.invoke_clicked(60);
    idle(state);
    capture("031-profile-list.png");
    let copied_id = copied.id.clone();
    let active = || {
        state.borrow().snapshot.as_ref().unwrap().items[0]
            .profile
            .id
            .clone()
    };
    assert!(
        !ui.get_sheet_nodes()
            .iter()
            .any(|n| matches!(n.id, 910 | 911) || n.label.contains("active marked"))
    );
    press(window, Key::UpArrow); // second row -> first
    press(window, Key::Return);
    idle(state);
    assert_ne!(active(), copied_id);
    press(window, Key::DownArrow); // first row -> second
    press(window, Key::Return);
    idle(state);
    assert_eq!(active(), copied_id);
    press(window, Key::DownArrow); // list -> grouped actions, without an empty row
    assert_eq!(ui.get_focused(), 912);
    let nodes: Vec<_> = ui.get_sheet_nodes().iter().collect();
    let row = nodes.iter().find(|n| n.id == 1001).unwrap();
    let action = nodes.iter().find(|n| n.id == 912).unwrap();
    assert_eq!(action.y - (row.y + row.h), 26.);
    press(window, Key::UpArrow);
    assert_eq!(ui.get_focused(), 1001);
    press(window, Key::Tab);
    idle(state); // R1 mapping
    assert_ne!(active(), copied_id);
    state
        .borrow_mut()
        .action(ui, crate::input::Action::PreviousTab);
    idle(state);
    assert_eq!(active(), copied_id);
    assert!(ui.get_profile_hint());
    press(window, Key::Escape);
    assert_eq!(ui.get_focused(), 60);
    press(window, Key::Tab);
    idle(state);
    assert_ne!(active(), copied_id);
    state
        .borrow_mut()
        .action(ui, crate::input::Action::PreviousTab);
    idle(state);
    assert_eq!(active(), copied_id);
    assert_eq!(state.borrow().screen, Screen::Setup);
    press(window, Key::Return);
    idle(state);
    ui.invoke_clicked(915);
    // Confirm names the profile and its data; B preserves both and restores focus.
    assert_eq!(ui.get_focused(), 900);
    assert!(ui.get_sheet_title().contains("Casual"));
    assert!(ui.get_sheet_title().contains("saves and screenshots"));
    capture("018-delete-confirm.png");
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
    capture("018-home-continue.png");
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
    capture("018-library-profiles.png");
    // Delete explicitly, and ensure the other profile's dirs survive.
    {
        let mut r = state.borrow_mut();
        r.screen = Screen::Setup;
        r.focus = 60;
        r.sync(ui);
    }
    press(window, Key::Return);
    idle(state);
    ui.invoke_clicked(915);
    press(window, Key::RightArrow);
    press(window, Key::Return);
    idle(state);
    assert!(store.load_optional(&copied.id).unwrap().is_none());
    assert!(!store.dirs(&copied.id).root.exists());
    assert!(store.dirs("auger-zenith-default").root.exists());
    // Creation Back retains numbered names; long lists remain reachable by D-pad.
    for _ in 0..4 {
        ui.invoke_clicked(60);
        idle(state);
        ui.invoke_clicked(913);
        idle(state);
        assert!(ui.get_profile_name_open());
        press(window, Key::Escape);
    }
    ui.invoke_clicked(60);
    idle(state);
    for _ in 0..4 {
        press(window, Key::UpArrow);
    }
    assert_eq!(ui.get_focused(), 1000);
    for expected in 1001..=1004 {
        press(window, Key::DownArrow);
        assert_eq!(ui.get_focused(), expected);
    }
    assert!(ui.get_sheet_nodes().iter().any(|n| n.id == 1004));
    press(window, Key::Return);
    idle(state);
    capture("031-profile-list-scrolled.png");
    press(window, Key::Escape);
}

/// Spec 022 review fixes: each Mapset detail action appears once, exactly one is
/// primary, and the visible actions are stacked with no empty slot (a 68 px pitch
/// from y=298 whether or not the conditional Update action is present).
fn assert_detail_actions(ui: &AppWindow) {
    let mut actions: Vec<_> = ui
        .get_nodes()
        .iter()
        .filter(|n| n.kind == 2 && (1..=5).contains(&n.id))
        .collect();
    actions.sort_by(|a, b| a.y.total_cmp(&b.y));
    let labels: Vec<String> = actions.iter().map(|n| n.label.to_string()).collect();
    let unique: std::collections::BTreeSet<&String> = labels.iter().collect();
    assert_eq!(
        unique.len(),
        labels.len(),
        "duplicate detail action: {labels:?}"
    );
    assert_eq!(
        actions.iter().filter(|n| n.primary).count(),
        1,
        "exactly one primary detail action: {labels:?}"
    );
    let ys: Vec<f32> = actions.iter().map(|n| n.y).collect();
    assert_eq!(
        ys.first().copied(),
        Some(298.),
        "the first action starts the column: {ys:?}"
    );
    for pair in ys.windows(2) {
        assert_eq!(
            pair[1] - pair[0],
            68.,
            "actions are evenly stacked with no gap: {ys:?}"
        );
    }
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
    }
    .without_catalog_auto_updates();
    let state = crate::runtime::attach_environment(ui, env.clone());
    idle(&state);
    {
        let state = state.borrow();
        let snapshot = state.snapshot.as_ref().unwrap();
        assert!(!snapshot.catalog.auto_update, "the fixture stays offline");
        assert!(
            !snapshot.catalog.updated,
            "only the embedded catalog is used"
        );
        assert_eq!(
            snapshot.catalog.last_check, None,
            "no HTTP check was attempted"
        );
    }
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
    // The grid draws the cards that fit the viewport, and the active catalog may be a
    // fetched bundle rather than the embedded one (spec 019), so compare the grid with
    // the snapshot instead of the embedded recipe count.
    let shown = state
        .borrow()
        .snapshot
        .as_ref()
        .map_or(0, |s| s.items.len());
    assert!(shown > 0, "Discover has recipes");
    assert!(
        !all.is_empty() && all.len() <= shown,
        "the grid draws cards"
    );
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

    // Spec 023 R3/A2, spec 033 R4: every card that ships a screenshot resolves to the
    // embedded image with no caption over it; the credit is plain text on Mapset detail.
    // Spec 037 R4: a recipe may ship none and fall back to the painted concept art, so
    // the cards under test are the active catalog's recipes that declare a screenshot.
    let cards: Vec<ddl_core::catalog::Recipe> = state
        .borrow()
        .snapshot
        .as_ref()
        .map(|s| s.items.iter().map(|i| i.recipe.clone()).collect())
        .unwrap_or_default();
    for recipe in cards.iter().filter(|r| !r.screenshots.is_empty()) {
        let index = state
            .borrow()
            .snapshot
            .as_ref()
            .unwrap()
            .items
            .iter()
            .position(|i| i.recipe.id == recipe.id)
            .unwrap();
        let asset = crate::screenshots::presentation(recipe);
        assert!(
            !asset.is_empty(),
            "missing embedded screenshot for {}",
            recipe.id
        );
        let credit = format!(
            "Screenshots: {}",
            crate::screenshots::recipe_credits(recipe).join(", ")
        );
        assert!(credit.contains(&recipe.screenshots[0].credit));
        let image = ui
            .global::<crate::ScreenshotAssets>()
            .invoke_image(asset.clone());
        let size = image.size();
        assert!(
            (ddl_core::catalog::MIN_SCREENSHOT_WIDTH..=ddl_core::catalog::MAX_SCREENSHOT_WIDTH)
                .contains(&size.width)
                && (ddl_core::catalog::MIN_SCREENSHOT_HEIGHT
                    ..=ddl_core::catalog::MAX_SCREENSHOT_HEIGHT)
                    .contains(&size.height),
            "{} screenshot is {}x{} outside catalog bounds",
            recipe.id,
            size.width,
            size.height
        );
        {
            let mut r = state.borrow_mut();
            r.focus = 10 + index as i32;
            r.selected = index;
            r.sync(ui);
        }
        assert!(
            ui.get_nodes()
                .iter()
                .any(|n| n.kind == 4 && n.screenshot == asset)
        );
        assert_eq!(ui.get_band_screenshot(), asset);
        assert!(
            !ui.get_nodes()
                .iter()
                .any(|n| n.label.contains("Screenshot")),
            "no caption plate on cards"
        );
        capture(window, &format!("023-discover-{}.png", recipe.id));
        // The same first image reaches the detail composition, with the credit as text.
        press(window, Key::Return);
        assert_eq!(state.borrow().screen, Screen::Detail);
        assert!(
            ui.get_nodes()
                .iter()
                .any(|n| n.kind == 4 && n.screenshot == asset)
        );
        assert!(
            ui.get_nodes().iter().any(|n| n.label == credit),
            "detail lists `{credit}`"
        );
        if matches!(
            recipe.id.as_str(),
            "ancient-aliens" | "auger-zenith" | "back-to-saturn-x-e1"
        ) {
            capture(window, &format!("023-detail-{}.png", recipe.id));
        }
        press(window, Key::Escape);
    }
    // Recipe order, rather than filesystem order, selects the image and credit.
    let mut reordered = catalog.recipe("ancient-aliens").unwrap().clone();
    reordered.screenshots.reverse();
    let asset = crate::screenshots::presentation(&reordered);
    assert_eq!(
        asset,
        format!("assets/screenshots/{}", reordered.screenshots[0].asset)
    );
    // Bright white sky is the second bundled Ancient Aliens shot. A render-only
    // reordered recipe stress-tests the identical caption plate on card/detail/band.
    {
        let mut r = state.borrow_mut();
        let saved = r.snapshot.clone();
        let index = r
            .snapshot
            .as_ref()
            .unwrap()
            .items
            .iter()
            .position(|i| i.recipe.id == reordered.id)
            .unwrap();
        r.snapshot.as_mut().unwrap().items[index].recipe = reordered.clone();
        r.selected = index;
        r.focus = 10 + index as i32;
        r.sync(ui);
        capture(window, "023-discover-white-sky.png");
        r.screen = Screen::Detail;
        r.focus = 1;
        r.sync(ui);
        capture(window, "023-detail-white-sky.png");
        r.snapshot = saved;
        r.screen = Screen::Discover;
        r.focus = 10;
        r.selected = 0;
        r.sync(ui);
    }
    reordered.screenshots[0].asset = "unbundled-new-release/01.jpg".into();
    assert!(crate::screenshots::presentation(&reordered).is_empty());

    // Home carries no caption plates, whatever the credit length.
    {
        let mut r = state.borrow_mut();
        let saved = r.snapshot.clone();
        let item = r
            .snapshot
            .as_mut()
            .unwrap()
            .items
            .iter_mut()
            .find(|i| i.recipe.id == "back-to-saturn-x-e1")
            .unwrap();
        item.last_played = Some(1);
        r.screen = Screen::Home;
        r.focus = 1;
        r.sync(ui);
        assert!(
            ui.get_nodes()
                .iter()
                .filter(|n| n.kind == 4 && n.w >= 160.)
                .all(|n| !n.screenshot.is_empty())
        );
        assert!(!ui.get_nodes().iter().any(|n| n.kind == 16));
        capture(window, "023-home-long-credit.png");
        r.snapshot = saved;
        r.screen = Screen::Discover;
        r.focus = 10;
        r.selected = 0;
        r.sync(ui);
    }
    // Absence of screenshot metadata must retain the painted cover,
    // even when the recipe id has bundled assets.
    {
        let mut r = state.borrow_mut();
        let saved = r.snapshot.clone();
        r.snapshot.as_mut().unwrap().items[0]
            .recipe
            .screenshots
            .clear();
        r.focus = 10;
        r.sync(ui);
        assert!(ui.get_band_screenshot().is_empty());
        assert!(
            ui.get_nodes()
                .iter()
                .find(|n| n.kind == 4)
                .unwrap()
                .screenshot
                .is_empty()
        );
        assert!(
            !ui.get_nodes()
                .iter()
                .any(|n| n.label.contains("concept art")),
            "the painted cover has no caption either"
        );
        capture(window, "023-discover-fallback.png");
        r.snapshot = saved;
        r.sync(ui);
    }

    // Up from the grid reaches the strip; Left/Right switch the category as focus moves.
    press(window, Key::UpArrow);
    assert_eq!(ui.get_focused(), 200);
    assert!(!ui.get_favorite_hint(), "X has no target on the strip");
    press(window, Key::LeftArrow);
    assert_eq!(
        ui.get_focused(),
        201 + used.len() as i32,
        "wrap to Favorites"
    );
    press(window, Key::RightArrow);
    assert_eq!(ui.get_focused(), 200, "wrap back to All");
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
    // Triggers (L2/R2) step and wrap the same list, from the grid too.
    ui.invoke_action(11);
    assert_eq!(selected_chip(ui), used[used.len() - 2].label);
    ui.invoke_action(12);
    assert_eq!(selected_chip(ui), used[used.len() - 1].label);
    press(window, Key::DownArrow);
    assert!(card_ids(ui).contains(&ui.get_focused()));
    ui.invoke_action(12);
    assert_eq!(selected_chip(ui), "Favorites");
    ui.invoke_action(12);
    assert_eq!(selected_chip(ui), "All", "R2 wraps at the end");
    ui.invoke_action(11);
    assert_eq!(selected_chip(ui), "Favorites", "L2 wraps at the beginning");
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

    // Right reaches another card; favorite a non-first recipe with multiple tags
    // so the detail assertions exercise all tags regardless of catalog ordering.
    press(window, Key::RightArrow);
    assert_ne!(ui.get_focused(), card_ids(ui)[0]);
    let first = card_ids(ui)[0];
    let favorite_id = {
        let mut r = state.borrow_mut();
        let index = r
            .snapshot
            .as_ref()
            .unwrap()
            .items
            .iter()
            .enumerate()
            .find(|(index, item)| 10 + *index as i32 != first && item.recipe.tags.len() >= 2)
            .map(|(index, _)| index)
            .expect("favorite/detail journey needs a non-first recipe with multiple tags");
        r.focus = 10 + index as i32;
        r.selected = index;
        r.sync(ui);
        r.focus
    };
    press_char(window, "x");
    idle(&state);
    let marked = state.borrow().snapshot.as_ref().unwrap().items[(favorite_id - 10) as usize]
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
        .find(|n| n.id == favorite_id)
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
    assert_eq!(card_ids(ui), [favorite_id]);
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
    press(window, Key::F2);
    assert_eq!(state.borrow().screen, Screen::Library);
    press(window, Key::F3);
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
    assert_detail_actions(ui);
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
    assert_detail_actions(ui);
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

/// Spec 025: real catalog data in every production title/author slot.
/// A screen group of the spec 025 title-fit audit. `screens` selects the
/// production screens whose title/author slots are measured, `components`
/// adds the Play setup component-label frames, `sheets` selects the profile
/// and/or uninstall sheet headings, and `recipe_group` keeps one half of the
/// fixtures so no single render pass runs long.
pub(super) fn title_fit_journey(
    window: &MinimalSoftwareWindow,
    ui: &AppWindow,
    screens: &[&str],
    components: bool,
    sheets: &[&str],
    recipe_group: Option<usize>,
) {
    use crate::{
        input::Action,
        services::recipe_change_tests::{installed, setup},
    };
    let (_temp, _server, env, _hits) = setup();
    drop(installed(&env));
    let state = crate::runtime::attach_environment(ui, env);
    idle(&state);
    let template = state
        .borrow()
        .snapshot
        .as_ref()
        .unwrap()
        .items
        .iter()
        .find(|i| i.recipe.id == "auger-zenith")
        .unwrap()
        .clone();
    let mut recipes: Vec<_> = Catalog::embedded()
        .recipes
        .into_iter()
        .map(|r| r.recipe)
        .collect();
    let mut synthetic = recipes[0].clone();
    synthetic.id = "synthetic-60".into();
    synthetic.title = "ABCDEFGHIJKLMNOPQRSTUVWXYZ ABCDEFGHIJKLMNOPQRSTUVWXYZ ABCDEF".into();
    assert_eq!(synthetic.title.chars().count(), 60);
    synthetic.components[0].label = synthetic.title.clone();
    recipes.push(synthetic);
    if let Ok(id) = std::env::var("DDL_TITLE_RECIPE") {
        recipes.retain(|r| r.id == id);
        assert_eq!(recipes.len(), 1, "known render fixture");
    }
    let audit = std::env::var_os("DDL_TITLE_AUDIT").is_some();
    let mut findings = Vec::new();
    let mut fits = Vec::new();
    let recipe_count = recipes.len();
    for (index, recipe) in recipes.into_iter().enumerate() {
        if recipe_group.is_some_and(|group| index * 2 / recipe_count.max(1) != group) {
            continue;
        }
        let mut item = template.clone();
        item.recipe = recipe.clone();
        item.last_played = Some(1);
        {
            let mut r = state.borrow_mut();
            r.snapshot.as_mut().unwrap().items = vec![item];
            r.selected = 0;
        }
        for (screen, name) in [
            (Screen::Home, "home"),
            (Screen::Discover, "discover"),
            (Screen::Library, "library"),
            (Screen::Detail, "detail"),
            (Screen::Setup, "setup"),
            (Screen::Progress, "progress"),
        ] {
            if !screens.contains(&name) {
                continue;
            }
            {
                let mut r = state.borrow_mut();
                r.screen = screen;
                if screen == Screen::Setup {
                    r.title_fit_advanced();
                }
                r.focus = if matches!(screen, Screen::Discover | Screen::Library) {
                    10
                } else {
                    1
                };
                r.pending =
                    (screen == Screen::Progress).then(|| Command::Install(recipe.id.clone()));
                r.sync(ui);
            }
            if !audit {
                let nodes: Vec<_> = ui.get_nodes().iter().collect();
                for title in nodes.iter().filter(|n| [20, 22].contains(&n.kind)) {
                    if let Some(next) = nodes
                        .iter()
                        .filter(|n| n.x == title.x && n.y > title.y && !n.label.is_empty())
                        .min_by(|a, b| a.y.total_cmp(&b.y))
                    {
                        assert!(
                            title.y + title.h <= next.y,
                            "{} {name}: title overlaps {}",
                            recipe.id,
                            next.label
                        );
                    }
                }
            }
            let mut seen = (0, 0);
            for n in ui
                .get_nodes()
                .iter()
                .filter(|n| [0, 20, 21, 22].contains(&n.kind) && !n.label.is_empty())
            {
                let title = n.label == recipe.title || recipe.title.starts_with(n.label.as_str());
                let author = recipe
                    .authors
                    .iter()
                    .any(|a| n.label.starts_with(a.as_str()));
                if !title && !author {
                    continue;
                }
                if author {
                    seen.1 += 1;
                } else {
                    seen.0 += 1;
                }
                ui.set_fit_label(n.label.clone());
                ui.set_fit_width(n.w);
                ui.set_fit_height(n.h);
                ui.set_fit_size(n.size);
                ui.set_fit_smaller(if author {
                    20.
                } else if n.size > 36. {
                    36.
                } else if n.size > 26. {
                    26.
                } else {
                    22.
                });
                ui.set_fit_display(!author);
                ui.set_fit_strong(false);
                ui.set_fit_multiline(if audit { true } else { !author && n.h >= 60. });
                ui.set_fit_full(n.kind == 22);
                if audit && ui.get_fit_base_height() > n.h + 0.5 {
                    findings.push(format!(
                        "{} {name} {}: {:.1}px > {}px",
                        recipe.id,
                        if author { "author" } else { "title" },
                        ui.get_fit_base_height(),
                        n.h
                    ));
                }
                if !audit {
                    assert!(
                        [20, 21, 22].contains(&n.kind),
                        "unfitted {} {name}: {}",
                        recipe.id,
                        n.label
                    );
                    if n.kind == 22 {
                        assert_eq!(n.label, recipe.title);
                        assert!(
                            ui.get_fit_measured_height() <= ui.get_fit_content_height() + 0.5,
                            "full detail title {}",
                            recipe.id
                        );
                    }
                    assert!(n.h <= if author { 36. } else { 96. }, "bounded slot");
                    assert!(ui.get_fit_chosen_size() >= if author { 20. } else { 22. });
                    assert_title_pixels_fit(
                        window,
                        ui,
                        &format!("{} {name}: {}", recipe.id, n.label),
                    );
                }
            }
            assert_eq!(
                seen,
                match screen {
                    Screen::Home => (2, 1),
                    Screen::Discover | Screen::Library | Screen::Detail => (1, 1),
                    Screen::Progress => (1, 0),
                    Screen::Setup => (0, 0),
                    _ => unreachable!(),
                },
                "{} {name}: every title/author slot is exercised",
                recipe.id
            );
            ui.set_fit_label(ui.get_heading());
            ui.set_fit_width(if ui.get_art_band() { 784. } else { 1200. });
            ui.set_fit_height(48.);
            ui.set_fit_size(36.);
            ui.set_fit_smaller(26.);
            ui.set_fit_display(true);
            ui.set_fit_strong(false);
            ui.set_fit_multiline(false);
            ui.set_fit_full(false);
            if !audit {
                assert_title_pixels_fit(window, ui, &format!("{} {name} heading", recipe.id));
            }
            if ui.get_fit_base_width() > ui.get_fit_width() {
                fits.push(format!(
                    "{} {name} heading: base {:.1}px, slot {}px, chosen {}px",
                    recipe.id,
                    ui.get_fit_base_width(),
                    ui.get_fit_width(),
                    ui.get_fit_chosen_size()
                ));
            }
            if audit && ui.get_fit_base_width() > ui.get_fit_width() {
                findings.push(format!(
                    "{} {name} heading: {:.1}px > {}px",
                    recipe.id,
                    ui.get_fit_base_width(),
                    ui.get_fit_width()
                ));
            }
            if components && screen == Screen::Setup {
                let rows: Vec<_> = ui
                    .get_nodes()
                    .iter()
                    .filter(|n| n.fitted_label || (n.kind == 8 && (100..200).contains(&n.id)))
                    .collect();
                assert_eq!(
                    rows.len(),
                    recipe.components.len(),
                    "every component label is audited"
                );
                for n in rows {
                    ui.set_fit_label(n.label.clone());
                    ui.set_fit_width(n.w - if n.kind == 8 { 16. } else { 32. });
                    ui.set_fit_height(if n.kind == 8 { n.h - 16. } else { 30. });
                    ui.set_fit_size(n.size);
                    ui.set_fit_smaller(20.);
                    ui.set_fit_display(false);
                    ui.set_fit_strong(n.kind == 3);
                    ui.set_fit_multiline(audit);
                    ui.set_fit_full(false);
                    if audit && ui.get_fit_base_height() > ui.get_fit_height() {
                        findings.push(format!(
                            "{} component {}: {:.1}px > {}px",
                            recipe.id,
                            n.label,
                            ui.get_fit_base_height(),
                            ui.get_fit_height()
                        ));
                    }
                    if !audit {
                        assert_title_pixels_fit(
                            window,
                            ui,
                            &format!("{} component {}", recipe.id, n.label),
                        );
                    }
                    {
                        let mut r = state.borrow_mut();
                        r.focus = n.id;
                        r.sync(ui);
                    }
                    capture_component(
                        window,
                        ui,
                        &n,
                        &format!("025-component-{}-{}.png", recipe.id, n.id),
                    );
                }
            }
            capture(window, &format!("025-{name}-{}.png", recipe.id));
        }
        for name in ["profiles", "uninstall"] {
            if !sheets.contains(&name) {
                continue;
            }
            let mut r = state.borrow_mut();
            r.screen = Screen::Detail;
            r.pending = None;
            r.sync(ui);
            r.title_fit_sheet(ui, name == "uninstall");
            assert!(ui.get_sheet_open());
            ui.set_fit_label(ui.get_sheet_title());
            ui.set_fit_width(656.);
            ui.set_fit_height(if ui.get_sheet_height() > 400. {
                64.
            } else if ui.get_sheet_height() > 300. {
                164.
            } else {
                96.
            });
            ui.set_fit_size(26.);
            ui.set_fit_smaller(22.);
            ui.set_fit_multiline(true);
            ui.set_fit_display(true);
            ui.set_fit_strong(false);
            if audit && ui.get_fit_base_height() > ui.get_fit_height() {
                findings.push(format!(
                    "{} {name} sheet: {:.1}px > {}px",
                    recipe.id,
                    ui.get_fit_base_height(),
                    ui.get_fit_height()
                ));
            }
            ui.set_fit_full(false);
            if !audit {
                assert_title_pixels_fit(window, ui, &format!("{} {name} sheet", recipe.id));
            }
            capture(window, &format!("025-{name}-{}.png", recipe.id));
            r.action(ui, Action::Back);
            r.sync(ui);
        }
    }
    {
        let dir = headless_dir();
        std::fs::write(
            dir.join(if audit {
                "025-audit.txt"
            } else {
                "025-fit.txt"
            }),
            if audit { findings } else { fits }.join("\n"),
        )
        .unwrap();
    }
}

/// Render the exact production fitter in isolation and reject painted pixels outside
/// its assigned rectangle. This catches vertical clipping/overflow regressions that
/// checking fixed ViewNode rectangles alone would miss.
fn assert_title_pixels_fit(window: &MinimalSoftwareWindow, ui: &AppWindow, context: &str) {
    ui.set_fit_test_visible(true);
    let frame = render(window);
    ui.set_fit_test_visible(false);
    let (w, h) = (ui.get_fit_width(), ui.get_fit_height());
    assert!(ui.get_fit_content_height() <= h);
    assert!(
        ui.get_fit_content_height() + 0.5 >= ui.get_fit_line_height(),
        "{context}: even one complete line must fit"
    );
    assert!(
        ui.get_fit_content_height()
            <= ui.get_fit_line_height() * if ui.get_fit_multiline() { 2. } else { 1. } + 0.5
    );
    let mut ink = 0;
    for (index, p) in frame.as_slice().iter().enumerate() {
        if (p.r, p.g, p.b) == (0x12, 0x0c, 0x0d) {
            continue;
        }
        ink += 1;
        let (x, y) = (
            (index % WIDTH as usize) as f32,
            (index / WIDTH as usize) as f32,
        );
        assert!(
            x >= 40. && x < 40. + w && y >= 240. && y < 240. + h,
            "{context}: glyph escaped {w}×{h} slot at {x},{y}"
        );
    }
    assert!(ink > 0, "{context}: text must actually render");
}

/// Check the real row/button too: a correct isolated fitter must not hide a
/// component label through a production visibility or implicit-height binding.
fn capture_component(
    window: &MinimalSoftwareWindow,
    ui: &AppWindow,
    node: &crate::ViewNode,
    name: &str,
) {
    let frame = render(window);
    let button = node.kind == 8;
    let inset = if button { 8. } else { 20. };
    let x = (28. + node.x + inset) as u32;
    let y = (218. + node.y - ui.get_scroll_offset() + 8.) as u32;
    let w = (node.w - inset * 2.) as u32;
    let h = if button { node.h as u32 - 16 } else { 30 };
    assert!(
        x + w <= WIDTH && y >= 210 && y + h <= HEIGHT - 64,
        "{name}: component must be in the viewport"
    );
    let color = if node.primary {
        (0x12, 0x0c, 0x0d)
    } else {
        (0xf8, 0xea, 0xd5)
    };
    let ink = (y..y + h)
        .flat_map(|y| (x..x + w).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let p = frame.as_slice()[(*y * WIDTH + *x) as usize];
            (p.r, p.g, p.b) == color
        })
        .count();
    assert!(
        ink > 0,
        "{name}: production component label must be visible"
    );
    write_png(name, &frame);
}

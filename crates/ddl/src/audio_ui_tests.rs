//! Spec 067: Settings → Audio, driven by real key events and rendered off-screen.
use super::*;
use crate::{
    runtime::{Runtime, Screen},
    services::{Command, Environment, Problem, TestEnvironment},
};
use ddl_core::{
    catalog::Catalog,
    downloads::{InstallProgress, MemberRequest, Progress, Sha256},
    engines::dsda,
    soundfonts::{self, Choice, Soundfonts},
};
use std::{
    cell::RefCell,
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
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
        assert!(Instant::now() < until, "audio worker stalled");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn fixture(root: &Path) -> (Vec<MemberRequest>, PathBuf, Vec<u8>) {
    let font = b"RIFF\x04\0\0\0sfbk";
    let notice = b"fixture read me";
    let html = b"<script>inert fixture notice</script>";
    let members: Vec<_> = [
        (soundfonts::FONT_FILE, font.as_slice()),
        ("Read Me.txt", notice.as_slice()),
        ("Documentation/English.html", html.as_slice()),
    ]
    .iter()
    .map(|(path, bytes)| {
        MemberRequest::new(*path)
            .with_size(bytes.len() as u64)
            .with_sha256(Sha256::of(bytes))
    })
    .collect();
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (path, bytes) in [
        (soundfonts::FONT_FILE, font.as_slice()),
        ("Read Me.txt", notice.as_slice()),
        ("Documentation/English.html", html.as_slice()),
    ] {
        zip.start_file(path, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(bytes).unwrap();
    }
    let bytes = zip.finish().unwrap().into_inner();
    let archive = root.join("fixture.zip");
    fs::write(&archive, &bytes).unwrap();
    (members, archive, bytes)
}

fn publish(env: &Environment, members: &[MemberRequest]) {
    let (_, archive, bytes) = fixture(&env.home);
    Soundfonts::with_members(&env.paths, members.to_vec())
        .install_fixture(
            &archive,
            bytes.len() as u64,
            Sha256::of(&bytes),
            &ddl_core::downloads::CancellationToken::new(),
            &mut |_| {},
        )
        .expect("publish the fixture");
}

/// An environment with a fake active engine so Startup reaches Home instead of
/// First Run, plus a tiny reviewed soundfont fixture.
fn environment(root: &Path, members: Vec<MemberRequest>) -> Environment {
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    let paths = ddl_core::platform::paths::resolve(&|key: &str| {
        (key == "HOME").then(|| home.display().to_string())
    })
    .unwrap();
    let executable = home.join("fake-engine");
    fs::write(&executable, b"#!/bin/sh\necho 'dsda-doom v0.30.1'\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let engines = paths.data.join("engines/dsda-doom");
    fs::create_dir_all(&engines).unwrap();
    fs::write(
        engines.join("installed.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "installed": [{
                "version": "0.30.1",
                "path": executable,
                "mode": "manual",
                "sha256": "0".repeat(64),
                "tag": "v0.30.1",
                "reviewed": true,
                "origin": "manual",
                "installed_at": 1_700_000_000u64,
            }],
            "active": "0.30.1",
        }))
        .unwrap(),
    )
    .unwrap();
    Environment {
        paths,
        home,
        screen: None,
        test: Some(Arc::new(TestEnvironment {
            api: Arc::new(dsda::CannedApi::new()),
            catalog: Catalog::embedded(),
            engine_base: String::new(),
            catalog_url: String::new(),
            reviewed: dsda::ReviewedList::embedded(),
            runner: Arc::new(dsda::FakeEngineRunner::new(dsda::LaunchExit::Clean)),
            engine_response_timeout: None,
            soundfont_members: Some(members),
        })),
    }
}

struct Scene {
    window: Rc<slint::platform::software_renderer::MinimalSoftwareWindow>,
    ui: AppWindow,
    state: Rc<RefCell<Runtime>>,
    env: Environment,
    _temp: tempfile::TempDir,
}

fn scene(installed_font: bool, selected: Option<Choice>) -> Scene {
    let (window, ui) = setup();
    let temp = tempfile::tempdir().unwrap();
    let (members, _, _) = fixture(temp.path());
    let env = environment(temp.path(), members.clone());
    if installed_font {
        publish(&env, &members);
    }
    if let Some(choice) = selected {
        let fonts = Soundfonts::with_members(&env.paths, members);
        if choice == Choice::Arachno {
            fonts.select(Choice::Arachno).unwrap();
        } else {
            fonts.select(Choice::Default).unwrap();
        }
    }
    let state = crate::runtime::attach_environment(&ui, env.clone());
    settle(&state);
    Scene {
        window,
        ui,
        state,
        env,
        _temp: temp,
    }
}

/// F4 opens Settings; Down enters the section column; Down again reaches Audio.
fn open_audio(
    window: &slint::platform::software_renderer::MinimalSoftwareWindow,
    state: &Rc<RefCell<Runtime>>,
) {
    press(window, Key::F4);
    assert_eq!(state.borrow().screen, Screen::Settings);
    assert_eq!(state.borrow().focus, 1, "Engine is focused first");
    press(window, Key::DownArrow);
    assert_eq!(state.borrow().focus, 2, "Audio is the second section");
    press(window, Key::Return);
    assert_eq!(state.borrow().focus, 70, "Default is focused");
}

fn labels(ui: &AppWindow) -> String {
    ui.get_nodes()
        .iter()
        .map(|n| n.label.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn audio_section_renders_and_selects_default() {
    if isolated(module_path!(), "audio_section_renders_and_selects_default") {
        return;
    }
    let Scene {
        window,
        ui,
        state,
        env,
        _temp,
    } = scene(false, None);
    open_audio(&window, &state);
    let text = labels(&ui);
    for expected in [
        "MIDI soundfont",
        "Default",
        "Arachno SoundFont 1.0",
        "Not installed",
        "Download Arachno",
        "Original terms and documentation",
    ] {
        assert!(
            text.contains(expected),
            "{expected:?} missing from:\n{text}"
        );
    }
    assert!(
        !ui.get_nodes().iter().any(|n| n.id == 71 && n.actionable),
        "Arachno cannot be selected before its reviewed install"
    );
    write_png("067-audio-settings.png", &render(&window));

    press(&window, Key::Return);
    settle(&state);
    assert_eq!(
        Soundfonts::new(&env.paths).preferences().unwrap().choice,
        Some(Choice::Default)
    );
}

#[test]
fn installed_arachno_is_selectable_and_persists() {
    if isolated(
        module_path!(),
        "installed_arachno_is_selectable_and_persists",
    ) {
        return;
    }
    let Scene {
        window,
        ui,
        state,
        env,
        _temp,
    } = scene(true, None);
    open_audio(&window, &state);
    assert!(labels(&ui).contains("Installed"));
    assert!(
        ui.get_nodes().iter().any(|n| n.id == 71 && n.actionable),
        "Arachno becomes selectable after the reviewed install"
    );
    assert!(
        !ui.get_nodes().iter().any(|n| n.id == 72),
        "no download action once installed"
    );

    press(&window, Key::DownArrow);
    assert_eq!(state.borrow().focus, 71, "Arachno row is reachable");
    press(&window, Key::Return);
    settle(&state);
    assert_eq!(
        Soundfonts::new(&env.paths).preferences().unwrap().choice,
        Some(Choice::Arachno)
    );
    assert!(labels(&ui).contains("Arachno SoundFont 1.0"));
    write_png("067-audio-installed.png", &render(&window));
}

#[test]
fn audio_download_and_error_states_render() {
    if isolated(module_path!(), "audio_download_and_error_states_render") {
        return;
    }
    let Scene {
        window, ui, state, ..
    } = scene(false, None);
    open_audio(&window, &state);
    {
        let mut state = state.borrow_mut();
        state.screen = Screen::Progress;
        state.busy = true;
        state.pending = Some(Command::InstallSoundfont);
        state.progress = Some(InstallProgress::transfer(Progress::new(
            48 * 1024 * 1024,
            Some(ddl_core::soundfonts::DOWNLOAD_BYTES),
        )));
    }
    state.borrow_mut().sync(&ui);
    let text = labels(&ui);
    assert!(text.contains("Downloading"), "{text}");
    assert!(text.contains("Cancel"), "{text}");
    write_png("067-audio-downloading.png", &render(&window));

    {
        let mut state = state.borrow_mut();
        state.screen = Screen::Settings;
        state.busy = false;
        state.pending = None;
        state.progress = None;
        state.problem = Some(Problem::Network);
        state.focus = 800;
    }
    state.borrow_mut().sync(&ui);
    assert!(
        labels(&ui).contains("Download interrupted"),
        "{}",
        labels(&ui)
    );
    write_png("067-audio-error.png", &render(&window));
}

#[test]
fn a_missing_selected_font_stays_visible_and_recoverable() {
    if isolated(
        module_path!(),
        "a_missing_selected_font_stays_visible_and_recoverable",
    ) {
        return;
    }
    let Scene {
        window,
        ui,
        state,
        env,
        _temp,
    } = scene(false, None);
    // Simulate a selection whose install disappeared after it was made.
    fs::create_dir_all(&env.paths.config).unwrap();
    fs::write(
        env.paths.config.join("audio.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "choice": "arachno",
        }))
        .unwrap(),
    )
    .unwrap();
    state.borrow_mut().start(&ui, Command::Startup);
    settle(&state);
    open_audio(&window, &state);
    let text = labels(&ui);
    assert!(text.contains("Using Default"), "{text}");
    assert!(text.contains("Reinstall"), "{text}");
    write_png("067-audio-missing.png", &render(&window));
}

//! Spec 067 offline service checks: verified fixture installs, persistence and
//! guarded application of the Audio choice on the launch path.
use super::*;
use ddl_core::downloads::Sha256;
use std::io::Cursor;
use std::sync::Arc;

/// A tiny archive with the same reviewed member names as the production font.
fn fixture(root: &Path) -> (Vec<ddl_core::downloads::MemberRequest>, PathBuf, Vec<u8>) {
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
        ddl_core::downloads::MemberRequest::new(*path)
            .with_size(bytes.len() as u64)
            .with_sha256(Sha256::of(bytes))
    })
    .collect();
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (path, bytes) in members.iter().map(|member| {
        (
            member.path.clone(),
            match member.path.as_str() {
                soundfonts::FONT_FILE => font.as_slice(),
                "Read Me.txt" => notice.as_slice(),
                _ => html.as_slice(),
            },
        )
    }) {
        zip.start_file(path, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(bytes).unwrap();
    }
    let bytes = zip.finish().unwrap().into_inner();
    let archive = root.join("fixture.zip");
    fs::write(&archive, &bytes).unwrap();
    (members, archive, bytes)
}

fn environment(root: &Path, members: Vec<ddl_core::downloads::MemberRequest>) -> Environment {
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    let paths = ddl_core::platform::paths::resolve(&|key: &str| {
        (key == "HOME").then(|| home.display().to_string())
    })
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

fn publish(env: &Environment, members: &[ddl_core::downloads::MemberRequest]) {
    let (_, archive, bytes) = fixture(&env.home);
    let fonts = Soundfonts::with_members(&env.paths, members.to_vec());
    fonts
        .install_fixture(
            &archive,
            bytes.len() as u64,
            Sha256::of(&bytes),
            &CancellationToken::new(),
            &mut |_| {},
        )
        .expect("publish the fixture through the production safety path");
}

fn audio_preferences(env: &Environment) -> soundfonts::Preferences {
    Soundfonts::new(&env.paths).preferences().unwrap()
}

fn preferences_path(env: &Environment) -> PathBuf {
    env.paths.config.join("audio.json")
}

fn install_root(env: &Environment) -> PathBuf {
    env.paths.data.join("soundfonts")
}

#[test]
fn a_published_font_is_installed_but_never_selected_automatically() {
    let temp = tempfile::tempdir().unwrap();
    let (members, _, _) = fixture(temp.path());
    let env = environment(temp.path(), members.clone());
    publish(&env, &members);

    let mut services = Services::open(env.clone()).unwrap();
    let audio = services.snapshot().unwrap().audio;
    assert!(audio.installed);
    assert_eq!(audio.choice, None, "downloading never selects");
    assert!(audio.warning.is_none());

    services
        .execute(
            Command::SelectSoundfont(Choice::Arachno),
            &CancellationToken::new(),
            &mut |_| {},
        )
        .unwrap();
    let audio = services.snapshot().unwrap().audio;
    assert_eq!(audio.choice, Some(Choice::Arachno));
    assert!(audio.warning.is_none());

    // Restart: a fresh Services instance restores the explicit selection.
    let reopened = Services::open(env).unwrap();
    assert_eq!(
        reopened.snapshot().unwrap().audio.choice,
        Some(Choice::Arachno)
    );
}

#[test]
fn an_explicit_default_replaces_a_custom_engine_setting_between_runs() {
    let temp = tempfile::tempdir().unwrap();
    let (members, _, _) = fixture(temp.path());
    let env = environment(temp.path(), members.clone());
    let fonts = Soundfonts::with_members(&env.paths, members);
    fonts.select(Choice::Default).unwrap();

    let services = Services::open(env.clone()).unwrap();
    services.config.provision().unwrap();
    let custom = "# player comment\nuse_mouse 1\nsnd_midiplayer \"sdl\"\nsnd_soundfont \"/opt/custom.sf2\"\nunknown 7\n";
    fs::write(services.config.active_path(), custom).unwrap();

    services.begin_engine_run().unwrap().end().unwrap();

    let active = fs::read_to_string(services.config.active_path()).unwrap();
    for expected in [
        "# player comment\n",
        "use_mouse 1\n",
        "snd_midiplayer \"fluidsynth\"\n",
        "snd_soundfont \"\"\n",
        "unknown 7\n",
    ] {
        assert!(active.contains(expected), "{expected:?} in:\n{active}");
    }
    assert_eq!(
        fs::read_dir(services.config.backups_dir()).unwrap().count(),
        1,
        "the replaced custom setting is backed up first"
    );
}

#[test]
fn no_explicit_choice_preserves_a_custom_engine_setting() {
    let temp = tempfile::tempdir().unwrap();
    let (members, _, _) = fixture(temp.path());
    let env = environment(temp.path(), members);
    let services = Services::open(env).unwrap();
    services.config.provision().unwrap();
    let custom = "snd_midiplayer \"sdl\"\nsnd_soundfont \"/opt/my font.sf2\"\nuse_mouse 1\n";
    fs::write(services.config.active_path(), custom).unwrap();

    services.begin_engine_run().unwrap().end().unwrap();

    assert_eq!(
        fs::read_to_string(services.config.active_path()).unwrap(),
        custom,
        "DDL writes nothing before the player makes a choice"
    );
    assert_eq!(
        services.snapshot().unwrap().audio.current,
        soundfonts::engine::Current::Custom
    );
}

#[test]
fn an_explicit_arachno_choice_writes_the_verified_absolute_path() {
    let temp = tempfile::tempdir().unwrap();
    let (members, _, _) = fixture(temp.path());
    let env = environment(temp.path(), members.clone());
    publish(&env, &members);
    let fonts = Soundfonts::with_members(&env.paths, members);
    fonts.select(Choice::Arachno).unwrap();
    let font = fonts.verified_font().unwrap();
    let font = font.to_str().unwrap().to_owned();

    let services = Services::open(env).unwrap();
    services.config.provision().unwrap();
    services.begin_engine_run().unwrap().end().unwrap();

    let active = fs::read_to_string(services.config.active_path()).unwrap();
    assert!(
        active.contains("snd_midiplayer \"fluidsynth\"\n"),
        "{active}"
    );
    assert!(
        active.contains(&format!("snd_soundfont \"{font}\"\n")),
        "{active}"
    );
}

#[test]
fn a_selected_missing_font_falls_back_to_default_without_losing_the_choice() {
    let temp = tempfile::tempdir().unwrap();
    let (members, _, _) = fixture(temp.path());
    let env = environment(temp.path(), members.clone());
    let fonts = Soundfonts::with_members(&env.paths, members);
    assert!(
        fonts.select(Choice::Arachno).is_err(),
        "a missing font is never selectable"
    );
    fs::create_dir_all(&env.paths.config).unwrap();
    fs::write(
        preferences_path(&env),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "choice": "arachno",
        }))
        .unwrap(),
    )
    .unwrap();

    let services = Services::open(env).unwrap();
    services.config.provision().unwrap();
    fs::write(
        services.config.active_path(),
        "snd_midiplayer \"sdl\"\nsnd_soundfont \"/opt/custom.sf2\"\n",
    )
    .unwrap();
    services.begin_engine_run().unwrap().end().unwrap();

    let active = fs::read_to_string(services.config.active_path()).unwrap();
    assert!(active.contains("snd_soundfont \"\"\n"), "{active}");
    assert!(
        active.contains("snd_midiplayer \"fluidsynth\"\n"),
        "{active}"
    );
    let audio = services.snapshot().unwrap().audio;
    assert_eq!(
        audio.choice,
        Some(Choice::Arachno),
        "the selection stays for a reinstall"
    );
    assert!(!audio.installed);
    assert_eq!(
        audio.warning.as_deref(),
        Some(soundfonts::MISSING_FONT_WARNING)
    );
    assert_eq!(
        audio_preferences(&services.env.clone()).choice,
        Some(Choice::Arachno)
    );
}

#[test]
fn a_running_engine_is_never_changed() {
    let temp = tempfile::tempdir().unwrap();
    let (members, _, _) = fixture(temp.path());
    let env = environment(temp.path(), members.clone());
    Soundfonts::with_members(&env.paths, members)
        .select(Choice::Default)
        .unwrap();
    let services = Services::open(env).unwrap();
    services.config.provision().unwrap();
    let custom = "snd_soundfont \"/opt/custom.sf2\"\n";
    fs::write(services.config.active_path(), custom).unwrap();

    let guard = services.config.run_guard().unwrap();
    assert!(services.begin_engine_run().is_err());
    assert_eq!(
        fs::read_to_string(services.config.active_path()).unwrap(),
        custom
    );
    guard.end().unwrap();
}

/// Spec 067 review: a real config/path failure must abort the shared launch
/// path with an actionable problem, never silently launch with another font.
#[test]
fn an_unwritable_config_aborts_the_launch_with_an_actionable_problem() {
    let temp = tempfile::tempdir().unwrap();
    let (members, _, _) = fixture(temp.path());
    let env = environment(temp.path(), members.clone());
    Soundfonts::with_members(&env.paths, members)
        .select(Choice::Default)
        .unwrap();
    let services = Services::open(env).unwrap();
    services.config.provision().unwrap();
    let active = services.config.active_path();
    fs::remove_file(&active).unwrap();
    fs::create_dir(&active).unwrap();

    match services.begin_engine_run() {
        Ok(guard) => {
            let _ = guard.end();
            panic!("a config write failure must not start the engine");
        }
        Err(error) => {
            let problem = error.into_problem();
            assert_eq!(problem, Problem::Audio);
            assert!(
                problem.to_string().contains("Settings → Audio"),
                "the error tells the player where to act: {problem}"
            );
        }
    }
    // The failed attempt did not leave the run guard held.
    services.config.run_guard().unwrap().end().unwrap();
}

#[test]
fn cancelling_a_fixture_install_keeps_the_previous_install_and_choice() {
    let temp = tempfile::tempdir().unwrap();
    let (members, _, _) = fixture(temp.path());
    let env = environment(temp.path(), members.clone());
    publish(&env, &members);
    let fonts = Soundfonts::with_members(&env.paths, members.clone());
    fonts.select(Choice::Default).unwrap();
    let pointer = fs::read(install_root(&env).join("installed.json")).unwrap();

    let (_, archive, bytes) = fixture(&env.home);
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(
        fonts
            .install_fixture(
                &archive,
                bytes.len() as u64,
                Sha256::of(&bytes),
                &cancel,
                &mut |_| {},
            )
            .is_err()
    );
    assert_eq!(
        fs::read(install_root(&env).join("installed.json")).unwrap(),
        pointer
    );
    assert_eq!(fonts.preferences().unwrap().choice, Some(Choice::Default));
}

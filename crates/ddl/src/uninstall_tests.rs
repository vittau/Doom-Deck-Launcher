//! Spec 022 service and CLI checks with loopback HTTP and synthetic bytes only.
use super::*;
use crate::services::recipe_change_tests::{installed, setup};

/// The snapshot item for `id`, whether or not it is still installed.
fn mapset(s: &Services, id: &str) -> Item {
    s.snapshot()
        .unwrap()
        .items
        .into_iter()
        .find(|i| i.recipe.id == id)
        .unwrap()
}

fn profiles_of(s: &Services, id: &str) -> Vec<Profile> {
    s.profiles
        .list()
        .unwrap()
        .into_iter()
        .filter(|p| p.recipe_id == id)
        .collect()
}

fn run(s: &mut Services, command: Command) {
    s.execute(command, &CancellationToken::new(), &mut |_| {})
        .unwrap();
}

#[test]
fn uninstall_keeps_profiles_by_default_and_a_reinstall_finds_them() {
    let (_temp, _server, env, _hits) = setup();
    let mut s = installed(&env);
    run(&mut s, Command::NewProfile("auger-zenith".into(), false));
    let before = profiles_of(&s, "auger-zenith");
    assert_eq!(before.len(), 2);
    for profile in &before {
        fs::write(s.profiles.dirs(&profile.id).save.join("save.dsg"), b"save").unwrap();
        fs::write(s.profiles.dirs(&profile.id).shots.join("shot.png"), b"png").unwrap();
    }
    let selected = s.state.last_profiles.get("auger-zenith").cloned();
    assert!(selected.is_some());
    run(&mut s, Command::Favorite("auger-zenith".into()));

    let outcome = s.uninstall_mapset("auger-zenith", false).unwrap();
    assert_eq!(outcome.id, "auger-zenith");
    assert!(outcome.bytes > 0, "the dialog needs a real installed size");
    assert_eq!(outcome.profiles, 0);
    assert!(!outcome.imported);
    assert!(mapset(&s, "auger-zenith").manifest.is_none());
    // Profiles, saves, remembered selection and favorite all survive.
    assert_eq!(profiles_of(&s, "auger-zenith").len(), 2);
    for profile in &before {
        assert_eq!(
            fs::read(s.profiles.dirs(&profile.id).save.join("save.dsg")).unwrap(),
            b"save"
        );
        assert_eq!(
            fs::read(s.profiles.dirs(&profile.id).shots.join("shot.png")).unwrap(),
            b"png"
        );
    }
    assert_eq!(s.state.last_profiles.get("auger-zenith").cloned(), selected);
    assert!(s.state.favorites.contains("auger-zenith"));
    assert!(!s.installer.library_dir("auger-zenith").unwrap().exists());
    assert!(s.mapset_status(Some("auger-zenith")).is_err());

    // Reinstalling finds and reuses the kept profiles (spec 022 A1).
    let manifest = s.install_mapset("auger-zenith", &[], &mut |_| {}).unwrap();
    assert_eq!(manifest.recipe_id, "auger-zenith");
    assert_eq!(profiles_of(&s, "auger-zenith").len(), 2);
    assert!(mapset(&s, "auger-zenith").manifest.is_some());
}

#[test]
fn uninstall_can_delete_profiles_and_saves_but_keeps_favorites() {
    let (_temp, _server, env, _hits) = setup();
    let mut s = installed(&env);
    for dir in s.profiles.dirs("auger-zenith-default").all() {
        fs::write(dir.join("player-data"), b"keep me only when asked").unwrap();
    }
    run(&mut s, Command::Favorite("auger-zenith".into()));

    let outcome = s.uninstall_mapset("auger-zenith", true).unwrap();
    assert_eq!(outcome.profiles, 1);
    assert!(profiles_of(&s, "auger-zenith").is_empty());
    assert!(!s.profiles.dirs("auger-zenith-default").root.exists());
    assert!(!s.state.last_profiles.contains_key("auger-zenith"));
    // Favorites stay: they are a catalog preference, not an install fact.
    assert!(s.state.favorites.contains("auger-zenith"));
}

#[test]
fn an_interrupted_uninstall_stays_inactive_and_is_finished_on_the_next_start() {
    let (_temp, _server, env, _hits) = setup();
    let s = installed(&env);
    let recipe = s.recipe("auger-zenith").unwrap();
    let manifest = s.manifest(&recipe).unwrap().unwrap();
    // Crash between the atomic mark and any file deletion.
    s.installer.begin_uninstall(&recipe, &manifest).unwrap();
    let library = s.installer.library_dir("auger-zenith").unwrap();
    assert!(library.exists(), "the mark deletes nothing");
    drop(s);

    let mut reopened = Services::open(env.clone()).unwrap();
    // Inactive immediately: it can neither launch nor be reported installed.
    assert!(mapset(&reopened, "auger-zenith").manifest.is_none());
    assert!(matches!(
        reopened.resolve_mapset("auger-zenith", &LaunchOverrides::default()),
        Err(CliFailure::Blocker(Blocker::NotInstalled))
    ));
    run(&mut reopened, Command::Startup);
    assert!(!library.exists(), "startup finishes the interrupted delete");
    assert!(!reopened.state.recipes.contains_key("auger-zenith"));
}

#[test]
fn uninstall_refuses_mapsets_that_are_not_installed() {
    let (_temp, _server, env, _hits) = setup();
    let mut s = Services::open(env).unwrap();
    assert!(
        s.uninstall_mapset("auger-zenith", false)
            .unwrap_err()
            .to_string()
            .contains("not installed")
    );
    assert!(
        s.uninstall_mapset("no-such-mapset", false)
            .unwrap_err()
            .to_string()
            .contains("no catalog recipe")
    );
}

#[test]
fn cli_uninstall_uses_the_shared_transaction_and_reports_profiles() {
    let (_temp, _server, env, _hits) = setup();
    installed(&env);
    let mut out = Vec::new();
    let mut err = Vec::new();
    crate::cli_exec::run_io(
        crate::cli::Command::MapsetUninstall {
            id: "auger-zenith".into(),
            delete_profiles: false,
        },
        env.clone(),
        &mut out,
        &mut err,
    )
    .unwrap();
    let printed = String::from_utf8(out).unwrap();
    assert!(printed.contains("Uninstalled auger-zenith"), "{printed}");
    assert!(
        Services::open(env.clone())
            .unwrap()
            .snapshot()
            .unwrap()
            .items
            .iter()
            .find(|i| i.recipe.id == "auger-zenith")
            .unwrap()
            .manifest
            .is_none()
    );

    let mut out = Vec::new();
    let error = crate::cli_exec::run_io(
        crate::cli::Command::MapsetUninstall {
            id: "auger-zenith".into(),
            delete_profiles: false,
        },
        env.clone(),
        &mut out,
        &mut err,
    )
    .unwrap_err();
    assert!(error.to_string().contains("not installed"), "{error}");

    // `--delete-profiles` removes the profile and says so.
    let mut s = Services::open(env.clone()).unwrap();
    s.install_mapset("auger-zenith", &[], &mut |_| {}).unwrap();
    let mut out = Vec::new();
    crate::cli_exec::run_io(
        crate::cli::Command::MapsetUninstall {
            id: "auger-zenith".into(),
            delete_profiles: true,
        },
        env.clone(),
        &mut out,
        &mut err,
    )
    .unwrap();
    let printed = String::from_utf8(out).unwrap();
    assert!(printed.contains("deleted 1 profile(s)"), "{printed}");
}

#[test]
fn cli_uninstall_refuses_while_the_state_lock_is_held() {
    let (_temp, _server, env, _hits) = setup();
    installed(&env);
    let lock = ddl_core::platform::lock::StateLock::acquire(&env.paths.config).unwrap();
    let mut discard_out = Vec::new();
    let mut discard_err = Vec::new();
    let error = crate::cli_exec::run_io(
        crate::cli::Command::MapsetUninstall {
            id: "auger-zenith".into(),
            delete_profiles: false,
        },
        env.clone(),
        &mut discard_out,
        &mut discard_err,
    )
    .unwrap_err();
    assert!(error.to_string().contains("another DDL process"), "{error}");
    drop(lock);
    // Releasing the lock lets the same transaction run.
    let mut out = Vec::new();
    let mut err = Vec::new();
    crate::cli_exec::run_io(
        crate::cli::Command::MapsetUninstall {
            id: "auger-zenith".into(),
            delete_profiles: false,
        },
        env,
        &mut out,
        &mut err,
    )
    .unwrap();
    assert!(String::from_utf8(out).unwrap().contains("Uninstalled"));
}

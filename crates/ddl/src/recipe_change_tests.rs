//! Spec 020 service and CLI checks with synthetic bytes and loopback HTTP only.
use super::*;
use ddl_core::{
    catalog::{Complevel, PackageKind},
    downloads::{
        Sha256,
        test_http::{Response, Server},
    },
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub(crate) fn setup() -> (tempfile::TempDir, Server, Environment, Arc<AtomicUsize>) {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let paths = ddl_core::platform::paths::resolve(&|key: &str| {
        (key == "HOME").then(|| home.display().to_string())
    })
    .unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let hits = count.clone();
    let bytes = b"synthetic mapset content";
    let server = Server::start(move |r| {
        hits.fetch_add(1, Ordering::SeqCst);
        if r.target == "/bad" {
            Response::ok(b"bad content".to_vec())
        } else {
            Response::ok(bytes.to_vec())
        }
    });
    let mut catalog = Catalog::embedded();
    catalog.hosts.downloads = vec!["127.0.0.1".into()];
    catalog.recipes.retain(|r| r.recipe.id == "auger-zenith");
    let recipe = &mut catalog.recipes[0].recipe;
    recipe.packages.truncate(1);
    recipe.packages[0].members.truncate(1);
    let package = &mut recipe.packages[0];
    package.kind = PackageKind::Wad;
    package.urls = vec![server.url("/good")];
    package.size = bytes.len() as u64;
    package.sha256 = Sha256::of(bytes).to_hex();
    package.members[0].size = bytes.len() as u64;
    package.members[0].sha256 = package.sha256.clone();
    package.members[0].path = "main.wad".into();
    let main_members = recipe.components[0].members.clone();
    recipe.components.truncate(2);
    recipe.components[1].members = main_members;
    let env = Environment {
        paths,
        home,
        test: Some(Arc::new(TestEnvironment {
            api: Arc::new(dsda::CannedApi::new()),
            catalog,
            engine_base: server.url(""),
            reviewed: dsda::ReviewedList::embedded(),
            runner: Arc::new(dsda::FakeEngineRunner::new(dsda::LaunchExit::Clean)),
        })),
    };
    (temp, server, env, count)
}
pub(crate) fn catalog_env(env: &Environment, change: impl FnOnce(&mut Catalog)) -> Environment {
    let previous = env.test.as_ref().unwrap();
    let mut catalog = previous.catalog.clone();
    change(&mut catalog);
    Environment {
        paths: env.paths.clone(),
        home: env.home.clone(),
        test: Some(Arc::new(TestEnvironment {
            api: previous.api.clone(),
            catalog,
            engine_base: previous.engine_base.clone(),
            reviewed: previous.reviewed.clone(),
            runner: previous.runner.clone(),
        })),
    }
}
pub(crate) fn installed(env: &Environment) -> Services {
    let mut s = Services::open(env.clone()).unwrap();
    s.install_mapset("auger-zenith", &[], &mut |_| {}).unwrap();
    s
}
fn item(s: &Services) -> Item {
    s.snapshot()
        .unwrap()
        .items
        .into_iter()
        .find(|i| i.recipe.id == "auger-zenith")
        .unwrap()
}

#[test]
fn metadata_adoption_is_atomic_and_download_free() {
    let (_temp, _server, env, hits) = setup();
    let old = installed(&env);
    let before = item(&old).manifest.unwrap();
    let changed = catalog_env(&env, |c| {
        c.recipes[0].recipe.description = "Corrected credits and metadata".into();
        c.recipes[0].recipe.packages[0].urls.reverse();
    });
    let mut s = Services::open(changed.clone()).unwrap();
    assert_eq!(item(&s).recipe_status, RecipeStatus::MetadataOnly);
    let requests = hits.load(Ordering::SeqCst);
    s.adopt_metadata().unwrap();
    let after = item(&s);
    assert_eq!(after.recipe_status, RecipeStatus::Current);
    let after = after.manifest.unwrap();
    assert_ne!(before.recipe_sha256, after.recipe_sha256);
    assert_eq!(before.members, after.members);
    assert_eq!(before.installed_at, after.installed_at);
    assert_eq!(hits.load(Ordering::SeqCst), requests);
    assert_eq!(
        item(&Services::open(changed).unwrap()).manifest.unwrap(),
        after
    );
}

#[test]
fn update_publishes_atomically_preserves_profiles_and_retries_cleanup_after_crash() {
    let (_temp, _server, env, _hits) = setup();
    let mut s = installed(&env);
    let mut p = item(&s).profile;
    let old_recipe = s.recipe("auger-zenith").unwrap();
    p.components = vec![old_recipe.components[1].id.clone()];
    p.skill = 5;
    s.execute(
        Command::Save(p.clone()),
        &CancellationToken::new(),
        &mut |_| {},
    )
    .unwrap();
    for dir in s.profiles.dirs(&p.id).all() {
        fs::write(dir.join("preserve"), b"player data").unwrap();
    }
    let previous = item(&s).manifest.unwrap();
    let old_dir = s.installer.manifest_dir(&previous).unwrap();
    let changed = catalog_env(&env, |c| {
        let r = &mut c.recipes[0].recipe;
        r.release = "new-release".into();
        r.compatibility.complevel = Complevel::Number(11);
        r.components[1].id = "new-addon".into();
        r.components[1].default = true;
    });
    let mut s = Services::open(changed.clone()).unwrap();
    let mut observed_old = false;
    let updated = s
        .update_mapset("auger-zenith", &mut |_| {
            observed_old = true;
            let reopened = Services::open(changed.clone()).unwrap();
            assert_eq!(item(&reopened).manifest.as_ref(), Some(&previous));
            for m in &previous.members {
                assert!(m.path.is_file());
            }
        })
        .unwrap();
    assert!(observed_old);
    assert_eq!(updated.release, "new-release");
    assert!(!old_dir.exists(), "cleanup is after publication");
    let after = item(&s);
    assert_eq!(after.recipe_status, RecipeStatus::Current);
    assert_eq!(after.profile.components, ["new-addon"]);
    assert_eq!(after.profile.skill, 5);
    assert!(!after.update_notice.is_empty());
    assert_eq!(after.profile.id, p.id);
    for dir in s.profiles.dirs(&p.id).all() {
        assert_eq!(fs::read(dir.join("preserve")).unwrap(), b"player data");
    }
    // Simulate a crash after publication, before best-effort cleanup.
    fs::create_dir_all(&old_dir).unwrap();
    fs::write(old_dir.join("stale"), b"retired content").unwrap();
    let mut active = s.active("auger-zenith").unwrap().unwrap();
    active.retired_dirs.push(old_dir.clone());
    s.installer.publish(&active).unwrap();
    drop(s);
    let mut reopened = Services::open(changed).unwrap();
    reopened.cleanup_retired();
    assert!(!old_dir.exists());
    assert!(
        reopened
            .active("auger-zenith")
            .unwrap()
            .unwrap()
            .retired_dirs
            .is_empty()
    );
    for m in &updated.members {
        assert!(m.path.is_file());
    }
    // Profile changes after publication still survive reopening.
    let mut p = item(&reopened).profile;
    p.skill = 2;
    reopened
        .execute(Command::Save(p), &CancellationToken::new(), &mut |_| {})
        .unwrap();
    assert_eq!(
        item(&Services::open(reopened.env.clone()).unwrap())
            .profile
            .skill,
        2
    );
}

#[test]
fn failed_download_member_verification_cancellation_and_publication_keep_old_install() {
    let (_temp, server, env, _hits) = setup();
    let s = installed(&env);
    let previous = item(&s).manifest.unwrap();
    let state_bytes = fs::read(env.paths.config.join("app.json")).unwrap();
    for kind in 0..6 {
        let changed = catalog_env(&env, |c| {
            let r = &mut c.recipes[0].recipe;
            r.release = "changed".into();
            match kind {
                0 => {
                    r.packages[0].urls = vec![server.url("/bad")];
                    r.packages[0].sha256 = "0".repeat(64);
                }
                1 => r.packages[0].members[0].sha256 = "0".repeat(64),
                4 => r.packages[0].kind = PackageKind::Zip,
                _ => {}
            }
        });
        let mut s = Services::open(changed).unwrap();
        let cancel = CancellationToken::new();
        if kind == 2 {
            cancel.cancel();
        }
        let active_path = s.installer.active_path("auger-zenith").unwrap();
        assert!(
            s.update_mapset_inner("auger-zenith", &cancel, &mut |p| {
                // Inject failure at the actual publication point, after staging.
                if kind == 3 && p.stage == ddl_core::downloads::InstallStage::Commit {
                    fs::create_dir(&active_path).unwrap();
                }
                if kind == 5 {
                    cancel.cancel();
                }
            })
            .is_err(),
            "failure case {kind}"
        );
        if kind == 3 {
            fs::remove_dir(s.installer.active_path("auger-zenith").unwrap()).unwrap();
        }
        assert_eq!(item(&s).manifest.as_ref(), Some(&previous));
        for m in &previous.members {
            assert!(m.path.is_file());
        }
        assert_eq!(
            fs::read(env.paths.config.join("app.json")).unwrap(),
            state_bytes
        );
    }
}

#[test]
fn keep_current_is_per_hash_and_removed_install_stays_resolvable() {
    let (_temp, _server, env, _hits) = setup();
    let s = installed(&env);
    let previous = item(&s).manifest.unwrap();
    let changed = catalog_env(&env, |c| {
        c.recipes[0].recipe.compatibility.complevel = Complevel::Number(11)
    });
    let mut s = Services::open(changed.clone()).unwrap();
    s.execute(
        Command::KeepCurrent("auger-zenith".into()),
        &CancellationToken::new(),
        &mut |_| {},
    )
    .unwrap();
    assert!(item(&Services::open(changed.clone()).unwrap()).update_dismissed);
    assert_eq!(item(&s).manifest.as_ref(), Some(&previous));
    let newer = catalog_env(&changed, |c| {
        c.recipes[0].recipe.description.push_str(" new metadata")
    });
    assert!(!item(&Services::open(newer).unwrap()).update_dismissed);
    let removed = catalog_env(&env, |c| c.recipes.clear());
    let s = Services::open(removed).unwrap();
    let held = item(&s);
    assert_eq!(held.recipe_status, RecipeStatus::Removed);
    assert_eq!(held.manifest, Some(previous));
    assert_eq!(
        held.recipe.compatibility.complevel,
        env.test.as_ref().unwrap().catalog.recipes[0]
            .recipe
            .compatibility
            .complevel
    );
    assert!(!matches!(
        held.launch,
        Err(Blocker::NotInstalled | Blocker::IntegrityFailed { .. })
    ));
}

#[test]
fn cli_status_and_update_use_the_shared_transaction() {
    let (_temp, _server, env, _hits) = setup();
    installed(&env);
    let changed = catalog_env(&env, |c| c.recipes[0].recipe.release = "1.1".into());
    let mut out = Vec::new();
    let mut err = Vec::new();
    crate::cli_exec::run_io(
        crate::cli::Command::MapsetStatus { id: None },
        changed.clone(),
        &mut out,
        &mut err,
    )
    .unwrap();
    assert!(String::from_utf8(out).unwrap().contains("content change"));
    let mut out = Vec::new();
    crate::cli_exec::run_io(
        crate::cli::Command::MapsetUpdate {
            id: "auger-zenith".into(),
        },
        changed.clone(),
        &mut out,
        &mut err,
    )
    .unwrap();
    assert!(
        String::from_utf8(out)
            .unwrap()
            .contains("Updated auger-zenith 1.1")
    );
    assert!(
        String::from_utf8(err)
            .unwrap()
            .contains("Old saves may not load")
    );
    assert_eq!(
        item(&Services::open(changed).unwrap()).recipe_status,
        RecipeStatus::Current
    );
}

#[test]
fn interrupted_profile_reconciliation_is_retried_from_the_active_record() {
    let (_temp, _server, env, _hits) = setup();
    let s = installed(&env);
    let mut profile = item(&s).profile;
    profile
        .components
        .push(s.recipe("auger-zenith").unwrap().components[1].id.clone());
    s.profiles.save(&profile).unwrap();
    let changed = catalog_env(&env, |c| {
        c.recipes[0].recipe.components[1].id = "new-addon".into();
        c.recipes[0].recipe.components[1].default = true;
    });
    let mut updated = Services::open(changed.clone()).unwrap();
    updated.update_mapset("auger-zenith", &mut |_| {}).unwrap();
    // Restore pre-update profile and its pending marker to model interruption
    // immediately after publication, before any profile file was reconciled.
    s.profiles.save(&profile).unwrap();
    let mut active = updated.active("auger-zenith").unwrap().unwrap();
    active.pending_profiles.push(profile.id.clone());
    updated.installer.publish(&active).unwrap();
    drop(updated);
    let mut reopened = Services::open(changed).unwrap();
    reopened
        .execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
        .unwrap();
    assert_eq!(
        reopened.profiles.load(&profile.id).unwrap().components,
        ["new-addon"]
    );
    assert!(
        reopened
            .active("auger-zenith")
            .unwrap()
            .unwrap()
            .pending_profiles
            .is_empty()
    );
    // Once reconciled, a deliberate disabled default is never enabled again.
    let mut profile = reopened.profiles.load(&profile.id).unwrap();
    profile.components.clear();
    reopened.profiles.save(&profile).unwrap();
    reopened
        .execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
        .unwrap();
    assert!(
        reopened
            .profiles
            .load(&profile.id)
            .unwrap()
            .components
            .is_empty()
    );
}

#[test]
fn removed_catalog_install_launches_through_the_fake_engine() {
    let (_temp, _server, env, _hits) = setup();
    let mut s = installed(&env);
    s.config.provision().unwrap();
    s.config.ensure_ready().unwrap();
    s.register_fake_iwads(&env.home.join("test-iwads")).unwrap();
    let executable = env.home.join("fake-engine");
    fs::write(&executable, b"#!/bin/sh\necho 'dsda-doom v0.30.1'\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    }
    s.engine.register_manual(&executable).unwrap();
    let before = s
        .resolve_mapset("auger-zenith", &LaunchOverrides::default())
        .unwrap();
    let removed = catalog_env(&env, |c| c.recipes.clear());
    let mut removed = Services::open(removed).unwrap();
    let after = removed
        .resolve_mapset("auger-zenith", &LaunchOverrides::default())
        .unwrap();
    assert_eq!(before.summary(), after.summary());
    assert!(matches!(
        removed
            .run_mapset("auger-zenith", &LaunchOverrides::default())
            .unwrap()
            .exit_status,
        dsda::LaunchExit::Clean
    ));
}

#[test]
fn cli_update_errors_explain_unknown_and_uninstalled_mapsets() {
    let (_temp, _server, env, _hits) = setup();
    let mut s = Services::open(env).unwrap();
    assert!(
        s.update_mapset("auger-zenith", &mut |_| {})
            .unwrap_err()
            .to_string()
            .contains("ddl mapset install auger-zenith")
    );
    assert!(
        s.update_mapset("missing", &mut |_| {})
            .unwrap_err()
            .to_string()
            .contains("not in the catalog")
    );
    assert!(s.mapset_status(Some("missing")).is_err());
}

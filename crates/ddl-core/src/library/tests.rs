//! Installer tests (spec 009, A1/A2): a local HTTP server serves a generated
//! archive, so no test touches the real network.

use std::io::{Cursor, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tempfile::TempDir;

use super::*;
use crate::catalog::{
    BaseGame, Compatibility, Complevel, Component, ComponentMember, EngineEvidence, Game, Image,
    Member, Package, PackageKind, PortFamily, Recipe, Role, RuntimeDefaults, Verification,
    VerificationStatus,
};
use crate::downloads::test_http::{Response, Server};
use crate::downloads::{Fetcher, HostAllowlist, Sha256};

fn sha(bytes: &[u8]) -> String {
    Sha256::of(bytes).to_hex()
}

fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        writer
            .start_file(
                *name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )
            .unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn base_recipe(id: &str) -> Recipe {
    Recipe {
        schema_version: 1,
        id: id.to_owned(),
        title: "Test mapset".to_owned(),
        authors: vec!["Test Author".to_owned()],
        release: "1.0".to_owned(),
        page: "https://example.invalid/page".to_owned(),
        description: "A synthetic mapset for tests.".to_owned(),
        permission: "Test permission text.".to_owned(),
        screenshots: Vec::new(),
        images: Vec::<Image>::new(),
        base_game: BaseGame {
            game: Game::Doom2,
            variants: Vec::new(),
        },
        packages: Vec::new(),
        components: Vec::new(),
        compatibility: Compatibility {
            family: PortFamily::Vanilla,
            complevel: Complevel::Number(2),
            engine: EngineEvidence {
                range: "=0.30.1".to_owned(),
                status: VerificationStatus::Inspected,
            },
            runtime: RuntimeDefaults::default(),
        },
        verification: Verification {
            status: VerificationStatus::Inspected,
            tester: "test".to_owned(),
            date: "2026-10-16".to_owned(),
            engine_version: "0.30.1".to_owned(),
            inspected: vec!["synthetic fixture".to_owned()],
            notes: "synthetic".to_owned(),
            limitations: Vec::new(),
        },
        tags: Vec::new(),
    }
}

/// A recipe with one ZIP package and a main + optional add-on component.
struct Fixture {
    recipe: Recipe,
    main: Vec<u8>,
    addon: Vec<u8>,
    credits: Vec<u8>,
}

fn fixture(id: &str, url: &str) -> Fixture {
    let main = b"MAIN WAD BYTES".to_vec();
    let addon = b"ADDON WAD BYTES".to_vec();
    let credits = b"permission text".to_vec();
    let inner = zip_bytes(&[("addon.wad", &addon)]);
    let zip = zip_bytes(&[
        ("main.wad", &main),
        ("addons.zip", &inner),
        ("credits.txt", &credits),
    ]);

    let mut recipe = base_recipe(id);
    recipe.packages = vec![Package {
        id: "free".to_owned(),
        page: "https://example.invalid/page".to_owned(),
        urls: vec![url.to_owned()],
        size: zip.len() as u64,
        sha256: sha(&zip),
        kind: PackageKind::Zip,
        idgames_path: None,
        members: vec![
            Member {
                id: "main".to_owned(),
                path: "main.wad".to_owned(),
                size: main.len() as u64,
                sha256: sha(&main),
                keep: false,
            },
            Member {
                id: "addon".to_owned(),
                path: "addons.zip!/addon.wad".to_owned(),
                size: addon.len() as u64,
                sha256: sha(&addon),
                keep: false,
            },
            Member {
                id: "credits".to_owned(),
                path: "credits.txt".to_owned(),
                size: credits.len() as u64,
                sha256: sha(&credits),
                keep: true,
            },
        ],
    }];
    recipe.components = vec![
        Component {
            id: "main".to_owned(),
            label: "Main mapset".to_owned(),
            description: "Main content.".to_owned(),
            optional: false,
            default: true,
            position: 1,
            exclusive_group: None,
            replaces: None,
            requires: None,
            user_supplied: false,
            obtain: None,
            known_sha256: Vec::new(),
            members: vec![ComponentMember {
                package: Some("free".to_owned()),
                member: Some("main".to_owned()),
                role: Role::Maps,
            }],
        },
        Component {
            id: "addon".to_owned(),
            label: "Optional add-on".to_owned(),
            description: "An additive WAD.".to_owned(),
            optional: true,
            default: false,
            position: 2,
            exclusive_group: None,
            replaces: None,
            requires: None,
            user_supplied: false,
            obtain: None,
            known_sha256: Vec::new(),
            members: vec![ComponentMember {
                package: Some("free".to_owned()),
                member: Some("addon".to_owned()),
                role: Role::Maps,
            }],
        },
    ];

    Fixture {
        recipe,
        main,
        addon,
        credits,
    }
}

/// A deterministic clock for `installed_at` assertions.
struct TestClock(u64);

impl Clock for TestClock {
    fn now_unix_secs(&self) -> u64 {
        self.0
    }
}

fn installer(data: &TempDir, cache: &TempDir) -> Installer {
    Installer::new(data.path(), cache.path(), HostAllowlist::new(["127.0.0.1"]))
        .with_fetcher(Fetcher::new(cache.path()).allow_http_for_tests())
}

#[test]
fn installs_main_only_then_adds_the_addon_without_redownloading() {
    let fixture = fixture("auger-test", "https://127.0.0.1/unused");
    let mut recipe = fixture.recipe.clone();
    let zip = zip_bytes(&[
        ("main.wad", &fixture.main),
        ("addons.zip", &zip_bytes(&[("addon.wad", &fixture.addon)])),
        ("credits.txt", &fixture.credits),
    ]);
    let served = zip.clone();
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&hits);
    let server = Server::start(move |_request| {
        counter.fetch_add(1, Ordering::SeqCst);
        Response::ok(served.clone())
    });
    recipe.packages[0].urls = vec![server.url("/package.zip")];
    recipe.packages[0].size = zip.len() as u64;
    recipe.packages[0].sha256 = sha(&zip);

    let data = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    let installer = installer(&data, &cache).with_clock(Box::new(TestClock(1_700_000_000)));
    let install_dir = installer.install_dir(&recipe).expect("safe path");
    assert_eq!(install_dir, data.path().join("library/auger-test/1.0"));

    // Main only: the add-on member and its package are not needed.
    let manifest = installer.install(&recipe, &[]).expect("install main");
    assert_eq!(hits.load(Ordering::SeqCst), 1, "one download");
    assert_eq!(manifest.recipe_id, "auger-test");
    assert_eq!(manifest.release, "1.0");
    assert_eq!(manifest.packages.len(), 1);
    assert_eq!(manifest.installed_at, 1_700_000_000);

    let main_path = install_dir.join("free/main.wad");
    assert_eq!(std::fs::read(&main_path).unwrap(), fixture.main);
    // A keep member is installed so permission text survives.
    let credits_path = install_dir.join("free/credits.txt");
    assert_eq!(std::fs::read(&credits_path).unwrap(), fixture.credits);
    assert!(!install_dir.join("free/addon.wad").exists());
    assert!(install_dir.join(MANIFEST_FILE).is_file());

    // Adding the add-on reuses the cached package: no second download.
    let manifest = installer
        .install(&recipe, &["addon"])
        .expect("install with addon");
    assert_eq!(hits.load(Ordering::SeqCst), 1, "cache hit, no re-download");
    assert_eq!(manifest.members.len(), 3);
    let addon_path = install_dir.join("free/addon.wad");
    assert_eq!(std::fs::read(&addon_path).unwrap(), fixture.addon);
    assert!(manifest.find_downloaded("free", "addon").is_some());
    assert!(
        manifest
            .find_downloaded("free", "credits")
            .is_some_and(|held| held.keep)
    );
    // The install time is stable across the update.
    assert_eq!(manifest.installed_at, 1_700_000_000);

    // No staging directories are left behind.
    for entry in std::fs::read_dir(data.path().join("library")).unwrap() {
        let name = entry.unwrap().file_name();
        assert!(
            !name.to_string_lossy().starts_with(".staging-"),
            "leftover staging dir: {name:?}"
        );
    }

    // The record reloads from disk and round-trips.
    let reloaded = installer
        .load_manifest(&recipe)
        .expect("load")
        .expect("installed");
    assert_eq!(reloaded, manifest);
}

#[test]
fn a_changed_recipe_is_refused_rather_than_silently_updating() {
    let fixture = fixture("changed-test", "https://127.0.0.1/unused");
    let mut recipe = fixture.recipe.clone();
    let zip = zip_bytes(&[
        ("main.wad", &fixture.main),
        ("addons.zip", &zip_bytes(&[("addon.wad", &fixture.addon)])),
        ("credits.txt", &fixture.credits),
    ]);
    let served = zip.clone();
    let server = Server::start(move |_request| Response::ok(served.clone()));
    recipe.packages[0].urls = vec![server.url("/package.zip")];
    recipe.packages[0].size = zip.len() as u64;
    recipe.packages[0].sha256 = sha(&zip);

    let data = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    let installer = installer(&data, &cache);
    installer.install(&recipe, &[]).expect("install");

    // A catalog update changes the recipe's meaning.
    recipe.compatibility.complevel = Complevel::Number(9);
    let error = installer.install(&recipe, &[]).expect_err("recipe change");
    assert!(
        matches!(error, InstallError::RecipeChanged { .. }),
        "{error}"
    );
    assert!(error.to_string().contains("changed"));
}

#[test]
fn imports_known_and_unknown_user_files_and_never_rejects_unknown_hashes() {
    let fixture = fixture("import-test", "https://127.0.0.1/unused");
    let mut recipe = fixture.recipe.clone();
    let zip = zip_bytes(&[
        ("main.wad", &fixture.main),
        ("addons.zip", &zip_bytes(&[("addon.wad", &fixture.addon)])),
        ("credits.txt", &fixture.credits),
    ]);
    let served = zip.clone();
    let server = Server::start(move |_request| Response::ok(served.clone()));
    recipe.packages[0].urls = vec![server.url("/package.zip")];
    recipe.packages[0].size = zip.len() as u64;
    recipe.packages[0].sha256 = sha(&zip);

    let known = b"purchased soundtrack bytes".to_vec();
    let unknown = b"a different soundtrack file".to_vec();
    recipe.components.push(Component {
        id: "soundtrack".to_owned(),
        label: "Paid soundtrack".to_owned(),
        description: "User supplied.".to_owned(),
        optional: true,
        default: false,
        position: 1,
        exclusive_group: Some("edition".to_owned()),
        replaces: Some("main".to_owned()),
        requires: None,
        user_supplied: true,
        obtain: Some("Import your purchased file.".to_owned()),
        known_sha256: vec![sha(&known)],
        members: vec![ComponentMember {
            package: None,
            member: None,
            role: Role::Music,
        }],
    });

    let data = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    let installer = installer(&data, &cache);

    // Importing before the mapset is installed is a typed error.
    let mut fresh = recipe.clone();
    fresh.id = "not-installed".to_owned();
    assert!(matches!(
        installer.import_user_component(&fresh, "soundtrack", &data.path().join("x.wad")),
        Err(InstallError::NotInstalled)
    ));

    installer.install(&recipe, &[]).expect("install main");

    let known_path = data.path().join("known.wad");
    std::fs::write(&known_path, &known).unwrap();
    let outcome = installer
        .import_user_component(&recipe, "soundtrack", &known_path)
        .expect("import known");
    assert!(outcome.matched, "known hash must match");
    assert_eq!(outcome.sha256, sha(&known));
    assert_eq!(std::fs::read(&outcome.path).unwrap(), known);

    // An unknown hash is accepted, only reported as unmatched (D-014).
    let unknown_path = data.path().join("unknown.wad");
    std::fs::write(&unknown_path, &unknown).unwrap();
    let outcome = installer
        .import_user_component(&recipe, "soundtrack", &unknown_path)
        .expect("import unknown");
    assert!(!outcome.matched, "unknown hash is never rejected");
    assert_eq!(outcome.sha256, sha(&unknown));

    let manifest = installer
        .load_manifest(&recipe)
        .expect("load")
        .expect("installed");
    let imported = manifest.find_imported("soundtrack").expect("recorded");
    assert_eq!(imported.sha256, sha(&unknown));
    assert_eq!(imported.path, outcome.path);

    // A downloaded component cannot be imported.
    assert!(matches!(
        installer.import_user_component(&recipe, "main", &known_path),
        Err(InstallError::NotUserSupplied { .. })
    ));
    // An unknown component is a typed error.
    assert!(matches!(
        installer.import_user_component(&recipe, "nope", &known_path),
        Err(InstallError::UnknownComponent { .. })
    ));
}

#[test]
fn installing_a_user_supplied_component_never_downloads_it() {
    let fixture = fixture("user-only", "https://127.0.0.1/unused");
    let mut recipe = fixture.recipe.clone();
    let zip = zip_bytes(&[
        ("main.wad", &fixture.main),
        ("addons.zip", &zip_bytes(&[("addon.wad", &fixture.addon)])),
        ("credits.txt", &fixture.credits),
    ]);
    let served = zip.clone();
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&hits);
    let server = Server::start(move |_request| {
        counter.fetch_add(1, Ordering::SeqCst);
        Response::ok(served.clone())
    });
    recipe.packages[0].urls = vec![server.url("/package.zip")];
    recipe.packages[0].size = zip.len() as u64;
    recipe.packages[0].sha256 = sha(&zip);
    recipe.components.push(Component {
        id: "soundtrack".to_owned(),
        label: "Paid soundtrack".to_owned(),
        description: "User supplied.".to_owned(),
        optional: true,
        default: false,
        position: 2,
        exclusive_group: None,
        replaces: None,
        requires: None,
        user_supplied: true,
        obtain: Some("Import your purchased file.".to_owned()),
        known_sha256: Vec::new(),
        members: vec![ComponentMember {
            package: None,
            member: None,
            role: Role::Music,
        }],
    });

    let data = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    let installer = installer(&data, &cache);
    let manifest = installer
        .install(&recipe, &["soundtrack"])
        .expect("install with a user-supplied component enabled");
    // Only the downloadable package was fetched; the soundtrack was not.
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    assert!(manifest.find_imported("soundtrack").is_none());
}

#[test]
fn cancellable_install_reports_stages_and_never_commits_a_cancelled_selection() {
    use crate::downloads::{CancellationToken, InstallStage};
    for stop in [
        InstallStage::Download,
        InstallStage::Extract,
        InstallStage::Verify,
    ] {
        let dir = TempDir::new().unwrap();
        let fixture = fixture("cancel-ui", "http://127.0.0.1/not-used");
        let mut recipe = fixture.recipe;
        // Use the full fixture archive (including the nested add-on), as its digest specifies.
        let inner = zip_bytes(&[("addon.wad", &fixture.addon)]);
        let body = zip_bytes(&[
            ("main.wad", &fixture.main),
            ("addons.zip", &inner),
            ("credits.txt", &fixture.credits),
        ]);
        let server = Server::start(move |_| Response::ok(body.clone()));
        recipe.packages[0].urls = vec![server.url("/package")];
        let installer = Installer::new(
            dir.path().join("data"),
            dir.path().join("cache"),
            HostAllowlist::new(["127.0.0.1"]),
        )
        .with_fetcher(Fetcher::new(dir.path().join("cache")).allow_http_for_tests());
        let cancel = CancellationToken::new();
        let mut stages = vec![];
        let result = installer.install_with_progress(&recipe, &[], &cancel, &mut |p| {
            stages.push(p.stage);
            if p.stage == stop {
                cancel.cancel();
            }
        });
        assert!(matches!(
            result,
            Err(InstallError::Download(crate::downloads::Error::Cancelled))
        ));
        assert!(installer.load_manifest(&recipe).unwrap().is_none());
        assert!(stages.contains(&stop));
    }
}

#[test]
fn main_to_replacement_to_main_keeps_valid_records_without_redownloading() {
    use crate::downloads::CancellationToken;
    let dir = TempDir::new().unwrap();
    let f = fixture("replacement", "http://127.0.0.1/placeholder");
    let inner = zip_bytes(&[("addon.wad", &f.addon)]);
    let bytes = zip_bytes(&[
        ("main.wad", &f.main),
        ("addons.zip", &inner),
        ("credits.txt", &f.credits),
    ]);
    let requests = Arc::new(AtomicUsize::new(0));
    let count = requests.clone();
    let server = Server::start(move |_| {
        count.fetch_add(1, Ordering::SeqCst);
        Response::ok(bytes.clone())
    });
    let mut recipe = f.recipe;
    recipe.packages[0].urls = vec![server.url("/package")];
    recipe.components[1].replaces = Some("main".into());
    recipe.components[1].position = 1;
    let installer = Installer::new(
        dir.path().join("data"),
        dir.path().join("cache"),
        HostAllowlist::new(["127.0.0.1"]),
    )
    .with_fetcher(Fetcher::new(dir.path().join("cache")).allow_http_for_tests());
    let main = installer.install(&recipe, &[]).unwrap();
    let path = main.find_downloaded("free", "main").unwrap().path.clone();
    let alternate = installer.install(&recipe, &["addon"]).unwrap();
    assert!(alternate.find_downloaded("free", "main").is_some());
    assert!(alternate.find_downloaded("free", "addon").is_some());
    let restored = installer.install(&recipe, &[]).unwrap();
    assert!(restored.find_downloaded("free", "addon").is_some());
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    // Same-size drift of an unused member is dropped, rather than blessed in the new record.
    std::fs::write(&path, vec![b'X'; f.main.len()]).unwrap();
    let alternate = installer.install(&recipe, &["addon"]).unwrap();
    assert!(alternate.find_downloaded("free", "main").is_none());
    // Selecting main repairs it from the verified cached package, still without a network request.
    installer.install(&recipe, &[]).unwrap();
    assert_eq!(std::fs::read(path).unwrap(), f.main);
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(matches!(
        installer.import_user_component_with_cancel(&recipe, "unknown", dir.path(), &cancel),
        Err(InstallError::Download(crate::downloads::Error::Cancelled))
    ));
}

#[test]
fn recipe_change_categories_and_combinations() {
    let old = fixture("changes", "https://example.invalid/maps.zip").recipe;
    let manifest = InstallManifest {
        schema_version: SCHEMA_VERSION,
        recipe_id: old.id.clone(),
        release: old.release.clone(),
        recipe_sha256: super::install::recipe_content_sha256(&old).unwrap(),
        installed_at: 1,
        packages: vec![],
        members: vec![],
    };
    assert_eq!(classify(&manifest, &old, Some(&old)), RecipeStatus::Current);
    assert_eq!(classify(&manifest, &old, None), RecipeStatus::Removed);
    let mut new = old.clone();
    new.authors.push("Another author".into());
    new.compatibility.engine.status = VerificationStatus::Verified;
    new.tags.push("test".into());
    new.packages[0]
        .urls
        .push("https://mirror.invalid/maps.zip".into());
    new.packages[0].members.reverse();
    assert_eq!(
        classify(&manifest, &old, Some(&new)),
        RecipeStatus::MetadataOnly
    );
    for mutate in [
        |r: &mut Recipe| r.compatibility.complevel = Complevel::Number(11),
        |r: &mut Recipe| r.base_game.game = Game::Doom,
        |r: &mut Recipe| r.components[0].position = 5,
        |r: &mut Recipe| r.components[0].members[0].role = Role::Dehacked,
    ] {
        let mut new = old.clone();
        mutate(&mut new);
        assert!(matches!(
            classify(&manifest, &old, Some(&new)),
            RecipeStatus::Update(Changes {
                launch: true,
                content: false,
                ..
            })
        ));
    }
    for mutate in [
        |r: &mut Recipe| r.release = "1.1".into(),
        |r: &mut Recipe| r.packages[0].sha256 = "a".repeat(64),
        |r: &mut Recipe| r.packages[0].members[0].sha256 = "a".repeat(64),
    ] {
        let mut new = old.clone();
        mutate(&mut new);
        assert!(matches!(
            classify(&manifest, &old, Some(&new)),
            RecipeStatus::Update(Changes { content: true, .. })
        ));
    }
    let mut new = old.clone();
    new.components[1].id = "new-addon".into();
    let RecipeStatus::Update(changes) = classify(&manifest, &old, Some(&new)) else {
        panic!()
    };
    assert_eq!(changes.added_components, ["new-addon"]);
    assert_eq!(changes.removed_components, ["addon"]);
    assert!(!changes.launch && !changes.content);
    new.release = "1.1".into();
    new.compatibility.complevel = Complevel::Number(11);
    let status = classify(&manifest, &old, Some(&new));
    assert!(matches!(&status, RecipeStatus::Update(c) if c.launch && c.content && c.components()));
    assert!(
        status
            .summary(&old, Some(&new))
            .contains("complevel 2 → 11")
    );
}

#[test]
fn profile_reconciliation_drops_removed_and_applies_only_new_defaults() {
    let old = fixture("changes", "https://example.invalid/maps.zip").recipe;
    let mut new = old.clone();
    new.components[1].id = "new-addon".into();
    new.components[1].default = true;
    let mut profile = crate::profiles::Profile::new("run", "Run", &old.id);
    profile.components.push("addon".into());
    profile.skill = 5;
    profile.last_played = Some(100);
    assert_eq!(reconcile_profile(&mut profile, &old, &new), ["addon"]);
    assert_eq!(profile.components, ["new-addon"]);
    assert_eq!(profile.skill, 5);
    assert_eq!(profile.last_played, Some(100));
}

#[test]
fn retired_cleanup_refuses_profile_paths_and_symlink_escapes() {
    let data = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    let installer = installer(&data, &cache);
    let save = data.path().join("profiles/run/save");
    std::fs::create_dir_all(&save).unwrap();
    std::fs::write(save.join("save.dsg"), b"save").unwrap();
    assert!(installer.remove_retired(&save).is_err());
    assert!(
        installer
            .remove_retired(&data.path().join("library/id/../../profiles/run"))
            .is_err()
    );
    #[cfg(unix)]
    {
        std::fs::create_dir_all(data.path().join("library/id")).unwrap();
        std::os::unix::fs::symlink(&save, data.path().join("library/id/old")).unwrap();
        assert!(
            installer
                .remove_retired(&data.path().join("library/id/old"))
                .is_err()
        );
    }
    assert_eq!(std::fs::read(save.join("save.dsg")).unwrap(), b"save");
}

#[test]
fn active_record_round_trips_and_rejects_unknown_manifest_versions() {
    let recipe = fixture("active", "https://example.invalid/maps.zip").recipe;
    let active = ActiveInstall {
        schema_version: 1,
        manifest: InstallManifest {
            schema_version: SCHEMA_VERSION,
            recipe_id: recipe.id.clone(),
            release: recipe.release.clone(),
            recipe_sha256: recipe_content_sha256(&recipe).unwrap(),
            installed_at: 1,
            packages: vec![],
            members: vec![],
        },
        recipe,
        previous_recipe: None,
        pending_profiles: vec![],
        notices: vec![],
        retired_dirs: vec![],
        uninstalled: false,
    };
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("active.json");
    active.save(&path).unwrap();
    assert_eq!(ActiveInstall::load(&path).unwrap(), Some(active.clone()));
    let mut bad = active;
    bad.manifest.schema_version = 99;
    bad.save(&path).unwrap();
    assert!(matches!(
        ActiveInstall::load(&path),
        Err(ManifestError::UnsupportedSchema(99))
    ));
}

/// Installs `fixture(id)` from a loopback server, returning the installer.
fn installed_fixture(id: &str, data: &TempDir, cache: &TempDir) -> (Installer, Recipe, Vec<u8>) {
    let f = fixture(id, "http://127.0.0.1/placeholder");
    let bytes = zip_bytes(&[("main.wad", &f.main), ("credits.txt", &f.credits)]);
    let served = bytes.clone();
    let server = Server::start(move |_| Response::ok(served.clone()));
    let mut recipe = f.recipe;
    recipe.packages[0].urls = vec![server.url("/package")];
    recipe.packages[0].size = bytes.len() as u64;
    recipe.packages[0].sha256 = sha(&bytes);
    let installer = installer(data, cache);
    installer.install(&recipe, &[]).expect("install");
    (installer, recipe, bytes)
}

#[test]
fn uninstall_marks_inactive_first_and_removes_only_this_recipe() {
    let data = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    let (installer, recipe, _bytes) = installed_fixture("uninstall-test", &data, &cache);
    let manifest = installer
        .load_manifest(&recipe)
        .unwrap()
        .expect("installed");
    // A neighbouring mapset and the download cache must never be touched.
    let neighbour = data.path().join("library/other/1.0");
    std::fs::create_dir_all(&neighbour).unwrap();
    std::fs::write(neighbour.join("keep"), b"keep").unwrap();
    let cached = std::fs::read_dir(cache.path()).unwrap().count();

    installer.begin_uninstall(&recipe, &manifest).unwrap();
    // The mark is atomic and immediate: the install is already inactive ...
    assert_eq!(installer.load_manifest(&recipe).unwrap(), None);
    assert!(installer.active(&recipe.id).unwrap().unwrap().uninstalled);
    // ... but nothing has been deleted yet.
    assert!(manifest.members.iter().all(|m| m.path.is_file()));

    assert!(installer.finish_uninstall(&recipe.id).unwrap());
    assert!(!installer.library_dir(&recipe.id).unwrap().exists());
    assert_eq!(std::fs::read(neighbour.join("keep")).unwrap(), b"keep");
    assert_eq!(std::fs::read_dir(cache.path()).unwrap().count(), cached);
    // Finishing again is a no-op once the record is gone.
    assert!(!installer.finish_uninstall(&recipe.id).unwrap());
}

#[cfg(unix)]
#[test]
fn uninstall_never_follows_a_symlinked_directory_out_of_the_library() {
    let data = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    let (installer, recipe, _bytes) = installed_fixture("unlink-test", &data, &cache);
    let manifest = installer
        .load_manifest(&recipe)
        .unwrap()
        .expect("installed");
    let outside = data.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("sentinel"), b"safe").unwrap();
    std::os::unix::fs::symlink(&outside, data.path().join("library/unlink-test/escape")).unwrap();

    installer.begin_uninstall(&recipe, &manifest).unwrap();
    assert!(installer.finish_uninstall(&recipe.id).unwrap());
    // The link is removed as a link; its target is untouched.
    assert_eq!(std::fs::read(outside.join("sentinel")).unwrap(), b"safe");
    assert!(!data.path().join("library/unlink-test").exists());
}

#[test]
fn uninstall_refuses_records_whose_recipe_id_is_unsafe() {
    let data = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    let installer = installer(&data, &cache);
    let recipe = fixture("safe", "https://example.invalid/maps.zip").recipe;
    let manifest = InstallManifest {
        schema_version: SCHEMA_VERSION,
        recipe_id: recipe.id.clone(),
        release: recipe.release.clone(),
        recipe_sha256: super::install::recipe_content_sha256(&recipe).unwrap(),
        installed_at: 1,
        packages: vec![],
        members: vec![],
    };
    assert!(installer.library_dir("../escape").is_err());
    assert!(installer.begin_uninstall(&recipe, &manifest).is_ok());
    assert!(installer.finish_uninstall(&recipe.id).unwrap());
    assert!(!data.path().join("library/safe").exists());
}

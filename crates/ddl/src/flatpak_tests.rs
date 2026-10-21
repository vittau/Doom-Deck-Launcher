//! Spec 059: the Flatpak-ready behaviour that does not need a window — hidden
//! host paths (R4) and importing a tarball-era install through the spec 055
//! readers (R5). Everything runs against fake homes and a fake sandbox marker.
use super::*;
use ddl_core::platform::legacy_import::{self, Decision, LegacyInstall};

const APP: &str = "io.github.vittau.Doom-Deck-Launcher";

pub(crate) fn flatpak() -> Sandbox {
    Sandbox::flatpak(Some(APP.to_owned()))
}

/// Paths as Flatpak sets them: HOME is the real home, every XDG base
/// directory lives in `~/.var/app/<id>/`.
pub(crate) fn sandbox_env(root: &Path) -> Environment {
    let home = root.join("home");
    let app = home.join(".var/app").join(APP);
    fs::create_dir_all(&home).unwrap();
    let vars = [
        ("HOME", home.clone()),
        ("XDG_CONFIG_HOME", app.join("config")),
        ("XDG_DATA_HOME", app.join("data")),
        ("XDG_CACHE_HOME", app.join("cache")),
        ("XDG_STATE_HOME", app.join(".local/state")),
    ];
    let paths = ddl_core::platform::paths::resolve(&|key: &str| {
        vars.iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| value.display().to_string())
    })
    .unwrap();
    Environment {
        paths,
        home,
        screen: None,
        test: None,
    }
}

/// A Steam root whose `libraryfolders.vdf` names a library inside the home and
/// an SD card the sandbox cannot see.
pub(crate) fn steam_with_hidden_sd_card(home: &Path, root: &Path) -> (PathBuf, PathBuf) {
    let visible = home.join("SteamLibrary");
    let sd_card = root.join("run/media/deck/sd-card/SteamLibrary");
    fs::create_dir_all(visible.join("steamapps/common")).unwrap();
    let steamapps = home.join(".local/share/Steam/steamapps");
    fs::create_dir_all(&steamapps).unwrap();
    fs::write(
        steamapps.join("libraryfolders.vdf"),
        format!(
            "\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\"{}\"\n\t}}\n\t\"1\"\n\t{{\n\t\t\"path\"\t\"{}\"\n\t}}\n}}\n",
            visible.display(),
            sd_card.display()
        ),
    )
    .unwrap();
    (visible, sd_card)
}

/// Spec 059 A2: the "not visible" path shows up in the snapshot under Flatpak
/// only, and the visible library is still found.
#[test]
fn a_steam_library_the_flatpak_cannot_see_is_reported_not_silently_empty() {
    let temp = tempfile::tempdir().unwrap();
    let env = sandbox_env(temp.path());
    let (visible, sd_card) = steam_with_hidden_sd_card(&env.home, temp.path());

    let sandboxed = Services::open_with(env.clone(), flatpak()).unwrap();
    let snapshot = sandboxed.snapshot().unwrap();
    assert_eq!(snapshot.hidden_paths, std::slice::from_ref(&sd_card));

    // Outside the sandbox the same files report nothing: behaviour unchanged.
    let plain = Services::open_with(env.clone(), Sandbox::none()).unwrap();
    assert!(plain.snapshot().unwrap().hidden_paths.is_empty());

    // The visible library still reaches discovery and the picker.
    assert!(
        iwads::libraries(&env.home)
            .iter()
            .any(|library| library.path == visible)
    );
    let directory = sandboxed
        .browse(&env.home, false, &CancellationToken::new())
        .unwrap();
    assert!(directory.shortcuts.contains(&env.home));
    assert!(
        directory
            .shortcuts
            .contains(&visible.join("steamapps/common"))
    );
    assert!(
        !directory.shortcuts.iter().any(|p| p.starts_with(&sd_card)),
        "the hidden library is not offered as a place"
    );
}

#[test]
fn a_configured_folder_that_vanished_from_the_sandbox_is_named() {
    let temp = tempfile::tempdir().unwrap();
    let env = sandbox_env(temp.path());
    let gone = temp.path().join("run/media/deck/usb/Doom");
    let mut services = Services::open_with(env.clone(), flatpak()).unwrap();
    services.state.folders.push(gone.clone());
    let present = env.home.join("Doom");
    fs::create_dir_all(&present).unwrap();
    services.state.folders.push(present.clone());

    let snapshot = services.snapshot().unwrap();

    assert_eq!(snapshot.hidden_paths, [gone]);
    assert!(!snapshot.hidden_paths.contains(&present));
}

#[test]
fn browsing_a_folder_the_flatpak_cannot_open_says_so() {
    let temp = tempfile::tempdir().unwrap();
    let env = sandbox_env(temp.path());
    let missing = temp.path().join("run/media/deck/sd-card");
    let sandboxed = Services::open_with(env.clone(), flatpak()).unwrap();
    let plain = Services::open_with(env, Sandbox::none()).unwrap();

    match sandboxed.browse(&missing, true, &CancellationToken::new()) {
        Err(Problem::NotVisible(path)) => assert_eq!(path, missing),
        other => panic!("expected NotVisible, got {other:?}"),
    }
    // Unsandboxed, the same failure stays the generic storage problem.
    assert!(matches!(
        plain.browse(&missing, true, &CancellationToken::new()),
        Err(Problem::Storage)
    ));
    let text = Problem::NotVisible(missing.clone()).to_string();
    assert_eq!(
        text,
        format!(
            "{} is not visible to the Flatpak — see the guide",
            missing.display()
        )
    );
}

/// Where a tarball install keeps its state, filled from a spec 055 fixture
/// with the recorded paths pointing at that real location. The fixture's two
/// synthetic IWAD folders become real (empty) folders in the home, outside
/// DDL's own directories, as a player's would be.
pub(crate) fn install_fixture(tag: &str, home: &Path) -> LegacyInstall {
    fn place(from: &Path, to: &Path, replacements: &[(&str, String)]) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap().flatten() {
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                place(&entry.path(), &target, replacements);
            } else {
                let mut text = fs::read_to_string(entry.path()).unwrap();
                for (recorded, real) in replacements {
                    text = text.replace(recorded, real);
                }
                fs::write(target, text).unwrap();
            }
        }
    }
    let legacy = LegacyInstall::under(home);
    let tree = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/state")
        .join(tag);
    let folders = iwad_folders(home);
    for folder in &folders {
        fs::create_dir_all(folder).unwrap();
    }
    let replacements = [
        ("/ddl-fixture/config", legacy.config.display().to_string()),
        ("/ddl-fixture/data", legacy.data.display().to_string()),
        ("/ddl-fixture/iwads-one", folders[0].display().to_string()),
        ("/ddl-fixture/iwads-two", folders[1].display().to_string()),
    ];
    place(&tree.join("config"), &legacy.config, &replacements);
    place(&tree.join("data"), &legacy.data, &replacements);
    legacy
}

/// The player's IWAD folders the fixture refers to.
pub(crate) fn iwad_folders(home: &Path) -> [PathBuf; 2] {
    [home.join("iwads-one"), home.join("iwads-two")]
}

/// Spec 059 A3: every spec 055 fixture imports into an empty sandbox and the
/// current readers load it — engine, IWAD folders, installs, profiles.
#[test]
fn every_release_fixture_imports_and_loads_in_the_sandbox() {
    for tag in ["v0.0.5-screens.1", "v0.0.6-local", "v0.0.7-deck"] {
        let temp = tempfile::tempdir().unwrap();
        let env = sandbox_env(temp.path());
        let legacy = install_fixture(tag, &env.home);
        assert_eq!(
            legacy_import::find(&flatpak(), &env.home, &env.paths),
            Some(legacy.clone()),
            "{tag}: the offer applies to an empty sandbox"
        );

        let report = import_legacy(&env, &legacy, &CancellationToken::new())
            .unwrap_or_else(|e| panic!("{tag}: {e}"));
        assert!(report.rewritten > 0, "{tag}");
        assert_eq!(
            legacy_import::recorded_decision(&env.paths),
            Some(Decision::Imported)
        );
        assert_eq!(legacy_import::find(&flatpak(), &env.home, &env.paths), None);

        let services =
            Services::open_with(env.clone(), flatpak()).unwrap_or_else(|e| panic!("{tag}: {e}"));
        let snapshot = services.snapshot().unwrap_or_else(|e| panic!("{tag}: {e}"));
        assert_eq!(snapshot.engine.unwrap().version, "0.30.1", "{tag}");
        // IWAD references keep pointing at the player's files.
        assert_eq!(services.state.folders, iwad_folders(&env.home), "{tag}");
        assert_eq!(services.registry.entries().len(), 2, "{tag}");
        let installed: Vec<_> = snapshot
            .items
            .iter()
            .filter(|i| i.manifest.is_some())
            .collect();
        assert_eq!(
            installed.len(),
            2,
            "{tag}: both installed mapsets came across"
        );
        let auger = installed
            .iter()
            .find(|i| i.recipe.id == "auger-zenith")
            .unwrap();
        assert_eq!(auger.profiles.len(), 2, "{tag}");
        // Installed content now lives under the sandbox's data directory.
        let manifest = auger.manifest.as_ref().unwrap();
        assert!(
            manifest
                .members
                .iter()
                .all(|m| m.path.starts_with(&env.paths.data)),
            "{tag}: member paths were rewritten"
        );
        // The player's original files are untouched.
        assert!(legacy.config.join("app.json").is_file());
        assert!(
            legacy
                .data
                .join("library/auger-zenith/active.json")
                .is_file()
        );
    }
}

//! Startup validation and reversible recovery independent of a Services snapshot.
use super::*;
use std::io::Read;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnreadableFile {
    pub path: PathBuf,
    pub reason: String,
}

fn inspect<T>(
    path: &Path,
    load: impl FnOnce(&Path) -> Result<T, String>,
    errors: &mut Vec<UnreadableFile>,
) {
    let result = match fs::metadata(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return,
        Err(e) => Err(e.to_string()),
        Ok(m) if m.len() > 16 * 1024 * 1024 => Err("file exceeds the 16 MiB read limit".into()),
        Ok(_) => load(path).map(|_| ()),
    };
    if let Err(reason) = result {
        errors.push(UnreadableFile {
            path: path.into(),
            reason,
        });
    }
}

/// Bounded app-state parser with useful, single-line diagnostics.
pub(super) fn read_app(path: &Path) -> Result<AppState, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|f| f.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes))
        .map_err(|e| e.to_string())?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("file exceeds the 4 MiB read limit".into());
    }
    AppState::decode(&bytes)
}

fn entries(dir: &Path) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

/// Inventory the files whose failure otherwise prevents startup/snapshot.
/// Cache corruption and DSDA config's existing reset notice remain nonfatal.
pub fn unreadable_files(env: &Environment) -> Vec<UnreadableFile> {
    let mut errors = vec![];
    let paths = &env.paths;
    inspect(&paths.config.join("app.json"), read_app, &mut errors);
    inspect(
        &paths.config.join("iwads.json"),
        |p| Registry::load(p).map_err(|e| e.to_string()),
        &mut errors,
    );
    inspect(
        &EngineManager::for_paths(paths).state_path(),
        |p| dsda::InstalledState::load(p).map_err(|e| e.to_string()),
        &mut errors,
    );
    let profiles = ProfileStore::new(&paths.config, &paths.data);
    for path in entries(&profiles.root()) {
        if path.extension().is_some_and(|e| e == "json") {
            inspect(
                &path,
                |p| {
                    profiles
                        .load(p.file_stem().unwrap().to_str().unwrap_or(""))
                        .map_err(|e| e.to_string())
                },
                &mut errors,
            );
        }
    }
    let custom = SetupStore::new(&paths.data);
    for path in entries(&custom.root()) {
        if path.extension().is_some_and(|e| e == "json") {
            inspect(
                &path,
                |p| {
                    custom
                        .load(p.file_stem().unwrap().to_str().unwrap_or(""))
                        .map_err(|e| e.to_string())
                },
                &mut errors,
            );
        }
    }
    for dir in entries(&paths.data.join("library")) {
        // Only known state locations; never follow user-provided links.
        if !fs::symlink_metadata(&dir).is_ok_and(|m| m.is_dir()) {
            continue;
        }
        inspect(
            &dir.join("active.json"),
            |p| ActiveInstall::load(p).map_err(|e| e.to_string()),
            &mut errors,
        );
        for generation in entries(&dir) {
            if !fs::symlink_metadata(&generation).is_ok_and(|m| m.is_dir()) {
                continue;
            }
            if generation.file_name().is_some_and(|n| n == "generations") {
                for g in entries(&generation) {
                    if fs::symlink_metadata(&g).is_ok_and(|m| m.is_dir()) {
                        inspect(
                            &g.join("install.json"),
                            |p| InstallManifest::load(p).map_err(|e| e.to_string()),
                            &mut errors,
                        );
                    }
                }
            } else {
                inspect(
                    &generation.join("install.json"),
                    |p| InstallManifest::load(p).map_err(|e| e.to_string()),
                    &mut errors,
                );
            }
        }
    }
    errors
}

/// Revalidate under the normal state lock, then rename only still-unreadable
/// records. A corrected file is never moved, and backup names never overwrite.
pub fn start_fresh(env: &Environment) -> Result<Vec<PathBuf>, Problem> {
    let _lock = StateLock::acquire(&env.paths.config).map_err(|e| logged(Problem::Busy, e))?;
    unreadable_files(env)
        .into_iter()
        .map(|file| preserve_unreadable(&file.path))
        .collect()
}

fn preserve_unreadable(path: &Path) -> Result<PathBuf, Problem> {
    let timestamp = dsda::format_unix_utc(now_unix_secs()).replace(':', "");
    let name = path.file_name().ok_or(Problem::Storage)?.to_string_lossy();
    let mut target = path.with_file_name(format!("{name}.unreadable-{timestamp}"));
    let mut suffix = 0;
    while fs::symlink_metadata(&target).is_ok() {
        suffix += 1;
        target = path.with_file_name(format!("{name}.unreadable-{timestamp}-{suffix}"));
    }
    fs::rename(path, &target).map_err(|e| logged(Problem::Storage, e))?;
    Ok(target)
}

/// DSDA's rebuildable bookkeeping keeps its existing nonfatal reset flow, but
/// an explicit reset preserves unreadable bookkeeping before replacing it.
pub(super) fn preserve_config_before_reset(config: &DsdaConfigManager) -> Result<(), Problem> {
    match config.state() {
        Ok(_) => Ok(()),
        Err(ddl_core::dsda_config::Error::InvalidState(error)) => {
            if ddl_core::dsda_config::RunGuard::is_active(config.dir())
                .map_err(|e| logged(Problem::Config, e))?
            {
                return Err(Problem::Config);
            }
            tracing::warn!(%error, "preserving corrupt DSDA config state before reset");
            preserve_unreadable(&config.state_path()).map(|_| ())
        }
        Err(error) => Err(logged(Problem::Config, error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn environment(root: &Path) -> Environment {
        let paths = Paths {
            config: root.join("config"),
            data: root.join("data"),
            cache: root.join("cache"),
            state: root.join("state"),
        };
        fs::create_dir_all(&paths.config).unwrap();
        Environment {
            paths,
            home: root.into(),
            test: None,
        }
    }
    #[test]
    fn identifies_corrupt_and_future_files_and_preserves_valid_state() {
        let temp = tempfile::tempdir().unwrap();
        let env = environment(temp.path());
        let state = AppState {
            schema_version: 99,
            ..AppState::default()
        };
        save_json(&env.paths.config.join("app.json"), &state).unwrap();
        fs::write(env.paths.config.join("iwads.json"), b"broken registry").unwrap();
        let profiles = ProfileStore::new(&env.paths.config, &env.paths.data);
        let profile = Profile::new("kept", "Keep my profile", "sigil");
        profiles.save(&profile).unwrap();
        let profile_path = profiles.path("kept").unwrap();
        let original = fs::read(&profile_path).unwrap();
        let errors = unreadable_files(&env);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].reason.contains("unsupported schema version 99"));
        let moved = start_fresh(&env).unwrap();
        assert_eq!(moved.len(), 2);
        assert_eq!(fs::read(&moved[1]).unwrap(), b"broken registry");
        assert_eq!(fs::read(profile_path).unwrap(), original);
        assert!(unreadable_files(&env).is_empty());
        assert!(Services::open(env).is_ok());
    }
    #[test]
    fn revalidates_before_renaming_and_never_overwrites_a_backup() {
        let temp = tempfile::tempdir().unwrap();
        let env = environment(temp.path());
        let path = env.paths.config.join("app.json");
        fs::write(&path, b"first corruption").unwrap();
        assert_eq!(unreadable_files(&env).len(), 1);
        save_json(&path, &AppState::default()).unwrap();
        assert!(
            start_fresh(&env).unwrap().is_empty(),
            "a corrected file must stay"
        );
        fs::write(&path, b"first corruption").unwrap();
        let first = start_fresh(&env).unwrap().pop().unwrap();
        fs::write(&path, b"second corruption").unwrap();
        let second = start_fresh(&env).unwrap().pop().unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read(first).unwrap(), b"first corruption");
        assert_eq!(fs::read(second).unwrap(), b"second corruption");
    }
}

#[cfg(test)]
mod surviving_content_tests {
    use super::*;
    fn copy_tree(source: &Path, target: &Path) {
        fs::create_dir_all(target).unwrap();
        for entry in fs::read_dir(source).unwrap().flatten() {
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target.join(entry.file_name()));
            } else {
                fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
            }
        }
    }
    #[test]
    fn fresh_app_rediscovers_surviving_engine_installs_profiles_and_registry() {
        let temp = tempfile::tempdir().unwrap();
        let fixture =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/state/v0.0.6-local");
        copy_tree(&fixture, temp.path());
        let env = Environment {
            paths: Paths {
                config: temp.path().join("config"),
                data: temp.path().join("data"),
                cache: temp.path().join("cache"),
                state: temp.path().join("state"),
            },
            home: temp.path().into(),
            test: None,
        };
        fs::write(env.paths.config.join("app.json"), b"unreadable app state").unwrap();
        let backups = start_fresh(&env).unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), b"unreadable app state");
        let services = Services::open(env).unwrap();
        let snapshot = services.snapshot().unwrap();
        assert!(snapshot.engine.is_some());
        assert_eq!(
            snapshot
                .items
                .iter()
                .filter(|i| i.manifest.is_some())
                .count(),
            2
        );
        assert_eq!(services.installed_ids().len(), 2);
        assert_eq!(services.profiles.list().unwrap().len(), 3);
        assert_eq!(services.registry.entries().len(), 2);
        assert_eq!(snapshot.custom.len(), 1);
    }
}

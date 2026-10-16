//! Thin application services. Constructed and used exclusively on a worker thread.
use ddl_core::{
    catalog::{Catalog, Recipe},
    downloads::{CancellationToken, HostAllowlist, InstallProgress},
    dsda_config::DsdaConfigManager,
    engines::dsda::{self, EngineManager, EngineRunner},
    iwads::{self, Registry, RegistryEntry},
    library::{InstallManifest, Installer},
    platform::paths::Paths,
    profiles::{Profile, ProfileStore},
    resolver::{self, Blocker, Summary},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    Storage,
    State,
    Network,
    Update,
    Checksum,
    UnsafeArchive,
    Cancelled,
    Engine,
    Config,
    UnknownFile,
    Launch,
}
fn logged(kind: Problem, error: impl std::fmt::Display) -> Problem {
    tracing::warn!(%error, ?kind, "application operation failed");
    kind
}
fn download_error(error: &ddl_core::downloads::Error) -> Problem {
    use ddl_core::downloads::Error as E;
    match error {
        E::Cancelled => Problem::Cancelled,
        E::HashMismatch { .. } => Problem::Checksum,
        E::Network { .. } | E::HttpStatus { .. } => Problem::Network,
        E::Io { .. } => Problem::Storage,
        _ => Problem::UnsafeArchive,
    }
}
fn install_error(error: ddl_core::library::InstallError) -> Problem {
    let kind = match &error {
        ddl_core::library::InstallError::Download(e) => download_error(e),
        ddl_core::library::InstallError::Io { .. } => Problem::Storage,
        _ => Problem::State,
    };
    logged(kind, error)
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppState {
    schema_version: u32,
    folders: Vec<PathBuf>,
    recipes: BTreeMap<String, Recipe>,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            folders: vec![],
            recipes: BTreeMap::new(),
        }
    }
}
/// Versioned machine state, replace by rename after flushing the complete file.
fn save_json(path: &Path, value: &impl Serialize) -> Result<(), Problem> {
    let write = || -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(
            path.parent()
                .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?,
        )?;
        let temp = path.with_extension("json.tmp");
        let mut file = fs::File::create(&temp)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(&serde_json::to_vec_pretty(value)?)?;
        file.sync_all()?;
        fs::rename(temp, path)?;
        Ok(())
    };
    write().map_err(|e| logged(Problem::Storage, e))
}
#[derive(Debug, Clone)]
pub struct Item {
    pub recipe: Recipe,
    pub manifest: Option<InstallManifest>,
    pub profile: Profile,
    pub launch: Result<Summary, Blocker>,
}
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub items: Vec<Item>,
    pub engine: Option<dsda::InstalledEngine>,
    pub previous: Option<String>,
    pub attribution: Option<dsda::Attribution>,
    pub iwads: Vec<RegistryEntry>,
    pub folders: Vec<PathBuf>,
    pub config_notice: bool,
    pub config_usable: bool,
    pub config_version: String,
    pub logs: PathBuf,
}
#[derive(Debug, Clone)]
pub struct Directory {
    pub path: PathBuf,
    pub entries: Vec<(PathBuf, bool)>,
    pub shortcuts: Vec<PathBuf>,
}
#[derive(Debug, Clone)]
pub enum Command {
    Startup,
    InstallEngine(String),
    Scan(Option<PathBuf>),
    Choose(PathBuf, String),
    RemoveFolder(PathBuf),
    Install(String),
    Save(Profile),
    Play(String),
    Browse(PathBuf, bool),
    Updates,
    Rollback,
    Reset,
    Import(String, String, PathBuf),
}
#[derive(Debug)]
pub enum Event {
    Progress(InstallProgress),
    Working,
    Playing,
    Directory(Directory),
    Updates(Vec<String>),
    Finished(Result<Box<Snapshot>, Problem>),
    EngineExit(dsda::LaunchOutcome),
}
/// Test overrides are inert unless explicitly provided by the headless tests.
#[derive(Clone)]
pub struct Environment {
    pub paths: Paths,
    pub home: PathBuf,
    #[cfg(test)]
    pub test: Option<std::sync::Arc<TestEnvironment>>,
}
#[cfg(test)]
pub struct TestEnvironment {
    pub api: std::sync::Arc<dsda::CannedApi>,
    pub catalog: Catalog,
    pub engine_base: String,
    pub reviewed: dsda::ReviewedList,
    pub runner: std::sync::Arc<dyn EngineRunner + Send + Sync>,
}

pub struct Services {
    env: Environment,
    catalog: Catalog,
    state: AppState,
    registry: Registry,
    config: DsdaConfigManager,
    installer: Installer,
    engine: EngineManager,
    profiles: ProfileStore,
}
impl Services {
    pub fn open(env: Environment) -> Result<Self, Problem> {
        let paths = &env.paths;
        for dir in [&paths.config, &paths.data, &paths.cache, &paths.state] {
            fs::create_dir_all(dir).map_err(|e| logged(Problem::Storage, e))?;
        }
        let state_path = paths.config.join("app.json");
        let state: AppState = match fs::read(&state_path) {
            Ok(bytes) if bytes.len() <= 4 * 1024 * 1024 => {
                serde_json::from_slice(&bytes).map_err(|e| logged(Problem::State, e))?
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => AppState::default(),
            result => {
                return Err(logged(
                    Problem::State,
                    format!("app state unavailable: {result:?}"),
                ));
            }
        };
        if state.schema_version != 1 {
            return Err(Problem::State);
        }
        let catalog = Catalog::embedded();
        #[cfg(test)]
        let catalog = env.test.as_ref().map_or(catalog, |t| t.catalog.clone());
        let installer = Installer::new(
            &paths.data,
            &paths.cache,
            HostAllowlist::new(catalog.hosts.downloads.clone()),
        );
        #[cfg(test)]
        let installer = if env.test.is_some() {
            installer.with_fetcher(
                ddl_core::downloads::Fetcher::new(&paths.cache).allow_http_for_tests(),
            )
        } else {
            installer
        };
        let engine = EngineManager::for_paths(paths);
        #[cfg(test)]
        let engine = if let Some(test) = &env.test {
            EngineManager::with_parts(
                dsda::EngineDirs::for_paths(paths),
                test.reviewed.clone(),
                Box::new(TestApi(test.api.clone())),
                Box::new(dsda::SystemClock),
            )
            .with_http_for_tests()
            .with_download_base(&test.engine_base)
        } else {
            engine
        };
        let config =
            DsdaConfigManager::new(&paths.config).map_err(|e| logged(Problem::Config, e))?;
        let profiles = ProfileStore::new(&paths.config, &paths.data);
        let registry_path = paths.config.join("iwads.json");
        let registry = match Registry::load(&registry_path) {
            Ok(registry) => registry,
            Err(iwads::RegistryError::Io(e)) if e.kind() == io::ErrorKind::NotFound => {
                Registry::new()
            }
            Err(e) => return Err(logged(Problem::State, e)),
        };
        Ok(Self {
            env,
            catalog,
            state,
            registry,
            config,
            installer,
            engine,
            profiles,
        })
    }
    fn save(&self) -> Result<(), Problem> {
        save_json(&self.env.paths.config.join("app.json"), &self.state)?;
        self.registry
            .save(&self.env.paths.config.join("iwads.json"))
            .map_err(|e| logged(Problem::Storage, e))
    }
    fn recipe(&self, id: &str) -> Result<Recipe, Problem> {
        self.state
            .recipes
            .get(id)
            .or_else(|| self.catalog.recipe(id))
            .cloned()
            .ok_or(Problem::State)
    }
    pub fn snapshot(&self) -> Result<Snapshot, Problem> {
        let engines = self
            .engine
            .installed()
            .map_err(|e| logged(Problem::State, e))?;
        let engine = engines.active_entry().cloned();
        let handle = engine.as_ref().map(|e| resolver::Engine {
            executable: e.path.clone(),
            version: e.version.clone(),
        });
        let mut recipes: BTreeMap<String, Recipe> = self
            .catalog
            .recipes
            .iter()
            .map(|r| (r.recipe.id.clone(), r.recipe.clone()))
            .collect();
        recipes.extend(self.state.recipes.clone());
        let held_profiles = self
            .profiles
            .list()
            .map_err(|e| logged(Problem::State, e))?;
        let mut items = vec![];
        for recipe in recipes.into_values() {
            let manifest = self
                .installer
                .load_manifest(&recipe)
                .map_err(install_error)?;
            let profile = held_profiles
                .iter()
                .find(|p| p.recipe_id == recipe.id)
                .cloned()
                .unwrap_or_else(|| {
                    Profile::new(format!("{}-default", recipe.id), "Default", &recipe.id)
                });
            let launch = match &manifest {
                Some(m) => resolver::resolve(
                    &recipe,
                    &profile,
                    &resolver::Iwads::new(&self.env.paths.data, &self.registry),
                    m,
                    handle.as_ref(),
                    &self.config,
                )
                .map(|r| r.summary().clone()),
                None => Err(Blocker::NotInstalled),
            };
            items.push(Item {
                recipe,
                manifest,
                profile,
                launch,
            });
        }
        let config = self.config.state().ok();
        Ok(Snapshot {
            items,
            previous: engines.previous_good,
            attribution: engine
                .as_ref()
                .and_then(|e| self.engine.attribution(&e.version).ok()),
            engine,
            iwads: self.registry.entries().to_vec(),
            folders: self.state.folders.clone(),
            config_notice: config.as_ref().is_none_or(|s| {
                !s.conflicts.is_empty()
                    || s.template_version != self.config.template().template_version
                    || !self.config.base_path(&s.template_version).is_file()
            }),
            config_usable: config.is_some()
                && fs::read_to_string(self.config.active_path()).is_ok(),
            config_version: config.map_or_else(String::new, |s| s.template_version),
            logs: self.env.paths.state.join("logs"),
        })
    }
    pub fn execute(
        &mut self,
        command: Command,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(Event),
    ) -> Result<(), Problem> {
        if cancel.is_cancelled() {
            return Err(Problem::Cancelled);
        }
        emit(Event::Working);
        match command {
            Command::Startup => {
                if let Err(error) = self
                    .config
                    .provision()
                    .and_then(|_| self.config.ensure_ready().map(|_| ()))
                {
                    logged(Problem::Config, error);
                }

                self.scan(None, cancel, emit)?;
            }
            Command::InstallEngine(version) => {
                self.engine
                    .install_reviewed_with_progress(&version, cancel, &mut |p| {
                        emit(Event::Progress(p))
                    })
                    .map_err(|e| {
                        let kind = match &e {
                            dsda::Error::Install(dsda::InstallError::Download(d)) => {
                                download_error(d)
                            }
                            _ => Problem::Engine,
                        };
                        logged(kind, e)
                    })?;
                self.engine
                    .select_engine(&version)
                    .map_err(|e| logged(Problem::Engine, e))?;
            }
            Command::Scan(folder) => self.scan(folder, cancel, emit)?,
            Command::Choose(path, variant) => {
                let entry = self.registry.get(&path).ok_or(Problem::State)?;
                if !entry.candidates.contains(&variant) {
                    return Err(Problem::State);
                }
                self.registry
                    .choose(&path, &variant, &iwads::IwadTable::embedded())
                    .map_err(|e| logged(Problem::State, e))?;
                self.save()?;
            }
            Command::RemoveFolder(path) => {
                self.state.folders.retain(|p| p != &path);
                let retained: Vec<_> = self
                    .registry
                    .entries()
                    .iter()
                    .filter(|e| {
                        e.source == iwads::Source::UserFolder
                            && e.path.starts_with(&path)
                            && !self.state.folders.iter().any(|p| e.path.starts_with(p))
                    })
                    .map(|e| e.path.clone())
                    .collect();
                for p in retained {
                    self.registry.remove(&p);
                }
                self.save()?;
            }
            Command::Install(id) => {
                let recipe = self.recipe(&id)?;
                let p = self.profile(&recipe)?;
                self.profiles
                    .save(&p)
                    .map_err(|e| logged(Problem::Storage, e))?;
                self.installer
                    .install_with_progress(
                        &recipe,
                        &p.components.iter().map(String::as_str).collect::<Vec<_>>(),
                        cancel,
                        &mut |p| emit(Event::Progress(p)),
                    )
                    .map_err(install_error)?;
                self.state.recipes.insert(id, recipe);
                self.save()?;
            }
            Command::Save(profile) => {
                profile
                    .validate(&self.recipe(&profile.recipe_id)?)
                    .map_err(|e| logged(Problem::State, e))?;
                self.profiles
                    .save(&profile)
                    .map_err(|e| logged(Problem::Storage, e))?;
            }
            Command::Play(id) => {
                let recipe = self.recipe(&id)?;
                let mut profile = self.profile(&recipe)?;
                let manifest = self
                    .installer
                    .load_manifest(&recipe)
                    .map_err(install_error)?
                    .ok_or(Problem::State)?;
                let e = self
                    .engine
                    .active_engine()
                    .map_err(|e| logged(Problem::Engine, e))?
                    .ok_or(Problem::Engine)?;
                let engine = resolver::Engine {
                    executable: e.path,
                    version: e.version,
                };
                let guard = self
                    .config
                    .run_guard()
                    .map_err(|e| logged(Problem::Config, e))?;
                let r = resolver::resolve(
                    &recipe,
                    &profile,
                    &resolver::Iwads::new(&self.env.paths.data, &self.registry),
                    &manifest,
                    Some(&engine),
                    &self.config,
                )
                .map_err(|e| logged(Problem::State, e))?;
                r.prepare().map_err(|e| logged(Problem::Storage, e))?;
                r.validate().map_err(|e| logged(Problem::Launch, e))?;
                emit(Event::Playing);
                let runner: Box<dyn EngineRunner> =
                    Box::new(dsda::RealEngineRunner::for_state(&self.env.paths));
                #[cfg(test)]
                let runner: &dyn EngineRunner = match &self.env.test {
                    Some(t) => t.runner.as_ref(),
                    None => runner.as_ref(),
                };
                let outcome = runner
                    .launch(
                        r.plan(),
                        &dsda::AssignAllowlist::with_keys(
                            ddl_core::dsda_config::ALLOWED_KEYS.iter().copied(),
                        ),
                    )
                    .map_err(|e| logged(Problem::Launch, e))?;
                guard.end().map_err(|e| logged(Problem::Config, e))?;
                profile.touch(dsda::Clock::now_unix_secs(&dsda::SystemClock));
                self.profiles
                    .save(&profile)
                    .map_err(|e| logged(Problem::Storage, e))?;
                emit(Event::EngineExit(outcome));
            }
            Command::Browse(path, files) => {
                emit(Event::Directory(self.browse(&path, files, cancel)?))
            }
            Command::Updates => {
                let result = self
                    .engine
                    .discover(true)
                    .map_err(|e| logged(Problem::Update, e))?;
                if result.is_offline() {
                    return Err(Problem::Update);
                }
                if cancel.is_cancelled() {
                    return Err(Problem::Cancelled);
                }
                emit(Event::Updates(
                    result
                        .releases
                        .iter()
                        .filter(|r| matches!(r.availability, dsda::Availability::Reviewed))
                        .map(|r| r.version.clone())
                        .collect(),
                ));
            }
            Command::Rollback => {
                self.engine
                    .rollback()
                    .map_err(|e| logged(Problem::Engine, e))?;
            }
            Command::Reset => {
                self.config
                    .reset(true)
                    .map_err(|e| logged(Problem::Config, e))?;
            }
            Command::Import(id, component, path) => {
                let recipe = self.recipe(&id)?;
                let outcome = self
                    .installer
                    .import_user_component_with_cancel(&recipe, &component, &path, cancel)
                    .map_err(install_error)?;
                if !outcome.matched {
                    tracing::warn!(sha256=%outcome.sha256,component,"unrecognized imported component was stored but not enabled");
                    return Err(Problem::UnknownFile);
                }
                let mut profile = self.profile(&recipe)?;
                select_component(&recipe, &mut profile, &component);
                self.profiles
                    .save(&profile)
                    .map_err(|e| logged(Problem::Storage, e))?;
            }
        }
        Ok(())
    }
    fn profile(&self, recipe: &Recipe) -> Result<Profile, Problem> {
        Ok(self
            .profiles
            .load_optional(&format!("{}-default", recipe.id))
            .map_err(|e| logged(Problem::State, e))?
            .unwrap_or_else(|| {
                Profile::new(format!("{}-default", recipe.id), "Default", &recipe.id)
            }))
    }
    fn scan(
        &mut self,
        folder: Option<PathBuf>,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(Event),
    ) -> Result<(), Problem> {
        if let Some(folder) = folder {
            let path = fs::canonicalize(folder).map_err(|e| logged(Problem::Storage, e))?;
            if !self.state.folders.contains(&path) {
                self.state.folders.push(path);
            }
        }
        let mut paths: Vec<_> = iwads::discover_paths(&self.env.home)
            .into_iter()
            .map(|p| (p, iwads::Source::Steam))
            .collect();
        for folder in &self.state.folders {
            let scan = iwads::scan_folder(folder).map_err(|e| logged(Problem::Storage, e))?;
            paths.extend(
                scan.wads
                    .into_iter()
                    .map(|p| (p, iwads::Source::UserFolder)),
            );
        }
        self.registry.refresh(&iwads::IwadTable::embedded());
        if cancel.is_cancelled() {
            return Err(Problem::Cancelled);
        }
        for (path, source) in paths {
            if cancel.is_cancelled() {
                return Err(Problem::Cancelled);
            }
            emit(Event::Working);
            if self.registry.get(&path).is_some() {
                continue;
            }
            if let Ok(id) = iwads::Identifier::embedded().identify(&path) {
                self.registry.record(&path, &id, source);
            }
        }
        self.save()
    }
    fn browse(
        &self,
        path: &Path,
        files: bool,
        cancel: &CancellationToken,
    ) -> Result<Directory, Problem> {
        let mut entries = vec![];
        for entry in fs::read_dir(path)
            .map_err(|e| logged(Problem::Storage, e))?
            .take(4096)
        {
            if cancel.is_cancelled() {
                return Err(Problem::Cancelled);
            }
            let entry = entry.map_err(|e| logged(Problem::Storage, e))?;
            let p = entry.path();
            let metadata = fs::metadata(&p).map_err(|e| logged(Problem::Storage, e))?;
            if metadata.is_dir()
                || files
                    && metadata.is_file()
                    && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("wad"))
            {
                entries.push((p, metadata.is_dir()));
            }
        }
        entries.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let mut shortcuts = vec![self.env.home.clone()];
        shortcuts.extend(
            iwads::libraries(&self.env.home)
                .into_iter()
                .map(|l| l.common()),
        );
        if let Ok(users) = fs::read_dir("/run/media") {
            for user in users.flatten().take(64) {
                if let Ok(mounts) = fs::read_dir(user.path()) {
                    shortcuts.extend(
                        mounts
                            .flatten()
                            .take(64)
                            .map(|m| m.path())
                            .filter(|p| p.is_dir()),
                    );
                }
            }
        }
        Ok(Directory {
            path: path.to_path_buf(),
            entries,
            shortcuts,
        })
    }
}
pub fn select_component(recipe: &Recipe, profile: &mut Profile, id: &str) {
    let Some(c) = recipe.components.iter().find(|c| c.id == id) else {
        return;
    };
    let enabled = !profile.components.iter().any(|held| held == id);
    if enabled && let Some(group) = &c.exclusive_group {
        profile.components.retain(|id| {
            !recipe
                .components
                .iter()
                .any(|other| other.id == *id && other.exclusive_group.as_ref() == Some(group))
        });
    }
    profile.set_component(id, enabled);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn env(root: &Path) -> Environment {
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let paths = ddl_core::platform::paths::resolve(&|key: &str| {
            (key == "HOME").then(|| home.display().to_string())
        })
        .unwrap();
        Environment {
            paths,
            home,
            test: None,
        }
    }
    #[test]
    fn startup_provisions_owned_paths_and_restores_the_default_profile() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let mut s = Services::open(env.clone()).unwrap();
        s.execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
            .unwrap();
        let snapshot = s.snapshot().unwrap();
        assert_eq!(snapshot.items.len(), 4);
        assert!(snapshot.engine.is_none());
        assert!(snapshot.config_usable);
        assert!(snapshot.iwads.is_empty());
        let mut profile = snapshot
            .items
            .iter()
            .find(|i| i.recipe.id == "sigil")
            .unwrap()
            .profile
            .clone();
        profile.skill = 5;
        profile.pistol_start = true;
        s.execute(
            Command::Save(profile.clone()),
            &CancellationToken::new(),
            &mut |_| {},
        )
        .unwrap();
        let restored = Services::open(env).unwrap().snapshot().unwrap();
        assert_eq!(
            restored
                .items
                .iter()
                .find(|i| i.recipe.id == "sigil")
                .unwrap()
                .profile,
            profile
        );
        assert!(!temp.path().join("home/.dsda-doom").exists());
    }
    #[test]
    fn additive_choices_are_exclusive_and_replacements_remove_the_main() {
        let catalog = Catalog::embedded();
        let r = catalog.recipe("auger-zenith").unwrap();
        let mut profile = Profile::new("test", "Default", &r.id);
        let choices: Vec<_> = r
            .components
            .iter()
            .filter(|c| c.optional && !c.user_supplied)
            .collect();
        select_component(r, &mut profile, &choices[0].id);
        select_component(r, &mut profile, &choices[1].id);
        assert_eq!(profile.components, vec![choices[1].id.clone()]);
        select_component(r, &mut profile, &choices[1].id);
        assert!(profile.components.is_empty());
        let sigil = catalog.recipe("sigil").unwrap();
        let mut profile = Profile::new("test", "Default", &sigil.id);
        select_component(sigil, &mut profile, "compat");
        let active = sigil.active_components(&["compat"]).unwrap();
        assert!(active.iter().all(|c| c.id != "main"));
    }
    #[test]
    fn state_rejects_unknown_schema_and_cancelled_work_does_not_change_user_state() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let mut s = Services::open(env.clone()).unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert_eq!(
            s.execute(Command::Startup, &cancel, &mut |_| {}),
            Err(Problem::Cancelled)
        );
        assert!(!env.paths.config.join("app.json").exists());
        fs::write(
            env.paths.config.join("app.json"),
            br#"{"schema_version":2,"folders":[],"recipes":{}}"#,
        )
        .unwrap();
        assert!(matches!(Services::open(env), Err(Problem::State)));
    }
    #[test]
    fn unreadable_config_is_a_reset_notice_without_losing_the_catalog() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let mut s = Services::open(env.clone()).unwrap();
        s.execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
            .unwrap();
        fs::write(env.paths.config.join("dsda/state.json"), b"broken").unwrap();
        let mut s = Services::open(env).unwrap();
        s.execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
            .unwrap();
        let view = s.snapshot().unwrap();
        assert!(!view.config_usable);
        assert!(view.config_notice);
        assert_eq!(view.items.len(), 4);
    }
}

#[cfg(test)]
struct TestApi(std::sync::Arc<dsda::CannedApi>);
#[cfg(test)]
impl dsda::ReleaseApi for TestApi {
    fn fetch(&self, etag: Option<&str>) -> Result<dsda::ApiResponse, dsda::ApiError> {
        dsda::ReleaseApi::fetch(self.0.as_ref(), etag)
    }
}

#[cfg(test)]
mod steam_wiring_tests {
    use super::*;
    #[test]
    fn startup_discovers_steam_iwads_without_scanning_home_and_browser_offers_the_library() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        let common = home.join(".local/share/Steam/steamapps/common");
        let game = common.join(iwads::steam::KNOWN_INSTALL_DIRS[0]);
        fs::create_dir_all(&game).unwrap();
        let paths = ddl_core::platform::paths::resolve(&|key: &str| {
            (key == "HOME").then(|| home.display().to_string())
        })
        .unwrap();
        let mut bytes = b"IWAD".to_vec();
        bytes.extend_from_slice(&36i32.to_le_bytes());
        bytes.extend_from_slice(&12i32.to_le_bytes());
        for e in 1..=4 {
            for m in 1..=9 {
                bytes.extend_from_slice(&[0; 8]);
                let name = format!("E{e}M{m}");
                let mut lump = [0; 8];
                lump[..name.len()].copy_from_slice(name.as_bytes());
                bytes.extend_from_slice(&lump);
            }
        }
        fs::write(game.join("renamed.wad"), &bytes).unwrap();
        fs::write(home.join("not-authorized.wad"), &bytes).unwrap();
        let mut services = Services::open(Environment {
            paths,
            home: home.clone(),
            test: None,
        })
        .unwrap();
        services
            .execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
            .unwrap();
        let snapshot = services.snapshot().unwrap();
        assert_eq!(snapshot.iwads.len(), 1);
        assert_eq!(snapshot.iwads[0].source, iwads::Source::Steam);
        assert!(snapshot.iwads[0].variant.is_none());
        assert!(!snapshot.iwads[0].candidates.is_empty());
        let browser = services
            .browse(&home, false, &CancellationToken::new())
            .unwrap();
        assert!(browser.shortcuts.contains(&home));
        assert!(browser.shortcuts.contains(&common));
    }
}

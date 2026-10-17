//! Thin application services. Constructed and used exclusively on a worker thread.
use ddl_core::{
    catalog::{Catalog, Game, Recipe},
    downloads::{CancellationToken, HostAllowlist, InstallProgress},
    dsda_config::DsdaConfigManager,
    engines::dsda::{self, EngineManager, EngineRunner},
    iwads::{self, Registry, RegistryEntry},
    library::{InstallManifest, Installer},
    platform::{
        lock::{LockError, StateLock},
        paths::Paths,
    },
    profiles::{Profile, ProfileStore},
    resolver::{self, Blocker, Summary},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Another DDL process is changing the same state (spec 014, R2).
    Busy,
    /// The engine could not start because the host runtime is missing something
    /// (spec 014 follow-up).
    EngineRuntime(dsda::RuntimeError),
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage => f.write_str("could not write or read DDL's user storage"),
            Self::State => f.write_str("could not read DDL's saved state"),
            Self::Network => f.write_str("a network request failed"),
            Self::Update => f.write_str("the update check failed"),
            Self::Checksum => f.write_str("a downloaded file failed its checksum"),
            Self::UnsafeArchive => f.write_str("an archive was refused as unsafe"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::Engine => f.write_str("the engine operation failed"),
            Self::Config => f.write_str("the DSDA configuration operation failed"),
            Self::UnknownFile => f.write_str("the file is not a component DDL recognizes"),
            Self::Launch => f.write_str("the engine could not be launched"),
            Self::Busy => {
                f.write_str("another DDL process is changing DDL's state; close it and try again")
            }
            Self::EngineRuntime(runtime) => write!(f, "{runtime}"),
        }
    }
}

impl std::error::Error for Problem {}

/// A one-line, actionable failure for the headless CLI (spec 014, R1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliFailure {
    /// A typed resolver blocker, printed with its explanation.
    Blocker(Blocker),
    /// Anything else, already phrased for the user.
    Message(String),
}

impl CliFailure {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl fmt::Display for CliFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blocker(blocker) => write!(f, "cannot launch: {blocker}"),
            Self::Message(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for CliFailure {}

impl From<Blocker> for CliFailure {
    fn from(blocker: Blocker) -> Self {
        Self::Blocker(blocker)
    }
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
    /// R4: also probe the generic DOOM folders and `DOOMWADDIR`/`DOOMWADPATH`.
    /// Default off. The field is additive inside schema 1, so an existing state
    /// file without it loads as `false`; spec 015 folds it into its schema 2.
    #[serde(default)]
    search_standard_folders: bool,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            folders: vec![],
            recipes: BTreeMap::new(),
            search_standard_folders: false,
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
    /// R4: whether DDL probes the generic DOOM roots (default off).
    pub search_standard_folders: bool,
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
    /// R4: persist the "Search standard Linux folders" setting.
    SetSearchStandardFolders(bool),
}

impl Command {
    /// Whether this command writes user-visible state and so must hold the
    /// cross-process state lock (spec 014, R2).
    ///
    /// `Play` is deliberately excluded: it holds spec 005's run guard for the
    /// whole engine run, and a two-hour game must not block CLI installs.
    #[must_use]
    fn mutates_state(&self) -> bool {
        !matches!(self, Self::Play(_) | Self::Browse(..) | Self::Updates)
    }
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
        // Spec 021 R4's `search_standard_folders` is additive inside schema 1
        // (absent = `false`); spec 015 owns the 1 -> 2 bump.
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
            search_standard_folders: self.state.search_standard_folders,
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
        // One DDL process mutates its state at a time (spec 014, R2). The lock
        // is released when this command returns (or the process dies).
        let _state_lock = if command.mutates_state() {
            Some(self.lock_state()?)
        } else {
            None
        };
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
                    .install_reviewed_with_progress(&version, false, cancel, &mut |p| {
                        emit(Event::Progress(p))
                    })
                    .map_err(|e| {
                        let kind = match &e {
                            dsda::Error::Install(dsda::InstallError::Download(d)) => {
                                download_error(d)
                            }
                            // A host-runtime failure gets its own, actionable
                            // problem so the UI can name the cause (spec 014
                            // follow-up).
                            dsda::Error::Install(dsda::InstallError::Smoke(smoke)) => {
                                match smoke.as_ref() {
                                    dsda::SmokeError::Runtime(runtime) => {
                                        Problem::EngineRuntime(runtime.clone())
                                    }
                                    _ => Problem::Engine,
                                }
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
            Command::SetSearchStandardFolders(enabled) => {
                self.state.search_standard_folders = enabled;
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
                let guard = self
                    .config
                    .run_guard()
                    .map_err(|e| logged(Problem::Config, e))?;
                let r = self
                    .resolve_launch(&recipe, &profile)
                    .map_err(|blocker| logged(Problem::State, blocker))?;
                emit(Event::Playing);
                let outcome = self.run_resolution(&r)?;
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

    /// Acquires DDL's cross-process state lock for this process (spec 014, R2).
    fn lock_state(&self) -> Result<StateLock, Problem> {
        StateLock::acquire(&self.env.paths.config).map_err(|error| match error {
            LockError::Busy => Problem::Busy,
            LockError::Io(error) => logged(Problem::Storage, error),
        })
    }

    /// Resolves a recipe exactly as Play does, returning spec 009's typed
    /// [`Blocker`] when something is missing. Writes nothing.
    fn resolve_launch(
        &self,
        recipe: &Recipe,
        profile: &Profile,
    ) -> Result<resolver::Resolution, Blocker> {
        let manifest = match self.installer.load_manifest(recipe) {
            Ok(Some(manifest)) => manifest,
            Ok(None) => return Err(Blocker::NotInstalled),
            Err(error) => {
                return Err(Blocker::IntegrityFailed {
                    detail: error.to_string(),
                });
            }
        };
        let engine = self
            .engine
            .active_engine()
            .map_err(|_| Blocker::EngineMissing)?
            .ok_or(Blocker::EngineMissing)?;
        let engine = resolver::Engine {
            executable: engine.path,
            version: engine.version,
        };
        resolver::resolve(
            recipe,
            profile,
            &resolver::Iwads::new(&self.env.paths.data, &self.registry),
            &manifest,
            Some(&engine),
            &self.config,
        )
    }

    /// Prepares, validates and runs an already-resolved launch.
    fn run_resolution(
        &self,
        resolution: &resolver::Resolution,
    ) -> Result<dsda::LaunchOutcome, Problem> {
        resolution
            .prepare()
            .map_err(|e| logged(Problem::Storage, e))?;
        resolution
            .validate()
            .map_err(|e| logged(Problem::Launch, e))?;
        let runner: Box<dyn EngineRunner> =
            Box::new(dsda::RealEngineRunner::for_state(&self.env.paths));
        #[cfg(test)]
        let runner: &dyn EngineRunner = match &self.env.test {
            Some(t) => t.runner.as_ref(),
            None => runner.as_ref(),
        };
        runner
            .launch(
                resolution.plan(),
                &dsda::AssignAllowlist::with_keys(
                    ddl_core::dsda_config::ALLOWED_KEYS.iter().copied(),
                ),
            )
            .map_err(|e| logged(Problem::Launch, e))
    }
    fn scan(
        &mut self,
        folder: Option<PathBuf>,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(Event),
    ) -> Result<(), Problem> {
        let folders: Vec<PathBuf> = folder.into_iter().collect();
        self.scan_impl(&folders, cancel, emit).map(|_| ())
    }

    /// The shared scan behind the UI's `Scan` command and `ddl iwad scan`.
    ///
    /// Adds every folder to DDL's folders, discovers Steam and user-folder
    /// candidates, refreshes the registry and identifies new files. The report
    /// lists every candidate's current registry entry (identified or ambiguous)
    /// plus the files that are not recognised base games.
    fn scan_impl(
        &mut self,
        folders: &[PathBuf],
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(Event),
    ) -> Result<ScanReport, Problem> {
        for folder in folders {
            let path = fs::canonicalize(folder).map_err(|e| logged(Problem::Storage, e))?;
            if !self.state.folders.contains(&path) {
                self.state.folders.push(path);
            }
        }
        let mut paths: Vec<_> = iwads::discover_paths(&self.env.home)
            .into_iter()
            .map(|p| (p, iwads::Source::Steam))
            .collect();
        // GOG installs recorded by Heroic / minigalaxy plus the documented
        // GOG default folders (spec 021, R2). Garbled records only log.
        let stores = iwads::stores::discover(&self.env.home);
        for unreadable in &stores.unreadable {
            tracing::warn!(
                path = %unreadable.display(),
                "ignoring a garbled store launcher record"
            );
        }
        paths.extend(
            stores
                .candidates
                .into_iter()
                .map(|candidate| (candidate.path, store_source(candidate.launcher))),
        );
        // The generic roots are opt-in and never scanned by default (spec 021,
        // R4).
        if self.state.search_standard_folders {
            paths.extend(
                iwads::standard_paths(&self.env.home, &|key| std::env::var(key).ok())
                    .into_iter()
                    .map(|p| (p, iwads::Source::StandardFolder)),
            );
        }
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
        let mut unrecognized = Vec::new();
        for (path, source) in &paths {
            if cancel.is_cancelled() {
                return Err(Problem::Cancelled);
            }
            emit(Event::Working);
            if self.registry.get(path).is_some() {
                continue;
            }
            match iwads::Identifier::embedded().identify(path) {
                Ok(id) if self.registry.record(path, &id, *source) => {}
                _ => unrecognized.push(path.clone()),
            }
        }
        self.save()?;
        let entries = paths
            .iter()
            .filter_map(|(path, _)| self.registry.get(path).cloned())
            .collect();
        Ok(ScanReport {
            entries,
            unrecognized,
        })
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
        entries.sort_by(|a, b| {
            hidden_path(&a.0)
                .cmp(&hidden_path(&b.0))
                .then(b.1.cmp(&a.1))
                .then_with(|| {
                    a.0.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase()
                        .cmp(
                            &b.0.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_lowercase(),
                        )
                })
                .then(a.0.cmp(&b.0))
        });
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
/// One installed engine, for `ddl engine status` (spec 014, R1).
#[derive(Debug, Clone)]
pub struct EngineStatus {
    /// Every installed engine, in version order.
    pub installed: Vec<dsda::InstalledEngine>,
    /// The active version, if one was selected.
    pub active: Option<String>,
    /// The rollback target, if one is recorded.
    pub previous: Option<String>,
}

/// One catalog recipe, for `ddl mapset list` and `ddl doctor` (spec 014, R1).
#[derive(Debug, Clone)]
pub struct MapsetRow {
    /// Recipe id.
    pub id: String,
    /// Recipe title.
    pub title: String,
    /// Required base game.
    pub game: Game,
    /// Whether an install record exists.
    pub installed: bool,
}

/// The result of `ddl iwad scan` (spec 014, R1).
#[derive(Debug, Clone)]
pub struct ScanReport {
    /// The current registry entry for every candidate file (identified or ambiguous).
    pub entries: Vec<RegistryEntry>,
    /// Candidate files that are not recognised base games.
    pub unrecognized: Vec<PathBuf>,
}

/// Per-launch overrides layered on the saved profile, as `ddl launch` accepts
/// (spec 014, R1).
#[derive(Debug, Clone, Default)]
pub struct LaunchOverrides {
    /// Optional component ids to enable (`--with`).
    pub components: Vec<String>,
    /// Starting skill override (`--skill`).
    pub skill: Option<u8>,
    /// Reset to a pistol start on every map (`--pistol-start`).
    pub pistol_start: bool,
}

/// The data behind `ddl doctor` (spec 014, R1).
#[derive(Debug, Clone)]
pub struct DoctorReport {
    /// DDL's resolved XDG directories.
    pub paths: Paths,
    /// Engine state.
    pub engine: EngineStatus,
    /// The IWAD registry.
    pub iwads: Vec<RegistryEntry>,
    /// Every catalog recipe and its install state.
    pub mapsets: Vec<MapsetRow>,
    /// Whether the active DSDA config file exists and is readable.
    pub config_usable: bool,
    /// The active DSDA config file.
    pub config_active: PathBuf,
    /// The applied DSDA template version.
    pub config_template: String,
    /// Unresolved DSDA config conflicts.
    pub config_conflicts: usize,
    /// The log directory shown in Settings → Diagnostics.
    pub logs: PathBuf,
    /// The installed-engine state file.
    pub engine_state: PathBuf,
}

/// Headless subcommands (spec 014, R1). These reuse the same managers the UI's
/// services use, so the CLI and the UI share one state.
impl Services {
    /// Acquires the state lock for a CLI mutation, or explains why it cannot.
    fn lock_cli(&self) -> Result<StateLock, CliFailure> {
        StateLock::acquire(&self.env.paths.config)
            .map_err(|error| CliFailure::message(error.to_string()))
    }

    /// `ddl engine status`: installed versions, the active one and its mode.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] when the installed-engine state is unreadable.
    pub fn engine_status(&self) -> Result<EngineStatus, CliFailure> {
        let state = self
            .engine
            .installed()
            .map_err(|error| CliFailure::message(error.to_string()))?;
        let mut installed = state.entries().to_vec();
        installed.sort_by(|a, b| a.version.cmp(&b.version));
        Ok(EngineStatus {
            installed,
            active: state.active.clone(),
            previous: state.previous_good.clone(),
        })
    }

    /// `ddl engine install`: the reviewed install path, optionally forcing the
    /// extraction fallback, then selecting the installed version.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown version, a download/verification
    /// failure, a busy state lock, or a failed select.
    pub fn install_engine(
        &self,
        version: Option<&str>,
        force_extract: bool,
        progress: &mut dyn FnMut(InstallProgress),
    ) -> Result<dsda::InstalledEngine, CliFailure> {
        let _lock = self.lock_cli()?;
        let version = version.map_or_else(
            || self.engine.reviewed().newest().version().to_owned(),
            str::to_owned,
        );
        let engine = self
            .engine
            .install_reviewed_with_progress(
                &version,
                force_extract,
                &CancellationToken::new(),
                progress,
            )
            .map_err(|error| CliFailure::message(error.to_string()))?;
        self.engine
            .select_engine(&engine.version)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        Ok(engine)
    }

    /// `ddl iwad scan`: Steam discovery plus the given folders.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] when the state lock is busy or the scan fails.
    pub fn scan_iwads(&mut self, folders: &[PathBuf]) -> Result<ScanReport, CliFailure> {
        let _lock = self.lock_cli()?;
        self.scan_impl(folders, &CancellationToken::new(), &mut |_| {})
            .map_err(|problem| CliFailure::message(problem.to_string()))
    }

    /// `ddl mapset list`: every catalog recipe and its install state.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] when an install record cannot be read.
    pub fn mapset_list(&self) -> Result<Vec<MapsetRow>, CliFailure> {
        let mut rows = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        let recipes = self
            .catalog
            .recipes
            .iter()
            .map(|loaded| loaded.recipe.clone())
            .chain(self.state.recipes.values().cloned());
        for recipe in recipes {
            if !seen.insert(recipe.id.clone()) {
                continue;
            }
            let installed = self
                .installer
                .load_manifest(&recipe)
                .map_err(|error| CliFailure::message(error.to_string()))?
                .is_some();
            rows.push(MapsetRow {
                id: recipe.id.clone(),
                title: recipe.title.clone(),
                game: recipe.base_game.game,
                installed,
            });
        }
        Ok(rows)
    }

    /// `ddl mapset install`: install a recipe, optionally enabling components.
    ///
    /// With `--with` the profile's selection becomes exactly the requested
    /// components (honouring exclusive groups); without it the saved selection
    /// is kept. The profile is saved before the install, as the UI does, and the
    /// install record is what a later UI start reads.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown recipe, an unknown or
    /// user-supplied component, an invalid combination, a busy state lock or an
    /// install failure.
    pub fn install_mapset(
        &mut self,
        id: &str,
        components: &[String],
        progress: &mut dyn FnMut(InstallProgress),
    ) -> Result<InstallManifest, CliFailure> {
        let _lock = self.lock_cli()?;
        let recipe = self
            .recipe(id)
            .map_err(|_| CliFailure::message(format!("no catalog recipe with id `{id}`")))?;
        let mut profile = self
            .profile(&recipe)
            .map_err(|problem| CliFailure::message(problem.to_string()))?;
        if !components.is_empty() {
            profile.components.clear();
            for component in components {
                let Some(held) = recipe
                    .components
                    .iter()
                    .find(|candidate| &candidate.id == component)
                else {
                    return Err(CliFailure::message(format!(
                        "`{id}` has no component `{component}`"
                    )));
                };
                if !held.optional {
                    return Err(CliFailure::message(format!(
                        "component `{component}` is required and does not need `--with`"
                    )));
                }
                if held.user_supplied {
                    return Err(CliFailure::message(format!(
                        "component `{component}` is user-supplied; import it from the UI, DDL never downloads it"
                    )));
                }
                if let Some(group) = &held.exclusive_group {
                    profile.components.retain(|enabled| {
                        !recipe.components.iter().any(|other| {
                            &other.id == enabled && other.exclusive_group.as_ref() == Some(group)
                        })
                    });
                }
                profile.set_component(component, true);
            }
        }
        profile.validate(&recipe).map_err(|error| {
            CliFailure::message(format!("invalid component selection: {error}"))
        })?;
        self.profiles
            .save(&profile)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        let enabled: Vec<&str> = profile.components.iter().map(String::as_str).collect();
        let manifest = self
            .installer
            .install_with_progress(&recipe, &enabled, &CancellationToken::new(), progress)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        self.state.recipes.insert(id.to_owned(), recipe);
        self.save()
            .map_err(|problem| CliFailure::message(problem.to_string()))?;
        Ok(manifest)
    }

    /// The profile a CLI launch resolves, with the command-line overrides
    /// applied but not saved (spec 014, R1). Writes nothing.
    fn effective_profile(
        &self,
        recipe: &Recipe,
        overrides: &LaunchOverrides,
    ) -> Result<Profile, CliFailure> {
        let mut profile = self
            .profile(recipe)
            .map_err(|problem| CliFailure::message(problem.to_string()))?;
        if !overrides.components.is_empty() {
            profile.components.clear();
            for component in &overrides.components {
                let Some(held) = recipe
                    .components
                    .iter()
                    .find(|candidate| &candidate.id == component && candidate.optional)
                else {
                    return Err(CliFailure::message(format!(
                        "`{}` has no optional component `{component}`",
                        recipe.id
                    )));
                };
                // A user_supplied component is allowed here: it may have been
                // imported already, and the resolver reports it if not.
                if let Some(group) = &held.exclusive_group {
                    profile.components.retain(|enabled| {
                        !recipe.components.iter().any(|other| {
                            &other.id == enabled && other.exclusive_group.as_ref() == Some(group)
                        })
                    });
                }
                profile.set_component(component, true);
            }
        }
        if let Some(skill) = overrides.skill {
            profile.skill = skill;
        }
        if overrides.pistol_start {
            profile.pistol_start = true;
        }
        profile
            .validate(recipe)
            .map_err(|error| CliFailure::message(format!("invalid launch selection: {error}")))?;
        Ok(profile)
    }

    /// `ddl launch --dry-run`: resolve exactly as Play does and return the
    /// resolution, or the typed blocker. Writes nothing.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`]: [`CliFailure::Blocker`] for a typed resolver
    /// blocker, or [`CliFailure::Message`] for an unknown recipe or an invalid
    /// selection.
    pub fn resolve_mapset(
        &self,
        id: &str,
        overrides: &LaunchOverrides,
    ) -> Result<resolver::Resolution, CliFailure> {
        let recipe = self
            .recipe(id)
            .map_err(|_| CliFailure::message(format!("no catalog recipe with id `{id}`")))?;
        let profile = self.effective_profile(&recipe, overrides)?;
        self.resolve_launch(&recipe, &profile)
            .map_err(CliFailure::from)
    }

    /// `ddl launch` without `--dry-run`: launch through the same run guard as
    /// the UI. The override selection is not saved.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for a typed blocker, a busy run guard, or a
    /// launch failure.
    pub fn run_mapset(
        &self,
        id: &str,
        overrides: &LaunchOverrides,
    ) -> Result<dsda::LaunchOutcome, CliFailure> {
        let recipe = self
            .recipe(id)
            .map_err(|_| CliFailure::message(format!("no catalog recipe with id `{id}`")))?;
        let profile = self.effective_profile(&recipe, overrides)?;
        let guard = self
            .config
            .run_guard()
            .map_err(|error| CliFailure::message(error.to_string()))?;
        let resolution = self
            .resolve_launch(&recipe, &profile)
            .map_err(CliFailure::from)?;
        let outcome = self
            .run_resolution(&resolution)
            .map_err(|problem| CliFailure::message(problem.to_string()))?;
        guard
            .end()
            .map_err(|error| CliFailure::message(error.to_string()))?;
        Ok(outcome)
    }

    /// `ddl doctor`: one paste-friendly diagnostic report.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] when the snapshot or engine state cannot be read.
    pub fn doctor(&self) -> Result<DoctorReport, CliFailure> {
        let snapshot = self
            .snapshot()
            .map_err(|problem| CliFailure::message(problem.to_string()))?;
        let engine = self.engine_status()?;
        let config = self.config.state().ok();
        Ok(DoctorReport {
            paths: self.env.paths.clone(),
            engine,
            iwads: snapshot.iwads,
            mapsets: self.mapset_list()?,
            config_usable: snapshot.config_usable,
            config_active: self.config.active_path(),
            config_template: snapshot.config_version,
            config_conflicts: config.as_ref().map_or(0, |state| state.conflicts.len()),
            logs: snapshot.logs,
            engine_state: self.engine.state_path(),
        })
    }

    /// Test-only seam (spec 014, R3): register a fake IWAD for every variant the
    /// catalog needs, so the Linux e2e workflow can assert the dry-run argv.
    ///
    /// Compiled only for tests or with the `test-support` feature, which release
    /// builds never enable, and only invoked when `DDL_TEST_IWAD_SEAM` names a
    /// directory. It never downloads anything: it writes a sparse file of the
    /// variant's exact size and records it, which is all the dry-run resolver
    /// reads.
    #[cfg(any(test, feature = "test-support"))]
    pub fn register_fake_iwads(&mut self, dir: &Path) -> Result<Vec<PathBuf>, Problem> {
        let _lock = self.lock_state()?;
        let table = iwads::IwadTable::embedded();
        let mut wanted: Vec<String> = Vec::new();
        let recipes = self
            .catalog
            .recipes
            .iter()
            .map(|loaded| &loaded.recipe)
            .chain(self.state.recipes.values());
        for recipe in recipes {
            if recipe.base_game.variants.is_empty() {
                if let Some(variant) = table
                    .variants_for(iwad_game(recipe.base_game.game))
                    .find(|variant| variant.playable)
                {
                    wanted.push(variant.id.clone());
                }
            } else {
                wanted.extend(recipe.base_game.variants.iter().cloned());
            }
        }
        wanted.sort();
        wanted.dedup();
        fs::create_dir_all(dir).map_err(|error| logged(Problem::Storage, error))?;
        let mut written = Vec::new();
        for id in wanted {
            let Some(variant) = table.get(&id) else {
                continue;
            };
            let path = dir.join(format!("{id}.wad"));
            written.push(path.clone());
            if self.registry.get(&path).is_some() {
                continue;
            }
            let file = fs::File::create(&path).map_err(|error| logged(Problem::Storage, error))?;
            file.set_len(variant.size)
                .map_err(|error| logged(Problem::Storage, error))?;
            drop(file);
            let metadata = fs::metadata(&path).map_err(|error| logged(Problem::Storage, error))?;
            let mtime = iwads::FileTime::from_metadata(&metadata).ok_or(Problem::Storage)?;
            self.registry.insert(RegistryEntry {
                path,
                variant: Some(variant.id.clone()),
                candidates: Vec::new(),
                game: Some(variant.game),
                sha256: variant.sha256.clone().unwrap_or_else(|| "0".repeat(64)),
                size: variant.size,
                mtime,
                source: iwads::Source::Manual,
            });
        }
        self.save()?;
        Ok(written)
    }
}

/// The registry source for a GOG candidate's launcher (spec 021, R2).
fn store_source(launcher: iwads::GogLauncher) -> iwads::Source {
    match launcher {
        iwads::GogLauncher::Heroic => iwads::Source::GogHeroic,
        iwads::GogLauncher::Minigalaxy => iwads::Source::GogMinigalaxy,
        iwads::GogLauncher::Offline => iwads::Source::GogOffline,
    }
}

/// The embedded IWAD game a recipe's base game maps to (spec 007).
#[cfg(any(test, feature = "test-support"))]
fn iwad_game(game: Game) -> iwads::Game {
    match game {
        Game::Doom => iwads::Game::Doom,
        Game::Doom2 => iwads::Game::Doom2,
        Game::Tnt => iwads::Game::Tnt,
        Game::Plutonia => iwads::Game::Plutonia,
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
    fn browser_sorts_without_case_and_places_hidden_entries_last() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let services = Services::open(env.clone()).unwrap();
        let dir = env.home.join("browse");
        fs::create_dir_all(&dir).unwrap();
        for name in ["Zebra", "alpha", ".Hidden", "Beta", ".alpha"] {
            fs::create_dir(dir.join(name)).unwrap();
        }
        fs::write(dir.join("a.wad"), b"fixture").unwrap();
        let directory = services
            .browse(&dir, true, &CancellationToken::new())
            .unwrap();
        let names: Vec<_> = directory
            .entries
            .iter()
            .map(|(p, _)| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            names,
            ["alpha", "Beta", "Zebra", "a.wad", ".alpha", ".Hidden"]
        );
    }
    #[test]
    fn startup_provisions_owned_paths_and_restores_the_default_profile() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let mut s = Services::open(env.clone()).unwrap();
        s.execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
            .unwrap();
        let snapshot = s.snapshot().unwrap();
        assert_eq!(snapshot.items.len(), 13);
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
    fn explicit_exclusive_choices_are_generic_idempotent_and_clearable() {
        let mut recipe = Catalog::embedded().recipe("auger-zenith").unwrap().clone();
        recipe.id = "another-mapset".into();
        for (n, c) in recipe.components.iter_mut().enumerate().skip(1) {
            c.id = format!("choice-{n}");
            c.exclusive_group = Some("another-group".into());
        }
        let mut profile = Profile::new("test", "Default", &recipe.id);
        choose_exclusive_component(&recipe, &mut profile, "another-group", Some("choice-1"));
        choose_exclusive_component(&recipe, &mut profile, "another-group", Some("choice-1"));
        assert_eq!(profile.components, ["choice-1"]);
        choose_exclusive_component(&recipe, &mut profile, "another-group", Some("choice-2"));
        assert_eq!(profile.components, ["choice-2"]);
        choose_exclusive_component(&recipe, &mut profile, "another-group", Some("not-a-member"));
        assert_eq!(profile.components, ["choice-2"]);
        choose_exclusive_component(&recipe, &mut profile, "another-group", None);
        assert!(profile.components.is_empty());
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
            br#"{"schema_version":3,"folders":[],"recipes":{}}"#,
        )
        .unwrap();
        assert!(matches!(Services::open(env), Err(Problem::State)));
    }

    #[test]
    fn an_existing_state_file_without_the_setting_loads_off_and_round_trips() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        fs::create_dir_all(&env.paths.config).unwrap();
        // A state file written before R4 has no `search_standard_folders`.
        fs::write(
            env.paths.config.join("app.json"),
            br#"{"schema_version":1,"folders":[],"recipes":{}}"#,
        )
        .unwrap();
        let mut s = Services::open(env.clone()).unwrap();
        assert!(
            !s.snapshot().unwrap().search_standard_folders,
            "the new setting defaults off"
        );
        s.execute(
            Command::SetSearchStandardFolders(true),
            &CancellationToken::new(),
            &mut |_| {},
        )
        .unwrap();
        assert!(
            Services::open(env.clone())
                .unwrap()
                .snapshot()
                .unwrap()
                .search_standard_folders,
            "the setting round-trips"
        );
        let raw = fs::read_to_string(env.paths.config.join("app.json")).unwrap();
        assert!(raw.contains("\"schema_version\": 1"), "{raw}");
        assert!(raw.contains("\"search_standard_folders\": true"), "{raw}");
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
        assert_eq!(view.items.len(), 13);
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

#[cfg(test)]
mod store_wiring_tests {
    use super::*;

    /// A synthetic Ultimate DOOM (`E1M1`–`E4M9`), which identifies as the
    /// ambiguous `doomu` family without any copyrighted data.
    fn ultimate_wad() -> Vec<u8> {
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
        bytes
    }

    fn home_env(temp: &tempfile::TempDir) -> Environment {
        let home = temp.path().join("home");
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
    fn gog_install_via_heroic_is_found_and_labelled() {
        let temp = tempfile::tempdir().unwrap();
        let env = home_env(&temp);
        let install = env.home.join("Games/Heroic/DOOM + DOOM II");
        fs::create_dir_all(install.join("base")).unwrap();
        fs::write(install.join("base/doom.wad"), ultimate_wad()).unwrap();
        let record = env.home.join(".config/heroic/gog_store/installed.json");
        fs::create_dir_all(record.parent().unwrap()).unwrap();
        fs::write(
            &record,
            format!(
                "{{\"installed\":[{{\"appName\":\"1\",\"install_path\":\"{}\"}}]}}",
                install.display()
            ),
        )
        .unwrap();

        let mut services = Services::open(env).unwrap();
        services
            .execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
            .unwrap();
        let snapshot = services.snapshot().unwrap();
        let entry = snapshot
            .iwads
            .iter()
            .find(|e| e.path.ends_with("base/doom.wad"))
            .expect("the Heroic GOG install is discovered");
        assert_eq!(entry.source, iwads::Source::GogHeroic);
        assert!(entry.variant.is_none(), "lump-only match stays ambiguous");
        assert!(entry.candidates.iter().any(|id| id == "doom-1.9ud"));
    }

    #[test]
    fn garbled_store_records_do_not_fail_or_add_candidates() {
        let temp = tempfile::tempdir().unwrap();
        let env = home_env(&temp);
        let record = env.home.join(".config/heroic/gog_store/installed.json");
        fs::create_dir_all(record.parent().unwrap()).unwrap();
        fs::write(&record, "{ truncated").unwrap();

        let mut services = Services::open(env).unwrap();
        services
            .execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
            .unwrap();
        assert!(services.snapshot().unwrap().iwads.is_empty());
    }

    #[test]
    fn standard_folders_stay_off_until_the_setting_is_enabled() {
        let temp = tempfile::tempdir().unwrap();
        let env = home_env(&temp);
        let local = env.home.join(".local/share/games/doom");
        fs::create_dir_all(&local).unwrap();
        fs::write(local.join("doom.wad"), ultimate_wad()).unwrap();

        let mut services = Services::open(env).unwrap();
        services
            .execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
            .unwrap();
        assert!(
            services.snapshot().unwrap().iwads.is_empty(),
            "the generic roots are off by default"
        );

        services
            .execute(
                Command::SetSearchStandardFolders(true),
                &CancellationToken::new(),
                &mut |_| {},
            )
            .unwrap();
        services
            .execute(Command::Scan(None), &CancellationToken::new(), &mut |_| {})
            .unwrap();
        let snapshot = services.snapshot().unwrap();
        let entry = snapshot
            .iwads
            .iter()
            .find(|e| e.path == local.join("doom.wad"))
            .expect("the opted-in standard root is scanned");
        assert_eq!(entry.source, iwads::Source::StandardFolder);
    }
}

/// Dot-prefixed names are presented after visible entries, retaining controller access.
pub(crate) fn hidden_path(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

/// Explicit None / one-member selection, independent of recipe or component names.
pub fn choose_exclusive_component(
    recipe: &Recipe,
    profile: &mut Profile,
    group: &str,
    choice: Option<&str>,
) {
    if choice.is_some_and(|id| {
        !recipe
            .components
            .iter()
            .any(|c| c.id == id && c.exclusive_group.as_deref() == Some(group))
    }) {
        return;
    }
    profile.components.retain(|id| {
        !recipe
            .components
            .iter()
            .any(|c| c.id == *id && c.exclusive_group.as_deref() == Some(group))
    });
    if let Some(id) = choice {
        profile.set_component(id, true);
    }
}

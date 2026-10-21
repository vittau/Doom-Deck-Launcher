//! Thin application services. Constructed and used exclusively on a worker thread.
#[cfg(test)]
#[path = "flatpak_tests.rs"]
pub(crate) mod flatpak_tests;
#[path = "state_recovery.rs"]
mod state_recovery;
pub use state_recovery::{UnreadableFile, start_fresh};

use ddl_core::{
    catalog::{Catalog, Game, Recipe, TagVocabulary},
    downloads::{CancellationToken, HostAllowlist, InstallProgress},
    dsda_config::DsdaConfigManager,
    engines::dsda::{self, EngineManager, EngineRunner},
    iwads::{self, Registry, RegistryEntry},
    library::{
        ActiveInstall, InstallManifest, Installer, RecipeStatus, classify, recipe_content_sha256,
        reconcile_profile,
    },
    local::{
        self, Compatibility, CustomIwad, CustomSetup, Group as ExtraGroup, SetupStore, UserFile,
    },
    platform::{
        legacy_import::{self, LegacyInstall},
        lock::{LockError, StateLock},
        paths::Paths,
        sandbox::Sandbox,
    },
    profiles::{Profile, ProfileStore},
    resolver::{self, Blocker, Summary},
    soundfonts::{self, Choice, Soundfonts},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    Storage,
    State,
    Unreadable(Vec<UnreadableFile>),
    Network,
    /// Spec 024: the transfer went quiet past the idle timeout.
    Stalled,
    Update,
    /// Spec 019: the catalog update check failed.
    Catalog,
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
    /// Spec 067 R5: the explicit Audio choice could not be applied to the DSDA
    /// config, so the launch must not silently use a different font/backend.
    Audio,
    Local(String),
    LocalFile(PathBuf, bool),
    /// Spec 059 R4: the host has this path but the Flatpak cannot open it.
    NotVisible(PathBuf),
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local(message) => f.write_str(message),
            Self::LocalFile(path, _) => write!(f, "{}", path.display()),
            Self::NotVisible(path) => write!(
                f,
                "{} is not visible to the Flatpak — see the guide",
                path.display()
            ),
            Self::Storage => f.write_str("could not write or read DDL's user storage"),
            Self::Unreadable(files) => write!(
                f,
                "could not read {}",
                files
                    .iter()
                    .map(|f| format!("{}: {}", f.path.display(), f.reason))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            Self::State => f.write_str("could not read DDL's saved state"),
            Self::Network => f.write_str("a network request failed"),
            Self::Stalled => f.write_str(
                "the download stopped receiving data; check your connection and try again",
            ),
            Self::Update => f.write_str("the update check failed"),
            Self::Catalog => f.write_str("the catalog update check failed"),
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
            Self::Audio => f.write_str(
                "the MIDI soundfont setting could not be applied; check Settings → Audio and try again",
            ),
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
    /// A failure that asks the binary for a specific, documented non-zero exit
    /// status (spec 050 follow-up F3: the `/idgames` crawl stop shares a code
    /// with no other failure).
    Exit {
        /// The process exit status.
        code: u8,
        /// The message printed as `ddl: <message>`.
        message: String,
    },
}

impl CliFailure {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    /// A failure with an explicit process exit status.
    pub fn with_exit_code(code: u8, message: impl Into<String>) -> Self {
        Self::Exit {
            code,
            message: message.into(),
        }
    }

    /// The process exit status this failure asks for (1 unless it says
    /// otherwise).
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Exit { code, .. } => *code,
            Self::Blocker(_) | Self::Message(_) => 1,
        }
    }
}

impl fmt::Display for CliFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blocker(blocker) => write!(f, "cannot launch: {blocker}"),
            Self::Message(message) => f.write_str(message),
            Self::Exit { message, .. } => f.write_str(message),
        }
    }
}

impl std::error::Error for CliFailure {}

impl From<Blocker> for CliFailure {
    fn from(blocker: Blocker) -> Self {
        Self::Blocker(blocker)
    }
}

/// Spec 059 R5: copies a tarball-era install into the Flatpak's directories
/// under the normal state lock, then remembers that the question is settled.
pub fn import_legacy(
    env: &Environment,
    legacy: &LegacyInstall,
    cancel: &CancellationToken,
) -> Result<legacy_import::Report, Problem> {
    let _lock = StateLock::acquire(&env.paths.config).map_err(|e| logged(Problem::Busy, e))?;
    let report =
        legacy_import::import(legacy, &env.paths, cancel).map_err(|error| match error {
            legacy_import::Error::Cancelled => Problem::Cancelled,
            error => logged(Problem::Storage, error),
        })?;
    tracing::info!(%report, "imported the tarball-era setup");
    if let Err(error) =
        legacy_import::record_decision(&env.paths, legacy_import::Decision::Imported)
    {
        // The copied state now counts as the sandbox's own, which already
        // stops the offer; a lost marker costs nothing.
        tracing::warn!(%error, "could not record the import");
    }
    Ok(report)
}

/// Spec 059 R5: remembers a declined import so DDL never asks again.
pub fn decline_legacy(env: &Environment) {
    if let Err(error) =
        legacy_import::record_decision(&env.paths, legacy_import::Decision::Declined)
    {
        tracing::warn!(%error, "could not remember the declined import");
    }
}

/// The sandbox of this process. Unit tests never read the real filesystem
/// marker, so a test run inside a Flatpak behaves like one outside.
pub(crate) fn default_sandbox() -> Sandbox {
    #[cfg(test)]
    {
        Sandbox::none()
    }
    #[cfg(not(test))]
    {
        Sandbox::detect_host()
    }
}

/// Keep file identity data structured through the UI boundary.
fn identity_issue(issue: local::IdentityError) -> (PathBuf, bool) {
    match issue {
        local::IdentityError::Missing { path } | local::IdentityError::Unreadable { path, .. } => {
            (path, false)
        }
        local::IdentityError::Changed { path } => (path, true),
    }
}

/// Why a launch could not enter the guarded engine run (spec 067, R5).
#[derive(Debug)]
enum RunStartError {
    /// The explicit Audio choice could not be applied.
    Audio(Problem),
    /// Another engine run owns the guard.
    Guard(ddl_core::dsda_config::GuardError),
}

impl std::fmt::Display for RunStartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Audio(problem) => write!(f, "{problem}"),
            Self::Guard(error) => write!(f, "{error}"),
        }
    }
}

impl RunStartError {
    fn into_problem(self) -> Problem {
        match self {
            Self::Audio(problem) => problem,
            Self::Guard(error) => logged(Problem::Config, error),
        }
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
        E::Stalled { .. } => Problem::Stalled,
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
/// Spec 067 R2: map the soundfont installer's failure classes onto the existing
/// actionable problem text (cancel, network, checksum, unsafe archive, storage).
fn soundfont_problem(kind: soundfonts::Kind) -> Problem {
    match kind {
        soundfonts::Kind::Cancelled => Problem::Cancelled,
        soundfonts::Kind::Network => Problem::Network,
        soundfonts::Kind::Checksum => Problem::Checksum,
        soundfonts::Kind::UnsafeArchive => Problem::UnsafeArchive,
        soundfonts::Kind::Storage => Problem::Storage,
        soundfonts::Kind::State => Problem::State,
    }
}

/// The wall clock, in UTC Unix seconds (spec 019 R5 throttle).
fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

/// Makes a user-supplied path absolute, resolving a relative one against the
/// process's current directory (spec 026: user files are referenced in place by
/// absolute path).
fn absolute_path(path: &Path) -> Result<PathBuf, CliFailure> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    std::path::absolute(path).map_err(|error| {
        CliFailure::message(format!(
            "could not resolve `{}` to an absolute path: {error}",
            path.display()
        ))
    })
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppState {
    schema_version: u32,
    folders: Vec<PathBuf>,
    #[serde(deserialize_with = "ddl_core::catalog::persisted::recipes")]
    recipes: BTreeMap<String, Recipe>,
    /// Recipe ids the player marked (spec 015, R5); added by schema 2.
    #[serde(default)]
    favorites: BTreeSet<String>,
    /// Spec 018: additive schema-2 selection; old files default to no selection.
    #[serde(default)]
    last_profiles: BTreeMap<String, String>,
    /// Spec 020: dismissal is scoped to one canonical recipe identity.
    #[serde(default)]
    kept_recipes: BTreeMap<String, String>,
    /// Spec 021 R4: also probe the generic DOOM folders and
    /// `DOOMWADDIR`/`DOOMWADPATH`. Default off. Added while the file was still
    /// schema 1 (absent loads as `false`); schema 2 carries it unchanged.
    #[serde(default)]
    search_standard_folders: bool,
    /// Spec 019 R5: check for catalog updates automatically. Additive, default
    /// on, so an older app-state file keeps working and gains the default.
    #[serde(default = "default_true")]
    catalog_auto_update: bool,
    /// Spec 019 R5: UTC Unix seconds of the last catalog check attempt (success
    /// or failure), so a failed check still throttles to once per 24 h.
    #[serde(default)]
    catalog_last_check: Option<u64>,
}
/// Current `app.json` schema. 1 → 2 added `favorites` (spec 015) and kept spec
/// 021's additive `search_standard_folders`; a schema 1 file is migrated in
/// memory on load and written as schema 2 by the next save.
const STATE_SCHEMA: u32 = 2;
/// Additive booleans default on (spec 019 R5: automatic catalog updates).
fn default_true() -> bool {
    true
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            schema_version: STATE_SCHEMA,
            folders: vec![],
            recipes: BTreeMap::new(),
            favorites: BTreeSet::new(),
            last_profiles: BTreeMap::new(),
            kept_recipes: BTreeMap::new(),
            search_standard_folders: false,
            catalog_auto_update: true,
            catalog_last_check: None,
        }
    }
}
impl AppState {
    /// Reads `app.json` bytes of any supported schema; newer or unknown
    /// schemas are rejected rather than guessed at.
    fn parse(bytes: &[u8]) -> Result<Self, Problem> {
        Self::decode(bytes).map_err(|e| logged(Problem::State, e))
    }
    fn decode(bytes: &[u8]) -> Result<Self, String> {
        let mut state: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        match state.schema_version {
            1 => state.schema_version = STATE_SCHEMA,
            STATE_SCHEMA => {}
            version => return Err(format!("unsupported schema version {version}")),
        }
        Ok(state)
    }
}
/// Removes an unpublished generation after any pre-publication failure.
struct UnpublishedGeneration(Option<PathBuf>);
impl Drop for UnpublishedGeneration {
    fn drop(&mut self) {
        if let Some(path) = &self.0
            && let Err(error) = fs::remove_dir_all(path)
        {
            tracing::warn!(%error, path=%path.display(), "unpublished generation cleanup deferred");
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
    pub profiles: Vec<Profile>,
    /// History across every run, even when the selected profile has never played.
    pub last_played: Option<u64>,
    pub launch: Result<Summary, Blocker>,
    pub favorite: bool,
    pub recipe_status: RecipeStatus,
    pub change_summary: String,
    pub update_dismissed: bool,
    pub update_notice: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct BaseItem {
    pub base: iwads::base_games::BaseGame,
    pub profile: Profile,
    pub profiles: Vec<Profile>,
    pub last_played: Option<u64>,
}
impl BaseItem {
    pub fn copy_notice(&self, entries: &[RegistryEntry]) -> Option<&'static str> {
        self.profile
            .iwad_path
            .as_ref()
            .filter(|path| {
                !path.is_file()
                    || !entries.iter().any(|e| {
                        &e.path == *path && e.game == Some(self.base.game) && e.variant.is_some()
                    })
            })
            .map(|_| "Selected copy unavailable. Using the preferred IWAD copy.")
    }
}
#[derive(Debug, Clone)]
pub struct IwadFolder {
    pub path: PathBuf,
    pub source: iwads::Source,
    pub games: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub base_games: Vec<BaseItem>,
    pub automatic_folders: Vec<IwadFolder>,
    pub items: Vec<Item>,
    pub custom: Vec<CustomSetup>,
    pub custom_profiles: Vec<Profile>,
    pub local_issues: BTreeMap<String, (PathBuf, bool)>,
    pub profiles_with_saves: BTreeSet<String>,
    /// The reviewed tag vocabulary, for Discover categories and chips.
    pub tags: TagVocabulary,
    pub engine: Option<dsda::InstalledEngine>,
    pub previous: Option<String>,
    pub attribution: Option<dsda::Attribution>,
    pub iwads: Vec<RegistryEntry>,
    pub folders: Vec<PathBuf>,
    /// R4: whether DDL probes the generic DOOM roots (default off).
    pub search_standard_folders: bool,
    /// Spec 019 R5: the catalog in use and its automatic-update settings.
    pub catalog: CatalogInfo,
    /// Spec 059 R4: paths a launcher record or a configured folder names that
    /// the Flatpak cannot open. Always empty outside the sandbox.
    pub hidden_paths: Vec<PathBuf>,
    pub config_notice: bool,
    pub config_usable: bool,
    pub config_version: String,
    /// Spec 067: the optional MIDI soundfont choice and install state.
    pub audio: AudioState,
    pub logs: PathBuf,
}
/// Spec 067 R1/R2/R4: what Settings → Audio shows without hashing on every
/// snapshot. `None` means no explicit choice, so the engine's own configuration
/// is preserved and honestly described by [`Self::current`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioState {
    pub choice: Option<Choice>,
    /// A reviewed installation is published (cheap pointer/size/symlink check).
    pub installed: bool,
    /// What the engine's active configuration currently holds.
    pub current: soundfonts::engine::Current,
    /// Selected but missing/damaged, or unreadable audio preferences.
    pub warning: Option<String>,
}
/// The catalog the app is using, for Settings and `ddl catalog status`
/// (spec 019, R5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogInfo {
    /// The effective catalog's serial (UTC Unix seconds); `0` when the build
    /// never learned it (the embedded catalog only).
    pub serial: u64,
    /// Whether the catalog came from the cache (an accepted update) rather than
    /// the binary.
    pub updated: bool,
    /// The source commit of an updated catalog (empty for the embedded one).
    pub source_commit: String,
    /// When an updated catalog was fetched, UTC Unix seconds.
    pub fetched_at: Option<u64>,
    /// Whether automatic checks are enabled (spec 019 R5, default on).
    pub auto_update: bool,
    /// The last automatic/manual check attempt, UTC Unix seconds.
    pub last_check: Option<u64>,
}
#[derive(Debug, Clone)]
pub struct Directory {
    pub path: PathBuf,
    pub entries: Vec<(PathBuf, bool)>,
    pub shortcuts: Vec<PathBuf>,
    pub files: Vec<UserFile>,
    pub iwad_files: BTreeSet<PathBuf>,
}
/// UI local-content operations execute on the same worker as catalog services.
#[derive(Debug, Clone)]
pub enum LocalCommand {
    Browse(PathBuf),
    Create(String, NewIwad, Compatibility),
    Save(CustomSetup),
    Add(String, Option<String>, Vec<PathBuf>),
    Extras(Profile),
    Delete(String, bool),
    Accept(String, Option<String>),
    Play(String),
    BaseSave(Profile),
    /// Spec 075, R1/R2: the base-game Pistol start toggle, the second
    /// production control that must not rebuild the snapshot.
    BasePistolStart {
        /// The base profile to change.
        profile_id: String,
        /// The requested value.
        value: bool,
    },
    BaseNew(String, bool),
}
#[derive(Debug, Clone)]
pub enum Command {
    Local(LocalCommand),
    Startup,
    InstallEngine(String),
    Scan(Option<PathBuf>),
    Choose(PathBuf, String),
    RemoveFolder(PathBuf),
    Install(String),
    UpdateMapset(String),
    /// Spec 022: remove an installed mapset; the flag also deletes its profiles.
    Uninstall(String, bool),
    KeepCurrent(String),
    Save(Profile),
    SelectProfile(String, String),
    NewProfile(String, bool),
    RenameProfile(String, String),
    DeleteProfile(String),
    /// Toggles the favorite mark of a recipe id (spec 015, R5).
    Favorite(String),
    Play(String),
    Browse(PathBuf, bool),
    Updates,
    Rollback,
    Reset,
    Import(String, String, PathBuf),
    /// R4: persist the "Search standard Linux folders" setting.
    SetSearchStandardFolders(bool),
    /// Spec 019 R5: fetch a catalog update, the manual Settings/CLI action
    /// (spec 077, R1 keeps the per-startup automatic check on its own path).
    CatalogCheck,
    /// Spec 077, R1: rebuild the snapshot on a fresh services instance after a
    /// background catalog adoption; writes nothing.
    RefreshCatalog,
    /// Spec 075, R2: set one profile's Pistol start. The value is absolute, so a
    /// repeated toggle can never apply twice, and the command reports the saved
    /// profile instead of a full snapshot.
    SetPistolStart {
        /// The recipe the profile belongs to.
        recipe_id: String,
        /// The profile to change.
        profile_id: String,
        /// The requested value.
        value: bool,
    },
    /// Spec 019 R5: persist the "Automatic catalog updates" toggle.
    SetCatalogAutoUpdate(bool),
    /// Spec 067 R2: download and verify the reviewed Arachno archive without
    /// selecting it.
    InstallSoundfont,
    /// Spec 067 R4: persist an explicit Audio choice.
    SelectSoundfont(Choice),
}

impl Command {
    /// Whether this command writes user-visible state and so must hold the
    /// cross-process state lock (spec 014, R2).
    ///
    /// `Play` is deliberately excluded: it holds spec 005's run guard for the
    /// whole engine run, and a two-hour game must not block CLI installs.
    #[must_use]
    fn mutates_state(&self) -> bool {
        !matches!(
            self,
            Self::Play(_) | Self::Browse(..) | Self::Updates | Self::RefreshCatalog
        )
    }
}
#[derive(Debug)]
pub enum Event {
    Progress(InstallProgress),
    Working,
    Playing,
    Directory(Directory),
    Updates(Vec<String>),
    /// Spec 059 R5: the tarball-era setup was imported; the count is how many
    /// engine versions need installing again for the Flatpak.
    Imported {
        engines_to_reinstall: usize,
    },
    /// Spec 019 R5: the result of a catalog check.
    Catalog(CatalogOutcome),
    /// Spec 075, R2: the profile a light command persisted, as saved on disk.
    ProfileSaved(Profile),
    Finished(Result<Option<Box<Snapshot>>, Problem>),
    EngineExit(dsda::LaunchOutcome),
}
/// How the UI refreshes after a command (spec 075, R1).
#[derive(Debug)]
pub enum Refresh {
    /// Rebuild and install the whole snapshot.
    Full,
    /// The command reported the single value it changed, so no snapshot is
    /// built (Pistol start): no launch re-resolution and no member re-hashing.
    Profile(Box<Profile>),
}

/// What the per-startup catalog check produced (spec 077, R1). It deliberately
/// carries no snapshot: the check runs on an instance that opened before the
/// network call, so the UI adopts the new catalog through a fresh
/// [`Command::RefreshCatalog`] instead of a possibly stale view.
#[derive(Debug, Default)]
pub struct StartupCatalog {
    /// The accepted bundle's serial when a newer catalog became active.
    pub updated: Option<u64>,
}

/// What a catalog check did (spec 019, R5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogOutcome {
    /// A newer bundle was accepted and cached.
    Updated {
        /// The accepted bundle's serial.
        serial: u64,
        /// Its source commit.
        source_commit: String,
        /// Recipes dropped during validation, if any.
        dropped: usize,
    },
    /// No newer bundle: DDL already has the newest (or the remote is not newer).
    Current {
        /// The effective catalog's serial.
        serial: u64,
    },
}
/// Test overrides are inert unless explicitly provided by the headless tests.
#[derive(Clone)]
pub struct Environment {
    pub paths: Paths,
    pub home: PathBuf,
    /// The drawable size of DDL's fullscreen window for this launch (spec 064,
    /// R6). `None` keeps the template's `screen_resolution` fallback.
    pub screen: Option<dsda::ScreenSize>,
    #[cfg(test)]
    pub test: Option<std::sync::Arc<TestEnvironment>>,
}
#[cfg(test)]
pub struct TestEnvironment {
    pub api: std::sync::Arc<dsda::CannedApi>,
    pub catalog: Catalog,
    pub engine_base: String,
    /// Spec 019 R3: the bundle URL a test drives instead of GitHub.
    pub catalog_url: String,
    pub reviewed: dsda::ReviewedList,
    pub runner: std::sync::Arc<dyn EngineRunner + Send + Sync>,
    /// Spec 024 R4: a longer response-header timeout for the gated
    /// engine-download fixture, so a starved test thread cannot trip the
    /// production 15 s timeout. `None` keeps production behaviour.
    pub engine_response_timeout: Option<std::time::Duration>,
    /// Spec 067: reviewed-member override for a tiny offline install fixture.
    pub soundfont_members: Option<Vec<ddl_core::downloads::MemberRequest>>,
}

pub struct Services {
    env: Environment,
    catalog: Catalog,
    catalog_origin: CatalogOrigin,
    /// The hosts a catalog fetch may contact. Fixed when the app opens from the
    /// binary's own allowlist, so accepting a bundle can never widen it
    /// (spec 019 R3: remote changes to the hosts allowlist are out of scope).
    catalog_allowlist: HostAllowlist,
    state: AppState,
    registry: Registry,
    config: DsdaConfigManager,
    installer: Installer,
    engine: EngineManager,
    profiles: ProfileStore,
    /// Spec 026: player-made custom setups under `<data>/custom/`.
    custom: SetupStore,
    /// Spec 059: the sandbox DDL runs in; `Sandbox::none()` outside Flatpak.
    sandbox: Sandbox,
    /// Spec 067: optional reviewed MIDI soundfonts.
    soundfonts: Soundfonts,
    /// Spec 075, R4: deterministic per-instance counters proving the light path
    /// builds no snapshot and resolves no launch (test builds only).
    #[cfg(test)]
    counters: std::sync::Arc<Counters>,
}

/// Spec 075, R4: deterministic counters for the "no library-wide work" checks.
#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct Counters {
    snapshots: std::sync::atomic::AtomicUsize,
    resolves: std::sync::atomic::AtomicUsize,
}

#[cfg(test)]
impl Counters {
    pub(crate) fn snapshots(&self) -> usize {
        self.snapshots.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) fn resolves(&self) -> usize {
        self.resolves.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
impl Services {
    pub(crate) fn counters(&self) -> &Counters {
        &self.counters
    }
}
/// Where the effective catalog came from (spec 019, R4).
#[derive(Debug, Clone, PartialEq, Eq)]
struct CatalogOrigin {
    serial: u64,
    updated: bool,
    source_commit: String,
    fetched_at: Option<u64>,
}

impl Environment {
    /// Seed scratch fixture state before a Runtime can dispatch its automatic
    /// check. This exercises production services without contacting the catalog.
    #[cfg(test)]
    pub(crate) fn without_catalog_auto_updates(self) -> Self {
        let mut services = Services::open(self.clone()).expect("open scratch fixture services");
        services
            .execute(
                Command::SetCatalogAutoUpdate(false),
                &CancellationToken::new(),
                &mut |_| {},
            )
            .expect("disable catalog updates in scratch fixture state");
        self
    }

    /// Whether this is a production run (no test seam installed). The automatic
    /// catalog check is dispatched only in production; tests drive the command
    /// themselves with a loopback URL (spec 019, A3).
    pub(crate) fn is_production(&self) -> bool {
        #[cfg(test)]
        {
            self.test.is_none()
        }
        #[cfg(not(test))]
        {
            true
        }
    }
}
impl Services {
    /// Spec 075, R2: persist one profile's Pistol start.
    ///
    /// This is the whole toggle: validate the profile against its recipe, set
    /// the single field, save atomically and report the saved profile. It never
    /// resolves a launch and never rebuilds the snapshot, so installed members
    /// are not re-read or re-hashed here (075, R1/R3); the launch-time
    /// integrity check in `resolver` is unchanged and still runs on Play.
    fn set_pistol_start(
        &mut self,
        recipe_id: &str,
        profile_id: &str,
        value: bool,
    ) -> Result<Refresh, Problem> {
        let recipe = self.recipe(recipe_id)?;
        // Read under the same lock as the write, so a concurrent profile change
        // cannot be lost between the two.
        let _state_lock = self.lock_state()?;
        let mut profile = match self
            .profiles
            .load_optional(profile_id)
            .map_err(|e| logged(Problem::State, e))?
        {
            Some(profile) => profile,
            // First press before any Save: the UI shows an unsaved default, so
            // materialise it instead of refusing the toggle.
            None => {
                let mut profile = default_profile(&recipe);
                profile.id = profile_id.to_owned();
                profile
            }
        };
        if profile.recipe_id != recipe_id {
            return Err(Problem::State);
        }
        profile.pistol_start = value;
        profile
            .validate(&recipe)
            .map_err(|e| logged(Problem::State, e))?;
        // Profile-only atomic write: no app.json/iwads.json rewrite, so there is
        // no partial success to roll back and no way to clobber a concurrent
        // favourite/scan (spec 075, R2 review).
        self.profiles
            .save(&profile)
            .map_err(|e| logged(Problem::Storage, e))?;
        Ok(Refresh::Profile(Box::new(profile)))
    }

    /// Spec 075, R1/R2: the base-game Pistol start toggle through the same light
    /// path: validate the profile against its base game, save it atomically and
    /// report it, with no snapshot and no launch re-resolution.
    fn set_base_pistol_start(&mut self, profile_id: &str, value: bool) -> Result<Refresh, Problem> {
        let _state_lock = self.lock_state()?;
        let existing = self
            .profiles
            .load_optional(profile_id)
            .map_err(|e| logged(Problem::State, e))?;
        let base_id = match &existing {
            Some(profile) => profile.recipe_id.clone(),
            // A first press before any Save: the id names the base game, e.g.
            // `base-doom2-profile-1`, so resolve the base from the registry.
            None => profile_id
                .rsplit_once("-profile-")
                .map(|(base, _)| base.to_owned())
                .ok_or(Problem::State)?,
        };
        let base = iwads::base_games::list(&self.registry)
            .into_iter()
            .find(|base| base.id() == base_id)
            .ok_or(Problem::State)?;
        let mut profile =
            existing.unwrap_or_else(|| Profile::new(profile_id, "Default", base.id()));
        // Base profiles have no recipe component combination to validate; keep
        // the same schema and skill rules `Profile::validate` applies.
        if profile.schema_version != ddl_core::profiles::SCHEMA_VERSION
            || !(1..=5).contains(&profile.skill)
        {
            return Err(Problem::State);
        }
        profile.pistol_start = value;
        // Profile-only atomic write, as above (the lock is already held).
        self.profiles
            .save(&profile)
            .map_err(|e| logged(Problem::Storage, e))?;
        Ok(Refresh::Profile(Box::new(profile)))
    }

    fn execute_local(
        &mut self,
        command: LocalCommand,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(Event),
    ) -> Result<(), Problem> {
        let launch_id = match &command {
            LocalCommand::Play(id) => Some(id.clone()),
            _ => None,
        };
        let result: Result<(), CliFailure> = (|| {
            match command {
                LocalCommand::Browse(path) => {
                    let mut directory = self
                        .browse(&path, false, cancel)
                        .map_err(|e| CliFailure::message(e.to_string()))?;
                    for entry in fs::read_dir(&path)
                        .map_err(|e| CliFailure::message(e.to_string()))?
                        .take(4096)
                    {
                        if cancel.is_cancelled() {
                            return Err(CliFailure::message("cancelled"));
                        }
                        if let Ok(entry) = entry
                            && let Ok(file) = UserFile::read(&entry.path())
                        {
                            directory.entries.push((file.path.clone(), false));
                            if local::is_iwad_header(&file.path).unwrap_or(false) {
                                directory.iwad_files.insert(file.path.clone());
                            }
                            directory.files.push(file);
                        }
                    }
                    directory.entries.sort_by(|a, b| {
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
                    });
                    directory.shortcuts.splice(
                        1..1,
                        [
                            self.env.home.join("Downloads"),
                            self.env.home.join("Documents"),
                        ]
                        .into_iter()
                        .filter(|p| p.is_dir()),
                    );
                    emit(Event::Directory(directory));
                }
                LocalCommand::Create(name, iwad, compatibility) => {
                    self.custom_create(&name, None, iwad, compatibility)?;
                }
                LocalCommand::Save(setup) => {
                    let _lock = self.lock_cli()?;
                    self.save_custom(setup)?;
                }
                LocalCommand::Add(id, profile, paths) => {
                    if self.has_custom(&id) {
                        self.custom_add_file(&id, &paths)?;
                    } else {
                        self.extras_add(&id, profile.as_deref(), &paths)?;
                    }
                }
                LocalCommand::Extras(profile) => {
                    let _lock = self.lock_cli()?;
                    self.save_extras(profile)?;
                }
                LocalCommand::Delete(id, profiles) => self.custom_delete(&id, profiles)?,
                LocalCommand::Accept(id, profile) => {
                    if self.has_custom(&id) {
                        let _lock = self.lock_cli()?;
                        let mut setup = self.custom_show(&id)?;
                        local::accept_changed(&mut setup.files)
                            .map_err(|e| CliFailure::message(e.to_string()))?;
                        local::accept_changed(&mut setup.patches)
                            .map_err(|e| CliFailure::message(e.to_string()))?;
                        if let CustomIwad::File { file } = &mut setup.iwad {
                            local::accept_changed(std::slice::from_mut(file))
                                .map_err(|e| CliFailure::message(e.to_string()))?;
                        }
                        self.save_custom(setup)?;
                    } else {
                        self.accept_extra_changes(&id, profile.as_deref())?;
                    }
                }
                LocalCommand::BasePistolStart { .. } => {
                    unreachable!("the light base Pistol start path returns before this match")
                }
                LocalCommand::BaseSave(profile) => {
                    let _lock = self.lock_cli()?;
                    self.base_item(&profile.recipe_id, None)?;
                    self.save_extras(profile.clone())?;
                    self.state
                        .last_profiles
                        .insert(profile.recipe_id, profile.id);
                    self.save()
                        .map_err(|e| CliFailure::message(e.to_string()))?;
                }
                LocalCommand::BaseNew(id, copy) => {
                    let _lock = self.lock_cli()?;
                    let item = self.base_item(&id, None)?;
                    self.profiles
                        .save(&item.profile)
                        .map_err(|e| CliFailure::message(e.to_string()))?;
                    let mut profile = if copy {
                        item.profile
                    } else {
                        self.fresh_base_profile(&item.base)?
                    };
                    profile.id = self.free_base_profile_id(item.base.game)?;
                    profile.name = (2..)
                        .find_map(|n| {
                            ddl_core::profiles::validate_name(
                                &format!("Profile {n}"),
                                &id,
                                None,
                                &item.profiles,
                            )
                            .ok()
                        })
                        .expect("a free numbered name");
                    profile.last_played = None;
                    self.profiles
                        .save(&profile)
                        .map_err(|e| CliFailure::message(e.to_string()))?;
                    self.state.last_profiles.insert(id, profile.id);
                    self.save()
                        .map_err(|e| CliFailure::message(e.to_string()))?;
                }
                LocalCommand::Play(id) => {
                    emit(Event::Playing);
                    let outcome = if id.starts_with("base:") {
                        self.run_base(&id, &LaunchOverrides::default(), false)?
                    } else {
                        self.run_custom(&id, None, false)?
                    };
                    emit(Event::EngineExit(outcome));
                }
            }
            Ok(())
        })();
        result.map_err(|e| {
            if let Some(id) = launch_id
                && let Ok(setup) = self.custom_show(&id)
                && let Some(issue) = local::verify_setup(&setup).into_iter().next()
            {
                let (path, changed) = identity_issue(issue);
                return Problem::LocalFile(path, changed);
            }
            Problem::Local(e.to_string())
        })
    }

    pub fn open(env: Environment) -> Result<Self, Problem> {
        Self::open_with(env, default_sandbox())
    }

    /// Opens the services for an explicit sandbox (spec 059); tests pass a fake.
    pub fn open_with(env: Environment, sandbox: Sandbox) -> Result<Self, Problem> {
        let paths = &env.paths;
        for dir in [&paths.config, &paths.data, &paths.cache, &paths.state] {
            fs::create_dir_all(dir).map_err(|e| logged(Problem::Storage, e))?;
        }
        let unreadable = state_recovery::unreadable_files(&env);
        if !unreadable.is_empty() {
            return Err(Problem::Unreadable(unreadable));
        }
        let state_path = paths.config.join("app.json");
        let state: AppState = match fs::read(&state_path) {
            Ok(bytes) if bytes.len() <= 4 * 1024 * 1024 => AppState::parse(&bytes)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => AppState::default(),
            result => {
                return Err(logged(
                    Problem::State,
                    format!("app state unavailable: {result:?}"),
                ));
            }
        };
        let catalog = Catalog::embedded();
        #[cfg(test)]
        let catalog = env.test.as_ref().map_or(catalog, |t| t.catalog.clone());
        // The catalog fetch may only contact the binary's own allowlist. It is
        // captured here, before any cached bundle can replace `catalog`, so
        // accepting a bundle can never widen it (spec 019, R3).
        let catalog_allowlist = HostAllowlist::new(catalog.hosts.downloads.clone());
        // Spec 019 R4: load the newer of embedded vs cached by serial. A corrupt
        // cache is ignored and logged, never fatal.
        let embedded_serial = ddl_core::catalog::embedded_serial();
        let mut catalog_origin = CatalogOrigin {
            serial: embedded_serial,
            updated: false,
            source_commit: String::new(),
            fetched_at: None,
        };
        let catalog = match ddl_core::catalog::load_cache(&paths.data) {
            Some(loaded) if loaded.bundle.catalog_serial > embedded_serial => {
                catalog_origin = CatalogOrigin {
                    serial: loaded.bundle.catalog_serial,
                    updated: true,
                    source_commit: loaded.bundle.source_commit.clone(),
                    fetched_at: Some(loaded.fetched_at),
                };
                loaded.catalog
            }
            _ => catalog,
        };
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
        let engine = EngineManager::for_paths(paths).with_sandbox(sandbox.clone());
        #[cfg(test)]
        let engine = if let Some(test) = &env.test {
            let mut engine = EngineManager::with_parts(
                dsda::EngineDirs::for_paths(paths),
                test.reviewed.clone(),
                Box::new(TestApi(test.api.clone())),
                Box::new(dsda::SystemClock),
            )
            .with_http_for_tests()
            .with_download_base(&test.engine_base);
            if let Some(timeout) = test.engine_response_timeout {
                engine = engine.with_response_timeout(timeout);
            }
            engine.with_sandbox(sandbox.clone())
        } else {
            engine
        };
        let config =
            DsdaConfigManager::new(&paths.config).map_err(|e| logged(Problem::Config, e))?;
        let profiles = ProfileStore::new(&paths.config, &paths.data);
        let custom = SetupStore::new(&paths.data);
        let registry_path = paths.config.join("iwads.json");
        let registry = match Registry::load(&registry_path) {
            Ok(registry) => registry,
            Err(iwads::RegistryError::Io(e)) if e.kind() == io::ErrorKind::NotFound => {
                Registry::new()
            }
            Err(e) => return Err(logged(Problem::State, e)),
        };
        let soundfonts = Self::soundfonts_for(&env);
        Ok(Self {
            env,
            catalog,
            catalog_origin,
            catalog_allowlist,
            state,
            registry,
            config,
            installer,
            engine,
            profiles,
            custom,
            sandbox,
            soundfonts,
            #[cfg(test)]
            counters: std::sync::Arc::new(Counters::default()),
        })
    }
    /// Spec 067: production uses the reviewed author archive; test-support may
    /// substitute a tiny verified fixture so offline UI/service tests can cover
    /// installed and selected states.
    fn soundfonts_for(env: &Environment) -> Soundfonts {
        #[cfg(test)]
        if let Some(members) = env.test.as_ref().and_then(|t| t.soundfont_members.clone()) {
            return Soundfonts::with_members(&env.paths, members);
        }
        Soundfonts::new(&env.paths)
    }
    fn save(&self) -> Result<(), Problem> {
        save_json(&self.env.paths.config.join("app.json"), &self.state)?;
        self.registry
            .save(&self.env.paths.config.join("iwads.json"))
            .map_err(|e| logged(Problem::Storage, e))
    }
    fn manifest(
        &self,
        recipe: &Recipe,
    ) -> Result<Option<InstallManifest>, ddl_core::library::InstallError> {
        self.installer.load_manifest(recipe)
    }
    fn active(&self, id: &str) -> Result<Option<ActiveInstall>, Problem> {
        self.installer.active(id).map_err(install_error)
    }
    fn install_content(
        &mut self,
        recipe: &Recipe,
        enabled: &[&str],
        cancel: &CancellationToken,
        progress: &mut dyn FnMut(InstallProgress),
    ) -> Result<InstallManifest, ddl_core::library::InstallError> {
        if let Some(mut active) = self.installer.active(&recipe.id)? {
            // A reinstall after an interrupted uninstall adopts the record and
            // makes it active again (spec 022, R3).
            active.uninstalled = false;
            let previous = active.manifest.clone();
            let mut manifest = self
                .installer
                .stage_update(recipe, enabled, &previous, cancel, progress)?;
            let mut unpublished =
                UnpublishedGeneration(Some(self.installer.manifest_dir(&manifest)?));
            for profile in self
                .profiles
                .list()
                .map_err(|e| ddl_core::library::InstallError::Serialize(e.to_string()))?
                .iter()
                .filter(|p| p.recipe_id == recipe.id)
            {
                manifest = self.installer.extend_install(
                    recipe,
                    &profile
                        .components
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                    &manifest,
                    cancel,
                    progress,
                )?;
            }
            self.installer.verify_staged(&manifest, cancel)?;
            active
                .retired_dirs
                .push(self.installer.manifest_dir(&previous)?);
            active.manifest = manifest.clone();
            self.installer.publish(&active)?;
            unpublished.0 = None;
            self.cleanup_retired();
            Ok(manifest)
        } else {
            self.installer
                .install_with_progress(recipe, enabled, cancel, progress)
        }
    }
    fn adopt_metadata(&mut self) -> Result<(), Problem> {
        for id in self.installed_ids() {
            let old = self.recipe(&id)?;
            let Some(new) = self.catalog.recipe(&id).cloned() else {
                continue;
            };
            let Some(mut manifest) = self.manifest(&old).map_err(install_error)? else {
                continue;
            };
            if classify(&manifest, &old, Some(&new)) == RecipeStatus::MetadataOnly {
                manifest.recipe_sha256 = recipe_content_sha256(&new).map_err(install_error)?;
                let mut active = self.active(&id)?.unwrap_or(ActiveInstall {
                    schema_version: 1,
                    recipe: old,
                    manifest: manifest.clone(),
                    previous_recipe: None,
                    pending_profiles: vec![],
                    notices: vec![],
                    retired_dirs: vec![],
                    uninstalled: false,
                });
                active.recipe = new;
                active.manifest = manifest;
                self.installer.publish(&active).map_err(install_error)?;
            }
        }
        Ok(())
    }
    fn installed_ids(&self) -> BTreeSet<String> {
        let mut ids: BTreeSet<_> = self.state.recipes.keys().cloned().collect();
        if let Ok(entries) = fs::read_dir(self.env.paths.data.join("library")) {
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|t| t.is_dir())
                    && entry.path().join("active.json").is_file()
                    && let Some(id) = entry.file_name().to_str()
                {
                    ids.insert(id.to_owned());
                }
            }
        }
        ids
    }
    fn reconcile_install(&self, id: &str) -> Result<(), Problem> {
        let Some(mut active) = self.active(id)? else {
            return Ok(());
        };
        if active.uninstalled {
            return Ok(());
        }
        let Some(old) = active.previous_recipe.as_ref() else {
            return Ok(());
        };
        // Save each profile atomically before removing its pending marker. A crash
        // between those writes repeats the same deterministic reconciliation.
        for id in active.pending_profiles.clone() {
            if let Some(mut profile) = self
                .profiles
                .load_optional(&id)
                .map_err(|e| logged(Problem::State, e))?
            {
                reconcile_profile(&mut profile, old, &active.recipe);
                profile
                    .validate(&active.recipe)
                    .map_err(|e| logged(Problem::State, e))?;
                self.profiles
                    .save(&profile)
                    .map_err(|e| logged(Problem::Storage, e))?;
            }
            active.pending_profiles.retain(|p| p != &id);
            self.installer.publish(&active).map_err(install_error)?;
        }
        Ok(())
    }
    fn cleanup_retired(&self) {
        for id in self.installed_ids() {
            let Ok(Some(mut active)) = self.active(&id) else {
                continue;
            };
            if active.uninstalled {
                // Spec 022: uninstalls are finished by `finish_uninstalls`, which
                // also drops the record; never treat them as a retired generation.
                continue;
            }
            if active.retired_dirs.is_empty() {
                continue;
            }
            active.retired_dirs.retain(|dir| {
                if active.manifest.members.iter().any(|m| m.path.starts_with(dir)) { return true; }
                match self.installer.remove_retired(dir) {
                    Ok(()) => false,
                    Err(error) => { tracing::warn!(%error, path=%dir.display(), "retired install cleanup deferred"); true }
                }
            });
            if let Err(error) = self.installer.publish(&active) {
                tracing::warn!(%error, "retired cleanup queue retained");
            }
        }
    }
    /// Finishes uninstalls interrupted between the atomic mark and the delete
    /// (spec 022, R3). A mapset whose record is already gone is left alone.
    fn finish_uninstalls(&mut self) -> Result<(), Problem> {
        let mut removed = Vec::new();
        for id in self.installed_ids() {
            match self.installer.finish_uninstall(&id) {
                Ok(true) => removed.push(id),
                Ok(false) => {
                    tracing::warn!(mapset = id, "uninstall cleanup deferred to the next start");
                }
                Err(error) => {
                    tracing::warn!(%error, mapset = id, "uninstall cleanup deferred to the next start");
                }
            }
        }
        if !removed.is_empty() {
            for id in removed {
                self.state.recipes.remove(&id);
            }
            self.save()?;
        }
        Ok(())
    }
    fn update_mapset_inner(
        &mut self,
        id: &str,
        cancel: &CancellationToken,
        progress: &mut dyn FnMut(InstallProgress),
    ) -> Result<InstallManifest, Problem> {
        self.reconcile_install(id)?;
        let old = self.recipe(id)?;
        let new = self.catalog.recipe(id).cloned().ok_or(Problem::State)?;
        let previous = self
            .manifest(&old)
            .map_err(install_error)?
            .ok_or(Problem::State)?;
        let status = classify(&previous, &old, Some(&new));
        if matches!(status, RecipeStatus::MetadataOnly | RecipeStatus::Current) {
            self.adopt_metadata()?;
            return self
                .manifest(&new)
                .map_err(install_error)?
                .ok_or(Problem::State);
        }
        let mut profiles: Vec<_> = self
            .profiles
            .list()
            .map_err(|e| logged(Problem::State, e))?
            .into_iter()
            .filter(|p| p.recipe_id == id)
            .collect();
        if profiles.is_empty() {
            let p = default_profile(&old);
            self.profiles
                .save(&p)
                .map_err(|e| logged(Problem::Storage, e))?;
            profiles.push(p);
        }
        let pending_profiles = profiles.iter().map(|p| p.id.clone()).collect();
        let mut notices = Vec::new();
        for profile in &mut profiles {
            for dropped in reconcile_profile(profile, &old, &new) {
                notices.push(format!("{}: removed add-on {dropped}", profile.name));
            }
            profile
                .validate(&new)
                .map_err(|e| logged(Problem::State, e))?;
        }
        let mut staging_progress = |p: InstallProgress| {
            progress(if p.stage == ddl_core::downloads::InstallStage::Commit {
                InstallProgress::stage(ddl_core::downloads::InstallStage::Verify)
            } else {
                p
            });
        };
        let mut manifest = self
            .installer
            .stage_update(
                &new,
                &profiles[0]
                    .components
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                &previous,
                cancel,
                &mut staging_progress,
            )
            .map_err(install_error)?;
        let mut unpublished = UnpublishedGeneration(Some(
            self.installer
                .manifest_dir(&manifest)
                .map_err(install_error)?,
        ));
        for profile in profiles.iter().skip(1) {
            manifest = self
                .installer
                .extend_install(
                    &new,
                    &profile
                        .components
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                    &manifest,
                    cancel,
                    &mut staging_progress,
                )
                .map_err(install_error)?;
        }
        self.installer
            .verify_staged(&manifest, cancel)
            .map_err(install_error)?;
        if cancel.is_cancelled() {
            return Err(Problem::Cancelled);
        }
        let mut retired_dirs = self.active(id)?.map_or_else(Vec::new, |a| a.retired_dirs);
        retired_dirs.push(
            self.installer
                .manifest_dir(&previous)
                .map_err(install_error)?,
        );
        let active = ActiveInstall {
            schema_version: 1,
            recipe: new,
            manifest: manifest.clone(),
            previous_recipe: Some(old),
            pending_profiles,
            notices,
            retired_dirs,
            uninstalled: false,
        };
        progress(InstallProgress::stage(
            ddl_core::downloads::InstallStage::Commit,
        ));
        self.installer.publish(&active).map_err(install_error)?;
        unpublished.0 = None;
        // Publication succeeded. Reconciliation/cleanup failures are retryable
        // maintenance, never reported as a failed content transaction.
        if let Err(error) = self.reconcile_install(id) {
            tracing::warn!(%error, "profile reconciliation deferred");
        }
        self.cleanup_retired();
        Ok(manifest)
    }
    pub fn update_mapset(
        &mut self,
        id: &str,
        progress: &mut dyn FnMut(InstallProgress),
    ) -> Result<InstallManifest, CliFailure> {
        let _lock = self.lock_cli()?;
        if self.catalog.recipe(id).is_none() {
            return Err(CliFailure::message(format!(
                "`{id}` is not in the catalog; an installed copy can still be launched"
            )));
        }
        let recipe = self
            .recipe(id)
            .map_err(|e| CliFailure::message(e.to_string()))?;
        if self
            .manifest(&recipe)
            .map_err(|e| CliFailure::message(e.to_string()))?
            .is_none()
        {
            return Err(CliFailure::message(format!(
                "`{id}` is not installed; use `ddl mapset install {id}`"
            )));
        }
        self.update_mapset_inner(id, &CancellationToken::new(), progress)
            .map_err(|e| CliFailure::message(e.to_string()))
    }

    /// Marks an install inactive atomically, then deletes its files; when
    /// `delete_profiles` is set the mapset's profile documents and their
    /// save/data/shot/run dirs go too (spec 022, R3/R4).
    ///
    /// Profile documents are removed before their runtime directories (spec 018
    /// order). A failure to finish deleting files still leaves the mapset
    /// "not installed"; the next start retries the leftovers.
    ///
    /// # Errors
    ///
    /// Returns [`Problem::State`] when the mapset is not installed or its record
    /// is unreadable, and [`Problem::Storage`] on a filesystem failure.
    fn uninstall_mapset_inner(
        &mut self,
        id: &str,
        delete_profiles: bool,
    ) -> Result<UninstallOutcome, Problem> {
        let recipe = self.recipe(id)?;
        let manifest = self
            .manifest(&recipe)
            .map_err(install_error)?
            .ok_or(Problem::State)?;
        let profiles: Vec<Profile> = self
            .profiles
            .list()
            .map_err(|e| logged(Problem::State, e))?
            .into_iter()
            .filter(|p| p.recipe_id == id)
            .collect();
        // R3: make the install inactive atomically before touching any file.
        self.installer
            .begin_uninstall(&recipe, &manifest)
            .map_err(install_error)?;
        let mut deleted_profiles = 0;
        if delete_profiles {
            for profile in &profiles {
                match self.profiles.delete_fully(&profile.id) {
                    Ok(()) => deleted_profiles += 1,
                    Err(error) => {
                        tracing::warn!(%error, profile = profile.id, "profile deletion deferred");
                    }
                }
            }
            // R4: clear the remembered profile only when its profiles are deleted.
            self.state.last_profiles.remove(id);
        }
        match self.installer.finish_uninstall(id) {
            Ok(true) => {
                self.state.recipes.remove(id);
            }
            Ok(false) => {
                tracing::warn!(mapset = id, "uninstall cleanup deferred to the next start");
            }
            Err(error) => {
                tracing::warn!(%error, mapset = id, "uninstall cleanup deferred to the next start");
            }
        }
        self.save()?;
        Ok(UninstallOutcome {
            id: recipe.id,
            bytes: manifest.installed_bytes(),
            profiles: deleted_profiles,
            imported: manifest.has_imports(),
        })
    }

    /// `ddl mapset uninstall <id> [--delete-profiles]`: the same transaction the
    /// UI uses, refusing while another DDL process holds the state lock.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown or uninstalled mapset, a busy
    /// state lock or a filesystem failure.
    pub fn uninstall_mapset(
        &mut self,
        id: &str,
        delete_profiles: bool,
    ) -> Result<UninstallOutcome, CliFailure> {
        let _lock = self.lock_cli()?;
        if self.catalog.recipe(id).is_none() && !self.state.recipes.contains_key(id) {
            return Err(CliFailure::message(format!(
                "no catalog recipe with id `{id}`"
            )));
        }
        let recipe = self
            .recipe(id)
            .map_err(|e| CliFailure::message(e.to_string()))?;
        if self
            .manifest(&recipe)
            .map_err(|e| CliFailure::message(e.to_string()))?
            .is_none()
        {
            return Err(CliFailure::message(format!("`{id}` is not installed")));
        }
        self.uninstall_mapset_inner(id, delete_profiles)
            .map_err(|e| CliFailure::message(e.to_string()))
    }
    pub fn mapset_status(
        &self,
        id: Option<&str>,
    ) -> Result<Vec<(String, RecipeStatus, String)>, CliFailure> {
        let snapshot = self
            .snapshot()
            .map_err(|e| CliFailure::message(e.to_string()))?;
        if id.is_some_and(|id| {
            !snapshot
                .items
                .iter()
                .any(|i| i.recipe.id == id && i.manifest.is_some())
        }) {
            return Err(CliFailure::message(
                "mapset is not installed; use `ddl mapset list`",
            ));
        }
        Ok(snapshot
            .items
            .into_iter()
            .filter(|i| i.manifest.is_some() && id.is_none_or(|id| i.recipe.id == id))
            .map(|i| (i.recipe.id, i.recipe_status, i.change_summary))
            .collect())
    }
    fn recipe(&self, id: &str) -> Result<Recipe, Problem> {
        if let Some(active) = self.active(id)? {
            return Ok(active.recipe);
        }
        self.state
            .recipes
            .get(id)
            .or_else(|| self.catalog.recipe(id))
            .cloned()
            .ok_or(Problem::State)
    }
    pub fn snapshot(&self) -> Result<Snapshot, Problem> {
        #[cfg(test)]
        self.counters
            .snapshots
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
        for id in self.installed_ids() {
            if let Some(active) = self.active(&id)? {
                recipes.insert(id, active.recipe);
            }
        }
        // Tags are display metadata: an install snapshot taken before a recipe
        // was tagged still shows the catalog's current tags (launch data stays
        // frozen in the snapshot, D-019).
        for recipe in recipes.values_mut() {
            if let Some(current) = self.catalog.recipe(&recipe.id) {
                recipe.tags.clone_from(&current.tags);
            }
        }
        let held_profiles = self
            .profiles
            .list()
            .map_err(|e| logged(Problem::State, e))?;
        let mut items = vec![];
        for recipe in recipes.into_values() {
            let manifest = self.manifest(&recipe).map_err(install_error)?;
            let mut profiles: Vec<_> = held_profiles
                .iter()
                .filter(|p| p.recipe_id == recipe.id)
                .cloned()
                .collect();
            if profiles.is_empty() {
                profiles.push(default_profile(&recipe));
            }
            if let Some(active) = self.active(&recipe.id)?
                && let Some(old) = &active.previous_recipe
            {
                for profile in &mut profiles {
                    if active.pending_profiles.contains(&profile.id) {
                        reconcile_profile(profile, old, &active.recipe);
                    }
                }
            }
            let profile = selected_profile(&profiles, self.state.last_profiles.get(&recipe.id));
            let last_played = profiles.iter().filter_map(|p| p.last_played).max();
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
            let recipe_status = manifest.as_ref().map_or(RecipeStatus::Current, |m| {
                classify(
                    m,
                    &self.recipe(&recipe.id).unwrap_or_else(|_| recipe.clone()),
                    self.catalog.recipe(&recipe.id),
                )
            });
            let change_summary = recipe_status.summary(&recipe, self.catalog.recipe(&recipe.id));
            let update_dismissed = self
                .catalog
                .recipe(&recipe.id)
                .and_then(|r| recipe_content_sha256(r).ok())
                .is_some_and(|hash| self.state.kept_recipes.get(&recipe.id) == Some(&hash));
            let active = self.active(&recipe.id)?;
            let update_notice = active.as_ref().map_or_else(Vec::new, |a| a.notices.clone());
            items.push(Item {
                recipe_status,
                change_summary,
                update_dismissed,
                update_notice,
                favorite: self.state.favorites.contains(&recipe.id),
                recipe,
                manifest,
                profile,
                profiles,
                last_played,
                launch,
            });
        }
        let config = self.config.state().ok();
        let setups = self.custom.list().map_err(|e| logged(Problem::State, e))?;
        let mut local_issues: BTreeMap<String, (PathBuf, bool)> = setups
            .iter()
            .filter_map(|setup| {
                local::verify_setup(setup)
                    .into_iter()
                    .next()
                    .map(|issue| (setup.id.clone(), identity_issue(issue)))
            })
            .collect();
        let profiles_with_saves = held_profiles
            .iter()
            .filter(|p| {
                local::has_saves(
                    &ddl_core::profiles::ProfileDirs::new(&self.env.paths.data, &p.id).save,
                )
            })
            .map(|p| p.id.clone())
            .collect();
        let base_games: Vec<BaseItem> = iwads::base_games::list(&self.registry)
            .into_iter()
            .map(|base| {
                self.base_item(&base.id(), None)
                    .map_err(|e| logged(Problem::State, e))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for base in &base_games {
            if let Some(issue) = local::first_issue(&base.profile.extras) {
                local_issues.insert(base.base.id(), identity_issue(issue));
            }
        }
        Ok(Snapshot {
            base_games,
            automatic_folders: self.automatic_iwad_folders(),
            local_issues,
            profiles_with_saves,
            custom: setups,
            custom_profiles: held_profiles,
            items,
            tags: self.catalog.tags.clone(),
            previous: engines.previous_good,
            attribution: engine
                .as_ref()
                .and_then(|e| self.engine.attribution(&e.version).ok()),
            engine,
            iwads: self.registry.entries().to_vec(),
            folders: self.state.folders.clone(),
            hidden_paths: iwads::visibility::hidden_paths(
                &self.sandbox,
                &self.env.home,
                &self.state.folders,
            ),
            search_standard_folders: self.state.search_standard_folders,
            config_notice: config.as_ref().is_none_or(|s| {
                !s.conflicts.is_empty()
                    || s.template_version != self.config.template().template_version
                    || !self.config.base_path(&s.template_version).is_file()
            }),
            config_usable: config.is_some()
                && fs::read_to_string(self.config.active_path()).is_ok(),
            config_version: config.map_or_else(String::new, |s| s.template_version),
            audio: self.audio_state(),
            logs: self.env.paths.state.join("logs"),
            catalog: self.catalog_info(),
        })
    }

    /// Spec 067 R1/R2/R4: the Settings snapshot without a full font hash.
    ///
    /// `install_present` only checks the published pointer, reviewed sizes and
    /// symlinks. A selected Arachno font is verified through the metadata-keyed
    /// cache so a damaged file surfaces before launch; every launch still hashes
    /// again on the worker.
    fn audio_state(&self) -> AudioState {
        let preferences = self.soundfonts.preferences();
        let (choice, mut warning) = match &preferences {
            Ok(preferences) => (preferences.choice, None),
            Err(_) => (
                None,
                Some(soundfonts::UNREADABLE_PREFERENCES_WARNING.to_owned()),
            ),
        };
        if warning.is_none()
            && choice == Some(Choice::Arachno)
            && self.soundfonts.cached_font().is_err()
        {
            warning = Some(soundfonts::MISSING_FONT_WARNING.to_owned());
        }
        let current = fs::read_to_string(self.config.active_path())
            .map(|text| {
                soundfonts::engine::current(&ddl_core::dsda_config::format::Document::parse(&text))
            })
            .unwrap_or(soundfonts::engine::Current::Default);
        AudioState {
            choice,
            installed: self.soundfonts.install_present(),
            current,
            warning,
        }
    }

    /// Applies the persisted Audio choice just before the run guard is acquired
    /// (spec 067, R5), so every launch route uses it and no running engine is
    /// changed. `None` preserves whatever backend/font the engine holds.
    ///
    /// A missing/damaged Arachno install is an intentional Default fallback
    /// (`effective()` reports it in Settings), but a real write/path failure
    /// must abort the launch: starting the game anyway would silently use a
    /// different font or backend than the selection claims.
    fn prepare_audio(&self) -> Result<(), Problem> {
        let effective = self.soundfonts.effective();
        let Some(choice) = effective.choice else {
            return Ok(());
        };
        let keys = soundfonts::engine::keys_for(choice, effective.path.as_deref())
            .map_err(|error| logged(Problem::Audio, error))?;
        self.config
            .provision()
            .map_err(|error| logged(Problem::Audio, error))?;
        self.config
            .apply_audio(&keys)
            .map_err(|error| logged(Problem::Audio, error))?;
        Ok(())
    }

    /// Every launch route prepares the Audio choice before taking the run guard
    /// (spec 067, R5): a running engine is never changed, and no route can skip
    /// the application.
    fn begin_engine_run(&self) -> Result<ddl_core::dsda_config::RunGuard, RunStartError> {
        self.prepare_audio().map_err(RunStartError::Audio)?;
        self.config.run_guard().map_err(RunStartError::Guard)
    }

    /// The catalog in use and its automatic-update settings (spec 019, R5).
    #[must_use]
    pub fn catalog_info(&self) -> CatalogInfo {
        CatalogInfo {
            serial: self.catalog_origin.serial,
            updated: self.catalog_origin.updated,
            source_commit: self.catalog_origin.source_commit.clone(),
            fetched_at: self.catalog_origin.fetched_at,
            auto_update: self.state.catalog_auto_update,
            last_check: self.state.catalog_last_check,
        }
    }

    /// The bundle URL a run fetches: GitHub in production, the fixture's
    /// loopback URL under test (spec 019, R3).
    fn catalog_url(&self) -> String {
        #[cfg(test)]
        if let Some(test) = &self.env.test {
            return test.catalog_url.clone();
        }
        ddl_core::catalog::BUNDLE_URL.to_owned()
    }

    /// The bundle transport: the real download stack, relaxed to loopback HTTP
    /// only in tests (spec 019, R3).
    fn remote_fetch(&self) -> Box<dyn ddl_core::catalog::RemoteFetch> {
        #[cfg(test)]
        if self.env.test.is_some() {
            return Box::new(
                ddl_core::catalog::HttpsRemoteFetch::new(&self.env.paths.cache)
                    .allow_http_for_tests(),
            );
        }
        Box::new(ddl_core::catalog::HttpsRemoteFetch::new(
            &self.env.paths.cache,
        ))
    }

    /// The manual "Check for updates" (spec 019, R5): always runs, surfaces
    /// failures and records the attempt before touching the network.
    fn catalog_check(&mut self, cancel: &CancellationToken) -> Result<CatalogOutcome, Problem> {
        let now = now_unix_secs();
        if cancel.is_cancelled() {
            return Err(Problem::Cancelled);
        }
        self.state.catalog_last_check = Some(now);
        self.save()?;
        self.fetch_and_accept_catalog(now, cancel)
    }

    /// Spec 077, R1: the automatic check at every application startup.
    ///
    /// The network wait touches **no user state and no state lock**: the bundle
    /// is fetched and validated first, and only then is the attempt recorded
    /// with a fresh read-merge-write of `app.json`, so a profile or favourite
    /// saved while the request was in flight is never overwritten. A failure or
    /// "not newer" is silent (the current catalog stays); a newer bundle is
    /// cached and reported, and the UI adopts it through a fresh
    /// [`Command::RefreshCatalog`].
    pub fn startup_catalog_check(
        &mut self,
        cancel: &CancellationToken,
    ) -> Result<StartupCatalog, Problem> {
        if !self.state.catalog_auto_update {
            return Ok(StartupCatalog::default());
        }
        if cancel.is_cancelled() {
            return Err(Problem::Cancelled);
        }
        let now = now_unix_secs();
        match self.fetch_and_accept_catalog(now, cancel) {
            Ok(CatalogOutcome::Updated {
                serial,
                source_commit,
                dropped,
            }) => {
                tracing::info!(serial, dropped, "catalog updated at startup");
                let _ = source_commit;
                self.record_catalog_attempt(now)?;
                Ok(StartupCatalog {
                    updated: Some(serial),
                })
            }
            // Current (not newer) and an unusable bundle both keep the catalog.
            Ok(_) => {
                self.record_catalog_attempt(now)?;
                Ok(StartupCatalog::default())
            }
            Err(problem) => {
                // Silent on failure: keep the cached/built-in catalog.
                tracing::warn!(?problem, "startup catalog check failed");
                if self.record_catalog_attempt(now).is_err() {
                    tracing::warn!("could not record the catalog check attempt");
                }
                Ok(StartupCatalog::default())
            }
        }
    }

    /// Records the catalog attempt without clobbering newer user state
    /// (spec 077, R1): re-read `app.json`, change only `catalog_last_check` and
    /// write **that file only** under the short cross-process lock. A malformed
    /// or newer-schema `app.json` is left untouched (the in-memory state is
    /// kept) rather than being replaced by this instance's older copy, and
    /// `iwads.json` is never rewritten from a background instance.
    fn record_catalog_attempt(&mut self, now: u64) -> Result<(), Problem> {
        let _state_lock = self.lock_state()?;
        let path = self.env.paths.config.join("app.json");
        match fs::read(&path) {
            Ok(bytes) if bytes.len() <= 4 * 1024 * 1024 => match AppState::parse(&bytes) {
                Ok(parsed) => {
                    self.state = parsed;
                    self.state.catalog_last_check = Some(now);
                    save_json(&path, &self.state)
                }
                // Malformed or a newer schema: never overwrite it with this
                // instance's older copy.
                Err(_) => Ok(()),
            },
            Ok(_) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.state.catalog_last_check = Some(now);
                save_json(&path, &self.state)
            }
            Err(_) => Ok(()),
        }
    }

    /// The shared fetch/validate/accept step, with no user-state write.
    fn fetch_and_accept_catalog(
        &mut self,
        now: u64,
        cancel: &CancellationToken,
    ) -> Result<CatalogOutcome, Problem> {
        if cancel.is_cancelled() {
            return Err(Problem::Cancelled);
        }
        // Never let a bundle walk the catalog backwards: the floor is the
        // effective catalog's serial, which is at least the embedded one.
        let floor = self
            .catalog_origin
            .serial
            .max(ddl_core::catalog::embedded_serial());
        let allowlist = self.catalog_allowlist.clone();
        let url = self.catalog_url();
        let result = {
            let mut fetch = self.remote_fetch();
            ddl_core::catalog::load_remote(
                fetch.as_mut(),
                &url,
                &allowlist,
                ddl_core::catalog::EMBEDDED_HOSTS,
                floor,
                crate::cli::VERSION,
            )
        };
        match result {
            Ok(accepted) => {
                for skipped in accepted.dropped() {
                    tracing::warn!(file = %skipped.file, reason = %skipped.reason, "catalog bundle recipe dropped");
                }
                if accepted.catalog.recipes.is_empty() {
                    tracing::warn!(
                        "catalog bundle carried no usable recipe; keeping the current catalog"
                    );
                    return Ok(CatalogOutcome::Current {
                        serial: self.catalog_origin.serial,
                    });
                }
                // Spec 077, R1 review: a slower automatic response must never
                // roll back a newer catalog adopted while it was in flight — not
                // even by another process. Compare with the serial effective on
                // disk right now (cached bundle, this instance and the embedded
                // fallback), not only with this instance's origin.
                let cached = ddl_core::catalog::load_cache(&self.env.paths.data)
                    .map(|loaded| loaded.bundle.catalog_serial)
                    .unwrap_or(0);
                let effective = cached
                    .max(self.catalog_origin.serial)
                    .max(ddl_core::catalog::embedded_serial());
                if accepted.serial() <= effective {
                    tracing::info!(
                        serial = accepted.serial(),
                        effective = self.catalog_origin.serial,
                        "ignoring a catalog bundle older than the one in use"
                    );
                    return Ok(CatalogOutcome::Current { serial: effective });
                }
                ddl_core::catalog::store_cache(&self.env.paths.data, &accepted.bundle, now)
                    .map_err(|error| logged(Problem::Storage, error))?;
                let serial = accepted.serial();
                let source_commit = accepted.bundle.source_commit.clone();
                let dropped = accepted.dropped().len();
                self.catalog = accepted.catalog;
                self.catalog_origin = CatalogOrigin {
                    serial,
                    updated: true,
                    source_commit: source_commit.clone(),
                    fetched_at: Some(now),
                };
                Ok(CatalogOutcome::Updated {
                    serial,
                    source_commit,
                    dropped,
                })
            }
            // "Not newer" is a normal outcome for a manual check, not a failure.
            Err(ddl_core::catalog::CatalogUpdateError::Rejected(
                ddl_core::catalog::BundleRejection::NotNewer { .. },
            )) => Ok(CatalogOutcome::Current {
                serial: self.catalog_origin.serial,
            }),
            Err(error) => Err(logged(Problem::Catalog, error)),
        }
    }

    /// Executes one command and reports how the UI should refresh (spec 075,
    /// R1/R2): a full snapshot, or nothing when the command already reported
    /// the one value that changed.
    pub fn execute(
        &mut self,
        command: Command,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(Event),
    ) -> Result<Refresh, Problem> {
        if let Command::SetPistolStart {
            recipe_id,
            profile_id,
            value,
        } = &command
        {
            return self.set_pistol_start(recipe_id, profile_id, *value);
        }
        if let Command::Local(LocalCommand::BasePistolStart { profile_id, value }) = &command {
            return self.set_base_pistol_start(profile_id, *value);
        }
        self.execute_full(command, cancel, emit)
            .map(|()| Refresh::Full)
    }

    /// The whole-command path: every command that needs a snapshot afterwards.
    fn execute_full(
        &mut self,
        command: Command,
        cancel: &CancellationToken,
        emit: &mut dyn FnMut(Event),
    ) -> Result<(), Problem> {
        if cancel.is_cancelled() {
            return Err(Problem::Cancelled);
        }
        if let Command::Local(operation) = command {
            return self.execute_local(operation, cancel, emit);
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
            Command::Local(_) => unreachable!(),
            Command::Startup => {
                if let Err(error) = self
                    .config
                    .provision()
                    .and_then(|_| self.config.ensure_ready().map(|_| ()))
                {
                    logged(Problem::Config, error);
                }

                self.cleanup_retired();
                self.finish_uninstalls()?;
                self.adopt_metadata()?;
                for id in self.installed_ids() {
                    self.reconcile_install(&id)?;
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
            Command::SetCatalogAutoUpdate(enabled) => {
                self.state.catalog_auto_update = enabled;
                self.save()?;
            }
            // Spec 067 R2: verification and atomic publication happen on this
            // worker; the download never touches the current choice.
            Command::InstallSoundfont => {
                self.soundfonts
                    .install(cancel, &mut |p| emit(Event::Progress(p)))
                    .map_err(|e| logged(soundfont_problem(e.kind()), e))?;
            }
            Command::SelectSoundfont(choice) => {
                self.soundfonts
                    .select(choice)
                    .map_err(|e| logged(soundfont_problem(e.kind()), e))?;
            }
            Command::CatalogCheck => {
                let outcome = self.catalog_check(cancel)?;
                emit(Event::Catalog(outcome));
            }
            // Spec 077, R1: a background adoption only needs the fresh view.
            Command::RefreshCatalog => {}
            Command::SetPistolStart { .. } => {
                unreachable!("the light Pistol start path returns before this match")
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
                self.install_content(
                    &recipe,
                    &p.components.iter().map(String::as_str).collect::<Vec<_>>(),
                    cancel,
                    &mut |p| emit(Event::Progress(p)),
                )
                .map_err(install_error)?;
                self.state.recipes.insert(id, recipe);
                self.save()?;
            }
            Command::UpdateMapset(id) => {
                self.update_mapset_inner(&id, cancel, &mut |p| emit(Event::Progress(p)))?;
            }
            Command::Uninstall(id, delete_profiles) => {
                self.uninstall_mapset_inner(&id, delete_profiles)?;
            }
            Command::KeepCurrent(id) => {
                let recipe = self.catalog.recipe(&id).ok_or(Problem::State)?;
                let hash = recipe_content_sha256(recipe).map_err(install_error)?;
                self.state.kept_recipes.insert(id, hash);
                self.save()?;
            }
            Command::Save(profile) => {
                profile
                    .validate(&self.recipe(&profile.recipe_id)?)
                    .map_err(|e| logged(Problem::State, e))?;
                self.profiles
                    .save(&profile)
                    .map_err(|e| logged(Problem::Storage, e))?;
                self.state
                    .last_profiles
                    .insert(profile.recipe_id.clone(), profile.id.clone());
                self.save()?;
            }
            Command::SelectProfile(recipe_id, id) => {
                let profile = self
                    .profiles
                    .load(&id)
                    .map_err(|e| logged(Problem::State, e))?;
                if profile.recipe_id != recipe_id {
                    return Err(Problem::State);
                }
                self.state.last_profiles.insert(recipe_id, id);
                self.save()?;
            }
            Command::NewProfile(recipe_id, copy) => {
                let recipe = self.recipe(&recipe_id)?;
                let current = self.profile(&recipe)?;
                self.profiles
                    .save(&current)
                    .map_err(|e| logged(Problem::Storage, e))?;
                let all = self
                    .profiles
                    .list()
                    .map_err(|e| logged(Problem::State, e))?;
                let n = (2..)
                    .find(|n| {
                        !all.iter().any(|p| {
                            p.recipe_id == recipe_id
                                && p.name.to_lowercase() == format!("profile {n}")
                        })
                    })
                    .ok_or(Problem::State)?;
                let suffix = (2..)
                    .find(|n| {
                        let id = format!("{recipe_id}-profile-{n}");
                        !all.iter().any(|p| p.id == id)
                            && self.profiles.path(&id).is_ok_and(|p| !p.exists())
                            && !self.profiles.dirs(&id).root.exists()
                    })
                    .ok_or(Problem::State)?;
                let mut profile = if copy {
                    current
                } else {
                    default_profile(&recipe)
                };
                profile.id = format!("{recipe_id}-profile-{suffix}");
                profile.name = format!("Profile {n}");
                profile.last_played = None;
                profile
                    .validate(&recipe)
                    .map_err(|e| logged(Problem::State, e))?;
                self.profiles
                    .save(&profile)
                    .map_err(|e| logged(Problem::Storage, e))?;
                self.state.last_profiles.insert(recipe_id, profile.id);
                self.save()?;
            }
            Command::RenameProfile(id, name) => {
                let mut profile = self
                    .profiles
                    .load(&id)
                    .map_err(|e| logged(Problem::State, e))?;
                let all = self
                    .profiles
                    .list()
                    .map_err(|e| logged(Problem::State, e))?;
                profile.name = ddl_core::profiles::validate_name(
                    &name,
                    &profile.recipe_id,
                    Some(&profile.id),
                    &all,
                )
                .map_err(|e| logged(Problem::State, e))?;
                self.profiles
                    .save(&profile)
                    .map_err(|e| logged(Problem::Storage, e))?;
            }
            Command::DeleteProfile(id) => {
                self.profiles
                    .delete_with_data(&id)
                    .map_err(|e| logged(Problem::Storage, e))?;
                self.state.last_profiles.retain(|_, held| held != &id);
                self.save()?;
            }
            Command::Favorite(id) => {
                self.recipe(&id)?;
                if !self.state.favorites.remove(&id) {
                    self.state.favorites.insert(id);
                }
                self.save()?;
            }
            Command::Play(id) => {
                {
                    let _lock = self.lock_state()?;
                    self.reconcile_install(&id)?;
                }
                let recipe = self.recipe(&id)?;
                let mut profile = self.profile(&recipe)?;
                let guard = self
                    .begin_engine_run()
                    .map_err(RunStartError::into_problem)?;
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
                state_recovery::preserve_config_before_reset(&self.config)?;
                self.config
                    .reset(true)
                    .map_err(|e| logged(Problem::Config, e))?;
            }
            Command::Import(id, component, path) => {
                let recipe = self.recipe(&id)?;
                // Select the component before staging so an initial SHREDS
                // import also fetches its required free COMPAT maps. The profile
                // is saved only after the import succeeds.
                let mut profile = self.profile(&recipe)?;
                enable_component(&recipe, &mut profile, &component);
                let outcome = if let Some(mut active) = self.active(&id)? {
                    let previous = active.manifest.clone();
                    // The file being replaced may be missing or corrupt. Drop
                    // its old import from the staging base so stage_update does
                    // not reject the recovery before the replacement is added.
                    let mut stage_base = previous.clone();
                    stage_base.members.retain(|member| {
                        !(member.package.is_none()
                            && member.component.as_deref() == Some(component.as_str()))
                    });
                    let mut staged = self
                        .installer
                        .stage_update(
                            &recipe,
                            &profile
                                .components
                                .iter()
                                .map(String::as_str)
                                .collect::<Vec<_>>(),
                            &stage_base,
                            cancel,
                            &mut |_| {},
                        )
                        .map_err(install_error)?;
                    let mut unpublished = UnpublishedGeneration(Some(
                        self.installer
                            .manifest_dir(&staged)
                            .map_err(install_error)?,
                    ));
                    for profile in self
                        .profiles
                        .list()
                        .map_err(|e| logged(Problem::State, e))?
                        .iter()
                        .filter(|p| p.recipe_id == id)
                    {
                        staged = self
                            .installer
                            .extend_install(
                                &recipe,
                                &profile
                                    .components
                                    .iter()
                                    .map(String::as_str)
                                    .collect::<Vec<_>>(),
                                &staged,
                                cancel,
                                &mut |_| {},
                            )
                            .map_err(install_error)?;
                    }
                    let (outcome, manifest) = self
                        .installer
                        .import_into_manifest(&recipe, &component, &path, cancel, &staged)
                        .map_err(install_error)?;
                    self.installer
                        .verify_staged(&manifest, cancel)
                        .map_err(install_error)?;
                    active.retired_dirs.push(
                        self.installer
                            .manifest_dir(&previous)
                            .map_err(install_error)?,
                    );
                    active.manifest = manifest;
                    self.installer.publish(&active).map_err(install_error)?;
                    unpublished.0 = None;
                    self.cleanup_retired();
                    outcome
                } else {
                    self.installer
                        .import_user_component_with_cancel(&recipe, &component, &path, cancel)
                        .map_err(install_error)?
                };
                if !outcome.matched {
                    tracing::warn!(sha256=%outcome.sha256,component,"unrecognized imported component was stored but not enabled");
                    return Err(Problem::UnknownFile);
                }
                self.profiles
                    .save(&profile)
                    .map_err(|e| logged(Problem::Storage, e))?;
            }
        }
        Ok(())
    }
    fn profile(&self, recipe: &Recipe) -> Result<Profile, Problem> {
        let profiles: Vec<_> = self
            .profiles
            .list()
            .map_err(|e| logged(Problem::State, e))?
            .into_iter()
            .filter(|p| p.recipe_id == recipe.id)
            .collect();
        if profiles.is_empty() {
            return Ok(default_profile(recipe));
        }
        Ok(selected_profile(
            &profiles,
            self.state.last_profiles.get(&recipe.id),
        ))
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
        #[cfg(test)]
        self.counters
            .resolves
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let manifest = match self.manifest(recipe) {
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
        .map(|resolution| resolution.with_screen(self.env.screen))
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
    /// The problem for a folder that cannot be opened. Inside the sandbox a
    /// missing or refused folder is most likely the host's, not the player's
    /// mistake, so say that instead of a generic storage error (spec 059 R4).
    fn unreadable_directory(&self, path: &Path, error: io::Error) -> Problem {
        if self.sandbox.is_flatpak() && iwads::visibility::access(path).is_hidden() {
            tracing::info!(path = %path.display(), %error, "folder not visible to the Flatpak");
            return Problem::NotVisible(path.to_path_buf());
        }
        logged(Problem::Storage, error)
    }
    fn browse(
        &self,
        path: &Path,
        files: bool,
        cancel: &CancellationToken,
    ) -> Result<Directory, Problem> {
        let mut entries = vec![];
        for entry in fs::read_dir(path)
            .map_err(|e| self.unreadable_directory(path, e))?
            .take(4096)
        {
            if cancel.is_cancelled() {
                return Err(Problem::Cancelled);
            }
            let entry = entry.map_err(|e| logged(Problem::Storage, e))?;
            let p = entry.path();
            let Ok(metadata) = fs::metadata(&p) else {
                continue;
            };
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
            files: vec![],
            iwad_files: BTreeSet::new(),
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

/// The result of uninstalling a mapset (spec 022, R2/R3).
#[derive(Debug, Clone)]
pub struct UninstallOutcome {
    /// Recipe id.
    pub id: String,
    /// Bytes the install occupied, for the CLI's one-line summary.
    pub bytes: u64,
    /// Profiles deleted with it (zero when they were kept).
    pub profiles: usize,
    /// Whether a user-imported component (D-014) was removed too.
    pub imported: bool,
}

/// The result of `ddl iwad scan` (spec 014, R1).#[derive(Debug, Clone)]
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
    pub profile: Option<String>,
    /// Optional component ids to enable (`--with`).
    pub components: Vec<String>,
    /// Starting skill override (`--skill`).
    pub skill: Option<u8>,
    /// Reset to a pistol start on every map (`--pistol-start`).
    pub pistol_start: bool,
}

/// The base game a new custom setup launches with (spec 026, R2/R7).
#[derive(Debug, Clone)]
pub enum NewIwad {
    /// An identified IWAD variant id from spec 004's registry.
    Variant(String),
    /// A player file whose header is `IWAD`.
    File(PathBuf),
}

/// The result of changing a profile's extras (spec 026, R5): the saved profile
/// and whether the change may strand existing saves.
#[derive(Debug, Clone)]
pub struct ExtrasOutcome {
    /// The saved profile.
    pub profile: Profile,
    /// Whether the profile already had save slots, so the CLI warns (R5).
    pub saves_warn: bool,
}

/// A resolved custom launch (spec 026, R2/R3): the plan and the profile it used.
#[derive(Debug, Clone)]
pub struct CustomLaunch {
    /// The prepared DSDA-Doom launch plan (spec 003).
    pub plan: dsda::LaunchPlan,
    /// The profile whose save/data directories the plan uses.
    pub profile: Profile,
}

/// The data behind `ddl doctor` (spec 014, R1).
#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub profiles: BTreeMap<String, Vec<Profile>>,
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

    /// `ddl catalog status`: the catalog in use, its serial date and source.
    /// Reads only local state, never the network.
    #[must_use]
    pub fn catalog_status(&self) -> CatalogInfo {
        self.catalog_info()
    }

    /// `ddl catalog update`: fetch and accept a newer bundle now (spec 019, R5).
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for a busy state lock, a fetch/rejection failure
    /// or a filesystem failure.
    pub fn catalog_update(&mut self) -> Result<CatalogOutcome, CliFailure> {
        let _lock = self.lock_cli()?;
        self.catalog_check(&CancellationToken::new())
            .map_err(|problem| CliFailure::message(problem.to_string()))
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
            let recipe = self
                .recipe(&recipe.id)
                .map_err(|e| CliFailure::message(e.to_string()))?;
            let installed = self
                .manifest(&recipe)
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
            }
            profile.components = recipe.explicit_selection(components).map_err(|error| {
                CliFailure::message(format!("invalid component selection: {error}"))
            })?;
        }
        profile.validate(&recipe).map_err(|error| {
            CliFailure::message(format!("invalid component selection: {error}"))
        })?;
        self.profiles
            .save(&profile)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        let enabled: Vec<&str> = profile.components.iter().map(String::as_str).collect();
        let manifest = self
            .install_content(&recipe, &enabled, &CancellationToken::new(), progress)
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
            .snapshot()
            .map_err(|e| CliFailure::message(e.to_string()))?
            .items
            .into_iter()
            .find(|i| i.recipe.id == recipe.id)
            .ok_or_else(|| CliFailure::message("unknown mapset"))?
            .profile;
        if let Some(selector) = &overrides.profile {
            let snapshot = self
                .snapshot()
                .map_err(|e| CliFailure::message(e.to_string()))?;
            let item = snapshot
                .items
                .iter()
                .find(|i| i.recipe.id == recipe.id)
                .ok_or_else(|| CliFailure::message("unknown mapset"))?;
            profile = if let Some(exact) = item.profiles.iter().find(|p| &p.id == selector) {
                exact.clone()
            } else {
                let matches: Vec<_> = item
                    .profiles
                    .iter()
                    .filter(|p| &p.name == selector)
                    .collect();
                match matches.as_slice() {
                    [one] => (*one).clone(),
                    [] => {
                        return Err(CliFailure::message(format!(
                            "no profile `{selector}` for `{}`",
                            recipe.id
                        )));
                    }
                    _ => {
                        return Err(CliFailure::message(format!(
                            "ambiguous profile `{selector}`; use its id"
                        )));
                    }
                }
            };
        }
        if !overrides.components.is_empty() {
            for component in &overrides.components {
                if !recipe
                    .components
                    .iter()
                    .any(|candidate| &candidate.id == component && candidate.optional)
                {
                    return Err(CliFailure::message(format!(
                        "`{}` has no optional component `{component}`",
                        recipe.id
                    )));
                }
                // A user_supplied component is allowed here: it may have been
                // imported already, and the resolver reports it if not.
            }
            profile.components =
                recipe
                    .explicit_selection(&overrides.components)
                    .map_err(|error| {
                        CliFailure::message(format!("invalid launch selection: {error}"))
                    })?;
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
    /// the UI. The selected profile and play history are remembered; per-launch
    /// option overrides are not saved.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for a typed blocker, a busy run guard, or a
    /// launch failure.
    pub fn run_mapset(
        &mut self,
        id: &str,
        overrides: &LaunchOverrides,
    ) -> Result<dsda::LaunchOutcome, CliFailure> {
        {
            let _lock = self.lock_cli()?;
            self.reconcile_install(id)
                .map_err(|e| CliFailure::message(e.to_string()))?;
        }
        let recipe = self
            .recipe(id)
            .map_err(|_| CliFailure::message(format!("no catalog recipe with id `{id}`")))?;
        let profile = self.effective_profile(&recipe, overrides)?;
        let guard = self
            .begin_engine_run()
            .map_err(|error| CliFailure::message(error.to_string()))?;
        let resolution = self
            .resolve_launch(&recipe, &profile)
            .map_err(CliFailure::from)?;
        // Save only the selected profile's original settings, never CLI overrides.
        // The cross-process state lock is short-lived; the run guard owns the game.
        {
            let _lock = self.lock_cli()?;
            let mut saved = self
                .profiles
                .load_optional(&profile.id)
                .map_err(|e| CliFailure::message(e.to_string()))?
                .unwrap_or_else(|| default_profile(&recipe));
            saved.touch(dsda::Clock::now_unix_secs(&dsda::SystemClock));
            self.profiles
                .save(&saved)
                .map_err(|e| CliFailure::message(e.to_string()))?;
            self.state
                .last_profiles
                .insert(recipe.id.clone(), profile.id.clone());
            self.save()
                .map_err(|e| CliFailure::message(e.to_string()))?;
        }
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
            profiles: snapshot
                .items
                .iter()
                .map(|i| (i.recipe.id.clone(), i.profiles.clone()))
                .chain(
                    snapshot
                        .base_games
                        .iter()
                        .map(|b| (b.base.id(), b.profiles.clone())),
                )
                .collect(),
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

    // ---- spec 032: original games, without recipes or installs -----------
    fn free_base_profile_id(&self, game: iwads::Game) -> Result<String, CliFailure> {
        for n in 1.. {
            let id = format!("base-{game}-profile-{n}");
            if !self
                .profiles
                .path(&id)
                .map_err(|e| CliFailure::message(e.to_string()))?
                .exists()
                && !self.profiles.dirs(&id).root.exists()
            {
                return Ok(id);
            }
        }
        unreachable!()
    }
    fn fresh_base_profile(
        &self,
        base: &iwads::base_games::BaseGame,
    ) -> Result<Profile, CliFailure> {
        Ok(Profile::new(
            self.free_base_profile_id(base.game)?,
            "Default",
            base.id(),
        ))
    }
    pub fn base_item(&self, id: &str, selector: Option<&str>) -> Result<BaseItem, CliFailure> {
        let base = iwads::base_games::list(&self.registry)
            .into_iter()
            .find(|b| b.id() == id)
            .ok_or_else(|| {
                CliFailure::message(format!(
                    "no identified base game `{id}`; rescan IWAD folders"
                ))
            })?;
        let mut profiles: Vec<_> = self
            .profiles
            .list()
            .map_err(|e| CliFailure::message(e.to_string()))?
            .into_iter()
            .filter(|p| p.recipe_id == id)
            .collect();
        if profiles.is_empty() {
            profiles.push(self.fresh_base_profile(&base)?);
        }
        let profile = if let Some(selector) = selector {
            let exact = profiles.iter().find(|p| p.id == selector);
            let names: Vec<_> = profiles.iter().filter(|p| p.name == selector).collect();
            exact
                .or_else(|| {
                    if names.len() == 1 {
                        names.first().copied()
                    } else {
                        None
                    }
                })
                .cloned()
                .ok_or_else(|| {
                    CliFailure::message(format!(
                        "unknown or ambiguous profile `{selector}` for `{id}`"
                    ))
                })?
        } else {
            selected_profile(&profiles, self.state.last_profiles.get(id))
        };
        let last_played = profiles.iter().filter_map(|p| p.last_played).max();
        Ok(BaseItem {
            base,
            profile,
            profiles,
            last_played,
        })
    }
    pub fn base_copy_notice(&self, item: &BaseItem) -> Option<&'static str> {
        item.copy_notice(self.registry.entries())
    }
    pub fn base_resolution(
        &mut self,
        id: &str,
        overrides: &LaunchOverrides,
        accept_changes: bool,
    ) -> Result<CustomLaunch, CliFailure> {
        if !overrides.components.is_empty() {
            return Err(CliFailure::message(
                "base games have no components; drop `--with`",
            ));
        }
        if accept_changes {
            self.accept_extra_changes(id, overrides.profile.as_deref())?;
        }
        let item = self.base_item(id, overrides.profile.as_deref())?;
        let mut profile = item.profile;
        if let Some(issue) = local::first_issue(&profile.extras) {
            return Err(CliFailure::message(format!(
                "cannot launch `{id}`: {issue}"
            )));
        }
        if let Some(skill) = overrides.skill {
            profile.skill = skill;
        }
        if overrides.pistol_start {
            profile.pistol_start = true;
        }
        let engine = self
            .engine
            .active_engine()
            .map_err(|e| CliFailure::message(e.to_string()))?
            .ok_or_else(|| CliFailure::message("DSDA-Doom is not installed"))?;
        let plan = iwads::base_games::build_plan(
            &item.base,
            &profile,
            &local::Engine {
                executable: engine.path,
                version: engine.version,
            },
            &self.registry,
            &self.env.paths.data,
            &self.config,
        )
        .map_err(|e| CliFailure::message(e.to_string()))?;
        let plan = plan_with_screen(plan, self.env.screen);
        plan.prepare_dirs()
            .map_err(|e| CliFailure::message(e.to_string()))?;
        plan.validate(&local::assign_allowlist())
            .map_err(|e| CliFailure::message(e.to_string()))?;
        Ok(CustomLaunch { plan, profile })
    }
    pub fn run_base(
        &mut self,
        id: &str,
        overrides: &LaunchOverrides,
        accept_changes: bool,
    ) -> Result<dsda::LaunchOutcome, CliFailure> {
        let launch = self.base_resolution(id, overrides, accept_changes)?;
        let guard = self
            .begin_engine_run()
            .map_err(|e| CliFailure::message(e.to_string()))?;
        {
            let _lock = self.lock_cli()?;
            let mut saved = self.base_item(id, overrides.profile.as_deref())?.profile;
            saved.touch(now_unix_secs());
            self.profiles
                .save(&saved)
                .map_err(|e| CliFailure::message(e.to_string()))?;
            self.state.last_profiles.insert(id.into(), saved.id);
            self.save()
                .map_err(|e| CliFailure::message(e.to_string()))?;
        }
        let outcome = self.run_plan(&launch.plan)?;
        guard
            .end()
            .map_err(|e| CliFailure::message(e.to_string()))?;
        Ok(outcome)
    }
    fn automatic_iwad_folders(&self) -> Vec<IwadFolder> {
        let mut sources: Vec<_> = iwads::steam::source_folders(&self.env.home)
            .into_iter()
            .map(|p| (p, iwads::Source::Steam))
            .collect();
        sources.extend(
            iwads::stores::discover(&self.env.home)
                .folders
                .into_iter()
                .map(|(p, launcher)| (p, store_source(launcher))),
        );
        if self.state.search_standard_folders {
            sources.extend(
                iwads::stores::standard_folders(&self.env.home, &|key| std::env::var(key).ok())
                    .into_iter()
                    .map(|p| (p, iwads::Source::StandardFolder)),
            );
        }
        // Also retain identified automatic folders whose launcher record disappeared.
        sources.extend(
            self.registry
                .entries()
                .iter()
                .filter(|e| !matches!(e.source, iwads::Source::Manual | iwads::Source::UserFolder))
                .filter_map(|e| e.path.parent().map(|p| (p.to_path_buf(), e.source))),
        );
        let mut folders: Vec<IwadFolder> = Vec::new();
        for (path, source) in sources {
            if folders.iter().any(|f| f.path == path && f.source == source) {
                continue;
            }
            let mut games: Vec<_> = iwads::base_games::list(&self.registry)
                .iter()
                .filter(|b| {
                    self.registry.entries().iter().any(|e| {
                        e.game == Some(b.game)
                            && e.path.is_file()
                            && e.source == source
                            && e.path.parent() == Some(path.as_path())
                    })
                })
                .map(|b| b.title().to_owned())
                .collect();
            games.sort();
            games.dedup();
            folders.push(IwadFolder {
                path,
                source,
                games,
            });
        }
        folders.sort_by(|a, b| {
            a.source
                .label()
                .cmp(b.source.label())
                .then(a.path.cmp(&b.path))
        });
        folders
    }

    // ---- spec 026: local content (custom setups and extra files) ----------

    /// `ddl custom list`: every custom setup, in id order.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] when the setup directory cannot be read.
    pub fn custom_list(&self) -> Result<Vec<CustomSetup>, CliFailure> {
        self.custom
            .list()
            .map_err(|error| CliFailure::message(error.to_string()))
    }

    /// `ddl custom show`: one setup with its recorded user files.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown or unreadable setup.
    pub fn custom_show(&self, id: &str) -> Result<CustomSetup, CliFailure> {
        self.custom
            .load_optional(id)
            .map_err(|error| CliFailure::message(error.to_string()))?
            .ok_or_else(|| CliFailure::message(format!("no custom setup with id `{id}`")))
    }

    /// Whether `id` names a custom setup (spec 026, R7).
    #[must_use]
    pub fn has_custom(&self, id: &str) -> bool {
        self.custom.contains(id)
    }

    /// Whether `id` names a catalog recipe.
    #[must_use]
    pub fn has_recipe(&self, id: &str) -> bool {
        self.recipe(id).is_ok()
    }

    /// `ddl custom create`: a new setup with its default profile.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for a duplicate or unsafe id, a file that is
    /// not an IWAD, or a storage failure.
    pub fn custom_create(
        &mut self,
        name: &str,
        id: Option<&str>,
        iwad: NewIwad,
        compatibility: Compatibility,
    ) -> Result<CustomSetup, CliFailure> {
        let _lock = self.lock_cli()?;
        let id = match id {
            Some(id) => {
                if !local::safe_id(id) {
                    return Err(CliFailure::message(format!(
                        "`{id}` is not a safe setup id (lowercase letters, digits and `-` only)"
                    )));
                }
                if self.custom.contains(id) {
                    return Err(CliFailure::message(format!(
                        "a custom setup `{id}` already exists"
                    )));
                }
                id.to_owned()
            }
            None => self.custom.free_id(name),
        };
        let iwad = match iwad {
            NewIwad::Variant(variant) => CustomIwad::Identified { variant },
            NewIwad::File(path) => {
                let path = absolute_path(&path)?;
                let file = UserFile::read(&path)
                    .map_err(|error| CliFailure::message(error.to_string()))?;
                if file.kind != local::Kind::Wad || !local::is_iwad_header(&path).unwrap_or(false) {
                    return Err(CliFailure::message(format!(
                        "`{}` is not an IWAD: its header is not `IWAD`",
                        path.display()
                    )));
                }
                CustomIwad::File { file }
            }
        };
        let setup = CustomSetup::new(id.clone(), name, iwad, compatibility, now_unix_secs());
        setup
            .validate()
            .map_err(|error| CliFailure::message(error.to_string()))?;
        self.custom
            .save(&setup)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        // A default profile, so the setup launches with isolated saves like a
        // catalog mapset (spec 026, R2/R6).
        let profile = Profile::new(format!("{id}-default"), "Default", &id);
        self.profiles
            .save(&profile)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        Ok(setup)
    }

    /// `ddl custom add-file`: append one or more user files, routing each by its
    /// detected kind (spec 026, R7).
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown setup, a rejected file, a
    /// duplicate path or a storage failure.
    pub fn custom_add_file(
        &mut self,
        id: &str,
        paths: &[PathBuf],
    ) -> Result<CustomSetup, CliFailure> {
        let _lock = self.lock_cli()?;
        let mut setup = self.custom_show(id)?;
        for path in paths {
            let path = absolute_path(path)?;
            let file =
                UserFile::read(&path).map_err(|error| CliFailure::message(error.to_string()))?;
            setup
                .add(file)
                .map_err(|error| CliFailure::message(error.to_string()))?;
        }
        self.save_custom(setup)
    }

    /// `ddl custom move`: reorder one entry inside a group (spec 026, R7).
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown setup, an out-of-range index or a
    /// storage failure.
    pub fn custom_move(
        &mut self,
        id: &str,
        group: ExtraGroup,
        from: usize,
        to: usize,
    ) -> Result<CustomSetup, CliFailure> {
        let _lock = self.lock_cli()?;
        let mut setup = self.custom_show(id)?;
        setup
            .move_entry(group, from, to)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        self.save_custom(setup)
    }

    /// `ddl custom remove-file`: drop one entry from a group. The file on disk is
    /// never touched (spec 026, R6).
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown setup, an out-of-range index or a
    /// storage failure.
    pub fn custom_remove_file(
        &mut self,
        id: &str,
        group: ExtraGroup,
        index: usize,
    ) -> Result<CustomSetup, CliFailure> {
        let _lock = self.lock_cli()?;
        let mut setup = self.custom_show(id)?;
        setup
            .remove(group, index)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        self.save_custom(setup)
    }

    /// `ddl custom delete`: remove DDL's record, and with `delete_profiles` its
    /// profiles and saves (spec 026, R6; D-022). Player files are untouched.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown setup or a filesystem failure.
    pub fn custom_delete(&mut self, id: &str, delete_profiles: bool) -> Result<(), CliFailure> {
        let _lock = self.lock_cli()?;
        if self
            .custom
            .load_optional(id)
            .map_err(|error| CliFailure::message(error.to_string()))?
            .is_none()
        {
            return Err(CliFailure::message(format!(
                "no custom setup with id `{id}`"
            )));
        }
        let profile_ids: Vec<String> = self
            .profiles
            .list()
            .map_err(|error| CliFailure::message(error.to_string()))?
            .into_iter()
            .filter(|profile| profile.recipe_id == id)
            .map(|profile| profile.id)
            .collect();
        self.custom
            .delete(id)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        if delete_profiles {
            for profile_id in profile_ids {
                self.profiles
                    .delete_fully(&profile_id)
                    .map_err(|error| CliFailure::message(error.to_string()))?;
            }
        }
        Ok(())
    }

    /// Resolves a custom setup into a launch plan, re-recording changed files
    /// first when the player confirms them (spec 026, R4).
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown setup or profile, a missing or
    /// changed user file, a missing engine or an unusable IWAD.
    pub fn custom_resolution(
        &mut self,
        id: &str,
        profile_selector: Option<&str>,
        accept_changes: bool,
    ) -> Result<CustomLaunch, CliFailure> {
        let mut setup = self.custom_show(id)?;
        if accept_changes {
            let files = local::accept_changed(&mut setup.files)
                .map_err(|error| CliFailure::message(error.to_string()))?;
            let patches = local::accept_changed(&mut setup.patches)
                .map_err(|error| CliFailure::message(error.to_string()))?;
            if !files.is_empty() || !patches.is_empty() {
                let _lock = self.lock_cli()?;
                setup.updated_at = now_unix_secs();
                self.custom
                    .save(&setup)
                    .map_err(|error| CliFailure::message(error.to_string()))?;
            }
        }
        let profile = self.custom_profile(&setup, profile_selector)?;
        // Spec 026, R4: check every user file before anything else, so a
        // missing/changed file is named even when no engine is installed.
        if let Some(issue) = local::verify_setup(&setup).into_iter().next() {
            return Err(CliFailure::message(format!(
                "cannot launch `{id}`: {issue}"
            )));
        }
        let engine = self
            .engine
            .active_engine()
            .map_err(|error| CliFailure::message(error.to_string()))?
            .ok_or_else(|| CliFailure::message("DSDA-Doom is not installed"))?;
        let engine = local::Engine {
            executable: engine.path,
            version: engine.version,
        };
        let plan = local::build_plan(
            &setup,
            &profile,
            &engine,
            &self.registry,
            &self.env.paths.data,
            &self.config,
        )
        .map_err(|error| CliFailure::message(error.to_string()))?;
        let plan = plan_with_screen(plan, self.env.screen);
        plan.prepare_dirs()
            .map_err(|error| CliFailure::message(error.to_string()))?;
        plan.validate(&local::assign_allowlist())
            .map_err(|error| CliFailure::message(error.to_string()))?;
        Ok(CustomLaunch { plan, profile })
    }

    /// `ddl launch <custom id>` without `--dry-run`: launch through the same run
    /// guard as the UI, remembering the selected profile.
    ///
    /// # Errors
    ///
    /// As [`Services::custom_resolution`], plus a busy run guard or a launch
    /// failure.
    pub fn run_custom(
        &mut self,
        id: &str,
        profile_selector: Option<&str>,
        accept_changes: bool,
    ) -> Result<dsda::LaunchOutcome, CliFailure> {
        let custom = self.custom_resolution(id, profile_selector, accept_changes)?;
        let guard = self
            .begin_engine_run()
            .map_err(|error| CliFailure::message(error.to_string()))?;
        {
            let _lock = self.lock_cli()?;
            let mut saved = self
                .profiles
                .load_optional(&custom.profile.id)
                .map_err(|error| CliFailure::message(error.to_string()))?
                .unwrap_or_else(|| custom.profile.clone());
            saved.touch(dsda::Clock::now_unix_secs(&dsda::SystemClock));
            self.profiles
                .save(&saved)
                .map_err(|error| CliFailure::message(error.to_string()))?;
        }
        let outcome = self.run_plan(&custom.plan)?;
        guard
            .end()
            .map_err(|error| CliFailure::message(error.to_string()))?;
        Ok(outcome)
    }

    /// `ddl profile extras … list`: the selected profile with its extras.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown recipe or profile.
    pub fn extras_list(
        &self,
        recipe_id: &str,
        selector: Option<&str>,
    ) -> Result<Profile, CliFailure> {
        let profile = self.extras_profile(recipe_id, selector)?;
        Ok(profile)
    }

    /// `ddl profile extras … add`: append user files to the selected profile,
    /// after the recipe's own files (spec 026, R5).
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown recipe or profile, a rejected or
    /// duplicate file, or a storage failure.
    pub fn extras_add(
        &mut self,
        recipe_id: &str,
        selector: Option<&str>,
        paths: &[PathBuf],
    ) -> Result<ExtrasOutcome, CliFailure> {
        let _lock = self.lock_cli()?;
        let mut profile = self.extras_profile(recipe_id, selector)?;
        for path in paths {
            let path = absolute_path(path)?;
            let file =
                UserFile::read(&path).map_err(|error| CliFailure::message(error.to_string()))?;
            if profile.extras.iter().any(|held| held.path == file.path) {
                return Err(CliFailure::message(format!(
                    "`{}` is already an extra of this profile",
                    file.path.display()
                )));
            }
            profile.extras.push(file);
        }
        self.save_extras(profile)
    }

    /// `ddl profile extras … remove`: drop one extra; the file is untouched.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown recipe or profile, an
    /// out-of-range index or a storage failure.
    pub fn extras_remove(
        &mut self,
        recipe_id: &str,
        selector: Option<&str>,
        group: ExtraGroup,
        index: usize,
    ) -> Result<ExtrasOutcome, CliFailure> {
        let _lock = self.lock_cli()?;
        let mut profile = self.extras_profile(recipe_id, selector)?;
        let indices = local::group_indices(&profile.extras, group);
        let at = *indices.get(index).ok_or_else(|| {
            CliFailure::message(format!(
                "index {index} is past the {group} list's {} entries",
                indices.len()
            ))
        })?;
        profile.extras.remove(at);
        self.save_extras(profile)
    }

    /// `ddl profile extras … move`: reorder one extra inside a group.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown recipe or profile, an
    /// out-of-range index or a storage failure.
    pub fn extras_move(
        &mut self,
        recipe_id: &str,
        selector: Option<&str>,
        group: ExtraGroup,
        from: usize,
        to: usize,
    ) -> Result<ExtrasOutcome, CliFailure> {
        let _lock = self.lock_cli()?;
        let mut profile = self.extras_profile(recipe_id, selector)?;
        let indices = local::group_indices(&profile.extras, group);
        if from >= indices.len() || to >= indices.len() {
            return Err(CliFailure::message(format!(
                "index {} is past the {group} list's {} entries",
                from.max(to),
                indices.len()
            )));
        }
        let entry = profile.extras.remove(indices[from]);
        let after = local::group_indices(&profile.extras, group);
        let at = after
            .get(to)
            .copied()
            .unwrap_or_else(|| after.last().map_or(profile.extras.len(), |last| last + 1));
        profile.extras.insert(at, entry);
        self.save_extras(profile)
    }

    /// Re-records the changed extras of a catalog profile, for
    /// `ddl launch --accept-changes` (spec 026, R4). Returns how many changed.
    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown recipe or profile, an unreadable
    /// file or a storage failure.
    pub fn accept_extra_changes(
        &mut self,
        recipe_id: &str,
        selector: Option<&str>,
    ) -> Result<usize, CliFailure> {
        let _lock = self.lock_cli()?;
        let mut profile = self.extras_profile(recipe_id, selector)?;
        let accepted = local::accept_changed(&mut profile.extras)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        if !accepted.is_empty() {
            self.profiles
                .save(&profile)
                .map_err(|error| CliFailure::message(error.to_string()))?;
        }
        Ok(accepted.len())
    }

    /// `ddl profile extras … clear`: drop every extra from the profile.    ///
    /// # Errors
    ///
    /// Returns a [`CliFailure`] for an unknown recipe or profile or a storage
    /// failure.
    pub fn extras_clear(
        &mut self,
        recipe_id: &str,
        selector: Option<&str>,
    ) -> Result<ExtrasOutcome, CliFailure> {
        let _lock = self.lock_cli()?;
        let mut profile = self.extras_profile(recipe_id, selector)?;
        profile.extras.clear();
        self.save_extras(profile)
    }

    /// Saves a mutated custom setup, bumping its update time.
    fn save_custom(&self, mut setup: CustomSetup) -> Result<CustomSetup, CliFailure> {
        setup.updated_at = now_unix_secs();
        self.custom
            .save(&setup)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        Ok(setup)
    }

    /// The profile a custom launch uses: an explicit selector, else the setup's
    /// `-default` profile, else its most recently played.
    fn custom_profile(
        &self,
        setup: &CustomSetup,
        selector: Option<&str>,
    ) -> Result<Profile, CliFailure> {
        let profiles: Vec<Profile> = self
            .profiles
            .list()
            .map_err(|error| CliFailure::message(error.to_string()))?
            .into_iter()
            .filter(|profile| profile.recipe_id == setup.id)
            .collect();
        let selected = match selector {
            Some(selector) => {
                if let Some(exact) = profiles.iter().find(|profile| profile.id == selector) {
                    exact.clone()
                } else {
                    let matches: Vec<&Profile> = profiles
                        .iter()
                        .filter(|profile| profile.name == selector)
                        .collect();
                    match matches.as_slice() {
                        [one] => (*one).clone(),
                        [] => {
                            return Err(CliFailure::message(format!(
                                "no profile `{selector}` for `{}`",
                                setup.id
                            )));
                        }
                        _ => {
                            return Err(CliFailure::message(format!(
                                "ambiguous profile `{selector}`; use its id"
                            )));
                        }
                    }
                }
            }
            None => profiles
                .iter()
                .find(|profile| profile.id == format!("{}-default", setup.id))
                .or_else(|| profiles.iter().max_by_key(|profile| profile.last_played))
                .cloned()
                .unwrap_or_else(|| {
                    Profile::new(format!("{}-default", setup.id), "Default", &setup.id)
                }),
        };
        Ok(selected)
    }

    /// The selected profile of a catalog recipe, for extras editing (spec 026,
    /// R5). Validated against the recipe so a bad skill or component fails early.
    fn extras_profile(
        &self,
        recipe_id: &str,
        selector: Option<&str>,
    ) -> Result<Profile, CliFailure> {
        if recipe_id.starts_with("base:") {
            return Ok(self.base_item(recipe_id, selector)?.profile);
        }
        let recipe = self
            .recipe(recipe_id)
            .map_err(|_| CliFailure::message(format!("no catalog recipe with id `{recipe_id}`")))?;
        let profile = self.effective_profile(
            &recipe,
            &LaunchOverrides {
                profile: selector.map(str::to_owned),
                ..LaunchOverrides::default()
            },
        )?;
        Ok(profile)
    }

    /// Saves a profile whose extras changed and reports whether it has saves.
    fn save_extras(&self, profile: Profile) -> Result<ExtrasOutcome, CliFailure> {
        let saves_warn = local::has_saves(&self.profiles.dirs(&profile.id).save);
        self.profiles
            .save(&profile)
            .map_err(|error| CliFailure::message(error.to_string()))?;
        Ok(ExtrasOutcome {
            profile,
            saves_warn,
        })
    }

    /// Runs an already-prepared custom plan through the real (or test) runner.
    fn run_plan(&self, plan: &dsda::LaunchPlan) -> Result<dsda::LaunchOutcome, CliFailure> {
        let runner: Box<dyn EngineRunner> =
            Box::new(dsda::RealEngineRunner::for_state(&self.env.paths));
        #[cfg(test)]
        let runner: &dyn EngineRunner = match &self.env.test {
            Some(test) => test.runner.as_ref(),
            None => runner.as_ref(),
        };
        #[cfg(not(test))]
        let runner: &dyn EngineRunner = runner.as_ref();
        runner
            .launch(plan, &local::assign_allowlist())
            .map_err(|error| CliFailure::message(error.to_string()))
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

fn selected_profile(profiles: &[Profile], remembered: Option<&String>) -> Profile {
    profiles
        .iter()
        .find(|p| Some(&p.id) == remembered)
        .or_else(|| profiles.iter().max_by_key(|p| p.last_played))
        .expect("at least one profile")
        .clone()
}
/// The registry source for a GOG candidate's launcher (spec 021, R2).
/// Attaches the detected drawable size to a locally built plan (spec 064, R6).
fn plan_with_screen(
    mut plan: dsda::LaunchPlan,
    screen: Option<dsda::ScreenSize>,
) -> dsda::LaunchPlan {
    plan.screen = screen;
    plan
}

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

/// A fresh profile for `recipe`, enabling the recipe's default-on optional
/// components (spec 017, R5): the widescreen status bar starts on.
fn default_profile(recipe: &Recipe) -> Profile {
    let mut profile = Profile::new(format!("{}-default", recipe.id), "Default", &recipe.id);
    profile.components = recipe.default_components();
    profile
}

pub fn select_component(recipe: &Recipe, profile: &mut Profile, id: &str) {
    let enabled = !profile.components.iter().any(|held| held == id);
    match recipe.apply_component_choice(&profile.components, id, enabled) {
        Ok(components) => profile.components = components,
        Err(error) => {
            tracing::warn!(%error, component = id, "could not change component selection");
        }
    }
}

/// Enables an imported component without toggling an existing selection off.
/// The import flow uses this for an initial import; replacing the file keeps the
/// row enabled.
fn enable_component(recipe: &Recipe, profile: &mut Profile, id: &str) {
    match recipe.apply_component_choice(&profile.components, id, true) {
        Ok(components) => profile.components = components,
        Err(error) => {
            tracing::warn!(%error, component = id, "could not enable imported component");
        }
    }
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
            screen: None,
            test: None,
        }
        .without_catalog_auto_updates()
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
    fn multiple_profiles_copy_defaults_rename_delete_and_remember() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let mut s = Services::open(env.clone()).unwrap();
        let command = |s: &mut Services, c| s.execute(c, &CancellationToken::new(), &mut |_| {});
        let recipe = s.recipe("sigil").unwrap();
        let mut first = s.profile(&recipe).unwrap();
        assert_eq!(first.components, ["wide-status-bar"]);
        first.set_component("wide-status-bar", false);
        first.skill = 5;
        first.pistol_start = true;
        first.set_component("compat", true);
        first.overrides.insert("test_scalar".into(), 1);
        first.iwad_variant = Some("test-variant".into());
        first.touch(100);
        command(&mut s, Command::Save(first.clone())).unwrap();
        command(&mut s, Command::NewProfile("sigil".into(), true)).unwrap();
        let second = s.profile(&recipe).unwrap();
        assert_eq!(second.name, "Profile 2");
        assert_eq!(second.skill, 5);
        assert!(second.pistol_start);
        assert_eq!(second.components, first.components);
        assert_eq!(second.overrides, first.overrides);
        assert_eq!(second.iwad_variant, first.iwad_variant);
        assert_eq!(second.last_played, None);
        assert_ne!(s.profiles.dirs(&second.id), s.profiles.dirs(&first.id));
        command(&mut s, Command::NewProfile("sigil".into(), false)).unwrap();
        let third = s.profile(&recipe).unwrap();
        assert_eq!(third.name, "Profile 3");
        assert_eq!(third.skill, 4);
        assert!(!third.pistol_start);
        assert_eq!(third.components, ["wide-status-bar"]);
        command(
            &mut s,
            Command::RenameProfile(second.id.clone(), "  Casual  ".into()),
        )
        .unwrap();
        let renamed = s.profiles.load(&second.id).unwrap();
        let mut expected = second.clone();
        expected.name = "Casual".into();
        assert_eq!(renamed, expected, "rename changes only the display name");
        assert_eq!(s.profiles.dirs(&renamed.id), s.profiles.dirs(&second.id));
        command(
            &mut s,
            Command::SelectProfile("sigil".into(), second.id.clone()),
        )
        .unwrap();
        let restored = Services::open(env).unwrap();
        assert_eq!(restored.profile(&recipe).unwrap().id, second.id);
        let snapshot = restored.snapshot().unwrap();
        let item = snapshot
            .items
            .iter()
            .find(|i| i.recipe.id == "sigil")
            .unwrap();
        assert_eq!(item.profile.name, "Casual");
        assert_eq!(item.last_played, Some(100));
        assert_eq!(item.profiles.len(), 3);
        assert_eq!(
            restored
                .effective_profile(&recipe, &LaunchOverrides::default())
                .unwrap()
                .id,
            second.id
        );
        for selector in [&second.id, &"Casual".to_owned()] {
            let overrides = LaunchOverrides {
                profile: Some(selector.clone()),
                ..Default::default()
            };
            assert_eq!(
                s.effective_profile(&recipe, &overrides).unwrap().id,
                second.id
            );
        }
        let wrong = LaunchOverrides {
            profile: Some("missing".into()),
            ..Default::default()
        };
        assert!(s.effective_profile(&recipe, &wrong).is_err());
        assert!(
            command(
                &mut s,
                Command::RenameProfile(third.id.clone(), "casUAL".into()),
            )
            .is_err()
        );
        command(&mut s, Command::DeleteProfile(second.id)).unwrap();
        command(&mut s, Command::DeleteProfile(third.id)).unwrap();
        assert!(command(&mut s, Command::DeleteProfile(first.id)).is_err());
        command(&mut s, Command::NewProfile("sigil".into(), false)).unwrap();
        assert_eq!(s.profile(&recipe).unwrap().name, "Profile 2");
    }

    #[test]
    fn new_profile_does_not_adopt_orphaned_runtime_data() {
        let temp = tempfile::tempdir().unwrap();
        let mut s = Services::open(env(temp.path())).unwrap();
        let orphan = s.profiles.dirs("sigil-profile-2");
        fs::create_dir_all(&orphan.save).unwrap();
        fs::write(orphan.save.join("old-save"), b"old run").unwrap();
        s.execute(
            Command::NewProfile("sigil".into(), false),
            &CancellationToken::new(),
            &mut |_| {},
        )
        .unwrap();
        let profile = s.profile(&s.recipe("sigil").unwrap()).unwrap();
        assert_eq!(profile.name, "Profile 2");
        assert_eq!(profile.id, "sigil-profile-3");
        assert!(orphan.save.join("old-save").exists());
        assert!(!s.profiles.dirs(&profile.id).save.join("old-save").exists());
    }

    #[test]
    fn schema_two_without_last_profiles_loads_and_persists_selection() {
        let state = AppState::parse(
            br#"{"schema_version":2,"folders":[],"recipes":{},"favorites":["sigil"]}"#,
        )
        .unwrap();
        assert!(state.last_profiles.is_empty());
        assert!(state.favorites.contains("sigil"));
        let mut updated = state;
        updated
            .last_profiles
            .insert("sigil".into(), "sigil-default".into());
        let restored = AppState::parse(&serde_json::to_vec(&updated).unwrap()).unwrap();
        assert_eq!(restored.last_profiles["sigil"], "sigil-default");
    }

    #[test]
    fn startup_provisions_owned_paths_and_restores_the_default_profile() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let mut s = Services::open(env.clone()).unwrap();
        s.execute(Command::Startup, &CancellationToken::new(), &mut |_| {})
            .unwrap();
        let snapshot = s.snapshot().unwrap();
        // Compare with the catalog the service loaded (the embedded one here: the temp
        // state holds no fetched bundle), so adding recipes needs no test edit.
        assert_eq!(snapshot.items.len(), s.catalog.recipes.len());
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
    fn compatibility_music_switches_maps_and_clears_orphans() {
        let catalog = Catalog::embedded();
        let sigil = catalog.recipe("sigil").unwrap().clone();
        let mut profile = default_profile(&sigil);

        select_component(&sigil, &mut profile, "compat-soundtrack");
        assert!(profile.components.iter().any(|id| id == "compat"));
        assert!(
            profile
                .components
                .iter()
                .any(|id| id == "compat-soundtrack")
        );
        let enabled: Vec<_> = profile.components.iter().map(String::as_str).collect();
        let active = sigil.active_components(&enabled).unwrap();
        assert_eq!(
            active
                .iter()
                .map(|component| component.id.as_str())
                .collect::<Vec<_>>(),
            ["compat", "compat-soundtrack", "wide-status-bar"]
        );

        // Replacing an imported file must not toggle an enabled row off.
        let enabled = profile.components.clone();
        enable_component(&sigil, &mut profile, "compat-soundtrack");
        assert_eq!(profile.components, enabled);

        // Switching to the native paid replacement drops the COMPAT maps and
        // their dependent music in one action.
        select_component(&sigil, &mut profile, "soundtrack");
        assert_eq!(
            profile.components,
            ["wide-status-bar", "soundtrack"],
            "native replacement must not leave compatibility music behind"
        );

        // Choosing no exclusive member leaves only the default wide bar.
        profile.components = vec![
            "compat".to_owned(),
            "compat-soundtrack".to_owned(),
            "wide-status-bar".to_owned(),
        ];
        choose_exclusive_component(&sigil, &mut profile, "edition", None);
        assert_eq!(profile.components, ["wide-status-bar"]);

        // Persisted orphan state is rejected, not silently repaired at launch.
        profile.components = vec!["compat-soundtrack".to_owned()];
        assert!(profile.validate(&sigil).is_err());

        // Explicit contradictory CLI-style ids fail, not last-one-wins.
        let error = sigil
            .explicit_selection(&["compat".to_owned(), "soundtrack".to_owned()])
            .unwrap_err();
        assert!(error.contains("replacement"), "{error}");
        let error = sigil
            .explicit_selection(&["soundtrack".to_owned(), "compat-soundtrack".to_owned()])
            .unwrap_err();
        assert!(
            error.contains("replacement") || error.contains("requires"),
            "{error}"
        );
    }

    #[test]
    fn a_new_profile_enables_default_on_components_like_the_widescreen_bar() {
        let r = Catalog::embedded().recipe("eviternity").unwrap().clone();
        let profile = default_profile(&r);
        assert_eq!(profile.components, ["wide-status-bar"]);
        assert!(r.active_components(&["wide-status-bar"]).is_ok());
        let auger = Catalog::embedded().recipe("auger-zenith").unwrap().clone();
        assert!(default_profile(&auger).components.is_empty());
    }
    #[test]
    fn state_rejects_unknown_schema_and_cancelled_work_does_not_change_user_state() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        // This cancellation case deliberately starts without a state file;
        // it drives Services directly and cannot dispatch Runtime's HTTP check.
        fs::remove_file(env.paths.config.join("app.json")).unwrap();
        let mut s = Services::open(env.clone()).unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert_eq!(
            s.execute(Command::Startup, &cancel, &mut |_| {})
                .map(|refresh| {
                    assert!(matches!(refresh, Refresh::Full));
                }),
            Err(Problem::Cancelled)
        );
        assert!(!env.paths.config.join("app.json").exists());
        fs::write(
            env.paths.config.join("app.json"),
            br#"{"schema_version":3,"folders":[],"recipes":{}}"#,
        )
        .unwrap();
        assert!(matches!(Services::open(env), Err(Problem::Unreadable(_))));
    }
    /// Spec 075/077 tests in this module drive the full-command path.
    fn run(s: &mut Services, command: Command) -> Result<(), Problem> {
        s.execute(command, &CancellationToken::new(), &mut |_| {})
            .map(|refresh| {
                assert!(matches!(refresh, Refresh::Full));
            })
    }
    fn favorite(s: &mut Services, id: &str) -> Result<(), Problem> {
        s.execute(
            Command::Favorite(id.into()),
            &CancellationToken::new(),
            &mut |_| {},
        )
        .map(|refresh| {
            assert!(matches!(refresh, Refresh::Full));
        })
    }
    /// Spec 075, R2/R4: the Pistol start toggle persists one field through the
    /// light path — both directions, repeated input, every other profile field
    /// intact — and builds no snapshot and resolves no launch, so installed
    /// members are never re-hashed (the counters are deterministic, not timings).
    #[test]
    fn pistol_start_is_a_light_atomic_profile_change() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let mut s = Services::open(env.clone()).unwrap();
        let recipe = s.recipe("sigil").unwrap();
        let mut profile = s.profile(&recipe).unwrap();
        profile.skill = 5;
        run(&mut s, Command::Save(profile.clone())).unwrap();
        let snapshots = s.counters().snapshots();
        let resolves = s.counters().resolves();

        let toggle = |s: &mut Services, value: bool| {
            s.execute(
                Command::SetPistolStart {
                    recipe_id: "sigil".into(),
                    profile_id: profile.id.clone(),
                    value,
                },
                &CancellationToken::new(),
                &mut |_| {},
            )
            .expect("the light path succeeds")
        };

        match toggle(&mut s, true) {
            Refresh::Profile(saved) => {
                assert!(saved.pistol_start);
                assert_eq!(saved.skill, 5, "other fields are preserved");
                assert_eq!(saved.components, profile.components);
            }
            Refresh::Full => panic!("the toggle must not rebuild the snapshot"),
        }
        match toggle(&mut s, false) {
            Refresh::Profile(saved) => assert!(!saved.pistol_start),
            Refresh::Full => panic!("the toggle must not rebuild the snapshot"),
        }
        match toggle(&mut s, false) {
            Refresh::Profile(saved) => assert!(!saved.pistol_start),
            Refresh::Full => panic!("the toggle must not rebuild the snapshot"),
        }
        assert_eq!(
            s.counters().snapshots(),
            snapshots,
            "no catalog snapshot was built"
        );
        assert_eq!(
            s.counters().resolves(),
            resolves,
            "no launch was resolved (no member hashing)"
        );

        let reopened = Services::open(env).unwrap();
        let stored = reopened.profiles.load(&profile.id).unwrap();
        assert!(!stored.pistol_start);
        assert_eq!(stored.skill, 5);
        assert_eq!(stored.components, profile.components);
        assert_eq!(stored.extras, profile.extras);

        let mut s = reopened;
        assert!(
            s.execute(
                Command::SetPistolStart {
                    recipe_id: "eviternity".into(),
                    profile_id: profile.id.clone(),
                    value: true,
                },
                &CancellationToken::new(),
                &mut |_| {},
            )
            .is_err()
        );
        assert!(
            !s.profiles.load(&profile.id).unwrap().pistol_start,
            "the refused request left the stored value alone"
        );
    }

    /// Spec 075, R2 review: the first-ever press, before any Save wrote a
    /// profile JSON, materialises the default profile and persists the value.
    #[test]
    fn the_first_pistol_press_works_without_a_saved_profile() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let mut s = Services::open(env.clone()).unwrap();
        let recipe = s.recipe("sigil").unwrap();
        let profile = s.profile(&recipe).unwrap();
        assert!(
            !s.profiles.path(&profile.id).unwrap().is_file(),
            "no profile JSON exists yet"
        );
        let snapshots = s.counters().snapshots();
        match s
            .execute(
                Command::SetPistolStart {
                    recipe_id: "sigil".into(),
                    profile_id: profile.id.clone(),
                    value: true,
                },
                &CancellationToken::new(),
                &mut |_| {},
            )
            .expect("the first press succeeds")
        {
            Refresh::Profile(saved) => assert!(saved.pistol_start),
            Refresh::Full => panic!("the toggle must not rebuild the snapshot"),
        }
        assert_eq!(s.counters().snapshots(), snapshots, "no snapshot built");
        assert!(s.profiles.load(&profile.id).unwrap().pistol_start);
    }

    /// Spec 075, R2: a failed write is reported, never silently dropped.
    #[cfg(unix)]
    #[test]
    fn a_failed_pistol_start_write_is_reported() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let mut s = Services::open(env.clone()).unwrap();
        let recipe = s.recipe("sigil").unwrap();
        let profile = s.profile(&recipe).unwrap();
        run(&mut s, Command::Save(profile.clone())).unwrap();

        let path = s.profiles.path(&profile.id).unwrap();
        let parent = path.parent().unwrap().to_path_buf();
        let mut permissions = fs::metadata(&parent).unwrap().permissions();
        permissions.set_mode(0o500);
        fs::set_permissions(&parent, permissions).unwrap();
        let result = s.execute(
            Command::SetPistolStart {
                recipe_id: "sigil".into(),
                profile_id: profile.id.clone(),
                value: true,
            },
            &CancellationToken::new(),
            &mut |_| {},
        );
        let mut restore = fs::metadata(&parent).unwrap().permissions();
        restore.set_mode(0o700);
        fs::set_permissions(&parent, restore).unwrap();

        assert!(matches!(result, Err(Problem::Storage)), "{result:?}");
        assert!(!s.profiles.load(&profile.id).unwrap().pistol_start);
    }

    /// Spec 075, R1/R4: the base-game Pistol start refuses an unknown base
    /// profile without building a snapshot or resolving a launch; the positive
    /// path is the same light routine the catalog toggle covers.
    #[test]
    fn base_pistol_start_is_a_light_path_and_refuses_unknown_profiles() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let mut s = Services::open(env).unwrap();
        let snapshots = s.counters().snapshots();
        let resolves = s.counters().resolves();
        assert!(
            s.execute(
                Command::Local(LocalCommand::BasePistolStart {
                    profile_id: "no-such-profile".into(),
                    value: true,
                }),
                &CancellationToken::new(),
                &mut |_| {},
            )
            .is_err(),
            "an unknown profile is refused"
        );
        assert_eq!(s.counters().snapshots(), snapshots, "no snapshot built");
        assert_eq!(s.counters().resolves(), resolves, "no launch resolved");
    }

    fn state_file(env: &Environment) -> serde_json::Value {
        serde_json::from_slice(&fs::read(env.paths.config.join("app.json")).unwrap()).unwrap()
    }
    #[test]
    fn favorites_toggle_persist_and_reject_unknown_recipes() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let mut s = Services::open(env.clone()).unwrap();
        assert!(s.snapshot().unwrap().items.iter().all(|i| !i.favorite));
        favorite(&mut s, "sigil").unwrap();
        let state = state_file(&env);
        assert_eq!(state["schema_version"], 2);
        assert_eq!(state["favorites"], serde_json::json!(["sigil"]));
        let reopened = Services::open(env.clone()).unwrap().snapshot().unwrap();
        let marked: Vec<_> = reopened
            .items
            .iter()
            .filter(|i| i.favorite)
            .map(|i| i.recipe.id.as_str())
            .collect();
        assert_eq!(marked, ["sigil"]);
        // An id the catalog does not know is refused and changes nothing.
        assert_eq!(favorite(&mut s, "no-such-mapset"), Err(Problem::State));
        assert_eq!(state_file(&env)["favorites"], serde_json::json!(["sigil"]));
        favorite(&mut s, "sigil").unwrap();
        assert_eq!(state_file(&env)["favorites"], serde_json::json!([]));
    }
    #[test]
    fn schema_one_state_migrates_to_two_keeping_folders_and_installs() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        fs::create_dir_all(&env.paths.config).unwrap();
        let folder = temp.path().join("iwads");
        fs::write(
            env.paths.config.join("app.json"),
            format!(r#"{{"schema_version":1,"folders":[{folder:?}],"recipes":{{}}}}"#),
        )
        .unwrap();
        let mut s = Services::open(env.clone()).unwrap();
        // Loading alone leaves the old file as written.
        assert_eq!(state_file(&env)["schema_version"], 1);
        let snapshot = s.snapshot().unwrap();
        assert_eq!(snapshot.folders, std::slice::from_ref(&folder));
        assert!(snapshot.items.iter().all(|i| !i.favorite));
        // Spec 019 R5: the additive fields default on/absent for an old file.
        assert!(snapshot.catalog.auto_update);
        assert_eq!(snapshot.catalog.last_check, None);
        assert!(!snapshot.catalog.updated);
        // The next save writes schema 2 with the old data and the new field.
        favorite(&mut s, "sigil-ii").unwrap();
        let state = state_file(&env);
        assert_eq!(state["schema_version"], 2);
        assert_eq!(state["favorites"], serde_json::json!(["sigil-ii"]));
        assert_eq!(state["folders"], serde_json::json!([folder]));
        assert_eq!(state["catalog_auto_update"], serde_json::json!(true));
        assert_eq!(state["catalog_last_check"], serde_json::Value::Null);
        assert!(Services::open(env).is_ok());
    }
    #[test]
    fn install_snapshots_without_tags_show_the_catalog_tags() {
        let temp = tempfile::tempdir().unwrap();
        let env = env(temp.path());
        let catalog = Catalog::embedded();
        let mut old = catalog.recipe("sigil").unwrap().clone();
        old.tags.clear();
        fs::create_dir_all(&env.paths.config).unwrap();
        fs::write(
            env.paths.config.join("app.json"),
            serde_json::json!({
                "schema_version": 1, "folders": [], "recipes": { "sigil": old }
            })
            .to_string(),
        )
        .unwrap();
        let snapshot = Services::open(env).unwrap().snapshot().unwrap();
        let sigil = snapshot
            .items
            .iter()
            .find(|i| i.recipe.id == "sigil")
            .unwrap();
        assert!(!sigil.recipe.tags.is_empty());
        assert_eq!(sigil.recipe.tags, catalog.recipe("sigil").unwrap().tags);
        assert_eq!(snapshot.tags, catalog.tags);
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
        // The save migrated the file to schema 2 (spec 015) and kept the setting.
        assert!(raw.contains("\"schema_version\": 2"), "{raw}");
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
        assert_eq!(view.items.len(), s.catalog.recipes.len());
        s.execute(Command::Reset, &CancellationToken::new(), &mut |_| {})
            .unwrap();
        let backups = fs::read_dir(s.env.paths.config.join("dsda"))
            .unwrap()
            .flatten()
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("state.json.unreadable-")
            })
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(backups[0].path()).unwrap(), b"broken");
        assert!(s.config.state().is_ok());
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
        let mut services = Services::open(
            Environment {
                paths,
                home: home.clone(),
                screen: None,
                test: None,
            }
            .without_catalog_auto_updates(),
        )
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
            screen: None,
            test: None,
        }
        .without_catalog_auto_updates()
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
    if let Some(id) = choice {
        match recipe.apply_component_choice(&profile.components, id, true) {
            Ok(components) => profile.components = components,
            Err(error) => {
                tracing::warn!(%error, component = id, "could not choose exclusive component");
            }
        }
        return;
    }
    let mut components = profile.components.clone();
    for component in recipe
        .components
        .iter()
        .filter(|component| component.exclusive_group.as_deref() == Some(group))
    {
        match recipe.apply_component_choice(&components, &component.id, false) {
            Ok(next) => components = next,
            Err(error) => {
                tracing::warn!(%error, component = component.id, "could not clear exclusive group");
                return;
            }
        }
    }
    profile.components = components;
}

#[cfg(test)]
#[path = "recipe_change_tests.rs"]
pub(crate) mod recipe_change_tests;

#[cfg(test)]
#[path = "uninstall_tests.rs"]
pub(crate) mod uninstall_tests;

#[cfg(test)]
#[path = "audio_tests.rs"]
mod audio_tests;

#[cfg(test)]
mod catalog_tests {
    use super::*;
    use ddl_core::catalog::Bundle;
    use ddl_core::downloads::test_http::{Response, Server};
    use ddl_core::engines::dsda;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A real, lint-clean bundle built from this repository's `catalog/`.
    fn bundle_json(serial: u64) -> Vec<u8> {
        let catalog = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog");
        let bundle = ddl_core::catalog::build_bundle(&catalog, serial, "test-commit", "0.0.1")
            .expect("the repository catalog lints");
        serde_json::to_vec(&bundle).unwrap()
    }

    fn environment(root: &Path, catalog_url: String) -> Environment {
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let paths = ddl_core::platform::paths::resolve(&|key: &str| {
            (key == "HOME").then(|| home.display().to_string())
        })
        .unwrap();
        let mut catalog = Catalog::embedded();
        catalog.hosts.downloads.push("127.0.0.1".into());
        Environment {
            paths,
            home,
            screen: None,
            test: Some(Arc::new(TestEnvironment {
                api: Arc::new(dsda::CannedApi::new()),
                catalog,
                engine_base: String::new(),
                catalog_url,
                reviewed: dsda::ReviewedList::embedded(),
                runner: Arc::new(dsda::FakeEngineRunner::new(dsda::LaunchExit::Clean)),
                engine_response_timeout: None,
                soundfont_members: None,
            })),
        }
    }

    fn run(s: &mut Services, command: Command) -> Result<(), Problem> {
        s.execute(command, &CancellationToken::new(), &mut |_| {})
            .map(|refresh| {
                assert!(matches!(refresh, Refresh::Full));
            })
    }

    /// Spec 077, R1: the startup check on an open services instance.
    fn startup(s: &mut Services) -> StartupCatalog {
        s.startup_catalog_check(&CancellationToken::new())
            .expect("the startup check never fails the caller")
    }

    #[test]
    fn a_newer_bundle_is_accepted_cached_and_loaded_next_start() {
        let temp = tempfile::tempdir().unwrap();
        let served = bundle_json(500);
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = hits.clone();
        let server = Server::start(move |_r| {
            counter.fetch_add(1, Ordering::SeqCst);
            Response::ok(served.clone())
        });
        let env = environment(temp.path(), server.url("/catalog.json"));
        let mut s = Services::open(env.clone()).unwrap();
        assert!(!s.catalog_info().updated);
        run(&mut s, Command::CatalogCheck).unwrap();
        let info = s.catalog_info();
        assert!(info.updated, "the accepted bundle is the catalog in use");
        assert_eq!(info.serial, 500);
        assert_eq!(info.source_commit, "test-commit");
        assert!(info.fetched_at.is_some());
        assert!(ddl_core::catalog::cache_path(&env.paths.data).is_file());
        assert!(s.catalog.recipe("sigil").is_some(), "accepted recipes load");

        // R4: a later start loads the newer cached catalog.
        let reopened = Services::open(env).unwrap();
        let info = reopened.catalog_info();
        assert!(info.updated);
        assert_eq!(info.serial, 500);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    /// Spec 077, R1/R5: two separate startups inside 24 hours both check, the
    /// opt-out still wins, and the manual action always checks.
    #[test]
    fn every_startup_checks_once_within_the_same_day() {
        let temp = tempfile::tempdir().unwrap();
        let served = bundle_json(500);
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = hits.clone();
        let server = Server::start(move |_r| {
            counter.fetch_add(1, Ordering::SeqCst);
            Response::ok(served.clone())
        });
        let env = environment(temp.path(), server.url("/catalog.json"));

        let mut first = Services::open(env.clone()).unwrap();
        assert!(first.catalog_info().auto_update, "on by default");
        assert_eq!(startup(&mut first).updated, Some(500));
        assert!(first.catalog_info().last_check.is_some());
        assert_eq!(hits.load(Ordering::SeqCst), 1);

        let mut second = Services::open(env.clone()).unwrap();
        assert_eq!(startup(&mut second).updated, None);
        assert_eq!(hits.load(Ordering::SeqCst), 2, "no daily throttle");
        assert_eq!(second.catalog_info().serial, 500);

        run(&mut second, Command::SetCatalogAutoUpdate(false)).unwrap();
        let mut third = Services::open(env.clone()).unwrap();
        assert_eq!(startup(&mut third).updated, None);
        assert_eq!(hits.load(Ordering::SeqCst), 2, "opted out");

        run(&mut third, Command::CatalogCheck).unwrap();
        assert_eq!(hits.load(Ordering::SeqCst), 3, "manual always checks");
    }

    /// Spec 077, R1/R5: a failed startup check is silent, keeps the current
    /// catalog and does not throttle the next startup.
    #[test]
    fn a_failed_startup_check_is_silent_and_keeps_the_catalog() {
        let temp = tempfile::tempdir().unwrap();
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = hits.clone();
        let server = Server::start(move |_r| {
            counter.fetch_add(1, Ordering::SeqCst);
            Response::status(503, "offline")
        });
        let env = environment(temp.path(), server.url("/catalog.json"));
        let mut s = Services::open(env.clone()).unwrap();
        assert_eq!(startup(&mut s).updated, None, "silent, no snapshot");
        assert!(s.catalog.recipe("sigil").is_some(), "catalog retained");
        assert!(s.catalog_info().last_check.is_some(), "attempt recorded");
        assert_eq!(hits.load(Ordering::SeqCst), 1);

        let mut next = Services::open(env).unwrap();
        assert_eq!(startup(&mut next).updated, None);
        assert_eq!(hits.load(Ordering::SeqCst), 2, "a failure never throttles");

        assert!(run(&mut s, Command::CatalogCheck).is_err());
    }

    /// Spec 077, R1: the network wait must not overwrite user state saved while
    /// it was in flight.
    #[test]
    fn a_startup_check_keeps_state_saved_during_the_request() {
        let temp = tempfile::tempdir().unwrap();
        let served = bundle_json(500);
        let server = Server::start(move |_r| Response::ok(served.clone()));
        let env = environment(temp.path(), server.url("/catalog.json"));

        let mut checking = Services::open(env.clone()).unwrap();
        let mut ui = Services::open(env.clone()).unwrap();
        run(&mut ui, Command::Favorite("sigil".into())).unwrap();
        let profile = ui.profile(&ui.recipe("sigil").unwrap()).unwrap();
        run(&mut ui, Command::Save(profile)).unwrap();

        assert_eq!(startup(&mut checking).updated, Some(500));
        let snapshot = Services::open(env.clone()).unwrap().snapshot().unwrap();
        assert!(
            snapshot
                .items
                .iter()
                .any(|item| item.recipe.id == "sigil" && item.favorite),
            "the favorite saved during the request survives"
        );
        let reopened = Services::open(env).unwrap();
        assert!(reopened.state.favorites.contains("sigil"));
        assert!(reopened.catalog_info().updated);
    }

    /// Spec 077, R1: a stalled fetch must not touch user state.
    #[test]
    fn a_stalled_startup_fetch_never_loses_state_saved_meanwhile() {
        let temp = tempfile::tempdir().unwrap();
        let served = bundle_json(500);
        let (release, wait) = std::sync::mpsc::channel::<()>();
        let wait = std::sync::Mutex::new(wait);
        let server = Server::start(move |_r| {
            let _ = wait.lock().unwrap().recv();
            Response::ok(served.clone())
        });
        let env = environment(temp.path(), server.url("/catalog.json"));

        let thread_env = env.clone();
        let handle = std::thread::spawn(move || {
            let mut checking = Services::open(thread_env).unwrap();
            checking
                .startup_catalog_check(&CancellationToken::new())
                .unwrap()
        });
        std::thread::sleep(std::time::Duration::from_millis(100));
        let mut ui = Services::open(env.clone()).unwrap();
        run(&mut ui, Command::Favorite("sigil".into())).unwrap();
        let profile = ui.profile(&ui.recipe("sigil").unwrap()).unwrap();
        run(&mut ui, Command::Save(profile)).unwrap();
        let iwads_before = fs::read(env.paths.config.join("iwads.json")).ok();
        release.send(()).unwrap();

        let outcome = handle.join().unwrap();
        assert_eq!(outcome.updated, Some(500));
        let reopened = Services::open(env.clone()).unwrap();
        assert!(
            reopened.state.favorites.contains("sigil"),
            "favorite survives"
        );
        assert!(
            reopened.catalog_info().last_check.is_some(),
            "attempt recorded"
        );
        assert!(reopened.catalog_info().updated, "the new catalog is cached");
        assert_eq!(
            fs::read(env.paths.config.join("iwads.json")).ok(),
            iwads_before,
            "the background check never rewrites the IWAD registry"
        );
        let snapshot = reopened.snapshot().unwrap();
        assert!(
            snapshot
                .items
                .iter()
                .any(|item| item.recipe.id == "sigil" && item.favorite),
            "the view the UI adopts shows the state saved meanwhile"
        );
    }

    /// Spec 077, R1 review: a slower automatic response never rolls back a newer
    /// catalog adopted (by another process) while the request was in flight.
    #[test]
    fn an_older_automatic_response_never_rolls_back_a_newer_catalog() {
        let temp = tempfile::tempdir().unwrap();
        let served = bundle_json(500);
        let server = Server::start(move |_r| Response::ok(served.clone()));
        let env = environment(temp.path(), server.url("/catalog.json"));

        // The background instance opens while the embedded catalog is in use…
        let mut checking = Services::open(env.clone()).unwrap();
        // …then another process adopts serial 900 into the cache.
        let newer: ddl_core::catalog::Bundle = serde_json::from_slice(&bundle_json(900)).unwrap();
        ddl_core::catalog::store_cache(&env.paths.data, &newer, 1).unwrap();

        let outcome = startup(&mut checking);
        assert_eq!(outcome.updated, None, "the older response is ignored");
        let cached = ddl_core::catalog::load_cache(&env.paths.data).expect("cache");
        assert_eq!(
            cached.bundle.catalog_serial, 900,
            "the cache keeps the newer bundle"
        );
    }

    #[test]
    fn a_corrupt_or_older_cache_never_replaces_the_embedded_catalog() {
        let temp = tempfile::tempdir().unwrap();
        let env = environment(temp.path(), String::new());
        let cache = ddl_core::catalog::cache_path(&env.paths.data);
        fs::create_dir_all(cache.parent().unwrap()).unwrap();
        fs::write(&cache, b"{ not json").unwrap();
        let s = Services::open(env.clone()).unwrap();
        assert!(!s.catalog_info().updated, "corrupt cache ignored");
        assert!(s.catalog.recipe("sigil").is_some());

        // A well-formed cache with serial 0 is not newer than the embedded one.
        let bundle: Bundle = serde_json::from_slice(&bundle_json(0)).unwrap();
        ddl_core::catalog::store_cache(&env.paths.data, &bundle, 7).unwrap();
        let s = Services::open(env).unwrap();
        assert!(!s.catalog_info().updated, "an equal serial is not newer");
    }
}

#[cfg(test)]
mod release_state_tests {
    use super::*;
    #[test]
    fn release_state_trees_remain_readable() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/state");
        let required_tags = [
            "v0.0.5-screens.1",
            "v0.0.6-local",
            "v0.0.7-deck",
            "v0.0.8-upgrade",
            "v0.0.9-focus",
            "v0.0.10-audio",
        ];
        for tag in required_tags {
            assert!(root.join(tag).is_dir(), "missing release fixture {tag}");
        }
        let mut tags = fs::read_dir(&root)
            .unwrap()
            .flatten()
            .filter(|e| {
                e.file_type().unwrap().is_dir() && e.file_name().to_string_lossy().starts_with('v')
            })
            .map(|e| e.path())
            .collect::<Vec<_>>();
        tags.sort();
        assert!(
            tags.len() >= required_tags.len(),
            "release fixtures must never be silently removed"
        );
        for tree in tags {
            let tag = tree.file_name().unwrap().to_str().unwrap();
            let env = Environment {
                paths: Paths {
                    config: tree.join("config"),
                    data: tree.join("data"),
                    cache: tree.join("cache"),
                    state: tree.join("state"),
                },
                home: "/ddl-fixture".into(),
                screen: None,
                test: None,
            };
            // This is the production open and snapshot path, not a fixture-only parser.
            let services = Services::open(env.clone()).unwrap_or_else(|e| panic!("{tag}: {e}"));
            let snapshot = services.snapshot().unwrap_or_else(|e| panic!("{tag}: {e}"));
            assert_eq!(snapshot.engine.unwrap().version, "0.30.1", "{tag}");
            assert!(
                snapshot.config_usable,
                "{tag}: provisioned DSDA config remains usable"
            );
            assert_eq!(
                snapshot.config_version,
                match tag {
                    "v0.0.5-screens.1" | "v0.0.6-local" => "1",
                    "v0.0.7-deck" => "3",
                    "v0.0.8-upgrade" | "v0.0.9-focus" => "4",
                    _ => "5",
                },
                "{tag}: saved DSDA baseline is preserved"
            );
            assert_eq!(
                services.state.folders,
                [
                    PathBuf::from("/ddl-fixture/iwads-one"),
                    PathBuf::from("/ddl-fixture/iwads-two")
                ]
            );
            assert_eq!(services.registry.entries().len(), 2, "{tag}");
            assert!(services.state.search_standard_folders);
            assert!(!services.state.catalog_auto_update);
            assert!(services.state.favorites.contains("auger-zenith"));
            assert_eq!(services.state.kept_recipes.len(), 2);
            let installed = snapshot
                .items
                .iter()
                .filter(|i| i.manifest.is_some())
                .collect::<Vec<_>>();
            assert_eq!(installed.len(), 2, "{tag}");
            let auger = installed
                .iter()
                .find(|i| i.recipe.id == "auger-zenith")
                .unwrap();
            assert_eq!(auger.profiles.len(), 2);
            assert_eq!(auger.profile.id, "fixture-alternate");
            assert_eq!(auger.profile.skill, 2);
            assert_eq!(
                auger
                    .profiles
                    .iter()
                    .find(|p| p.id == "fixture-auger-zenith")
                    .unwrap()
                    .skill,
                match tag {
                    "v0.0.5-screens.1" | "v0.0.6-local" | "v0.0.7-deck" => 3,
                    _ => 4,
                },
                "saved default skill is preserved for {tag}"
            );
            assert_eq!(auger.profile.components, ["alternate-blaster"]);
            assert!(auger.favorite);
            assert_eq!(
                auger.recipe.compatibility.complevel,
                ddl_core::catalog::Complevel::Alias(ddl_core::catalog::ComplevelAlias::Vanilla)
            );
            for item in installed {
                let active = services.active(&item.recipe.id).unwrap().unwrap();
                // The snapshot refreshes display tags from the current catalog by
                // design; the frozen install keeps the tags it was installed
                // with. Normalise only that display metadata, then compare every
                // launch field, and assert both tag sources separately.
                let mut frozen = active.recipe.clone();
                frozen.tags.clone_from(&item.recipe.tags);
                assert_eq!(
                    frozen, item.recipe,
                    "{tag}: frozen launch recipe preserved (display tags refreshed separately)"
                );
                assert_eq!(
                    item.recipe.tags,
                    services.catalog.recipe(&item.recipe.id).unwrap().tags,
                    "{tag}: display tags follow the current catalog"
                );
                let stored: serde_json::Value = serde_json::from_slice(
                    &fs::read(
                        tree.join("data/library")
                            .join(&item.recipe.id)
                            .join("active.json"),
                    )
                    .unwrap(),
                )
                .unwrap();
                let stored_tags: Vec<String> =
                    serde_json::from_value(stored["recipe"]["tags"].clone()).unwrap();
                assert_eq!(
                    active.recipe.tags, stored_tags,
                    "{tag}: the frozen recipe keeps its installed tags"
                );
                assert!(active.previous_recipe.is_some());
                let manifest = item.manifest.as_ref().unwrap();
                assert!(!manifest.members.is_empty());
                let generation = InstallManifest::load(
                    &tree
                        .join("data/library")
                        .join(&item.recipe.id)
                        .join("generations/fixture/install.json"),
                )
                .unwrap();
                assert_eq!(
                    &generation, manifest,
                    "{tag}: generation metadata preserved"
                );
            }
            assert_eq!(
                snapshot.custom.len(),
                usize::from(tag != "v0.0.5-screens.1")
            );
            if let Some(setup) = snapshot.custom.first() {
                assert_eq!(setup.name, "My custom setup");
            }
            let has_audio = tag == "v0.0.10-audio";
            assert_eq!(tree.join("config/audio.json").is_file(), has_audio, "{tag}");
            assert_eq!(
                services.soundfonts.preferences().unwrap().choice,
                has_audio.then_some(Choice::Default),
                "{tag}: explicit audio choice is preserved"
            );
            assert_eq!(
                snapshot.audio.choice,
                has_audio.then_some(Choice::Default),
                "{tag}"
            );
            assert!(
                !snapshot.audio.installed,
                "{tag}: no font payload is bundled"
            );
            assert!(
                snapshot.audio.warning.is_none(),
                "{tag}: audio preferences readable"
            );
            // Saving loaded state emits only the current shape and is atomic.
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("app.json");
            save_json(&path, &services.state).unwrap();
            let saved = fs::read(&path).unwrap();
            assert!(!String::from_utf8_lossy(&saved).contains("complevel_status"));
            assert_eq!(
                AppState::parse(&saved).unwrap().last_profiles,
                services.state.last_profiles
            );
            let active = services.active("auger-zenith").unwrap().unwrap();
            active.save(&temp.path().join("active.json")).unwrap();
            assert!(
                !fs::read_to_string(temp.path().join("active.json"))
                    .unwrap()
                    .contains("complevel_status")
            );
        }
    }
}

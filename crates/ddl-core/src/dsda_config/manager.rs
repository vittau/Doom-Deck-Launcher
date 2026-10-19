//! `DsdaConfigManager`: DDL's private, versioned DSDA-Doom configuration
//! (spec 005, R3–R6, R8).
//!
//! Layout under DDL's XDG config directory (`~/.config/ddl` by default):
//!
//! ```text
//! dsda/active.cfg                     # what DSDA reads and rewrites via -config
//! dsda/base/<template_version>.cfg    # exact copy of what DDL last applied
//! dsda/backups/<timestamp>-<version>.cfg
//! dsda/run.lock                       # engine-run guard (R4)
//! dsda/state.json                     # template version, last migration, conflicts
//! ```
//!
//! DDL never reads or writes DSDA's own default configuration under
//! `~/.dsda-doom/` or `$XDG_DATA_HOME/dsda-doom/` (decision D-012).

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::Error;
use super::backups;
use super::format::{Document, Value};
use super::guard::{RunGuard, recorded_pid};
use super::merge;
use super::state::{Conflict, PersistedState, STATE_FILE, STATE_SCHEMA_VERSION, State};
use super::template::Template;

/// Directory appended to DDL's config directory.
pub const DSDA_DIR: &str = "dsda";
const ACTIVE_FILE: &str = "active.cfg";
const BASE_DIR: &str = "base";

/// A source of wall-clock time, injectable so backup names are deterministic in
/// tests.
pub trait Clock {
    /// Seconds since the Unix epoch.
    fn now_unix_secs(&self) -> u64;
}

/// The real clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_secs(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }
}

/// What [`DsdaConfigManager::provision`] created (R3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Provision {
    /// Whether the active file was created (first use) or already existed.
    pub created_active: bool,
    /// Whether the state file was created.
    pub created_state: bool,
}

/// What [`DsdaConfigManager::migrate`] did (R5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    /// `false` when the template version had not changed.
    pub migrated: bool,
    /// The template version that was applied before.
    pub from_version: String,
    /// The template version now applied.
    pub to_version: String,
    /// Conflicts found while merging.
    pub conflicts: Vec<Conflict>,
    /// The backup written before the merge, if a merge happened.
    pub backup: Option<PathBuf>,
}

/// What [`DsdaConfigManager::reset`] did (R6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reset {
    /// The backup written before the reset.
    pub backup: PathBuf,
}

/// DDL's private DSDA-Doom configuration manager.
pub struct DsdaConfigManager {
    dir: PathBuf,
    template: Template,
    clock: Box<dyn Clock>,
}

impl DsdaConfigManager {
    /// Creates a manager for `config_dir` using the embedded template.
    ///
    /// # Errors
    /// Returns [`Error::InvalidTemplate`] if the embedded template is broken.
    pub fn new(config_dir: impl Into<PathBuf>) -> Result<Self, Error> {
        Ok(Self::with_template(config_dir, Template::embedded()?))
    }

    /// Creates a manager with a specific template (useful for tests and future
    /// baselines).
    pub fn with_template(config_dir: impl Into<PathBuf>, template: Template) -> Self {
        Self {
            dir: config_dir.into().join(DSDA_DIR),
            template,
            clock: Box::new(SystemClock),
        }
    }

    /// Replaces the clock (tests).
    #[must_use]
    pub fn with_clock(mut self, clock: Box<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// DDL's DSDA config directory.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The template in use.
    #[must_use]
    pub fn template(&self) -> &Template {
        &self.template
    }

    /// Path of the active configuration file.
    #[must_use]
    pub fn active_path(&self) -> PathBuf {
        self.dir.join(ACTIVE_FILE)
    }

    /// Directory holding the applied baselines.
    #[must_use]
    pub fn base_dir(&self) -> PathBuf {
        self.dir.join(BASE_DIR)
    }

    /// Path of the baseline for `template_version`.
    #[must_use]
    pub fn base_path(&self, template_version: &str) -> PathBuf {
        self.base_dir().join(format!("{template_version}.cfg"))
    }

    /// Directory holding backups.
    #[must_use]
    pub fn backups_dir(&self) -> PathBuf {
        self.dir.join(backups::BACKUP_DIR)
    }

    /// Path of the persisted state file.
    #[must_use]
    pub fn state_path(&self) -> PathBuf {
        self.dir.join(STATE_FILE)
    }

    /// Acquires the shared run guard (R4). Hold it for the whole engine run.
    ///
    /// # Errors
    /// Returns [`super::GuardError::RunActive`] when a run is already active.
    pub fn run_guard(&self) -> Result<RunGuard, super::GuardError> {
        RunGuard::acquire(&self.dir)
    }

    /// First-use provisioning (R3): create the active file from the template and
    /// store an exact copy as the baseline. Idempotent; it only fills in what is
    /// missing, so it never overwrites user or engine changes.
    ///
    /// # Errors
    /// Returns [`Error::RunActive`] while an engine run is active, and
    /// [`Error::Io`] on filesystem failures.
    pub fn provision(&self) -> Result<Provision, Error> {
        self.ensure_no_run()?;
        fs::create_dir_all(&self.dir)?;
        let text = self.template.config_text();

        let active = self.active_path();
        let created_active = !active.exists();
        if created_active {
            self.write_atomic(&active, text.as_bytes())?;
        }

        let base = self.base_path(&self.template.template_version);
        if !base.exists() {
            self.write_atomic(&base, text.as_bytes())?;
        }

        let created_state = self.load_state_opt()?.is_none();
        if created_state {
            self.save_state(&PersistedState {
                template_version: self.template.template_version.clone(),
                ..PersistedState::default()
            })?;
        }

        Ok(Provision {
            created_active,
            created_state,
        })
    }

    /// Provisions on first use and migrates if the template version changed.
    ///
    /// # Errors
    /// See [`DsdaConfigManager::provision`] and [`DsdaConfigManager::migrate`].
    pub fn ensure_ready(&self) -> Result<Migration, Error> {
        self.provision()?;
        self.migrate()
    }

    /// Three-way merges the active file onto the current template when the
    /// template version changed (R5). A no-op when the version already matches.
    ///
    /// # Errors
    /// Returns [`Error::NotProvisioned`] before first use,
    /// [`Error::MissingBaseline`] when the previous baseline is gone,
    /// [`Error::RunActive`] while an engine run is active.
    pub fn migrate(&self) -> Result<Migration, Error> {
        self.ensure_no_run()?;
        let state = self.load_state_opt()?.ok_or(Error::NotProvisioned)?;
        let to_version = self.template.template_version.clone();
        if state.template_version == to_version {
            return Ok(Migration {
                migrated: false,
                from_version: state.template_version,
                to_version,
                conflicts: state.conflicts,
                backup: None,
            });
        }

        let base_path = self.base_path(&state.template_version);
        if !base_path.exists() {
            return Err(Error::MissingBaseline {
                version: state.template_version,
            });
        }
        let base = Document::parse(&fs::read_to_string(&base_path)?);

        let active_path = self.active_path();
        if !active_path.exists() {
            return Err(Error::NotProvisioned);
        }
        let active = Document::parse(&fs::read_to_string(&active_path)?);

        let result = merge::merge(&base, &active, &self.template);

        let now = self.clock.now_unix_secs();
        let backup = backups::create(&self.backups_dir(), &active_path, now, &to_version)?;

        let text = result.document.to_text();
        self.write_atomic(&active_path, text.as_bytes())?;
        self.write_atomic(&self.base_path(&to_version), text.as_bytes())?;
        self.save_state(&PersistedState {
            schema_version: STATE_SCHEMA_VERSION,
            template_version: to_version.clone(),
            last_migration: Some(now),
            conflicts: result.conflicts.clone(),
        })?;

        Ok(Migration {
            migrated: true,
            from_version: state.template_version,
            to_version,
            conflicts: result.conflicts,
            backup: Some(backup),
        })
    }

    /// Replaces DDL-owned keys in the active file with the template values,
    /// keeping every other (DSDA-generated or user) key untouched, after a
    /// backup (R6). Requires `confirm`.
    ///
    /// # Errors
    /// Returns [`Error::ConfirmationRequired`] unless `confirm` is `true`, plus
    /// the same errors as [`DsdaConfigManager::migrate`].
    pub fn reset(&self, confirm: bool) -> Result<Reset, Error> {
        if !confirm {
            return Err(Error::ConfirmationRequired);
        }
        self.ensure_no_run()?;
        let active_path = self.active_path();
        if !active_path.exists() {
            return Err(Error::NotProvisioned);
        }
        let mut document = Document::parse(&fs::read_to_string(&active_path)?);
        for (key, value) in &self.template.values {
            document.set(key, Value::Int(*value));
        }
        for (key, value) in &self.template.strings {
            document.set(key, Value::Str(value.clone()));
        }
        for binding in &self.template.bindings {
            document.set(&binding.action, Value::Binding(binding.wanted.clone()));
        }

        let now = self.clock.now_unix_secs();
        let backup = backups::create(
            &self.backups_dir(),
            &active_path,
            now,
            &self.template.template_version,
        )?;

        let text = document.to_text();
        self.write_atomic(&active_path, text.as_bytes())?;
        self.write_atomic(
            &self.base_path(&self.template.template_version),
            text.as_bytes(),
        )?;
        self.save_state(&PersistedState {
            schema_version: STATE_SCHEMA_VERSION,
            template_version: self.template.template_version.clone(),
            last_migration: Some(now),
            conflicts: Vec::new(),
        })?;

        Ok(Reset { backup })
    }

    /// The diagnostics snapshot (R8).
    ///
    /// # Errors
    /// Returns [`Error::InvalidState`] when the state file is corrupt.
    pub fn state(&self) -> Result<State, Error> {
        let persisted = self.load_state_opt()?.unwrap_or_default();
        let template_version = if persisted.template_version.is_empty() {
            self.template.template_version.clone()
        } else {
            persisted.template_version
        };
        Ok(State {
            dsda_version_range: self.template.version_range(),
            template_version,
            active_file: self.active_path(),
            last_migration: persisted.last_migration,
            conflicts: persisted.conflicts,
        })
    }

    /// Fails when an engine run currently holds the guard (R4).
    fn ensure_no_run(&self) -> Result<(), Error> {
        if RunGuard::is_active(&self.dir).map_err(Error::from)? {
            let pid = recorded_pid(&self.dir).ok().flatten();
            return Err(Error::RunActive { pid });
        }
        Ok(())
    }

    fn load_state_opt(&self) -> Result<Option<PersistedState>, Error> {
        match fs::read_to_string(self.state_path()) {
            Ok(text) => Ok(Some(serde_json::from_str(&text)?)),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(Error::Io(err)),
        }
    }

    fn save_state(&self, state: &PersistedState) -> Result<(), Error> {
        let mut json = serde_json::to_string_pretty(state)?;
        json.push('\n');
        self.write_atomic(&self.state_path(), json.as_bytes())
    }

    /// Writes `bytes` to `path` atomically: a sibling temporary file is fully
    /// written and flushed, then renamed over the destination (R5).
    fn write_atomic(&self, path: &Path, bytes: &[u8]) -> Result<(), Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = temporary_path(path);
        let mut file = fs::File::create(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        if let Err(err) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(&temporary);
            return Err(Error::Io(err));
        }
        Ok(())
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config".to_owned());
    path.with_file_name(format!(".{name}.tmp.{}.{unique}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;

    use super::*;

    /// A deterministic clock: 1000 for the first migration, then 1001, ...
    #[derive(Debug)]
    struct TestClock(AtomicU64);

    impl TestClock {
        fn new() -> Arc<Self> {
            Arc::new(Self(AtomicU64::new(1000)))
        }
    }

    impl Clock for Arc<TestClock> {
        fn now_unix_secs(&self) -> u64 {
            self.0.fetch_add(1, Ordering::Relaxed)
        }
    }

    fn template(version: &str, values: &[(&str, i64)]) -> Template {
        Template {
            template_version: version.to_owned(),
            dsda_version_min: "0.30.1".to_owned(),
            dsda_version_max: "0.30.1".to_owned(),
            values: values
                .iter()
                .map(|(key, value)| ((*key).to_owned(), *value))
                .collect(),
            strings: Vec::new(),
            bindings: Vec::new(),
        }
    }

    fn manager(config: &Path, template: Template, clock: &Arc<TestClock>) -> DsdaConfigManager {
        DsdaConfigManager::with_template(config, template).with_clock(Box::new(Arc::clone(clock)))
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).expect("read file")
    }

    /// A pristine config directory plus a decoy standalone DSDA install whose
    /// files must never be touched.
    fn scratch() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().expect("temp dir");
        let home = temp.path().join("home");
        let config = home.join(".config/ddl");
        let standalone = home.join(".dsda-doom");
        fs::create_dir_all(&standalone).expect("standalone dir");
        fs::write(standalone.join("dsda-doom.cfg"), "use_game_controller 0\n")
            .expect("standalone cfg");
        (temp, config, standalone)
    }

    #[test]
    fn provisioning_creates_the_private_baseline_and_leaves_dsda_alone() {
        let (_temp, config, standalone) = scratch();
        let clock = TestClock::new();
        let manager = manager(
            &config,
            template("1", &[("use_game_controller", 1), ("use_mouse", 1)]),
            &clock,
        );

        let report = manager.provision().expect("first use");

        assert!(report.created_active);
        assert!(report.created_state);
        let expected = "use_game_controller 1\nuse_mouse 1\n";
        assert_eq!(read(&manager.active_path()), expected);
        assert_eq!(
            read(&manager.base_path("1")),
            expected,
            "the baseline is an exact copy of what was applied"
        );
        assert!(manager.state_path().exists());

        let second = manager.provision().expect("idempotent");
        assert!(!second.created_active);
        assert!(!second.created_state);

        assert_eq!(
            read(&standalone.join("dsda-doom.cfg")),
            "use_game_controller 0\n",
            "a separately installed DSDA config is untouched (D-012)"
        );
        assert!(
            !manager
                .dir()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join(".local/share/dsda-doom")
                .exists(),
            "DDL never creates DSDA's own data directory"
        );
    }

    #[test]
    fn writes_are_refused_while_an_engine_run_is_active() {
        let (_temp, config, _) = scratch();
        let clock = TestClock::new();
        let manager = manager(&config, template("1", &[("use_mouse", 1)]), &clock);
        manager.provision().expect("first use");

        let guard = manager.run_guard().expect("acquire run guard");
        assert!(matches!(
            manager.provision(),
            Err(Error::RunActive { pid: Some(_) })
        ));
        assert!(matches!(manager.migrate(), Err(Error::RunActive { .. })));
        assert!(matches!(manager.reset(true), Err(Error::RunActive { .. })));

        guard.end().expect("release");
        manager.provision().expect("writable again");
    }

    #[test]
    fn a_version_change_merges_all_five_cases_and_writes_a_backup() {
        let (_temp, config, _) = scratch();
        let clock = TestClock::new();
        let first = manager(
            &config,
            template(
                "1",
                &[
                    ("untouched", 1),
                    ("user_only", 10),
                    ("template_only", 10),
                    ("conflict", 10),
                    ("removed", 5),
                ],
            ),
            &clock,
        );
        first.provision().expect("first use");

        // The player changes two keys in DSDA's menu; DSDA rewrites the file and
        // adds its own unknown key.
        let edited =
            "untouched 1\nuser_only 20\ntemplate_only 10\nconflict 20\nremoved 7\nmystery 99\n";
        fs::write(first.active_path(), edited).expect("simulate a DSDA rewrite");

        let second = manager(
            &config,
            template(
                "2",
                &[
                    ("untouched", 2),
                    ("user_only", 10),
                    ("template_only", 12),
                    ("conflict", 12),
                ],
            ),
            &clock,
        );
        let migration = second.migrate().expect("migrate");

        assert!(migration.migrated);
        assert_eq!(migration.from_version, "1");
        assert_eq!(migration.to_version, "2");
        assert_eq!(migration.conflicts.len(), 1);
        assert_eq!(migration.conflicts[0].key, "conflict");

        let merged = read(&second.active_path());
        assert!(merged.contains("untouched 2\n"), "untouched takes T");
        assert!(merged.contains("user_only 20\n"), "user change is kept");
        assert!(
            merged.contains("template_only 12\n"),
            "template-only change wins"
        );
        assert!(
            merged.contains("conflict 20\n"),
            "conflict keeps the user value"
        );
        assert!(merged.contains("removed 7\n"), "a removed key stays as-is");
        assert!(merged.contains("mystery 99\n"), "unknown keys are kept");

        // The backup is the pre-migration file, named <timestamp>-<version>.cfg.
        let backup = migration.backup.expect("a backup path");
        assert_eq!(backup.file_name().unwrap(), "1000-2.cfg");
        assert_eq!(read(&backup), edited);

        // Writes are atomic: no temporary siblings are left behind.
        let leftovers: Vec<_> = fs::read_dir(second.dir())
            .expect("read dir")
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".tmp."))
            .collect();
        assert!(leftovers.is_empty(), "temp files: {leftovers:?}");

        // State records the migration and the unresolved conflict.
        let state = second.state().expect("state");
        assert_eq!(state.template_version, "2");
        assert_eq!(state.last_migration, Some(1000));
        assert_eq!(state.conflicts.len(), 1);
        assert_eq!(
            read(&second.base_path("2")),
            merged,
            "the new baseline is exactly what was written"
        );
    }

    /// A3 (spec 029, R4): migrating the shipped v1 baseline to the current
    /// template adds the Deck video defaults while a value the player changed in
    /// DSDA's menu survives.
    #[test]
    fn a_version_bump_adds_deck_defaults_and_keeps_a_user_change() {
        let (_temp, config, _) = scratch();
        let clock = TestClock::new();

        // The v1 baseline the current install would have: the same control and
        // mouse keys, without the spec-029 video keys or spec-034 bindings.
        let mut v1 = Template::embedded().expect("valid template");
        v1.template_version = "1".to_owned();
        v1.values.retain(|(key, _)| {
            !matches!(
                key.as_str(),
                "use_fullscreen" | "exclusive_fullscreen" | "render_aspect" | "render_stretch_hud"
            )
        });
        v1.strings.clear();
        v1.bindings.clear();

        let first = manager(&config, v1, &clock);
        first.provision().expect("first use");

        // The player turns the gamepad off and picks a different resolution in
        // DSDA's own menu (DSDA rewrites the file, adding its own keys).
        let edited = "use_game_controller 0\nuse_mouse 1\nscreen_resolution \"1920x1080\"\ndsda_engine_key 7\n";
        fs::write(first.active_path(), edited).expect("simulate a DSDA rewrite");

        let second = DsdaConfigManager::with_template(&config, Template::embedded().unwrap())
            .with_clock(Box::new(Arc::clone(&clock)));
        let migration = second.migrate().expect("migrate");

        assert!(migration.migrated);
        assert_eq!(migration.from_version, "1");
        assert_eq!(migration.to_version, "3");

        let merged = read(&second.active_path());
        assert!(
            merged.contains("use_game_controller 0\n"),
            "the player's change is kept:\n{merged}"
        );
        assert!(
            merged.contains("screen_resolution \"1920x1080\"\n"),
            "a user-changed string value is kept:\n{merged}"
        );
        assert!(
            merged.contains("dsda_engine_key 7\n"),
            "keys DDL does not own are kept:\n{merged}"
        );
        // The new defaults the player never touched are added.
        assert!(merged.contains("use_fullscreen 1\n"), "{merged}");
        assert!(merged.contains("exclusive_fullscreen 0\n"), "{merged}");
        assert!(merged.contains("render_aspect 2\n"), "{merged}");
        assert!(merged.contains("render_stretch_hud 2\n"), "{merged}");
        assert!(
            merged.contains("input_reverse 47 -1 2 | 47 -1 2 | 47 -1 2\n"),
            "the spec-034 bindings are added:\n{merged}"
        );
    }

    /// A2 (spec 034, R2): migrating the shipped v2 baseline to v3 applies the
    /// owner's controller layout while a binding the player rebound in DSDA's
    /// menu is kept. DSDA's own save rewrites every binding, so the previous
    /// baseline has none of them.
    #[test]
    fn a_version_bump_applies_the_deck_layout_and_keeps_a_rebound_button() {
        let (_temp, config, _) = scratch();
        let clock = TestClock::new();

        let current = Template::embedded().expect("valid template");

        // v2: the same scalars and strings, no bindings.
        let mut v2 = current.clone();
        v2.template_version = "2".to_owned();
        v2.bindings.clear();

        let first = manager(&config, v2.clone(), &clock);
        first.provision().expect("first use");

        // DSDA saves the whole config on exit: every scalar DDL wrote plus every
        // binding, including the defaults DDL does not own. The player rebound
        // the automap toggle to Start (`pad start` = 6).
        let mut edited = v2.config_text();
        for binding in &current.bindings {
            let joy = if binding.action == "input_map" {
                6
            } else {
                binding.upstream_joy().unwrap()
            };
            let profile = crate::dsda_config::format::BindingProfile {
                keys: binding.upstream.profiles[0].keys.clone(),
                mouse: binding.upstream.profiles[0].mouse,
                joy,
            };
            let stored = crate::dsda_config::format::Binding {
                profiles: vec![profile.clone(), profile.clone(), profile],
            };
            edited.push_str(&format!(
                "{} {}\n",
                binding.action,
                crate::dsda_config::format::Value::Binding(stored).render()
            ));
        }
        fs::write(first.active_path(), &edited).expect("simulate a DSDA save");

        let second = DsdaConfigManager::with_template(&config, current.clone())
            .with_clock(Box::new(Arc::clone(&clock)));
        let migration = second.migrate().expect("migrate");

        assert!(migration.migrated);
        assert_eq!(migration.from_version, "2");
        assert_eq!(migration.to_version, "3");

        let merged = second.active_path();
        let document = crate::dsda_config::format::Document::parse(&read(&merged));
        let joy = |action: &str| {
            let value = document.get(action).expect(action);
            let crate::dsda_config::format::Value::Binding(binding) = value else {
                panic!("{action} is not a binding: {value:?}");
            };
            binding.profiles[0].joy
        };

        assert_eq!(
            joy("input_map"),
            6,
            "the player's rebound button is kept:\n{}",
            read(&merged)
        );
        assert_eq!(
            migration.conflicts.len(),
            1,
            "only the rebound binding conflicts: {:?}",
            migration.conflicts
        );
        assert_eq!(migration.conflicts[0].key, "input_map");

        for (action, button) in [
            ("input_map_zoomin", 11),
            ("input_map_zoomout", 12),
            ("input_autorun", 1),
            ("input_nextweapon", 14),
            ("input_prevweapon", 13),
            ("input_quicksave", 9),
            ("input_quickload", 10),
            ("input_reverse", 2),
        ] {
            assert_eq!(
                joy(action),
                button,
                "the untouched binding takes the new button:\n{}",
                read(&merged)
            );
        }

        // Keyboard defaults survive the pad rebinding.
        assert!(
            read(&merged).contains("input_quicksave 192 -1 9 | 192 -1 9 | 192 -1 9\n"),
            "{}",
            read(&merged)
        );
        assert!(
            read(&merged).contains("input_quickload 195 -1 10 | 195 -1 10 | 195 -1 10\n"),
            "{}",
            read(&merged)
        );
    }

    #[test]
    fn migrate_is_a_no_op_when_the_template_version_is_unchanged() {
        let (_temp, config, _) = scratch();
        let clock = TestClock::new();
        let manager = manager(&config, template("1", &[("use_mouse", 1)]), &clock);
        manager.provision().expect("first use");

        let migration = manager.migrate().expect("migrate");
        assert!(!migration.migrated);
        assert!(migration.backup.is_none());
        assert_eq!(migration.from_version, "1");
    }

    #[test]
    fn a_missing_baseline_is_a_typed_error_and_touches_nothing() {
        let (_temp, config, _) = scratch();
        let clock = TestClock::new();
        let first = manager(&config, template("1", &[("use_mouse", 1)]), &clock);
        first.provision().expect("first use");
        let edited = "use_mouse 0\n";
        fs::write(first.active_path(), edited).expect("edit");

        fs::remove_file(first.base_path("1")).expect("remove baseline");
        let second = manager(&config, template("2", &[("use_mouse", 1)]), &clock);
        let error = second.migrate().expect_err("no baseline");
        assert!(matches!(error, Error::MissingBaseline { ref version } if version == "1"));
        assert_eq!(
            read(&second.active_path()),
            edited,
            "an impossible merge leaves the active file alone"
        );
    }

    #[test]
    fn repeated_migrations_rotate_the_backups_to_ten() {
        let (_temp, config, _) = scratch();
        let clock = TestClock::new();
        manager(&config, template("1", &[("key", 1)]), &clock)
            .provision()
            .expect("first use");

        for version in 2..=13 {
            let next = template(&version.to_string(), &[("key", version as i64)]);
            manager(&config, next, &clock).migrate().expect("migrate");
        }

        let backups = fs::read_dir(config.join(DSDA_DIR).join("backups")).expect("backups");
        assert_eq!(backups.count(), MAX_BACKUPS, "only ten backups are kept");
    }

    #[test]
    fn reset_requires_confirmation_and_preserves_engine_generated_keys() {
        let (_temp, config, _) = scratch();
        let clock = TestClock::new();
        let manager = manager(
            &config,
            template("1", &[("use_game_controller", 1)]),
            &clock,
        );
        manager.provision().expect("first use");

        let edited = "use_game_controller 0\nengine_generated 42\n";
        fs::write(manager.active_path(), edited).expect("edit");

        assert!(matches!(
            manager.reset(false),
            Err(Error::ConfirmationRequired)
        ));
        assert_eq!(
            read(&manager.active_path()),
            edited,
            "declined reset changes nothing"
        );

        let reset = manager.reset(true).expect("reset");
        assert_eq!(reset.backup.file_name().unwrap(), "1000-1.cfg");
        assert_eq!(read(&reset.backup), edited);

        let after = read(&manager.active_path());
        assert!(
            after.contains("use_game_controller 1\n"),
            "the DDL-owned key is back to the template value"
        );
        assert!(
            after.contains("engine_generated 42\n"),
            "DSDA-generated keys are left untouched"
        );
    }

    #[test]
    fn state_snapshot_reports_the_range_template_path_and_conflicts() {
        let (_temp, config, _) = scratch();
        let home = config.parent().expect("home");
        let clock = TestClock::new();
        let manager = manager(&config, template("1", &[("use_mouse", 1)]), &clock);

        // Before provisioning the snapshot still answers, from the template.
        let fresh = manager.state().expect("state");
        assert_eq!(fresh.template_version, "1");
        assert_eq!(fresh.dsda_version_range.to_string(), "0.30.1");
        assert_eq!(fresh.last_migration, None);
        assert!(fresh.conflicts.is_empty());

        manager.provision().expect("first use");
        let state = manager.state().expect("state");
        assert_eq!(state.active_file, manager.active_path());
        assert!(state.active_file_display(home).starts_with("~/"));
    }

    // Referenced by the rotation test above through the re-exported constant.
    use super::super::backups::MAX_BACKUPS;
}

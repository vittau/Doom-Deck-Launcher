//! One-time import of a pre-Flatpak (tarball-era) install (spec 059, R5).
//!
//! A player who ran DDL from the GitHub tarball has state in the host's
//! `~/.config/ddl` and `~/.local/share/ddl`. The Flatpak keeps its own state
//! under `~/.var/app/<id>/`, but with read-only home access it can still *read*
//! the old directories. When the sandbox has no state of its own, DDL offers to
//! copy the old setup across, once.
//!
//! The import
//!
//! * **copies, never moves**: the source tree is only ever read;
//! * rewrites absolute paths that pointed into the old config or data
//!   directory so they point into the sandbox's own directories, and leaves
//!   every other path alone, so IWAD references and IWAD links still name the
//!   player's own files;
//! * keeps older shapes as they are: the readers of spec 055 upgrade them when
//!   DDL loads the copied files, so any state those readers accept imports;
//! * drops engines installed in AppImage mode (they would need FUSE; the
//!   Flatpak reinstalls and extracts them) and reports their versions;
//! * stages everything next to the destination first and moves the pieces into
//!   place at the end, so a failed or cancelled import leaves no half state.
//!
//! Caches, logs and lock files are not imported.

use std::fmt;
use std::fs;
use std::io::{self, Read as _, Write as _};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::paths::{APP_DIR, Paths};
use super::sandbox::Sandbox;
use crate::downloads::CancellationToken;
use crate::engines::dsda::{InstalledState, Mode, STATE_FILE};

/// Top-level entries copied from the old config directory.
pub const CONFIG_ENTRIES: &[&str] = &["app.json", "iwads.json", "profiles", "dsda"];

/// Top-level entries copied from the old data directory.
pub const DATA_ENTRIES: &[&str] = &["library", "custom", "engines", "profiles", "iwads"];

/// The engine directory under the data directory.
const ENGINES: [&str; 2] = ["engines", "dsda-doom"];

/// File remembering the player's answer, inside the state directory.
pub const DECISION_FILE: &str = "legacy-import.json";

/// Staging directory (inside each destination root) used while copying.
const STAGING: &str = ".import-staging";

/// JSON and config text larger than this is copied byte for byte.
const MAX_REWRITE_BYTES: u64 = 16 * 1024 * 1024;

/// Nesting limit for the copied trees.
const MAX_DEPTH: usize = 24;

/// Entries copied from one directory, as a safety bound.
const MAX_DIR_ENTRIES: usize = 100_000;

/// Where a tarball-era install keeps its state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyInstall {
    /// The old config directory (`~/.config/ddl`).
    pub config: PathBuf,
    /// The old data directory (`~/.local/share/ddl`).
    pub data: PathBuf,
}

impl LegacyInstall {
    /// The tarball layout under `home`.
    #[must_use]
    pub fn under(home: &Path) -> Self {
        Self {
            config: home.join(".config").join(APP_DIR),
            data: home.join(".local/share").join(APP_DIR),
        }
    }

    /// Whether the old directories hold anything worth importing.
    #[must_use]
    pub fn has_state(&self) -> bool {
        has_state(&self.config, &self.data)
    }
}

/// The player's recorded answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    /// The setup was imported.
    Imported,
    /// The player declined; do not offer again.
    Declined,
}

#[derive(Serialize, Deserialize)]
struct DecisionFile {
    schema_version: u32,
    decision: Decision,
}

/// What an import did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// Regular files copied.
    pub files: usize,
    /// Bytes copied.
    pub bytes: u64,
    /// JSON/config files whose paths were rewritten.
    pub rewritten: usize,
    /// Engine versions left behind because they run through FUSE.
    pub skipped_engines: Vec<String>,
    /// Existing sandbox entries moved aside as `<name>.pre-import`.
    pub set_aside: Vec<PathBuf>,
}

/// Why an import failed.
#[derive(Debug)]
pub enum Error {
    /// The player cancelled.
    Cancelled,
    /// A file operation failed.
    Io {
        /// What was being done.
        context: &'static str,
        /// The underlying error.
        source: io::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => f.write_str("the import was cancelled"),
            Self::Io { context, source } => write!(f, "could not {context}: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Cancelled => None,
        }
    }
}

fn io_error(context: &'static str) -> impl FnOnce(io::Error) -> Error {
    move |source| Error::Io { context, source }
}

/// The old install to offer importing, if the offer applies.
///
/// All of these must hold: DDL is sandboxed, the player has not answered
/// before, the sandbox has no state of its own, and the host's old directories
/// (which must not be the sandbox's own) hold state.
#[must_use]
pub fn find(sandbox: &Sandbox, home: &Path, paths: &Paths) -> Option<LegacyInstall> {
    if !sandbox.is_flatpak() || recorded_decision(paths).is_some() || has_own_state(paths) {
        return None;
    }
    let legacy = LegacyInstall::under(home);
    if same_dir(&legacy.config, &paths.config) || same_dir(&legacy.data, &paths.data) {
        return None;
    }
    legacy.has_state().then_some(legacy)
}

/// Whether the sandbox's own directories already hold player state. Lock files,
/// the provisioned DSDA config, caches and logs do not count.
#[must_use]
pub fn has_own_state(paths: &Paths) -> bool {
    has_state(&paths.config, &paths.data)
}

fn has_state(config: &Path, data: &Path) -> bool {
    config.join("app.json").is_file()
        || config.join("iwads.json").is_file()
        || non_empty_dir(&config.join("profiles"))
        || non_empty_dir(&data.join("library"))
        || non_empty_dir(&data.join("custom"))
        || non_empty_dir(&data.join("profiles"))
        || data.join(ENGINES[0]).join(ENGINES[1]).join(STATE_FILE).is_file()
}

fn non_empty_dir(path: &Path) -> bool {
    fs::read_dir(path).is_ok_and(|mut entries| entries.next().is_some())
}

fn same_dir(a: &Path, b: &Path) -> bool {
    a == b
        || matches!(
            (fs::canonicalize(a), fs::canonicalize(b)),
            (Ok(a), Ok(b)) if a == b
        )
}

fn decision_path(paths: &Paths) -> PathBuf {
    paths.state.join(DECISION_FILE)
}

/// The answer the player gave before, if any.
#[must_use]
pub fn recorded_decision(paths: &Paths) -> Option<Decision> {
    let text = fs::read_to_string(decision_path(paths)).ok()?;
    let file: DecisionFile = serde_json::from_str(&text).ok()?;
    (file.schema_version == 1).then_some(file.decision)
}

/// Remembers the player's answer (atomic write).
///
/// # Errors
///
/// Returns the I/O error when the state directory cannot be written.
pub fn record_decision(paths: &Paths, decision: Decision) -> io::Result<()> {
    fs::create_dir_all(&paths.state)?;
    let target = decision_path(paths);
    let temporary = target.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(&DecisionFile {
        schema_version: 1,
        decision,
    })
    .map_err(io::Error::other)?;
    let mut file = fs::File::create(&temporary)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    fs::rename(temporary, target)
}

/// Copies the old install into the sandbox's directories.
///
/// Callers hold the state lock. Existing entries at a destination are moved
/// aside as `<name>.pre-import` (never deleted).
///
/// # Errors
///
/// Returns [`Error::Cancelled`] when `cancel` fires and [`Error::Io`] when a
/// file operation fails. Either way the destination directories are left as
/// they were: only the staging directories are removed.
pub fn import(
    legacy: &LegacyInstall,
    paths: &Paths,
    cancel: &CancellationToken,
) -> Result<Report, Error> {
    let config_stage = paths.config.join(STAGING);
    let data_stage = paths.data.join(STAGING);
    let result = stage_and_commit(legacy, paths, &config_stage, &data_stage, cancel);
    let _ = fs::remove_dir_all(&config_stage);
    let _ = fs::remove_dir_all(&data_stage);
    result
}

fn stage_and_commit(
    legacy: &LegacyInstall,
    paths: &Paths,
    config_stage: &Path,
    data_stage: &Path,
    cancel: &CancellationToken,
) -> Result<Report, Error> {
    for stage in [config_stage, data_stage] {
        let _ = fs::remove_dir_all(stage);
        fs::create_dir_all(stage).map_err(io_error("create the import staging directory"))?;
    }
    let mut copier = Copier {
        rewrites: vec![
            (legacy.data.clone(), paths.data.clone()),
            (legacy.config.clone(), paths.config.clone()),
        ],
        cancel,
        report: Report::default(),
    };

    for name in CONFIG_ENTRIES {
        copier.copy_entry(&legacy.config.join(name), &config_stage.join(name), 0)?;
    }
    for name in DATA_ENTRIES {
        if *name == ENGINES[0] {
            copier.copy_engines(&legacy.data.join(name), &data_stage.join(name))?;
        } else {
            copier.copy_entry(&legacy.data.join(name), &data_stage.join(name), 0)?;
        }
    }

    let mut report = copier.report;
    commit(config_stage, &paths.config, &mut report)?;
    commit(data_stage, &paths.data, &mut report)?;
    Ok(report)
}

/// Moves each staged top-level entry into `root`, setting aside what is there.
fn commit(stage: &Path, root: &Path, report: &mut Report) -> Result<(), Error> {
    let mut names: Vec<_> = fs::read_dir(stage)
        .map_err(io_error("list the staged import"))?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name())
        .collect();
    names.sort();
    for name in names {
        let destination = root.join(&name);
        if fs::symlink_metadata(&destination).is_ok() {
            let aside = free_name(&destination, "pre-import");
            fs::rename(&destination, &aside)
                .map_err(io_error("set the existing entry aside before importing"))?;
            report.set_aside.push(aside);
        }
        fs::rename(stage.join(&name), &destination)
            .map_err(io_error("move the imported entry into place"))?;
    }
    Ok(())
}

fn free_name(path: &Path, suffix: &str) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut candidate = path.with_file_name(format!("{name}.{suffix}"));
    let mut attempt = 1;
    while fs::symlink_metadata(&candidate).is_ok() {
        attempt += 1;
        candidate = path.with_file_name(format!("{name}.{suffix}-{attempt}"));
    }
    candidate
}

struct Copier<'a> {
    /// `(old prefix, new prefix)`, longest-purpose first.
    rewrites: Vec<(PathBuf, PathBuf)>,
    cancel: &'a CancellationToken,
    report: Report,
}

impl Copier<'_> {
    fn copy_entry(&mut self, source: &Path, target: &Path, depth: usize) -> Result<(), Error> {
        if self.cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let Ok(metadata) = fs::symlink_metadata(source) else {
            return Ok(());
        };
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            self.copy_symlink(source, target)
        } else if file_type.is_dir() {
            if depth >= MAX_DEPTH {
                return Ok(());
            }
            fs::create_dir_all(target).map_err(io_error("create an imported directory"))?;
            let mut children: Vec<_> = fs::read_dir(source)
                .map_err(io_error("read the old install"))?
                .filter_map(Result::ok)
                .take(MAX_DIR_ENTRIES)
                .map(|entry| entry.file_name())
                .filter(|name| !is_lock(name))
                .collect();
            children.sort();
            for name in children {
                self.copy_entry(&source.join(&name), &target.join(&name), depth + 1)?;
            }
            Ok(())
        } else if file_type.is_file() {
            self.copy_file(source, target, metadata.len())
        } else {
            Ok(())
        }
    }

    fn copy_file(&mut self, source: &Path, target: &Path, len: u64) -> Result<(), Error> {
        let extension = source
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase);
        if len <= MAX_REWRITE_BYTES
            && matches!(extension.as_deref(), Some("json" | "cfg"))
            && let Some(bytes) = self.rewritten(source, extension.as_deref() == Some("json"))
        {
            fs::write(target, &bytes).map_err(io_error("write an imported file"))?;
            copy_permissions(source, target);
            self.report.files += 1;
            self.report.bytes += bytes.len() as u64;
            self.report.rewritten += 1;
            return Ok(());
        }
        let copied = fs::copy(source, target).map_err(io_error("copy a file from the old install"))?;
        self.report.files += 1;
        self.report.bytes += copied;
        Ok(())
    }

    /// The file's bytes with paths rewritten, or `None` when nothing changes
    /// (or it cannot be read as text/JSON, in which case it is copied as is).
    fn rewritten(&self, source: &Path, json: bool) -> Option<Vec<u8>> {
        let mut text = String::new();
        fs::File::open(source)
            .ok()?
            .take(MAX_REWRITE_BYTES)
            .read_to_string(&mut text)
            .ok()?;
        if json {
            let mut value: serde_json::Value = serde_json::from_str(&text).ok()?;
            if !self.rewrite_json(&mut value) {
                return None;
            }
            let mut out = serde_json::to_string_pretty(&value).ok()?;
            out.push('\n');
            Some(out.into_bytes())
        } else {
            let mut changed = false;
            let mut out = text;
            for (old, new) in &self.rewrites {
                let (Some(old), Some(new)) = (old.to_str(), new.to_str()) else {
                    continue;
                };
                let from = format!("{old}/");
                if out.contains(&from) {
                    out = out.replace(&from, &format!("{new}/"));
                    changed = true;
                }
            }
            changed.then(|| out.into_bytes())
        }
    }

    /// Rewrites every string that is, or starts with, an old directory.
    fn rewrite_json(&self, value: &mut serde_json::Value) -> bool {
        match value {
            serde_json::Value::String(text) => {
                if let Some(new) = self.rewrite_str(text) {
                    *text = new;
                    true
                } else {
                    false
                }
            }
            serde_json::Value::Array(items) => items
                .iter_mut()
                .fold(false, |changed, item| self.rewrite_json(item) | changed),
            serde_json::Value::Object(map) => map
                .values_mut()
                .fold(false, |changed, item| self.rewrite_json(item) | changed),
            _ => false,
        }
    }

    fn rewrite_str(&self, text: &str) -> Option<String> {
        self.rewrites.iter().find_map(|(old, new)| {
            let (old, new) = (old.to_str()?, new.to_str()?);
            let rest = text.strip_prefix(old)?;
            (rest.is_empty() || rest.starts_with('/')).then(|| format!("{new}{rest}"))
        })
    }

    fn rewrite_path(&self, path: &Path) -> PathBuf {
        path.to_str()
            .and_then(|text| self.rewrite_str(text))
            .map_or_else(|| path.to_path_buf(), PathBuf::from)
    }

    #[cfg(unix)]
    fn copy_symlink(&mut self, source: &Path, target: &Path) -> Result<(), Error> {
        let destination = fs::read_link(source).map_err(io_error("read a link in the old install"))?;
        // An absolute target into the old directories follows the data to its
        // new home; any other target (an IWAD, say) keeps pointing at the
        // player's own file.
        let destination = if destination.is_absolute() {
            self.rewrite_path(&destination)
        } else {
            destination
        };
        std::os::unix::fs::symlink(destination, target)
            .map_err(io_error("recreate a link from the old install"))
    }

    #[cfg(not(unix))]
    fn copy_symlink(&mut self, _source: &Path, _target: &Path) -> Result<(), Error> {
        Ok(())
    }

    /// Copies the engine directory: installed versions that do not need FUSE.
    fn copy_engines(&mut self, source: &Path, target: &Path) -> Result<(), Error> {
        let source = source.join(ENGINES[1]);
        let state_path = source.join(STATE_FILE);
        if !state_path.is_file() {
            return Ok(());
        }
        let Ok(state) = InstalledState::load(&state_path) else {
            // The 055 recovery screen owns unreadable state; the Flatpak starts
            // without an engine rather than importing something it cannot read.
            tracing::warn!(path = %state_path.display(), "not importing an unreadable engine state");
            return Ok(());
        };
        let target = target.join(ENGINES[1]);
        let mut kept = InstalledState::new();
        for mut engine in state.installed {
            if engine.mode == Mode::Appimage {
                self.report.skipped_engines.push(engine.version);
                continue;
            }
            if engine.mode == Mode::Extracted {
                if !is_safe_component(&engine.version) {
                    continue;
                }
                self.copy_entry(
                    &source.join(&engine.version),
                    &target.join(&engine.version),
                    0,
                )?;
            }
            engine.path = self.rewrite_path(&engine.path);
            kept.installed.push(engine);
        }
        if kept.installed.is_empty() {
            return Ok(());
        }
        let has = |version: &Option<String>| {
            version
                .as_ref()
                .is_some_and(|v| kept.installed.iter().any(|e| &e.version == v))
        };
        kept.active = state.active.clone().filter(|_| has(&state.active));
        kept.previous_good = state.previous_good.clone().filter(|_| has(&state.previous_good));
        kept.save(&target.join(STATE_FILE))
            .map_err(|error| Error::Io {
                context: "write the imported engine state",
                source: io::Error::other(error.to_string()),
            })?;
        self.report.files += 1;
        self.report.rewritten += 1;
        Ok(())
    }
}

fn is_lock(name: &std::ffi::OsStr) -> bool {
    name.to_str().is_some_and(|name| name.ends_with(".lock"))
}

fn is_safe_component(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\', '\0'])
}

fn copy_permissions(source: &Path, target: &Path) {
    if let Ok(metadata) = fs::metadata(source) {
        let _ = fs::set_permissions(target, metadata.permissions());
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} files ({} bytes), {} rewritten, {} engine(s) left to reinstall",
            self.files,
            self.bytes,
            self.rewritten,
            self.skipped_engines.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/state/v0.0.7-deck"
    );
    const APP: &str = "io.github.vittau.Doom-Deck-Launcher";

    fn sandbox() -> Sandbox {
        Sandbox::flatpak(Some(APP.to_owned()))
    }

    /// The sandbox's own, still empty, directories.
    fn sandbox_paths(home: &Path) -> Paths {
        let app = home.join(".var/app").join(APP);
        Paths {
            config: app.join("config").join(APP_DIR),
            data: app.join("data").join(APP_DIR),
            cache: app.join("cache").join(APP_DIR),
            state: app.join(".local/state").join(APP_DIR),
        }
    }

    /// Copies a spec 055 fixture tree to where a tarball install keeps it and
    /// points its recorded paths at that real location, as a real host's would.
    fn install_fixture(home: &Path) -> LegacyInstall {
        let legacy = LegacyInstall::under(home);
        for (from, to, recorded) in [
            ("config", &legacy.config, "/ddl-fixture/config"),
            ("data", &legacy.data, "/ddl-fixture/data"),
        ] {
            copy_tree(&Path::new(FIXTURE).join(from), to, recorded, to);
        }
        legacy
    }

    fn copy_tree(from: &Path, to: &Path, recorded: &str, real: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap().flatten() {
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target, recorded, real);
            } else {
                let text = fs::read_to_string(entry.path()).unwrap();
                fs::write(
                    target,
                    text.replace(recorded, real.to_str().unwrap()),
                )
                .unwrap();
            }
        }
    }

    /// Every file and link below `root`: `relative path -> content or target`.
    fn snapshot(root: &Path) -> BTreeMap<PathBuf, String> {
        fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, String>) {
            let Ok(entries) = fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let relative = path.strip_prefix(root).unwrap().to_path_buf();
                let kind = entry.file_type().unwrap();
                if kind.is_symlink() {
                    out.insert(relative, format!("-> {}", fs::read_link(&path).unwrap().display()));
                } else if kind.is_dir() {
                    out.insert(relative, "dir".into());
                    walk(root, &path, out);
                } else {
                    out.insert(
                        relative,
                        String::from_utf8_lossy(&fs::read(&path).unwrap()).into_owned(),
                    );
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(root, root, &mut out);
        out
    }

    fn host_tree(home: &Path) -> (BTreeMap<PathBuf, String>, BTreeMap<PathBuf, String>) {
        let legacy = LegacyInstall::under(home);
        (snapshot(&legacy.config), snapshot(&legacy.data))
    }

    #[test]
    fn the_offer_needs_a_sandbox_an_empty_own_dir_and_an_old_install() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path();
        let paths = sandbox_paths(home);
        // No old install yet.
        assert_eq!(find(&sandbox(), home, &paths), None);

        let legacy = install_fixture(home);
        assert_eq!(find(&sandbox(), home, &paths), Some(legacy.clone()));
        // Outside the sandbox the offer never applies.
        assert_eq!(find(&Sandbox::none(), home, &paths), None);

        // The provisioned DSDA config and the lock are not player state.
        fs::create_dir_all(paths.config.join("dsda")).unwrap();
        fs::write(paths.config.join("dsda/active.cfg"), "x").unwrap();
        fs::write(paths.config.join("state.lock"), "").unwrap();
        assert_eq!(find(&sandbox(), home, &paths), Some(legacy));

        // Real state of its own: no offer.
        fs::write(paths.config.join("app.json"), "{}").unwrap();
        assert_eq!(find(&sandbox(), home, &paths), None);
    }

    #[test]
    fn a_recorded_answer_stops_the_offer() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path();
        let paths = sandbox_paths(home);
        install_fixture(home);
        assert_eq!(recorded_decision(&paths), None);

        record_decision(&paths, Decision::Declined).unwrap();

        assert_eq!(recorded_decision(&paths), Some(Decision::Declined));
        assert_eq!(find(&sandbox(), home, &paths), None);
        // A garbled answer file is no answer.
        fs::write(paths.state.join(DECISION_FILE), "nonsense").unwrap();
        assert_eq!(recorded_decision(&paths), None);
    }

    /// Spec 059 A3: a spec 055 fixture tree imports into an empty sandbox
    /// directory with paths rewritten, IWAD links untouched and the originals
    /// byte-identical.
    #[test]
    fn importing_a_fixture_tree_rewrites_paths_and_leaves_the_source_alone() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path();
        let paths = sandbox_paths(home);
        let legacy = install_fixture(home);
        // The player's IWAD, linked from DDL's data dir the way DDL does (D-017).
        let iwad = home.join("Games/doom2.wad");
        fs::create_dir_all(iwad.parent().unwrap()).unwrap();
        fs::write(&iwad, b"IWAD").unwrap();
        #[cfg(unix)]
        {
            let links = legacy.data.join("iwads/doom2-1.9");
            fs::create_dir_all(&links).unwrap();
            std::os::unix::fs::symlink(&iwad, links.join("doom2.wad")).unwrap();
            // A link that points back into DDL's own data is carried along.
            std::os::unix::fs::symlink(
                legacy.data.join("library/sigil/active.json"),
                links.join("internal.wad"),
            )
            .unwrap();
        }
        let before = host_tree(home);

        let report = import(&legacy, &paths, &CancellationToken::new()).expect("import");

        assert_eq!(before, host_tree(home), "the old install is only read");
        assert!(report.files > 5, "{report}");
        assert!(report.rewritten > 0, "{report}");
        assert!(report.set_aside.is_empty());
        assert!(paths.config.join("app.json").is_file());
        assert!(paths.config.join("profiles/fixture-sigil.json").is_file());
        assert!(paths.data.join("custom/fixture-custom.json").is_file());
        assert!(paths.data.join("library/sigil/active.json").is_file());
        assert!(!paths.config.join(STAGING).exists());
        assert!(!paths.data.join(STAGING).exists());

        // Nothing imported still names the old data directory…
        let old_data = legacy.data.to_str().unwrap();
        let new_data = paths.data.to_str().unwrap();
        let imported = snapshot(&paths.data);
        for (path, content) in &imported {
            assert!(!content.contains(old_data), "{} still names {old_data}", path.display());
        }
        // …and the recorded paths followed the data.
        let manifest = &imported[Path::new("library/sigil/active.json")];
        assert!(manifest.contains(&format!("{new_data}/library/sigil/generations/fixture/")));

        // IWAD references stay pointing at the player's files.
        let registry = fs::read_to_string(paths.config.join("iwads.json")).unwrap();
        assert!(registry.contains("/ddl-fixture/iwads-one/doom2.wad"));
        // The manual engine lives outside DDL's data and is untouched.
        let engines = fs::read_to_string(paths.data.join("engines/dsda-doom/installed.json")).unwrap();
        assert!(engines.contains("\"/ddl-fixture/dsda\""));
        #[cfg(unix)]
        {
            assert_eq!(
                fs::read_link(paths.data.join("iwads/doom2-1.9/doom2.wad")).unwrap(),
                iwad,
                "IWAD links are untouched"
            );
            assert_eq!(
                fs::read_link(paths.data.join("iwads/doom2-1.9/internal.wad")).unwrap(),
                paths.data.join("library/sigil/active.json")
            );
        }

        // Now the sandbox has state of its own: no second offer.
        assert_eq!(find(&sandbox(), home, &paths), None);
    }

    #[test]
    fn engines_that_need_fuse_are_left_to_reinstall_and_extracted_ones_follow() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path();
        let paths = sandbox_paths(home);
        let legacy = LegacyInstall::under(home);
        let engines = legacy.data.join("engines/dsda-doom");
        fs::create_dir_all(engines.join("0.30.0")).unwrap();
        fs::write(engines.join("0.30.0/dsda-doom.AppImage"), b"image").unwrap();
        fs::create_dir_all(engines.join("0.30.1/squashfs-root")).unwrap();
        fs::write(engines.join("0.30.1/dsda-doom.AppImage"), b"image").unwrap();
        fs::write(engines.join("0.30.1/squashfs-root/AppRun"), b"#!/bin/sh\n").unwrap();
        let engine = |version: &str, mode: &str, path: PathBuf| {
            serde_json::json!({
                "version": version, "path": path, "mode": mode, "sha256": "00",
                "reviewed": true, "origin": "downloaded", "installed_at": 1
            })
        };
        let state = serde_json::json!({
            "schema_version": 1,
            "installed": [
                engine("0.30.0", "appimage", engines.join("0.30.0/dsda-doom.AppImage")),
                engine("0.30.1", "extracted", engines.join("0.30.1/squashfs-root/AppRun")),
            ],
            "active": "0.30.0",
            "previous_good": "0.30.1",
        });
        fs::write(engines.join("installed.json"), state.to_string()).unwrap();
        assert!(legacy.has_state());

        let report = import(&legacy, &paths, &CancellationToken::new()).expect("import");

        assert_eq!(report.skipped_engines, ["0.30.0"]);
        let new_engines = paths.data.join("engines/dsda-doom");
        assert!(!new_engines.join("0.30.0").exists(), "FUSE-mode engine not copied");
        assert!(new_engines.join("0.30.1/squashfs-root/AppRun").is_file());
        let loaded = InstalledState::load(&new_engines.join(STATE_FILE)).unwrap();
        assert_eq!(loaded.installed.len(), 1);
        assert_eq!(loaded.installed[0].path, new_engines.join("0.30.1/squashfs-root/AppRun"));
        assert_eq!(loaded.active, None, "the dropped engine is no longer active");
        assert_eq!(loaded.previous_good.as_deref(), Some("0.30.1"));
    }

    #[test]
    fn existing_sandbox_entries_are_set_aside_not_deleted() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path();
        let paths = sandbox_paths(home);
        let legacy = install_fixture(home);
        // What a first start provisions before the answer comes.
        fs::create_dir_all(paths.config.join("dsda")).unwrap();
        fs::write(paths.config.join("dsda/active.cfg"), "fresh").unwrap();

        let report = import(&legacy, &paths, &CancellationToken::new()).unwrap();

        assert_eq!(report.set_aside, [paths.config.join("dsda.pre-import")]);
        assert_eq!(
            fs::read_to_string(paths.config.join("dsda.pre-import/active.cfg")).unwrap(),
            "fresh"
        );
        assert!(paths.config.join("dsda/active.cfg").is_file());
        assert_ne!(
            fs::read_to_string(paths.config.join("dsda/active.cfg")).unwrap(),
            "fresh"
        );
    }

    #[test]
    fn a_cancelled_import_leaves_no_half_state() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path();
        let paths = sandbox_paths(home);
        let legacy = install_fixture(home);
        let cancel = CancellationToken::new();
        cancel.cancel();

        let error = import(&legacy, &paths, &cancel).unwrap_err();

        assert!(matches!(error, Error::Cancelled));
        assert!(!has_own_state(&paths));
        assert!(snapshot(&paths.config).is_empty() && snapshot(&paths.data).is_empty());
    }

    #[test]
    fn lock_files_and_unlisted_entries_are_not_imported() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path();
        let paths = sandbox_paths(home);
        let legacy = install_fixture(home);
        fs::write(legacy.config.join("state.lock"), "").unwrap();
        fs::write(legacy.config.join("dsda/run.lock"), "123").unwrap();
        fs::create_dir_all(legacy.data.join("catalog")).unwrap();
        fs::write(legacy.data.join("catalog/catalog.json"), "{}").unwrap();

        import(&legacy, &paths, &CancellationToken::new()).unwrap();

        assert!(!paths.config.join("state.lock").exists());
        assert!(!paths.config.join("dsda/run.lock").exists());
        assert!(!paths.data.join("catalog").exists());
    }
}

//! GOG and other store IWAD discovery (spec 021, R2 and R4).
//!
//! Spec 007's Steam discovery only knows Steam libraries. This module adds the
//! store routes researched in `docs/research/iwad-identification.md` §4.1:
//!
//! * **Heroic Games Launcher** — reads the GOG install record
//!   `<config>/heroic/gog_store/installed.json` (native `~/.config`, Flatpak
//!   `~/.var/app/com.heroicgameslauncher.hgl/config`) and takes each record's
//!   `install_path`.
//! * **minigalaxy** — reads `install_dir` from
//!   `<config>/minigalaxy/config.json` (native and Flatpak).
//! * **GOG offline installers** — probes the documented default
//!   `~/GOG Games`.
//!
//! All of it is **read-only** and tolerant: a missing or garbled record yields
//! no candidates rather than an error, and record files and directory listings
//! are size-bounded. Because the internal folder layout of a GOG DOOM product
//! is *Needs Deck Validation* (research §4.1), DDL never asserts one: it treats
//! a launcher-recorded install directory as a root and looks for `*.wad` files
//! in that directory and (bounded) one level of its subfolders, letting the
//! content identifier decide. A wrong or empty directory simply yields nothing.
//!
//! R4's generic roots (`DOOMWADDIR`/`DOOMWADPATH`, `/usr/share/games/doom`,
//! `~/.local/share/games/doom`) live in [`standard_roots`] and are only used
//! when the user opts in; the caller decides, never this module.

use std::path::{Path, PathBuf};

/// Largest launcher record file DDL will read.
pub const MAX_RECORD_BYTES: u64 = 4 * 1024 * 1024;

/// Largest number of candidate files returned by one discovery.
pub const MAX_CANDIDATES: usize = 4096;

/// Largest number of directory entries examined per directory level.
const MAX_ENTRIES: usize = 4096;

/// Which launcher owns a discovered GOG install.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GogLauncher {
    /// Heroic Games Launcher (`gog_store/installed.json`).
    Heroic,
    /// minigalaxy (`config.json` `install_dir`).
    Minigalaxy,
    /// A GOG offline installer's default `~/GOG Games`.
    Offline,
}

impl GogLauncher {
    /// The label shown in the IWAD list and `ddl iwad scan` (spec 021, R3).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            GogLauncher::Heroic => "GOG via Heroic",
            GogLauncher::Minigalaxy => "GOG via minigalaxy",
            GogLauncher::Offline => "GOG offline installer",
        }
    }
}

/// One candidate file found through a store, with the store it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreCandidate {
    /// The `*.wad` file.
    pub path: PathBuf,
    /// The launcher that pointed DDL at it.
    pub launcher: GogLauncher,
}

/// The outcome of store discovery.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoreDiscovery {
    /// Candidate WAD files, sorted and de-duplicated.
    pub candidates: Vec<StoreCandidate>,
    /// Record files that exist but could not be read or parsed. The caller
    /// logs these; they are never an error and never a dialog.
    pub unreadable: Vec<PathBuf>,
}

/// Heroic's config directories for `home` (native, then Flatpak).
#[must_use]
pub fn heroic_config_dirs(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".config/heroic"),
        home.join(".var/app/com.heroicgameslauncher.hgl/config/heroic"),
    ]
}

/// Heroic's GOG install-record files for `home`.
#[must_use]
pub fn heroic_installed_files(home: &Path) -> Vec<PathBuf> {
    heroic_config_dirs(home)
        .into_iter()
        .map(|dir| dir.join("gog_store/installed.json"))
        .collect()
}

/// minigalaxy's config directories for `home` (native, then Flatpak).
#[must_use]
pub fn minigalaxy_config_dirs(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".config/minigalaxy"),
        home.join(".var/app/io.github.sharkwouter.Minigalaxy/config/minigalaxy"),
    ]
}

/// minigalaxy's config files for `home`.
#[must_use]
pub fn minigalaxy_config_files(home: &Path) -> Vec<PathBuf> {
    minigalaxy_config_dirs(home)
        .into_iter()
        .map(|dir| dir.join("config.json"))
        .collect()
}

/// Heroic's documented default install directory, `~/Games/Heroic` (§4.1).
#[must_use]
pub fn heroic_default_dir(home: &Path) -> PathBuf {
    home.join("Games/Heroic")
}

/// GOG's documented default install directory, `~/GOG Games` (§4.1).
#[must_use]
pub fn gog_games_dir(home: &Path) -> PathBuf {
    home.join("GOG Games")
}

/// Extracts every `install_path` string from a Heroic `installed.json`.
///
/// Tolerant by design: any parse error, or a document without `install_path`
/// values, yields an empty list. The record's exact wrapper object is not
/// asserted (research §4.1 records `install_path`; the surrounding schema may
/// change), so every string value under an `install_path` key is collected.
#[must_use]
pub fn install_paths_from_heroic(text: &str) -> Vec<PathBuf> {
    parse_heroic_installed(text).unwrap_or_default()
}

/// Parses a Heroic `installed.json`, distinguishing a parse failure from an
/// empty result so callers can log a garbled record.
fn parse_heroic_installed(text: &str) -> Result<Vec<PathBuf>, serde_json::Error> {
    let value = serde_json::from_str::<serde_json::Value>(text)?;
    let mut paths = Vec::new();
    collect_install_paths(&value, &mut paths);
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Recursively collects strings stored under an `install_path` key.
fn collect_install_paths(value: &serde_json::Value, out: &mut Vec<PathBuf>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                if key == "install_path" {
                    if let serde_json::Value::String(text) = child {
                        let path = Path::new(text);
                        if path.is_absolute() {
                            out.push(path.to_path_buf());
                        }
                    }
                } else {
                    collect_install_paths(child, out);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_install_paths(item, out);
            }
        }
        _ => {}
    }
}

/// Reads minigalaxy's `install_dir` from a `config.json`.
///
/// Returns `None` for missing, garbled or relative values, and for a file that
/// does not set `install_dir`. Every such case just means "no candidates".
#[must_use]
pub fn install_dir_from_minigalaxy(text: &str) -> Option<PathBuf> {
    parse_minigalaxy(text).ok().flatten()
}

/// Parses minigalaxy's `config.json`, distinguishing a parse failure from an
/// absent or relative `install_dir`.
fn parse_minigalaxy(text: &str) -> Result<Option<PathBuf>, serde_json::Error> {
    let value: serde_json::Value = serde_json::from_str(text)?;
    let Some(text) = value.get("install_dir").and_then(|value| value.as_str()) else {
        return Ok(None);
    };
    let path = Path::new(text);
    Ok(path.is_absolute().then(|| path.to_path_buf()))
}

/// Discovers every GOG candidate reachable from `home`.
///
/// Heroic records and minigalaxy's setting are read from their native and
/// Flatpak locations; the documented defaults (`~/Games/Heroic`,
/// `~/GOG Games`) are probed whether or not a record exists. Files are
/// returned sorted and de-duplicated; the same file reached through two routes
/// keeps the first (recorded) route.
#[must_use]
pub fn discover(home: &Path) -> StoreDiscovery {
    let mut discovery = StoreDiscovery::default();
    // A recorded install directory is a game root (one subfolder deep); a
    // store's default games folder holds one directory per game, so it needs
    // one more level.
    let mut roots: Vec<(PathBuf, GogLauncher, RootDepth)> = Vec::new();

    for record in heroic_installed_files(home) {
        match read_record(&record) {
            Record::Missing => {}
            Record::Unreadable => discovery.unreadable.push(record),
            Record::Text(text) => match parse_heroic_installed(&text) {
                Ok(paths) => roots.extend(
                    paths
                        .into_iter()
                        .map(|path| (path, GogLauncher::Heroic, RootDepth::Game)),
                ),
                Err(_) => discovery.unreadable.push(record),
            },
        }
    }
    // Heroic's default install directory, with or without a record.
    roots.push((
        heroic_default_dir(home),
        GogLauncher::Heroic,
        RootDepth::Doom,
    ));

    for config in minigalaxy_config_files(home) {
        match read_record(&config) {
            Record::Missing => {}
            Record::Unreadable => discovery.unreadable.push(config),
            Record::Text(text) => match parse_minigalaxy(&text) {
                Ok(Some(dir)) => roots.push((dir, GogLauncher::Minigalaxy, RootDepth::Game)),
                Ok(None) => {}
                Err(_) => discovery.unreadable.push(config),
            },
        }
    }

    // GOG offline installers' documented default, one level of games.
    roots.push((gog_games_dir(home), GogLauncher::Offline, RootDepth::Doom));

    let mut seen = std::collections::BTreeSet::new();
    for (root, launcher, depth) in roots {
        for path in wad_candidates(&root, depth) {
            if discovery.candidates.len() >= MAX_CANDIDATES {
                break;
            }
            if seen.insert(path.clone()) {
                discovery.candidates.push(StoreCandidate { path, launcher });
            }
        }
    }
    discovery.candidates.sort_by(|a, b| a.path.cmp(&b.path));
    discovery
}

/// The non-store roots of spec 021 R4, from `home` and the environment.
///
/// `env` supplies `DOOMWADDIR`/`DOOMWADPATH`; it is a closure so tests never
/// touch the process environment. Only absolute directories are returned.
#[must_use]
pub fn standard_roots(home: &Path, env: &dyn Fn(&str) -> Option<String>) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for key in ["DOOMWADDIR", "DOOMWADPATH"] {
        if let Some(value) = env(key) {
            for part in value.split(':') {
                let path = Path::new(part);
                if path.is_absolute() {
                    roots.push(path.to_path_buf());
                }
            }
        }
    }
    roots.push(PathBuf::from("/usr/share/games/doom"));
    roots.push(home.join(".local/share/games/doom"));
    roots.sort();
    roots.dedup();
    roots
}

/// Candidate WAD files under every R4 standard root, sorted and de-duplicated.
#[must_use]
pub fn standard_paths(home: &Path, env: &dyn Fn(&str) -> Option<String>) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for root in standard_roots(home, env) {
        for path in wad_candidates(&root, RootDepth::Doom) {
            if found.len() >= MAX_CANDIDATES {
                break;
            }
            found.push(path);
        }
    }
    found.sort();
    found.dedup();
    found
}

/// How deep below a root DDL looks for `*.wad`.
#[derive(Debug, Clone, Copy)]
enum RootDepth {
    /// A launcher-recorded install directory: the directory and one level of
    /// subfolders (e.g. `base/`, `rerelease/`, `doom2/`).
    Game,
    /// A store's default games folder: each game directory and one level of
    /// subfolders below it.
    Doom,
}

impl RootDepth {
    const fn levels(self) -> usize {
        match self {
            RootDepth::Game => 1,
            RootDepth::Doom => 2,
        }
    }
}

/// `*.wad` regular files under `root`, to `depth.levels()` subfolders.
///
/// Never errors: a missing root, an unreadable directory or a symlinked
/// directory all yield what was found so far. Symlinked directories are not
/// traversed (loop safety); a symlinked file is included only if it is a
/// regular file.
fn wad_candidates(root: &Path, depth: RootDepth) -> Vec<PathBuf> {
    let mut found = Vec::new();
    walk(root, depth.levels(), &mut found);
    found.sort();
    found.dedup();
    found
}

/// Walks `dir` `remaining` levels deep, collecting `*.wad` regular files.
fn walk(dir: &Path, remaining: usize, found: &mut Vec<PathBuf>) {
    if found.len() >= MAX_CANDIDATES {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten().take(MAX_ENTRIES) {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            // Never traverse a symlinked directory; include a symlinked file
            // when it really resolves to a regular file.
            let target = std::fs::metadata(&path);
            if target.is_ok_and(|metadata| metadata.is_file()) && is_wad(&path) {
                found.push(path);
                if found.len() >= MAX_CANDIDATES {
                    return;
                }
            }
        } else if file_type.is_dir() {
            if remaining > 0 {
                walk(&path, remaining - 1, found);
            }
        } else if file_type.is_file() && is_wad(&path) {
            found.push(path);
            if found.len() >= MAX_CANDIDATES {
                return;
            }
        }
    }
}

/// `true` when the path ends in `.wad`, whatever the case.
fn is_wad(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.to_string_lossy().eq_ignore_ascii_case("wad"))
}

/// What reading a launcher record produced.
enum Record {
    /// The file does not exist.
    Missing,
    /// The file exists but is too large, unreadable or not valid UTF-8.
    Unreadable,
    /// The file's text.
    Text(String),
}

/// Reads a launcher record with a size bound.
fn read_record(path: &Path) -> Record {
    let Ok(metadata) = std::fs::metadata(path) else {
        return Record::Missing;
    };
    if !metadata.is_file() || metadata.len() > MAX_RECORD_BYTES {
        return Record::Unreadable;
    }
    match std::fs::read_to_string(path) {
        Ok(text) => Record::Text(text),
        Err(_) => Record::Unreadable,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        fs::write(path, b"fake wad").expect("write fixture");
    }

    #[test]
    fn heroic_record_yields_install_paths() {
        let text = include_str!("../../tests/fixtures/gog/heroic-installed.json");
        assert_eq!(
            install_paths_from_heroic(text),
            vec![
                PathBuf::from("/home/deck/Games/Heroic/DOOM + DOOM II"),
                PathBuf::from("/run/media/deck/sdcard/Games/Heroic/Final DOOM"),
            ]
        );
    }

    #[test]
    fn heroic_malformed_record_is_tolerated() {
        let text = include_str!("../../tests/fixtures/gog/heroic-installed-malformed.json");
        assert!(install_paths_from_heroic(text).is_empty());
        // Garbage, empty and non-object documents never panic.
        for case in ["", "not json", "[]", "{}", "{\"installed\": 3}"] {
            let _ = install_paths_from_heroic(case);
        }
        // Only absolute paths are accepted.
        assert!(install_paths_from_heroic(r#"{"install_path":"relative/dir"}"#).is_empty());
    }

    #[test]
    fn minigalaxy_config_yields_the_install_dir() {
        let text = include_str!("../../tests/fixtures/gog/minigalaxy-config.json");
        assert_eq!(
            install_dir_from_minigalaxy(text),
            Some(PathBuf::from("/home/deck/GOG Games"))
        );
    }

    #[test]
    fn minigalaxy_garbled_or_relative_config_is_tolerated() {
        for case in [
            "",
            "not json",
            "{}",
            r#"{"install_dir": 7}"#,
            r#"{"install_dir": "relative/dir"}"#,
        ] {
            assert_eq!(install_dir_from_minigalaxy(case), None, "{case}");
        }
    }

    #[test]
    fn heroic_record_is_preferred_over_the_default_dir() {
        let home = TempDir::new().expect("tempdir");
        let custom = home.path().join("custom/doom");
        touch(&custom.join("base/doom.wad"));
        // The same WAD is also reachable through the default Heroic root.
        touch(&home.path().join("Games/Heroic/doom/base/doom.wad"));
        touch(&home.path().join("Games/Heroic/doom/rerelease/doom.wad"));

        let record = format!(
            "{{\"installed\":[{{\"appName\":\"1\",\"install_path\":\"{}\"}}]}}",
            custom.display()
        );
        let config = home.path().join(".config/heroic/gog_store/installed.json");
        fs::create_dir_all(config.parent().unwrap()).expect("mkdir");
        fs::write(&config, record).expect("record");

        let discovery = discover(home.path());
        assert!(discovery.unreadable.is_empty());
        let mut paths: Vec<_> = discovery
            .candidates
            .iter()
            .map(|c| c.path.clone())
            .collect();
        paths.sort();
        let mut expected = vec![
            custom.join("base/doom.wad"),
            home.path().join("Games/Heroic/doom/base/doom.wad"),
            home.path().join("Games/Heroic/doom/rerelease/doom.wad"),
        ];
        expected.sort();
        assert_eq!(paths, expected);
        // The recorded route attributes the custom file to Heroic.
        assert!(
            discovery.candidates.iter().any(
                |c| c.path == custom.join("base/doom.wad") && c.launcher == GogLauncher::Heroic
            )
        );
    }

    #[test]
    fn missing_and_garbled_records_still_probe_defaults() {
        let home = TempDir::new().expect("tempdir");
        // An unreadable record is reported, not fatal.
        let record = home.path().join(".config/heroic/gog_store/installed.json");
        fs::create_dir_all(record.parent().unwrap()).expect("mkdir");
        fs::write(&record, "{ truncated").expect("record");
        // A relative minigalaxy install_dir is ignored.
        let config = home.path().join(".config/minigalaxy/config.json");
        fs::create_dir_all(config.parent().unwrap()).expect("mkdir");
        fs::write(&config, r#"{"install_dir":"relative"}"#).expect("config");
        // The documented GOG offline default has one game with a WAD.
        touch(&home.path().join("GOG Games/Final DOOM/tnt.wad"));

        let discovery = discover(home.path());
        assert_eq!(discovery.unreadable, vec![record]);
        assert_eq!(
            discovery.candidates,
            vec![StoreCandidate {
                path: home.path().join("GOG Games/Final DOOM/tnt.wad"),
                launcher: GogLauncher::Offline,
            }]
        );
    }

    #[test]
    fn flatpak_paths_are_read() {
        let home = TempDir::new().expect("tempdir");
        let install = home.path().join("var/heroic/doom");
        touch(&install.join("doom.wad"));
        let record = home
            .path()
            .join(".var/app/com.heroicgameslauncher.hgl/config/heroic/gog_store/installed.json");
        fs::create_dir_all(record.parent().unwrap()).expect("mkdir");
        fs::write(
            &record,
            format!(
                "{{\"installed\":[{{\"install_path\":\"{}\"}}]}}",
                install.display()
            ),
        )
        .expect("record");

        // minigalaxy's Flatpak config points at an offline-style folder.
        let mini = home.path().join("var/mini/GOG");
        touch(&mini.join("doom2.wad"));
        let config = home
            .path()
            .join(".var/app/io.github.sharkwouter.Minigalaxy/config/minigalaxy/config.json");
        fs::create_dir_all(config.parent().unwrap()).expect("mkdir");
        fs::write(
            &config,
            format!("{{\"install_dir\":\"{}\"}}", mini.display()),
        )
        .expect("config");

        let discovery = discover(home.path());
        let paths: Vec<_> = discovery
            .candidates
            .iter()
            .map(|c| c.path.clone())
            .collect();
        assert!(paths.contains(&install.join("doom.wad")));
        assert!(paths.contains(&mini.join("doom2.wad")));
        assert!(
            discovery
                .candidates
                .iter()
                .any(|c| c.launcher == GogLauncher::Minigalaxy)
        );
    }

    #[test]
    fn discovery_stops_one_subfolder_deep_and_never_traverses_symlinks() {
        let home = TempDir::new().expect("tempdir");
        let install = home.path().join("Games/Heroic/Doom");
        touch(&install.join("base/doom.wad"));
        touch(&install.join("base/deep/doom.wad"));
        touch(&install.join("loose.wad"));
        touch(&install.join("readme.txt"));

        let discovery = discover(home.path());
        let paths: Vec<_> = discovery
            .candidates
            .iter()
            .map(|c| c.path.clone())
            .collect();
        assert!(paths.contains(&install.join("loose.wad")));
        assert!(paths.contains(&install.join("base/doom.wad")));
        assert!(
            !paths.contains(&install.join("base/deep/doom.wad")),
            "no deeper than one subfolder below a game root"
        );
        assert!(!paths.iter().any(|p| p.extension().unwrap() == "txt"));
    }

    #[test]
    fn standard_roots_include_env_and_fixed_paths() {
        let home = TempDir::new().expect("tempdir");
        let env = |key: &str| match key {
            "DOOMWADDIR" => Some("/opt/doom".to_owned()),
            "DOOMWADPATH" => Some("/a:/b".to_owned()),
            _ => None,
        };
        let roots = standard_roots(home.path(), &env);
        for expected in [
            PathBuf::from("/a"),
            PathBuf::from("/b"),
            PathBuf::from("/opt/doom"),
            PathBuf::from("/usr/share/games/doom"),
            home.path().join(".local/share/games/doom"),
        ] {
            assert!(
                roots.contains(&expected),
                "missing {expected:?} in {roots:?}"
            );
        }
        assert_eq!(roots.len(), 5, "no duplicate or extra roots: {roots:?}");
    }

    #[test]
    fn standard_paths_scans_the_opted_in_roots() {
        let home = TempDir::new().expect("tempdir");
        let local = home.path().join(".local/share/games/doom");
        touch(&local.join("doom.wad"));
        let env = |key: &str| (key == "DOOMWADDIR").then(|| local.display().to_string());
        let paths = standard_paths(home.path(), &env);
        assert_eq!(paths, vec![local.join("doom.wad")]);
    }

    #[test]
    fn missing_home_yields_nothing() {
        let home = TempDir::new().expect("tempdir");
        let discovery = discover(home.path());
        assert!(discovery.candidates.is_empty());
        assert!(discovery.unreadable.is_empty());
        assert!(standard_paths(home.path(), &|_| None).is_empty());
    }
}

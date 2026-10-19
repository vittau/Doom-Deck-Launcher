//! Read-only Steam library discovery (spec 007, R3; spec 028 adds the real
//! `Ultimate Doom/` layout).
//!
//! DDL finds the Steam roots a Steam Deck user can have, parses the Valve
//! KeyValues `libraryfolders.vdf` that lists extra libraries, and probes only a
//! short allow-list of known classic-Doom install directories under each
//! library's `steamapps/common/` (research §4). It never walks `$HOME`, never
//! writes to Steam, and treats every missing path as normal.
//!
//! The KeyValues reader is a small, tolerant, read-only parser: malformed input
//! yields no libraries rather than an error, and nesting is depth-limited so a
//! hostile file cannot exhaust the stack.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

/// Install directories under `steamapps/common/` that may hold IWADs, from
/// research §4. Matched case-insensitively; the Deck layout (2026-10-19) is
/// confirmed for `Ultimate Doom/`, the legacy folders are not.
///
/// Every entry is scanned flat (one directory, no recursion), except that the
/// Deck's `base/` subfolders are named explicitly. This keeps the scan bounded
/// while covering `base/doom2/`, `base/tnt/` and `base/plutonia/`. It also
/// keeps `base/master/wads/` out of scope: those are the 20 Master Levels
/// PWADs, which are never base games.
pub const KNOWN_INSTALL_DIRS: &[&str] = &[
    "DOOM + DOOM II/base",
    "DOOM + DOOM II/rerelease",
    "Ultimate Doom/base",
    "Ultimate Doom/base/doom2",
    "Ultimate Doom/base/tnt",
    "Ultimate Doom/base/plutonia",
    "Ultimate Doom/rerelease",
    "Doom 2/base",
    "Doom 2/finaldoombase",
    "Final Doom/base",
    "Master Levels of Doom/doom2",
];

/// KEX files under a `rerelease/` folder that are resources or PWAD
/// expansions, never base-game IWADs (spec 028, R2). They are filtered by name
/// before identification, so even a hash-identical `nerve.wad` is not offered
/// as an IWAD.
pub const KEX_RESOURCE_WADS: &[&str] = &[
    "extras.wad",
    "id1.wad",
    "id1-mus.wad",
    "id1-res.wad",
    "id1-tex.wad",
    "id1-weap.wad",
    "id24res.wad",
    "iddm1.wad",
    "masterlevels.wad",
    "nerve.wad",
    "sigil.wad",
    "sigil2.wad",
];

/// Maximum KeyValues nesting depth; beyond this the document is rejected.
const MAX_DEPTH: u32 = 64;

/// A Steam library root: the directory that contains `steamapps/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamLibrary {
    /// Absolute path to the library root.
    pub path: PathBuf,
}

impl SteamLibrary {
    /// The library's `steamapps/` directory.
    #[must_use]
    pub fn steamapps(&self) -> PathBuf {
        self.path.join("steamapps")
    }

    /// The library's `steamapps/common/` directory.
    #[must_use]
    pub fn common(&self) -> PathBuf {
        self.steamapps().join("common")
    }
}

/// The Steam roots for `home`, in probe order.
///
/// Covers the default install, the `~/.steam/root` symlink and Flatpak Steam
/// (research §4). Paths are returned whether or not they exist; callers filter.
#[must_use]
pub fn steam_roots(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".local/share/Steam"),
        home.join(".steam/root"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ]
}

/// Discovers every existing Steam library reachable from `home`.
///
/// Each Steam root is itself a library when it has a `steamapps/` directory,
/// and `libraryfolders.vdf` adds the rest (internal drive, SD card, …).
/// Duplicates (e.g. `~/.steam/root` → `~/.local/share/Steam`) are collapsed.
#[must_use]
pub fn libraries(home: &Path) -> Vec<SteamLibrary> {
    let mut libraries = Vec::new();
    let mut seen = BTreeSet::new();

    for root in steam_roots(home) {
        add_library(&mut libraries, &mut seen, root.clone());

        let vdf = root.join("steamapps").join("libraryfolders.vdf");
        if let Ok(text) = std::fs::read_to_string(&vdf) {
            for path in library_paths_from_vdf(&text) {
                add_library(&mut libraries, &mut seen, path);
            }
        }
    }

    libraries
}

/// Adds `path` as a library when its `steamapps/` directory exists.
fn add_library(libraries: &mut Vec<SteamLibrary>, seen: &mut BTreeSet<PathBuf>, path: PathBuf) {
    if !path.join("steamapps").is_dir() {
        return;
    }
    let key = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
    if seen.insert(key) {
        libraries.push(SteamLibrary { path });
    }
}

/// Every candidate IWAD file under a library's known install directories.
///
/// Directories are scanned flat: the known entries name the nested Deck
/// folders explicitly, so no recursive walk is needed. Files in a
/// `rerelease/` folder that are known KEX resources ([`KEX_RESOURCE_WADS`])
/// are skipped. Files are returned unsorted-by-identity; callers identify
/// them. A library with none of the known install directories yields an empty
/// list.
#[must_use]
pub fn candidate_iwads(library: &SteamLibrary) -> Vec<PathBuf> {
    let common = library.common();
    let mut found = Vec::new();
    for install in KNOWN_INSTALL_DIRS {
        if let Some(dir) = resolve_dir_case_insensitive(&common, install) {
            let rerelease = install
                .rsplit('/')
                .next()
                .is_some_and(|last| last.eq_ignore_ascii_case("rerelease"));
            found.extend(
                wad_files(&dir)
                    .into_iter()
                    .filter(|path| !(rerelease && is_kex_resource(path))),
            );
        }
    }
    found.sort();
    found.dedup();
    found
}

/// Existing known install folders read by Steam discovery (spec 032).
#[must_use]
pub fn source_folders(home: &Path) -> Vec<PathBuf> {
    let mut found: Vec<_> = libraries(home)
        .iter()
        .flat_map(|library| {
            KNOWN_INSTALL_DIRS
                .iter()
                .filter_map(|install| resolve_dir_case_insensitive(&library.common(), install))
                .collect::<Vec<_>>()
        })
        .collect();
    found.sort();
    found.dedup();
    found
}

/// `true` when the file name is one of the KEX resource WADs.
fn is_kex_resource(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    KEX_RESOURCE_WADS
        .iter()
        .any(|resource| name.eq_ignore_ascii_case(resource))
}

/// Every candidate IWAD file reachable from `home` across all libraries.
#[must_use]
pub fn discover_paths(home: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for library in libraries(home) {
        found.extend(candidate_iwads(&library));
    }
    found.sort();
    found.dedup();
    found
}

/// Extracts library roots from a `libraryfolders.vdf` document.
///
/// Tolerant by design: any parse error, including a truncated or deeply nested
/// file, yields an empty list. Both the current schema (nested objects with a
/// `path` key) and the legacy flat schema (`"1" "/path"`) are supported, by
/// collecting every absolute-path string value in the tree.
#[must_use]
pub fn library_paths_from_vdf(text: &str) -> Vec<PathBuf> {
    let Ok(value) = parse_vdf(text) else {
        return Vec::new();
    };
    let mut paths = Vec::new();
    collect_paths(&value, &mut paths);
    paths.dedup();
    paths
}

/// Recursively collects absolute-path string values.
fn collect_paths(value: &Value, out: &mut Vec<PathBuf>) {
    match value {
        Value::Str(text) => {
            let path = Path::new(text);
            if path.is_absolute() {
                out.push(path.to_path_buf());
            }
        }
        Value::Object(pairs) => {
            for (_, child) in pairs {
                collect_paths(child, out);
            }
        }
    }
}

/// Resolves `relative` (slash-separated) under `base`, matching each path
/// component case-insensitively against the directory's real entries.
fn resolve_dir_case_insensitive(base: &Path, relative: &str) -> Option<PathBuf> {
    let mut current = base.to_path_buf();
    for component in relative.split('/').filter(|part| !part.is_empty()) {
        let mut matched = None;
        for entry in std::fs::read_dir(&current).ok()?.flatten() {
            if entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(component)
            {
                matched = Some(entry.path());
                break;
            }
        }
        current = matched?;
    }
    current.is_dir().then_some(current)
}

/// `*.wad` regular files directly inside `dir`, case-insensitively.
fn wad_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_wad = path
            .extension()
            .is_some_and(|extension| extension.to_string_lossy().eq_ignore_ascii_case("wad"));
        // Follow a symlink only to check it is a file; Steam trees are trusted.
        let is_file = std::fs::metadata(&path).is_ok_and(|metadata| metadata.is_file());
        if is_wad && is_file {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// A parsed KeyValues value.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Str(String),
    Object(Vec<(String, Value)>),
}

/// A KeyValues parse failure.
#[derive(Debug, Clone, PartialEq, Eq)]
struct VdfError(String);

impl fmt::Display for VdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Parses a KeyValues document. Never panics; deep nesting is an error.
fn parse_vdf(text: &str) -> Result<Value, VdfError> {
    let bytes = text.strip_prefix('\u{feff}').unwrap_or(text).as_bytes();
    let mut parser = Parser {
        bytes,
        pos: 0,
        depth: 0,
    };
    parser.parse_document()
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
    depth: u32,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(byte) if byte.is_ascii_whitespace() => self.pos += 1,
                Some(b'/') if self.bytes.get(self.pos + 1) == Some(&b'/') => {
                    while let Some(byte) = self.peek() {
                        self.pos += 1;
                        if byte == b'\n' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn parse_document(&mut self) -> Result<Value, VdfError> {
        self.skip_trivia();
        if self.peek() == Some(b'{') {
            return self.parse_object();
        }
        if self.peek().is_none() {
            return Ok(Value::Object(Vec::new()));
        }
        let key = self.parse_string()?;
        self.skip_trivia();
        let value = if self.peek() == Some(b'{') {
            self.parse_object()?
        } else {
            Value::Str(self.parse_string()?)
        };
        Ok(Value::Object(vec![(key, value)]))
    }

    fn parse_object(&mut self) -> Result<Value, VdfError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(VdfError("KeyValues nesting is too deep".to_owned()));
        }
        if self.peek() != Some(b'{') {
            return Err(VdfError("expected `{`".to_owned()));
        }
        self.pos += 1;

        let mut pairs = Vec::new();
        loop {
            self.skip_trivia();
            match self.peek() {
                None => return Err(VdfError("unterminated object".to_owned())),
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                Some(_) => {
                    let key = self.parse_string()?;
                    self.skip_trivia();
                    let value = if self.peek() == Some(b'{') {
                        self.parse_object()?
                    } else {
                        Value::Str(self.parse_string()?)
                    };
                    pairs.push((key, value));
                }
            }
        }
        Ok(Value::Object(pairs))
    }

    fn parse_string(&mut self) -> Result<String, VdfError> {
        self.skip_trivia();
        match self.peek() {
            Some(b'"') => {
                self.pos += 1;
                let mut buffer = Vec::new();
                loop {
                    match self.peek() {
                        None => return Err(VdfError("unterminated string".to_owned())),
                        Some(b'"') => {
                            self.pos += 1;
                            break;
                        }
                        Some(b'\\') => {
                            self.pos += 1;
                            match self.peek() {
                                Some(b'n') => buffer.push(b'\n'),
                                Some(b't') => buffer.push(b'\t'),
                                Some(byte) => buffer.push(byte),
                                None => return Err(VdfError("unterminated escape".to_owned())),
                            }
                            self.pos += 1;
                        }
                        Some(byte) => {
                            buffer.push(byte);
                            self.pos += 1;
                        }
                    }
                }
                Ok(String::from_utf8_lossy(&buffer).into_owned())
            }
            Some(_) => {
                let mut buffer = Vec::new();
                while let Some(byte) = self.peek() {
                    if byte.is_ascii_whitespace() || matches!(byte, b'{' | b'}' | b'"') {
                        break;
                    }
                    buffer.push(byte);
                    self.pos += 1;
                }
                if buffer.is_empty() {
                    Err(VdfError("expected a token".to_owned()))
                } else {
                    Ok(String::from_utf8_lossy(&buffer).into_owned())
                }
            }
            None => Err(VdfError("expected a token, found end of input".to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::iwads::identify::{Family, Identifier, Outcome};

    const NEW_SCHEMA: &str = include_str!("../../tests/fixtures/steam/libraryfolders.vdf");
    const LEGACY_SCHEMA: &str =
        include_str!("../../tests/fixtures/steam/libraryfolders-legacy.vdf");
    const MALFORMED: &str = include_str!("../../tests/fixtures/steam/libraryfolders-malformed.vdf");

    #[test]
    fn new_schema_yields_both_libraries() {
        let paths = library_paths_from_vdf(NEW_SCHEMA);
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/home/deck/.local/share/Steam"),
                PathBuf::from("/run/media/deck/extsd/SteamLibrary"),
            ]
        );
    }

    #[test]
    fn legacy_schema_yields_libraries_and_ignores_timestamps() {
        let paths = library_paths_from_vdf(LEGACY_SCHEMA);
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/home/deck/.steam/steam"),
                PathBuf::from("/run/media/deck/extsd/SteamLibrary"),
            ],
            "non-path values such as TimeNextStatsReport must be ignored"
        );
    }

    #[test]
    fn malformed_documents_yield_no_libraries() {
        assert!(library_paths_from_vdf(MALFORMED).is_empty());
    }

    #[test]
    fn malformed_inputs_never_panic() {
        let cases = [
            "",
            "   ",
            "{",
            "}",
            "{}",
            "\"key\"",
            "\"key\" {",
            "\"key\" { \"a\" }",
            "\"key\" { \"a\" \"b\"",
            "\"unterminated",
            "\"a\" { \"b\" { \"c\" { \"d\" } } }",
            "// just a comment\n",
            "\"key\" \"C:\\\\Games\\\\Steam\"",
            "\u{feff}\"libraryfolders\" { \"0\" { \"path\" \"/x\" } }",
            "\"0\" { \"path\" 12345 }",
            "\"k\" {\r\n\t\"path\"\t\"/a b/c\"\r\n}",
        ];
        for case in cases {
            let _ = library_paths_from_vdf(case);
        }
    }

    #[test]
    fn deeply_nested_input_is_rejected_without_overflow() {
        let mut document = String::new();
        for _ in 0..(MAX_DEPTH + 50) {
            document.push('{');
        }
        assert!(
            library_paths_from_vdf(&document).is_empty(),
            "depth limit must reject rather than recurse forever"
        );
    }

    #[test]
    fn escaped_backslashes_and_utf8_paths_survive() {
        let text = "\"libraryfolders\" { \"0\" { \"path\" \"/home/Dööm/Steam\" } }";
        assert_eq!(
            library_paths_from_vdf(text),
            vec![PathBuf::from("/home/Dööm/Steam")]
        );
    }

    #[test]
    fn missing_roots_are_normal() {
        let home = TempDir::new().expect("tempdir");
        assert!(libraries(home.path()).is_empty());
        assert!(discover_paths(home.path()).is_empty());
    }

    #[test]
    fn discovers_two_libraries_with_odd_casing_and_missing_folders() {
        let home = TempDir::new().expect("tempdir");
        let primary = home.path().join(".local/share/Steam");
        let sdcard = home.path().join("extsd/SteamLibrary");

        // Primary library: odd casing for both the install dir and the file.
        let base = primary.join("steamapps/common/doom + doom ii/BASE");
        fs::create_dir_all(&base).expect("mkdir");
        fs::write(base.join("DOOM.WAD"), b"fake").expect("wad");
        // An unrelated file that must not be listed.
        fs::write(base.join("readme.txt"), b"x").expect("txt");

        // SD-card library: only one install dir present, the others missing.
        let doom2 = sdcard.join("steamapps/common/Doom 2/base");
        fs::create_dir_all(&doom2).expect("mkdir");
        fs::write(doom2.join("doom2.wad"), b"fake").expect("wad");
        // A nested folder must NOT be scanned by Steam discovery.
        fs::create_dir_all(doom2.join("nested")).expect("mkdir");
        fs::write(doom2.join("nested/deep.wad"), b"fake").expect("wad");

        let vdf = format!(
            "\"libraryfolders\"\n{{\n\t\"0\" {{ \"path\" \"{}\" }}\n\t\"1\" {{ \"path\" \"{}\" }}\n}}\n",
            primary.display(),
            sdcard.display()
        );
        fs::create_dir_all(primary.join("steamapps")).expect("mkdir");
        fs::write(primary.join("steamapps/libraryfolders.vdf"), vdf).expect("vdf");

        let libraries = libraries(home.path());
        assert_eq!(
            libraries.len(),
            2,
            "both libraries discovered: {libraries:?}"
        );

        let mut found = discover_paths(home.path());
        found.sort();
        assert_eq!(
            found,
            vec![base.join("DOOM.WAD"), doom2.join("doom2.wad"),],
            "case-insensitive matching, missing folders normal, no recursion"
        );
    }

    #[test]
    fn candidate_iwads_ignores_non_wad_and_nested_files() {
        let home = TempDir::new().expect("tempdir");
        let base = home
            .path()
            .join("Steam/steamapps/common/DOOM + DOOM II/base");
        fs::create_dir_all(&base).expect("mkdir");
        fs::write(base.join("doom.wad"), b"x").expect("wad");
        fs::write(base.join("doom.WAD.bak"), b"x").expect("bak");
        fs::create_dir_all(base.join("sub")).expect("mkdir");
        fs::write(base.join("sub/other.wad"), b"x").expect("wad");

        let library = SteamLibrary {
            path: home.path().join("Steam"),
        };
        let found = candidate_iwads(&library);
        assert_eq!(found, vec![base.join("doom.wad")]);
    }

    /// A synthetic WAD: a valid header plus a lump directory with `lumps`.
    /// Fixtures carry names and headers only, never real IWAD content.
    fn wad_bytes(magic: &[u8; 4], lumps: &[String]) -> Vec<u8> {
        let mut directory = Vec::new();
        for name in lumps {
            directory.extend_from_slice(&0_i32.to_le_bytes()); // filepos
            directory.extend_from_slice(&0_i32.to_le_bytes()); // size
            let mut padded = [0_u8; 8];
            let bytes = name.as_bytes();
            let len = bytes.len().min(8);
            padded[..len].copy_from_slice(&bytes[..len]);
            directory.extend_from_slice(&padded);
        }

        let mut out = Vec::new();
        out.extend_from_slice(magic);
        out.extend_from_slice(&(lumps.len() as i32).to_le_bytes());
        out.extend_from_slice(&12_i32.to_le_bytes());
        out.extend_from_slice(&directory);
        out
    }

    fn ultimate_lumps() -> Vec<String> {
        let mut lumps = Vec::new();
        for episode in 1..=4 {
            for map in 1..=9 {
                lumps.push(format!("E{episode}M{map}"));
            }
        }
        lumps
    }

    fn commercial_lumps() -> Vec<String> {
        (1..=32).map(|map| format!("MAP{map:02}")).collect()
    }

    fn pwad_lumps() -> Vec<String> {
        vec!["MAP01".to_owned()]
    }

    /// The IWADs the Deck layout must yield, keyed by path, plus the files that
    /// must never be offered.
    struct DeckLayout {
        expected: BTreeMap<PathBuf, Family>,
        master_level: PathBuf,
        resources: Vec<PathBuf>,
    }

    /// Writes the confirmed Deck tree under `install` (`.../Ultimate Doom`).
    ///
    /// `upper` uppercases every component, exercising case-insensitive
    /// matching on the second library.
    fn write_deck_layout(install: &Path, upper: bool) -> DeckLayout {
        let base = install.join(if upper { "BASE" } else { "base" });
        let rerelease = install.join(if upper { "RERELEASE" } else { "rerelease" });
        let name = |file: &str| {
            if upper {
                file.to_ascii_uppercase()
            } else {
                file.to_owned()
            }
        };
        let mut expected = BTreeMap::new();
        let mut put = |dir: &Path, file: &str, magic: &[u8; 4], lumps: &[String], family| {
            fs::create_dir_all(dir).expect("mkdir");
            let path = dir.join(name(file));
            fs::write(&path, wad_bytes(magic, lumps)).expect("write wad");
            if let Some(family) = family {
                expected.insert(path, family);
            }
        };

        // Classic copies, upper case as on the Deck.
        put(
            &base,
            "DOOM.WAD",
            b"IWAD",
            &ultimate_lumps(),
            Some(Family::Ultimate),
        );
        put(
            &base.join(name("doom2")),
            "DOOM2.WAD",
            b"IWAD",
            &commercial_lumps(),
            Some(Family::Commercial),
        );
        put(
            &base.join(name("tnt")),
            "TNT.WAD",
            b"IWAD",
            &commercial_lumps(),
            Some(Family::Commercial),
        );
        put(
            &base.join(name("plutonia")),
            "PLUTONIA.WAD",
            b"IWAD",
            &commercial_lumps(),
            Some(Family::Commercial),
        );

        // The 20 Master Levels PWADs live three levels down and are never
        // scanned (nor identified as IWADs when they are).
        let master_dir = base.join(name("master")).join(name("wads"));
        fs::create_dir_all(&master_dir).expect("mkdir");
        let master_level = master_dir.join(name("ATTACK.WAD"));
        fs::write(&master_level, wad_bytes(b"PWAD", &pwad_lumps())).expect("write pwad");

        // KEX copies and resource WADs, lower case as on the Deck.
        for file in ["doom.wad", "doom2.wad", "tnt.wad", "plutonia.wad"] {
            let family = if file == "doom.wad" {
                Family::Ultimate
            } else {
                Family::Commercial
            };
            let lumps = if family == Family::Ultimate {
                ultimate_lumps()
            } else {
                commercial_lumps()
            };
            put(&rerelease, file, b"IWAD", &lumps, Some(family));
        }

        let mut resources = Vec::new();
        for file in KEX_RESOURCE_WADS {
            fs::create_dir_all(&rerelease).expect("mkdir");
            let path = rerelease.join(name(file));
            // Some resource WADs are PWADs; give one IWAD magic so the name
            // filter, not identification, is what keeps it out.
            let magic = if *file == "extras.wad" {
                b"IWAD"
            } else {
                b"PWAD"
            };
            fs::write(&path, wad_bytes(magic, &pwad_lumps())).expect("write resource");
            resources.push(path);
        }

        DeckLayout {
            expected,
            master_level,
            resources,
        }
    }

    /// Writes a two-library Steam home: the internal drive and an SD card.
    fn two_library_home(primary: &Path, sdcard: &Path) {
        fs::create_dir_all(primary.join("steamapps")).expect("mkdir");
        let vdf = format!(
            "\"libraryfolders\"\n{{\n\t\"0\" {{ \"path\" \"{}\" }}\n\t\"1\" {{ \"path\" \"{}\" }}\n}}\n",
            primary.display(),
            sdcard.display()
        );
        fs::write(primary.join("steamapps/libraryfolders.vdf"), vdf).expect("vdf");
    }

    #[test]
    fn base_games_from_deck_layout_prefer_classic_and_support_kex_only() {
        use crate::iwads::{IwadTable, Registry, Source};
        let home = TempDir::new().unwrap();
        let primary = home.path().join(".local/share/Steam");
        write_deck_layout(&primary.join("steamapps/common/Ultimate Doom"), false);
        let mut registry = Registry::new();
        for path in discover_paths(home.path()) {
            let identification = Identifier::embedded().identify(&path).unwrap();
            registry.record(&path, &identification, Source::Steam);
            let file = path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_ascii_lowercase();
            let variant = match file.as_ref() {
                "doom.wad" => "doom-1.9ud",
                "doom2.wad" => "doom2-1.9",
                "tnt.wad" => "tnt-1.9",
                "plutonia.wad" => "plutonia-1.9",
                _ => continue,
            };
            registry
                .choose(&path, variant, &IwadTable::embedded())
                .unwrap();
        }
        let empty = primary.join("steamapps/common/DOOM 2/base");
        fs::create_dir_all(&empty).unwrap();
        assert!(source_folders(home.path()).contains(&empty));
        let games = crate::iwads::base_games::list(&registry);
        assert_eq!(games.len(), 4);
        assert_eq!(
            games.iter().map(|g| g.complevel()).collect::<Vec<_>>(),
            [3, 2, 4, 4]
        );
        assert!(
            games
                .iter()
                .all(|g| g.iwad.path.components().any(|c| c.as_os_str() == "base"))
        );
        registry
            .entries
            .retain(|e| e.path.ends_with("rerelease/doom2.wad"));
        let games = crate::iwads::base_games::list(&registry);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id(), "base:doom2");
        assert_eq!(games[0].complevel(), 2);
    }

    #[test]
    fn deck_layout_finds_the_four_games_in_base_and_rerelease_in_every_library() {
        let home = TempDir::new().expect("tempdir");
        let primary = home.path().join(".local/share/Steam");
        let sdcard = home.path().join("extsd/SteamLibrary");
        let common = |library: &Path| library.join("steamapps/common");

        let classic = write_deck_layout(&common(&primary).join("Ultimate Doom"), false);
        let odd = write_deck_layout(&common(&sdcard).join("ULTIMATE DOOM"), true);
        two_library_home(&primary, &sdcard);

        let libraries = libraries(home.path());
        assert_eq!(
            libraries.len(),
            2,
            "both libraries discovered: {libraries:?}"
        );

        let expected_sets = [&classic.expected, &odd.expected];
        for (library, expected) in libraries.iter().zip(expected_sets) {
            let candidates = candidate_iwads(library);
            let expected_paths: Vec<PathBuf> = expected.keys().cloned().collect();
            assert_eq!(
                candidates, expected_paths,
                "exactly the classic and KEX copies, in path order"
            );

            let families: Vec<Family> = candidates
                .iter()
                .map(|path| {
                    let identification = Identifier::embedded()
                        .identify(path)
                        .expect("identify fixture");
                    match identification.outcome {
                        Outcome::Ambiguous { family, .. } => family,
                        other => panic!("{} must classify, got {other:?}", path.display()),
                    }
                })
                .collect();
            let expected_families: Vec<Family> = expected.values().copied().collect();
            assert_eq!(
                families, expected_families,
                "identities must match the classic/KEX games"
            );
        }

        let mut found = discover_paths(home.path());
        found.sort();
        let mut expected_all: Vec<PathBuf> = classic
            .expected
            .keys()
            .chain(odd.expected.keys())
            .cloned()
            .collect();
        expected_all.sort();
        assert_eq!(
            found, expected_all,
            "Steam discovery covers both libraries and never scans $HOME"
        );

        // R2: the identifier rejects the Master Levels PWAD and every KEX
        // resource. `extras.wad` carries IWAD magic in this fixture, so its
        // exclusion proves the name filter too.
        for path in classic
            .resources
            .iter()
            .chain(odd.resources.iter())
            .chain(std::iter::once(&classic.master_level))
        {
            let identification = Identifier::embedded()
                .identify(path)
                .expect("identify non-IWAD");
            assert_eq!(
                identification.outcome,
                Outcome::Unknown,
                "{} must not be offered as an IWAD",
                path.display()
            );
        }
    }

    #[test]
    fn rerelease_resources_are_filtered_by_name_before_identification() {
        let home = TempDir::new().expect("tempdir");
        let rerelease = home
            .path()
            .join("Steam/steamapps/common/Ultimate Doom/rerelease");
        fs::create_dir_all(&rerelease).expect("mkdir");
        // IWAD magic plus base-game-looking lumps: only the name filter can
        // keep these out.
        for file in KEX_RESOURCE_WADS {
            fs::write(
                rerelease.join(file.to_ascii_uppercase()),
                wad_bytes(b"IWAD", &commercial_lumps()),
            )
            .expect("write");
        }
        fs::write(
            rerelease.join("doom2.wad"),
            wad_bytes(b"IWAD", &commercial_lumps()),
        )
        .expect("write");

        let library = SteamLibrary {
            path: home.path().join("Steam"),
        };
        assert_eq!(
            candidate_iwads(&library),
            vec![rerelease.join("doom2.wad")],
            "resource WADs are not candidates even when they look like IWADs"
        );
    }
}

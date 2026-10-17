//! Catalog bundles: the transport for catalog updates without a DDL release
//! (spec 019, R1/R3/R4).
//!
//! A **bundle** is one JSON document carrying every recipe plus `tags.toml` and
//! `hosts.toml` as their original TOML text, a schema version, the serial of the
//! source commit it was built from, the commit itself and the oldest DDL that
//! may consume it. `ddl catalog bundle` builds one from a local `catalog/`
//! directory; CI publishes it to the `catalog` branch (R2); the app fetches it
//! (`remote.rs`) and validates it here.
//!
//! Validation is deliberately split so a **malformed bundle is rejected whole**
//! while a **bad recipe is dropped alone** (R3):
//!
//! - whole-document checks ([`parse_bundle`], [`accept_bundle`]): schema,
//!   `min_ddl_version`, serial newer than the embedded catalog;
//! - per-recipe checks: the same [`Catalog::from_sources`] validation CI runs,
//!   with the host allowlist taken from the **embedded** `hosts.toml`, so a
//!   remote recipe naming a host the binary does not allow is dropped with a
//!   diagnostic rather than accepted.
//!
//! The accepted bundle is written atomically under the XDG data directory and
//! the app loads the newer of embedded vs cached on start ([`load_cache`]).

use std::collections::BTreeMap;
use std::fmt;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use super::lint::{Diagnostic, MAX_RECIPE_BYTES, lint_paths};
use super::{Catalog, SkippedRecipe};

/// The bundle format this DDL understands (spec 019, R1).
pub const BUNDLE_SCHEMA: u32 = 1;

/// Largest bundle DDL accepts, matching spec 019 R3's ≤ 2 MiB bound.
pub const MAX_BUNDLE_BYTES: u64 = 2 * 1024 * 1024;

/// The automatic-check interval (spec 019, R5).
pub const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;

/// File names that are not recipes inside a bundle.
const NON_RECIPE_FILES: [&str; 2] = ["hosts.toml", "tags.toml"];

/// One catalog bundle (spec 019, R1). The TOML is carried verbatim so the
/// receiver validates exactly the bytes CI validated, not a re-serialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    /// Bundle format version; only [`BUNDLE_SCHEMA`] is accepted.
    pub bundle_schema: u32,
    /// UTC Unix seconds of the source commit the catalog was built from.
    pub catalog_serial: u64,
    /// The source commit, for diagnostics and maintainer notes.
    pub source_commit: String,
    /// The oldest DDL version allowed to consume this bundle.
    pub min_ddl_version: String,
    /// File name → TOML text: every recipe plus `hosts.toml` and `tags.toml`.
    pub files: BTreeMap<String, String>,
}

/// Why a whole bundle was refused (spec 019, R3). These are never per-recipe:
/// a bundle that trips one of these is not stored at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleRejection {
    /// The document is larger than [`MAX_BUNDLE_BYTES`].
    Oversize { limit: u64, got: u64 },
    /// The document is not the expected JSON shape.
    Malformed { detail: String },
    /// `bundle_schema` is not [`BUNDLE_SCHEMA`].
    UnknownSchema { got: u32 },
    /// `min_ddl_version` is newer than the running DDL.
    TooNew { min: String, running: String },
    /// `catalog_serial` is not newer than the embedded catalog's serial.
    NotNewer { serial: u64, embedded: u64 },
}

impl fmt::Display for BundleRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Oversize { limit, got } => {
                write!(f, "catalog bundle is larger than {limit} bytes (got {got})")
            }
            Self::Malformed { detail } => write!(f, "catalog bundle is malformed: {detail}"),
            Self::UnknownSchema { got } => write!(
                f,
                "catalog bundle schema {got} is not supported (this DDL understands {BUNDLE_SCHEMA})"
            ),
            Self::TooNew { min, running } => write!(
                f,
                "catalog bundle needs DDL {min} or newer; this is {running}"
            ),
            Self::NotNewer { serial, embedded } => write!(
                f,
                "catalog bundle serial {serial} is not newer than the embedded catalog's {embedded}"
            ),
        }
    }
}

impl std::error::Error for BundleRejection {}

/// A bundle that passed the whole-document checks and whose recipes were
/// validated against the embedded host allowlist (spec 019, R3). Recipes that
/// failed are in `catalog.skipped`; the rest are usable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedBundle {
    /// The accepted bundle to persist: only the accepted recipes plus the
    /// original `tags.toml`/`hosts.toml` entries.
    pub bundle: Bundle,
    /// The catalog built from the accepted recipes. `catalog.hosts` is the
    /// embedded allowlist, never the bundle's `hosts.toml`.
    pub catalog: Catalog,
}

impl AcceptedBundle {
    /// The serial of the accepted bundle.
    #[must_use]
    pub fn serial(&self) -> u64 {
        self.bundle.catalog_serial
    }

    /// Recipes dropped during validation, with their reasons.
    #[must_use]
    pub fn dropped(&self) -> &[SkippedRecipe] {
        &self.catalog.skipped
    }
}

/// The embedded catalog's serial, or `0` when the build did not learn it
/// (spec 019, R1; see `build.rs`).
#[must_use]
pub fn embedded_serial() -> u64 {
    super::EMBEDDED_SERIAL
}

impl Bundle {
    /// The `(file name, TOML)` pairs that are recipes, in file-name order.
    #[must_use]
    pub fn recipe_files(&self) -> Vec<(&str, &str)> {
        self.files
            .iter()
            .filter(|(name, _)| !NON_RECIPE_FILES.contains(&name.as_str()))
            .map(|(name, text)| (name.as_str(), text.as_str()))
            .collect()
    }

    /// The bundle's tag vocabulary text (empty when absent).
    #[must_use]
    pub fn tags_text(&self) -> &str {
        self.files.get("tags.toml").map_or("", String::as_str)
    }

    /// The bundle's `hosts.toml` text. Recorded for provenance; the receiver
    /// always validates against the embedded allowlist instead (R3).
    #[must_use]
    pub fn hosts_text(&self) -> &str {
        self.files.get("hosts.toml").map_or("", String::as_str)
    }
}

/// Builds a bundle from a local catalog directory (spec 019, R1).
///
/// Runs the same lint CI runs first and **refuses to build on any error**, so a
/// published bundle is always a lint-clean catalog.
///
/// # Errors
///
/// Returns the lint diagnostics, or a single diagnostic when a parameter is
/// invalid or a file cannot be read.
pub fn build_bundle(
    catalog_dir: &Path,
    serial: u64,
    source_commit: &str,
    min_ddl_version: &str,
) -> Result<Bundle, Vec<Diagnostic>> {
    if semver::Version::parse(min_ddl_version).is_err() {
        return Err(vec![Diagnostic {
            file: catalog_dir.into(),
            line: 1,
            message: format!("`{min_ddl_version}` is not a semantic version"),
            hint: "pass a version such as 0.0.4".into(),
        }]);
    }
    let hosts_path = catalog_dir.join("hosts.toml");
    let errors = lint_paths(&[catalog_dir.to_path_buf()], &hosts_path, None);
    if !errors.is_empty() {
        return Err(errors);
    }

    let mut files = BTreeMap::new();
    for name in NON_RECIPE_FILES {
        let path = catalog_dir.join(name);
        let text = read_bounded(&path)?;
        files.insert(name.to_owned(), text);
    }
    let entries = std::fs::read_dir(catalog_dir).map_err(|error| {
        vec![Diagnostic {
            file: catalog_dir.into(),
            line: 1,
            message: error.to_string(),
            hint: "check the catalog directory".into(),
        }]
    })?;
    let mut recipes: Vec<(String, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if NON_RECIPE_FILES.contains(&name) {
            continue;
        }
        recipes.push((name.to_owned(), path));
    }
    recipes.sort();
    for (name, path) in recipes {
        files.insert(name, read_bounded(&path)?);
    }

    Ok(Bundle {
        bundle_schema: BUNDLE_SCHEMA,
        catalog_serial: serial,
        source_commit: source_commit.to_owned(),
        min_ddl_version: min_ddl_version.to_owned(),
        files,
    })
}

fn read_bounded(path: &Path) -> Result<String, Vec<Diagnostic>> {
    let text = std::fs::read_to_string(path).map_err(|error| {
        vec![Diagnostic {
            file: path.into(),
            line: 1,
            message: error.to_string(),
            hint: "check the file path and permissions".into(),
        }]
    })?;
    if text.len() as u64 > MAX_RECIPE_BYTES {
        return Err(vec![Diagnostic {
            file: path.into(),
            line: 1,
            message: format!("catalog document exceeds {MAX_RECIPE_BYTES} bytes"),
            hint: "reduce the document size".into(),
        }]);
    }
    Ok(text)
}

/// Parses a bundle document, enforcing the size bound and the JSON shape
/// (spec 019, R3). A failure here rejects the bundle whole.
///
/// # Errors
///
/// See [`BundleRejection`].
pub fn parse_bundle(bytes: &[u8]) -> Result<Bundle, BundleRejection> {
    if bytes.len() as u64 > MAX_BUNDLE_BYTES {
        return Err(BundleRejection::Oversize {
            limit: MAX_BUNDLE_BYTES,
            got: bytes.len() as u64,
        });
    }
    serde_json::from_slice(bytes).map_err(|error| BundleRejection::Malformed {
        detail: error.to_string(),
    })
}

/// Applies the whole-document checks and validates every recipe against the
/// **embedded** host allowlist (spec 019, R3).
///
/// # Errors
///
/// Returns a [`BundleRejection`] when the whole bundle is refused. Per-recipe
/// problems never reach here: they land in [`AcceptedBundle::dropped`].
pub fn accept_bundle(
    bundle: &Bundle,
    embedded_hosts_text: &str,
    embedded_serial: u64,
    running_version: &str,
) -> Result<AcceptedBundle, BundleRejection> {
    if bundle.bundle_schema != BUNDLE_SCHEMA {
        return Err(BundleRejection::UnknownSchema {
            got: bundle.bundle_schema,
        });
    }
    let running =
        semver::Version::parse(running_version).map_err(|error| BundleRejection::Malformed {
            detail: format!("running DDL version `{running_version}` is not semantic: {error}"),
        })?;
    let min = semver::Version::parse(&bundle.min_ddl_version).map_err(|error| {
        BundleRejection::Malformed {
            detail: format!(
                "min_ddl_version `{}` is not semantic: {error}",
                bundle.min_ddl_version
            ),
        }
    })?;
    if min > running {
        return Err(BundleRejection::TooNew {
            min: bundle.min_ddl_version.clone(),
            running: running_version.to_owned(),
        });
    }
    if bundle.catalog_serial <= embedded_serial {
        return Err(BundleRejection::NotNewer {
            serial: bundle.catalog_serial,
            embedded: embedded_serial,
        });
    }
    for name in bundle.files.keys() {
        if !safe_file_name(name) {
            return Err(BundleRejection::Malformed {
                detail: format!("unsafe bundle file name `{name}`"),
            });
        }
    }

    // The remote `hosts.toml` is ignored on purpose (R3, "out of scope: remote
    // changes to the hosts allowlist"): a recipe is only ever allowed the hosts
    // compiled into this binary. Tags travel with the bundle, like CI.
    let recipes = bundle.recipe_files();
    let catalog = Catalog::from_sources(embedded_hosts_text, bundle.tags_text(), &recipes);

    // Persist only what was accepted; the original tags/hosts text is kept for
    // provenance but is re-ignored on load.
    let mut files = BTreeMap::new();
    for (name, text) in &recipes {
        if catalog
            .recipes
            .iter()
            .any(|held| held.file.as_str() == *name)
        {
            files.insert((*name).to_owned(), (*text).to_owned());
        }
    }
    for name in NON_RECIPE_FILES {
        if let Some(text) = bundle.files.get(name) {
            files.insert(name.to_owned(), text.clone());
        }
    }
    let accepted = Bundle {
        bundle_schema: BUNDLE_SCHEMA,
        catalog_serial: bundle.catalog_serial,
        source_commit: bundle.source_commit.clone(),
        min_ddl_version: bundle.min_ddl_version.clone(),
        files,
    };
    Ok(AcceptedBundle {
        bundle: accepted,
        catalog,
    })
}

/// A bundle key must be a bare `*.toml` file name; anything path-like is a
/// malformed bundle, not a recipe.
fn safe_file_name(name: &str) -> bool {
    name.ends_with(".toml")
        && !name.contains(['/', '\\', ':', '\0'])
        && !name.starts_with('.')
        && !name.contains("..")
}

/// Where the accepted bundle is cached (spec 019, R4).
#[must_use]
pub fn cache_path(data_dir: &Path) -> PathBuf {
    data_dir.join("catalog").join("catalog.json")
}

/// The on-disk cache document: the accepted bundle plus when it was fetched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CachedBundle {
    /// UTC Unix seconds of the fetch (spec 019, R4).
    pub fetched_at: u64,
    /// The accepted bundle.
    pub bundle: Bundle,
}

/// A cache entry that loaded and rebuilt into a catalog (spec 019, R4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedCache {
    /// When the bundle was fetched, UTC Unix seconds.
    pub fetched_at: u64,
    /// The cached bundle.
    pub bundle: Bundle,
    /// The catalog rebuilt from the cached recipes against the embedded hosts.
    pub catalog: Catalog,
}

/// Writes the accepted bundle atomically under the XDG data directory
/// (spec 019, R4).
///
/// # Errors
///
/// Returns an [`std::io::Error`] when the file cannot be written, and refuses a
/// bundle with no usable recipe (an empty catalog must never replace the
/// embedded one).
pub fn store_cache(data_dir: &Path, bundle: &Bundle, fetched_at: u64) -> std::io::Result<()> {
    if bundle.recipe_files().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "refusing to cache a bundle with no recipes",
        ));
    }
    let document = CachedBundle {
        fetched_at,
        bundle: bundle.clone(),
    };
    let mut json = serde_json::to_vec_pretty(&document).map_err(std::io::Error::other)?;
    json.push(b'\n');
    write_atomic(&cache_path(data_dir), &json)
}

/// Loads the cached bundle and rebuilds a catalog from it (spec 019, R4).
///
/// A missing, oversized, corrupt or empty cache is ignored and logged, never
/// fatal: the embedded catalog is always available.
#[must_use]
pub fn load_cache(data_dir: &Path) -> Option<LoadedCache> {
    let path = cache_path(data_dir);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            tracing::warn!(%error, path = %path.display(), "ignoring unreadable catalog cache");
            return None;
        }
    };
    if bytes.len() as u64 > MAX_BUNDLE_BYTES * 2 {
        tracing::warn!(path = %path.display(), "ignoring oversized catalog cache");
        return None;
    }
    let document: CachedBundle = match serde_json::from_slice(&bytes) {
        Ok(document) => document,
        Err(error) => {
            tracing::warn!(%error, path = %path.display(), "ignoring corrupt catalog cache");
            return None;
        }
    };
    let recipes = document.bundle.recipe_files();
    let catalog =
        Catalog::from_sources(super::EMBEDDED_HOSTS, document.bundle.tags_text(), &recipes);
    if catalog.recipes.is_empty() {
        tracing::warn!(path = %path.display(), "ignoring catalog cache with no usable recipes");
        return None;
    }
    if !catalog.skipped.is_empty() {
        tracing::warn!(
            path = %path.display(),
            skipped = catalog.skipped.len(),
            "catalog cache dropped recipes that no longer validate"
        );
    }
    Some(LoadedCache {
        fetched_at: document.fetched_at,
        bundle: document.bundle,
        catalog,
    })
}

/// Writes `bytes` to `path` by writing a sibling temp file, fsyncing it and
/// renaming it into place (spec 019, R4 "atomically").
fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temp = temp_path(path);
    let write = || -> std::io::Result<()> {
        let mut file = std::fs::File::create(&temp)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(bytes)?;
        file.sync_all()
    };
    if let Err(error) = write() {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }
    std::fs::rename(&temp, path)
}

fn temp_path(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "catalog.json".to_owned());
    path.with_file_name(format!(".{name}.tmp.{}.{unique}", std::process::id()))
}

/// Whether an automatic check is due (spec 019, R5): never checked, or at least
/// [`CHECK_INTERVAL_SECS`] ago. `now` is passed in so tests use a fake clock;
/// a clock that moved backwards is treated as "not due".
#[must_use]
pub fn catalog_check_due(last_check: Option<u64>, now: u64) -> bool {
    last_check.is_none_or(|last| now.saturating_sub(last) >= CHECK_INTERVAL_SECS)
}

/// The UTC calendar date of a serial (`YYYY-MM-DD`), or `None` for serial `0`
/// ("unknown"). Used by Settings and `ddl catalog status` (spec 019, R5).
#[must_use]
pub fn serial_date(serial: u64) -> Option<String> {
    if serial == 0 {
        return None;
    }
    let (year, month, day) = civil_from_days((serial / 86_400) as i64);
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

/// Days since 1970-01-01 → (year, month, day), Howard Hinnant's
/// `civil_from_days` (public domain algorithm, exact for the whole range).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (year + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    const HOSTS: &str = r#"
schema_version = 1
pages = ["www.doomworld.com"]
downloads = ["www.gamers.org"]
"#;

    fn recipe(id: &str, url_host: &str) -> String {
        format!(
            r#"
schema_version = 1
id = "{id}"
title = "Test {id}"
authors = ["Tester"]
release = "1.0"
page = "https://www.doomworld.com/{id}"
description = "A test mapset."
permission = "Free to redistribute."
base_game = {{ game = "doom2" }}
[[packages]]
id = "main"
page = "https://www.doomworld.com/{id}"
urls = ["https://{url_host}/{id}.zip"]
size = 4
sha256 = "{hash}"
kind = "zip"
[[packages.members]]
id = "main"
path = "{id}.wad"
size = 4
sha256 = "{hash}"
[[components]]
id = "main"
label = "Main"
description = "The mapset."
optional = false
default = true
position = 1
[[components.members]]
package = "main"
member = "main"
role = "maps"
[compatibility]
family = "vanilla"
complevel = 3
complevel_status = "proposed"
[compatibility.engine]
range = ">=0.30"
status = "inspected"
[verification]
status = "inspected"
tester = "Tester"
date = "2026-01-01"
engine_version = "0.30.1"
inspected = ["source page"]
notes = "Desk research only."
limitations = []
"#,
            hash = "0".repeat(64),
        )
    }

    fn bundle_with(serial: u64, min: &str, recipes: &[(&str, &str)]) -> Bundle {
        let mut files = BTreeMap::new();
        files.insert("hosts.toml".to_owned(), HOSTS.to_owned());
        for (name, text) in recipes {
            files.insert((*name).to_owned(), (*text).to_owned());
        }
        Bundle {
            bundle_schema: BUNDLE_SCHEMA,
            catalog_serial: serial,
            source_commit: "abc1234".to_owned(),
            min_ddl_version: min.to_owned(),
            files,
        }
    }

    #[test]
    fn accept_keeps_good_recipes_and_drops_disallowed_hosts() {
        let bundle = bundle_with(
            200,
            "0.0.1",
            &[
                ("good.toml", &recipe("good", "www.gamers.org")),
                ("evil.toml", &recipe("evil", "evil.example")),
            ],
        );
        let accepted = accept_bundle(&bundle, HOSTS, 100, "0.0.4").expect("accepted");
        assert_eq!(accepted.serial(), 200);
        assert_eq!(
            accepted.catalog.ids().collect::<Vec<_>>(),
            vec!["good"],
            "the non-allowlisted recipe is dropped, the other kept"
        );
        assert_eq!(accepted.dropped().len(), 1);
        assert_eq!(accepted.dropped()[0].file, "evil.toml");
        // The stored bundle only carries the accepted recipe.
        assert_eq!(accepted.bundle.recipe_files().len(), 1);
        assert!(!accepted.bundle.files.contains_key("tags.toml"));
    }

    #[test]
    fn accept_drops_a_lint_failure_and_keeps_the_rest() {
        let broken = recipe("broken", "www.gamers.org").replace("complevel = 3", "complevel = 18");
        let bundle = bundle_with(
            200,
            "0.0.1",
            &[
                ("good.toml", &recipe("good", "www.gamers.org")),
                ("broken.toml", &broken),
            ],
        );
        let accepted = accept_bundle(&bundle, HOSTS, 100, "0.0.4").expect("accepted");
        assert_eq!(accepted.catalog.ids().collect::<Vec<_>>(), vec!["good"]);
        assert_eq!(accepted.dropped().len(), 1);
    }

    #[test]
    fn whole_document_rejections_are_distinct() {
        let good = bundle_with(
            200,
            "0.0.1",
            &[("good.toml", &recipe("good", "www.gamers.org"))],
        );
        assert_eq!(
            accept_bundle(&good, HOSTS, 300, "0.0.4"),
            Err(BundleRejection::NotNewer {
                serial: 200,
                embedded: 300
            })
        );
        assert_eq!(
            accept_bundle(&good, HOSTS, 200, "0.0.4"),
            Err(BundleRejection::NotNewer {
                serial: 200,
                embedded: 200
            }),
            "equal serials are not newer"
        );
        let mut schema = good.clone();
        schema.bundle_schema = 2;
        assert_eq!(
            accept_bundle(&schema, HOSTS, 100, "0.0.4"),
            Err(BundleRejection::UnknownSchema { got: 2 })
        );
        let mut newer = good.clone();
        newer.min_ddl_version = "9.9.9".into();
        assert!(matches!(
            accept_bundle(&newer, HOSTS, 100, "0.0.4"),
            Err(BundleRejection::TooNew { .. })
        ));
        let mut unsafe_name = good.clone();
        unsafe_name
            .files
            .insert("../escape.toml".into(), recipe("escape", "www.gamers.org"));
        assert!(matches!(
            accept_bundle(&unsafe_name, HOSTS, 100, "0.0.4"),
            Err(BundleRejection::Malformed { .. })
        ));
    }

    #[test]
    fn parse_rejects_malformed_and_oversized_documents() {
        assert!(matches!(
            parse_bundle(b"{not json"),
            Err(BundleRejection::Malformed { .. })
        ));
        assert!(matches!(
            parse_bundle(br#"{"bundle_schema":1}"#),
            Err(BundleRejection::Malformed { .. })
        ));
        assert!(
            matches!(
                parse_bundle(br#"{"bundle_schema":1,"extra":true}"#),
                Err(BundleRejection::Malformed { .. })
            ),
            "unknown fields are refused"
        );
        let huge = vec![b' '; MAX_BUNDLE_BYTES as usize + 1];
        assert!(matches!(
            parse_bundle(&huge),
            Err(BundleRejection::Oversize { .. })
        ));
    }

    #[test]
    fn a_bundle_round_trips_through_json() {
        let bundle = bundle_with(
            200,
            "0.0.1",
            &[("good.toml", &recipe("good", "www.gamers.org"))],
        );
        let json = serde_json::to_vec(&bundle).unwrap();
        assert_eq!(parse_bundle(&json).unwrap(), bundle);
    }

    #[test]
    fn cache_round_trips_and_is_ignored_when_corrupt() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = bundle_with(
            200,
            "0.0.1",
            &[("good.toml", &recipe("good", "www.gamers.org"))],
        );
        assert!(load_cache(dir.path()).is_none(), "no cache yet");
        store_cache(dir.path(), &bundle, 1234).unwrap();
        let loaded = load_cache(dir.path()).expect("cache loads");
        assert_eq!(loaded.fetched_at, 1234);
        assert_eq!(loaded.catalog.ids().collect::<Vec<_>>(), vec!["good"]);

        std::fs::write(cache_path(dir.path()), b"{corrupt").unwrap();
        assert!(load_cache(dir.path()).is_none(), "corrupt cache ignored");
        std::fs::write(cache_path(dir.path()), b"{}").unwrap();
        assert!(load_cache(dir.path()).is_none(), "wrong shape ignored");
    }

    #[test]
    fn an_empty_bundle_is_never_cached() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = bundle_with(200, "0.0.1", &[]);
        assert!(store_cache(dir.path(), &bundle, 1).is_err());
        assert!(load_cache(dir.path()).is_none());
    }

    #[test]
    fn build_refuses_a_catalog_that_does_not_lint() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("hosts.toml"), HOSTS).unwrap();
        std::fs::write(dir.path().join("bad.toml"), recipe("bad", "evil.example")).unwrap();
        let errors = build_bundle(dir.path(), 1, "abc", "0.0.4").expect_err("lint fails");
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.file.ends_with("bad.toml")));
    }

    #[test]
    fn build_collects_recipes_hosts_and_tags() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("hosts.toml"), HOSTS).unwrap();
        std::fs::write(
            dir.path().join("tags.toml"),
            "schema_version = 1\n[[tags]]\nid = \"featured\"\nlabel = \"Featured\"\ndescription = \"d\"\norder = 1\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("good.toml"),
            recipe("good", "www.gamers.org"),
        )
        .unwrap();
        let bundle = build_bundle(dir.path(), 42, "deadbeef", "0.0.4").expect("builds");
        assert_eq!(bundle.catalog_serial, 42);
        assert_eq!(bundle.source_commit, "deadbeef");
        assert_eq!(bundle.recipe_files().len(), 1);
        assert!(bundle.files.contains_key("tags.toml"));
        assert!(bundle.files.contains_key("hosts.toml"));
        assert!(build_bundle(dir.path(), 42, "deadbeef", "not-semver").is_err());
    }

    #[test]
    fn throttle_is_due_when_never_checked_or_older_than_a_day() {
        assert!(catalog_check_due(None, 1_000));
        assert!(!catalog_check_due(Some(1_000), 1_000));
        assert!(!catalog_check_due(
            Some(1_000),
            1_000 + CHECK_INTERVAL_SECS - 1
        ));
        assert!(catalog_check_due(Some(1_000), 1_000 + CHECK_INTERVAL_SECS));
        assert!(
            !catalog_check_due(Some(2_000), 1_000),
            "a backwards clock is not due"
        );
    }

    #[test]
    fn serials_format_as_utc_dates() {
        assert_eq!(serial_date(0), None);
        assert_eq!(serial_date(86_400).as_deref(), Some("1970-01-02"));
        assert_eq!(serial_date(1_700_000_000).as_deref(), Some("2023-11-14"));
        assert_eq!(serial_date(1_767_225_600).as_deref(), Some("2026-01-01"));
    }
}

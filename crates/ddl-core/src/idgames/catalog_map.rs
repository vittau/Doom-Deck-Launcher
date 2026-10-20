//! Matching archive paths against the curated catalog (spec 050, R8).
//!
//! The `in_catalog` ranking signal says "a DDL recipe pins this archive", and
//! the browse UI uses it to point at the reviewed recipe instead of presenting
//! the same mapset twice. There is no need for the full recipe schema here:
//! [`catalog_paths`] reads the top-level `id` and each `[[packages]]`
//! `idgames_path`, using `toml` as an untyped document so one recipe under
//! review elsewhere can never fail the index build.

use std::collections::BTreeMap;
use std::path::Path;

/// The largest recipe DDL reads while collecting paths.
const MAX_RECIPE_BYTES: u64 = 1024 * 1024;

/// Archive path (lowercase, with `.zip`) → recipe id.
pub type CatalogPaths = BTreeMap<String, String>;

/// Collects `idgames_path` → recipe id from a catalog directory.
///
/// A missing directory yields an empty map: the index is also useful without a
/// checkout of the recipes. Unreadable or malformed recipes are logged and
/// skipped, never fatal.
#[must_use]
pub fn catalog_paths(catalog_dir: &Path) -> CatalogPaths {
    let mut paths = CatalogPaths::new();
    let Ok(entries) = std::fs::read_dir(catalog_dir) else {
        return paths;
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
        .filter(|path| {
            !matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("hosts.toml" | "tags.toml")
            )
        })
        .collect();
    files.sort();
    for path in files {
        let Some(text) = read_bounded(&path) else {
            continue;
        };
        let Ok(document) = toml::from_str::<toml::Value>(&text) else {
            tracing::warn!(file = %path.display(), "skipping a recipe that is not TOML");
            continue;
        };
        let Some(recipe) = document.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        if recipe.is_empty() || recipe.len() > 64 {
            continue;
        }
        let Some(packages) = document.get("packages").and_then(toml::Value::as_array) else {
            continue;
        };
        for package in packages {
            let Some(idgames_path) = package.get("idgames_path").and_then(toml::Value::as_str)
            else {
                continue;
            };
            if !idgames_path.ends_with(".zip") || idgames_path.contains("!/") {
                continue;
            }
            paths
                .entry(idgames_path.to_ascii_lowercase())
                .or_insert_with(|| recipe.to_owned());
        }
    }
    paths
}

fn read_bounded(path: &Path) -> Option<String> {
    let metadata = std::fs::metadata(path).ok()?;
    if metadata.len() > MAX_RECIPE_BYTES {
        tracing::warn!(file = %path.display(), "skipping an oversized recipe");
        return None;
    }
    match std::fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(error) => {
            tracing::warn!(file = %path.display(), error = %error, "skipping an unreadable recipe");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_paths_are_read_without_the_full_schema() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("auger-zenith.toml"),
            r#"
id = "auger-zenith"
title = "AUGER;ZENITH"

[[packages]]
id = "free"
idgames_path = "levels/doom2/Ports/megawads/dbp37_augzen.zip"
"#,
        )
        .unwrap();
        std::fs::write(dir.path().join("hosts.toml"), "pages = []\n").unwrap();
        std::fs::write(dir.path().join("broken.toml"), "this is not toml = = =\n").unwrap();
        let paths = catalog_paths(dir.path());
        assert_eq!(
            paths.get("levels/doom2/ports/megawads/dbp37_augzen.zip"),
            Some(&"auger-zenith".to_owned())
        );
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn a_missing_directory_is_empty_not_an_error() {
        assert!(catalog_paths(Path::new("/nonexistent/ddl/catalog")).is_empty());
    }

    #[test]
    fn the_real_catalog_is_readable() {
        let dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../catalog"));
        let paths = catalog_paths(dir);
        assert_eq!(
            paths.get("levels/doom2/megawads/av.zip"),
            Some(&"alien-vendetta".to_owned())
        );
        assert!(paths.len() > 10);
    }
}

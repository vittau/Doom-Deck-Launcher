//! Loading a catalog from a directory on disk (specs 044 and the e2e sample).
//!
//! The app embeds `catalog/` at build time ([`Catalog::embedded`]); the
//! maintenance commands (`check-links`, `e2e-sample`) read the checkout
//! instead, so they see a recipe the moment it is edited.

use std::path::Path;

use super::lint::MAX_RECIPE_BYTES;
use super::{Catalog, SkippedRecipe};

/// Files in a catalog directory that are not recipes.
const NON_RECIPE_FILES: [&str; 2] = ["hosts.toml", "tags.toml"];

/// Loads and validates every recipe in `dir` with the same rules as the
/// embedded catalog. A recipe that cannot be read or does not validate lands
/// in [`Catalog::skipped`] with its reason, exactly as at app start.
///
/// # Errors
///
/// A message when `dir` or its `hosts.toml` cannot be read.
pub fn load_catalog_dir(dir: &Path) -> Result<Catalog, String> {
    let read = |path: &Path| {
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
    };
    let hosts_text = read(&dir.join("hosts.toml"))?;
    // A missing vocabulary is an empty one, as in `lint_paths`.
    let tags_text = read(&dir.join("tags.toml")).unwrap_or_default();

    let entries = std::fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".toml") && !NON_RECIPE_FILES.contains(&name.as_str()))
        .collect();
    names.sort();

    let mut files = Vec::with_capacity(names.len());
    let mut unreadable = Vec::new();
    for name in names {
        let path = dir.join(&name);
        match read(&path) {
            Ok(text) if text.len() as u64 > MAX_RECIPE_BYTES => unreadable.push(SkippedRecipe {
                file: name,
                reason: format!("catalog document exceeds {MAX_RECIPE_BYTES} bytes"),
            }),
            Ok(text) => files.push((name, text)),
            Err(reason) => unreadable.push(SkippedRecipe { file: name, reason }),
        }
    }
    let pairs: Vec<(&str, &str)> = files
        .iter()
        .map(|(name, text)| (name.as_str(), text.as_str()))
        .collect();
    let mut catalog = Catalog::from_sources(&hosts_text, &tags_text, &pairs);
    catalog.skipped.extend(unreadable);
    Ok(catalog)
}

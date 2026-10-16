//! Loading, validation and embedding of the curated catalog (spec 009, R7).
//!
//! The binary ships the repository `catalog/` directory compiled in (see
//! `crates/ddl-core/build.rs`). [`Catalog::embedded`] parses every recipe with
//! spec 002's typed model and runs the same semantic validation `ddl catalog
//! lint` does, so a bad recipe is a **skipped** entry with a logged reason
//! rather than a crash or a silently accepted recipe (D-002, D-010).

use std::path::Path;

use crate::downloads::Sha256;

use super::lint::{MAX_RECIPE_BYTES, validate};
use super::{Hosts, Recipe};

/// One recipe that passed validation, with the identity of the file it came
/// from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedRecipe {
    /// File name inside the catalog directory, e.g. `sigil.toml`.
    pub file: String,
    /// SHA-256 of the recipe's source bytes at load time. Used as the install
    /// identity so a later catalog update is detectable (spec 009, R2).
    pub sha256: String,
    /// The parsed recipe.
    pub recipe: Recipe,
}

/// A recipe that was skipped and why, so the caller can log it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedRecipe {
    /// File name the problem is in (or `hosts.toml`).
    pub file: String,
    /// Human-readable reason, including the lint hint when one exists.
    pub reason: String,
}

/// The validated catalog: the host allowlist plus every usable recipe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    /// The download/page host allowlist shared by every recipe.
    pub hosts: Hosts,
    /// Recipes that loaded and validated.
    pub recipes: Vec<LoadedRecipe>,
    /// Recipes (or the host file) that were rejected, with reasons.
    pub skipped: Vec<SkippedRecipe>,
}

impl Catalog {
    /// Loads the catalog compiled into the binary.
    #[must_use]
    pub fn embedded() -> Self {
        Self::from_sources(super::EMBEDDED_HOSTS, super::EMBEDDED_RECIPES)
    }

    /// Loads a catalog from raw source text: the host allowlist plus
    /// `(file name, TOML)` pairs.
    ///
    /// Never panics. Every recipe is validated with spec 002's rules; invalid
    /// ones are recorded in [`Catalog::skipped`] with their reason, and a
    /// duplicate id keeps the first file.
    #[must_use]
    pub fn from_sources(hosts_text: &str, files: &[(&str, &str)]) -> Self {
        let mut skipped = Vec::new();
        let hosts = match toml::from_str::<Hosts>(hosts_text) {
            Ok(hosts) => hosts,
            Err(error) => {
                let reason = format!("catalog hosts are invalid: {error}");
                tracing::warn!(reason, "catalog hosts rejected; loading no recipes");
                skipped.push(SkippedRecipe {
                    file: "hosts.toml".to_owned(),
                    reason,
                });
                Hosts {
                    schema_version: 1,
                    pages: Vec::new(),
                    downloads: Vec::new(),
                }
            }
        };

        let mut recipes: Vec<LoadedRecipe> = Vec::with_capacity(files.len());
        for (file, text) in files {
            match load_recipe(file, text, &hosts) {
                Ok(loaded)
                    if recipes
                        .iter()
                        .any(|held| held.recipe.id == loaded.recipe.id) =>
                {
                    let reason = format!("duplicate recipe id `{}`", loaded.recipe.id);
                    tracing::warn!(file, reason, "catalog recipe skipped");
                    skipped.push(SkippedRecipe {
                        file: (*file).to_owned(),
                        reason,
                    });
                }
                Ok(loaded) => recipes.push(loaded),
                Err(reason) => {
                    tracing::warn!(file, reason, "catalog recipe skipped");
                    skipped.push(SkippedRecipe {
                        file: (*file).to_owned(),
                        reason,
                    });
                }
            }
        }

        Catalog {
            hosts,
            recipes,
            skipped,
        }
    }

    /// The loaded recipe with this id, if present.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&LoadedRecipe> {
        self.recipes.iter().find(|held| held.recipe.id == id)
    }

    /// The parsed recipe with this id, if present.
    #[must_use]
    pub fn recipe(&self, id: &str) -> Option<&Recipe> {
        self.get(id).map(|held| &held.recipe)
    }

    /// The SHA-256 of the recipe file with this id, if present.
    #[must_use]
    pub fn recipe_sha256(&self, id: &str) -> Option<&str> {
        self.get(id).map(|held| held.sha256.as_str())
    }

    /// Every loaded recipe id, in catalog order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.recipes.iter().map(|held| held.recipe.id.as_str())
    }
}

fn load_recipe(file: &str, text: &str, hosts: &Hosts) -> Result<LoadedRecipe, String> {
    if text.len() as u64 > MAX_RECIPE_BYTES {
        return Err(format!("recipe exceeds {MAX_RECIPE_BYTES} bytes"));
    }
    let recipe: Recipe = toml::from_str(text).map_err(|error| error.message().to_owned())?;
    let violations = validate(&recipe, Path::new(file), hosts);
    if !violations.is_empty() {
        let reason = violations
            .iter()
            .map(|violation| format!("{} (hint: {})", violation.message, violation.hint))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(reason);
    }
    Ok(LoadedRecipe {
        file: file.to_owned(),
        sha256: Sha256::of(text.as_bytes()).to_hex(),
        recipe,
    })
}

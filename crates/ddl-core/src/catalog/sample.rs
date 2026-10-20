//! The lean install set of the real-network e2e (spec 044, R4).
//!
//! The e2e used to install every recipe and, at 89 recipes, spent 41 minutes
//! and died on one slow 156 MB archive. [`e2e_sample`] picks a small,
//! deterministic subset from the catalog itself, so nobody keeps a list by
//! hand, and `ddl catalog e2e-sample` prints it for the workflow:
//!
//! - the **smallest recipe per first-source download host**: the host of each
//!   package's first URL, which is the mirror the downloader tries first;
//! - **every recipe with optional components**, each free component (one the
//!   player does not have to supply) installed once, which includes every
//!   widescreen status-bar case;
//! - each recipe's total download stays under a **size cap** (64 MiB by
//!   default), unless no recipe for that host fits under it.
//!
//! Full-catalog coverage lives in `ddl catalog check-links` (spec 044, R1–R3),
//! which checks every mirror's headers without downloading anything.

use std::collections::{BTreeMap, BTreeSet};

use url::Url;

use super::Recipe;

/// Default cap on one recipe's total download size: 64 MiB (spec 044, R4).
pub const DEFAULT_MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;

/// One `ddl mapset install` the e2e should run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleEntry {
    /// The recipe id.
    pub recipe: String,
    /// The optional component to enable with `--with`; `None` is a plain
    /// install.
    pub component: Option<String>,
    /// Why the entry is in the sample, for the run summary.
    pub reason: String,
}

/// The chosen installs and the recipes the size cap kept out.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Sample {
    /// Installs in a stable order: by recipe id, plain install first.
    pub entries: Vec<SampleEntry>,
    /// Recipes that have optional components but exceed the cap, with the
    /// reason; they are not in `entries`.
    pub skipped: Vec<(String, String)>,
}

impl Sample {
    /// The distinct recipe ids in the sample, sorted.
    #[must_use]
    pub fn recipe_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.entries.iter().map(|e| e.recipe.as_str()).collect();
        ids.dedup();
        ids
    }

    /// The sample as tab-separated lines `recipe<TAB>component<TAB>reason`, for
    /// the workflow. The component is `-` for a plain install: a shell `read`
    /// with a tab `IFS` merges adjacent tabs, so an empty field would shift the
    /// reason into its place. No component id can be `-` (ids start with a
    /// letter or digit).
    #[must_use]
    pub fn to_tsv(&self) -> String {
        let mut text = String::new();
        for entry in &self.entries {
            text.push_str(&format!(
                "{}\t{}\t{}\n",
                entry.recipe,
                entry.component.as_deref().unwrap_or("-"),
                entry.reason
            ));
        }
        text
    }
}

/// A recipe's total download size: every package, once.
fn download_bytes(recipe: &Recipe) -> u64 {
    recipe.packages.iter().map(|package| package.size).sum()
}

/// The download hosts of a recipe's packages' first URLs, lowercase.
fn first_hosts(recipe: &Recipe) -> BTreeSet<String> {
    recipe
        .packages
        .iter()
        .filter_map(|package| package.urls.first())
        .filter_map(|url| Url::parse(url).ok())
        .filter_map(|url| url.host_str().map(str::to_ascii_lowercase))
        .collect()
}

/// The optional components a player can install without supplying a file.
fn free_components(recipe: &Recipe) -> impl Iterator<Item = &str> {
    recipe
        .components
        .iter()
        .filter(|component| component.optional && !component.user_supplied)
        .map(|component| component.id.as_str())
}

/// Chooses the e2e install set from `recipes` (spec 044, R4).
///
/// Deterministic: the same catalog always gives the same sample, with ties
/// broken by recipe id. For each first-source host the smallest recipe under
/// `max_archive_bytes` is taken; when none fits, the smallest one overall, so
/// the host stays covered. A recipe with free optional components is taken for
/// each of them, unless it is over the cap and not also needed for a host.
#[must_use]
pub fn e2e_sample(recipes: &[&Recipe], max_archive_bytes: u64) -> Sample {
    let mut by_host: BTreeMap<String, Vec<&Recipe>> = BTreeMap::new();
    for recipe in recipes {
        for host in first_hosts(recipe) {
            by_host.entry(host).or_default().push(recipe);
        }
    }

    // recipe id -> reasons its plain install is wanted
    let mut plain: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (host, mut candidates) in by_host {
        candidates.sort_by(|a, b| (download_bytes(a), &a.id).cmp(&(download_bytes(b), &b.id)));
        let fits = candidates
            .iter()
            .find(|recipe| download_bytes(recipe) <= max_archive_bytes);
        let (pick, over_cap) = match fits {
            Some(recipe) => (*recipe, false),
            None => (candidates[0], true),
        };
        let reason = if over_cap {
            format!("smallest recipe for host {host} (over the size cap; none fits)")
        } else {
            format!("smallest recipe for host {host}")
        };
        plain.entry(pick.id.as_str()).or_default().push(reason);
    }

    let mut entries = Vec::new();
    let mut skipped = Vec::new();
    let mut ordered: Vec<&&Recipe> = recipes.iter().collect();
    ordered.sort_by(|a, b| a.id.cmp(&b.id));
    for recipe in ordered {
        let id = recipe.id.as_str();
        let components: Vec<&str> = free_components(recipe).collect();
        let hosted = plain.contains_key(id);
        let too_big = download_bytes(recipe) > max_archive_bytes;
        if !components.is_empty() && too_big && !hosted {
            skipped.push((
                id.to_owned(),
                format!(
                    "{} bytes is over the {max_archive_bytes}-byte cap",
                    download_bytes(recipe)
                ),
            ));
            continue;
        }
        if let Some(reasons) = plain.get(id) {
            entries.push(SampleEntry {
                recipe: id.to_owned(),
                component: None,
                reason: reasons.join("; "),
            });
        }
        for component in components {
            entries.push(SampleEntry {
                recipe: id.to_owned(),
                component: Some(component.to_owned()),
                reason: format!("optional component `{component}`"),
            });
        }
    }
    Sample { entries, skipped }
}

#[cfg(test)]
mod tests;

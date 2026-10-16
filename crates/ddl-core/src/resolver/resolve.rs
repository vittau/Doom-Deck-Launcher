//! Turning a recipe, a profile and installed state into a validated
//! [`LaunchPlan`] (spec 009, R5).
//!
//! [`resolve`] is deliberately pure: it reads the IWAD registry, the install
//! record and the files they name, and it computes paths and the plan, but it
//! writes nothing. Creating the per-profile directories and the canonical IWAD
//! symlink is a separate [`Resolution::prepare`] step, so a resolution can be
//! inspected and shown before anything is touched.

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::catalog::{
    Complevel as RecipeComplevel, ComplevelAlias, Component, ComponentMember, Game, Recipe, Role,
};
use crate::dsda_config::{ALLOWED_KEYS, DsdaConfigManager, Overrides};
use crate::engines::dsda::{
    AssignAllowlist, AssignOverride, Complevel, LaunchPlan, Skill, ValidationError, Warp,
};
use crate::iwads::{CanonicalLinks, Game as IwadsGame, IwadTable, Registry, RegistryEntry};
use crate::library::{InstallManifest, InstalledMember};
use crate::profiles::{Profile, ProfileDirs};

use super::blocker::{Blocker, Engine, Summary, SummaryContent};

/// The IWAD state and data root a resolution reads from.
///
/// The registry (spec 007) answers which file to use; `data_dir` locates DDL's
/// canonical-link tree (`<data>/iwads/`) and per-profile directories, which is
/// also why it is passed here rather than threaded through the recipe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Iwads<'a> {
    /// DDL's data dir (`~/.local/share/ddl`).
    pub data_dir: PathBuf,
    /// The identified-IWAD registry (spec 007, R5).
    pub registry: &'a Registry,
}

impl<'a> Iwads<'a> {
    /// Bundles `registry` with the data dir its canonical links live under.
    #[must_use]
    pub fn new(data_dir: impl Into<PathBuf>, registry: &'a Registry) -> Self {
        Self {
            data_dir: data_dir.into(),
            registry,
        }
    }
}

/// A resolved, launchable mapset.
#[derive(Debug)]
pub struct Resolution {
    plan: LaunchPlan,
    summary: Summary,
    iwad: IwadBinding,
}

#[derive(Debug)]
struct IwadBinding {
    entry: RegistryEntry,
    links: CanonicalLinks,
    table: IwadTable,
}

impl Resolution {
    /// The launch plan (spec 003) for the UI to execute.
    #[must_use]
    pub fn plan(&self) -> &LaunchPlan {
        &self.plan
    }

    /// The "Will launch" summary for the Play setup panel (spec 009, R6).
    #[must_use]
    pub fn summary(&self) -> &Summary {
        &self.summary
    }

    /// Creates the per-profile directories and the canonical IWAD symlink the
    /// plan needs, so [`Resolution::validate`] can pass.
    ///
    /// # Errors
    ///
    /// Returns the first filesystem error, including a failed IWAD link.
    pub fn prepare(&self) -> io::Result<()> {
        self.plan.prepare_dirs()?;
        self.iwad
            .links
            .canonical_path(&self.iwad.entry, &self.iwad.table)
            .map_err(|error| io::Error::other(format!("canonical IWAD link: {error}")))?;
        Ok(())
    }

    /// Validates the plan (spec 003, R3). Call after [`Resolution::prepare`].
    ///
    /// # Errors
    ///
    /// Returns the first [`ValidationError`].
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.plan.validate(&allowlist())
    }
}

/// The `-assign` keys DDL owns (spec 005, R7).
fn allowlist() -> AssignAllowlist {
    AssignAllowlist::with_keys(ALLOWED_KEYS.iter().copied())
}

/// Resolves `recipe` and `profile` into a launch plan, or a typed [`Blocker`].
///
/// # Errors
///
/// See [`Blocker`]. Resolution writes nothing; call [`Resolution::prepare`].
pub fn resolve(
    recipe: &Recipe,
    profile: &Profile,
    iwads: &Iwads<'_>,
    manifest: &InstallManifest,
    engine: Option<&Engine>,
    config: &DsdaConfigManager,
) -> Result<Resolution, Blocker> {
    if profile.recipe_id != recipe.id {
        return Err(Blocker::IncompatibleChoice {
            detail: format!(
                "this profile is for `{}`, not `{}`",
                profile.recipe_id, recipe.id
            ),
        });
    }
    if !(1..=5).contains(&profile.skill) {
        return Err(Blocker::IncompatibleChoice {
            detail: format!(
                "skill {} is outside DDL's accepted range 1-5",
                profile.skill
            ),
        });
    }
    let engine = engine.ok_or(Blocker::EngineMissing)?;
    if !engine.executable.is_absolute() || !engine.executable.is_file() {
        return Err(Blocker::EngineMissing);
    }

    let game = recipe.base_game.game;
    let iwad_game = map_game(game);
    let table = IwadTable::embedded();

    // IWAD selection and a cheap drift check; full verification happens when the
    // canonical link is refreshed in `prepare`.
    let accepted = recipe.base_game.variants.clone();
    let entry = select_iwad(iwads.registry, profile, iwad_game, &accepted).ok_or_else(|| {
        Blocker::MissingIwad {
            game,
            accepted: accepted.clone(),
        }
    })?;
    match fs::metadata(&entry.path) {
        Ok(metadata) if metadata.is_file() && metadata.len() == entry.size => {}
        _ => {
            return Err(Blocker::MissingIwad { game, accepted });
        }
    }
    let variant_id = entry.variant.clone().unwrap_or_default();
    let variant = table
        .get(&variant_id)
        .ok_or_else(|| Blocker::IntegrityFailed {
            detail: format!("unknown IWAD variant `{variant_id}`"),
        })?;
    let links = CanonicalLinks::new(&iwads.data_dir);
    let canonical = links.root().join(&variant.id).join(&variant.filename);

    let enabled: Vec<&str> = profile.components.iter().map(String::as_str).collect();
    let active = recipe
        .active_components(&enabled)
        .map_err(|detail| Blocker::IncompatibleChoice { detail })?;

    let mut files: Vec<PathBuf> = Vec::new();
    let mut patches: Vec<PathBuf> = Vec::new();
    let mut content: Vec<SummaryContent> = Vec::new();
    for component in &active {
        for reference in &component.members {
            let installed = installed_member(manifest, component, reference)?;
            verify_installed(installed)?;
            if reference.role == Role::Dehacked {
                patches.push(installed.path.clone());
            } else {
                files.push(installed.path.clone());
                content.push(SummaryContent {
                    position: files.len(),
                    label: component.label.clone(),
                    path: installed.path.clone(),
                });
            }
        }
    }

    let complevel = resolve_complevel(recipe, iwad_game)?;
    let skill = Skill::new(profile.skill).map_err(|error| Blocker::IncompatibleChoice {
        detail: error.to_string(),
    })?;
    let warp = resolve_warp(recipe)?;
    let dirs = ProfileDirs::new(&iwads.data_dir, &profile.id);
    let assigns = resolve_assigns(profile)?;

    let plan = LaunchPlan {
        executable: engine.executable.clone(),
        iwad: canonical.clone(),
        files,
        patches,
        complevel,
        skill,
        warp,
        pistol_start: profile.pistol_start,
        record: None,
        config: config.active_path(),
        save_dir: dirs.save,
        data_dir: dirs.data,
        shot_dir: dirs.shots,
        working_dir: dirs.run,
        assigns,
    };

    let summary = Summary {
        base_game: game,
        iwad_present: true,
        iwad_variant: entry.variant.clone(),
        iwad_path: canonical,
        content,
        complevel: complevel.value(),
        complevel_status: recipe.compatibility.complevel_status,
        skill: profile.skill,
        pistol_start: profile.pistol_start,
        engine_version: engine.version.clone(),
    };

    Ok(Resolution {
        plan,
        summary,
        iwad: IwadBinding {
            entry: entry.clone(),
            links,
            table,
        },
    })
}

/// The best registry entry for the profile's choice, honouring an override.
fn select_iwad<'a>(
    registry: &'a Registry,
    profile: &Profile,
    game: IwadsGame,
    accepted: &[String],
) -> Option<&'a RegistryEntry> {
    match &profile.iwad_variant {
        Some(variant) => registry.entries().iter().find(|entry| {
            entry.variant.as_deref() == Some(variant.as_str())
                && entry.game == Some(game)
                && (accepted.is_empty() || accepted.iter().any(|held| held == variant))
        }),
        None => registry.best_for(game, accepted),
    }
}

fn installed_member<'a>(
    manifest: &'a InstallManifest,
    component: &Component,
    reference: &ComponentMember,
) -> Result<&'a InstalledMember, Blocker> {
    if component.user_supplied {
        return manifest.find_imported(&component.id).ok_or_else(|| {
            Blocker::ComponentNotProvided {
                component: component.id.clone(),
                label: component.label.clone(),
            }
        });
    }
    let (Some(package), Some(member)) = (reference.package.as_deref(), reference.member.as_deref())
    else {
        return Err(Blocker::IncompatibleChoice {
            detail: format!(
                "component `{}` has a member without a package reference",
                component.id
            ),
        });
    };
    manifest
        .find_downloaded(package, member)
        .ok_or(Blocker::NotInstalled)
}

/// Confirms an installed file still matches the install record (R5).
fn verify_installed(entry: &InstalledMember) -> Result<(), Blocker> {
    let metadata = fs::metadata(&entry.path).map_err(|_| Blocker::IntegrityFailed {
        detail: format!("`{}` is missing", entry.path.display()),
    })?;
    if !metadata.is_file() || metadata.len() != entry.size {
        return Err(Blocker::IntegrityFailed {
            detail: format!(
                "`{}` changed size since it was installed",
                entry.path.display()
            ),
        });
    }
    let sha256 = crate::iwads::identify::sha256_file(&entry.path).map_err(|error| {
        Blocker::IntegrityFailed {
            detail: format!("could not read `{}`: {error}", entry.path.display()),
        }
    })?;
    if !sha256.eq_ignore_ascii_case(&entry.sha256) {
        return Err(Blocker::IntegrityFailed {
            detail: format!("`{}` failed its SHA-256 check", entry.path.display()),
        });
    }
    Ok(())
}

fn resolve_complevel(recipe: &Recipe, game: IwadsGame) -> Result<Complevel, Blocker> {
    // Aliases as DSDA-Doom resolves them (`docs/research/dsda-engine.md` §1):
    // `boom` is 9 (Boom 2.02), `mbf` is 11, `mbf21` is 21. `vanilla` is resolved
    // by DDL from the identified family (D-017).
    let value = match &recipe.compatibility.complevel {
        RecipeComplevel::Number(number) => *number,
        RecipeComplevel::Alias(ComplevelAlias::Vanilla) => vanilla_complevel(game),
        RecipeComplevel::Alias(ComplevelAlias::Boom) => 9,
        RecipeComplevel::Alias(ComplevelAlias::Mbf) => 11,
        RecipeComplevel::Alias(ComplevelAlias::Mbf21) => 21,
    };
    Complevel::new(value).map_err(|error| Blocker::IncompatibleChoice {
        detail: error.to_string(),
    })
}

/// The numeric complevel a recipe's `vanilla` means for an identified family
/// (decision D-017: doom 3, doom2 2, tnt/plutonia 4).
fn vanilla_complevel(game: IwadsGame) -> u8 {
    match game {
        IwadsGame::Doom => 3,
        IwadsGame::Doom2 => 2,
        IwadsGame::Tnt | IwadsGame::Plutonia => 4,
        IwadsGame::Nerve => 2,
    }
}

fn resolve_warp(recipe: &Recipe) -> Result<Option<Warp>, Blocker> {
    let Some(values) = recipe.compatibility.runtime.warp.as_ref() else {
        return Ok(None);
    };
    let detail = |error: crate::engines::dsda::ValueRangeError| Blocker::IncompatibleChoice {
        detail: error.to_string(),
    };
    match values.as_slice() {
        [] => Ok(None),
        [single] => Warp::single(*single).map(Some).map_err(detail),
        [episode, map, ..] => Warp::episode_map(*episode, *map).map(Some).map_err(detail),
    }
}

fn resolve_assigns(profile: &Profile) -> Result<Vec<AssignOverride>, Blocker> {
    let mut overrides = Overrides::new();
    for (key, value) in &profile.overrides {
        overrides
            .set(key, *value)
            .map_err(|error| Blocker::IncompatibleChoice {
                detail: error.to_string(),
            })?;
    }
    Ok(overrides
        .iter()
        .map(|(key, value)| AssignOverride::new(key, value.to_string()))
        .collect())
}

/// Maps a recipe's base game onto spec 007's identified game.
fn map_game(game: Game) -> IwadsGame {
    match game {
        Game::Doom => IwadsGame::Doom,
        Game::Doom2 => IwadsGame::Doom2,
        Game::Tnt => IwadsGame::Tnt,
        Game::Plutonia => IwadsGame::Plutonia,
    }
}

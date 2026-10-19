//! Original games derived from identified IWADs (spec 032, D-027).
//!
//! Confirmed vanilla levels: DSDA-Doom v0.30.1 settings.c L59–75:
//! <https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L59-L75>.
//! Nerve is an expansion PWAD, not a standalone base game.
use std::path::Path;

use super::{Game, Registry, RegistryEntry};
use crate::dsda_config::DsdaConfigManager;
use crate::engines::dsda::LaunchPlan;
use crate::local::{self, Compatibility, CustomIwad, CustomSetup};
use crate::profiles::Profile;

/// One original game, independent of the number of identified copies.
#[derive(Debug, Clone)]
pub struct BaseGame {
    pub game: Game,
    pub iwad: RegistryEntry,
}
impl BaseGame {
    /// Stable target namespace; neither recipe nor custom ids accept `:`.
    #[must_use]
    pub fn id(&self) -> String {
        format!("base:{}", self.game)
    }
    #[must_use]
    pub fn title(&self) -> &'static str {
        match self.game {
            Game::Doom
                if self
                    .iwad
                    .variant
                    .as_deref()
                    .is_some_and(|v| v.contains("shareware")) =>
            {
                "Doom (shareware)"
            }
            Game::Doom
                if self.iwad.variant.as_deref().is_some_and(|v| {
                    v.starts_with("doom-")
                        && !v.contains("ultimate")
                        && !v.contains("1.9ud")
                        && !v.contains("kex")
                        && !v.contains("bfg")
                }) =>
            {
                "Doom"
            }
            Game::Doom => "Ultimate Doom",
            Game::Doom2 => "Doom II",
            Game::Tnt => "TNT: Evilution",
            Game::Plutonia => "The Plutonia Experiment",
            Game::Nerve => unreachable!("Nerve is not an IWAD"),
        }
    }
    #[must_use]
    pub fn complevel(&self) -> u8 {
        match self.game {
            Game::Doom => 3,
            Game::Doom2 => 2,
            Game::Tnt | Game::Plutonia => 4,
            Game::Nerve => unreachable!(),
        }
    }
}
/// Identified games in table order; the registry owns classic-copy preference.
#[must_use]
pub fn list(registry: &Registry) -> Vec<BaseGame> {
    let mut present = Registry::new();
    for entry in registry.entries().iter().filter(|e| e.path.is_file()) {
        present.insert(entry.clone());
    }
    Game::ALL
        .into_iter()
        .filter(|g| *g != Game::Nerve)
        .filter_map(|game| {
            present
                .best_for(game, &[])
                .cloned()
                .map(|iwad| BaseGame { game, iwad })
        })
        .collect()
}
/// Builds through spec 026's verified user-file path; no recipe or setup is saved.
///
/// # Errors
/// Returns an error for a mismatched profile, unavailable copy, or changed file.
pub fn build_plan(
    base: &BaseGame,
    profile: &Profile,
    engine: &local::Engine,
    registry: &Registry,
    data: &Path,
    config: &DsdaConfigManager,
) -> Result<LaunchPlan, local::LaunchError> {
    if profile.recipe_id != base.id() {
        return Err(local::LaunchError::Iwad(
            "profile belongs to another game".into(),
        ));
    }
    let selected = if let Some(path) = &profile.iwad_path {
        registry
            .get(path)
            .filter(|e| e.game == Some(base.game) && e.variant.is_some() && e.path.is_file())
            .or(Some(&base.iwad))
    } else if let Some(variant) = &profile.iwad_variant {
        registry.best_for(base.game, std::slice::from_ref(variant))
    } else {
        Some(&base.iwad)
    }
    .ok_or_else(|| {
        local::LaunchError::Iwad(
            "selected IWAD copy is no longer identified; rescan IWAD folders".into(),
        )
    })?;
    // Restrict the transient registry to the selected physical copy; canonical
    // links verify its recorded hash before local::build_plan returns it.
    let mut selected_registry = Registry::new();
    selected_registry.insert(selected.clone());
    let mut setup = CustomSetup::new(
        format!("base-{}", base.game),
        base.title(),
        CustomIwad::Identified {
            variant: selected.variant.clone().expect("identified entry"),
        },
        Compatibility::parse(&base.complevel().to_string())?,
        0,
    );
    for extra in &profile.extras {
        setup.add(extra.clone())?;
    }
    local::build_plan(&setup, profile, engine, &selected_registry, data, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iwads::{FileTime, Source};
    use crate::profiles::ProfileDirs;
    use tempfile::TempDir;
    fn entry(root: &Path, file: &str, variant: &str, game: Game) -> RegistryEntry {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"IWAD\0\0\0\0\x0c\0\0\0").unwrap();
        let meta = std::fs::metadata(&path).unwrap();
        RegistryEntry {
            path: path.clone(),
            variant: Some(variant.into()),
            candidates: vec![],
            game: Some(game),
            sha256: crate::iwads::identify::sha256_file(&path).unwrap(),
            size: meta.len(),
            mtime: FileTime::from_metadata(&meta).unwrap(),
            source: Source::Steam,
        }
    }
    #[test]
    fn base_plan_has_vanilla_argv_and_isolated_profiles_and_extras() {
        let temp = TempDir::new().unwrap();
        let mut registry = Registry::new();
        registry.insert(entry(
            temp.path(),
            "base/doom2.wad",
            "doom2-1.9",
            Game::Doom2,
        ));
        registry.insert(entry(
            temp.path(),
            "other/doom2.wad",
            "doom2-1.9",
            Game::Doom2,
        ));
        let base = list(&registry).pop().unwrap();
        let profile = Profile::new("base-doom2-profile-1", "Default", base.id());
        let engine = local::Engine {
            executable: temp.path().join("dsda.appimage"),
            version: "0.30.1".into(),
        };
        let config = DsdaConfigManager::new(temp.path().join("config")).unwrap();
        let plan = build_plan(&base, &profile, &engine, &registry, temp.path(), &config).unwrap();
        let argv: Vec<_> = plan
            .to_command()
            .iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert!(argv.windows(2).any(|w| w == ["-complevel", "2"]));
        assert!(argv.contains(&"-iwad".into()));
        assert!(!argv.contains(&"-file".into()));
        assert_eq!(
            std::fs::read_link(&plan.iwad).unwrap(),
            std::fs::canonicalize(&base.iwad.path).unwrap()
        );
        assert_ne!(
            plan.save_dir,
            ProfileDirs::new(temp.path(), "base-doom-profile-1").save
        );
        let mut selected = profile.clone();
        selected.iwad_path = Some(registry.entries()[1].path.clone());
        selected.skill = 5;
        selected.pistol_start = true;
        let extra = temp.path().join("fix.deh");
        std::fs::write(&extra, b"Patch File for DeHackEd v3.0\n").unwrap();
        selected.extras.push(local::UserFile::read(&extra).unwrap());
        let plan = build_plan(&base, &selected, &engine, &registry, temp.path(), &config).unwrap();
        assert_eq!(
            std::fs::read_link(&plan.iwad).unwrap(),
            std::fs::canonicalize(&registry.entries()[1].path).unwrap()
        );
        assert_eq!(plan.skill.value(), 5);
        assert!(plan.pistol_start);
        assert_eq!(plan.patches, vec![extra.clone()]);
        std::fs::remove_file(&extra).unwrap();
        assert!(build_plan(&base, &selected, &engine, &registry, temp.path(), &config).is_err());
        selected.extras.clear();
        std::fs::remove_file(selected.iwad_path.as_ref().unwrap()).unwrap();
        let plan = build_plan(&base, &selected, &engine, &registry, temp.path(), &config).unwrap();
        assert_eq!(
            std::fs::read_link(&plan.iwad).unwrap(),
            std::fs::canonicalize(&base.iwad.path).unwrap()
        );
        assert!(
            !serde_json::to_string(&profile)
                .unwrap()
                .contains("iwad_path")
        );
        assert!(
            serde_json::to_string(&selected)
                .unwrap()
                .contains("iwad_path")
        );
        let recipe_profile = Profile::new("sigil-default", "Default", "sigil");
        assert!(
            !serde_json::to_string(&recipe_profile)
                .unwrap()
                .contains("iwad_path")
        );
    }
}

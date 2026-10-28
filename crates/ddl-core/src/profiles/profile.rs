//! A per-mapset play profile (spec 009, R4).
//!
//! A profile is the player's choices layered on top of a recipe: which optional
//! components are enabled, pistol start, an optional IWAD-variant
//! override, the last time it was played, and the transient scalar overrides
//! spec 005 renders as `-assign`. It never contains recipe-required
//! compatibility values, which stay owned by the recipe.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::catalog::Recipe;

/// Schema version written into each profile document.
pub const SCHEMA_VERSION: u32 = 1;

/// One saved profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Persisted schema version.
    pub schema_version: u32,
    /// Stable profile id, also the profile directory and file name.
    pub id: String,
    /// Friendly name for the UI.
    pub name: String,
    /// The recipe id this profile plays.
    pub recipe_id: String,
    /// Enabled optional component ids (main is implicit).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
    /// Whether to reset to a pistol start on every map (`-pistolstart`).
    #[serde(default)]
    pub pistol_start: bool,
    /// Optional IWAD variant override (spec 007); `None` picks the best match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iwad_variant: Option<String>,
    /// Preferred physical copy for a base game (spec 032), rechecked at launch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iwad_path: Option<std::path::PathBuf>,
    /// Unix seconds of the last launch, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_played: Option<u64>,
    /// Per-profile scalar DSDA overrides (spec 005, R7), rendered as `-assign`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub overrides: BTreeMap<String, i64>,
    /// Ordered user files layered on top of the recipe's own files (spec 026,
    /// R5). Additive: a profile written before spec 026 loads with none. Files
    /// stay in this single order; DSDA applies WAD/ZIP entries as the `-file`
    /// list and DEH/BEX entries as the `-deh` list, each in list order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extras: Vec<crate::local::UserFile>,
}

impl<'de> Deserialize<'de> for Profile {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Accept only the retired field in addition to the current schema. Its
        // value is discarded by the deserializer and never reaches the model.
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Document {
            schema_version: u32,
            id: String,
            name: String,
            recipe_id: String,
            #[serde(default)]
            components: Vec<String>,
            #[serde(default)]
            pistol_start: bool,
            #[serde(default)]
            iwad_variant: Option<String>,
            #[serde(default)]
            iwad_path: Option<std::path::PathBuf>,
            #[serde(default)]
            last_played: Option<u64>,
            #[serde(default)]
            overrides: BTreeMap<String, i64>,
            #[serde(default)]
            extras: Vec<crate::local::UserFile>,
            #[serde(default, rename = "skill")]
            _retired_skill: serde::de::IgnoredAny,
        }
        let doc = Document::deserialize(deserializer)?;
        Ok(Self {
            schema_version: doc.schema_version,
            id: doc.id,
            name: doc.name,
            recipe_id: doc.recipe_id,
            components: doc.components,
            pistol_start: doc.pistol_start,
            iwad_variant: doc.iwad_variant,
            iwad_path: doc.iwad_path,
            last_played: doc.last_played,
            overrides: doc.overrides,
            extras: doc.extras,
        })
    }
}

impl Profile {
    /// A new profile for `recipe_id`.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        recipe_id: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id: id.into(),
            name: name.into(),
            recipe_id: recipe_id.into(),
            components: Vec::new(),
            pistol_start: false,
            iwad_variant: None,
            iwad_path: None,
            last_played: None,
            overrides: BTreeMap::new(),
            extras: Vec::new(),
        }
    }

    /// Records that the profile was played at `now` (Unix seconds).
    pub fn touch(&mut self, now: u64) {
        self.last_played = Some(now);
    }

    /// Enables or disables an optional component id, keeping ids unique.
    pub fn set_component(&mut self, id: &str, enabled: bool) {
        self.components.retain(|held| held != id);
        if enabled {
            self.components.push(id.to_owned());
        }
    }

    /// Checks the profile against `recipe`: schema, target and a legal
    /// component combination (exclusive groups and replacements included).
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError`] describing the first problem found.
    pub fn validate(&self, recipe: &Recipe) -> Result<(), ProfileError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(ProfileError::UnsupportedSchema(self.schema_version));
        }
        if self.recipe_id != recipe.id {
            return Err(ProfileError::RecipeMismatch {
                profile: self.recipe_id.clone(),
                recipe: recipe.id.clone(),
            });
        }
        let enabled: Vec<&str> = self.components.iter().map(String::as_str).collect();
        recipe
            .active_components(&enabled)
            .map_err(ProfileError::InvalidCombination)?;
        Ok(())
    }
}

/// Why a profile is invalid.
#[derive(Debug)]
pub enum ProfileError {
    /// A filesystem operation failed.
    Io(std::io::Error),
    /// The profile document is not valid JSON.
    Parse(String),
    /// The profile exceeds the read bound.
    TooLarge(u64),
    /// The profile declares an unsupported schema version.
    UnsupportedSchema(u32),
    /// The profile id is not a safe file name.
    UnsafeId(String),
    /// The profile targets a different recipe.
    RecipeMismatch {
        /// The recipe id the profile names.
        profile: String,
        /// The recipe id it was validated against.
        recipe: String,
    },
    /// The enabled components are not a legal combination.
    InvalidCombination(String),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "profile I/O error: {error}"),
            Self::Parse(message) => write!(f, "invalid profile: {message}"),
            Self::TooLarge(limit) => write!(f, "profile exceeds the {limit} byte read limit"),
            Self::UnsupportedSchema(version) => write!(
                f,
                "profile schema_version {version} is not supported (expected {SCHEMA_VERSION})"
            ),
            Self::UnsafeId(id) => write!(f, "`{id}` is not a safe profile id"),
            Self::RecipeMismatch { profile, recipe } => {
                write!(f, "this profile is for `{profile}`, not `{recipe}`")
            }
            Self::InvalidCombination(detail) => write!(f, "invalid component selection: {detail}"),
        }
    }
}

impl std::error::Error for ProfileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProfileError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_recipe;
    use super::*;

    #[test]
    fn a_default_profile_validates_and_exports_overrides() {
        let recipe = test_recipe();
        let profile = Profile::new("p1", "Play SIGIL", "sigil-test");
        assert!(profile.validate(&recipe).is_ok());
        assert!(profile.components.is_empty());
    }

    #[test]
    fn old_skill_is_ignored_and_not_saved() {
        for skill in [
            serde_json::json!(3),
            serde_json::json!(999),
            serde_json::json!("ignored"),
        ] {
            let mut document =
                serde_json::to_value(Profile::new("p1", "Existing", "sigil-test")).unwrap();
            document["skill"] = skill;
            let loaded: Profile = serde_json::from_value(document).unwrap();
            assert!(loaded.validate(&test_recipe()).is_ok());
            assert!(serde_json::to_value(loaded).unwrap().get("skill").is_none());
        }
    }

    #[test]
    fn invalid_target_and_combination_are_rejected() {
        let recipe = test_recipe();
        let profile = Profile::new("p1", "Play", "other");
        assert!(matches!(
            profile.validate(&recipe),
            Err(ProfileError::RecipeMismatch { .. })
        ));

        let mut profile = Profile::new("p1", "Play", "sigil-test");
        profile.components = vec!["does-not-exist".to_owned()];
        assert!(matches!(
            profile.validate(&recipe),
            Err(ProfileError::InvalidCombination(_))
        ));
    }

    #[test]
    fn toggling_components_keeps_ids_unique() {
        let mut profile = Profile::new("p1", "Play", "sigil-test");
        profile.set_component("compat", true);
        profile.set_component("compat", true);
        assert_eq!(profile.components, ["compat"]);
        profile.set_component("compat", false);
        assert!(profile.components.is_empty());
    }

    #[test]
    fn unknown_profile_fields_still_fail() {
        let mut document = serde_json::to_value(Profile::new("p1", "Play", "sigil")).unwrap();
        document["unknown"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Profile>(document).is_err());
    }
}

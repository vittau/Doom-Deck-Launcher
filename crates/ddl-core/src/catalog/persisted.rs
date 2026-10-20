//! Read adapters for recipes copied into machine-written JSON (spec 055).
//! Catalog TOML continues to deserialize the strict current `Recipe` directly.
use super::*;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredRecipe {
    schema_version: u32,
    id: String,
    title: String,
    authors: Vec<String>,
    release: String,
    page: String,
    description: String,
    permission: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    screenshots: Vec<Screenshot>,
    #[serde(default)]
    images: Vec<Image>,
    base_game: BaseGame,
    packages: Vec<Package>,
    components: Vec<Component>,
    compatibility: StoredCompatibility,
    verification: Verification,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum LegacyEvidenceStatus {
    Confirmed,
    Proposed,
    NeedsDeckValidation,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredCompatibility {
    family: PortFamily,
    complevel: Complevel,
    // Removed in 046; accepted only for machine-written recipe copies.
    #[serde(default, rename = "complevel_status")]
    _complevel_status: Option<LegacyEvidenceStatus>,
    engine: EngineEvidence,
    #[serde(default)]
    runtime: RuntimeDefaults,
}
impl From<StoredRecipe> for Recipe {
    fn from(old: StoredRecipe) -> Self {
        let mut complevel = old.compatibility.complevel;
        // Only copies written before 046 (marked by the removed status field)
        // are mapped; current-schema state keeps its exact representation.
        if old.compatibility._complevel_status.is_some()
            && let Complevel::Number(n) = complevel
            && let Some(alias) = ComplevelAlias::naming(n, old.base_game.game)
        {
            complevel = Complevel::Alias(alias);
        }
        Self {
            schema_version: old.schema_version,
            id: old.id,
            title: old.title,
            authors: old.authors,
            release: old.release,
            page: old.page,
            description: old.description,
            permission: old.permission,
            tags: old.tags,
            screenshots: old.screenshots,
            images: old.images,
            base_game: old.base_game,
            packages: old.packages,
            components: old.components,
            verification: old.verification,
            compatibility: Compatibility {
                family: old.compatibility.family,
                complevel,
                engine: old.compatibility.engine,
                runtime: old.compatibility.runtime,
            },
        }
    }
}
/// Reads a recipe held in state; serialization still writes the current type.
pub fn recipe<'de, D: Deserializer<'de>>(de: D) -> Result<Recipe, D::Error> {
    StoredRecipe::deserialize(de).map(Into::into)
}
/// Reads a previous-generation recipe held in state.
pub fn optional_recipe<'de, D: Deserializer<'de>>(de: D) -> Result<Option<Recipe>, D::Error> {
    Option::<StoredRecipe>::deserialize(de).map(|r| r.map(Into::into))
}
/// Reads app.json's installed recipe copies.
pub fn recipes<'de, D: Deserializer<'de>>(de: D) -> Result<BTreeMap<String, Recipe>, D::Error> {
    BTreeMap::<String, StoredRecipe>::deserialize(de)
        .map(|recipes| recipes.into_iter().map(|(k, v)| (k, v.into())).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Deserialize)]
    struct Holder {
        #[serde(deserialize_with = "recipe")]
        recipe: Recipe,
    }
    #[test]
    fn state_only_accepts_removed_status_and_normalizes_named_numbers() {
        let current = Catalog::embedded().recipe("auger-zenith").unwrap().clone();
        for (game, number, expected) in [
            (Game::Doom2, 2, Some(ComplevelAlias::Vanilla)),
            (Game::Doom, 3, Some(ComplevelAlias::Vanilla)),
            (Game::Tnt, 4, Some(ComplevelAlias::Vanilla)),
            (Game::Doom2, 9, Some(ComplevelAlias::Boom)),
            (Game::Doom2, 11, Some(ComplevelAlias::Mbf)),
            (Game::Doom2, 21, Some(ComplevelAlias::Mbf21)),
            (Game::Doom2, 3, None),
            (Game::Doom2, 17, None),
        ] {
            for status in ["confirmed", "proposed", "needs-deck-validation"] {
                let mut old = current.clone();
                old.base_game.game = game;
                old.compatibility.complevel = Complevel::Number(number);
                let mut value = serde_json::to_value(old).unwrap();
                value["compatibility"]["complevel_status"] = status.into();
                assert!(
                    serde_json::from_value::<Recipe>(value.clone()).is_err(),
                    "catalog type must stay strict"
                );
                let loaded: Holder =
                    serde_json::from_value(serde_json::json!({"recipe": value})).unwrap();
                assert_eq!(
                    loaded.recipe.compatibility.complevel,
                    expected.map_or(Complevel::Number(number), Complevel::Alias)
                );
                let saved = serde_json::to_value(loaded.recipe).unwrap();
                assert!(saved["compatibility"].get("complevel_status").is_none());
            }
        }
    }
    #[test]
    fn migrated_v006_copy_matches_a_catalog_recipe_that_now_uses_the_alias() {
        // A v0.0.6 copy holds `complevel = 11` plus the removed status; the catalog
        // now names the same level `mbf`. After migration the spec-020 comparison
        // sees no difference, so no update is reported.
        let mut catalog = Catalog::embedded().recipe("auger-zenith").unwrap().clone();
        catalog.base_game.game = Game::Doom2;
        catalog.compatibility.complevel = Complevel::Alias(ComplevelAlias::Mbf);
        let mut old = catalog.clone();
        old.compatibility.complevel = Complevel::Number(11);
        let mut value = serde_json::to_value(old).unwrap();
        value["compatibility"]["complevel_status"] = "confirmed".into();
        let loaded: Holder = serde_json::from_value(serde_json::json!({"recipe": value})).unwrap();
        assert_eq!(loaded.recipe, catalog);
    }
    #[test]
    fn current_schema_numeric_complevel_round_trips_unchanged() {
        let mut current = Catalog::embedded().recipe("auger-zenith").unwrap().clone();
        current.base_game.game = Game::Doom2;
        current.compatibility.complevel = Complevel::Number(11);
        let value = serde_json::to_value(&current).unwrap();
        assert!(value["compatibility"].get("complevel_status").is_none());
        let loaded: Holder = serde_json::from_value(serde_json::json!({"recipe": value})).unwrap();
        assert_eq!(loaded.recipe, current);
        assert_eq!(loaded.recipe.compatibility.complevel, Complevel::Number(11));
    }
    #[test]
    fn state_recipe_still_rejects_unrelated_unknown_fields() {
        let mut recipe =
            serde_json::to_value(Catalog::embedded().recipe("sigil").unwrap()).unwrap();
        recipe["compatibility"]["future_meaning"] = true.into();
        assert!(serde_json::from_value::<Holder>(serde_json::json!({"recipe": recipe})).is_err());
    }
}

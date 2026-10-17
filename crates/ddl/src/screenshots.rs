//! Bundled mapset screenshots (spec 023, R2/R3; D-024).
//!
//! The images are committed under `ui/assets/screenshots/<recipe-id>/` with a
//! `SOURCES.toml` beside them and are embedded into the binary by Slint
//! (`embed_resources(EmbedFiles)`), so they show with no runtime download. This
//! module exposes the build-time index of that directory to Rust, the way the
//! painted covers reach the UI as `@image-url` assets: the UI draws the screen
//! itself and captions it with the recipe's `credit` (spec 023, R3).
//!
//! `EMBEDDED_SCREENSHOTS` and [`card`] are the seam the UI wiring (spec 023, R3)
//! consumes; until then they are exercised only by the test below so the index
//! can never drift from the committed catalog.
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/screenshots.rs"));

/// Asset paths relative to the `ui/` asset root for `recipe_id`, card first.
#[must_use]
pub fn for_recipe(recipe_id: &str) -> &'static [&'static str] {
    EMBEDDED_SCREENSHOTS
        .iter()
        .find(|(id, _)| *id == recipe_id)
        .map_or(&[], |(_, assets)| *assets)
}

/// The card screenshot for `recipe_id`, if one is bundled.
#[must_use]
pub fn card(recipe_id: &str) -> Option<&'static str> {
    for_recipe(recipe_id).first().copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddl_core::catalog::Catalog;

    /// Every recipe that ships a screenshot has it on disk, and every bundled
    /// screenshot directory belongs to a catalog recipe (spec 023, A1).
    #[test]
    fn embedded_screenshots_match_the_catalog() {
        let catalog = Catalog::embedded();
        assert!(catalog.skipped.is_empty(), "{:?}", catalog.skipped);

        let ui = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
        for loaded in &catalog.recipes {
            let id = loaded.recipe.id.as_str();
            let assets = for_recipe(id);
            assert!(
                !assets.is_empty(),
                "recipe `{id}` has no bundled screenshot"
            );
            // The recipe reference and the directory index agree.
            let declared: Vec<&str> = loaded
                .recipe
                .screenshots
                .iter()
                .map(|s| s.asset.as_str())
                .collect();
            assert_eq!(
                assets
                    .iter()
                    .map(|a| a.strip_prefix("assets/screenshots/").unwrap())
                    .collect::<Vec<_>>(),
                declared,
                "recipe `{id}` screenshot assets disagree with the directory index"
            );
            for asset in assets {
                let path = ui.join(asset);
                assert!(path.is_file(), "missing asset {}", path.display());
            }
        }
        for (id, _) in EMBEDDED_SCREENSHOTS {
            assert!(
                catalog.recipe(id).is_some(),
                "bundled screenshots for unknown recipe `{id}`"
            );
        }
    }
}

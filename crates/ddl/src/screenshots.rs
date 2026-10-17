//! Bundled mapset screenshots (spec 023, R2/R3; D-024).
//!
//! The images are committed under `ui/assets/screenshots/<recipe-id>/` with a
//! `SOURCES.toml` beside them and are embedded into the binary by Slint
//! (`embed_resources(EmbedFiles)`), so they show with no runtime download. This
//! module exposes the build-time index of that directory to Rust, the way the
//! painted covers reach the UI as `@image-url` assets: the UI draws the screen
//! itself and captions it with the recipe's `credit` (spec 023, R3).
//!
include!(concat!(env!("OUT_DIR"), "/screenshots.rs"));

/// Bundled paths under the `ui/` asset root, in deterministic filename order.
#[must_use]
pub fn for_recipe(recipe_id: &str) -> &'static [&'static str] {
    EMBEDDED_SCREENSHOTS
        .iter()
        .find(|(id, _)| *id == recipe_id)
        .map_or(&[], |(_, assets)| *assets)
}

/// The first indexed screenshot; `presentation` follows the recipe metadata order.
#[must_use]
pub fn card(recipe_id: &str) -> Option<&'static str> {
    for_recipe(recipe_id).first().copied()
}

/// Resolve only bundled assets, preserving the recipe's order and credit. Updated
/// community recipes without a bundled image retain the painted fallback offline.
pub fn presentation(
    ui: &crate::AppWindow,
    recipe: &ddl_core::catalog::Recipe,
) -> (slint::SharedString, slint::SharedString) {
    let Some(first) = recipe.screenshots.first() else {
        return Default::default();
    };
    let path = format!("assets/screenshots/{}", first.asset);
    let bundled =
        card(&recipe.id) == Some(path.as_str()) || for_recipe(&recipe.id).contains(&path.as_str());
    if !bundled {
        return Default::default();
    }
    (
        path.into(),
        crate::wiring_copy::value(
            ui,
            crate::wiring_copy::Text::ScreenshotCredit,
            &first.credit,
        ),
    )
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
            // The asset inventory agrees; metadata order is exercised by the UI test.
            let mut declared: Vec<&str> = loaded
                .recipe
                .screenshots
                .iter()
                .map(|s| s.asset.as_str())
                .collect();
            declared.sort_unstable();
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

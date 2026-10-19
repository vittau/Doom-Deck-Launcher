//! Bundled mapset screenshots (spec 023, R2/R3; D-024).
//!
//! The images are committed under `ui/assets/screenshots/<recipe-id>/` with a
//! `SOURCES.toml` beside them and are embedded into the binary by Slint
//! (`embed_resources(EmbedFiles)`), so they show with no runtime download. This
//! module exposes the build-time index of that directory to Rust, the way the
//! painted covers reach the UI as `@image-url` assets: the UI draws the screen
//! itself with no caption over it; the credit is plain text on Mapset detail and
//! in About (spec 033, R4).
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

/// Whether the recipe's declared screenshot is one of the bundled assets.
fn bundled(recipe_id: &str, screenshot: &ddl_core::catalog::Screenshot) -> bool {
    let path = format!("assets/screenshots/{}", screenshot.asset);
    for_recipe(recipe_id).contains(&path.as_str())
}

/// Resolve only bundled assets, preserving the recipe's order. Updated
/// community recipes without a bundled image retain the painted fallback offline.
/// The image carries no caption: credits are plain text (spec 033, R4).
pub fn presentation(recipe: &ddl_core::catalog::Recipe) -> slint::SharedString {
    recipe
        .screenshots
        .first()
        .filter(|first| bundled(&recipe.id, first))
        .map_or_else(Default::default, |first| {
            format!("assets/screenshots/{}", first.asset).into()
        })
}

/// The distinct credits of a recipe's bundled screenshots, in recipe order, for
/// Mapset detail's "Screenshots: <credit>" line.
#[must_use]
pub fn recipe_credits(recipe: &ddl_core::catalog::Recipe) -> Vec<&str> {
    let mut credits: Vec<&str> = Vec::new();
    for shot in recipe
        .screenshots
        .iter()
        .filter(|shot| bundled(&recipe.id, shot))
    {
        if !credits.contains(&shot.credit.as_str()) {
            credits.push(&shot.credit);
        }
    }
    credits
}

/// Every distinct credit in the bundled `SOURCES.toml` files, for About.
#[must_use]
pub fn all_credits() -> &'static [&'static str] {
    SCREENSHOT_CREDITS
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
            // Spec 037 (R4): a recipe may ship no screenshot at all and fall back to the
            // painted concept art, so an empty directory is not an error. What must hold
            // is that everything a recipe declares is bundled and that no directory is
            // orphaned.
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

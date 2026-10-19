//! Versioned, offline-first recipe catalog (spec 002; embedded loading in
//! spec 009, R7).
//!
//! No runtime installation, UI, IWAD acquisition or process execution lives
//! here. [`Catalog::embedded`] loads the recipes compiled into the binary and
//! validates them with the spec 002 model, skipping invalid ones with a logged
//! reason.
pub mod bundle;
mod fetch;
mod image;
mod lint;
mod loader;
mod model;
pub mod remote;

pub use bundle::{
    AcceptedBundle, BUNDLE_SCHEMA, Bundle, BundleRejection, CHECK_INTERVAL_SECS, CachedBundle,
    DEFAULT_MIN_DDL_VERSION, LoadedCache, MAX_BUNDLE_BYTES, accept_bundle, build_bundle,
    cache_path, catalog_check_due, embedded_serial, load_cache, parse_bundle, serial_date,
    store_cache,
};
pub use fetch::{Fetch, HttpsFetch, check_package};
pub use image::{
    ImageError, ImageFormat, MAX_SCREENSHOT_BYTES, MAX_SCREENSHOT_HEIGHT, MAX_SCREENSHOT_WIDTH,
    MIN_SCREENSHOT_HEIGHT, MIN_SCREENSHOT_WIDTH, accepted_asset_extension, image_info,
};
pub use lint::{
    Diagnostic, SCREENSHOTS_DIR, check_status_bar_lumps, lint_paths, validate, validate_tags,
    validate_vocabulary, wad_lump_names,
};
pub use loader::{Catalog, LoadedRecipe, SkippedRecipe};
pub use model::*;
pub use remote::{BUNDLE_URL, CatalogUpdateError, HttpsRemoteFetch, RemoteFetch, load_remote};

#[cfg(test)]
mod tests;

include!(concat!(env!("OUT_DIR"), "/embedded_catalog.rs"));

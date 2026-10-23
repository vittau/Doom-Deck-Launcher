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
mod links;
mod lint;
mod loader;
mod model;
mod order;
pub mod persisted;
#[cfg(test)]
mod recipe_fixture;
pub mod remote;
mod sample;
mod source_dir;

pub use bundle::{
    AcceptedBundle, BUNDLE_SCHEMA, Bundle, BundleRejection, CachedBundle, DEFAULT_MIN_DDL_VERSION,
    LoadedCache, MAX_BUNDLE_BYTES, accept_bundle, build_bundle, cache_path, embedded_serial,
    load_cache, parse_bundle, serial_date, serial_datetime, store_cache,
};
pub use fetch::{Fetch, HttpsFetch, check_package};
pub use image::{
    ImageError, ImageFormat, MAX_SCREENSHOT_BYTES, MAX_SCREENSHOT_HEIGHT, MAX_SCREENSHOT_WIDTH,
    MIN_SCREENSHOT_HEIGHT, MIN_SCREENSHOT_WIDTH, accepted_asset_extension, image_info,
};
pub use links::{
    LinkChecker, LinkResult, MAX_IN_FLIGHT, Outcome, PackageFinding, Report as LinkReport,
};
pub use lint::{
    Diagnostic, SCREENSHOTS_DIR, check_status_bar_lumps, lint_paths, validate, validate_tags,
    validate_vocabulary, wad_lump_names,
};
pub use loader::{Catalog, LoadedRecipe, SkippedRecipe};
pub use model::*;
pub use order::{
    COMMUNITY_PROJECTS, DOOMER_BOARDS, compare_browsing_order, compare_discover_order,
    shows_in_category,
};
pub use remote::{BUNDLE_URL, CatalogUpdateError, HttpsRemoteFetch, RemoteFetch, load_remote};
pub use sample::{DEFAULT_MAX_ARCHIVE_BYTES, Sample, SampleEntry, e2e_sample};
pub use source_dir::load_catalog_dir;

#[cfg(test)]
mod tests;

include!(concat!(env!("OUT_DIR"), "/embedded_catalog.rs"));

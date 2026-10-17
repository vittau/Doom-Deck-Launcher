//! Versioned, offline-first recipe catalog (spec 002; embedded loading in
//! spec 009, R7).
//!
//! No runtime installation, UI, IWAD acquisition or process execution lives
//! here. [`Catalog::embedded`] loads the recipes compiled into the binary and
//! validates them with the spec 002 model, skipping invalid ones with a logged
//! reason.
mod fetch;
mod lint;
mod loader;
mod model;

pub use fetch::{Fetch, HttpsFetch, check_package};
pub use lint::{Diagnostic, lint_paths, validate, validate_tags, validate_vocabulary};
pub use loader::{Catalog, LoadedRecipe, SkippedRecipe};
pub use model::*;

#[cfg(test)]
mod tests;

include!(concat!(env!("OUT_DIR"), "/embedded_catalog.rs"));

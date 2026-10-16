//! Versioned, offline-first recipe catalog (spec 002).
//! No runtime installation, UI, IWAD acquisition or process execution lives here.
mod fetch;
mod lint;
mod model;

pub use fetch::{Fetch, HttpsFetch, check_package};
pub use lint::{Diagnostic, lint_paths, validate};
pub use model::*;

#[cfg(test)]
mod tests;

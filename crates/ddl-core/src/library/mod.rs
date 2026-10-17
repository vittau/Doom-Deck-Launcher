//! Installed content: the mapset library and the immutable install record
//! (spec 009, R1–R3).
//!
//! An install turns a validated catalog [`crate::catalog::Recipe`] plus a
//! selection of components into files under
//! `<data>/library/<recipe id>/<release>/`, described by an
//! [`InstallManifest`]. Downloads and extraction go through spec 006's
//! [`crate::downloads`] pipeline; nothing DDL downloads is ever executed.
//!
//! ```no_run
//! use ddl_core::downloads::HostAllowlist;
//! use ddl_core::library::Installer;
//!
//! let installer = Installer::new(
//!     "/home/deck/.local/share/ddl",
//!     "/home/deck/.cache/ddl",
//!     HostAllowlist::new(["www.gamers.org"]),
//! );
//! let recipe = ddl_core::catalog::Catalog::embedded()
//!     .recipe("auger-zenith")
//!     .expect("shipped recipe")
//!     .clone();
//! let manifest = installer
//!     .install(&recipe, &["alternate-blaster"])
//!     .expect("install succeeds");
//! assert_eq!(manifest.recipe_id, "auger-zenith");
//! ```

mod active;
mod change;
mod install;
mod manifest;

#[cfg(test)]
mod tests;

pub use active::ActiveInstall;
pub use change::{Changes, RecipeStatus, classify, reconcile_profile};
pub use install::{
    Clock, IMPORT_DIR, ImportOutcome, InstallError, Installer, LIBRARY_DIR, MAX_MEMBER_BYTES,
    SystemClock, recipe_content_sha256,
};
pub use manifest::{
    InstallManifest, InstalledMember, InstalledPackage, MANIFEST_FILE, ManifestError,
    SCHEMA_VERSION,
};

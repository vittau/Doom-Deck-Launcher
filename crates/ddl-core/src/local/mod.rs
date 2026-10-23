//! Local content: custom setups and extra files with a player-set load order
//! (spec 026, decision D-026).
//!
//! DDL launches **arbitrary user files** — whatever DSDA-Doom accepts on
//! `-file` and `-deh` — with a player-adjustable order, both standalone
//! ([`CustomSetup`]) and layered on top of an installed catalog mapset
//! (`Profile::extras`). User files are referenced **in place** by absolute path,
//! identified by size and SHA-256, and never copied, moved, modified or
//! executed (spec 026, R1/R4/R6).
//!
//! The module is pure logic with no UI dependencies. The pieces:
//!
//! * [`kind`] detects WAD, ZIP and DEH/BEX content and rejects what the engine
//!   would abort on, citing upstream tag `v0.30.1`;
//! * [`setup`] is the versioned model;
//! * [`store`] reads and writes it atomically under `<data>/custom/`;
//! * [`launch`] assembles the DSDA-Doom [`LaunchPlan`](crate::engines::dsda::LaunchPlan)
//!   through the existing spec 003 builder and applies the identity checks.
//!
//! Catalog recipes stay curated and verified; a custom setup and a profile with
//! extras are labelled **Custom — not verified** by the UI.

pub mod kind;
pub mod launch;
pub mod picker;
pub mod setup;
pub mod store;

pub use kind::{
    FileError, IdentityError, Inspection, Kind, MAX_USER_FILE_BYTES, TEXT_PREFIX_BYTES, UserFile,
    detect_kind, is_iwad_header,
};
pub use launch::{
    Engine, LaunchError, accept_changed, assign_allowlist, build_plan, first_issue, recipe_game,
    verify_setup,
};
pub use setup::{
    Compatibility, CustomIwad, CustomSetup, Group, SCHEMA_VERSION, SetupError, group_indices,
    safe_id, slug,
};
pub use store::{DIR, SetupStore, has_saves};

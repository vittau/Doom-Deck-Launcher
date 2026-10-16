//! DoomDeckLauncher core logic.
//!
//! This crate must never depend on a UI toolkit (decision D-005): see
//! `AGENTS.md`. Spec 001 only needs the XDG path resolver in
//! [`platform::paths`]; the module list from `OVERVIEW.md` §4 (`catalog`,
//! `recipes`, `resolver`, ...) arrives with later specs.
#![forbid(unsafe_code)]

pub mod platform;

pub mod catalog;

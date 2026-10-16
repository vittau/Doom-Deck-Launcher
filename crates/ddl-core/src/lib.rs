//! DoomDeckLauncher core logic.
//!
//! This crate must never depend on a UI toolkit (decision D-005): see
//! `AGENTS.md`. Spec 001 needs the XDG path resolver in [`platform::paths`];
//! spec 003 adds the DSDA-Doom launch plan and process runner in
//! [`engines::dsda`]; spec 005 adds the DDL-owned DSDA config in [`dsda_config`]. The rest of the module list from `OVERVIEW.md` §4
//! (`catalog`, `recipes`, `resolver`, ...) arrives with later specs.
#![forbid(unsafe_code)]

pub mod dsda_config;
pub mod engines;
pub mod platform;

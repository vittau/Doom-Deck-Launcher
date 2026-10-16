//! DoomDeckLauncher core logic.
//!
//! This crate must never depend on a UI toolkit (decision D-005): see
//! `AGENTS.md`. Spec 001 needs the XDG path resolver in [`platform::paths`];
//! spec 003 adds the DSDA-Doom launch plan and process runner in
//! [`engines::dsda`]; spec 005 adds the DDL-owned DSDA config in [`dsda_config`].
//! Spec 009 adds the mapset [`library`], [`profiles`] and the [`resolver`] that
//! ties a recipe, a profile and installed state into a launch plan. Earlier
//! specs add [`catalog`] (recipes), [`downloads`] (fetching and safe extraction)
//! and [`iwads`] (the base-game library).
#![forbid(unsafe_code)]

pub mod catalog;
pub mod downloads;
pub mod dsda_config;
pub mod engines;
pub mod iwads;
pub mod library;
pub mod platform;
pub mod profiles;
pub mod resolver;

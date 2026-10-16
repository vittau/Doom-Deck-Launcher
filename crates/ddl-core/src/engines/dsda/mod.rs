//! DSDA-Doom integration (decision D-011).
//!
//! The single choke point for launching the engine:
//!
//! * [`plan`] turns a resolved description of "what to play" into an argv
//!   vector and validates it.
//!
//! Facts about DSDA-Doom's parser come from `docs/research/dsda-engine.md`,
//! pinned to upstream tag `v0.30.1`. Every value DDL emits is confirmed there
//! or marked there as needing Deck validation.

pub mod plan;

pub use plan::{
    AssignAllowlist, AssignOverride, Complevel, LaunchPlan, PathField, Skill, ValidationError,
    ValueRangeError, Warp,
};

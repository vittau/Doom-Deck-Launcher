//! DSDA-Doom integration (decision D-011).
//!
//! The single choke point for launching the engine:
//!
//! * [`plan`] turns a resolved description of "what to play" into an argv
//!   vector and validates it;
//! * [`runner`] starts the engine without a shell and reports how it ended,
//!   with a fake for tests;
//! * [`smoke`] probes an installation with `-v`.
//!
//! Facts about DSDA-Doom's parser come from `docs/research/dsda-engine.md`,
//! pinned to upstream tag `v0.30.1`. Every value DDL emits is confirmed there
//! or marked there as needing Deck validation.

pub mod plan;
pub mod runner;
pub mod smoke;

pub use plan::{
    AssignAllowlist, AssignOverride, Complevel, LaunchPlan, PathField, Skill, ValidationError,
    ValueRangeError, Warp,
};
pub use runner::{
    EngineRunner, FakeEngineRunner, LaunchError, LaunchExit, LaunchOutcome, RealEngineRunner,
    RecordedLaunch, SCRUBBED_ENV,
};
pub use smoke::{SMOKE_TIMEOUT, SmokeError, VERSION_PREFIX, smoke_test, smoke_test_with_timeout};

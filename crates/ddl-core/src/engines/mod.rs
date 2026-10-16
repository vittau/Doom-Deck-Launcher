//! Engine integrations.
//!
//! DDL launches exactly one engine (decision D-011): DSDA-Doom. Hardware- and
//! process-dependent behaviour lives behind small traits with fakes
//! (decision D-003), so the logic is testable without the engine, a Deck or a
//! network connection.
//!
//! Spec 003 implements the launch contract in [`dsda`]; spec 004 adds engine
//! discovery, download, installation, rollback and state in the same module.
//! Hardware- and process-dependent behaviour lives behind small traits with
//! fakes (decision D-003), so the logic is testable without the engine, a Deck
//! or a network connection.

pub mod dsda;

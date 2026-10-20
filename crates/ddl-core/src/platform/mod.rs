//! Platform-specific behaviour, behind small, testable interfaces.
//!
//! Spec 001 (R2) only requires XDG directory resolution; process management and
//! Steam Deck integration belong to later specs.

pub mod legacy_import;
pub mod lock;
pub mod paths;
pub mod sandbox;

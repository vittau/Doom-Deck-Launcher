//! IWAD library (spec 007): the known-IWAD table, content-based
//! identification, bounded Steam and user-folder discovery, a persisted
//! registry and the canonical symlinks DSDA-Doom needs.
//!
//! The module is pure logic with no UI dependencies. It never bundles,
//! downloads or executes IWADs; discovery is read-only and bounded, and every
//! remote or on-disk input is treated as untrusted.

pub mod table;

pub use table::{Game, HashKind, IwadTable, IwadVariant, TableError};

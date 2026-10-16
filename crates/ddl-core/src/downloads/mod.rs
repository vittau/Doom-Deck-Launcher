//! Downloads and safe archive extraction (spec 006).
//!
//! This module holds the hardened pipeline that fetches every remote file DDL
//! uses (engine AppImages and mapset packages), verifies it against a pinned
//! SHA-256, and extracts **only the files a recipe names** from ZIP archives.
//! All remote input is untrusted (`OVERVIEW.md` §9, §12), so the pipeline
//! refuses plain HTTP, keeps an explicit host allowlist, bounds every transfer
//! and extraction, and never executes anything it downloads.
//!
//! The submodules are layered: [`hash`], [`host`], [`cancel`] and [`error`]
//! are the small shared types; [`fetch`] is the download pipeline and
//! [`extract`] is the safe ZIP extractor.

pub mod cancel;
pub mod error;
pub mod hash;
pub mod host;

pub use cancel::CancellationToken;
pub use error::{Error, LimitKind};
pub use hash::Sha256;
pub use host::HostAllowlist;

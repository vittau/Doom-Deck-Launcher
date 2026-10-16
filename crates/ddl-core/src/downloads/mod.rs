//! Downloads and safe archive extraction (spec 006).
//!
//! This module holds the hardened pipeline that fetches every remote file DDL
//! uses — engine AppImages and mapset packages — verifies it against a pinned
//! SHA-256, caches it by hash, and extracts **only the files a recipe names**
//! from ZIP archives, including archives nested inside archives. All remote
//! input is untrusted (`OVERVIEW.md` §9, §12), so the pipeline refuses plain
//! HTTP, keeps an explicit host allowlist, bounds every transfer and
//! extraction, and never executes anything it downloads.
//!
//! # Choosing blocking IO
//!
//! The pipeline is **synchronous** and the caller runs it on a worker thread
//! ([`std::thread::spawn`]) rather than on the Slint event loop. That keeps
//! `ddl-core` free of an async runtime and avoids a second executor next to
//! Slint's; the [`CancellationToken`] is the only cross-thread channel needed
//! for cancel, and [`Fetcher`] and [`Extractor`] are `Send + Sync`, so they can
//! be shared. Spec 006 R2 allows either model and requires a justification.
//!
//! # Example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use ddl_core::downloads::{
//!     CancellationToken, DownloadRequest, Fetcher, HostAllowlist, Sha256,
//! };
//!
//! let hash = Sha256::of(b"a known file");
//! let fetcher = Fetcher::new(PathBuf::from("/home/deck/.cache/ddl"));
//! let request = DownloadRequest::new(
//!     "https://example.invalid/package.zip",
//!     hash,
//!     64 * 1024 * 1024,
//!     HostAllowlist::new(["example.invalid"]),
//! );
//! let cancel = CancellationToken::new();
//! let mut progress = |_p| {};
//! let _path = fetcher.download(&request, &cancel, &mut progress);
//! ```

pub mod cancel;
pub mod error;
pub mod extract;
pub mod fetch;
pub mod hash;
pub mod host;

#[cfg(test)]
mod test_http;

pub use cancel::CancellationToken;
pub use error::{Error, LimitKind};
pub use extract::{ExtractLimits, ExtractedMember, Extraction, Extractor, MemberRequest};
pub use fetch::{DEFAULT_REDIRECT_EXCEPTIONS, DownloadRequest, Fetcher, MAX_REDIRECTS, Progress};
pub use hash::Sha256;
pub use host::HostAllowlist;

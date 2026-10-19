//! The hardened download pipeline (spec 006, R1–R3).
//!
//! One [`Fetcher`] fetches any remote file DDL uses — engine AppImages and
//! mapset packages — into a SHA-256-addressed cache. It is deliberately
//! **blocking**: the UI runs it on a worker thread instead of the event loop
//! (spec 006 R2). That keeps `ddl-core` free of an async runtime and lets the
//! Slint event loop and the download share a plain `std::thread` boundary.
//!
//! Guarantees, all enforced here rather than trusted from the caller:
//!
//! - **HTTPS only.** Plain HTTP is refused before a connection is made.
//! - **Host allowlist.** Every URL *and every redirect target* is checked
//!   against the caller's [`HostAllowlist`]. Two documented exceptions, both
//!   redirect-target-only: GitHub's signed asset host, accepted from any
//!   origin, and Dropbox's per-request `*.dl.dropboxusercontent.com` host,
//!   accepted only inside a chain that began at `www.dropbox.com` and only for
//!   a same-host hop (spec 038, R1).
//! - **Bounded.** At most [`MAX_REDIRECTS`] redirects, a connect timeout, a
//!   response-header timeout, an idle timeout ([`BODY_IDLE_TIMEOUT`]) on the
//!   gap between body reads, a size-scaled body deadline (see `body_deadline`),
//!   a hard maximum size and an optional exact expected size.
//! - **Verified.** The body is streamed to a temporary file while SHA-256 is
//!   computed; a mismatch is reported with both hashes.
//! - **Cached.** A verified file is stored as `downloads/<sha256>` and reused;
//!   a cache hit is re-hashed before use.
//! - **Cancellable.** A [`CancellationToken`] is checked between chunks and a
//!   cancel removes the temporary file.
//!
//! Mirrors are tried in order; the next URL is used on any failure (network,
//! HTTP status, host, size or hash), never after a successful match.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256 as Sha256Digest};
use ureq::Agent;
use ureq::Timeout;
use ureq::config::AutoHeaderValue;
use ureq::http::Uri;
use ureq::http::header::LOCATION;
use ureq::unversioned::resolver::DefaultResolver;
use ureq::unversioned::transport::{
    Buffers, ConnectionDetails, Connector, DefaultConnector, NextTimeout, Transport,
};

use super::cancel::CancellationToken;
use super::error::Error;
use super::hash::Sha256;
use super::host::{DROPBOX_REDIRECT_ORIGIN, HostAllowlist, is_dropbox_download_host};

/// The maximum number of redirects followed per URL (spec 006, R1).
pub const MAX_REDIRECTS: u32 = 5;

/// Hosts that are accepted as *redirect targets* even when they are not on the
/// caller's allowlist. GitHub serves release assets through a signed,
/// short-lived `release-assets.githubusercontent.com` URL (confirmed in
/// `docs/research/upstream-sources.md` §1.4); it is a redirect target only.
pub const DEFAULT_REDIRECT_EXCEPTIONS: &[&str] = &["release-assets.githubusercontent.com"];

/// Read size while streaming a body. The progress callback fires once per read,
/// so a larger buffer means fewer, coarser updates.
const READ_BUFFER: usize = 64 * 1024;

/// Connect timeout. An unreachable host should fail fast.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Deadline for receiving the response *headers*. Separate from the body so a
/// server that accepts a connection and then says nothing does not tie up the
/// worker forever.
const RESPONSE_HEADERS_TIMEOUT: Duration = Duration::from_secs(15);

/// Fixed part of a body deadline. Covers connection setup, TLS, redirects and
/// the first byte, so even a tiny file gets a real chance.
pub const BODY_DEADLINE_BASE: Duration = Duration::from_secs(60);

/// Maximum gap between two body reads (spec 024, R1): a connection that stops
/// delivering bytes for this long is treated as stalled and aborted with
/// [`Error::Stalled`], regardless of how much budget the size-scaled body
/// deadline would still allow. It is enforced per read, so a slow-but-steady
/// link that keeps sending below this gap is never punished.
pub const BODY_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

/// Minimum sustained throughput a body must keep, in bytes per second
/// (64 KiB/s). Below this the transfer is treated as stalled, not merely slow.
pub const MIN_SUSTAINED_RATE: u64 = 64 * 1024;

/// Ceiling on the size-scaled body deadline (6 h), so a hostile `expected_size`
/// or `max_size` cannot turn the deadline into an infinite wait.
pub const MAX_BODY_DEADLINE: Duration = Duration::from_secs(6 * 60 * 60);

/// The knobs behind [`body_deadline`] and the idle timeout. Production uses
/// [`Self::default`]; tests shrink all four so a bound can fire in
/// milliseconds.
#[derive(Debug, Clone, Copy)]
struct DeadlinePolicy {
    base: Duration,
    min_rate: u64,
    max: Duration,
    idle: Duration,
}

impl Default for DeadlinePolicy {
    fn default() -> Self {
        Self {
            base: BODY_DEADLINE_BASE,
            min_rate: MIN_SUSTAINED_RATE,
            max: MAX_BODY_DEADLINE,
            idle: BODY_IDLE_TIMEOUT,
        }
    }
}

/// The total receive-body deadline for a transfer of `size` bytes:
/// `base + size / min_rate`, clamped to `max` (6 h in production).
///
/// A *total* deadline is the right shape for a download the user can cancel:
/// it stops a stalled or hostile server without punishing a slow-but-steady
/// link, because the budget grows with the number of bytes still to come. At
/// the 512 MiB cap this allows ~2.3 h (≈ 64 KiB/s), so Eviternity II's 121 MiB
/// archive gets ~33 min — far more than the fixed 300 s it replaced.
///
/// `size` is the exact `expected_size` when the caller knows it, otherwise the
/// request's hard `max_size`; both come from the recipe/engine cap. Sizes are
/// saturated, never overflowed.
#[must_use]
fn body_deadline(policy: DeadlinePolicy, size: u64) -> Duration {
    let transfer = Duration::from_secs(size / policy.min_rate.max(1));
    policy.base.saturating_add(transfer).min(policy.max)
}

/// Progress of one transfer, reported to the caller's callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Bytes received so far.
    pub downloaded: u64,
    /// Total bytes, when the server announced a `Content-Length` or the caller
    /// supplied an expected size.
    pub total: Option<u64>,
}

/// What to fetch and how to constrain it.
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    /// Primary URL followed by mirrors. Tried in order.
    pub urls: Vec<String>,
    /// The SHA-256 the file must have. Always known for DDL's reviewed sources
    /// (D-007, D-010), and the cache key. `None` for a document fetched without
    /// a published hash (spec 019, R3: the catalog bundle), in which case there
    /// is no pin to verify against and no stable cache key to look up, so the
    /// transfer is never reused as if it were verified.
    pub expected_sha256: Option<Sha256>,
    /// The exact expected size, when known. Exceeding it aborts the transfer.
    pub expected_size: Option<u64>,
    /// Hard upper bound on the response size, mirroring the recipe/engine cap.
    pub max_size: u64,
    /// Hosts the download may contact.
    pub allowed_hosts: HostAllowlist,
}

impl DownloadRequest {
    /// A request for a single URL with an explicit size cap.
    #[must_use]
    pub fn new(
        url: impl Into<String>,
        expected_sha256: Sha256,
        max_size: u64,
        allowed_hosts: HostAllowlist,
    ) -> Self {
        Self {
            urls: vec![url.into()],
            expected_sha256: Some(expected_sha256),
            expected_size: None,
            max_size,
            allowed_hosts,
        }
    }

    /// A request with no SHA-256 pin, for a document whose exact bytes DDL
    /// cannot know ahead of time (spec 019, R3: the catalog bundle). Scheme,
    /// host, redirect, size and timeout policy are identical to [`Self::new`];
    /// only the integrity pin and the cache lookup are absent.
    #[must_use]
    pub fn unpinned(url: impl Into<String>, max_size: u64, allowed_hosts: HostAllowlist) -> Self {
        Self {
            urls: vec![url.into()],
            expected_sha256: None,
            expected_size: None,
            max_size,
            allowed_hosts,
        }
    }

    /// Adds the exact expected size.
    #[must_use]
    pub fn with_size(mut self, size: u64) -> Self {
        self.expected_size = Some(size);
        self
    }

    /// Replaces the URL list with a primary plus mirrors.
    #[must_use]
    pub fn with_urls<I, S>(mut self, urls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.urls = urls.into_iter().map(Into::into).collect();
        self
    }
}

/// How one hop sits in its redirect chain, for the redirect-only exceptions
/// (spec 006 R1, spec 038 R1).
#[derive(Debug, Clone, Copy, Default)]
struct Hop<'a> {
    /// The host of the URL that issued this hop's redirect, or `None` for a
    /// request's first URL. A first URL is never reached by a redirect, so no
    /// exception may admit it.
    from: Option<&'a str>,
    /// Whether the chain began at an allowlisted `www.dropbox.com` URL. Only
    /// such a chain may land on a Dropbox per-request download host.
    dropbox_entry: bool,
}

/// Fetches remote files into a content-addressed cache.
pub struct Fetcher {
    agent: Agent,
    downloads_dir: PathBuf,
    redirect_exceptions: Vec<String>,
    deadlines: DeadlinePolicy,
    /// Kept so the agent can be rebuilt when a test knob changes the idle
    /// timeout, which is baked into the transport at construction time.
    #[cfg(any(test, feature = "test-support"))]
    connect_timeout: Duration,
    #[cfg(any(test, feature = "test-support"))]
    response_timeout: Duration,
    #[cfg(any(test, feature = "test-support"))]
    allow_http_for_tests: bool,
}

impl Fetcher {
    /// Creates a fetcher writing into `<cache_dir>/downloads` (spec 006, R3).
    ///
    /// Connect and response-header timeouts are short so an unreachable or
    /// silent server fails fast. The **body** has two bounds (spec 024): an
    /// idle timeout ([`BODY_IDLE_TIMEOUT`], per read) and a size-scaled total
    /// deadline (`base + size / 64 KiB/s`, capped at 6 h). The total deadline
    /// alone would let a big archive that stalled after the headers wait for
    /// the whole budget; the idle timeout aborts it in about a minute, while a
    /// slow-but-steady link below the idle gap still completes.
    #[must_use]
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self::with_timeouts(cache_dir, CONNECT_TIMEOUT, RESPONSE_HEADERS_TIMEOUT)
    }

    /// Creates a fetcher with explicit connect and response-header timeouts.
    ///
    /// The body bounds are still size-scaled and idle-bounded; to shrink them
    /// (tests) use the `with_deadline_policy` and `with_idle_timeout` test
    /// constructors.
    #[must_use]
    pub fn with_timeouts(
        cache_dir: impl Into<PathBuf>,
        connect_timeout: Duration,
        response_timeout: Duration,
    ) -> Self {
        let deadlines = DeadlinePolicy::default();
        let agent = build_agent(connect_timeout, response_timeout, deadlines.idle);
        Self {
            agent,
            downloads_dir: cache_dir.into().join("downloads"),
            redirect_exceptions: DEFAULT_REDIRECT_EXCEPTIONS
                .iter()
                .map(|host| (*host).to_owned())
                .collect(),
            deadlines,
            #[cfg(any(test, feature = "test-support"))]
            connect_timeout,
            #[cfg(any(test, feature = "test-support"))]
            response_timeout,
            #[cfg(any(test, feature = "test-support"))]
            allow_http_for_tests: false,
        }
    }

    /// Test-only: a fetcher with an overridden response-header timeout while
    /// the connect timeout and every body bound keep their production defaults.
    /// The gated headless engine-download fixture needs this so a starved test
    /// thread cannot trip the 15 s header timeout (spec 024, R4).
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn with_response_header_timeout(
        cache_dir: impl Into<PathBuf>,
        response_timeout: Duration,
    ) -> Self {
        Self::with_timeouts(cache_dir, CONNECT_TIMEOUT, response_timeout)
    }

    /// Replaces the size-scaling constants behind the body deadline. Test-only:
    /// production uses [`Fetcher::new`]'s defaults, which keep tests under two
    /// seconds while this lets them fire a deadline in milliseconds.
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn with_deadline_policy(mut self, base: Duration, min_rate: u64, max: Duration) -> Self {
        self.deadlines = DeadlinePolicy {
            base,
            min_rate,
            max,
            idle: self.deadlines.idle,
        };
        self
    }

    /// Replaces the body idle timeout. Test-only: shrinking it lets a loopback
    /// test observe a stall in milliseconds. Rebuilds the agent because the cap
    /// lives in the transport (spec 024, R1).
    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    pub fn with_idle_timeout(mut self, idle: Duration) -> Self {
        self.deadlines.idle = idle;
        self.agent = build_agent(self.connect_timeout, self.response_timeout, idle);
        self
    }

    /// Replaces the redirect exception list (defaults to
    /// [`DEFAULT_REDIRECT_EXCEPTIONS`]).
    #[must_use]
    pub fn with_redirect_exceptions<I, S>(mut self, hosts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.redirect_exceptions = hosts.into_iter().map(Into::into).collect();
        self
    }

    /// The directory completed downloads are stored in.
    #[must_use]
    pub fn downloads_dir(&self) -> &Path {
        &self.downloads_dir
    }

    /// Allows plain-HTTP URLs. Test-only: production is HTTPS-only.
    #[cfg(any(test, feature = "test-support"))]
    pub fn allow_http_for_tests(mut self) -> Self {
        self.allow_http_for_tests = true;
        self
    }

    /// The body deadline for `request`: size-scaled from the exact expected
    /// size when it is known, otherwise from the request's hard size cap.
    fn body_deadline_for(&self, request: &DownloadRequest) -> Duration {
        let size = request.expected_size.unwrap_or(request.max_size);
        body_deadline(self.deadlines, size)
    }

    /// Downloads `request`, returning the path of the verified cached file.
    ///
    /// `progress` is called at least once and only ever moves forward within a
    /// single transfer. `cancel` aborts between chunks and removes the
    /// temporary file.
    ///
    /// # Errors
    ///
    /// See [`Error`]. On failure nothing is left in the cache directory.
    pub fn download(
        &self,
        request: &DownloadRequest,
        cancel: &CancellationToken,
        progress: &mut dyn FnMut(Progress),
    ) -> Result<PathBuf, Error> {
        if request.urls.is_empty() {
            return Err(Error::InvalidUrl { url: String::new() });
        }

        let cache_path = request
            .expected_sha256
            .map(|hash| (self.downloads_dir.join(hash.to_hex()), hash));
        if let Some((path, expected)) = &cache_path
            && let Some(hit) = self.cached(path, request, *expected, progress)?
        {
            return Ok(hit);
        }

        let mut last: Option<Error> = None;
        for url in &request.urls {
            if cancel.is_cancelled() {
                return Err(Error::Cancelled);
            }
            match self.transfer(url, request, cancel, progress) {
                Ok(transfer) => {
                    if let Some(expected) = request.expected_sha256
                        && transfer.sha256 != expected
                    {
                        last = Some(Error::HashMismatch {
                            expected,
                            actual: transfer.sha256,
                        });
                        continue;
                    }
                    // A pinned transfer lands on its expected cache key; an
                    // unpinned document is stored by the hash of what arrived.
                    let destination = cache_path.as_ref().map_or_else(
                        || self.downloads_dir.join(transfer.sha256.to_hex()),
                        |(path, _)| path.clone(),
                    );
                    transfer.temp.persist(&destination)?;
                    return Ok(destination);
                }
                Err(error @ Error::Cancelled) => return Err(error),
                Err(error) => last = Some(error),
            }
        }

        Err(last.expect("urls is non-empty"))
    }

    /// Downloads a small document whose exact bytes are not known ahead of
    /// time and returns them (spec 019, R3: the catalog bundle).
    ///
    /// It runs through the same hardened path as [`Fetcher::download`] —
    /// HTTPS-only, host allowlist on every hop, redirect limit, size-scaled
    /// deadline and a hard size cap — but with no SHA-256 pin, so the caller
    /// must treat the bytes as untrusted input. The bytes are still stored in
    /// the content-addressed cache.
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub fn download_document(
        &self,
        url: impl Into<String>,
        max_size: u64,
        allowed_hosts: HostAllowlist,
        cancel: &CancellationToken,
    ) -> Result<Vec<u8>, Error> {
        let request = DownloadRequest::unpinned(url, max_size, allowed_hosts);
        let mut quiet = |_progress: Progress| {};
        let path = self.download(&request, cancel, &mut quiet)?;
        fs::read(&path).map_err(|error| io_error("read downloaded document", error))
    }

    /// Returns the cached path when it exists and re-hashes to `expected`;
    /// otherwise `None` (removing a stale file). Spec 006, R3.
    fn cached(
        &self,
        path: &Path,
        request: &DownloadRequest,
        expected: Sha256,
        progress: &mut dyn FnMut(Progress),
    ) -> Result<Option<PathBuf>, Error> {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(io_error("read cached download", error)),
        };
        if !metadata.is_file() {
            return Ok(None);
        }
        let size = metadata.len();
        if let Some(expected) = request.expected_size
            && size != expected
        {
            let _ = fs::remove_file(path);
            return Ok(None);
        }
        if hash_file(path)? == expected {
            progress(Progress {
                downloaded: size,
                total: Some(size),
            });
            Ok(Some(path.to_path_buf()))
        } else {
            let _ = fs::remove_file(path);
            Ok(None)
        }
    }

    /// Fetches one URL (following redirects) into a fresh temporary file.
    fn transfer(
        &self,
        url: &str,
        request: &DownloadRequest,
        cancel: &CancellationToken,
        progress: &mut dyn FnMut(Progress),
    ) -> Result<Transfer, Error> {
        let mut current = parse_uri(url)?;
        // Whether this chain began at an allowlisted `www.dropbox.com` URL.
        // Only such a chain may land on a Dropbox per-request download host
        // (spec 038, R1); the first URL is checked against the allowlist below,
        // so a lookalike origin never sets this.
        let dropbox_entry = host_of(&current)
            .is_some_and(|host| host.eq_ignore_ascii_case(DROPBOX_REDIRECT_ORIGIN));
        // One body deadline for the whole transfer, derived from the size we
        // expect (or the request's hard cap when it is unknown). Every redirect
        // hop gets the same budget; only one hop ever streams a body.
        let deadline = self.body_deadline_for(request);
        let mut redirects: u32 = 0;
        // The host of the URL that issued the current redirect; `None` for the
        // first URL, which no redirect exception may ever admit.
        let mut redirect_from: Option<String> = None;
        loop {
            let hop = Hop {
                from: redirect_from.as_deref(),
                dropbox_entry,
            };
            self.check_target(&current, &request.allowed_hosts, hop)?;
            // ureq 3.4.2 exposes `timeout_recv_body` per request through
            // `RequestBuilder::config()`; it is a *total* budget for the body,
            // not restarted per read. See
            // <https://docs.rs/ureq/3.4.2/ureq/config/struct.ConfigBuilder.html#method.timeout_recv_body>.
            let response = self
                .agent
                .get(current.clone())
                .config()
                .timeout_recv_body(Some(deadline))
                .build()
                .call()
                .map_err(|error| map_ureq(error, current.to_string()))?;
            let status = response.status();

            if status.is_redirection() {
                if redirects >= MAX_REDIRECTS {
                    return Err(Error::TooManyRedirects {
                        limit: MAX_REDIRECTS,
                        url: current.to_string(),
                    });
                }
                let location = response
                    .headers()
                    .get(LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .ok_or_else(|| Error::HttpStatus {
                        url: current.to_string(),
                        status: status.as_u16(),
                    })?;
                redirect_from = host_of(&current);
                current = resolve_location(&current, location)?;
                redirects += 1;
                continue;
            }

            if !status.is_success() {
                return Err(Error::HttpStatus {
                    url: current.to_string(),
                    status: status.as_u16(),
                });
            }

            return self.stream(response, request, cancel, progress, &current);
        }
    }

    /// Enforces scheme and host policy for one hop.
    ///
    /// `hop` describes where this hop sits in its redirect chain, which is what
    /// keeps a redirect exception redirect-only: a first URL (no `from`) never
    /// has one (spec 006 R1, spec 038 R1).
    fn check_target(&self, uri: &Uri, allowed: &HostAllowlist, hop: Hop<'_>) -> Result<(), Error> {
        let url = uri.to_string();
        let scheme = uri.scheme_str().unwrap_or_default();
        #[allow(unused_mut)]
        let mut scheme_ok = scheme == "https";
        #[cfg(any(test, feature = "test-support"))]
        {
            if self.allow_http_for_tests && scheme == "http" {
                scheme_ok = true;
            }
        }
        if !scheme_ok {
            return Err(Error::NotHttps { url });
        }

        let host = host_of(uri).ok_or_else(|| Error::InvalidUrl { url: url.clone() })?;
        if allowed.permits(&host) {
            return Ok(());
        }
        if let Some(from) = hop.from {
            // Documented redirect exceptions (spec 006, D-015): GitHub's signed
            // asset host, accepted from any origin.
            if self
                .redirect_exceptions
                .iter()
                .any(|exception| exception.eq_ignore_ascii_case(&host))
            {
                return Ok(());
            }
            // Dropbox's per-request download host (spec 038, R1) is narrower:
            // the chain must have begun at the allowlisted `www.dropbox.com`,
            // the identity is exact, and a per-request host may only hand off to
            // itself (Dropbox's own root-relative `/cd/0/inline` → `inline2`
            // hop), never to another label.
            if hop.dropbox_entry
                && is_dropbox_download_host(&host)
                && (from.eq_ignore_ascii_case(DROPBOX_REDIRECT_ORIGIN)
                    || from.eq_ignore_ascii_case(&host))
            {
                return Ok(());
            }
        }
        Err(Error::HostNotAllowed { host, url })
    }

    /// Streams a successful response into a temporary file, hashing as it goes.
    fn stream(
        &self,
        mut response: ureq::http::Response<ureq::Body>,
        request: &DownloadRequest,
        cancel: &CancellationToken,
        progress: &mut dyn FnMut(Progress),
        url: &Uri,
    ) -> Result<Transfer, Error> {
        let total = response.body().content_length().or(request.expected_size);
        let hard_limit = match request.expected_size {
            Some(expected) => expected.min(request.max_size),
            None => request.max_size,
        };

        let mut temp = TempFile::create(&self.downloads_dir)?;
        let mut hasher = Sha256Digest::new();
        let mut downloaded: u64 = 0;
        let mut buffer = vec![0u8; READ_BUFFER];
        // Wall clock for the error messages. The transport enforces both body
        // bounds; these only decide which one fired and how long the body ran.
        let started = Instant::now();
        let mut last_byte = started;

        progress(Progress {
            downloaded: 0,
            total,
        });

        let mut reader = response.body_mut().as_reader();
        loop {
            if cancel.is_cancelled() {
                return Err(Error::Cancelled);
            }
            // Both body bounds are enforced by the transport (spec 024, R1):
            // each read blocks for at most `min(remaining total deadline,
            // BODY_IDLE_TIMEOUT)`. A read that times out therefore means either
            // the idle gap or the total deadline fired; the gap since the last
            // byte says which, so the user gets `Stalled` for a silent
            // connection and `TooSlow` only when the whole budget is gone.
            // A cancel that raced the timer wins, as required by R3.
            let read = match reader.read(&mut buffer) {
                Ok(read) => read,
                Err(error) => {
                    if cancel.is_cancelled() {
                        return Err(Error::Cancelled);
                    }
                    return Err(if is_ureq_timeout(&error) {
                        if last_byte.elapsed() >= self.deadlines.idle {
                            Error::Stalled {
                                url: url.to_string(),
                                received: downloaded,
                                total,
                            }
                        } else {
                            Error::TooSlow {
                                url: url.to_string(),
                                downloaded,
                                total,
                                elapsed: started.elapsed(),
                            }
                        }
                    } else {
                        Error::Network {
                            url: url.to_string(),
                            detail: error.to_string(),
                        }
                    });
                }
            };
            if read == 0 {
                break;
            }
            last_byte = Instant::now();
            downloaded += read as u64;
            if downloaded > hard_limit {
                return Err(Error::SizeExceeded {
                    limit: hard_limit,
                    got: downloaded,
                });
            }
            hasher.update(&buffer[..read]);
            temp.file()
                .write_all(&buffer[..read])
                .map_err(|error| io_error("write downloaded data", error))?;
            progress(Progress { downloaded, total });
        }

        if let Some(expected) = request.expected_size
            && downloaded != expected
        {
            return Err(Error::SizeExceeded {
                limit: expected,
                got: downloaded,
            });
        }

        Ok(Transfer {
            temp,
            sha256: Sha256::finish(hasher),
        })
    }
}

/// Builds the ureq agent, capping every body read at `idle` (spec 024, R1).
///
/// ureq 3.4.2's `timeout_recv_body` is a *total* budget for the body, not a
/// per-read one (confirmed against ureq 3.4.2: `timings::CallTimings::
/// next_timeout` starts the `RecvBody` budget at the response headers and the
/// budget "is not restarted for each read"). A total budget cannot express an
/// idle timeout: it would abort a large archive that is still progressing. The
/// idle cap is therefore applied in the transport, where the socket read
/// timeout for each read is chosen: [`IdleCapTransport`] lowers ureq's computed
/// `RecvBody` timeout to at most `idle`, so a silent connection fails after
/// `idle` while a progressing one keeps resetting the gap. The size-scaled
/// total deadline still applies through ureq's own budget.
fn build_agent(connect_timeout: Duration, response_timeout: Duration, idle: Duration) -> Agent {
    let mut builder = Agent::config_builder()
        .user_agent(concat!("DoomDeckLauncher/", env!("CARGO_PKG_VERSION")))
        // The fetcher follows redirects itself so it can re-check every
        // target against the allowlist and enforce the redirect limit.
        .max_redirects(0)
        // Status codes are inspected and mapped explicitly.
        .http_status_as_error(false)
        // A binary download must be the server's bytes, not an encoding.
        .accept_encoding(AutoHeaderValue::None)
        .timeout_connect(Some(connect_timeout))
        // Header receipt is bounded here; the body gets an idle cap (transport)
        // and a size-scaled total deadline per request (see `transfer`).
        .timeout_recv_response(Some(response_timeout));
    // Production is HTTPS-only. Tests relax the transport to reach a
    // loopback HTTP server; the scheme policy is still exercised directly.
    #[cfg(not(any(test, feature = "test-support")))]
    {
        builder = builder.https_only(true);
    }
    #[cfg(any(test, feature = "test-support"))]
    {
        builder = builder.https_only(false);
    }
    let config = builder.build();
    // Keep ureq's default connector chain (SOCKS, TCP, TLS) exactly as
    // `Agent::new_with_config` would, and only wrap its final transport.
    let connector = DefaultConnector::default().chain(IdleCapConnector { idle });
    Agent::with_parts(config, connector, DefaultResolver::default())
}

/// A connector that wraps the default transport in [`IdleCapTransport`].
#[derive(Debug)]
struct IdleCapConnector {
    idle: Duration,
}

impl Connector<Box<dyn Transport>> for IdleCapConnector {
    type Out = IdleCapTransport<Box<dyn Transport>>;

    fn connect(
        &self,
        _details: &ConnectionDetails,
        chained: Option<Box<dyn Transport>>,
    ) -> Result<Option<Self::Out>, ureq::Error> {
        Ok(chained.map(|inner| IdleCapTransport {
            inner,
            idle: self.idle,
        }))
    }
}

/// A transport that lowers ureq's per-read `RecvBody` timeout to an idle cap.
///
/// ureq computes each body read's timeout as the remaining total budget; this
/// wrapper passes the smaller of that and the idle timeout to the inner
/// transport, which applies it to the socket. Every phase is delegated, so
/// pooling, TLS detection and connect/send/header timeouts are untouched.
#[derive(Debug)]
struct IdleCapTransport<T> {
    inner: T,
    idle: Duration,
}

impl<T: Transport> IdleCapTransport<T> {
    /// The timeout to hand the inner transport for one read.
    fn cap(&self, timeout: NextTimeout) -> NextTimeout {
        if timeout.reason != Timeout::RecvBody {
            return timeout;
        }
        let idle = ureq::unversioned::transport::time::Duration::from(self.idle);
        NextTimeout {
            after: timeout.after.min(idle),
            reason: timeout.reason,
        }
    }
}

impl<T: Transport> Transport for IdleCapTransport<T> {
    fn buffers(&mut self) -> &mut dyn Buffers {
        self.inner.buffers()
    }

    fn transmit_output(&mut self, amount: usize, timeout: NextTimeout) -> Result<(), ureq::Error> {
        self.inner.transmit_output(amount, timeout)
    }

    fn await_input(&mut self, timeout: NextTimeout) -> Result<bool, ureq::Error> {
        self.inner.await_input(self.cap(timeout))
    }

    fn is_open(&mut self) -> bool {
        self.inner.is_open()
    }

    fn is_tls(&self) -> bool {
        self.inner.is_tls()
    }
}

impl std::fmt::Debug for Fetcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fetcher")
            .field("downloads_dir", &self.downloads_dir)
            .field("redirect_exceptions", &self.redirect_exceptions)
            .finish_non_exhaustive()
    }
}

/// A verified transfer waiting to be moved into the cache.
struct Transfer {
    temp: TempFile,
    sha256: Sha256,
}

/// A temporary file that is removed on drop unless persisted.
struct TempFile {
    path: PathBuf,
    file: Option<File>,
    keep: bool,
}

impl TempFile {
    /// Creates a fresh temporary file in `dir`.
    fn create(dir: &Path) -> Result<Self, Error> {
        fs::create_dir_all(dir).map_err(|error| io_error("create downloads directory", error))?;
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let unique = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = dir.join(format!(".tmp-{}-{unique}", std::process::id()));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => {
                    return Ok(Self {
                        path,
                        file: Some(file),
                        keep: false,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(io_error("create temporary download file", error)),
            }
        }
        Err(Error::Io {
            context: "create temporary download file".to_owned(),
            detail: "could not find a unique file name".to_owned(),
        })
    }

    fn file(&mut self) -> &mut File {
        self.file.as_mut().expect("temporary file handle")
    }

    /// Flushes, fsyncs and atomically moves the file to `dest`.
    fn persist(mut self, dest: &Path) -> Result<(), Error> {
        let file = self.file.take().expect("temporary file handle");
        file.sync_all()
            .map_err(|error| io_error("flush downloaded data", error))?;
        drop(file);
        fs::rename(&self.path, dest).map_err(|error| io_error("store downloaded file", error))?;
        self.keep = true;
        Ok(())
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.keep {
            self.file.take();
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// Hashes an existing file in bounded chunks.
fn hash_file(path: &Path) -> Result<Sha256, Error> {
    let mut file = File::open(path).map_err(|error| io_error("open cached download", error))?;
    let mut hasher = Sha256Digest::new();
    let mut buffer = vec![0u8; READ_BUFFER];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| io_error("read cached download", error))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(Sha256::finish(hasher))
}

fn parse_uri(url: &str) -> Result<Uri, Error> {
    url.parse::<Uri>().map_err(|_| Error::InvalidUrl {
        url: url.to_owned(),
    })
}

fn host_of(uri: &Uri) -> Option<String> {
    uri.host().map(|host| {
        host.trim_start_matches('[')
            .trim_end_matches(']')
            .to_ascii_lowercase()
    })
}

/// Resolves a redirect `Location`, which may be absolute or root-relative.
fn resolve_location(base: &Uri, location: &str) -> Result<Uri, Error> {
    let location = location.trim();
    let resolved = if location.contains("://") {
        location.to_owned()
    } else if let Some(rest) = location.strip_prefix("//") {
        format!("{}://{rest}", base.scheme_str().unwrap_or("https"))
    } else if location.starts_with('/') {
        let authority = base.authority().map(|a| a.as_str()).unwrap_or_default();
        format!(
            "{}://{authority}{location}",
            base.scheme_str().unwrap_or("https")
        )
    } else {
        return Err(Error::InvalidUrl {
            url: location.to_owned(),
        });
    };
    resolved
        .parse::<Uri>()
        .map_err(|_| Error::InvalidUrl { url: resolved })
}

fn map_ureq(error: ureq::Error, url: String) -> Error {
    use ureq::Error as Ureq;
    match error {
        Ureq::StatusCode(status) => Error::HttpStatus { url, status },
        Ureq::BadUri(detail) => Error::InvalidUrl {
            url: format!("{url} ({detail})"),
        },
        // Connect/response-header timeouts: the body has its own deadline and
        // is reported as `Error::TooSlow` from the read loop.
        Ureq::Timeout(timeout) => Error::Network {
            url,
            detail: format!("request timed out ({timeout})"),
        },
        Ureq::HostNotFound => Error::Network {
            url,
            detail: "host not found".to_owned(),
        },
        Ureq::ConnectionFailed => Error::Network {
            url,
            detail: "connection failed".to_owned(),
        },
        Ureq::Io(error) => Error::Network {
            url,
            detail: error.to_string(),
        },
        Ureq::Tls(detail) => Error::Network {
            url,
            detail: format!("TLS error: {detail}"),
        },
        Ureq::Protocol(error) => Error::Network {
            url,
            detail: format!("HTTP protocol error: {error}"),
        },
        Ureq::TooManyRedirects => Error::TooManyRedirects {
            limit: MAX_REDIRECTS,
            url,
        },
        other => Error::Network {
            url,
            detail: other.to_string(),
        },
    }
}

fn io_error(context: &str, error: std::io::Error) -> Error {
    Error::Io {
        context: context.to_owned(),
        detail: error.to_string(),
    }
}

/// Whether an `io::Error` from ureq's body reader wraps
/// [`ureq::Error::Timeout`]. ureq's handler maps its typed error into
/// `std::io::Error` via `into_io` (`io::Error::other`), so the ureq error is
/// recoverable through `get_ref().downcast_ref()`. Confirmed against ureq
/// 3.4.2 (`run.rs`, `impl io::Read for BodyHandler`).
fn is_ureq_timeout(error: &std::io::Error) -> bool {
    error
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<ureq::Error>())
        .is_some_and(|error| matches!(error, ureq::Error::Timeout(_)))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    use super::*;
    use crate::downloads::test_http::{Response, Server, TestDir};

    /// Local servers are reachable through `127.0.0.1`, which is the host we
    /// allow in tests; the parameter keeps call sites explicit.
    fn allow(_servers: &[&Server]) -> HostAllowlist {
        HostAllowlist::new(["127.0.0.1"])
    }

    /// Builds the redirect context for `check_target`: `from` is the host that
    /// issued the redirect (`None` for a first URL) and `dropbox_entry` whether
    /// the chain began at `www.dropbox.com`.
    fn hop(from: Option<&str>, dropbox_entry: bool) -> Hop<'_> {
        Hop {
            from,
            dropbox_entry,
        }
    }

    #[test]
    fn happy_path_downloads_verifies_and_caches() {
        let body = b"the quick brown fox".to_vec();
        let body_len = body.len() as u64;
        let requests = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&requests);
        let served = body.clone();
        let server = Server::start(move |_req| {
            counter.fetch_add(1, Ordering::SeqCst);
            Response::ok(served.clone())
        });
        let dir = TestDir::new("happy");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();

        let hash = Sha256::of(&body);
        let request = DownloadRequest::new(server.url("/file"), hash, body_len, allow(&[&server]));

        let mut last_progress = 0;
        let mut progress = |p: Progress| {
            assert!(p.downloaded >= last_progress);
            last_progress = p.downloaded;
        };
        let cancel = CancellationToken::new();

        let path = fetcher
            .download(&request, &cancel, &mut progress)
            .expect("download succeeds");
        assert_eq!(path, dir.path().join("downloads").join(hash.to_hex()));
        assert_eq!(std::fs::read(&path).unwrap(), body);
        assert_eq!(last_progress, body_len);
        assert_eq!(requests.load(Ordering::SeqCst), 1);

        // A second download is served from the cache without a request.
        let mut quiet = |_p: Progress| {};
        let again = fetcher
            .download(&request, &cancel, &mut quiet)
            .expect("cache hit");
        assert_eq!(again, path);
        assert_eq!(requests.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cache_hit_is_rehashed_and_repaired() {
        let body = b"payload".to_vec();
        let served = body.clone();
        let server = Server::start(move |_req| Response::ok(served.clone()));
        let dir = TestDir::new("rehash");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let hash = Sha256::of(&body);
        let request = DownloadRequest::new(server.url("/file"), hash, 1024, allow(&[&server]));
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};

        let path = fetcher.download(&request, &cancel, &mut progress).unwrap();
        // Corrupt the cached file; the next call must detect it and re-fetch.
        std::fs::write(&path, b"tampered").unwrap();
        let repaired = fetcher.download(&request, &cancel, &mut progress).unwrap();
        assert_eq!(std::fs::read(&repaired).unwrap(), b"payload");
    }

    #[test]
    fn refuses_plain_http() {
        let server = Server::start(|_req| Response::ok(b"x".to_vec()));
        let dir = TestDir::new("http");
        // Deliberately NOT allow_http_for_tests: production behaviour.
        let fetcher = Fetcher::new(dir.path());
        let request = DownloadRequest::new(
            server.url("/file"),
            Sha256::of(b"x"),
            1024,
            allow(&[&server]),
        );
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        let error = fetcher
            .download(&request, &cancel, &mut progress)
            .expect_err("http refused");
        assert!(matches!(error, Error::NotHttps { .. }), "{error}");
    }

    #[test]
    fn enforces_the_redirect_limit() {
        let server = Server::start(|_req| Response::redirect("/loop"));
        let dir = TestDir::new("redirect-limit");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let request = DownloadRequest::new(
            server.url("/loop"),
            Sha256::of(b"whatever"),
            1024,
            allow(&[&server]),
        );
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        let error = fetcher
            .download(&request, &cancel, &mut progress)
            .expect_err("redirect loop fails");
        assert!(
            matches!(
                error,
                Error::TooManyRedirects {
                    limit: MAX_REDIRECTS,
                    ..
                }
            ),
            "{error}"
        );
    }

    #[test]
    fn rejects_a_redirect_to_a_non_allowlisted_host() {
        let server = Server::start(|_req| Response::redirect("https://evil.example/payload"));
        let dir = TestDir::new("redirect-host");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let request = DownloadRequest::new(
            server.url("/file"),
            Sha256::of(b"whatever"),
            1024,
            allow(&[&server]),
        );
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        let error = fetcher
            .download(&request, &cancel, &mut progress)
            .expect_err("non-allowlisted redirect fails");
        match error {
            Error::HostNotAllowed { host, .. } => assert_eq!(host, "evil.example"),
            other => panic!("expected HostNotAllowed, got {other}"),
        }
    }

    #[test]
    fn rejects_a_dropbox_lookalike_redirect_from_another_origin() {
        // The whole chain runs through `transfer`: a redirect from any host
        // other than `www.dropbox.com` must not admit a Dropbox-looking host,
        // and nothing is fetched from it (spec 038, R1).
        let server = Server::start(|_req| {
            Response::redirect(
                "https://uc3b1e38517d45e48a062e295c44.dl.dropboxusercontent.com/file?dl=1",
            )
        });
        let dir = TestDir::new("dropbox-other-origin");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let request = DownloadRequest::new(
            server.url("/start"),
            Sha256::of(b"whatever"),
            1024,
            allow(&[&server]),
        );
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        let error = fetcher
            .download(&request, &cancel, &mut progress)
            .expect_err("only www.dropbox.com may redirect to a Dropbox host");
        match error {
            Error::HostNotAllowed { host, .. } => {
                assert_eq!(
                    host,
                    "uc3b1e38517d45e48a062e295c44.dl.dropboxusercontent.com"
                );
            }
            other => panic!("expected HostNotAllowed, got {other}"),
        }
        assert_no_temp_files(dir.path());
    }

    #[test]
    fn follows_a_redirect_to_an_allowed_host() {
        let server = Server::start(|req| {
            if req.target == "/start" {
                Response::redirect("/final")
            } else {
                Response::ok(b"final body".to_vec())
            }
        });
        let dir = TestDir::new("redirect-ok");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let request = DownloadRequest::new(
            server.url("/start"),
            Sha256::of(b"final body"),
            1024,
            allow(&[&server]),
        );
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        let path = fetcher.download(&request, &cancel, &mut progress).unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"final body");
    }

    #[test]
    fn aborts_when_the_size_cap_is_exceeded() {
        let body = vec![7u8; 4096];
        let served = body.clone();
        let server = Server::start(move |_req| Response::ok(served.clone()));
        let dir = TestDir::new("too-big");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let request = DownloadRequest::new(
            server.url("/file"),
            Sha256::of(&body),
            1024,
            allow(&[&server]),
        );
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        let error = fetcher
            .download(&request, &cancel, &mut progress)
            .expect_err("size cap");
        assert!(
            matches!(error, Error::SizeExceeded { limit: 1024, .. }),
            "{error}"
        );
        assert_no_temp_files(dir.path());
    }

    #[test]
    fn aborts_when_the_expected_size_is_exceeded() {
        let body = vec![1u8; 4096];
        let served = body.clone();
        let server = Server::start(move |_req| Response::ok(served.clone()));
        let dir = TestDir::new("expected-size");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let request = DownloadRequest::new(
            server.url("/file"),
            Sha256::of(&body),
            u64::MAX,
            allow(&[&server]),
        )
        .with_size(16);
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        let error = fetcher
            .download(&request, &cancel, &mut progress)
            .expect_err("expected size");
        assert!(
            matches!(error, Error::SizeExceeded { limit: 16, .. }),
            "{error}"
        );
    }

    #[test]
    fn uses_the_mirror_after_a_hash_mismatch() {
        let good = b"genuine bytes".to_vec();
        let served = good.clone();
        let good_server = Server::start(move |_req| Response::ok(served.clone()));
        let bad_server = Server::start(|_req| Response::ok(b"corrupted".to_vec()));
        let dir = TestDir::new("mirror");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let request = DownloadRequest::new(
            bad_server.url("/file"),
            Sha256::of(&good),
            1024,
            allow(&[&bad_server, &good_server]),
        )
        .with_urls([bad_server.url("/file"), good_server.url("/file")]);
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        let path = fetcher.download(&request, &cancel, &mut progress).unwrap();
        assert_eq!(std::fs::read(path).unwrap(), good);
        assert_no_temp_files(dir.path());
    }

    #[test]
    fn does_not_use_a_mirror_after_success() {
        let good = b"genuine bytes".to_vec();
        let served = good.clone();
        let good_server = Server::start(move |_req| Response::ok(served.clone()));
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&hits);
        let bad_server = Server::start(move |_req| {
            counter.fetch_add(1, Ordering::SeqCst);
            Response::ok(b"corrupted".to_vec())
        });
        let dir = TestDir::new("no-mirror");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let request = DownloadRequest::new(
            good_server.url("/file"),
            Sha256::of(&good),
            1024,
            allow(&[&good_server, &bad_server]),
        )
        .with_urls([good_server.url("/file"), bad_server.url("/file")]);
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        fetcher.download(&request, &cancel, &mut progress).unwrap();
        assert_eq!(hits.load(Ordering::SeqCst), 0, "mirror must not be used");
    }

    #[test]
    fn progress_is_monotonic_and_cancel_removes_the_temp_file() {
        // A body far larger than one read, so there are several chunks.
        let body = vec![42u8; 512 * 1024];
        let body_len = body.len() as u64;
        let served = body.clone();
        let server = Server::start(move |_req| Response::ok(served.clone()));
        let dir = TestDir::new("cancel");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let request = DownloadRequest::new(
            server.url("/file"),
            Sha256::of(&body),
            body_len,
            allow(&[&server]),
        );
        let cancel = CancellationToken::new();
        let cancel_handle = cancel.clone();

        let mut seen = Vec::new();
        let mut progress = |p: Progress| {
            seen.push(p.downloaded);
            if p.downloaded > 0 {
                cancel_handle.cancel();
            }
        };
        let error = fetcher
            .download(&request, &cancel, &mut progress)
            .expect_err("cancelled");
        assert!(matches!(error, Error::Cancelled), "{error}");
        assert!(seen.windows(2).all(|w| w[1] >= w[0]), "{seen:?}");
        assert!(
            seen.last().copied().unwrap_or(0) < body_len,
            "cancel should fire mid-stream"
        );
        assert_no_temp_files(dir.path());
    }

    #[test]
    fn redirect_exceptions_apply_only_to_redirects() {
        let dir = TestDir::new("exceptions");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let allowed = HostAllowlist::new(["github.com"]);
        let target: Uri = "https://release-assets.githubusercontent.com/asset"
            .parse()
            .unwrap();

        // As an initial URL it is rejected...
        let error = fetcher
            .check_target(&target, &allowed, hop(None, false))
            .expect_err("initial URL");
        assert!(matches!(error, Error::HostNotAllowed { .. }), "{error}");
        // ...but as a redirect target the documented exception permits it.
        fetcher
            .check_target(&target, &allowed, hop(Some("github.com"), false))
            .expect("redirect exception");

        // A custom exception list replaces the default.
        let custom = Fetcher::new(dir.path())
            .allow_http_for_tests()
            .with_redirect_exceptions(["cdn.example"]);
        let custom_target: Uri = "https://cdn.example/x".parse().unwrap();
        custom
            .check_target(&custom_target, &allowed, hop(Some("github.com"), false))
            .expect("custom exception");
        assert!(
            custom
                .check_target(&target, &allowed, hop(Some("github.com"), false))
                .is_err(),
            "default exception must be gone"
        );
    }

    #[test]
    fn the_test_seam_allows_loopback_http() {
        let dir = TestDir::new("redirect-scheme");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let allowed = HostAllowlist::new(["127.0.0.1"]);
        let target: Uri = "http://127.0.0.1/x".parse().unwrap();
        // With the seam on, loopback HTTP is accepted; the production refusal
        // is covered by `refuses_plain_http`.
        fetcher
            .check_target(&target, &allowed, hop(None, false))
            .unwrap();
    }

    #[test]
    fn dropbox_chain_allows_the_per_request_host_and_its_same_host_hop() {
        let dir = TestDir::new("dropbox-allow");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        // A recipe's first URL is the allowlisted share link; the per-request
        // host is not (and cannot be) listed.
        let allowed = HostAllowlist::new(["www.dropbox.com"]);
        // The share link itself is allowlisted exactly.
        let share: Uri = "https://www.dropbox.com/scl/fi/x/DBP78.zip?dl=1"
            .parse()
            .unwrap();
        fetcher
            .check_target(&share, &allowed, hop(None, false))
            .expect("share link is allowlisted");
        // Hop 2: `www.dropbox.com` lands on the per-request host.
        let download: Uri =
            "https://uc3b1e38517d45e48a062e295c44.dl.dropboxusercontent.com/cd/0/inline/file?dl=1"
                .parse()
                .unwrap();
        fetcher
            .check_target(&download, &allowed, hop(Some("www.dropbox.com"), true))
            .expect("Dropbox per-request host is a permitted redirect target");
        // Hop 3: Dropbox redirects root-relative to `/cd/0/inline2/...` on the
        // same per-request host (observed with `curl -L`).
        let inline2: Uri =
            "https://uc3b1e38517d45e48a062e295c44.dl.dropboxusercontent.com/cd/0/inline2/file?dl=1"
                .parse()
                .unwrap();
        fetcher
            .check_target(
                &inline2,
                &allowed,
                hop(
                    Some("uc3b1e38517d45e48a062e295c44.dl.dropboxusercontent.com"),
                    true,
                ),
            )
            .expect("the same per-request host may redirect a second time");
    }

    #[test]
    fn dropbox_rule_never_applies_to_a_first_url() {
        let dir = TestDir::new("dropbox-first");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let allowed = HostAllowlist::new(["www.dropbox.com"]);
        let target: Uri = "https://ucXXXX.dl.dropboxusercontent.com/file?dl=1"
            .parse()
            .unwrap();
        // No redirect origin: refused, even with the rule in place.
        let error = fetcher
            .check_target(&target, &allowed, hop(None, false))
            .expect_err("first URL");
        match error {
            Error::HostNotAllowed { host, .. } => {
                assert_eq!(host, "ucxxxx.dl.dropboxusercontent.com");
            }
            other => panic!("expected HostNotAllowed, got {other}"),
        }
        // The bare suffix is refused too, as a first URL and as a target.
        let bare: Uri = "https://dl.dropboxusercontent.com/file".parse().unwrap();
        assert!(
            fetcher
                .check_target(&bare, &allowed, hop(None, false))
                .is_err()
        );
    }

    #[test]
    fn dropbox_chain_must_start_at_www_dropbox_com() {
        let dir = TestDir::new("dropbox-entry");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let allowed = HostAllowlist::new(["www.dropbox.com", "evil.example"]);
        let target: Uri = "https://ucXXXX.dl.dropboxusercontent.com/file"
            .parse()
            .unwrap();
        // A chain that did not begin at `www.dropbox.com` is refused, even
        // when the redirecting host is exactly the Dropbox share host.
        assert!(
            fetcher
                .check_target(&target, &allowed, hop(Some("www.dropbox.com"), false))
                .is_err(),
            "a chain that did not enter via www.dropbox.com must be refused"
        );
    }

    #[test]
    fn dropbox_rule_refuses_a_lookalike_or_wrong_origin() {
        let dir = TestDir::new("dropbox-refuse");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let allowed = HostAllowlist::new(["www.dropbox.com", "evil.example"]);

        // Exactly one label: a deeper subdomain is refused.
        let deep: Uri = "https://a.b.dl.dropboxusercontent.com/file"
            .parse()
            .unwrap();
        assert!(
            fetcher
                .check_target(&deep, &allowed, hop(Some("www.dropbox.com"), true))
                .is_err(),
            "two labels must be refused"
        );

        // A lookalike suffix is refused.
        let lookalike: Uri = "https://dl.dropboxusercontent.com.evil.example/file"
            .parse()
            .unwrap();
        assert!(
            fetcher
                .check_target(&lookalike, &allowed, hop(Some("www.dropbox.com"), true))
                .is_err(),
            "lookalike suffix must be refused"
        );

        // The rule belongs to a chain from `www.dropbox.com` alone: any other
        // origin, even an allowlisted one, cannot redirect there.
        let target: Uri = "https://ucXXXX.dl.dropboxusercontent.com/file"
            .parse()
            .unwrap();
        for origin in [
            "evil.example",
            "notwww.dropbox.com",
            "www.dropbox.com.evil.example",
            "dl.dropboxusercontent.com",
        ] {
            assert!(
                fetcher
                    .check_target(&target, &allowed, hop(Some(origin), true))
                    .is_err(),
                "origin {origin} must not permit the Dropbox download host"
            );
        }

        // One per-request host may not hand off to a *different* one.
        let other_label: Uri = "https://ucYYYY.dl.dropboxusercontent.com/file"
            .parse()
            .unwrap();
        assert!(
            fetcher
                .check_target(
                    &other_label,
                    &allowed,
                    hop(Some("ucXXXX.dl.dropboxusercontent.com"), true),
                )
                .is_err(),
            "a per-request host may redirect only to itself"
        );

        // The Dropbox rule admits Dropbox hosts only, not arbitrary ones.
        let other: Uri = "https://attacker.example/x".parse().unwrap();
        assert!(
            fetcher
                .check_target(&other, &allowed, hop(Some(DROPBOX_REDIRECT_ORIGIN), true))
                .is_err(),
            "the Dropbox origin cannot admit arbitrary hosts"
        );
    }

    #[test]
    fn dropbox_rule_is_inert_once_the_host_is_allowlisted_exactly() {
        // Exact matching still wins and needs no exception: if an operator ever
        // lists the host, it is permitted as a first URL too.
        let dir = TestDir::new("dropbox-exact");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let allowed = HostAllowlist::new(["ucXXXX.dl.dropboxusercontent.com"]);
        let target: Uri = "https://ucXXXX.dl.dropboxusercontent.com/file"
            .parse()
            .unwrap();
        fetcher
            .check_target(&target, &allowed, hop(None, false))
            .unwrap();
    }

    #[test]
    fn body_deadline_is_base_plus_size_over_min_rate() {
        let policy = DeadlinePolicy::default();
        // Even an empty body gets the base.
        assert_eq!(body_deadline(policy, 0), BODY_DEADLINE_BASE);
        // Eviternity II's 121 MiB archive: 60 s + 121 MiB / 64 KiB/s ≈ 33 min.
        let eviternity = 121 * 1024 * 1024;
        assert_eq!(
            body_deadline(policy, eviternity),
            BODY_DEADLINE_BASE + Duration::from_secs(eviternity / MIN_SUSTAINED_RATE)
        );
        // The 512 MiB cap is about 2.3 h, still under the 6 h ceiling.
        let cap = 512 * 1024 * 1024;
        assert!(body_deadline(policy, cap) < MAX_BODY_DEADLINE);
        assert_eq!(
            body_deadline(policy, cap),
            BODY_DEADLINE_BASE + Duration::from_secs(cap / MIN_SUSTAINED_RATE)
        );
    }

    #[test]
    fn body_deadline_clamps_to_the_maximum() {
        let policy = DeadlinePolicy::default();
        // A gigantic (or hostile) size cannot buy an unbounded wait.
        assert_eq!(body_deadline(policy, u64::MAX), MAX_BODY_DEADLINE);
        // A 1 TiB cap would otherwise ask for ~194 days.
        let one_tib = 1024 * 1024 * 1024 * 1024;
        assert_eq!(body_deadline(policy, one_tib), MAX_BODY_DEADLINE);
    }

    #[test]
    fn body_deadline_for_prefers_expected_size_over_the_cap() {
        let dir = TestDir::new("deadline-size");
        let fetcher = Fetcher::new(dir.path())
            .allow_http_for_tests()
            .with_deadline_policy(Duration::from_secs(1), 1000, Duration::from_secs(10));
        let hosts = HostAllowlist::new(["127.0.0.1"]);
        let hash = Sha256::of(b"x");

        // expected_size wins, even though max_size is much larger.
        let with_size =
            DownloadRequest::new("https://h/file", hash, 1_000_000, hosts.clone()).with_size(5000);
        assert_eq!(
            fetcher.body_deadline_for(&with_size),
            Duration::from_secs(6)
        );

        // Unknown size falls back to the request's hard cap.
        let unknown = DownloadRequest::new("https://h/file", hash, 5000, hosts.clone());
        assert_eq!(fetcher.body_deadline_for(&unknown), Duration::from_secs(6));

        // ...and the cap is clamped.
        let clamped = DownloadRequest::new("https://h/file", hash, 20_000, hosts);
        assert_eq!(fetcher.body_deadline_for(&clamped), Duration::from_secs(10));
    }

    #[test]
    fn slow_but_steady_transfer_survives_a_deadline_a_fixed_one_would_fail() {
        // 64 KiB sent as four 16 KiB pieces, 150 ms apart: the whole body takes
        // ~600 ms, far longer than a fixed 50 ms deadline but well inside the
        // size-scaled one.
        let body = vec![0xAB; 64 * 1024];
        let body_len = body.len() as u64;
        let served = body.clone();
        let server = Server::start(move |_req| {
            Response::ok(served.clone()).trickle(16 * 1024, Duration::from_millis(150))
        });
        let dir = TestDir::new("slow-steady");
        let hash = Sha256::of(&body);
        let request = DownloadRequest::new(server.url("/file"), hash, body_len, allow(&[&server]));
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};

        // A near-zero scaling term makes the deadline ≈ the base (50 ms), which
        // this transfer cannot meet: it fails, proving the deadline is real.
        let fixed_short = Fetcher::new(dir.path())
            .allow_http_for_tests()
            .with_deadline_policy(Duration::from_millis(50), 1 << 40, Duration::from_secs(60));
        let error = fixed_short
            .download(&request, &cancel, &mut progress)
            .expect_err("fixed short deadline fails");
        assert!(matches!(error, Error::TooSlow { .. }), "{error}");

        // The same transfer succeeds when the deadline scales with its size:
        // 50 ms + 64 KiB / 64 KiB/s = 1050 ms > 600 ms.
        let scaled = Fetcher::new(dir.path())
            .allow_http_for_tests()
            .with_deadline_policy(
                Duration::from_millis(50),
                MIN_SUSTAINED_RATE,
                Duration::from_secs(60),
            );
        let path = scaled
            .download(&request, &cancel, &mut progress)
            .expect("size-scaled deadline allows slow but steady");
        assert_eq!(std::fs::read(path).unwrap(), body);
    }

    #[test]
    fn a_transfer_that_outlasts_the_total_deadline_reports_too_slow() {
        // Headers arrive, then the server says nothing. With the idle timeout at
        // its production default, the shrunk size-scaled total deadline is the
        // first bound to fire, so the error is `TooSlow` (spec 024: the total
        // deadline stays as the overall bound).
        let body = vec![9u8; 32 * 1024];
        let served = body.clone();
        let server = Server::start(move |_req| {
            Response::ok(served.clone()).trickle(32 * 1024, Duration::from_secs(10))
        });
        let dir = TestDir::new("stalled");
        let hash = Sha256::of(&body);
        let request = DownloadRequest::new(
            server.url("/file"),
            hash,
            body.len() as u64,
            allow(&[&server]),
        );
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        let fetcher = Fetcher::new(dir.path())
            .allow_http_for_tests()
            .with_deadline_policy(Duration::from_millis(100), 1 << 40, Duration::from_secs(60));

        let error = fetcher
            .download(&request, &cancel, &mut progress)
            .expect_err("stalled transfer is aborted");
        match error {
            Error::TooSlow {
                downloaded, total, ..
            } => {
                assert_eq!(downloaded, 0, "no body bytes arrived");
                assert_eq!(total, Some(body.len() as u64));
            }
            other => panic!("expected TooSlow, got {other}"),
        }
        assert_no_temp_files(dir.path());
    }

    #[test]
    fn an_idle_transfer_is_aborted_with_the_stalled_error() {
        // The server sends one chunk immediately and then goes quiet for far
        // longer than the idle timeout. The total deadline is untouched, so a
        // `TooSlow` would only fire after the whole (60 s) budget; the idle cap
        // must abort first with a distinct error (spec 024, R1/R2).
        let body = vec![9u8; 32 * 1024];
        let served = body.clone();
        let server = Server::start(move |_req| {
            Response::ok(served.clone()).stall_after(16 * 1024, Duration::from_secs(5))
        });
        let dir = TestDir::new("idle");
        let hash = Sha256::of(&body);
        let request = DownloadRequest::new(
            server.url("/file"),
            hash,
            body.len() as u64,
            allow(&[&server]),
        );
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        let fetcher = Fetcher::new(dir.path())
            .allow_http_for_tests()
            .with_idle_timeout(Duration::from_millis(250));

        let error = fetcher
            .download(&request, &cancel, &mut progress)
            .expect_err("idle transfer is aborted");
        match error {
            Error::Stalled {
                received, total, ..
            } => {
                assert_eq!(received, 16 * 1024, "the first chunk did arrive");
                assert_eq!(total, Some(body.len() as u64));
            }
            other => panic!("expected Stalled, got {other}"),
        }
        assert_no_temp_files(dir.path());
    }

    #[test]
    fn a_slow_but_steady_transfer_below_the_idle_gap_succeeds() {
        // 64 KiB in four 16 KiB pieces 150 ms apart: each gap is below the 400 ms
        // idle timeout, so the idle cap never fires and the size-scaled total
        // deadline lets it finish (spec 024, R1).
        let body = vec![0xAB; 64 * 1024];
        let body_len = body.len() as u64;
        let served = body.clone();
        let server = Server::start(move |_req| {
            Response::ok(served.clone()).trickle(16 * 1024, Duration::from_millis(150))
        });
        let dir = TestDir::new("idle-steady");
        let request = DownloadRequest::new(
            server.url("/file"),
            Sha256::of(&body),
            body_len,
            allow(&[&server]),
        );
        let cancel = CancellationToken::new();
        let mut progress = |_p: Progress| {};
        let fetcher = Fetcher::new(dir.path())
            .allow_http_for_tests()
            .with_idle_timeout(Duration::from_millis(400));

        let path = fetcher
            .download(&request, &cancel, &mut progress)
            .expect("a steady link below the idle gap completes");
        assert_eq!(std::fs::read(path).unwrap(), body);
        assert_no_temp_files(dir.path());
    }

    #[test]
    fn cancel_during_a_stall_reports_cancelled() {
        // The server delivers a chunk then stalls; a cancel lands while the read
        // is blocked. Cancel must win over the idle timer (spec 024, R3): the
        // read times out on the idle cap, but the error is `Cancelled`.
        let body = vec![7u8; 32 * 1024];
        let served = body.clone();
        let server = Server::start(move |_req| {
            Response::ok(served.clone()).stall_after(16 * 1024, Duration::from_secs(5))
        });
        let dir = TestDir::new("idle-cancel");
        let request = DownloadRequest::new(
            server.url("/file"),
            Sha256::of(&body),
            body.len() as u64,
            allow(&[&server]),
        );
        let cancel = CancellationToken::new();
        let handle = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            handle.cancel();
        });
        let mut progress = |_p: Progress| {};
        let fetcher = Fetcher::new(dir.path())
            .allow_http_for_tests()
            .with_idle_timeout(Duration::from_millis(300));

        let error = fetcher
            .download(&request, &cancel, &mut progress)
            .expect_err("cancelled during a stall");
        assert!(matches!(error, Error::Cancelled), "{error}");
        assert_no_temp_files(dir.path());
    }

    #[test]
    fn unpinned_document_is_fetched_without_a_hash_pin() {
        let body = b"bundle bytes".to_vec();
        let requests = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&requests);
        let served = body.clone();
        let server = Server::start(move |_req| {
            counter.fetch_add(1, Ordering::SeqCst);
            Response::ok(served.clone())
        });
        let dir = TestDir::new("document");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let cancel = CancellationToken::new();

        let bytes = fetcher
            .download_document(
                server.url("/catalog.json"),
                4096,
                allow(&[&server]),
                &cancel,
            )
            .expect("document download");
        assert_eq!(bytes, body);

        // Without a pin there is no cache key to look up, so a second fetch
        // reaches the server again rather than reusing a verified file.
        let again = fetcher
            .download_document(
                server.url("/catalog.json"),
                4096,
                allow(&[&server]),
                &cancel,
            )
            .expect("second document download");
        assert_eq!(again, body);
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        assert_no_temp_files(dir.path());
    }

    #[test]
    fn unpinned_document_respects_the_size_cap() {
        let body = vec![3u8; 4096];
        let served = body.clone();
        let server = Server::start(move |_req| Response::ok(served.clone()));
        let dir = TestDir::new("document-cap");
        let fetcher = Fetcher::new(dir.path()).allow_http_for_tests();
        let cancel = CancellationToken::new();
        let error = fetcher
            .download_document(
                server.url("/catalog.json"),
                1024,
                allow(&[&server]),
                &cancel,
            )
            .expect_err("size cap");
        assert!(
            matches!(error, Error::SizeExceeded { limit: 1024, .. }),
            "{error}"
        );
        assert_no_temp_files(dir.path());
    }

    fn assert_no_temp_files(root: &Path) {
        let downloads = root.join("downloads");
        if let Ok(entries) = std::fs::read_dir(&downloads) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                assert!(
                    !name.to_string_lossy().starts_with(".tmp-"),
                    "leftover temp file: {name:?}"
                );
            }
        }
    }
}

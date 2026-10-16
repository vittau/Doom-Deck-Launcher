//! DSDA-Doom release discovery and its offline cache (spec 004, R2/R3).
//!
//! One blocking, bounded HTTPS request reads the upstream GitHub releases list
//! (`GET /repos/dsda-org/dsda-doom/releases?per_page=10`). The response is
//! cached with its weak `ETag` and a `fetched_at` timestamp so DDL works fully
//! offline: [`refresh`] serves the cache for 24 h, sends `If-None-Match` on an
//! explicit refresh, and treats a 403/429 or a network error as "offline with
//! cached data" instead of a hard failure. It never retries in a loop.
//!
//! Each release is classified ([`Availability`]) against the reviewed list
//! ([`super::reviewed`]): only [`Availability::Reviewed`] releases are offered
//! by default; [`Availability::Unreviewed`] ones need an explicit opt-in; ones
//! without a Linux AppImage are [`Availability::Unsupported`].
//!
//! The transport sits behind [`ReleaseApi`], so the caching, throttling and
//! classification logic is tested with recorded API fixtures and a
//! [`CannedApi`] double, while [`GithubApi`] is the real client. The real client
//! and the optional `GITHUB_TOKEN` follow `docs/research/upstream-sources.md`
//! §1.4.

use std::collections::VecDeque;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use ureq::Agent;

use super::clock::{Clock, format_unix_utc};
use super::reviewed::ReviewedList;
use crate::downloads::Sha256;

/// The reviewed-list repository's releases endpoint. The query string is fixed
/// so the `ETag` stays meaningful between polls (research §1.4).
pub const RELEASES_URL: &str =
    "https://api.github.com/repos/dsda-org/dsda-doom/releases?per_page=10";

/// Timeout for the discovery request (R2).
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// How long a cached response is considered fresh (R2: refresh at most once per
/// 24 h unless forced).
pub const THROTTLE: u64 = 24 * 60 * 60;

/// Subdirectory of DDL's cache directory holding the discovery cache.
pub const CACHE_DIR: &str = "engines";

/// File name of the discovery cache.
pub const CACHE_FILE: &str = "releases.json";

/// Schema version of the discovery cache document.
pub const CACHE_SCHEMA_VERSION: u32 = 1;

/// One raw release entry from the GitHub API, trimmed to the fields DDL uses.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GhRelease {
    /// Upstream git tag, e.g. `v0.30.1`.
    pub tag_name: String,
    /// Whether the release is a draft (skipped).
    #[serde(default)]
    pub draft: bool,
    /// Whether the release is a prerelease (skipped).
    #[serde(default)]
    pub prerelease: bool,
    /// Publish timestamp, informational only.
    #[serde(default)]
    pub published_at: Option<String>,
    /// The release's HTML page (attribution).
    #[serde(default)]
    pub html_url: Option<String>,
    /// Release assets.
    #[serde(default)]
    pub assets: Vec<GhAsset>,
}

/// One raw asset from the GitHub API.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GhAsset {
    /// Asset file name.
    pub name: String,
    /// Asset size in bytes.
    #[serde(default)]
    pub size: u64,
    /// Browser download URL.
    #[serde(default)]
    pub browser_download_url: String,
    /// GitHub's own SHA-256 digest (`sha256:<hex>`), present only on newer
    /// releases (research §1.3).
    #[serde(default)]
    pub digest: Option<String>,
}

/// How DDL relates a discovered release to its reviewed list (R3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    /// On the reviewed list; offered by default.
    Reviewed,
    /// Has a Linux AppImage but is not on the reviewed list; needs opt-in.
    Unreviewed,
    /// No Linux AppImage at all; cannot be installed on the Deck.
    Unsupported,
}

/// The Linux AppImage asset of a release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetInfo {
    /// Asset file name.
    pub name: String,
    /// Asset size in bytes.
    pub size: u64,
    /// Browser download URL.
    pub url: String,
    /// GitHub's digest, when present and SHA-256.
    pub digest: Option<Sha256>,
}

/// A discovered release, classified against the reviewed list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredRelease {
    /// Upstream git tag.
    pub tag: String,
    /// Version without the leading `v` (the install directory name).
    pub version: String,
    /// The release's HTML page, when the API provided it.
    pub release_url: Option<String>,
    /// Publish timestamp, when the API provided it.
    pub published_at: Option<String>,
    /// The Linux AppImage, or `None` for [`Availability::Unsupported`].
    pub asset: Option<AssetInfo>,
    /// Classification against the reviewed list.
    pub availability: Availability,
}

impl DiscoveredRelease {
    /// Whether this release can only be installed with an explicit opt-in.
    #[must_use]
    pub fn is_unreviewed(&self) -> bool {
        self.availability == Availability::Unreviewed
    }
}

/// The version part of a tag (`v0.30.1` → `0.30.1`).
#[must_use]
pub fn version_from_tag(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

/// Selects the Linux AppImage among `assets`, matching the case-insensitive
/// pattern `-linux.*\.appimage$` (R2). Upstream has used both
/// `-linux-x86_64.appimage` and `-Linux.appimage`, so the match is by pattern,
/// never by an exact name (research §1.2).
#[must_use]
pub fn select_linux_appimage(assets: &[GhAsset]) -> Option<&GhAsset> {
    assets.iter().find(|asset| {
        let name = asset.name.to_ascii_lowercase();
        name.contains("-linux") && name.ends_with(".appimage")
    })
}

/// Parses a GitHub `digest` value, accepting only SHA-256.
#[must_use]
pub fn parse_digest(digest: Option<&str>) -> Option<Sha256> {
    digest?.strip_prefix("sha256:").and_then(Sha256::parse_hex)
}

/// Classifies one API release, or `None` for a draft or prerelease.
#[must_use]
pub fn classify_release(release: &GhRelease, reviewed: &ReviewedList) -> Option<DiscoveredRelease> {
    if release.draft || release.prerelease {
        return None;
    }
    let version = version_from_tag(&release.tag_name).to_owned();
    let asset = select_linux_appimage(&release.assets);
    let availability = if asset.is_none() {
        Availability::Unsupported
    } else if reviewed.contains_tag(&release.tag_name) {
        Availability::Reviewed
    } else {
        Availability::Unreviewed
    };
    Some(DiscoveredRelease {
        version,
        tag: release.tag_name.clone(),
        release_url: release.html_url.clone(),
        published_at: release.published_at.clone(),
        asset: asset.map(|asset| AssetInfo {
            name: asset.name.clone(),
            size: asset.size,
            url: asset.browser_download_url.clone(),
            digest: parse_digest(asset.digest.as_deref()),
        }),
        availability,
    })
}

/// A response from the releases API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiResponse {
    /// HTTP status code.
    pub status: u16,
    /// The response `ETag`, when present.
    pub etag: Option<String>,
    /// The response body.
    pub body: String,
}

/// A transport-level failure (no HTTP response).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    /// A human-readable detail.
    pub detail: String,
}

impl ApiError {
    /// Builds an error from a detail message.
    #[must_use]
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

/// The releases transport seam (decision D-003).
pub trait ReleaseApi {
    /// Fetches the releases list, sending `If-None-Match` when a stored ETag is
    /// available.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError`] for a network, TLS or protocol failure. HTTP status
    /// codes are returned in [`ApiResponse::status`], not as errors.
    fn fetch(&self, if_none_match: Option<&str>) -> Result<ApiResponse, ApiError>;
}

/// The real GitHub releases client.
pub struct GithubApi {
    url: String,
    agent: Agent,
    token: Option<String>,
}

impl GithubApi {
    /// Creates a client for `url` with an optional bearer token.
    #[must_use]
    pub fn new(url: impl Into<String>, token: Option<String>) -> Self {
        let mut builder = Agent::config_builder()
            .user_agent(concat!("DoomDeckLauncher/", env!("CARGO_PKG_VERSION")))
            // Status codes are inspected explicitly.
            .http_status_as_error(false)
            // Return the server's exact bytes, not a re-encoded body.
            .accept_encoding(ureq::config::AutoHeaderValue::None)
            .timeout_global(Some(REQUEST_TIMEOUT));
        #[cfg(not(test))]
        {
            builder = builder.https_only(true);
        }
        #[cfg(test)]
        {
            // Tests reach a loopback HTTP server; the scheme policy itself is
            // covered by the production `https_only(true)` default.
            builder = builder.https_only(false);
        }
        Self {
            url: url.into(),
            agent: Agent::new_with_config(builder.build()),
            token,
        }
    }

    /// The official endpoint.
    #[must_use]
    pub fn official(token: Option<String>) -> Self {
        Self::new(RELEASES_URL, token)
    }

    /// Reads `GITHUB_TOKEN` from the environment. It is never required; when
    /// present it raises the rate limit and makes a `304` free (research §1.4).
    #[must_use]
    pub fn from_env() -> Self {
        Self::official(std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.is_empty()))
    }
}

impl ReleaseApi for GithubApi {
    fn fetch(&self, if_none_match: Option<&str>) -> Result<ApiResponse, ApiError> {
        let mut request = self
            .agent
            .get(&self.url)
            .header("Accept", "application/vnd.github+json");
        if let Some(etag) = if_none_match {
            request = request.header("If-None-Match", etag);
        }
        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {token}"));
        }
        let response = request
            .call()
            .map_err(|error| ApiError::new(error.to_string()))?;
        let status = response.status().as_u16();
        let etag = response
            .headers()
            .get("etag")
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        let mut response = response;
        let body = response
            .body_mut()
            .read_to_string()
            .map_err(|error| ApiError::new(error.to_string()))?;
        Ok(ApiResponse { status, etag, body })
    }
}

/// A test double for [`ReleaseApi`] that answers from a fixed queue and records
/// the `If-None-Match` values it was called with.
#[derive(Debug, Default)]
pub struct CannedApi {
    responses: Mutex<VecDeque<Result<ApiResponse, ApiError>>>,
    requests: Mutex<Vec<Option<String>>>,
}

impl CannedApi {
    /// An empty double.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues a response.
    pub fn push(&self, response: ApiResponse) {
        self.responses
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push_back(Ok(response));
    }

    /// Queues a transport error.
    pub fn push_error(&self, error: ApiError) {
        self.responses
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push_back(Err(error));
    }

    /// The `If-None-Match` values passed to [`ReleaseApi::fetch`], oldest first.
    #[must_use]
    pub fn requests(&self) -> Vec<Option<String>> {
        self.requests
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl ReleaseApi for CannedApi {
    fn fetch(&self, if_none_match: Option<&str>) -> Result<ApiResponse, ApiError> {
        self.requests
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(if_none_match.map(ToOwned::to_owned));
        self.responses
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pop_front()
            .unwrap_or_else(|| Err(ApiError::new("no canned response left")))
    }
}

/// The persisted discovery cache (R2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedReleases {
    /// Cache schema version.
    pub schema_version: u32,
    /// The weak `ETag` of the cached body.
    pub etag: Option<String>,
    /// When the body was last confirmed, in Unix seconds.
    pub fetched_at: u64,
    /// The raw releases JSON.
    pub body: String,
}

/// How a [`DiscoveryOutcome`] was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryStatus {
    /// A fresh 200 response.
    Fresh,
    /// A 304 response; the cache is unchanged.
    NotModified,
    /// The 24 h throttle served the cache without a request.
    Throttled,
    /// A 403/429 or network error; the cache was served.
    Offline,
}

/// The result of [`refresh`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryOutcome {
    /// Filtered and classified releases, newest first as the API lists them.
    pub releases: Vec<DiscoveredRelease>,
    /// When the served body was last confirmed, in Unix seconds.
    pub fetched_at: u64,
    /// How this outcome was produced.
    pub status: DiscoveryStatus,
}

impl DiscoveryOutcome {
    /// Whether the network failed and the cache was served instead.
    #[must_use]
    pub fn is_offline(&self) -> bool {
        self.status == DiscoveryStatus::Offline
    }

    /// Whether no request was made or the request failed.
    #[must_use]
    pub fn is_from_cache(&self) -> bool {
        matches!(
            self.status,
            DiscoveryStatus::Throttled | DiscoveryStatus::Offline
        )
    }

    /// The "offline (cached <time>)" note (R2), or `None` when online.
    #[must_use]
    pub fn offline_note(&self) -> Option<String> {
        self.is_offline()
            .then(|| format!("offline (cached {})", format_unix_utc(self.fetched_at)))
    }
}

/// Why discovery failed when no usable cache existed.
#[derive(Debug)]
pub enum DiscoveryError {
    /// The cache could not be read or written.
    Io(std::io::Error),
    /// The releases JSON could not be parsed.
    Parse(String),
    /// The transport failed and there was no cache to fall back to.
    Offline {
        /// A human-readable detail.
        detail: String,
    },
    /// The server answered `304` but no cache existed.
    NotModifiedWithoutCache,
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "release cache i/o error: {error}"),
            Self::Parse(detail) => write!(f, "could not parse the releases list: {detail}"),
            Self::Offline { detail } => {
                write!(
                    f,
                    "could not reach GitHub and no cached release list exists: {detail}"
                )
            }
            Self::NotModifiedWithoutCache => {
                f.write_str("GitHub answered 304 but DDL has no cached release list")
            }
        }
    }
}

impl std::error::Error for DiscoveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// Fetches the releases list, using or refreshing the cache (R2/R3).
///
/// `force` bypasses the 24 h throttle for an explicit "check for updates".
/// Without a usable cache a 403/429 or a network error is returned as
/// [`DiscoveryError::Offline`]; with a cache it is a [`DiscoveryStatus::Offline`]
/// outcome that keeps the cached list.
///
/// # Errors
///
/// See [`DiscoveryError`].
pub fn refresh(
    api: &dyn ReleaseApi,
    reviewed: &ReviewedList,
    cache_path: &Path,
    clock: &dyn Clock,
    force: bool,
) -> Result<DiscoveryOutcome, DiscoveryError> {
    let cache = load_cache(cache_path);
    let now = clock.now_unix_secs();

    if !force
        && let Some(cache) = &cache
        && now.saturating_sub(cache.fetched_at) < THROTTLE
    {
        return outcome_from_body(
            &cache.body,
            cache.fetched_at,
            DiscoveryStatus::Throttled,
            reviewed,
        );
    }

    let conditional = cache.as_ref().and_then(|cache| cache.etag.clone());
    match api.fetch(conditional.as_deref()) {
        Ok(response) => match response.status {
            200 => {
                let fresh = CachedReleases {
                    schema_version: CACHE_SCHEMA_VERSION,
                    etag: response.etag,
                    fetched_at: now,
                    body: response.body,
                };
                save_cache(cache_path, &fresh)?;
                outcome_from_body(&fresh.body, now, DiscoveryStatus::Fresh, reviewed)
            }
            304 => {
                let Some(mut cache) = cache else {
                    return Err(DiscoveryError::NotModifiedWithoutCache);
                };
                cache.fetched_at = now;
                save_cache(cache_path, &cache)?;
                outcome_from_body(&cache.body, now, DiscoveryStatus::NotModified, reviewed)
            }
            status => match cache {
                // 403/429 (or any other HTTP failure): keep the cache.
                Some(cache) => outcome_from_body(
                    &cache.body,
                    cache.fetched_at,
                    DiscoveryStatus::Offline,
                    reviewed,
                ),
                None => Err(DiscoveryError::Offline {
                    detail: format!("GitHub returned HTTP {status}"),
                }),
            },
        },
        Err(error) => match cache {
            Some(cache) => outcome_from_body(
                &cache.body,
                cache.fetched_at,
                DiscoveryStatus::Offline,
                reviewed,
            ),
            None => Err(DiscoveryError::Offline {
                detail: error.detail,
            }),
        },
    }
}

/// Parses a cached or freshly fetched body and classifies it.
fn outcome_from_body(
    body: &str,
    fetched_at: u64,
    status: DiscoveryStatus,
    reviewed: &ReviewedList,
) -> Result<DiscoveryOutcome, DiscoveryError> {
    let releases: Vec<GhRelease> =
        serde_json::from_str(body).map_err(|error| DiscoveryError::Parse(error.to_string()))?;
    let releases = releases
        .iter()
        .filter_map(|release| classify_release(release, reviewed))
        .collect();
    Ok(DiscoveryOutcome {
        releases,
        fetched_at,
        status,
    })
}

/// Reads the discovery cache, or `None` when it is absent or unreadable.
///
/// A corrupt or future-schema cache is ignored (and later overwritten) so a
/// damaged file never blocks an online refresh.
fn load_cache(path: &Path) -> Option<CachedReleases> {
    let text = fs::read_to_string(path).ok()?;
    let cache: CachedReleases = serde_json::from_str(&text).ok()?;
    (cache.schema_version == CACHE_SCHEMA_VERSION).then_some(cache)
}

/// Writes the discovery cache atomically (temp file + rename).
fn save_cache(path: &Path, cache: &CachedReleases) -> Result<(), DiscoveryError> {
    let json =
        serde_json::to_string(cache).map_err(|error| DiscoveryError::Parse(error.to_string()))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(DiscoveryError::Io)?;
    }
    let temporary = temporary_path(path);
    let write = || -> std::io::Result<()> {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        file.sync_all()
    };
    if let Err(error) = write() {
        let _ = fs::remove_file(&temporary);
        return Err(DiscoveryError::Io(error));
    }
    fs::rename(&temporary, path).map_err(DiscoveryError::Io)
}

/// A unique sibling temporary path, so concurrent writers never collide.
fn temporary_path(path: &Path) -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| CACHE_FILE.to_owned());
    path.with_file_name(format!(".{name}.tmp.{}.{unique}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::engines::dsda::reviewed::ReviewedList;

    /// A deterministic, advancing clock.
    struct TestClock(AtomicU64);

    impl Clock for TestClock {
        fn now_unix_secs(&self) -> u64 {
            self.0.load(Ordering::Relaxed)
        }
    }

    fn clock_at(secs: u64) -> TestClock {
        TestClock(AtomicU64::new(secs))
    }

    fn reviewed() -> ReviewedList {
        ReviewedList::embedded()
    }

    fn cache_path(dir: &Path) -> PathBuf {
        dir.join(CACHE_DIR).join(CACHE_FILE)
    }

    /// A trimmed fixture with three historical naming schemes plus a draft and
    /// a prerelease (research §1.1/§1.2).
    const FIXTURE: &str = include_str!("../../../tests/fixtures/engines/releases.json");

    fn ok(body: &str, etag: &str) -> ApiResponse {
        ApiResponse {
            status: 200,
            etag: Some(etag.to_owned()),
            body: body.to_owned(),
        }
    }

    #[test]
    fn classifies_all_three_historical_naming_schemes_and_skips_drafts() {
        let api = CannedApi::new();
        api.push(ok(FIXTURE, "W/\"one\""));
        let dir = tempfile::tempdir().unwrap();
        let outcome = refresh(
            &api,
            &reviewed(),
            &cache_path(dir.path()),
            &clock_at(1000),
            false,
        )
        .expect("refresh");

        assert_eq!(outcome.status, DiscoveryStatus::Fresh);
        // Draft and prerelease are skipped; v0.28.1 has no Linux asset.
        assert_eq!(outcome.releases.len(), 3, "{:?}", outcome.releases);

        let reviewed_release = outcome
            .releases
            .iter()
            .find(|r| r.tag == "v0.30.1")
            .unwrap();
        assert_eq!(reviewed_release.availability, Availability::Reviewed);
        let asset = reviewed_release.asset.as_ref().expect("asset");
        assert_eq!(asset.name, "dsda-doom-0.30.1-linux-x86_64.appimage");
        assert_eq!(asset.size, 14_608_888);
        assert!(asset.digest.is_some(), "v0.30.1 has a digest");

        // Capital-L, no architecture: the second historical scheme.
        let capital = outcome
            .releases
            .iter()
            .find(|r| r.tag == "v0.29.3")
            .unwrap();
        assert_eq!(capital.availability, Availability::Unreviewed);
        assert_eq!(
            capital.asset.as_ref().unwrap().name,
            "dsda-doom-0.29.3-Linux.appimage"
        );

        // No Linux AppImage: unsupported.
        let unsupported = outcome
            .releases
            .iter()
            .find(|r| r.tag == "v0.28.1")
            .unwrap();
        assert_eq!(unsupported.availability, Availability::Unsupported);
        assert!(unsupported.asset.is_none());
    }

    #[test]
    fn a_fresh_response_is_cached_and_reused_within_the_throttle() {
        let api = CannedApi::new();
        api.push(ok(FIXTURE, "W/\"one\""));
        let dir = tempfile::tempdir().unwrap();
        let path = cache_path(dir.path());
        let clock = clock_at(1000);

        refresh(&api, &reviewed(), &path, &clock, false).expect("first");
        assert!(path.exists());

        // A second non-forced refresh within 24 h makes no request.
        let second = refresh(&api, &reviewed(), &path, &clock, false).expect("throttled");
        assert_eq!(second.status, DiscoveryStatus::Throttled);
        assert!(second.is_from_cache());
        assert_eq!(api.requests().len(), 1);
    }

    #[test]
    fn a_forced_refresh_sends_if_none_match_and_handles_304() {
        let api = CannedApi::new();
        api.push(ok(FIXTURE, "W/\"one\""));
        api.push(ApiResponse {
            status: 304,
            etag: None,
            body: String::new(),
        });
        let dir = tempfile::tempdir().unwrap();
        let path = cache_path(dir.path());
        let clock = clock_at(1000);

        refresh(&api, &reviewed(), &path, &clock, false).expect("first");
        // More than 24 h later, force a conditional request.
        clock.0.store(1000 + THROTTLE + 1, Ordering::Relaxed);
        let second = refresh(&api, &reviewed(), &path, &clock, true).expect("304");

        assert_eq!(second.status, DiscoveryStatus::NotModified);
        assert_eq!(
            second.fetched_at,
            1000 + THROTTLE + 1,
            "304 refreshes fetched_at"
        );
        assert_eq!(second.releases.len(), 3);
        assert_eq!(
            api.requests(),
            vec![None, Some("W/\"one\"".to_owned())],
            "the stored ETag is sent on the conditional request"
        );
    }

    #[test]
    fn a_rate_limited_response_falls_back_to_the_cache() {
        for status in [403, 429] {
            let api = CannedApi::new();
            api.push(ok(FIXTURE, "W/\"one\""));
            api.push(ApiResponse {
                status,
                etag: None,
                body: String::new(),
            });
            let dir = tempfile::tempdir().unwrap();
            let path = cache_path(dir.path());
            let clock = clock_at(5000);

            refresh(&api, &reviewed(), &path, &clock, false).expect("first");
            clock.0.store(5000 + THROTTLE + 1, Ordering::Relaxed);
            let offline = refresh(&api, &reviewed(), &path, &clock, true).expect("offline");

            assert_eq!(offline.status, DiscoveryStatus::Offline);
            assert!(offline.is_offline());
            assert_eq!(offline.releases.len(), 3, "cached list is kept");
            assert_eq!(
                offline.offline_note(),
                Some(format!("offline (cached {})", format_unix_utc(5000)))
            );
        }
    }

    #[test]
    fn a_network_error_falls_back_to_the_cache() {
        let api = CannedApi::new();
        api.push(ok(FIXTURE, "W/\"one\""));
        let dir = tempfile::tempdir().unwrap();
        let path = cache_path(dir.path());
        let clock = clock_at(1);

        refresh(&api, &reviewed(), &path, &clock, false).expect("first");
        api.push_error(ApiError::new("connection refused"));
        clock.0.store(1 + THROTTLE + 1, Ordering::Relaxed);
        let offline = refresh(&api, &reviewed(), &path, &clock, true).expect("cached");

        assert!(offline.is_offline());
        assert_eq!(api.requests().len(), 2, "exactly one retry: never a loop");
    }

    #[test]
    fn an_offline_error_without_a_cache_is_typed() {
        let api = CannedApi::new();
        api.push(ApiResponse {
            status: 429,
            etag: None,
            body: String::new(),
        });
        let dir = tempfile::tempdir().unwrap();
        let error = refresh(
            &api,
            &reviewed(),
            &cache_path(dir.path()),
            &clock_at(1),
            false,
        )
        .expect_err("no cache");
        assert!(matches!(error, DiscoveryError::Offline { .. }));
    }

    #[test]
    fn a_corrupt_cache_is_ignored_and_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = cache_path(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "not json").unwrap();

        let api = CannedApi::new();
        api.push(ok(FIXTURE, "W/\"one\""));
        let outcome = refresh(&api, &reviewed(), &path, &clock_at(1), false).expect("refetch");
        assert_eq!(outcome.status, DiscoveryStatus::Fresh);
        assert_eq!(outcome.releases.len(), 3);
    }

    #[test]
    fn the_matching_rule_accepts_both_naming_schemes() {
        let assets = |name: &str| {
            vec![GhAsset {
                name: name.to_owned(),
                size: 1,
                browser_download_url: "https://example.invalid/x".to_owned(),
                digest: None,
            }]
        };
        assert!(select_linux_appimage(&assets("dsda-doom-0.30.1-linux-x86_64.appimage")).is_some());
        assert!(select_linux_appimage(&assets("dsda-doom-0.29.3-Linux.appimage")).is_some());
        assert!(select_linux_appimage(&assets("dsda-doom-0.30.1-win-x64.zip")).is_none());
    }

    #[test]
    fn digest_parsing_ignores_other_algorithms() {
        assert!(parse_digest(Some("sha256:00")).is_none(), "wrong length");
        assert!(parse_digest(Some("md5:abc")).is_none());
        assert!(parse_digest(None).is_none());
        let hex = "b06adf3f0c7d4e82dc1870baf97e67e4dc4e663e37f69070ddc02fa3e97608b0";
        assert_eq!(
            parse_digest(Some(&format!("sha256:{hex}"))),
            Sha256::parse_hex(hex)
        );
    }

    #[test]
    fn the_real_client_parses_status_etag_and_body_against_a_loopback_server() {
        use crate::downloads::test_http::{Response, Server};

        let server =
            Server::start(|_req| Response::ok(FIXTURE.to_owned()).header("ETag", "W/\"live\""));
        let api = GithubApi::new(server.url("/releases"), None);
        let response = api.fetch(None).expect("fetch");
        assert_eq!(response.status, 200);
        assert_eq!(response.etag.as_deref(), Some("W/\"live\""));
        assert!(response.body.contains("v0.30.1"));
    }
}

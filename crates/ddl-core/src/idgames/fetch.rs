//! Polite, bounded sidecar fetching (spec 050, R2/R6, D-031).
//!
//! The archive is a volunteer service: `docs/research/idgames-browse.md` §2.5
//! records that the master rate-limits bursts and asks clients to use a mirror.
//! So this module:
//!
//! - knows **only mirrors** ([`MIRRORS`]) and refuses the master and the two
//!   mirrors the research found unusable (Cloudflare 403, redirect to a host
//!   outside the allowlist);
//! - issues **one request at a time** — [`TxtSource::get`] is synchronous and
//!   the builder never runs two — with at least [`MIN_REQUEST_GAP`] between
//!   requests plus up to [`MAX_JITTER_MS`] of random jitter ([`SlowPacer`]);
//! - sends `DoomDeckLauncher/<version> (+repo URL)`;
//! - reads at most [`MAX_TXT_BYTES`] and treats 403/429/5xx as "stop the run"
//!   ([`TxtResponse::Blocked`]), which the builder turns into "keep what we
//!   have and resume tomorrow" (R6).
//!
//! The trait is the test seam: no test opens a socket.

use std::io::Read as _;
use std::time::Duration;

use ureq::Agent;
use ureq::config::AutoHeaderValue;

use super::txt::MAX_TXT_BYTES;

/// The gap between two sidecar requests (D-031: "at least 3 s").
pub const MIN_REQUEST_GAP: Duration = Duration::from_secs(3);

/// Extra random delay on top of [`MIN_REQUEST_GAP`], in milliseconds.
pub const MAX_JITTER_MS: u64 = 1_000;

/// The most sidecar requests one run may make (R6: 1,500 per day).
pub const DEFAULT_DAILY_BUDGET: u32 = 1_500;

/// Hard ceiling for `--budget`, so a typo cannot turn the crawl into a hammer.
pub const MAX_DAILY_BUDGET: u32 = 1_500;

/// A single request's total deadline.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Stop after this many consecutive transport failures: a mirror that is down
/// should not burn the whole budget in timeouts.
pub const MAX_CONSECUTIVE_FAILURES: u32 = 3;

/// The user agent DDL sends (D-031): version plus the repository URL.
pub const USER_AGENT: &str = concat!(
    "DoomDeckLauncher/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/vittau/Doom-Deck-Launcher)"
);

/// One `/idgames` mirror: host plus its base path.
///
/// Base paths are **not** uniform ([`README`](https://www.gamers.org/pub/idgames/README),
/// confirmed in the research §2.6): `ftp.fu-berlin.de` publishes
/// `/pc/games/idgames/`, the master and three mirrors use `/pub/idgames/`, and
/// `lethe`/`mtu` use `/idgames/`. Guessing `<host>/pub/idgames` gives 404s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mirror {
    /// DNS host, lowercase.
    pub host: &'static str,
    /// Path prefix without a trailing slash.
    pub base_path: &'static str,
}

/// The mirrors DDL is willing to crawl, in preference order.
///
/// Deliberately absent:
///
/// - `www.gamers.org` — the master; D-031 and its README ask clients to use a
///   mirror for bulk traffic;
/// - `files.xvertigox.com` — Cloudflare 403 for automated clients (research §2.6);
/// - `planetzdoom.com` — 302-redirects to `idgames.planetzdoom.com`, a host not
///   in `catalog/hosts.toml`, so a crawler would leave the allowlist.
pub const MIRRORS: [Mirror; 6] = [
    Mirror {
        host: "youfailit.net",
        base_path: "/pub/idgames",
    },
    Mirror {
        host: "ftp.fu-berlin.de",
        base_path: "/pc/games/idgames",
    },
    Mirror {
        host: "mirrors.lug.mtu.edu",
        base_path: "/idgames",
    },
    Mirror {
        host: "lethe.chinstrap.org",
        base_path: "/idgames",
    },
    Mirror {
        host: "ftpmirror.infania.net",
        base_path: "/pub/idgames",
    },
    Mirror {
        host: "mirror.braindrainlan.nu",
        base_path: "/pub/idgames",
    },
];

/// Finds a crawlable mirror by host, case-insensitively.
#[must_use]
pub fn mirror_for(host: &str) -> Option<&'static Mirror> {
    MIRRORS
        .iter()
        .find(|mirror| mirror.host.eq_ignore_ascii_case(host.trim()))
}

impl Mirror {
    /// The absolute base URL (`https://host/path`).
    #[must_use]
    pub fn base_url(self) -> String {
        format!("https://{}{}", self.host, self.base_path)
    }

    /// The absolute URL of an archive path (`levels/…/stem.txt`).
    ///
    /// Path segments are percent-encoded for anything outside the archive's
    /// documented alphabet, so a legacy name with a space or `#` cannot change
    /// which URL is requested.
    #[must_use]
    pub fn url(self, path: &str) -> String {
        format!("{}/{}", self.base_url(), encode_path(path))
    }
}

/// Percent-encodes everything outside `[A-Za-z0-9._~/-]`, keeping `/` as the
/// separator and never emitting a traversal segment.
fn encode_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.trim_start_matches('/').bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'~' | b'-' | b'/') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

/// One sidecar response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxtResponse {
    /// `200` with at most [`MAX_TXT_BYTES`] of body.
    Fetched {
        /// The raw body; decoding happens in [`super::txt`].
        bytes: Vec<u8>,
        /// The server's `ETag`, kept in the crawl state (R2).
        etag: Option<String>,
        /// The server's `Last-Modified`.
        last_modified: Option<String>,
    },
    /// `304`: the validators still describe the copy we already have.
    NotModified,
    /// `404`/`410`: no sidecar for this upload.
    Missing,
    /// `403`, `429` or `5xx`: stop the run, keep what was fetched (R6).
    Blocked {
        /// The HTTP status.
        status: u16,
    },
    /// A transport, TLS, timeout or other failure.
    Failed {
        /// Human-readable reason for the build log.
        detail: String,
    },
}

/// The injection seam for sidecar requests; production is [`HttpTxtSource`].
pub trait TxtSource {
    /// Fetches one sidecar, sending `etag`/`last_modified` as conditional
    /// validators when the crawl state has them.
    fn get(
        &mut self,
        url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> TxtResponse;
}

/// A single blocking HTTPS client with one connection at a time.
pub struct HttpTxtSource {
    agent: Agent,
}

impl HttpTxtSource {
    /// A client for the production crawl.
    #[must_use]
    pub fn new() -> Self {
        let mut builder = Agent::config_builder()
            .user_agent(USER_AGENT)
            // A sidecar must be the mirror's bytes, not an encoding.
            .accept_encoding(AutoHeaderValue::None)
            // Statuses are inspected explicitly.
            .http_status_as_error(false)
            .timeout_global(Some(REQUEST_TIMEOUT));
        #[cfg(not(any(test, feature = "test-support")))]
        {
            builder = builder.https_only(true);
        }
        #[cfg(any(test, feature = "test-support"))]
        {
            // Tests reach a loopback HTTP server; release builds never enable
            // the `test-support` feature.
            builder = builder.https_only(false);
        }
        Self {
            agent: Agent::new_with_config(builder.build()),
        }
    }
}

impl Default for HttpTxtSource {
    fn default() -> Self {
        Self::new()
    }
}

impl TxtSource for HttpTxtSource {
    fn get(
        &mut self,
        url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> TxtResponse {
        let mut request = self.agent.get(url);
        if let Some(etag) = etag {
            request = request.header("If-None-Match", etag);
        }
        if let Some(last_modified) = last_modified {
            request = request.header("If-Modified-Since", last_modified);
        }
        let mut response = match request.call() {
            Ok(response) => response,
            Err(error) => {
                return TxtResponse::Failed {
                    detail: error.to_string(),
                };
            }
        };
        let status = response.status().as_u16();
        let etag = header(&response, "etag");
        let last_modified = header(&response, "last-modified");
        match status {
            200 => {
                let mut bytes = Vec::new();
                let read = response
                    .body_mut()
                    .as_reader()
                    .take(MAX_TXT_BYTES + 1)
                    .read_to_end(&mut bytes);
                match read {
                    Ok(_) if bytes.len() as u64 <= MAX_TXT_BYTES => TxtResponse::Fetched {
                        bytes,
                        etag,
                        last_modified,
                    },
                    Ok(_) => TxtResponse::Failed {
                        detail: format!("sidecar exceeds {MAX_TXT_BYTES} bytes"),
                    },
                    Err(error) => TxtResponse::Failed {
                        detail: error.to_string(),
                    },
                }
            }
            304 => TxtResponse::NotModified,
            404 | 410 => TxtResponse::Missing,
            403 | 429 | 500..=599 => TxtResponse::Blocked { status },
            other => TxtResponse::Failed {
                detail: format!("HTTP {other}"),
            },
        }
    }
}

fn header(response: &ureq::http::Response<ureq::Body>, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

/// Waits between sidecar requests.
pub trait Pacer {
    /// Blocks for the polite delay before the next request.
    fn pause(&mut self);
}

/// The production pacer: 3 s plus a pseudo-random 0–1000 ms (D-031).
pub struct SlowPacer {
    gap: Duration,
    jitter_ms: u64,
    state: u64,
}

impl SlowPacer {
    /// A pacer with an explicit gap and jitter ceiling, for tests and callers
    /// that need a different pace.
    #[must_use]
    pub fn new(gap: Duration, jitter_ms: u64) -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0x9e37_79b9_7f4a_7c15, |elapsed| elapsed.as_nanos() as u64);
        Self {
            gap,
            jitter_ms,
            state: seed | 1,
        }
    }

    fn next_jitter(&mut self) -> u64 {
        // A 64-bit LCG (Knuth's constants): no `rand` dependency for 0–1 s of
        // jitter, and the sequence is not a fixed value a mirror could pin.
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        if self.jitter_ms == 0 {
            0
        } else {
            (self.state >> 33) % self.jitter_ms
        }
    }
}

impl Default for SlowPacer {
    fn default() -> Self {
        Self::new(MIN_REQUEST_GAP, MAX_JITTER_MS)
    }
}

impl Pacer for SlowPacer {
    fn pause(&mut self) {
        let jitter = Duration::from_millis(self.next_jitter());
        std::thread::sleep(self.gap + jitter);
    }
}

/// A pacer that never sleeps, for tests.
pub struct NoPacer;

impl Pacer for NoPacer {
    fn pause(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_mirrors_are_crawlable() {
        assert_eq!(mirror_for("youfailit.net").unwrap().base_path, "/pub/idgames");
        assert_eq!(
            mirror_for("FTP.FU-BERLIN.DE").unwrap().base_path,
            "/pc/games/idgames"
        );
        assert_eq!(
            mirror_for("mirrors.lug.mtu.edu").unwrap().base_url(),
            "https://mirrors.lug.mtu.edu/idgames"
        );
        assert!(mirror_for("www.gamers.org").is_none());
        assert!(mirror_for("files.xvertigox.com").is_none());
        assert!(mirror_for("planetzdoom.com").is_none());
        assert!(mirror_for("evil.example").is_none());
    }

    #[test]
    fn urls_join_the_archive_path_once() {
        let mirror = mirror_for("youfailit.net").unwrap();
        assert_eq!(
            mirror.url("levels/doom2/Ports/megawads/dbp37_augzen.txt"),
            "https://youfailit.net/pub/idgames/levels/doom2/Ports/megawads/dbp37_augzen.txt"
        );
        assert_eq!(
            mirror.url("/levels/doom/a-c/x.txt"),
            "https://youfailit.net/pub/idgames/levels/doom/a-c/x.txt"
        );
    }

    #[test]
    fn the_user_agent_names_the_project_and_its_repository() {
        assert!(USER_AGENT.starts_with("DoomDeckLauncher/"));
        assert!(USER_AGENT.contains("github.com/vittau/Doom-Deck-Launcher"));
    }

    #[test]
    fn the_budget_and_gap_match_d031() {
        assert_eq!(MIN_REQUEST_GAP, Duration::from_secs(3));
        assert_eq!(DEFAULT_DAILY_BUDGET, 1_500);
        assert!(MAX_DAILY_BUDGET <= 1_500);
    }

    #[test]
    fn jitter_stays_inside_the_ceiling() {
        let mut pacer = SlowPacer::new(Duration::from_secs(3), 1_000);
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..50 {
            let jitter = pacer.next_jitter();
            assert!(jitter < 1_000);
            seen.insert(jitter);
        }
        assert!(seen.len() > 1, "jitter must vary, not be a constant");
    }
}

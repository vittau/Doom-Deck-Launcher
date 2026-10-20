//! Header-only probes of a download URL (spec 044, R1).
//!
//! [`Fetcher::probe`] asks "is this file still there, and how big is it?"
//! without downloading it. It walks the same redirect chain as a download —
//! every hop passes the host allowlist and the redirect exceptions of specs
//! 006 and 038, at most [`MAX_REDIRECTS`] hops — but sends `HEAD`, or `GET`
//! with `Range: bytes=0-0` when the host refuses `HEAD`, and never reads a body.

use super::*;

/// The whole-request budget of one probe request (spec 044, R1), on top of the
/// connect and response-header timeouts every [`Fetcher`] has. Probes read no
/// body, so this only bounds a host that accepts the connection and then drips.
pub const PROBE_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// How a probe obtained its answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeMethod {
    /// `HEAD`.
    Head,
    /// `GET` with `Range: bytes=0-0`, used when the host refuses `HEAD` or
    /// does not report a size for it.
    RangeGet,
}

/// A successful probe: the URL answered with a 2xx status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeResponse {
    /// The URL that finally answered, after redirects.
    pub final_url: String,
    /// The HTTP status of the final answer (`200`, `206`, or `416` for an
    /// empty file asked for a range).
    pub status: u16,
    /// The size of the whole file: `Content-Length`, or the total of
    /// `Content-Range`. `None` when the server reported neither.
    pub size: Option<u64>,
    /// The request method that produced the answer.
    pub method: ProbeMethod,
    /// How many redirects were followed.
    pub redirects: u32,
}

/// Statuses that mean "this server does not answer `HEAD`" rather than "the
/// file is not there": method not allowed, not implemented, and the 400/403
/// that signed-URL front ends (S3 and friends) return for a method their
/// signature does not cover. A real 403 repeats on the range request and is
/// reported then.
const HEAD_REFUSED: [u16; 4] = [400, 403, 405, 501];

impl Fetcher {
    /// A fetcher for probes with the production connect and response-header
    /// timeouts (spec 044). It never touches the filesystem.
    #[must_use]
    pub(crate) fn for_probes() -> Self {
        Self::headers_only(CONNECT_TIMEOUT, RESPONSE_HEADERS_TIMEOUT)
    }

    /// Probes `url` for its size without downloading it (spec 044, R1).
    ///
    /// Sends `HEAD`; falls back to `GET` with `Range: bytes=0-0` when the host
    /// refuses `HEAD` or the `HEAD` answer carries no size. Redirects are
    /// followed under the same allowlist and redirect rules as a download.
    ///
    /// # Errors
    ///
    /// [`Error::HostNotAllowed`] when the URL or a redirect leaves `allowed`,
    /// [`Error::HttpStatus`] for a non-success final status (404 and 410 mean
    /// the file is gone), [`Error::Network`] for a connect, TLS or timeout
    /// failure, and the other redirect and URL errors of a download.
    pub fn probe(&self, url: &str, allowed: &HostAllowlist) -> Result<ProbeResponse, Error> {
        match self.probe_chain(url, allowed, ProbeMethod::Head) {
            Ok(head) if head.size.is_some() => Ok(head),
            // The host answered but did not say how big the file is: a range
            // request usually reveals it. Keep the `HEAD` answer when it does
            // not, because the file is evidently there.
            Ok(head) => match self.probe_chain(url, allowed, ProbeMethod::RangeGet) {
                Ok(range) if range.size.is_some() => Ok(range),
                _ => Ok(head),
            },
            Err(Error::HttpStatus { status, .. }) if HEAD_REFUSED.contains(&status) => {
                self.probe_chain(url, allowed, ProbeMethod::RangeGet)
            }
            Err(error) => Err(error),
        }
    }

    /// One redirect chain with one request method.
    fn probe_chain(
        &self,
        url: &str,
        allowed: &HostAllowlist,
        method: ProbeMethod,
    ) -> Result<ProbeResponse, Error> {
        let mut current = parse_uri(url)?;
        // Same rule as `attempt`: only a chain that began at the allowlisted
        // `www.dropbox.com` may land on a per-request download host.
        let dropbox_entry = host_of(&current)
            .is_some_and(|host| host.eq_ignore_ascii_case(DROPBOX_REDIRECT_ORIGIN));
        let mut redirects: u32 = 0;
        let mut redirect_from: Option<String> = None;
        loop {
            let hop = Hop {
                from: redirect_from.as_deref(),
                dropbox_entry,
            };
            self.check_target(&current, allowed, hop)?;
            let builder = match method {
                ProbeMethod::Head => self.agent.head(current.clone()),
                ProbeMethod::RangeGet => {
                    self.agent.get(current.clone()).header("Range", "bytes=0-0")
                }
            };
            let response = builder
                .config()
                .timeout_global(Some(PROBE_REQUEST_TIMEOUT))
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

            let header = |name: &str| {
                response
                    .headers()
                    .get(name)
                    .and_then(|value| value.to_str().ok())
            };
            let status = status.as_u16();
            let size = match (method, status) {
                (_, 206) => header("content-range")
                    .and_then(partial::parse_content_range)
                    .and_then(|range| range.total),
                // A range past the end of an empty file: the total follows
                // `bytes */`.
                (ProbeMethod::RangeGet, 416) => header("content-range")
                    .and_then(|value| value.trim().strip_prefix("bytes */"))
                    .and_then(|total| total.trim().parse().ok()),
                (_, 200..=299) => header("content-length").and_then(|v| v.trim().parse().ok()),
                _ => None,
            };
            let success = (200..300).contains(&status) || (status == 416 && size.is_some());
            if !success {
                return Err(Error::HttpStatus {
                    url: current.to_string(),
                    status,
                });
            }
            // The response (and, for a `GET` the server answered in full, its
            // unread body) is dropped here: the connection closes without
            // transferring the file.
            return Ok(ProbeResponse {
                final_url: current.to_string(),
                status,
                size,
                method,
                redirects,
            });
        }
    }
}

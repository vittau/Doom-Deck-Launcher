//! Catalog link health (spec 044, R1–R2).
//!
//! [`LinkChecker::check`] asks every source URL of every recipe — all mirrors,
//! not only the first — whether the file is still there and still the pinned
//! size, **without downloading it**: a `HEAD` (or a one-byte `Range` request
//! when the host refuses `HEAD`) through the downloader's own redirect and
//! allowlist rules ([`Fetcher::probe`]).
//!
//! # Outcomes and verdict
//!
//! Each URL ends as one [`Outcome`]. A *package* (one set of mirrors) is
//! **healthy** with at least one `ok` mirror, a **warning** when some mirrors
//! are bad but one is good, and **failed** with no `ok` mirror left. A recipe
//! fails when any of its packages failed, because it cannot be installed then.
//! [`Report::has_failures`] is what the CLI turns into its exit code.
//!
//! # Bounds
//!
//! At most [`MAX_IN_FLIGHT`] requests run at once and never two to the same
//! host (the scheduler keys on `host:port`), so a weekly run is polite to
//! small mirrors. Each request has the connect, header and whole-request
//! timeouts of [`Fetcher::probe`]. A distinct URL is probed once however many
//! recipes list it, and a transient failure (a `5xx`, a `429`, a timeout) is
//! asked again up to three times ([`RETRY_DELAYS`]) before it counts, because
//! some CDNs answer a bare `503` to about every other request.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write as _;
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::Duration;

use serde::Serialize;
use url::Url;

use crate::downloads::{Error, Fetcher, HostAllowlist, ProbeMethod, ProbeResponse};

use super::Recipe;

/// Most probes in flight at once (spec 044, R1).
pub const MAX_IN_FLIGHT: usize = 4;

/// What one source URL answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    /// Reachable, and the size matches the recipe (or the host reports none).
    Ok,
    /// Reachable, but the size differs from the recipe's pinned size.
    SizeChanged,
    /// `404` or `410`: the file is gone.
    Gone,
    /// The URL, or a redirect it issued, left the download allowlist.
    HostRefused,
    /// Any other status, a timeout or a network failure.
    Error,
}

impl Outcome {
    /// The label used in the table and the JSON (`ok`, `size-changed`, …).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::SizeChanged => "size-changed",
            Self::Gone => "gone",
            Self::HostRefused => "host-refused",
            Self::Error => "error",
        }
    }
}

/// The verdict for one source URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LinkResult {
    pub recipe: String,
    pub package: String,
    pub url: String,
    pub outcome: Outcome,
    /// A short explanation: the size seen, the status, the refused host.
    pub detail: String,
    /// The size the recipe pins for the package.
    pub expected_size: u64,
    /// The size the server reported, when it did.
    pub observed_size: Option<u64>,
}

/// A package (set of mirrors) with a problem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PackageFinding {
    pub recipe: String,
    pub package: String,
    /// How many mirrors answered `ok`.
    pub ok: usize,
    /// How many mirrors the package lists.
    pub total: usize,
}

/// Every verdict of one run, in recipe, package and mirror order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub results: Vec<LinkResult>,
    /// Packages with no `ok` mirror: their recipe cannot be installed.
    pub failed: Vec<PackageFinding>,
    /// Packages with some bad mirror and at least one `ok` one.
    pub warnings: Vec<PackageFinding>,
}

impl Report {
    fn from_results(results: Vec<LinkResult>) -> Self {
        let mut failed = Vec::new();
        let mut warnings = Vec::new();
        let mut index = 0;
        while index < results.len() {
            let first = &results[index];
            let group = results[index..]
                .iter()
                .take_while(|r| r.recipe == first.recipe && r.package == first.package)
                .count();
            let slice = &results[index..index + group];
            let ok = slice.iter().filter(|r| r.outcome == Outcome::Ok).count();
            let finding = PackageFinding {
                recipe: first.recipe.clone(),
                package: first.package.clone(),
                ok,
                total: group,
            };
            if ok == 0 {
                failed.push(finding);
            } else if ok < group {
                warnings.push(finding);
            }
            index += group;
        }
        Self {
            results,
            failed,
            warnings,
        }
    }

    /// Whether any package has no `ok` mirror left (spec 044, R2): the
    /// condition for a non-zero exit code.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }

    /// The ids of the recipes that cannot be installed, once each, in catalog
    /// order.
    #[must_use]
    pub fn failed_recipes(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.failed.iter().map(|f| f.recipe.as_str()).collect();
        ids.dedup();
        ids
    }

    /// The report as a Markdown summary line and table (spec 044, R2). With
    /// `problems_only`, only the rows that are not `ok` are listed.
    #[must_use]
    pub fn to_markdown(&self, problems_only: bool) -> String {
        let count = |outcome: Outcome| self.results.iter().filter(|r| r.outcome == outcome).count();
        let mut text = String::new();
        let _ = writeln!(
            text,
            "**{} URLs checked:** {} ok, {} size-changed, {} gone, {} host-refused, {} error. \
             **{} recipe(s) with no working source**, {} package(s) with a bad mirror.",
            self.results.len(),
            count(Outcome::Ok),
            count(Outcome::SizeChanged),
            count(Outcome::Gone),
            count(Outcome::HostRefused),
            count(Outcome::Error),
            self.failed_recipes().len(),
            self.warnings.len(),
        );
        text.push('\n');
        text.push_str("| Recipe | URL | Outcome | Detail |\n| --- | --- | --- | --- |\n");
        for result in &self.results {
            if problems_only && result.outcome == Outcome::Ok {
                continue;
            }
            let _ = writeln!(
                text,
                "| {} | {} | {} | {} |",
                cell(&result.recipe),
                cell(&result.url),
                result.outcome.label(),
                cell(&result.detail),
            );
        }
        text
    }

    /// The report as pretty JSON, for machines (spec 044, R2).
    ///
    /// # Panics
    ///
    /// Never in practice: the report holds only strings, numbers and enums.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self).expect("a report is plain data");
        json.push('\n');
        json
    }
}

/// A table cell: pipes escaped, newlines flattened.
fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\n', '\r'], " ")
}

/// Probes the source URLs of recipes (spec 044, R1).
pub struct LinkChecker {
    fetcher: Fetcher,
    max_in_flight: usize,
    /// Waits before each retry of a transient failure.
    retry_delays: Vec<Duration>,
}

impl Default for LinkChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Waits before each retry of a transient failure: raw.githubusercontent.com
/// answers a bare `503` to about every other request, so one failure proves
/// nothing. Four attempts in all.
pub const RETRY_DELAYS: [Duration; 3] = [
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(4),
];

/// One distinct URL waiting for a worker.
struct Job {
    index: usize,
    url: String,
    /// `host:port` of the URL; one job per key runs at a time.
    host: String,
}

#[derive(Default)]
struct Queue {
    pending: VecDeque<Job>,
    busy: HashSet<String>,
}

/// What a URL answered, kept apart from [`Error`] so one answer can be shared
/// by every recipe that lists the URL.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Answer {
    /// Reachable; the size when the host reported one.
    Reached {
        size: Option<u64>,
        via: &'static str,
    },
    HostRefused {
        host: String,
    },
    Status(u16),
    /// A failure with no status; `transient` when the connection, not the URL,
    /// is the likely culprit.
    Failed {
        detail: String,
        transient: bool,
    },
}

impl Answer {
    fn from_result(result: Result<ProbeResponse, Error>) -> Self {
        match result {
            Ok(response) => Self::Reached {
                size: response.size,
                via: match response.method {
                    ProbeMethod::Head => "HEAD",
                    ProbeMethod::RangeGet => "GET Range",
                },
            },
            Err(Error::HostNotAllowed { host, .. }) => Self::HostRefused { host },
            Err(Error::HttpStatus { status, .. }) => Self::Status(status),
            Err(error) => Self::Failed {
                transient: matches!(
                    error,
                    Error::Network { .. } | Error::TooSlow { .. } | Error::Stalled { .. }
                ),
                detail: error.to_string(),
            },
        }
    }

    /// Whether asking again could change the answer: a server-side error, a
    /// rate limit, or a connection that failed or timed out.
    fn is_transient(&self) -> bool {
        match self {
            Self::Status(status) => *status == 429 || *status >= 500,
            Self::Failed { transient, .. } => *transient,
            Self::Reached { .. } | Self::HostRefused { .. } => false,
        }
    }

    /// The outcome of this answer for a package pinned at `expected` bytes.
    fn judge(&self, expected: u64) -> (Outcome, String, Option<u64>) {
        match self {
            Self::Reached {
                size: Some(size),
                via,
            } if *size == expected => (Outcome::Ok, format!("{size} bytes ({via})"), Some(*size)),
            Self::Reached {
                size: Some(size),
                via,
            } => (
                Outcome::SizeChanged,
                format!("pinned {expected} bytes, server reports {size} ({via})"),
                Some(*size),
            ),
            // The file answered but the host states no size, so there is
            // nothing to compare: reachable is all that can be said.
            Self::Reached { size: None, via } => (
                Outcome::Ok,
                format!("reachable, size not reported ({via})"),
                None,
            ),
            Self::HostRefused { host } => (
                Outcome::HostRefused,
                format!("host `{host}` is not on the download allowlist"),
                None,
            ),
            Self::Status(status @ (404 | 410)) => (Outcome::Gone, format!("HTTP {status}"), None),
            Self::Status(status) => (Outcome::Error, format!("HTTP {status}"), None),
            Self::Failed { detail, .. } => (Outcome::Error, detail.clone(), None),
        }
    }
}

impl LinkChecker {
    /// A checker with the production timeouts, [`MAX_IN_FLIGHT`] and
    /// [`RETRY_DELAYS`].
    #[must_use]
    pub fn new() -> Self {
        Self::with_fetcher(Fetcher::for_probes())
    }

    pub(crate) fn with_fetcher(fetcher: Fetcher) -> Self {
        Self {
            fetcher,
            max_in_flight: MAX_IN_FLIGHT,
            retry_delays: RETRY_DELAYS.to_vec(),
        }
    }

    /// Replaces the waits between retries. Tests use short ones.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_retry_delays(mut self, delays: Vec<Duration>) -> Self {
        self.retry_delays = delays;
        self
    }

    /// Checks every source URL of `recipes` against `allowed`.
    ///
    /// Each distinct URL is probed once, however many recipes list it, and the
    /// answer is judged against every package's own pinned size. Results come
    /// back in recipe, package and mirror order whatever order the probes
    /// finished in.
    #[must_use]
    pub fn check(&self, recipes: &[&Recipe], allowed: &HostAllowlist) -> Report {
        let mut distinct: Vec<String> = Vec::new();
        let mut seen: HashSet<&str> = HashSet::new();
        for recipe in recipes {
            for package in &recipe.packages {
                for url in &package.urls {
                    if seen.insert(url.as_str()) {
                        distinct.push(url.clone());
                    }
                }
            }
        }
        let answers = self.probe_all(&distinct, allowed);
        let by_url: HashMap<&str, &Answer> = distinct
            .iter()
            .map(String::as_str)
            .zip(answers.iter())
            .collect();

        let mut results = Vec::new();
        for recipe in recipes {
            for package in &recipe.packages {
                for url in &package.urls {
                    let (outcome, detail, observed_size) = by_url[url.as_str()].judge(package.size);
                    results.push(LinkResult {
                        recipe: recipe.id.clone(),
                        package: package.id.clone(),
                        url: url.clone(),
                        outcome,
                        detail,
                        expected_size: package.size,
                        observed_size,
                    });
                }
            }
        }
        Report::from_results(results)
    }

    /// Probes each URL on a pool of at most `max_in_flight` threads, never two
    /// at once for one host. Answers are returned in `urls` order.
    fn probe_all(&self, urls: &[String], allowed: &HostAllowlist) -> Vec<Answer> {
        let pending: VecDeque<Job> = urls
            .iter()
            .enumerate()
            .map(|(index, url)| Job {
                index,
                url: url.clone(),
                host: host_key(url),
            })
            .collect();
        let total = pending.len();
        let queue = Mutex::new(Queue {
            pending,
            busy: HashSet::new(),
        });
        let wake = Condvar::new();
        let done: Mutex<Vec<(usize, Answer)>> = Mutex::new(Vec::with_capacity(total));

        std::thread::scope(|scope| {
            for _ in 0..self.max_in_flight.clamp(1, total.max(1)) {
                scope.spawn(|| {
                    while let Some(job) = next_job(&queue, &wake) {
                        let answer = self.probe_with_retries(&job.url, allowed);
                        done.lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .push((job.index, answer));
                        queue
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .busy
                            .remove(&job.host);
                        wake.notify_all();
                    }
                });
            }
        });

        let mut done = done.into_inner().unwrap_or_else(PoisonError::into_inner);
        done.sort_by_key(|(index, _)| *index);
        done.into_iter().map(|(_, answer)| answer).collect()
    }

    /// One URL, asked again after a transient failure. The host stays busy
    /// while it waits, so a struggling mirror is not hit harder.
    fn probe_with_retries(&self, url: &str, allowed: &HostAllowlist) -> Answer {
        let mut answer = Answer::from_result(self.fetcher.probe(url, allowed));
        for delay in &self.retry_delays {
            if !answer.is_transient() {
                break;
            }
            std::thread::sleep(*delay);
            answer = Answer::from_result(self.fetcher.probe(url, allowed));
        }
        answer
    }
}

/// Takes the first queued job whose host is idle and marks the host busy,
/// waiting while every queued job's host is busy. `None` when the queue is
/// empty.
fn next_job(queue: &Mutex<Queue>, wake: &Condvar) -> Option<Job> {
    let mut state = queue.lock().unwrap_or_else(PoisonError::into_inner);
    loop {
        if state.pending.is_empty() {
            return None;
        }
        let position = state
            .pending
            .iter()
            .position(|job| !state.busy.contains(&job.host));
        if let Some(position) = position {
            let job = state.pending.remove(position)?;
            state.busy.insert(job.host.clone());
            return Some(job);
        }
        state = wake.wait(state).unwrap_or_else(PoisonError::into_inner);
    }
}

/// The scheduling key of `url`: lowercase host and port. An unparsable URL
/// shares the empty key; its probe fails fast anyway.
fn host_key(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|url| {
            let host = url.host_str()?.to_ascii_lowercase();
            Some(match url.port_or_known_default() {
                Some(port) => format!("{host}:{port}"),
                None => host,
            })
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;

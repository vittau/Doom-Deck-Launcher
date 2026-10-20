//! The crawl's resumable state and telemetry, committed beside the index
//! (spec 050, R6/R9).
//!
//! The first full pass over about 10,900 sidecars is deliberately spread over
//! several days at 1,500 requests a day. That only works if tomorrow's run
//! knows which files it already tried today: [`CrawlState`] records the outcome
//! and the HTTP validators (`ETag`, `Last-Modified`) per archive path, so a run
//! that hit its budget — or was stopped by a `429` — picks up where it left
//! off instead of restarting.
//!
//! The same file carries the **crawl telemetry and back-off** the owner asked
//! for (R9, 2026-10-20): every run appends a [`RunTelemetry`] record (requests,
//! bytes, status codes, spacing, retries, stop reason) and [`Backoff`] tracks
//! the last mirror stop. A stopped run makes the next one wait at least 24 h
//! and halves its budget (floor 100) until three clean runs pass
//! ([`CrawlState::effective_budget`], [`CrawlState::record_run`]).
//!
//! The state is **not** part of `index.json`: the published index has exactly
//! the fields spec 050 R3 lists. `crawl-state.json` is a second small file on
//! the same branch.
//!
//! Nothing here is trusted when read back: the file is bounded, its schema is
//! checked, and an unreadable file simply starts a fresh state (the index is
//! still authoritative for everything already fetched).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use super::date::{Date, format_unix_utc, is_rfc3339_utc};

/// The crawl-state format this DDL understands.
pub const STATE_SCHEMA: u32 = 1;

/// Largest state document DDL accepts.
pub const MAX_STATE_BYTES: u64 = 8 * 1024 * 1024;

/// How many run records the state keeps (newest last).
pub const MAX_TELEMETRY_RUNS: usize = 30;

/// A stopped run makes the next one wait this long (R9).
pub const BACKOFF_WINDOW_SECS: u64 = 24 * 60 * 60;

/// The back-off never halves the budget below this (R9).
pub const BACKOFF_FLOOR: u32 = 100;

/// Clean runs needed to clear the back-off (R9).
pub const CLEAN_RUNS_TO_RECOVER: u32 = 3;

/// How one sidecar attempt ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttemptOutcome {
    /// The sidecar was read (or had not changed since the cached copy).
    Ok,
    /// There is no sidecar for this upload.
    Missing,
    /// The sidecar existed but was unusable, or the request failed.
    Bad,
}

/// One recorded attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    /// `YYYY-MM-DD` of the last attempt.
    pub date: String,
    /// What happened.
    pub outcome: AttemptOutcome,
    /// The server's `ETag`, when it sent one (R2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    /// The server's `Last-Modified`, when it sent one (R2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

/// What one run did, kept in the committed state and printed in CI (R9).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunTelemetry {
    /// When the run started, RFC 3339 UTC.
    pub started_at: String,
    /// When the run finished, RFC 3339 UTC.
    pub finished_at: String,
    /// The same instant as Unix seconds, for the 24 h back-off window.
    pub finished_unix: u64,
    /// The budget this run was allowed to use (after back-off).
    pub budget: u32,
    /// Sidecar requests made.
    pub requests: u32,
    /// Body bytes read from `200` responses.
    pub bytes: u64,
    /// Sidecars longer than the 64 KiB prefix; parsed from that prefix (F2).
    #[serde(default)]
    pub truncated: u32,
    /// Status label (`200`, `304`, `404`, `403`, `429`, `5xx`, `error`) → count.
    pub statuses: BTreeMap<String, u32>,
    /// Smallest gap between two request starts, in milliseconds.
    pub min_gap_ms: u64,
    /// Mean gap between two request starts, in milliseconds.
    pub mean_gap_ms: u64,
    /// Requests retried inside the run. DDL never retries: always 0 (R9).
    pub retries: u32,
    /// `finished`, `budget`, `http-<status>`, `network`, `backoff-wait` or
    /// `error` (the document failed its final validation).
    pub stop: String,
}

impl RunTelemetry {
    /// Whether this run stopped because the mirror pushed back (R9).
    #[must_use]
    pub fn is_mirror_stop(&self) -> bool {
        self.stop.starts_with("http-") || self.stop == "network"
    }

    /// A one-line summary for the CI log and job summary.
    #[must_use]
    pub fn summary(&self) -> String {
        let statuses = if self.statuses.is_empty() {
            "none".to_owned()
        } else {
            self.statuses
                .iter()
                .map(|(status, count)| format!("{status}={count}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        format!(
            "{} requests, {} bytes, {} truncated, statuses: {statuses}, gap min {} ms / mean {} ms, \
             {} retries, budget {}, stop: {}",
            self.requests,
            self.bytes,
            self.truncated,
            self.min_gap_ms,
            self.mean_gap_ms,
            self.retries,
            self.budget,
            self.stop
        )
    }
}

/// The mirror-stop back-off (R9).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Backoff {
    /// Mirror stops since the last full recovery (capped at 8 halvings).
    #[serde(default)]
    pub consecutive_stops: u32,
    /// Unix seconds of the last mirror stop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_stop_unix: Option<u64>,
    /// Clean runs since that stop (3 clears the back-off).
    #[serde(default)]
    pub clean_runs: u32,
}

/// The crawl state: attempts, telemetry and back-off.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrawlState {
    /// Format version; only [`STATE_SCHEMA`] is accepted.
    pub schema: u32,
    /// When this state was written, RFC 3339 UTC.
    pub generated_at: String,
    /// Archive path (`levels/…/stem.zip`) → attempt.
    pub attempts: BTreeMap<String, Attempt>,
    /// Run records, newest last (bounded by [`MAX_TELEMETRY_RUNS`]).
    #[serde(default)]
    pub telemetry: Vec<RunTelemetry>,
    /// Mirror-stop back-off (R9).
    #[serde(default)]
    pub backoff: Backoff,
}

impl Default for CrawlState {
    fn default() -> Self {
        Self::new(0)
    }
}

impl CrawlState {
    /// An empty state stamped with `now_unix`.
    #[must_use]
    pub fn new(now_unix: u64) -> Self {
        Self {
            schema: STATE_SCHEMA,
            generated_at: format_unix_utc(now_unix),
            attempts: BTreeMap::new(),
            telemetry: Vec::new(),
            backoff: Backoff::default(),
        }
    }

    /// Parses a state file, or starts fresh when it is unusable.
    ///
    /// A bad state file must never block a publish: the index rebuilds from the
    /// inventory and the previous index, and this only costs a re-fetch of
    /// sidecars that failed yesterday.
    #[must_use]
    pub fn load_or_new(bytes: Option<&[u8]>, now_unix: u64) -> Self {
        let Some(bytes) = bytes else {
            return Self::new(now_unix);
        };
        if bytes.len() as u64 > MAX_STATE_BYTES {
            tracing::warn!("crawl state is oversized; starting a fresh one");
            return Self::new(now_unix);
        }
        match serde_json::from_slice::<Self>(bytes) {
            Ok(state) if state.validate().is_ok() => state,
            _ => {
                tracing::warn!("crawl state is unreadable; starting a fresh one");
                Self::new(now_unix)
            }
        }
    }

    /// Checks the whole document.
    ///
    /// # Errors
    ///
    /// Returns a human-readable reason.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != STATE_SCHEMA {
            return Err(format!(
                "crawl state schema {} is not {STATE_SCHEMA}",
                self.schema
            ));
        }
        if !is_rfc3339_utc(&self.generated_at) {
            return Err("crawl state has an invalid timestamp".into());
        }
        if self.telemetry.len() > MAX_TELEMETRY_RUNS * 2 {
            return Err("crawl state has too many telemetry records".into());
        }
        for (path, attempt) in &self.attempts {
            if path.len() > 512 || !path.starts_with("levels/") || path.contains("..") {
                return Err(format!("crawl state has an unsafe path `{path}`"));
            }
            if Date::parse_iso(&attempt.date).is_none() {
                return Err(format!("crawl state has an invalid date for `{path}`"));
            }
            for value in [&attempt.etag, &attempt.last_modified]
                .into_iter()
                .flatten()
            {
                if value.len() > 512 || value.chars().any(char::is_control) {
                    return Err(format!("crawl state has an invalid validator for `{path}`"));
                }
            }
        }
        if let Some(stop) = self.backoff.last_stop_unix
            && stop == 0
        {
            return Err("crawl state has a zero back-off timestamp".into());
        }
        Ok(())
    }

    /// The serialized document, bounded by [`MAX_STATE_BYTES`].
    ///
    /// # Errors
    ///
    /// Returns a reason when the document would be too large.
    pub fn to_json(&self) -> Result<Vec<u8>, String> {
        let mut bytes = serde_json::to_vec(self).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        if bytes.len() as u64 > MAX_STATE_BYTES {
            return Err(format!(
                "crawl state is larger than {MAX_STATE_BYTES} bytes"
            ));
        }
        Ok(bytes)
    }

    /// The `ETag` recorded for a path.
    #[must_use]
    pub fn etag(&self, path: &str) -> Option<&str> {
        self.attempts.get(path)?.etag.as_deref()
    }

    /// The `Last-Modified` recorded for a path.
    #[must_use]
    pub fn last_modified(&self, path: &str) -> Option<&str> {
        self.attempts.get(path)?.last_modified.as_deref()
    }

    /// Whether a failed attempt should be retried today.
    ///
    /// Successful attempts are not re-fetched because the published index
    /// carries their metadata, not because of this state. Missing or bad files
    /// are tried once per day: a mirror hiccup clears by the next run, and one
    /// permanently absent sidecar cannot consume tomorrow's budget.
    #[must_use]
    pub fn retry_allowed(&self, path: &str, today: Date) -> bool {
        match self.attempts.get(path) {
            None => true,
            Some(attempt) if attempt.outcome == AttemptOutcome::Ok => true,
            Some(attempt) => attempt.date != today.to_iso(),
        }
    }

    /// Records one attempt.
    pub fn record(
        &mut self,
        path: &str,
        today: Date,
        outcome: AttemptOutcome,
        etag: Option<String>,
        last_modified: Option<String>,
    ) {
        self.attempts.insert(
            path.to_owned(),
            Attempt {
                date: today.to_iso(),
                outcome,
                etag,
                last_modified,
            },
        );
    }

    /// Drops attempts for paths that are no longer in the inventory.
    pub fn retain(&mut self, paths: &BTreeSet<String>) {
        self.attempts.retain(|path, _| paths.contains(path));
    }

    /// The number of recorded attempts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.attempts.len()
    }

    /// Whether nothing has been attempted yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.attempts.is_empty()
    }

    /// The budget this run may use, after the R9 back-off.
    ///
    /// Returns `0` while the 24 h window after a mirror stop is open; otherwise
    /// halves the configured budget once per consecutive stop (floor
    /// [`BACKOFF_FLOOR`], never above the configured budget) until three clean
    /// runs clear the back-off.
    #[must_use]
    pub fn effective_budget(&self, configured: u32, now_unix: u64) -> u32 {
        if self.backoff_until().is_some_and(|until| now_unix < until) {
            return 0;
        }
        let halvings = self.backoff.consecutive_stops.min(8);
        let halved = configured >> halvings;
        let floor = configured.clamp(1, BACKOFF_FLOOR);
        halved.max(floor)
    }

    /// The instant until which the R9 window must be respected, if any.
    #[must_use]
    pub fn backoff_until(&self) -> Option<u64> {
        self.backoff
            .last_stop_unix
            .map(|stop| stop.saturating_add(BACKOFF_WINDOW_SECS))
    }

    /// Appends a run record and updates the back-off (R9).
    pub fn record_run(&mut self, telemetry: RunTelemetry) {
        if telemetry.is_mirror_stop() {
            self.backoff.consecutive_stops = (self.backoff.consecutive_stops + 1).min(8);
            self.backoff.last_stop_unix = Some(telemetry.finished_unix);
            self.backoff.clean_runs = 0;
        } else if telemetry.stop != "backoff-wait" && telemetry.stop != "error" {
            // A budget stop is a deliberate, healthy pause: it counts as clean.
            // A failed build (`error`) does not: it says nothing about the
            // mirror's health (R9).
            if self.backoff.consecutive_stops > 0 {
                self.backoff.clean_runs += 1;
                if self.backoff.clean_runs >= CLEAN_RUNS_TO_RECOVER {
                    self.backoff.consecutive_stops = 0;
                    self.backoff.clean_runs = 0;
                    self.backoff.last_stop_unix = None;
                }
            }
        }
        self.telemetry.push(telemetry);
        if self.telemetry.len() > MAX_TELEMETRY_RUNS {
            let excess = self.telemetry.len() - MAX_TELEMETRY_RUNS;
            self.telemetry.drain(..excess);
        }
    }

    /// The most recent run record.
    #[must_use]
    pub fn last_telemetry(&self) -> Option<&RunTelemetry> {
        self.telemetry.last()
    }
}

impl fmt::Display for AttemptOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Ok => "ok",
            Self::Missing => "missing",
            Self::Bad => "bad",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> Date {
        Date::new(2026, 10, 20).unwrap()
    }

    fn telemetry(stop: &str, finished_unix: u64) -> RunTelemetry {
        RunTelemetry {
            started_at: "2026-10-20T04:17:00Z".into(),
            finished_at: format_unix_utc(finished_unix),
            finished_unix,
            budget: 1_500,
            requests: 10,
            bytes: 32_000,
            truncated: 2,
            statuses: BTreeMap::from([("200".to_owned(), 8), ("404".to_owned(), 2)]),
            min_gap_ms: 3_012,
            mean_gap_ms: 3_501,
            retries: 0,
            stop: stop.to_owned(),
        }
    }

    #[test]
    fn a_state_round_trips() {
        let mut state = CrawlState::new(1_792_386_428);
        state.record(
            "levels/doom2/Ports/megawads/x.zip",
            today(),
            AttemptOutcome::Ok,
            Some("\"v1\"".into()),
            Some("Mon, 19 Oct 2026 05:07:08 GMT".into()),
        );
        let bytes = state.to_json().unwrap();
        let loaded = CrawlState::load_or_new(Some(&bytes), 0);
        assert_eq!(loaded, state);
        assert_eq!(
            loaded.etag("levels/doom2/Ports/megawads/x.zip"),
            Some("\"v1\"")
        );
    }

    #[test]
    fn a_bad_state_starts_fresh_instead_of_failing() {
        assert_eq!(CrawlState::load_or_new(None, 0), CrawlState::new(0));
        assert_eq!(
            CrawlState::load_or_new(Some(b"not json"), 0),
            CrawlState::new(0)
        );
        assert_eq!(
            CrawlState::load_or_new(Some(&vec![b' '; MAX_STATE_BYTES as usize + 1]), 0),
            CrawlState::new(0)
        );
        let mut wrong = CrawlState::new(0);
        wrong.schema = 99;
        assert_eq!(
            CrawlState::load_or_new(Some(&wrong.to_json().unwrap()), 0),
            CrawlState::new(0)
        );
    }

    #[test]
    fn failures_are_retried_once_per_day_and_successes_are_not_blocked() {
        let mut state = CrawlState::new(0);
        state.record(
            "levels/doom2/a-c/missing.zip",
            today(),
            AttemptOutcome::Missing,
            None,
            None,
        );
        assert!(!state.retry_allowed("levels/doom2/a-c/missing.zip", today()));
        assert!(state.retry_allowed(
            "levels/doom2/a-c/missing.zip",
            Date::new(2026, 10, 21).unwrap()
        ));
        state.record(
            "levels/doom2/a-c/ok.zip",
            today(),
            AttemptOutcome::Ok,
            None,
            None,
        );
        assert!(state.retry_allowed("levels/doom2/a-c/ok.zip", today()));
        assert!(state.retry_allowed("levels/doom2/a-c/never-tried.zip", today()));
    }

    #[test]
    fn paths_that_left_the_inventory_are_pruned() {
        let mut state = CrawlState::new(0);
        state.record(
            "levels/doom2/a-c/gone.zip",
            today(),
            AttemptOutcome::Bad,
            None,
            None,
        );
        state.record(
            "levels/doom2/a-c/stays.zip",
            today(),
            AttemptOutcome::Ok,
            None,
            None,
        );
        let mut paths = BTreeSet::new();
        paths.insert("levels/doom2/a-c/stays.zip".to_owned());
        state.retain(&paths);
        assert_eq!(state.len(), 1);
        assert!(state.attempts.contains_key("levels/doom2/a-c/stays.zip"));
    }

    #[test]
    fn unsafe_paths_are_refused_on_validation() {
        let mut state = CrawlState::new(0);
        state.attempts.insert(
            "../etc/passwd".into(),
            Attempt {
                date: "2026-10-20".into(),
                outcome: AttemptOutcome::Ok,
                etag: None,
                last_modified: None,
            },
        );
        assert!(state.validate().is_err());
    }

    #[test]
    fn a_mirror_stop_opens_a_24h_window_and_halves_the_budget() {
        let mut state = CrawlState::new(0);
        let stop_at = 1_792_386_428;
        state.record_run(telemetry("http-429", stop_at));
        assert_eq!(state.backoff.consecutive_stops, 1);
        assert_eq!(state.backoff_until(), Some(stop_at + BACKOFF_WINDOW_SECS));
        // Inside the window nothing is fetched.
        assert_eq!(state.effective_budget(1_500, stop_at + 60), 0);
        // After it, half the budget.
        assert_eq!(
            state.effective_budget(1_500, stop_at + BACKOFF_WINDOW_SECS),
            750
        );
        // A second stop halves again, then the floor holds.
        state.record_run(telemetry("network", stop_at + 90_000));
        assert_eq!(
            state.effective_budget(1_500, stop_at + 90_000 + BACKOFF_WINDOW_SECS),
            375
        );
        for extra in 0..6 {
            state.record_run(telemetry("http-500", stop_at + 200_000 + extra));
        }
        assert_eq!(
            state.effective_budget(1_500, stop_at + 400_000 + BACKOFF_WINDOW_SECS),
            BACKOFF_FLOOR
        );
    }

    #[test]
    fn three_clean_runs_clear_the_back_off_but_a_budget_stop_counts_as_clean() {
        let mut state = CrawlState::new(0);
        let stop_at = 1_792_386_428;
        state.record_run(telemetry("http-403", stop_at));
        state.record_run(telemetry("finished", stop_at + 100_000));
        assert_eq!(state.backoff.clean_runs, 1);
        state.record_run(telemetry("budget", stop_at + 200_000));
        assert_eq!(state.backoff.clean_runs, 2);
        state.record_run(telemetry("finished", stop_at + 300_000));
        assert_eq!(state.backoff.consecutive_stops, 0);
        assert_eq!(state.backoff.clean_runs, 0);
        assert_eq!(
            state.effective_budget(1_500, stop_at + 400_000),
            1_500,
            "the back-off must recover after three clean runs"
        );
    }

    #[test]
    fn waiting_runs_do_not_count_as_clean() {
        let mut state = CrawlState::new(0);
        state.record_run(telemetry("http-429", 1_000_000));
        state.record_run(telemetry("backoff-wait", 1_001_000));
        assert_eq!(state.backoff.clean_runs, 0);
        assert_eq!(state.backoff.consecutive_stops, 1);
        // A hard build failure says nothing about the mirror's health: it must
        // not shorten the recovery either (spec 050 follow-up F4).
        state.record_run(telemetry("error", 1_002_000));
        assert_eq!(state.backoff.clean_runs, 0);
        assert_eq!(state.backoff.consecutive_stops, 1);
    }

    #[test]
    fn telemetry_is_bounded_and_keeps_the_newest() {
        let mut state = CrawlState::new(0);
        for run in 0..(MAX_TELEMETRY_RUNS + 10) {
            state.record_run(telemetry("finished", 1_000 + run as u64));
        }
        assert_eq!(state.telemetry.len(), MAX_TELEMETRY_RUNS);
        assert_eq!(
            state.last_telemetry().map(|run| run.finished_unix),
            Some(1_000 + MAX_TELEMETRY_RUNS as u64 + 9)
        );
    }

    #[test]
    fn the_summary_names_every_number_r9_asks_for() {
        let run = telemetry("http-429", 1_792_386_428);
        let summary = run.summary();
        for needle in [
            "10 requests",
            "32000 bytes",
            "2 truncated",
            "200=8",
            "404=2",
            "gap min 3012 ms",
            "mean 3501 ms",
            "0 retries",
            "budget 1500",
            "stop: http-429",
        ] {
            assert!(summary.contains(needle), "{needle}: {summary}");
        }
        assert!(run.is_mirror_stop());
        assert!(!telemetry("budget", 0).is_mirror_stop());
    }
}

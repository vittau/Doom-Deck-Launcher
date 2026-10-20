//! The crawl's resumable state, committed beside the index (spec 050, R6).
//!
//! The first full pass over about 10,900 sidecars is deliberately spread over
//! several days at 1,500 requests a day. That only works if tomorrow's run
//! knows which files it already tried today: [`CrawlState`] records the outcome
//! and the HTTP validators (`ETag`, `Last-Modified`) per archive path, so a run
//! that hit its budget — or was stopped by a `429` — picks up where it left
//! off instead of restarting.
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

/// The crawl state: one attempt per archive path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrawlState {
    /// Format version; only [`STATE_SCHEMA`] is accepted.
    pub schema: u32,
    /// When this state was written, RFC 3339 UTC.
    pub generated_at: String,
    /// Archive path (`levels/…/stem.zip`) → attempt.
    pub attempts: BTreeMap<String, Attempt>,
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
        for (path, attempt) in &self.attempts {
            if path.len() > 512 || !path.starts_with("levels/") || path.contains("..") {
                return Err(format!("crawl state has an unsafe path `{path}`"));
            }
            if Date::parse_iso(&attempt.date).is_none() {
                return Err(format!("crawl state has an invalid date for `{path}`"));
            }
            for value in [&attempt.etag, &attempt.last_modified].into_iter().flatten() {
                if value.len() > 512 || value.chars().any(char::is_control) {
                    return Err(format!("crawl state has an invalid validator for `{path}`"));
                }
            }
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
        assert_eq!(loaded.etag("levels/doom2/Ports/megawads/x.zip"), Some("\"v1\""));
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
}

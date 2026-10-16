//! Injectable wall-clock time and a small UTC formatter (spec 004, R2/R6).
//!
//! Discovery stamps `fetched_at`, installation stamps `installed_at` and the
//! 24 h refresh throttle compares against "now"; putting all of that behind
//! [`Clock`] keeps tests deterministic (decision D-003) and keeps `ddl-core`
//! free of a date/time dependency.

use std::time::{SystemTime, UNIX_EPOCH};

/// A source of wall-clock time.
pub trait Clock {
    /// Seconds since the Unix epoch.
    fn now_unix_secs(&self) -> u64;
}

/// The real clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_secs(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }
}

/// Formats Unix seconds as a UTC ISO-8601 timestamp, e.g.
/// `2026-10-16T19:16:04Z`.
///
/// Implemented directly (Hinnant's civil-from-days) so DDL needs no date
/// library; it is used for the "offline (cached <time>)" message (R2).
#[must_use]
pub fn format_unix_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let time = secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    let (hour, minute, second) = (time / 3600, (time % 3600) / 60, time % 60);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Converts days since 1970-01-01 into a proleptic Gregorian `(year, month, day)`.
///
/// Source: Howard Hinnant, "chrono-Compatible Low-Level Date Algorithms"
/// (<https://howardhinnant.github.io/date_algorithms.html>), public domain.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_known_instants() {
        assert_eq!(format_unix_utc(0), "1970-01-01T00:00:00Z");
        // 2026-10-09T19:16:04Z, the v0.30.1 publish time from the research.
        assert_eq!(format_unix_utc(1_791_573_364), "2026-10-09T19:16:04Z");
        // A leap day.
        assert_eq!(format_unix_utc(1_582_934_400), "2020-02-29T00:00:00Z");
    }

    #[test]
    fn system_clock_is_after_the_epoch() {
        assert!(SystemClock.now_unix_secs() > 1_600_000_000);
    }
}

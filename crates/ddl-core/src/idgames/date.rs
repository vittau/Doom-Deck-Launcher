//! Day-precision archive dates without a calendar crate (spec 050, R1).
//!
//! The `/idgames` inventory prints dates the way `ls -laR` does: `Jul 26  2021`
//! for files older than about six months and `Oct 18 11:48` for recent ones.
//! The recent form has **no year**, so it has to be inferred from the day the
//! listing was generated, and the timestamps are server-local: only the day is
//! trustworthy (see `docs/research/idgames-browse.md` §2.6). [`Date`] is that
//! day, and nothing here ever invents a time of day.

use std::fmt;

/// The month names `ls` prints, in order.
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// HTTP/IMF-fixpoint weekday names, Sunday first.
const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/// A valid civil day (`YYYY-MM-DD`). Ordering is chronological.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    /// Builds a date, refusing an impossible day such as 31 February.
    #[must_use]
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if !(1900..=2200).contains(&year) || !(1..=12).contains(&month) {
            return None;
        }
        if day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// The year.
    #[must_use]
    pub fn year(self) -> i32 {
        self.year
    }

    /// The month, 1–12.
    #[must_use]
    pub fn month(self) -> u8 {
        self.month
    }

    /// The day of the month, 1–31.
    #[must_use]
    pub fn day(self) -> u8 {
        self.day
    }

    /// Parses the canonical `YYYY-MM-DD` form.
    #[must_use]
    pub fn parse_iso(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let year = parse_digits(&text[0..4])?;
        let month = parse_digits(&text[5..7])?;
        let day = parse_digits(&text[8..10])?;
        Self::new(year as i32, month as u8, day as u8)
    }

    /// Parses the `YYYY/MM/DD` form `fullsort.gz` uses.
    #[must_use]
    pub fn parse_slash_iso(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'/' || bytes[7] != b'/' {
            return None;
        }
        let mut dashed = String::with_capacity(10);
        dashed.push_str(&text[0..4]);
        dashed.push('-');
        dashed.push_str(&text[5..7]);
        dashed.push('-');
        dashed.push_str(&text[8..10]);
        Self::parse_iso(&dashed)
    }

    /// The canonical `YYYY-MM-DD` form.
    #[must_use]
    pub fn to_iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// The UTC day containing a Unix timestamp.
    #[must_use]
    pub fn from_unix(seconds: u64) -> Self {
        let (year, month, day) = civil_from_days((seconds / 86_400) as i64);
        Self::new(year, month, day).unwrap_or(Self {
            year: 1970,
            month: 1,
            day: 1,
        })
    }

    /// The HTTP-date form at midnight UTC (`Wed, 26 Jul 2021 00:00:00 GMT`),
    /// used for `If-Modified-Since` (spec 050, R2).
    #[must_use]
    pub fn to_http(self) -> String {
        let days = days_from_civil(self.year, self.month, self.day);
        // 1970-01-01 was a Thursday; `days` 0 maps to index 4 in a Sunday-first week.
        let weekday = WEEKDAYS[(days + 4).rem_euclid(7) as usize];
        let month = MONTHS[usize::from(self.month - 1)];
        format!(
            "{weekday}, {:02} {month} {:04} 00:00:00 GMT",
            self.day, self.year
        )
    }

    /// Whole days since the Unix epoch, for ordering.
    #[must_use]
    pub fn days(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_iso())
    }
}

/// The three-digit month `ls` prints (`Jan` … `Dec`).
#[must_use]
pub fn parse_month(name: &str) -> Option<u8> {
    MONTHS
        .iter()
        .position(|month| month.eq_ignore_ascii_case(name))
        .map(|index| index as u8 + 1)
}

/// The year for a recent `ls` date (`Oct 18 11:48`), inferred against `today`.
///
/// `ls` omits the year for files newer than roughly six months. The listing is
/// generated "now", so a month later than today's month belongs to the previous
/// year and everything else to the current one. That is wrong only for a file
/// that is more than a year old yet still printed without a year, which the
/// six-month rule makes impossible.
#[must_use]
pub fn infer_recent_year(month: u8, today: Date) -> i32 {
    if month > today.month {
        today.year - 1
    } else {
        today.year
    }
}

/// Parses the `month day year-or-time` triple of an `ls` line.
///
/// `year_or_time` is either a four-digit year (`2021`) or a clock time
/// (`11:48`), in which case the year is inferred from `today`.
#[must_use]
pub fn parse_ls_date(month: &str, day: &str, year_or_time: &str, today: Date) -> Option<Date> {
    let month = parse_month(month)?;
    let day = parse_digits(day)? as u8;
    let year = if year_or_time.contains(':') {
        infer_recent_year(month, today)
    } else {
        parse_digits(year_or_time)? as i32
    };
    Date::new(year, month, day)
}

/// Formats Unix seconds as the RFC 3339 UTC stamp the index publishes
/// (`2026-10-19T05:07:08Z`).
#[must_use]
pub fn format_unix_utc(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let (year, month, day) = civil_from_days(days);
    let rest = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        (rest % 3600) / 60,
        rest % 60
    )
}

/// Whether `text` is the `YYYY-MM-DDTHH:MM:SSZ` form [`format_unix_utc`] writes.
#[must_use]
pub fn is_rfc3339_utc(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    let parts = [
        &text[0..4],
        &text[5..7],
        &text[8..10],
        &text[11..13],
        &text[14..16],
        &text[17..19],
    ];
    if parts.iter().any(|part| parse_digits(part).is_none()) {
        return false;
    }
    if Date::parse_iso(&text[0..10]).is_none() {
        return false;
    }
    let hours = parse_digits(&text[11..13]).unwrap_or(99);
    let minutes = parse_digits(&text[14..16]).unwrap_or(99);
    let seconds = parse_digits(&text[17..19]).unwrap_or(99);
    hours < 24 && minutes < 60 && seconds < 60
}

fn parse_digits(text: &str) -> Option<u32> {
    if text.is_empty() || text.len() > 4 || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Days since 1970-01-01 (Howard Hinnant's `days_from_civil`).
fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let day = i64::from(day);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The inverse of [`days_from_civil`].
fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (year as i32, month as u8, day as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> Date {
        Date::new(2026, 10, 20).unwrap()
    }

    #[test]
    fn rejects_impossible_days() {
        assert!(Date::new(2026, 2, 29).is_none());
        assert!(Date::new(2024, 2, 29).is_some());
        assert!(Date::new(2026, 4, 31).is_none());
        assert!(Date::new(2026, 13, 1).is_none());
        assert!(Date::new(2026, 1, 0).is_none());
    }

    #[test]
    fn iso_round_trips() {
        let date = Date::new(2021, 7, 26).unwrap();
        assert_eq!(date.to_iso(), "2021-07-26");
        assert_eq!(Date::parse_iso("2021-07-26"), Some(date));
        assert_eq!(Date::parse_iso("2021-7-26"), None);
        assert_eq!(Date::parse_iso("2021-02-30"), None);
    }

    #[test]
    fn unix_epoch_and_known_timestamps() {
        assert_eq!(Date::from_unix(0), Date::new(1970, 1, 1).unwrap());
        // 2026-10-19T05:07:08Z, the inventory `Last-Modified` in the research.
        assert_eq!(format_unix_utc(1_792_386_428), "2026-10-19T05:07:08Z");
        assert_eq!(
            Date::from_unix(1_792_386_428),
            Date::new(2026, 10, 19).unwrap()
        );
    }

    #[test]
    fn recent_ls_dates_infer_the_year_from_today() {
        // Same month or earlier: this year.
        assert_eq!(
            parse_ls_date("Oct", "18", "11:48", today()).map(Date::to_iso),
            Some("2026-10-18".to_owned())
        );
        // A later month belongs to last year (a January listing shows December).
        assert_eq!(
            parse_ls_date("Dec", "24", "12:00", Date::new(2026, 1, 5).unwrap()).map(Date::to_iso),
            Some("2025-12-24".to_owned())
        );
        // An explicit year is used verbatim.
        assert_eq!(
            parse_ls_date("Jul", "26", "2021", today()).map(Date::to_iso),
            Some("2021-07-26".to_owned())
        );
        assert_eq!(parse_ls_date("Smarch", "1", "2021", today()), None);
    }

    #[test]
    fn http_date_has_the_right_weekday() {
        // 2026-10-19 was a Monday.
        assert_eq!(
            Date::new(2026, 10, 19).unwrap().to_http(),
            "Mon, 19 Oct 2026 00:00:00 GMT"
        );
        assert_eq!(
            Date::new(1970, 1, 1).unwrap().to_http(),
            "Thu, 01 Jan 1970 00:00:00 GMT"
        );
    }

    #[test]
    fn rfc3339_shape_is_checked() {
        assert!(is_rfc3339_utc("2026-10-19T05:07:08Z"));
        assert!(!is_rfc3339_utc("2026-10-19 05:07:08"));
        assert!(!is_rfc3339_utc("2026-10-19T25:07:08Z"));
        assert!(!is_rfc3339_utc("2026-02-30T05:07:08Z"));
    }
}

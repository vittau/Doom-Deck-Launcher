//! The bounded, strict parser for DSDA-Doom's `stats.txt` (spec 116, R2).
//!
//! Upstream's own loader is deliberately permissive
//! (`docs/research/dsda-stats.md` Q5): it accepts any header at or below
//! version 1, deletes a file with an unknown header and skips malformed rows.
//! DDL treats the file as untrusted input instead and rejects the **whole**
//! document on any violation, so a truncated or edited snapshot can never
//! inflate progress from a valid prefix.
//!
//! Contract (all bounds are DDL's product limits, not DSDA's):
//!
//! * at most [`MAX_BYTES`] read, at most [`MAX_ROWS`] rows and
//!   [`MAX_LINE_BYTES`] bytes per line;
//! * exactly version [`STATS_VERSION`], exactly two header lines and exactly
//!   15 space-separated tokens per row;
//! * checked signed integers in the engine's `int` range;
//! * `-1` only for the fields DSDA documents as unknown, skill `0`-`5`;
//! * ASCII map IDs of at most [`MAX_MAP_ID_BYTES`] bytes, no duplicates;
//! * LF or CRLF, and the final row must be terminated.

use std::fmt;

/// Largest `stats.txt` DDL reads (1 MiB).
pub const MAX_BYTES: u64 = 1024 * 1024;
/// Largest number of map rows DDL accepts.
pub const MAX_ROWS: usize = 4096;
/// Largest number of bytes in one line, including its CR LF.
pub const MAX_LINE_BYTES: usize = 512;
/// Largest number of bytes in a map lump ID.
pub const MAX_MAP_ID_BYTES: usize = 8;
/// The only stats format version DDL understands.
pub const STATS_VERSION: i32 = 1;
/// The highest built-in DSDA-Doom skill (Nightmare).
pub const MAX_SKILL: u8 = 5;
/// Tokens DSDA writes on every map row (lump plus 14 integers).
pub const ROW_TOKENS: usize = 15;
/// Header lines before the map rows: version, then the WAD-wide kill total.
pub const HEADER_LINES: usize = 2;

/// One map row, in DSDA's write order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The map lump name (`MAP01`, `E1M1`).
    pub id: String,
    /// Parsed episode; `-1` when the engine did not recognize the name.
    pub episode: i32,
    /// Parsed map number; `-1` when the engine did not recognize the name.
    pub map: i32,
    /// Highest recorded one-based skill, or `0` for none.
    pub best_skill: u8,
    /// Best eligible map time in tics; `-1` unknown.
    pub best_time: i32,
    /// Best eligible max time in tics; `-1` unknown.
    pub best_max_time: i32,
    /// Best Nightmare time in tics; `-1` unknown.
    pub best_nm_time: i32,
    /// Number of recorded exits.
    pub total_exits: u32,
    /// Cumulative kills recorded on this map.
    pub total_kills: u32,
    /// Best eligible kill count.
    pub best_kills: u32,
    /// Best eligible item count.
    pub best_items: u32,
    /// Best eligible secret count.
    pub best_secrets: u32,
    /// Denominator from the most recent eligible exit; `-1` unknown.
    pub max_kills: i32,
    /// Denominator from the most recent eligible exit; `-1` unknown.
    pub max_items: i32,
    /// Denominator from the most recent eligible exit; `-1` unknown.
    pub max_secrets: i32,
}

impl Row {
    /// Whether DSDA recorded a non-no-monsters skill for this map: D-045's
    /// completion rule.
    #[must_use]
    pub const fn completed(&self) -> bool {
        self.best_skill > 0
    }

    /// Whether this row earns the D-045 **UV-Max** mark (spec 116, R10): a UV or
    /// higher skill, both totals known, and best kills/secrets reaching them.
    ///
    /// DSDA keeps each best independently, so the mark means "all kills and all
    /// secrets at UV", possibly across separate runs; it is not proof of one
    /// continuous UV-Max demo. Totals of `-1` mean unknown and never qualify.
    #[must_use]
    pub const fn uv_max(&self) -> bool {
        self.best_skill >= 4
            && self.max_kills >= 0
            && self.max_secrets >= 0
            && self.best_kills as i64 >= self.max_kills as i64
            && self.best_secrets as i64 >= self.max_secrets as i64
    }
}

/// A whole parsed file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stats {
    /// The WAD-wide cumulative kill count on the second header line.
    pub total_kills: u32,
    /// Map rows in file order.
    pub rows: Vec<Row>,
}

/// Why a `stats.txt` was rejected as a whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input is larger than [`MAX_BYTES`].
    Oversize {
        /// The accepted limit.
        limit: u64,
        /// The observed size.
        got: usize,
    },
    /// A line is longer than [`MAX_LINE_BYTES`].
    LineTooLong {
        /// 1-based file line.
        line: usize,
    },
    /// More than [`MAX_ROWS`] map rows.
    TooManyRows {
        /// The accepted limit.
        limit: usize,
    },
    /// The file does not end with a newline, so the last row is unterminated.
    MissingFinalNewline,
    /// The first line is not exactly version [`STATS_VERSION`].
    UnsupportedVersion {
        /// What the file declared, bounded for display.
        got: String,
    },
    /// The file has fewer than [`HEADER_LINES`] lines.
    MissingHeaders,
    /// A token is not a checked integer in the engine's `int` range.
    BadInteger {
        /// 1-based file line.
        line: usize,
        /// The offending token.
        token: String,
    },
    /// A row does not have exactly [`ROW_TOKENS`] tokens.
    WrongTokenCount {
        /// 1-based file line.
        line: usize,
        /// Tokens found.
        got: usize,
    },
    /// A value is outside the range its field documents.
    OutOfRange {
        /// 1-based file line.
        line: usize,
        /// The field name.
        field: &'static str,
    },
    /// The map lump ID is empty, longer than eight bytes or not printable ASCII.
    BadMapId {
        /// 1-based file line.
        line: usize,
    },
    /// Two rows name the same map lump.
    DuplicateMapId {
        /// 1-based file line.
        line: usize,
        /// The repeated ID.
        id: String,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Oversize { limit, got } => {
                write!(f, "stats.txt is larger than {limit} bytes (got {got})")
            }
            Self::LineTooLong { line } => write!(f, "stats.txt line {line} is too long"),
            Self::TooManyRows { limit } => {
                write!(f, "stats.txt has more than {limit} map rows")
            }
            Self::MissingFinalNewline => f.write_str("stats.txt does not end with a newline"),
            Self::UnsupportedVersion { got } => {
                write!(f, "stats.txt version `{got}` is not supported")
            }
            Self::MissingHeaders => f.write_str("stats.txt has fewer than two header lines"),
            Self::BadInteger { line, token } => {
                write!(f, "stats.txt line {line} token `{token}` is not an integer")
            }
            Self::WrongTokenCount { line, got } => {
                write!(
                    f,
                    "stats.txt line {line} has {got} tokens, expected {ROW_TOKENS}"
                )
            }
            Self::OutOfRange { line, field } => {
                write!(f, "stats.txt line {line} field `{field}` is out of range")
            }
            Self::BadMapId { line } => write!(f, "stats.txt line {line} has an invalid map ID"),
            Self::DuplicateMapId { line, id } => {
                write!(f, "stats.txt line {line} repeats map ID `{id}`")
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses `bytes` as a complete `stats.txt`.
///
/// # Errors
///
/// Returns the first [`ParseError`]; any error rejects the whole file.
pub fn parse(bytes: &[u8]) -> Result<Stats, ParseError> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err(ParseError::Oversize {
            limit: MAX_BYTES,
            got: bytes.len(),
        });
    }
    if bytes.is_empty() {
        return Err(ParseError::MissingHeaders);
    }
    if bytes.last() != Some(&b'\n') {
        return Err(ParseError::MissingFinalNewline);
    }
    let mut lines: Vec<&[u8]> = Vec::new();
    let mut start = 0;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'\n' {
            continue;
        }
        let mut end = index;
        if end > start && bytes[end - 1] == b'\r' {
            end -= 1;
        }
        if end - start > MAX_LINE_BYTES {
            return Err(ParseError::LineTooLong {
                line: lines.len() + 1,
            });
        }
        lines.push(&bytes[start..end]);
        start = index + 1;
    }
    if lines.len() < HEADER_LINES {
        return Err(ParseError::MissingHeaders);
    }
    if lines[0] != STATS_VERSION.to_string().as_bytes() {
        return Err(ParseError::UnsupportedVersion {
            got: bounded_text(lines[0]),
        });
    }
    let total_kills = match integer(2, lines[1])? {
        value if value >= 0 => {
            #[allow(clippy::cast_sign_loss)]
            let count = value as u32;
            count
        }
        _ => {
            return Err(ParseError::OutOfRange {
                line: 2,
                field: "total_kills",
            });
        }
    };
    let rows = &lines[HEADER_LINES..];
    if rows.len() > MAX_ROWS {
        return Err(ParseError::TooManyRows { limit: MAX_ROWS });
    }
    let mut parsed: Vec<Row> = Vec::with_capacity(rows.len());
    let mut ids: Vec<String> = Vec::with_capacity(rows.len());
    for (offset, line) in rows.iter().enumerate() {
        let number = offset + HEADER_LINES + 1;
        let mut tokens = line.split(|byte| *byte == b' ');
        let mut fields: Vec<&[u8]> = Vec::with_capacity(ROW_TOKENS);
        for token in tokens.by_ref() {
            if token.is_empty() {
                return Err(ParseError::WrongTokenCount {
                    line: number,
                    got: fields.len(),
                });
            }
            fields.push(token);
        }
        if fields.len() != ROW_TOKENS {
            return Err(ParseError::WrongTokenCount {
                line: number,
                got: fields.len(),
            });
        }
        let id = map_id(number, fields[0])?;
        if ids.iter().any(|held| held == &id) {
            return Err(ParseError::DuplicateMapId { line: number, id });
        }
        ids.push(id.clone());
        let values: Vec<i64> = fields[1..]
            .iter()
            .map(|token| integer(number, token))
            .collect::<Result<_, _>>()?;
        parsed.push(Row {
            id,
            episode: bounded(number, "episode", values[0], -1, 99)?,
            map: bounded(number, "map", values[1], -1, 99)?,
            best_skill: u8::try_from(bounded(
                number,
                "best_skill",
                values[2],
                0,
                i64::from(MAX_SKILL),
            )?)
            .unwrap_or(0),
            best_time: bounded(number, "best_time", values[3], -1, i64::from(i32::MAX))?,
            best_max_time: bounded(number, "best_max_time", values[4], -1, i64::from(i32::MAX))?,
            best_nm_time: bounded(number, "best_nm_time", values[5], -1, i64::from(i32::MAX))?,
            total_exits: counter(number, "total_exits", values[6])?,
            total_kills: counter(number, "total_kills", values[7])?,
            best_kills: counter(number, "best_kills", values[8])?,
            best_items: counter(number, "best_items", values[9])?,
            best_secrets: counter(number, "best_secrets", values[10])?,
            max_kills: bounded(number, "max_kills", values[11], -1, i64::from(i32::MAX))?,
            max_items: bounded(number, "max_items", values[12], -1, i64::from(i32::MAX))?,
            max_secrets: bounded(number, "max_secrets", values[13], -1, i64::from(i32::MAX))?,
        });
    }
    Ok(Stats {
        total_kills,
        rows: parsed,
    })
}

/// A checked integer in the range of the engine's `int`.
fn integer(line: usize, token: &[u8]) -> Result<i64, ParseError> {
    let text = std::str::from_utf8(token).map_err(|_| ParseError::BadInteger {
        line,
        token: bounded_text(token),
    })?;
    text.parse::<i64>()
        .ok()
        .filter(|value| (i64::from(i32::MIN)..=i64::from(i32::MAX)).contains(value))
        .ok_or_else(|| ParseError::BadInteger {
            line,
            token: bounded_text(token),
        })
}

/// A value in `min..=max`, reported as `field` when it is not.
fn bounded(
    line: usize,
    field: &'static str,
    value: i64,
    min: i64,
    max: i64,
) -> Result<i32, ParseError> {
    if (min..=max).contains(&value) {
        #[allow(clippy::cast_possible_truncation)]
        Ok(value as i32)
    } else {
        Err(ParseError::OutOfRange { line, field })
    }
}

/// A non-negative counter.
fn counter(line: usize, field: &'static str, value: i64) -> Result<u32, ParseError> {
    if value >= 0 {
        #[allow(clippy::cast_sign_loss)]
        let count = value as u32;
        Ok(count)
    } else {
        Err(ParseError::OutOfRange { line, field })
    }
}

/// A printable ASCII map lump ID of at most [`MAX_MAP_ID_BYTES`] bytes.
fn map_id(line: usize, token: &[u8]) -> Result<String, ParseError> {
    if token.is_empty()
        || token.len() > MAX_MAP_ID_BYTES
        || !token.iter().all(|byte| matches!(*byte, 0x21..=0x7e))
    {
        return Err(ParseError::BadMapId { line });
    }
    Ok(String::from_utf8_lossy(token).into_owned())
}

/// At most 16 bytes of untrusted text, for a diagnostic only.
fn bounded_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(&bytes[..bytes.len().min(16)]).into_owned()
}

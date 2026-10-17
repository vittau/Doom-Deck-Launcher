//! Typed, user-explainable errors for downloads and extraction (spec 006, R6).
//!
//! Every variant describes something a person can act on: a network problem,
//! a rejected host, an exceeded size, a hash mismatch naming both values, a
//! missing archive member (with the closest case-insensitive match), an unsafe
//! member name, or a violated safety limit. The UI renders [`std::fmt::Display`].

use std::fmt;
use std::time::Duration;

use super::hash::Sha256;

/// Which safety limit was exceeded (spec 006, R5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitKind {
    /// Total uncompressed bytes across the whole archive.
    TotalSize,
    /// Uncompressed bytes of a single member.
    MemberSize,
    /// Number of entries in an archive.
    MemberCount,
    /// Uncompressed-to-compressed ratio (zip-bomb guard).
    CompressionRatio,
    /// Nesting depth of `!/`-separated archive paths.
    NestingDepth,
}

impl LimitKind {
    /// A short, human-readable name for the limit.
    #[must_use]
    pub fn describe(self) -> &'static str {
        match self {
            Self::TotalSize => "total uncompressed size",
            Self::MemberSize => "single member size",
            Self::MemberCount => "member count",
            Self::CompressionRatio => "compression ratio",
            Self::NestingDepth => "archive nesting depth",
        }
    }
}

/// A download or extraction failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The URL does not use HTTPS. Plain HTTP is refused (spec 006, R1).
    NotHttps { url: String },
    /// The URL's host is not on the caller's allowlist (or, for a redirect
    /// target, on the allowlist or the redirect exception list).
    HostNotAllowed { host: String, url: String },
    /// The URL could not be parsed, or a redirect `Location` could not be
    /// resolved.
    InvalidUrl { url: String },
    /// More than the permitted number of redirects was followed.
    TooManyRedirects { limit: u32, url: String },
    /// A connect, TLS or read failure.
    Network { url: String, detail: String },
    /// The transfer did not finish within its size-scaled body deadline
    /// (`fetch::body_deadline`), i.e. it sustained less than the minimum rate
    /// for long enough that the whole transfer ran out of budget.
    TooSlow {
        url: String,
        /// Bytes received before the deadline fired.
        downloaded: u64,
        /// Announced or expected total, when known.
        total: Option<u64>,
        /// How long the body was read before giving up.
        elapsed: Duration,
    },
    /// The server answered with a non-success status.
    HttpStatus { url: String, status: u16 },
    /// The transfer exceeded an allowed size.
    SizeExceeded { limit: u64, got: u64 },
    /// The downloaded bytes did not hash to the expected value.
    HashMismatch { expected: Sha256, actual: Sha256 },
    /// The caller cancelled the operation.
    Cancelled,
    /// The archive is malformed or could not be read.
    ArchiveCorrupt { detail: String },
    /// The archive uses a compression method DDL does not support.
    UnsupportedCompression { member: String, method: String },
    /// A requested member does not exist. `closest` is the nearest
    /// case-insensitive match, when there is one, to help the user fix it.
    MemberMissing {
        requested: String,
        closest: Option<String>,
    },
    /// A requested member is a directory, not a file.
    MemberNotAFile { name: String },
    /// A member name is unsafe (absolute, `..`, drive prefix, backslash or
    /// NUL), or the entry is a symlink.
    UnsafeMember { name: String, reason: String },
    /// Two requested members resolve to the same output path.
    MemberConflict { first: String, second: String },
    /// A safety limit was exceeded.
    LimitExceeded { kind: LimitKind, detail: String },
    /// A local filesystem operation failed.
    Io { context: String, detail: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotHttps { url } => {
                write!(f, "refusing to download over plain HTTP: {url}")
            }
            Self::HostNotAllowed { host, url } => {
                write!(f, "host '{host}' is not allowed for download: {url}")
            }
            Self::InvalidUrl { url } => write!(f, "invalid download URL: {url}"),
            Self::TooManyRedirects { limit, url } => {
                write!(f, "gave up after {limit} redirects while fetching {url}")
            }
            Self::Network { url, detail } => write!(f, "network error for {url}: {detail}"),
            Self::TooSlow {
                url,
                downloaded,
                total,
                elapsed,
            } => {
                let after = elapsed.as_secs_f64();
                match total {
                    Some(total) => write!(
                        f,
                        "download too slow: {downloaded} of {total} bytes after {after:.1}s; check your connection and try again ({url})"
                    ),
                    None => write!(
                        f,
                        "download too slow: {downloaded} bytes after {after:.1}s; check your connection and try again ({url})"
                    ),
                }
            }
            Self::HttpStatus { url, status } => {
                write!(f, "server returned HTTP {status} for {url}")
            }
            Self::SizeExceeded { limit, got } => write!(
                f,
                "download is larger than allowed (limit {limit} bytes, got {got} bytes)"
            ),
            Self::HashMismatch { expected, actual } => write!(
                f,
                "downloaded file failed the SHA-256 check (expected {expected}, got {actual})"
            ),
            Self::Cancelled => f.write_str("download cancelled"),
            Self::ArchiveCorrupt { detail } => write!(f, "archive is corrupt: {detail}"),
            Self::UnsupportedCompression { member, method } => write!(
                f,
                "archive member '{member}' uses unsupported compression ({method}); only stored and deflate are supported"
            ),
            Self::MemberMissing { requested, closest } => match closest {
                Some(hint) => write!(
                    f,
                    "archive member '{requested}' was not found; closest match is '{hint}' (member names are case-sensitive)"
                ),
                None => write!(f, "archive member '{requested}' was not found"),
            },
            Self::MemberNotAFile { name } => {
                write!(f, "archive member '{name}' is a directory, not a file")
            }
            Self::UnsafeMember { name, reason } => {
                write!(f, "unsafe archive member '{name}': {reason}")
            }
            Self::MemberConflict { first, second } => write!(
                f,
                "archive members '{first}' and '{second}' would extract to the same path"
            ),
            Self::LimitExceeded { kind, detail } => {
                write!(f, "{} limit exceeded: {detail}", kind.describe())
            }
            Self::Io { context, detail } => write!(f, "{context}: {detail}"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_mismatch_names_both_values() {
        let expected = Sha256::of(b"expected");
        let actual = Sha256::of(b"actual");
        let message = Error::HashMismatch { expected, actual }.to_string();
        assert!(message.contains(&expected.to_hex()));
        assert!(message.contains(&actual.to_hex()));
    }

    #[test]
    fn missing_member_includes_the_case_hint() {
        let with_hint = Error::MemberMissing {
            requested: "WADs/Main.WAD".to_owned(),
            closest: Some("wads/main.wad".to_owned()),
        }
        .to_string();
        assert!(with_hint.contains("wads/main.wad"));
        assert!(with_hint.contains("case-sensitive"));

        let without_hint = Error::MemberMissing {
            requested: "nope.wad".to_owned(),
            closest: None,
        }
        .to_string();
        assert!(without_hint.contains("nope.wad"));
    }

    #[test]
    fn limit_errors_name_the_limit() {
        let message = Error::LimitExceeded {
            kind: LimitKind::CompressionRatio,
            detail: "ratio 900 exceeds 100".to_owned(),
        }
        .to_string();
        assert!(message.contains("compression ratio"));
    }

    #[test]
    fn too_slow_names_progress_and_suggests_a_retry() {
        let with_total = Error::TooSlow {
            url: "https://example.invalid/pkg.zip".to_owned(),
            downloaded: 1024,
            total: Some(4096),
            elapsed: Duration::from_secs(90),
        }
        .to_string();
        assert!(with_total.contains("1024 of 4096 bytes"), "{with_total}");
        assert!(with_total.contains("90.0s"), "{with_total}");
        assert!(with_total.contains("check your connection"), "{with_total}");

        let without_total = Error::TooSlow {
            url: "https://example.invalid/pkg.zip".to_owned(),
            downloaded: 7,
            total: None,
            elapsed: Duration::from_millis(500),
        }
        .to_string();
        assert!(
            without_total.contains("7 bytes after 0.5s"),
            "{without_total}"
        );
    }
}

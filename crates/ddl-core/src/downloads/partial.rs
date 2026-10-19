//! Partial downloads and their sidecars (spec 040, R1/R2/R5).
//!
//! A pinned transfer writes into `<sha256>.part` in the downloads cache and
//! records what it holds in `<sha256>.part.json`: the schema version, the
//! request URL that served the bytes, how many bytes are on disk and the
//! server's validator (`ETag` or `Last-Modified`) when it sent one. The JSON
//! document is only written after the bytes it describes have been flushed,
//! and it is replaced atomically, so a crash leaves at worst an older (and
//! therefore still valid) byte count.
//!
//! The cache is bounded (R5): [`cleanup`] removes partials older than
//! [`PARTIAL_MAX_AGE`] and, oldest first, enough of the rest to stay under
//! [`PARTIAL_CACHE_LIMIT`]. It only ever looks at files whose names are a
//! SHA-256 followed by `.part`, so nothing else in the downloads cache is
//! touched.
//!
//! [`parse_content_range`] is here too: it is pure parsing for R2's `206`
//! check, kept beside the code that decides whether a partial may be resumed.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use super::hash::Sha256;

/// Schema version of [`Sidecar`]. Bump only with a migration; an unknown
/// version makes the partial unusable and it is discarded.
pub(crate) const SIDECAR_SCHEMA_VERSION: u32 = 1;

/// Partials older than this are removed when a [`Fetcher`] starts (R5).
///
/// [`Fetcher`]: super::fetch::Fetcher
pub(crate) const PARTIAL_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Total size of all kept partials, in bytes (2 GiB; R5).
pub(crate) const PARTIAL_CACHE_LIMIT: u64 = 2 * 1024 * 1024 * 1024;

/// The JSON sidecar of a partial download.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Sidecar {
    /// [`SIDECAR_SCHEMA_VERSION`].
    pub(crate) schema_version: u32,
    /// The request URL that served the bytes. A partial is resumed only by
    /// that same URL (R4: "kept only for the URL that served it").
    pub(crate) url: String,
    /// Bytes flushed to `<sha256>.part`.
    pub(crate) bytes: u64,
    /// `ETag` (preferred) or `Last-Modified` from the server, used verbatim
    /// as `If-Range` on the next attempt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) validator: Option<String>,
}

/// The data and sidecar paths for `hash` inside the downloads cache.
pub(crate) fn paths(dir: &Path, hash: Sha256) -> (PathBuf, PathBuf) {
    let hex = hash.to_hex();
    (
        dir.join(format!("{hex}.part")),
        dir.join(format!("{hex}.part.json")),
    )
}

/// Reads the sidecar for `hash` and checks that it may be resumed by `url`.
///
/// Returns `None` when there is no partial, the sidecar is unreadable, stale
/// or for another URL, or the bytes on disk are not what it claims. Damage
/// (unreadable sidecar, unsupported version, a shortened file, a byte count
/// beyond the recipe size) discards the partial; a partial for a different
/// mirror URL is left alone.
pub(crate) fn load(
    dir: &Path,
    hash: Sha256,
    url: &str,
    expected_size: Option<u64>,
) -> Option<Sidecar> {
    let (part, sidecar_path) = paths(dir, hash);
    let text = fs::read_to_string(&sidecar_path).ok()?;
    let sidecar: Sidecar = match serde_json::from_str(&text) {
        Ok(sidecar) => sidecar,
        Err(_) => {
            remove(dir, hash);
            return None;
        }
    };
    if sidecar.schema_version != SIDECAR_SCHEMA_VERSION {
        remove(dir, hash);
        return None;
    }
    if sidecar.url != url {
        // Another mirror's partial: keep it for the URL that served it.
        return None;
    }
    let on_disk = match fs::metadata(&part) {
        Ok(metadata) if metadata.is_file() => metadata.len(),
        _ => {
            remove(dir, hash);
            return None;
        }
    };
    if sidecar.bytes == 0 || sidecar.bytes > on_disk {
        remove(dir, hash);
        return None;
    }
    if expected_size.is_some_and(|expected| sidecar.bytes > expected) {
        remove(dir, hash);
        return None;
    }
    Some(sidecar)
}

/// Removes `<sha256>.part` and its sidecar, ignoring errors.
pub(crate) fn remove(dir: &Path, hash: Sha256) {
    let (part, sidecar) = paths(dir, hash);
    let _ = fs::remove_file(part);
    let _ = fs::remove_file(sidecar);
}

/// Writes the sidecar atomically (temporary file + rename) and flushes it.
///
/// The caller must have flushed the data file first: this document describes
/// bytes that are already on disk (R1).
pub(crate) fn write_sidecar(dir: &Path, hash: Sha256, sidecar: &Sidecar) -> std::io::Result<()> {
    let (_, path) = paths(dir, hash);
    let file_name = path.file_name().map_or_else(
        || "partial.json".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let temp = path.with_file_name(format!("{file_name}.{}.tmp", std::process::id()));
    let json = serde_json::to_string_pretty(sidecar)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    let write = || -> std::io::Result<()> {
        let mut file = File::create(&temp)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        file.sync_all()
    };
    if let Err(error) = write() {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    fs::rename(&temp, &path)
}

/// A parsed `Content-Range: bytes <start>-<end>/<total>` value (RFC 9110).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContentRange {
    pub(crate) start: u64,
    pub(crate) end: u64,
    /// `None` when the server sent `*` as the complete length.
    pub(crate) total: Option<u64>,
}

/// Parses a `Content-Range` header. Returns `None` for any malformed value,
/// including a unit other than `bytes`.
pub(crate) fn parse_content_range(value: &str) -> Option<ContentRange> {
    let (unit, rest) = value.trim().split_once(' ')?;
    if !unit.eq_ignore_ascii_case("bytes") {
        return None;
    }
    let (range, total) = rest.trim().split_once('/')?;
    let (start, end) = range.trim().split_once('-')?;
    let start: u64 = start.trim().parse().ok()?;
    let end: u64 = end.trim().parse().ok()?;
    if end < start {
        return None;
    }
    let total = match total.trim() {
        "*" => None,
        text => Some(text.parse().ok()?),
    };
    Some(ContentRange { start, end, total })
}

/// Removes expired and excess partials from `dir` (R5).
///
/// Age is taken from each `.part` file's modification time. Partials are then
/// removed oldest first until the total is at or below
/// [`PARTIAL_CACHE_LIMIT`]. Only names that parse as `<sha256>.part` are
/// considered; sidecars are removed with their data file.
pub(crate) fn cleanup(dir: &Path, now: SystemTime) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut partials: Vec<Entry> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            let hash = Sha256::parse_hex(name.strip_suffix(".part")?)?;
            let metadata = entry.metadata().ok()?;
            if !metadata.is_file() {
                return None;
            }
            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            Some(Entry {
                hash,
                len: metadata.len(),
                modified,
            })
        })
        .collect();
    partials.sort_by_key(|entry| (entry.modified, entry.hash));

    let mut kept = Vec::new();
    for entry in partials {
        let expired = now
            .duration_since(entry.modified)
            .is_ok_and(|age| age > PARTIAL_MAX_AGE);
        if expired {
            remove(dir, entry.hash);
        } else {
            kept.push(entry);
        }
    }

    let mut total: u64 = kept.iter().map(|entry| entry.len).sum();
    for entry in kept {
        if total <= PARTIAL_CACHE_LIMIT {
            break;
        }
        remove(dir, entry.hash);
        total = total.saturating_sub(entry.len);
    }
}

/// One partial file found by [`cleanup`]; `kept` is oldest first by
/// `(modified, hash)`.
#[derive(Debug)]
struct Entry {
    hash: Sha256,
    len: u64,
    modified: SystemTime,
}

/// Reads a file from the start, up to `limit` bytes, into `hasher`.
///
/// Used to continue a resumed transfer's SHA-256 from the bytes already on
/// disk. Returns how many bytes were consumed.
pub(crate) fn hash_prefix(
    path: &Path,
    limit: u64,
    hasher: &mut sha2::Sha256,
) -> std::io::Result<u64> {
    use sha2::Digest as _;

    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; 64 * 1024];
    let mut remaining = limit;
    while remaining > 0 {
        let want = buffer.len().min(remaining as usize);
        let read = file.read(&mut buffer[..want])?;
        if read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "partial download is shorter than its sidecar says",
            ));
        }
        hasher.update(&buffer[..read]);
        remaining -= read as u64;
    }
    Ok(limit)
}

#[cfg(test)]
mod tests {
    use sha2::Digest as _;

    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ddl-partial-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_partial(dir: &Path, hash: Sha256, bytes: &[u8], sidecar: &Sidecar) {
        let (part, _) = paths(dir, hash);
        fs::write(part, bytes).unwrap();
        write_sidecar(dir, hash, sidecar).unwrap();
    }

    #[test]
    fn parses_valid_content_ranges() {
        assert_eq!(
            parse_content_range("bytes 42-99/100"),
            Some(ContentRange {
                start: 42,
                end: 99,
                total: Some(100)
            })
        );
        assert_eq!(
            parse_content_range("Bytes 0-9/*"),
            Some(ContentRange {
                start: 0,
                end: 9,
                total: None
            })
        );
        // A server that omits the unit, has no total, or inverts the range is
        // unusable: the fetcher restarts from zero.
        assert_eq!(parse_content_range("42-99/100"), None);
        assert_eq!(parse_content_range("bytes 42-99"), None);
        assert_eq!(parse_content_range("bytes 99-42/100"), None);
        assert_eq!(parse_content_range("bytes x-42/100"), None);
        assert_eq!(parse_content_range("items 0-1/2"), None);
    }

    #[test]
    fn loads_a_matching_sidecar_and_refuses_another_url() {
        let dir = temp_dir("load");
        let hash = Sha256::of(b"the file");
        let sidecar = Sidecar {
            schema_version: SIDECAR_SCHEMA_VERSION,
            url: "https://a.example/file".to_owned(),
            bytes: 4,
            validator: Some("\"etag\"".to_owned()),
        };
        write_partial(&dir, hash, b"the file", &sidecar);

        let loaded = load(&dir, hash, "https://a.example/file", Some(9)).unwrap();
        assert_eq!(loaded, sidecar);

        // Another mirror may not resume bytes this URL did not serve; the
        // partial is left in place for the URL that did.
        assert!(load(&dir, hash, "https://b.example/file", Some(9)).is_none());
        assert!(paths(&dir, hash).0.exists());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn damage_discards_the_partial() {
        let dir = temp_dir("damage");
        let hash = Sha256::of(b"data");
        let sidecar = Sidecar {
            schema_version: SIDECAR_SCHEMA_VERSION,
            url: "https://a.example/file".to_owned(),
            bytes: 8,
            validator: None,
        };
        write_partial(&dir, hash, b"data", &sidecar);
        // The sidecar claims more bytes than the data file holds.
        assert!(load(&dir, hash, "https://a.example/file", None).is_none());
        let (part, meta) = paths(&dir, hash);
        assert!(!part.exists() && !meta.exists());

        // A future schema version is unusable rather than guessed at.
        let sidecar = Sidecar {
            schema_version: SIDECAR_SCHEMA_VERSION + 1,
            ..sidecar
        };
        write_partial(&dir, hash, b"data", &sidecar);
        assert!(load(&dir, hash, "https://a.example/file", None).is_none());
        assert!(!part.exists() && !meta.exists());

        // A byte count beyond the recipe size cannot be a prefix.
        let sidecar = Sidecar {
            schema_version: SIDECAR_SCHEMA_VERSION,
            bytes: 4,
            validator: None,
            url: "https://a.example/file".to_owned(),
        };
        write_partial(&dir, hash, b"data", &sidecar);
        assert!(load(&dir, hash, "https://a.example/file", Some(2)).is_none());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn hash_prefix_reads_exactly_the_recorded_bytes() {
        let dir = temp_dir("prefix");
        let path = dir.join("data");
        fs::write(&path, b"abcdef").unwrap();
        let mut hasher = sha2::Sha256::new();
        assert_eq!(hash_prefix(&path, 3, &mut hasher).unwrap(), 3);
        assert_eq!(Sha256::finish(hasher), Sha256::of(b"abc"));

        let mut short = sha2::Sha256::new();
        assert!(hash_prefix(&path, 7, &mut short).is_err());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn cleanup_removes_expired_partials_only() {
        let dir = temp_dir("expired");
        let now = SystemTime::now();
        let hash = Sha256::of(b"old");
        let sidecar = Sidecar {
            schema_version: SIDECAR_SCHEMA_VERSION,
            url: "https://a.example/file".to_owned(),
            bytes: 3,
            validator: None,
        };
        write_partial(&dir, hash, b"old", &sidecar);
        let (part, meta) = paths(&dir, hash);
        let old = now - PARTIAL_MAX_AGE - Duration::from_secs(60);
        File::options()
            .write(true)
            .open(&part)
            .unwrap()
            .set_modified(old)
            .unwrap();

        let fresh_hash = Sha256::of(b"new");
        write_partial(&dir, fresh_hash, b"new", &sidecar);
        let unrelated = dir.join("keep-me.bin");
        fs::write(&unrelated, b"not a partial").unwrap();

        cleanup(&dir, now);

        assert!(!part.exists(), "expired partial removed");
        assert!(!meta.exists(), "its sidecar goes with it");
        assert!(paths(&dir, fresh_hash).0.exists(), "fresh partial kept");
        assert!(unrelated.exists(), "nothing else is touched");

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn cleanup_keeps_the_total_under_the_limit_oldest_first() {
        let dir = temp_dir("bounded");
        let now = SystemTime::now();
        // Sparse files: the production 2 GiB cap is exercised without writing
        // 2 GiB to disk. Three 1 GiB partials are 1 GiB over the cap.
        let mut hashes = Vec::new();
        for (index, age) in [3_u64, 2, 1].into_iter().enumerate() {
            let hash = Sha256::of(format!("partial {index}").as_bytes());
            let (part, _) = paths(&dir, hash);
            let file = File::create(&part).unwrap();
            file.set_len(1024 * 1024 * 1024).unwrap();
            file.set_modified(now - Duration::from_secs(age * 60))
                .unwrap();
            hashes.push(hash);
        }
        let sidecar = Sidecar {
            schema_version: SIDECAR_SCHEMA_VERSION,
            url: "https://a.example/file".to_owned(),
            bytes: 1,
            validator: None,
        };
        // Sidecars are optional for this rule; write one to prove it follows.
        write_sidecar(&dir, hashes[0], &sidecar).unwrap();

        cleanup(&dir, now);

        // Oldest (index 0) is removed, taking the total from 3 GiB to exactly
        // the 2 GiB cap, so the rest stays. Its sidecar follows it.
        assert!(!paths(&dir, hashes[0]).0.exists());
        assert!(paths(&dir, hashes[1]).0.exists());
        assert!(paths(&dir, hashes[2]).0.exists());
        assert!(!paths(&dir, hashes[0]).1.exists());

        fs::remove_dir_all(dir).unwrap();
    }
}

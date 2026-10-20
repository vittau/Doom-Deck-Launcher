//! `/idgames` browse index: build, validate and publish (spec 050, D-031).
//!
//! The Doomworld `/idgames` API is unusable for automated clients (Cloudflare
//! 403, research §1), so DDL builds its own **facts-only** index from the
//! archive mirrors:
//!
//! 1. [`inventory`] reads `ls-laR.gz` (or `fullsort.gz`) and keeps the DOOM
//!    level trees;
//! 2. [`txt`] reads each upload's same-stem `.txt` sidecar for title, author,
//!    game and port evidence;
//! 3. [`build`] merges the two, adds the Cacoward ([`cacowards`]) and catalog
//!    ([`catalog_map`]) ranking signals, and writes [`index::Index`];
//! 4. [`fetch`] is the polite, bounded HTTP client and [`state`] is the
//!    resumable crawl state committed beside the index.
//!
//! CI publishes `index.json` on the `idgames-index` branch
//! (`.github/workflows/idgames-index.yml`); the app (spec 051) reads it like the
//! catalog bundle. Nothing here fetches at runtime and nothing executes
//! anything from the archive.
//!
//! # No descriptions, no credits, no txt bodies
//!
//! The index carries facts about a file — name, title, author, date, size,
//! inferred base game and port family — plus the ranking signals spec 050 R8
//! asks for. The author's text itself stays at the archive; the browse UI
//! fetches it on demand into the player's own cache (spec 051).

pub mod build;
pub mod cacowards;
pub mod catalog_map;
mod date;
pub mod fetch;
pub mod index;
pub mod inventory;
pub mod state;
pub mod txt;

pub use build::{BuildError, BuildOutcome, BuildReport, BuildSettings, StopReason, build_index};
pub use cacowards::{Cacowards, parse_cacowards};
pub use catalog_map::catalog_paths;
pub use date::{Date, format_unix_utc, is_rfc3339_utc};
pub use fetch::{Clock, HttpTxtSource, Mirror, SystemClock, TxtResponse, TxtSource, mirror_for};
pub use index::{
    DEFAULT_MIN_DDL_VERSION, Entry, INDEX_SCHEMA, Index, IndexError, IndexGame, IndexSource,
    MAX_INDEX_BYTES, Port,
};
pub use inventory::{
    Inventory, InventoryEntry, InventoryReport, MAX_INVENTORY_GZIP_BYTES, decode_inventory_gzip,
    parse_inventory, parse_inventory_gzip,
};
pub use state::{
    AttemptOutcome, BACKOFF_FLOOR, BACKOFF_WINDOW_SECS, Backoff, CLEAN_RUNS_TO_RECOVER, CrawlState,
    RunTelemetry,
};
pub use txt::{MAX_TXT_BYTES, TxtMeta, parse_txt};

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Writes `bytes` to `path` atomically: a sibling temp file, fsync, rename
/// (spec 050, R3).
///
/// # Errors
///
/// Returns an I/O error when the parent cannot be created, the temp file cannot
/// be written, or the rename fails. A failed write never leaves a partial file
/// at `path`.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temp = temp_path(path);
    let write = || -> std::io::Result<()> {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()
    };
    if let Err(error) = write() {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }
    std::fs::rename(&temp, path)
}

fn temp_path(path: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "idgames-index.json".to_owned());
    path.with_file_name(format!(".{name}.tmp.{}.{unique}", std::process::id()))
}

/// Reads a file no larger than `limit` bytes.
///
/// # Errors
///
/// Returns a human-readable reason when the file cannot be read or is oversized.
pub fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    if metadata.len() > limit {
        return Err(format!("{} is larger than {limit} bytes", path.display()));
    }
    std::fs::read(path).map_err(|error| format!("could not read {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_writes_leave_only_the_final_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/index.json");
        write_atomic(&path, b"{\"one\":1}\n").unwrap();
        write_atomic(&path, b"{\"two\":2}\n").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"{\"two\":2}\n");
        let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp."))
            .collect();
        assert!(leftovers.is_empty(), "no temp files may survive");
    }

    #[test]
    fn bounded_reads_refuse_oversized_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big");
        std::fs::write(&path, vec![b'x'; 32]).unwrap();
        assert!(read_bounded(&path, 31).is_err());
        assert_eq!(read_bounded(&path, 32).unwrap().len(), 32);
    }
}

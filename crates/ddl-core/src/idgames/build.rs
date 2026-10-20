//! The builder: inventory + sidecars → one deterministic index (spec 050).
//!
//! The build is a **very slow, polite crawl** (D-031, R6): one mirror, one
//! request in flight, at least [`MIN_REQUEST_GAP`] between requests plus
//! jitter, and at most [`DEFAULT_DAILY_BUDGET`] sidecar requests per run. A run
//! that stops early (budget, `403`, `429`, `5xx`, a mirror that keeps failing)
//! still publishes everything it has: every inventory entry is present from the
//! first day, with the filename fallback until its sidecar is read. The next
//! run continues because the previous index supplies the metadata already
//! fetched and [`super::state::CrawlState`] records the day's failures.
//!
//! Fetch order is by relevance (R7), so the first day's 1,500 requests buy the
//! entries a player is most likely to want:
//!
//! 1. Cacoward winners (spec 042's table, when present);
//! 2. `*/megawads/`;
//! 3. the classic episode trees (`levels/doom`, `levels/doom2`, not `Ports/`);
//! 4. everything else, which in practice is `Ports/<a-c>` single maps.
//!
//! Within a rank the newest uploads go first, then paths alphabetically, so the
//! order is deterministic.
//!
//! The map count is only known *after* a sidecar is read, so R7's
//! "episode-sized entries" cannot be ranked before the first pass. The classic
//! tree is the best available proxy, and every later pass ranks by the real
//! `maps` signal through the ordinary newest-first order. This is recorded in
//! the report as an open question rather than silently guessed.

use std::collections::{BTreeMap, BTreeSet};

use super::date::{Date, format_unix_utc};
use super::fetch::{
    DEFAULT_DAILY_BUDGET, MAX_CONSECUTIVE_FAILURES, MAX_DAILY_BUDGET, MIN_REQUEST_GAP, Mirror,
    Pacer, TxtResponse, TxtSource,
};
use super::index::{
    DEFAULT_MIN_DDL_VERSION, Entry, INDEX_SCHEMA, Index, IndexError, IndexGame, IndexSource,
};
use super::inventory::{InventoryEntry, parse_inventory};
use super::state::{AttemptOutcome, CrawlState};
use super::txt::parse_txt;

/// Everything the builder needs from the caller.
#[derive(Debug, Clone)]
pub struct BuildSettings {
    /// The one mirror this run reads from (R6).
    pub mirror: Mirror,
    /// URL of the inventory file, recorded in `source.index`.
    pub index_url: String,
    /// Lowercase hex SHA-256 of the inventory bytes, recorded in `source`.
    pub index_sha256: String,
    /// Oldest DDL allowed to read the result.
    pub min_ddl_version: String,
    /// Build time, UTC Unix seconds.
    pub now_unix: u64,
    /// Sidecar requests this run may make (1–[`MAX_DAILY_BUDGET`]).
    pub budget: u32,
    /// Cacoward winners by archive path (R7/R8).
    pub cacowards: BTreeMap<String, u16>,
    /// Recipe ids by archive path (R8).
    pub catalog: BTreeMap<String, String>,
}

impl BuildSettings {
    /// Settings with the deliberate defaults: the daily budget, the constant
    /// `min_ddl_version`, no Cacoward/catalog data and `now_unix` as the clock.
    #[must_use]
    pub fn new(mirror: Mirror, index_url: impl Into<String>, index_sha256: impl Into<String>, now_unix: u64) -> Self {
        Self {
            mirror,
            index_url: index_url.into(),
            index_sha256: index_sha256.into(),
            min_ddl_version: DEFAULT_MIN_DDL_VERSION.to_owned(),
            now_unix,
            budget: DEFAULT_DAILY_BUDGET,
            cacowards: BTreeMap::new(),
            catalog: BTreeMap::new(),
        }
    }
}

/// Why a run stopped fetching before exhausting the inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// The run used its daily request budget (R6).
    Budget,
    /// The mirror answered `403`, `429` or `5xx` (R6): stop, keep, resume.
    Blocked(u16),
    /// Too many consecutive transport failures: treat the mirror as down.
    Network,
}

impl std::fmt::Display for StopReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Budget => formatter.write_str("daily request budget reached"),
            Self::Blocked(status) => write!(formatter, "mirror answered HTTP {status}; stopping"),
            Self::Network => formatter.write_str("too many network failures; stopping"),
        }
    }
}

/// What one build did, for the CLI and the CI log.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildReport {
    /// Entries in the published index.
    pub entries: usize,
    /// Directories in the published index.
    pub directories: usize,
    /// Sidecar requests made.
    pub requests: u32,
    /// `200` responses read.
    pub fetched: u32,
    /// Entries whose metadata came from the previous index (no request).
    pub reused: u32,
    /// `304` responses and metadata kept.
    pub not_modified: u32,
    /// Uploads with no sidecar.
    pub missing_txt: u32,
    /// Sidecars that existed but yielded nothing usable, or failed.
    pub bad_txt: u32,
    /// Entries dropped from the inventory by the exclusion rules.
    pub excluded: usize,
    /// Why fetching stopped early, if it did.
    pub stopped: Option<StopReason>,
}

/// A finished build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOutcome {
    /// The validated, size-bounded document.
    pub index: Index,
    /// What happened.
    pub report: BuildReport,
}

/// A build that could not produce a document at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// The inventory had no DOOM level archives: almost always the wrong file.
    EmptyInventory,
    /// The produced document violated its own schema or the size bound.
    Index(IndexError),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInventory => formatter.write_str(
                "the inventory contains no levels/doom or levels/doom2 archives; \
                 check that --ls is the mirror's ls-laR.gz or fullsort.gz",
            ),
            Self::Index(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for BuildError {}

impl From<IndexError> for BuildError {
    fn from(error: IndexError) -> Self {
        Self::Index(error)
    }
}

/// Builds the index from inventory text.
///
/// `previous` is yesterday's published index (or a later one); entries whose
/// path, size and date are unchanged keep their metadata without a request.
/// `state` is updated in place and should be committed with the index.
///
/// # Errors
///
/// Returns [`BuildError`] when the inventory is empty or the result cannot be
/// validated or serialized within [`super::index::MAX_INDEX_BYTES`].
pub fn build_index(
    inventory_text: &str,
    settings: &BuildSettings,
    previous: Option<&Index>,
    state: &mut CrawlState,
    source: &mut dyn TxtSource,
    pacer: &mut dyn Pacer,
) -> Result<BuildOutcome, BuildError> {
    if settings.budget == 0 || settings.budget > MAX_DAILY_BUDGET {
        // Defensive: the CLI clamps, but the core must not hammer a mirror.
        return Err(BuildError::Index(IndexError::Malformed(format!(
            "budget {} is outside 1..={MAX_DAILY_BUDGET}",
            settings.budget
        ))));
    }
    let today = Date::from_unix(settings.now_unix);
    let inventory = parse_inventory(inventory_text, today);
    if inventory.entries.is_empty() {
        return Err(BuildError::EmptyInventory);
    }
    let mut report = BuildReport {
        excluded: inventory
            .report
            .excluded
            .iter()
            .map(|(_, count)| count)
            .sum(),
        ..BuildReport::default()
    };
    let previous_entries = flatten(previous);
    let mut rows: Vec<Row> = inventory
        .entries
        .into_iter()
        .map(|entry| row_for(entry, &previous_entries, &mut report))
        .collect();

    let mut candidates: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.needs_fetch && state.retry_allowed(&row.path, today))
        .map(|(index, _)| index)
        .collect();
    candidates.sort_by_key(|&index| {
        let row = &rows[index];
        (
            crawl_rank(&row.dir, &row.lower, &settings.cacowards),
            std::cmp::Reverse(row.entry.date.days()),
            row.lower.clone(),
        )
    });

    let mut consecutive_failures = 0u32;
    let mut stopped = None;
    for index in candidates {
        if report.requests >= settings.budget {
            stopped = Some(StopReason::Budget);
            break;
        }
        let path = rows[index].path.clone();
        let url = settings.mirror.url(&txt_path(&path));
        let etag = state.etag(&path).map(ToOwned::to_owned);
        let last_modified = state
            .last_modified(&path)
            .map(ToOwned::to_owned)
            .or_else(|| previous_entries.get(&path).map(|entry| entry.date.to_http()));
        pacer.pause();
        let response = source.get(&url, etag.as_deref(), last_modified.as_deref());
        report.requests += 1;
        match response {
            TxtResponse::Fetched {
                bytes,
                etag,
                last_modified,
            } => {
                consecutive_failures = 0;
                report.fetched += 1;
                let meta = parse_txt(&bytes);
                if meta.title.is_none() && meta.author.is_none() {
                    report.bad_txt += 1;
                }
                let row = &mut rows[index];
                row.entry.txt = true;
                row.entry.title = meta.title.unwrap_or_else(|| row.entry.stem.clone());
                row.entry.author = meta.author.unwrap_or_default();
                if let Some(game) = meta.game {
                    row.entry.game = game;
                }
                row.entry.port = meta.port;
                row.entry.maps = meta.maps;
                state.record(&path, today, AttemptOutcome::Ok, etag, last_modified);
            }
            TxtResponse::NotModified => {
                consecutive_failures = 0;
                if rows[index].entry.txt {
                    report.not_modified += 1;
                    state.record(&path, today, AttemptOutcome::Ok, etag, last_modified);
                } else {
                    // A 304 with no cached metadata is a mirror inconsistency:
                    // keep the filename fallback and try again next run.
                    report.bad_txt += 1;
                    state.record(&path, today, AttemptOutcome::Bad, None, None);
                }
            }
            TxtResponse::Missing => {
                consecutive_failures = 0;
                report.missing_txt += 1;
                let row = &mut rows[index];
                row.entry.txt = false;
                row.entry.title = row.entry.stem.clone();
                row.entry.author.clear();
                state.record(&path, today, AttemptOutcome::Missing, None, None);
            }
            TxtResponse::Blocked { status } => {
                stopped = Some(StopReason::Blocked(status));
                break;
            }
            TxtResponse::Failed { detail } => {
                tracing::warn!(url, %detail, "sidecar request failed");
                consecutive_failures += 1;
                report.bad_txt += 1;
                state.record(&path, today, AttemptOutcome::Bad, None, None);
                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    stopped = Some(StopReason::Network);
                    break;
                }
            }
        }
    }
    report.stopped = stopped;

    for row in &mut rows {
        row.entry.cacoward = settings.cacowards.get(&row.lower).copied();
        row.entry.in_catalog = settings.catalog.get(&row.lower).cloned();
        row.entry.latest_version = false;
    }
    mark_latest_versions(&mut rows);

    rows.sort_by(|a, b| {
        (a.dir.as_str(), a.entry.stem.as_str()).cmp(&(b.dir.as_str(), b.entry.stem.as_str()))
    });
    let mut entries: BTreeMap<String, Vec<Entry>> = BTreeMap::new();
    for row in rows {
        entries.entry(row.dir).or_default().push(row.entry);
    }

    let index = Index {
        schema: INDEX_SCHEMA,
        generated_at: format_unix_utc(settings.now_unix),
        source: IndexSource {
            index: settings.index_url.clone(),
            index_sha256: settings.index_sha256.to_ascii_lowercase(),
            mirror: settings.mirror.host.to_owned(),
        },
        min_ddl_version: settings.min_ddl_version.clone(),
        entries,
    };
    index.validate()?;
    // Enforce the 2 MiB bound here, not only in the CLI, so a build that would
    // publish an oversized file fails in CI with a clear reason.
    index.to_json()?;

    let paths: BTreeSet<String> = rows_paths(&index);
    state.retain(&paths);
    state.generated_at = format_unix_utc(settings.now_unix);
    state.schema = super::state::STATE_SCHEMA;

    report.entries = index.entry_count();
    report.directories = index.directories();
    Ok(BuildOutcome { index, report })
}

struct Row {
    dir: String,
    path: String,
    lower: String,
    entry: Entry,
    needs_fetch: bool,
}

fn row_for(
    inventory: InventoryEntry,
    previous: &BTreeMap<String, Entry>,
    report: &mut BuildReport,
) -> Row {
    let path = inventory.zip_path();
    let lower = path.to_ascii_lowercase();
    let reuse = previous.get(&path);
    let (entry, needs_fetch) = match reuse {
        Some(previous_entry) => {
            let unchanged = previous_entry.txt
                && previous_entry.size == inventory.size
                && previous_entry.date == inventory.date;
            if unchanged {
                report.reused += 1;
            }
            let mut entry = previous_entry.clone();
            // Refresh the inventory facts even when the upload changed: the
            // index always describes the listing it was built from.
            entry.date = inventory.date;
            entry.size = inventory.size;
            entry.stem.clone_from(&inventory.stem);
            (entry, !unchanged)
        }
        None => (
            Entry::fallback(
                &inventory.stem,
                inventory.date,
                inventory.size,
                game_for_dir(&inventory.dir),
            ),
            true,
        ),
    };
    Row {
        needs_fetch,
        dir: inventory.dir,
        path,
        lower,
        entry,
    }
}

fn flatten(previous: Option<&Index>) -> BTreeMap<String, Entry> {
    let mut entries = BTreeMap::new();
    if let Some(index) = previous {
        for (dir, entries_in_dir) in &index.entries {
            for entry in entries_in_dir {
                entries.insert(format!("{dir}/{}", entry.zip_name()), entry.clone());
            }
        }
    }
    entries
}

fn txt_path(zip_path: &str) -> String {
    zip_path
        .strip_suffix(".zip")
        .map_or_else(|| zip_path.to_owned(), |stem| format!("{stem}.txt"))
}

fn game_for_dir(dir: &str) -> IndexGame {
    if dir == "levels/doom2" || dir.starts_with("levels/doom2/") {
        IndexGame::Doom2
    } else if dir == "levels/doom" || dir.starts_with("levels/doom/") {
        IndexGame::Doom
    } else {
        IndexGame::Unknown
    }
}

/// Relevance rank for the crawl (R7): lower is fetched first.
fn crawl_rank(dir: &str, lower_path: &str, cacowards: &BTreeMap<String, u16>) -> u8 {
    if cacowards.contains_key(lower_path) {
        0
    } else if dir.split('/').any(|segment| segment.eq_ignore_ascii_case("megawads")) {
        1
    } else if !dir.split('/').any(|segment| segment.eq_ignore_ascii_case("Ports")) {
        2
    } else {
        3
    }
}

/// Flags the newest upload when several share a title (R8).
fn mark_latest_versions(rows: &mut [Row]) {
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        groups
            .entry(row.entry.title.trim().to_lowercase())
            .or_default()
            .push(index);
    }
    for members in groups.values() {
        if members.len() < 2 {
            continue;
        }
        let mut best = members[0];
        for &candidate in &members[1..] {
            let candidate_key = (
                rows[candidate].entry.date,
                rows[candidate].entry.size,
                rows[candidate].lower.as_str(),
            );
            let best_key = (
                rows[best].entry.date,
                rows[best].entry.size,
                rows[best].lower.as_str(),
            );
            if candidate_key > best_key {
                best = candidate;
            }
        }
        rows[best].entry.latest_version = true;
    }
}

fn rows_paths(index: &Index) -> BTreeSet<String> {
    index
        .entries
        .iter()
        .flat_map(|(dir, entries)| {
            entries
                .iter()
                .map(move |entry| format!("{dir}/{}", entry.zip_name()))
        })
        .collect()
}

/// A detail kept out of `StopReason` so tests can assert the reason alone.
pub const fn request_gap() -> std::time::Duration {
    MIN_REQUEST_GAP
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idgames::fetch::{NoPacer, TxtResponse, TxtSource, mirror_for};
    use crate::idgames::index::Port;
    use std::collections::VecDeque;

    /// A source that answers from a queue and records every call.
    #[derive(Default)]
    struct FakeSource {
        answers: BTreeMap<String, VecDeque<TxtResponse>>,
        calls: Vec<(String, Option<String>, Option<String>)>,
    }

    impl FakeSource {
        fn answer(&mut self, path_suffix: &str, response: TxtResponse) {
            let key = path_suffix.to_owned();
            self.answers.entry(key).or_default().push_back(response);
        }

        fn ok(&mut self, path_suffix: &str, body: &str) {
            self.answer(
                path_suffix,
                TxtResponse::Fetched {
                    bytes: body.as_bytes().to_vec(),
                    etag: Some(format!("\"{path_suffix}\"")),
                    last_modified: None,
                },
            );
        }

        fn urls(&self) -> Vec<String> {
            self.calls.iter().map(|(url, _, _)| url.clone()).collect()
        }
    }

    impl TxtSource for FakeSource {
        fn get(
            &mut self,
            url: &str,
            etag: Option<&str>,
            last_modified: Option<&str>,
        ) -> TxtResponse {
            self.calls
                .push((url.to_owned(), etag.map(ToOwned::to_owned), last_modified.map(ToOwned::to_owned)));
            let suffix = url.rsplit_once('/').map(|(_, name)| name).unwrap_or(url);
            if let Some(queue) = self.answers.get_mut(suffix) {
                if let Some(answer) = queue.pop_front() {
                    return answer;
                }
            }
            TxtResponse::Missing
        }
    }

    const INVENTORY: &str = "\
./levels/doom2/Ports/megawads:
-rw-rw-r--  1 idgmaint idgames  12771737 Jul 26  2021 dbp37_augzen.zip
-rw-rw-r--  1 idgmaint idgames  10370840 Oct 18 11:48 tistartan.zip
./levels/doom2/Ports/a-c:
-rw-rw-r--  1 idgmaint idgames   1000000 Jan  5  1999 abyspe65.zip
./levels/doom/a-c:
-rw-rw-r--  1 idgmaint idgames    500000 Feb  2  1996 old.zip
";

    fn settings() -> BuildSettings {
        BuildSettings::new(
            *mirror_for("youfailit.net").unwrap(),
            "https://youfailit.net/pub/idgames/ls-laR.gz",
            "502a7ccd3195452fb4955f077a078d0167bf80b721535be8c59e1daa269c2603",
            1_792_386_428,
        )
    }

    fn good_source() -> FakeSource {
        let mut source = FakeSource::default();
        source.ok(
            "dbp37_augzen.txt",
            "Title: AUGER;ZENITH\nAuthor: Doomer Boards Community\nGame: Doom 2\nAdvanced engine needed: MBF21\nMap #: 01-11\n",
        );
        source.ok(
            "tistartan.txt",
            "Title: Tistartan\nAuthor: Someone\nAdvanced engine needed: MBF21+UMAPINFO\nMap #: MAP01-MAP32\n",
        );
        source.ok(
            "abyspe65.txt",
            "Title: Abyspe\nAuthor: Another\nAdvanced engine needed: Boom\n",
        );
        source.ok(
            "old.txt",
            "Title: Old Times\nAuthor: Veteran\nEpisode and Level #: E1M1-E1M9\nAdvanced engine needed: None\n",
        );
        source
    }

    fn build(
        settings: &BuildSettings,
        previous: Option<&Index>,
        state: &mut CrawlState,
        source: &mut FakeSource,
    ) -> BuildOutcome {
        build_index(INVENTORY, settings, previous, state, source, &mut NoPacer).unwrap()
    }

    #[test]
    fn a_full_build_carries_the_fields_the_spec_lists() {
        let mut state = CrawlState::new(1_792_386_428);
        let mut source = good_source();
        let outcome = build(&settings(), None, &mut state, &mut source);
        let index = &outcome.index;
        assert_eq!(index.entry_count(), 4);
        assert_eq!(index.directories(), 3);
        assert_eq!(outcome.report.requests, 4);
        assert_eq!(outcome.report.fetched, 4);
        assert_eq!(outcome.report.stopped, None);

        let megawads = &index.entries["levels/doom2/Ports/megawads"];
        let augzen = megawads.iter().find(|entry| entry.stem == "dbp37_augzen").unwrap();
        assert_eq!(augzen.title, "AUGER;ZENITH");
        assert_eq!(augzen.author, "Doomer Boards Community");
        assert_eq!(augzen.game, IndexGame::Doom2);
        assert_eq!(augzen.port, Port::Mbf21);
        assert_eq!(augzen.maps, Some(11));
        assert!(augzen.txt);
        assert_eq!(augzen.date.to_iso(), "2021-07-26");
        assert_eq!(augzen.size, 12_771_737);

        let old = index.entries["levels/doom/a-c"]
            .iter()
            .find(|entry| entry.stem == "old")
            .unwrap();
        assert_eq!(old.game, IndexGame::Doom);
        assert_eq!(old.maps, Some(9));
        assert_eq!(old.port, Port::Unknown);
    }

    #[test]
    fn missing_and_bad_sidecars_keep_the_entry_with_a_filename_fallback() {
        let mut state = CrawlState::new(1_792_386_428);
        let mut source = FakeSource::default();
        source.ok("dbp37_augzen.txt", "no labelled fields at all\n");
        // tistartan and abyspe65 and old are Missing by default.
        let outcome = build(&settings(), None, &mut state, &mut source);
        assert_eq!(outcome.index.entry_count(), 4);
        assert_eq!(outcome.report.bad_txt, 1);
        assert_eq!(outcome.report.missing_txt, 3);
        let megawads = &outcome.index.entries["levels/doom2/Ports/megawads"];
        let tistartan = megawads.iter().find(|entry| entry.stem == "tistartan").unwrap();
        assert!(!tistartan.txt);
        assert_eq!(tistartan.title, "tistartan");
        assert_eq!(tistartan.author, "");
        assert_eq!(tistartan.port, Port::Unknown);
    }

    #[test]
    fn incremental_builds_fetch_only_added_or_changed_entries() {
        let mut state = CrawlState::new(1_792_386_428);
        let mut first = good_source();
        let first_outcome = build(&settings(), None, &mut state, &mut first);
        assert_eq!(first.calls.len(), 4);

        let mut second = good_source();
        let second_outcome = build(
            &settings(),
            Some(&first_outcome.index),
            &mut state,
            &mut second,
        );
        assert_eq!(second.calls.len(), 0, "unchanged entries must not be re-fetched");
        assert_eq!(second_outcome.report.reused, 4);
        assert_eq!(second_outcome.index, first_outcome.index);

        // Change one upload's size: only that entry is fetched again.
        let changed = INVENTORY.replace("   1000000 Jan  5  1999 abyspe65.zip", "   2000000 Jan  5  1999 abyspe65.zip");
        let mut third = good_source();
        let third_outcome = build_index(
            &changed,
            &settings(),
            Some(&first_outcome.index),
            &mut state,
            &mut third,
            &mut NoPacer,
        )
        .unwrap();
        assert_eq!(third.calls.len(), 1);
        assert!(third.urls()[0].ends_with("/abyspe65.txt"));
        assert_eq!(third_outcome.report.reused, 3);
    }

    #[test]
    fn a_304_keeps_the_previous_metadata() {
        let mut state = CrawlState::new(1_792_386_428);
        let mut first = good_source();
        let first_outcome = build(&settings(), None, &mut state, &mut first);

        // The upload changed date but the sidecar says 304: keep the old fields.
        let changed = INVENTORY.replace("Jul 26  2021 dbp37_augzen.zip", "Jul 27  2021 dbp37_augzen.zip");
        let mut second = FakeSource::default();
        second.answer("dbp37_augzen.txt", TxtResponse::NotModified);
        let second_outcome = build_index(
            &changed,
            &settings(),
            Some(&first_outcome.index),
            &mut state,
            &mut second,
            &mut NoPacer,
        )
        .unwrap();
        assert_eq!(second.calls.len(), 1);
        assert_eq!(second_outcome.report.not_modified, 1);
        let augzen = second_outcome.index.entries["levels/doom2/Ports/megawads"]
            .iter()
            .find(|entry| entry.stem == "dbp37_augzen")
            .unwrap();
        assert_eq!(augzen.title, "AUGER;ZENITH");
        assert_eq!(augzen.date.to_iso(), "2021-07-27");
    }

    #[test]
    fn the_budget_stops_the_run_but_keeps_every_entry() {
        let mut state = CrawlState::new(1_792_386_428);
        let mut settings = settings();
        settings.budget = 2;
        let mut source = good_source();
        let outcome = build(&settings, None, &mut state, &mut source);
        assert_eq!(outcome.report.requests, 2);
        assert_eq!(outcome.report.stopped, Some(StopReason::Budget));
        assert_eq!(outcome.index.entry_count(), 4);
        // Two fetches per day, four entries: tomorrow's run continues.
        assert_eq!(
            outcome
                .index
                .entries
                .values()
                .flatten()
                .filter(|entry| entry.txt)
                .count(),
            2
        );
    }

    #[test]
    fn a_blocked_response_stops_and_keeps_what_was_fetched() {
        let mut state = CrawlState::new(1_792_386_428);
        let mut source = FakeSource::default();
        // Crawl order: tistartan (2026), then dbp37_augzen (2021), then abyspe65.
        source.ok("tistartan.txt", "Title: First\nAuthor: A\n");
        source.ok("dbp37_augzen.txt", "Title: Second\nAuthor: B\n");
        source.answer("abyspe65.txt", TxtResponse::Blocked { status: 429 });
        let outcome = build(&settings(), None, &mut state, &mut source);
        assert_eq!(outcome.report.stopped, Some(StopReason::Blocked(429)));
        assert_eq!(outcome.report.fetched, 2);
        assert_eq!(outcome.index.entry_count(), 4);
        // The blocked entry is not recorded as attempted, so tomorrow retries it.
        assert!(state.retry_allowed(
            "levels/doom2/Ports/a-c/abyspe65.zip",
            Date::from_unix(1_792_386_428)
        ));
    }

    #[test]
    fn repeated_network_failures_stop_the_run() {
        let mut state = CrawlState::new(1_792_386_428);
        let mut source = FakeSource::default();
        for suffix in ["dbp37_augzen.txt", "tistartan.txt", "abyspe65.txt", "old.txt"] {
            source.answer(
                suffix,
                TxtResponse::Failed {
                    detail: "timed out".into(),
                },
            );
        }
        let outcome = build(&settings(), None, &mut state, &mut source);
        assert_eq!(
            outcome.report.stopped,
            Some(StopReason::Network),
            "three consecutive failures must stop a run"
        );
        assert_eq!(outcome.report.requests, MAX_CONSECUTIVE_FAILURES);
    }

    #[test]
    fn crawl_order_puts_cacowards_then_megawads_first() {
        let mut state = CrawlState::new(1_792_386_428);
        let mut settings = settings();
        settings.cacowards.insert(
            "levels/doom/a-c/old.zip".to_owned(),
            1999,
        );
        let mut source = good_source();
        let outcome = build(&settings, None, &mut state, &mut source);
        let urls = source.urls();
        assert!(urls[0].ends_with("/levels/doom/a-c/old.txt"), "{urls:?}");
        assert!(
            urls[1].contains("/levels/doom2/Ports/megawads/"),
            "{urls:?}"
        );
        assert!(
            urls[2].contains("/levels/doom2/Ports/megawads/"),
            "{urls:?}"
        );
        assert!(urls[3].ends_with("/levels/doom2/Ports/a-c/abyspe65.txt"));
        let old = outcome.index.entries["levels/doom/a-c"]
            .iter()
            .find(|entry| entry.stem == "old")
            .unwrap();
        assert_eq!(old.cacoward, Some(1999));
    }

    #[test]
    fn ranking_signals_come_from_the_catalog_and_the_titles() {
        let mut state = CrawlState::new(1_792_386_428);
        let mut settings = settings();
        settings.catalog.insert(
            "levels/doom2/ports/megawads/dbp37_augzen.zip".to_owned(),
            "auger-zenith".to_owned(),
        );
        let mut source = good_source();
        let outcome = build(&settings, None, &mut state, &mut source);
        let augzen = outcome.index.entries["levels/doom2/Ports/megawads"]
            .iter()
            .find(|entry| entry.stem == "dbp37_augzen")
            .unwrap();
        assert_eq!(augzen.in_catalog.as_deref(), Some("auger-zenith"));
        assert!(!augzen.latest_version);
    }

    #[test]
    fn the_newest_upload_wins_when_titles_collide() {
        let inventory = "\
./levels/doom2/a-c:
-rw-rw-r--  1 idgmaint idgames 1000 Jan  5  1999 twin.zip
-rw-rw-r--  1 idgmaint idgames 1000 Jan  5  2001 twin2.zip
";
        let mut settings = settings();
        settings.budget = 10;
        let mut state = CrawlState::new(1_792_386_428);
        let mut source = FakeSource::default();
        source.ok("twin.txt", "Title: Twin\nAuthor: A\n");
        source.ok("twin2.txt", "Title: Twin\nAuthor: A\n");
        let outcome = build_index(
            inventory,
            &settings,
            None,
            &mut state,
            &mut source,
            &mut NoPacer,
        )
        .unwrap();
        let entries = &outcome.index.entries["levels/doom2/a-c"];
        let twin = entries.iter().find(|entry| entry.stem == "twin").unwrap();
        let twin2 = entries.iter().find(|entry| entry.stem == "twin2").unwrap();
        assert!(!twin.latest_version);
        assert!(twin2.latest_version);
    }

    #[test]
    fn a_second_build_from_the_same_inputs_is_byte_identical() {
        let mut state = CrawlState::new(1_792_386_428);
        let mut source = good_source();
        let first = build(&settings(), None, &mut state, &mut source);
        let mut source = good_source();
        let second = build(&settings(), None, &mut state, &mut source);
        assert_eq!(
            first.index.to_json().unwrap(),
            second.index.to_json().unwrap()
        );
    }

    #[test]
    fn an_empty_inventory_is_a_clear_error() {
        let mut state = CrawlState::new(0);
        let error = build_index(
            "./incoming:\n-rw-rw-r-- 1 a b 10 Oct 18 11:48 x.zip\n",
            &settings(),
            None,
            &mut state,
            &mut FakeSource::default(),
            &mut NoPacer,
        )
        .unwrap_err();
        assert!(matches!(error, BuildError::EmptyInventory));
    }

    #[test]
    fn state_is_stamped_and_pruned_to_the_inventory() {
        let mut state = CrawlState::new(0);
        state.record(
            "levels/doom2/a-c/gone.zip",
            Date::new(2026, 1, 1).unwrap(),
            AttemptOutcome::Missing,
            None,
            None,
        );
        let mut source = good_source();
        let outcome = build(&settings(), None, &mut state, &mut source);
        assert_eq!(state.len(), 4);
        assert!(!state.attempts.contains_key("levels/doom2/a-c/gone.zip"));
        assert_eq!(state.generated_at, outcome.index.generated_at);
    }

    #[test]
    fn the_request_gap_is_the_d031_minimum() {
        assert_eq!(request_gap(), std::time::Duration::from_secs(3));
    }
}

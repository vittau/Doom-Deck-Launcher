# 116 — Per-mapset progress from DSDA-Doom's stats

## Outcome

Implemented spec 116 R1–R8 for the selected profile: DDL derives DSDA-Doom's organized
`stats.txt` path from the resolved launch, parses it with a strict bounded reader, keeps *no
stats yet* and *unavailable* distinct from *zero completed*, and shows `N/M maps` on Library
cards plus a short summary and per-map table on Mapset detail. Abandon excludes its four-thing
ending map through the new `[progress] exclude_maps` recipe field, and the catalog bundle's
`min_ddl_version` moves to `0.0.20` for that capability. A5 remains **Needs Deck Validation**.

## Changes

### Core (`crates/ddl-core/src/progress/`)

- **`stats_path` (R1)** follows the engine organizer at
  [data_organizer.c v0.30.1](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/data_organizer.c#L83-L139):
  `dsda_doom_data`, the lowercase IWAD stem, then the stems of up to the first eight `-file`
  entries ending in `.wad` (case-insensitive), then `stats.txt`. Patches never contribute; the
  layout type takes the `-deh` list anyway so callers pass the whole launch. A non-`.wad` entry
  in `-file` (for example a ZIP, whose engine load order the research has not verified), an
  unusable stem or an unusable IWAD stem yields an error the caller maps to *unavailable*
  rather than a guessed path.
- **Safe parser (R2)** in `progress/parse.rs`: 1 MiB cap (the reader takes `MAX + 1` to detect
  overflow), 4,096 rows, 512-byte lines, exactly version `1`, exactly two headers and 15
  space-separated tokens per row, checked `int`-range integers, `-1` only on the documented
  unknown fields, skill `0`–`5`, printable-ASCII map IDs of at most eight bytes, no duplicate
  IDs, LF or CRLF, and a terminated final row. Any violation rejects the whole file.
- **Contained read (R2)**: `read` treats a missing file as *no stats yet*, refuses a
  non-regular file, canonicalizes the profile data root and the target, refuses a target outside
  the root (symlink escape), and opens the canonical target with a bounded read. Nothing is ever
  written, repaired or deleted.
- **Engine gate (R3)**: `crates/ddl-core/data/engines.toml` gains an optional `stats_version`
  per reviewed release, set to `1` only for `v0.30.1` (audited in
  `docs/research/dsda-stats.md`). `ReviewedList::audits_stats` refuses every other version, and
  loading rejects a `stats_version` this build does not understand, so a future release needs an
  explicit audit. This is the allowlist beside the engine metadata the brief asked for.
- **Progress rule (R4)**: `Stats::progress(exclude_maps)` = parsed rows minus exclusions present
  in the file; completed = those rows with `best_skill > 0`; each map carries its ID,
  completion, `total_exits` and highest recorded skill (`None` when 0). `Request::read` combines
  the gate, path, read, parse and rule into one `Progress`: `Recorded`, `NoStatsYet` or
  `Unavailable` (ambiguous layout, unsupported engine, missing engine, unresolved launch,
  read error, invalid file).
- `iwads::CanonicalLinks::link_path` exposes the canonical IWAD link name without creating or
  verifying it, and `iwads::base_games::selected_iwad` is the shared copy-selection helper the
  base-game plan now uses too.

### Recipe, catalog and capability gate (R5)

- `Recipe` gains an optional `#[serde(default)] progress: Option<ProgressConfig>` with
  `exclude_maps`; unknown keys inside `[progress]` remain errors (`deny_unknown_fields`), and
  machine-written recipe copies read/write the field through `catalog::persisted`.
- Catalog lint accepts uppercase `MAPxx` / `ExMy` IDs once each with diagnostics
  `progress.exclude_maps.N`; it rejects lowercase, missing digits, extra characters and
  duplicates.
- `catalog/abandon.toml` declares `[progress] exclude_maps = ["MAP18"]`, citing its existing
  inspection evidence that MAP18 is a four-thing ending map (`HUSTR_18 = The End`).
- `DEFAULT_MIN_DDL_VERSION` is raised from `0.0.12` to `0.0.20` (spec 090/098 capability-gate
  pattern, D-038): a pre-0.0.20 reader refuses recipe fields it does not know, so the bundle gate
  must refuse the document whole instead of silently dropping Abandon.
- `CONTRIBUTING.md` documents the new optional table, its evidence requirement and the rule.

### Services and UI (R6/R7)

- `Services::snapshot` computes `Item::progress`, `BaseItem::progress` and
  `Snapshot::custom_progress` for the selected profile from the launch's own file list. The
  Library snapshot is built outside play, after the engine is reaped, and the detail view reads
  that snapshot; there is no cache, no extra hashing and no read while a game runs.
- Library cards (catalog mapsets, base games, custom setups) show `N/M maps` only when a valid
  file was read; list rows include the same count. Catalog cards reserve one extra text row
  (410→448 px); base-game cards 32 px taller; custom cards the same extra row. Discover keeps
  its exact geometry.
- Mapset detail shows a `Progress` section: `N/M maps completed`, then a plain table with
  `Map`, `Done`, `Exits` and `Highest skill` columns (missing values render `—`). The table is
  read-only content — actions keep focus and the right stick scrolls, as on other long detail
  content. *No stats yet* and *unavailable* each show one plain sentence, and no stats is never
  rendered as `0/N`.
- New user-facing copy lives in `crates/ddl/ui/wiring-strings.slint` (`@tr`) and
  `wiring_copy.rs`; a two-placeholder `pair` helper sets `{}/{}` without concatenating
  fragments. Amber remains focus-only; no decorative subtitles were added.

No dependency was added.

## Verification

Merged local `main` (`06b42ec`, "Decide and specify per-mapset progress…") before the final
checks; it was already up to date. All Rust builds used `--jobs 2`, one Cargo build at a time.

| Check | Result / real output |
| --- | --- |
| `cargo fmt --all --check` | Exit 0, no output. |
| `cargo clippy --workspace --all-targets --jobs 2 -- -D warnings` | Exit 0: `Finished dev profile [unoptimized + debuginfo] target(s) in 16.90s`; no warnings. |
| `cargo test -p ddl-core --jobs 2` | `673 passed; 0 failed` in the core unit group, plus every integration group (0 failures). |
| `cargo test -p ddl --jobs 2 services::progress_tests` | `2 passed; 0 failed`: the launch's own file list, Abandon's 12/17 rule, engine gate and base-game canonical path. |
| `cargo test -p ddl --jobs 2 "headless::library::"` | All Library journey and render tests pass, including the three new progress cases. |
| `target/debug/ddl catalog lint catalog` | Exit 0: `Catalog lint passed (offline).` |
| `cargo test --workspace --locked --jobs 2 -- --skip headless::journeys::title_fit_ --skip headless::journeys::categories_favorites_and_screenshots` | Exit 0: **1,044 passed**, 0 failed, 0 ignored across 23 unit/integration/doctest groups; DDL unit group `262 passed; 0 failed; 10 filtered out`, core unit group `673 passed; 0 failed`. |

### Core evidence (A1)

`crates/ddl-core/src/progress/tests.rs` covers every R8 fixture: the 18-row Abandon fixture with
12 rows at `best_skill > 0` and MAP18 excluded → **12/17**; zero/unknown values; LF vs CRLF;
short and long rows; malformed and overflow tokens; out-of-range skill/exit/time; version 2 and
missing headers; unterminated final row and blank lines; duplicate, over-long and non-ASCII IDs;
row/byte/line limits; catalog/base/custom/extras paths; nine WADs (first eight); DEH ignored;
mixed case; missing file → *no stats yet*; invalid file and unaudited engine → *unavailable*;
oversize file; directory instead of a file; symlink escape refused and a contained symlink read.
The path derivation is also checked in `crates/ddl/src/services.rs` against the real Abandon
recipe and a base game with an extra WAD plus a patch.

### Lint evidence (A2)

`catalog::tests::progress_exclusions_accept_map_lump_ids_once` reads the shipped Abandon recipe
and asserts its exclusion lints clean, then rejects `map18`, `MAP1`, `MAP018` and a duplicate
`MAP18` with the expected keys, and confirms unknown keys inside `[progress]` fail to parse. The
bundle tests assert the new `0.0.20` floor refuses a `0.0.19` client whole and accepts
`0.0.20`. The real catalog lints clean (table above).

### Headless evidence (A3)

Rendered through Slint's `MinimalSoftwareWindow` and software renderer at 1280×800; no window
was opened and no screen was captured. The new cases live in
`crates/ddl/src/headless_library.rs` and write:

- `target/headless/library_card_shows_recorded_progress/116-library-progress.png` — Abandon's
  Library card with `12/17 maps`, inside the 28 px card inset.
- `target/headless/detail_renders_a_short_and_a_long_scrolled_table/116-detail-short.png` —
  detail with a four-row table (`3/4 maps completed`, MAP13 completed no with `—` skill).
- `target/headless/detail_renders_a_short_and_a_long_scrolled_table/116-detail-long-scrolled.png`
  — all eighteen detected rows after a right-stick scroll (`12/18 maps completed`).
- `target/headless/detail_distinguishes_no_stats_yet_from_unavailable/116-detail-no-stats.png` —
  one plain line, no `0/N`.
- `target/headless/detail_distinguishes_no_stats_yet_from_unavailable/116-detail-unavailable.png`
  — one plain line for an unaudited engine.

All five PNGs were opened and inspected: columns stay aligned, the table scrolls, focus remains
on the actions and amber is only the focus ring. The owner/coordinator must review them before
merge (visible UI change, A3).

### Deck (A5) — Needs Deck Validation

Not run on hardware; the macOS source audit cannot certify the AppImage sandbox. Use the checks
listed in `docs/reports/115-dsda-stats-research.md` (“Deck checks required before delivery”),
in particular:

- Finish a map in an installed mapset, quit DSDA-Doom normally, and confirm the Library card and
  Mapset detail both update on return; force-quit and confirm the last written file is shown or
  the state stays honest.
- Play the same mapset in two profiles and confirm their counts stay separate; add an extra WAD
  and confirm the card switches to a separate, initially empty progress.
- Confirm the packaged Flatpak/extracted AppRun actually writes `stats.txt` where DDL reads it,
  and that an unwritable `-data` directory shows *unavailable* rather than a crash.
- Compare normal exit, controlled engine error and force-stop; corrupt/truncate a disposable
  fixture and confirm *unavailable*; check the right stick scrolls an 18-row table on the Deck.
- Confirm the wording is understandable at handheld distance and that no-monsters exits are not
  presented as completed maps.

### Acceptance coverage

- **A1 passed:** the R8 unit tests above pass; the 18-row fixture with 12 rows at
  `best_skill > 0` yields `12/17` with Abandon's exclusion in both the core test and the
  services wiring test.
- **A2 passed:** lint accepts Abandon's `exclude_maps` and rejects malformed/duplicate IDs; the
  bundle's `min_ddl_version` is raised to `0.0.20`; the real catalog lints clean.
- **A3 passed (worker evidence):** the five PNGs above were rendered headlessly and inspected;
  owner/coordinator review is still required before merge.
- **A4 passed:** `cargo fmt --all --check`, denied-warning workspace Clippy and the ordinary test
  command above all pass.
- **A5 — Needs Deck Validation:** the checks listed above must be run on hardware.

## Open questions

- **Detail placement.** Progress sits below the title/description/profile block, so at the
  default scroll only the summary is near the fold; the table needs a right-stick scroll. An
  alternative is to move the summary above the description. Left as implemented pending the
  owner's look at the PNGs.
- **Detected-map qualifier.** The card and summary say `N/M maps` without the research's
  “detected maps” qualifier, to keep the card line short. Report 115 recommended the qualifier;
  if the owner wants it, it belongs in the detail summary only.
- **Base/custom detail tables.** R7 asks for the table on Mapset detail only; base-game and
  custom setup cards show the count but their setup screens do not show a table yet.
- **Exclusion candidates noticed (not added, per the brief):** `arrival` documents MAP10 as the
  “1 ending” map of its eleven slots (`catalog/arrival.toml` limitations/inspected) and `haste`
  documents MAP17 as the authors' “Thanks for Playing” ending map (`catalog/haste.toml` notes).
  Both would need an owner/coordinator decision and a Deck check before a recipe change;
  `overboard`'s MAP14 is the author's “actual end” but is an ordinary playable map, so it is not
  a candidate.
- **Stale same-stem stats.** The format has no content identity; replacing a WAD's bytes while
  keeping the file name can inherit old progress. DDL shows what the engine wrote (D-045);
  suppression would need a separately approved cache/identity design and is out of scope.
- **Coordinator-owned docs.** Propose adding spec 116/A5 to `docs/status.md`,
  `docs/deck-testing.md` and `docs/journal.yml`; this worker did not edit them.

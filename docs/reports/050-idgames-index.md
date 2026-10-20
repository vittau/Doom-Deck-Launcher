# 050 — `/idgames` index build and publish

**Status:** implemented; full-workspace checks pass on the merged branch. A3 (first publish) is left
to the coordinator after the owner's approval; nothing was published from this branch.

**Spec:** `docs/specs/050-idgames-index.md` (R1–R9, including R9 "crawl telemetry and back-off"
added on `main` while this task ran). **Decision:** D-031.

## What changed

### Core (`crates/ddl-core/src/idgames/`)

| Module | Responsibility |
| --- | --- |
| `inventory.rs` | Gzip decode (8 MiB compressed / 32 MiB text bounds) and parse of `ls-laR.gz` or `fullsort.gz`; keeps only `levels/doom` and `levels/doom2`; counts `deathmatch`, `incoming/`, `newstuff/`, non-DOOM game trees, non-zips and unparsed lines; infers the year of recent `ls` dates; refuses unsafe path segments |
| `txt.rs` | 64 KiB-bounded sidecar parse; UTF-8, then Windows-1252 with the five undefined bytes from CP437; `Title`, `Author`, `Game`, `Episode and Level #`, `Advanced engine needed`, `May Not Run With`, `Primary purpose`, `Map #`; port family per research §6.4 (never scrapes free-text complevel); map ranges (`MAP01-MAP32`, `01-11`, `E1M1-E1M9`) |
| `index.rs` | Schema 1 (`schema`, `generated_at`, `source`, `min_ddl_version`, `entries` grouped by directory), 12-slot entry arrays, deterministic order, validation, 2 MiB bound, atomic write |
| `build.rs` | Incremental builder, relevance crawl order, budget, R6 stopping, R9 telemetry and back-off |
| `fetch.rs` | Mirror table with per-host base paths, bounded conditional GET, `Clock` seam, ≥ 3 s gap + jitter, UA |
| `state.rs` | Resumable attempts + validators, run telemetry (last 30 runs), 24 h / halving back-off |
| `cacowards.rs` | Cacoward year/path map parsed from `docs/research/cacowards.md` |
| `catalog_map.rs` | `idgames_path` → recipe id read from `catalog/*.toml` as untyped TOML |

`index.json` carries exactly the facts R3/R8 list — stem, title, author, date, size, base game, port
class, txt flag, Cacoward year, recipe id, map count, latest-version flag — and **no**
descriptions, credits, ratings or txt bodies (asserted in tests).

### CLI

```text
ddl idgames index build --ls <file> --mirror <host> --out <file>
    [--previous <old.json>] [--state <file>] [--cacowards <md>] [--catalog <dir>]
    [--budget <1-1500>] [--min-ddl-version <v>]
ddl idgames index validate <file>
```

The builder refuses non-mirror hosts (including the master), writes the index and the crawl state
atomically, prints counts and the R9 telemetry, and exits non-zero after writing both files when
the mirror stopped the crawl. A back-off wait exits 0 (a healthy no-op day).

### CI (`.github/workflows/idgames-index.yml`)

Daily at 04:17 UTC plus `workflow_dispatch`; downloads `ls-laR.gz` from `youfailit.net`; restores
the previous index and state from `idgames-index`; builds and validates; appends the build output
(telemetry) to the job summary; commits `index.json` and `crawl-state.json` to the orphan branch as
one ordinary commit. `contents: write` and `issues: write` are granted to this job only; the
workflow default is `contents: read`. A stop publishes the partial index/state, then creates or
comments on one `idgames-crawl` issue and fails the job. A hard failure leaves the branch untouched.

### Tests and fixtures

`crates/ddl-core/tests/idgames_index.rs` (8 tests) and `crates/ddl-core/tests/fixtures/idgames/`
(`ls-laR.txt`, `ls-laR.gz`, `fullsort.txt`, six sidecars including a CP1252 file and a free-form
one). Unit tests live beside each module. No test uses a socket.

## Requirements

- **R1** — both inventory formats, exact exclusion rules, year inference for recent dates,
  day precision, server-local caveat documented. A1.
- **R2** — bounded GET (64 KiB), `ETag`/`If-Modified-Since` from the crawl state, UTF-8 →
  Windows-1252/CP437 fallback, all listed fields, filename fallback and a count for bad/missing
  txts, never fatal. A1 and the conditional-request assertion in `a_304_keeps_the_previous_metadata`.
- **R3** — schema/source/`min_ddl_version`, 12-slot arrays grouped by directory, deterministic
  ordering, no bodies/credits/ratings, ≤ 2 MiB enforced before writing, atomic write.
- **R4** — both subcommands; unchanged path+size+date entries reuse the previous index without a
  request; the fake source counts requests.
- **R5** — workflow above; publish is a separate step after validation, so a failure keeps the last
  good index; no force push.
- **R6** — one mirror, one request in flight, ≥ 3 s + jitter between starts, 1,500 requests per
  run, descriptive UA with the repo URL, stop on 403/429/5xx, everything fetched is kept, state is
  resumable and committed with the index.
- **R7** — crawl order: Cacoward winners, then `*/megawads/`, then the classic (non-`Ports`) tree,
  then `Ports` single maps; newest first inside each rank.
- **R8** — `cacoward`, `in_catalog`, `maps`, `latest_version` on each entry.
- **R9** — `crawl-state.json` records the last 30 runs (requests, bytes, status counts, min/mean
  spacing, retries, stop) and the back-off. A mirror stop fails the job and opens/updates one
  `idgames-crawl` issue after publishing; the next run waits 24 h and halves the budget (floor 100)
  until three clean runs pass. No request is retried inside a run.

## Acceptance criteria

- **A1** — covered by the integration tests and module unit tests listed above; no network. Green.
- **A2** — `cli_exec::tests::idgames_index_build_then_validate_round_trips_without_network`
  (offline previous index, build with zero candidates, validate, broken-document refusal) and the
  incremental counting tests. Green.
- **A3** — the workflow was reviewed against the `checks` job's toolchain setup and the
  catalog-bundle publishing model. **The first publish was not triggered** (ownership: the
  coordinator after the owner's approval), so the size/entry count is still to be recorded from the
  run summary.
- **A4** — full-workspace checks green on the merged branch (`main` at the 0.0.7 release was
  merged before this run): `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --
  -D warnings`, `cargo test --workspace` (exit 0; 13 suites, including 74 `idgames` unit tests, 8
  `idgames_index` integration tests and 3 CLI tests). Targeted runs during development were also
  green for `ddl-core` and `ddl`.
- **A5** — virtual-clock build tests: min spacing ≥ 3,000 ms (mean < 4,000 ms with jitter); budget
  holds; 403/429/503 stops record statuses and keep the partial index; the next run makes no
  request inside 24 h and then uses half the budget; three clean runs restore it; a budget stop
  counts as clean; no URL is requested twice. `state::tests` cover the floor, recovery, bounding
  and the `summary()` used in the job summary and the issue.

## New dependencies

- **`flate2` `=1.1.10`** with `default-features = false`, `features = ["rust_backend"]`: it decodes
  `ls-laR.gz`/`fullsort.gz`. Pure Rust, so there is no C zlib or toolchain requirement on the Deck,
  and it was already in `Cargo.lock` through `zip`'s deflate support. Maintained by the
  `flate2-rs` project.
- Nothing else: HTTP reuses `ureq` (already reviewed), hashing reuses `sha2` through
  `downloads::Sha256`, state uses `serde_json`, recipes use `toml`. The Windows-1252/CP437 decoder
  and the LCG jitter are small hand-written tables/functions rather than new crates; dates are a
  self-contained civil-date implementation (no `chrono`/`time`).

## Open questions

1. **"Episode-sized" is a proxy (R7).** The map count is only known after a sidecar is fetched, so
   rank 3 is "not under `Ports/`". Ranking by the previous index's real `maps` is a possible
   follow-up; it only helps from the second run on.
2. **Special Cacoward awards** (Mordeth, Mockaward, multiplayer, …) use a different table in the
   research document and are not read; only the per-edition Top Ten/runner-up tables are. If spec
   051 should treat them as winners, that is a parser extension, not a data change.
3. **`min_ddl_version`** is deliberately `0.0.6` (the first consumer is spec 051), like the
   catalog bundle's constant; raising it when the schema changes is a one-line change.
4. **Two files on the branch.** R5 names `index.json`; R6/R9 require the crawl state next to it, so
   `idgames-index` holds exactly `index.json` and `crawl-state.json`. Spec 051 should ignore the
   second file.
5. **Back-off window semantics.** The 24 h window is measured from the stop's finish time; runs in
   the window make zero requests but still rebuild and publish. A run that only waits does not
   count as clean, so three *fetching* days are needed to recover.
6. **Budget is per run.** The CLI says "requests this run"; the workflow is daily, so it matches
   the daily budget. A manual re-run on the same day gets a fresh (possibly halved) budget.
7. **Mirror choice.** `youfailit.net` (`/pub/idgames`) is the workflow's mirror; the research
   measured it with a published rate limit. `files.xvertigox.com`, `planetzdoom.com` (redirects to
   an unallowlisted host) and the master are refused by the builder's table.
8. **Encoding.** Windows-1252 is the primary non-UTF-8 fallback (the research sample's `0xFC` is
   `ü`), with CP437 only for the five bytes Windows-1252 leaves undefined. A true CP437 text with
   box drawing would show the Windows-1252 glyphs; no sample had one.
9. **`in_catalog` duplicates.** If two recipes pin the same `idgames_path`, the first file name in
   sorted order wins. The catalog is linted against duplicate package paths elsewhere; this map
   only feeds a hint.
10. **`latest_version`** groups by exact lowercased title. Uploads that differ only in punctuation
    are not merged; that needs spec 051's ranking rules, not the index.

## Proposed (not edited here)

- `docs/status.md`: mark spec 050 **Done** after the first publish, with the recorded index size and
  entry count; A3 stays open until then.
- `docs/journal.yml`: one entry when the first index is published (size, entries, mirror, crawl
  telemetry).
- `docs/decisions.md` already carries the owner's 2026-10-20 approval in D-031; no change needed.

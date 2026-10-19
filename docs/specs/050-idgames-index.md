# 050 — `/idgames` index build and publish

**Status:** Ready (first index publish waits for the owner: spec 041 report Q1, Q5, Q6)  ·  **Milestone:** M4  ·  **Depends on:** 006, 019, 041
**Origin:** spec 041 research (`docs/research/idgames-browse.md`, report R5), 2026-10-19; owner: advance M4 while Deck testing waits.

**Goal:** one small, versioned index that gives DDL titles, authors, dates, sizes, base game and a
port hint for the archive, built from `ls-laR.gz` + sidecar txts, without the Doomworld API.

## Requirements

- **R1** — `ddl-core::idgames` builder: parse `ls-laR.gz` (and accept `fullsort.gz` as an
  alternative inventory); keep only `levels/doom` and `levels/doom2`; drop `/deathmatch/`,
  `incoming/`, `newstuff/`, heretic/hexen/strife/hacx/doom64; parse the lossy `ls` date (year
  inference for recent files, server-local, day precision).
- **R2** — fetch the same-stem `.txt` for each entry with a bounded GET (64 KiB), ETag/
  `If-Modified-Since`, UTF-8 → Windows-1252/CP437 fallback; parse `Title`, `Author`, `Game`/
  `Episode and Level #`, `Advanced engine needed`, `May Not Run With`, `Primary purpose`,
  `Map #`; a bad/missing txt keeps the entry with a filename fallback and is counted, never fatal.
- **R3** — output `index.json`: `schema`, `generated_at`, `source {index URL, index sha256, mirror}`,
  `min_ddl_version`, `entries` grouped by directory (compact arrays or short keys), carrying stem,
  title, author, date, size, game, port class and txt flag. Deterministic ordering; **no
  descriptions, credits, ratings or txt bodies**; ≤ 2 MiB; atomic write.
- **R4** — CLI: `ddl idgames index build --ls <file> --mirror <host> --out <file>
  [--previous <old.json>]`, `ddl idgames index validate <file>`. Incremental mode re-fetches only
  added/changed entries (path+size+date).
- **R5** — workflow `.github/workflows/idgames-index.yml`: daily schedule + `workflow_dispatch`;
  builds/validates, publishes `index.json` to the `idgames-index` branch as one ordinary commit,
  keeps the last good index on failure, `contents: write` only for this job; no force push.
- **R6** — politeness: one mirror at a time, ≤ 4 requests in flight, `DoomDeckLauncher/<version>`
  UA, stop on 403/429/5xx, never crawl the master for the bulk pass.

## Acceptance criteria

- [ ] **A1** (R1–R3) — fixture `ls-laR` snippets + txt fixtures: exclusion rules, recent-date year
  inference, missing/malformed/non-UTF-8 txt, field extraction, schema round-trip, size bound,
  deterministic order. No network.
- [ ] **A2** (R4) — CLI build/validate round-trip; incremental build fetches only the diff (fake
  fetcher counts requests).
- [ ] **A3** (R5) — workflow reviewed; first publish triggered by the coordinator after merge and the
  resulting index size/entry count recorded.
- [ ] **A4** — fmt, clippy `-D warnings`, tests green.

# 051 — `/idgames` browse core (fetch, cache, search)

**Status:** Ready (after 050)  ·  **Milestone:** M4  ·  **Depends on:** 006, 019, 050
**Origin:** spec 041 research (`docs/research/idgames-browse.md`, report R5), 2026-10-19; owner: advance M4 while Deck testing waits.

**Goal:** the app can list, search and page the archive offline from the cached index, and fetch a
single `.txt` on demand for the entry page.

## Requirements

- **R1** — typed schema + validation: known `schema`, `min_ddl_version` ≤ running version,
  `generated_at` parseable, entries sane (path under `levels/doom{,2}/`, no `..`, size > 0, hash
  format when present), ≤ 2 MiB; malformed index rejected whole.
- **R2** — fetch `…/idgames-index/index.json` over HTTPS with the existing download stack; ETag +
  `If-Modified-Since`; cache atomically at `<XDG data>/ddl/idgames/index.json` with fetch time;
  background refresh ≤ once per 24 h; manual refresh; a failed refresh keeps the old index; a
  corrupt cache is ignored and logged.
- **R3** — search API: case/diacritic-insensitive matching over title, author and filename
  (2-character minimum), filters (game, era, port class, txt availability), sort (**relevance**
  default, newest, title, size), offset/limit paging with a deterministic order; pure `ddl-core`,
  no network. **Relevance (D-031):** exact title, then title prefix, then title word, then author,
  then filename; ties broken by quality signals from the index (Cacoward winner, in the DDL
  catalog, map count, latest version), then newest. With no query, the default list is
  "Highlights" in the same signal order.
- **R4** — entry detail: on-demand `.txt` fetch (64 KiB, ETag, encoding fallback) with a small
  on-disk cache; the list never blocks on it; the parsed fields and the raw permission text are
  returned to the UI.
- **R5** — offline and stale-index behaviour are explicit (age shown, no empty list).
- **R6** — tests use the local HTTP server only.
- **R7** — **Responsiveness (owner, 2026-10-22):** index reads, JSON parsing/validation,
  search preparation, filtering/ranking and `.txt` work run outside the UI event loop. Prepare
  normalized search fields once per adopted index, and reuse the immutable prepared index
  between queries; do not clone or reparse the full index on each keystroke. Bound query work
  to one running request and one replaceable pending request; superseded work must yield or
  cancel, and stale results must never replace the latest query/filter/page/index generation.
  Return only the requested page and total count to the UI. No thread or unbounded task queue
  per keystroke. Refresh prepares a replacement snapshot in the background while the current
  one remains usable.

## Acceptance criteria

- [ ] **A1** (R1) — fixtures for each rejection case; partial acceptance is not allowed for a
  malformed index.
- [ ] **A2** (R2) — loopback server: 200, 304, 403, oversized, corrupt JSON, cache precedence, 24 h
  throttle with a fake clock.
- [ ] **A3** (R3) — fixture index: search, diacritics, filters, paging boundaries, stable order.
- [ ] **A4** (R4) — txt fetch/cache/encoding and offline fallback.
- [ ] **A5** — no network in tests; fmt, clippy `-D warnings`, tests green.
- [ ] **A6** (R7) — deterministic tests with a full-size synthetic index (at least 10,885
  entries) and a valid index near the 2 MiB limit: rapid query/filter/page changes keep the
  pending queue bounded; blocked old work and out-of-order completions cannot publish stale
  results; index preparation happens once per snapshot; refresh does not invalidate a usable
  snapshot before its replacement is ready. Use controlled workers/barriers, not timing sleeps.
- [ ] **A7** (R7) — record release-build timings and peak memory for cold preparation, warm
  searches (common, rare and no-match), filters and paging on the full-size fixture. Include
  repeated query/refresh cycles to detect retained snapshots or growing queues. Host timings
  are diagnostic; Steam Deck responsiveness acceptance belongs to spec 053 A7.

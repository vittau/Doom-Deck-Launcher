# 053 — Browse UI and promotion hand-off

**Status:** Deferred — no current priority (D-041)  ·  **Milestone:** M4  ·  **Depends on:** 030, 031, 045, 051, 052
**Origin:** spec 041 research (`docs/research/idgames-browse.md`, report R5), 2026-10-19; owner: advance M4 while Deck testing waits.

**Goal:** a controller-only, couch-readable browse/search flow that never implies compatibility.

**Priority update (owner, 2026-10-23):** the curated Discover catalog is sufficient for
now; occasional other content uses manual custom setups/extras. Keep this contract for
future consideration; it is not the next task to dispatch (D-041).

## Requirements

- **R1** — Discover section + pushed screen; four root tabs unchanged; tab bar disabled on the
  browse/detail screens; B = Back (spec 030).
- **R2** — controller: Y opens the Steam OSK search (spec 031) with the physical-keyboard fallback,
  typeless filters, 50-row pages via L1/R1 or "Load more", focus restored to the previous row.
- **R3** — entry page as in research §6.5: "Unverified" chip, author/date/size/source, engine quote,
  IWAD availability, no rating/screenshots/verified badge, other-port-required = Play disabled,
  warning sheet before the first download.
- **R4** — progress, cancel, offline/stale states, index-missing state; errors are specific
  (offline, host refused, size changed, hash mismatch, missing IWAD).
- **R5** — every string in `strings.slint` (English), no process jargon (spec 045); headless renders
  only.
- **R6** — "Export recipe draft" action for installed unverified entries, with the CLI instructions
  and a lint-clean draft path.
- **R7** — **Responsiveness (owner, 2026-10-22):** open the screen immediately with a loading
  state when needed; Back, focus movement and cancel remain responsive during preparation,
  search, refresh and downloads. Use spec 051 R7's bounded background queries, with a short
  text-input debounce (150 ms default); Back and controller navigation are never debounced.
  The list model holds at most one 50-row page, including when using "Load more"; never create
  UI rows for the entire archive or append pages indefinitely. Publish a page in one model
  update, preserve focus by entry identity where possible, and ignore completions after leaving
  the screen. Retain the current usable results during refresh; show progress without blocking
  input. Do not fetch entry `.txt` files while scrolling the list.

## Acceptance criteria

- [ ] **A1** (R1, R3) — headless renders of the browse list, entry page with each gating case, and the
  warning sheet; reviewed by the coordinator.
- [ ] **A2** (R2) — navigation test: open browse, type via a fake OSK, filter, page, open entry, back
  restores focus.
- [ ] **A3** (R4) — fake-download progress/cancel/error renders; offline and no-index states.
- [ ] **A4** (R6) — draft action test with a fake index/install.
- [ ] **A5** — fmt, clippy `-D warnings`, tests green; Deck check of OSK/focus/readability
  (**Needs Deck Validation**).
- [ ] **A6** (R7) — headless event-loop tests with the full-size index and controlled slow
  workers: Back/focus/cancel are handled before blocked preparation/search/network work is
  released; rapid typing/filtering/page changes show only the latest results; leaving the
  screen prevents late updates; the UI model never exceeds 50 rows; scrolling performs no
  `.txt` requests. Repeated open/search/back cycles do not accumulate jobs or models.
- [ ] **A7** (R7) — **Proposed acceptance targets, Needs Deck Validation:** in a release build
  on the Steam Deck LCD, p95 input-to-visible focus/Back response ≤ 100 ms while loading,
  searching and refreshing; warm local searches display the requested page within 300 ms of
  query dispatch (excluding the text debounce), and reopening with a prepared index displays
  the first page within 500 ms. Measure at least 100 interactions using the full-size fixture,
  record p95/max and peak memory, and check frame-time spikes during page replacement. Cold
  preparation has a responsive loading state and is timed separately. Host/headless results
  do not establish Deck performance; report misses explicitly before marking this complete.

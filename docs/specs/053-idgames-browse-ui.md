# 053 — Browse UI and promotion hand-off

**Status:** Ready (after 052)  ·  **Milestone:** M4  ·  **Depends on:** 030, 031, 045, 051, 052
**Origin:** spec 041 research (`docs/research/idgames-browse.md`, report R5), 2026-10-19; owner: advance M4 while Deck testing waits.

**Goal:** a controller-only, couch-readable browse/search flow that never implies compatibility.

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

## Acceptance criteria

- [ ] **A1** (R1, R3) — headless renders of the browse list, entry page with each gating case, and the
  warning sheet; reviewed by the coordinator.
- [ ] **A2** (R2) — navigation test: open browse, type via a fake OSK, filter, page, open entry, back
  restores focus.
- [ ] **A3** (R4) — fake-download progress/cancel/error renders; offline and no-index states.
- [ ] **A4** (R6) — draft action test with a fake index/install.
- [ ] **A5** — fmt, clippy `-D warnings`, tests green; Deck check of OSK/focus/readability
  (**Needs Deck Validation**).

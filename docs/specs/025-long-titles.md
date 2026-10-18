# 025 — Long mapset titles fit their slots

**Status:** Done  ·  **Milestone:** M3  ·  **Depends on:** 012, 023
**Origin:** spec 023 phase B report: long titles clip in Home and Discover text slots (visible
with Back to Saturn X). Owner approved the fix on 2026-10-17.

## Goal

Every mapset title and author line in the catalog reads cleanly on every screen at 1280×800 —
never cut mid-glyph or overflowing a frame — while the Ember Cathedral look (D-021) stays.

## Requirements

- **R1** — **Audit:** render every catalog recipe's title and author in each place they appear
  (Home hero and Recently played, Discover cards, Library cards, Mapset detail, Play setup,
  profile picker, uninstall dialog, art band if any) and list the clipped cases in the report.
- **R2** — **Fit:** titles wrap to at most two lines where the slot allows, otherwise shrink one
  step within the type scale, and only then elide with "…" — the full title is always shown on
  Mapset detail. Author lines elide to one line on cards. No text may overflow its frame or
  overlap another element.
- **R3** — **Test:** a headless test renders the longest catalog title and a synthetic 60-character
  title in each slot and asserts the text fits its slot (e.g. via measured text bounds or the
  layout's preferred size), so a future recipe cannot regress it.

## Acceptance criteria

- [x] **A1** (R1, R2) — before/after headless renders for the clipped cases, inspected; README
  screenshots regenerated if they change.
- [x] **A2** (R3) — the fit test passes for all 13 recipes and the synthetic title.
- [x] **A3** — fmt, clippy `-D warnings`, tests green.

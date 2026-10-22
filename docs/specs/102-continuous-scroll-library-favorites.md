# 102 — Continuous list scrolling and Library favorites first

**Status:** Implemented — awaiting coordinator visual review and physical Deck validation · **Depends on:** 061, 090, 095, 101
**Origin:** owner, 2026-10-22: scrolling feels like changing pages and loses their
position; use smooth continuous scrolling and order Library favorites first.

## Requirements

- **R1 — Continuous focus following.** In Discover and Library List mode, preserve the
  current offset while the focused row remains visible. When Up/Down crosses a viewport
  boundary, move only enough to reveal the next row, retaining neighboring rows as visual
  context rather than aligning every selection to the top or replacing a whole page.
  Repeated/held navigation, reversals and first/last rows must remain bounded and readable.
- **R2 — Smooth movement.** Use a short, unobtrusive interpolation for focus-follow scrolling
  in these list views. Keep logical focus and the final offset correct during rapid input;
  do not queue animations or defer selection/actions. Manual stick/wheel/Flickable scrolling
  stays continuous and bounded; neutral ticks and refreshes must not reset its position.
  Keep Cards and unrelated setup/sheet/picker behavior intact unless a shared fix is
  necessary, with a focused regression check if so. Tab/category changes and return from
  detail must retain valid focus and a visible selected item.
- **R3 — Library favorites first.** Installed favorite mapsets sort before installed
  nonfavorite mapsets in Library, in both Cards and List. Within each group keep the
  existing chronological/title tie-break order. Keep base-game/local sections and their
  existing controls reachable. Favorite toggles update order without changing the selected
  item identity or duplicating/losing rows. Discover retains its chronological ordering
  and existing Favorites filter. No saved-state schema or catalog format change.
- **R4 — Focused evidence.** Add small offline regressions for minimum-distance list
  scrolling, rapid/reverse focus, bounds/manual-scroll preservation, and Library's
  favorite grouping plus identity after toggling. Produce focused off-screen before/after
  or timed frames that make continuity and retained neighbors reviewable. No full title
  matrix, full workspace suite, network e2e or SDK build for this change. Use relevant
  targeted tests and formatting; run only the Clippy scope justified by the changed code.

## Ownership and acceptance

One supervised Codex UI worker owns app Rust/Slint, focused UI/runtime tests, relevant
user-guide wording and `docs/reports/102-continuous-scroll-library-favorites.md`. No
screenshots/catalog-recipe changes: an independent OpenCode worker owns those in spec 103.
Read AGENTS.md and the current proportionate verification guidance. Merge main before
final focused checks; commit small changes. No push/tag/release or nested workers. No
visible windows or screen capture: production off-screen renders only. Coordinator
reviews and shows the focused visual evidence before merging.

Acceptance: stepping through more than two viewports never produces a page-sized jump;
the first/last item and focused row after reversal/action remain correct; favorite
installed mapsets precede other installed mapsets in both layouts without changing
Discover. Record exact checks and remaining physical Deck validation in the report.

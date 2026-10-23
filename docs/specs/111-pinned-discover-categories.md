# 111 — Pinned Discover category strip

**Status:** Implemented — merged, awaiting physical Deck validation · **Depends on:** 072, 090, 102
**Origin:** owner, 2026-10-23: keep the Discover tabs fixed at the top while scrolling, so it
is easier to see where they are.

## Context

The Discover category strip (`Nodes::strip`, `crates/ddl/src/runtime.rs`) is drawn as ordinary
nodes at the top of the main scroll plane, so it scrolls away with the mapset cards or rows.
Only Discover has this strip; Library filters and other screens are out of scope.

## Requirements

- **R1 — Pinned strip.** In Discover, in both Cards and List layouts, the category strip stays
  at a fixed position at the top of the content area for any vertical scroll offset
  (focus-follow, stick, wheel or Flickable). Cards and rows scroll beneath it and are clipped
  at its lower edge: no mapset content draws over, behind or above the strip. The strip may
  use a solid or subtly graded background in the existing Ember tokens so the separation reads
  clearly; amber still means focus only (D-036).
- **R2 — Scroll math.** Focus-follow scrolling treats the pinned strip as outside the
  scrolling viewport: a focused card or row is fully visible below the strip, including its
  focus ring, and spec 102's continuous minimum-distance movement is retained in List mode.
  Up from the first row moves focus to the strip without a scroll jump; Down from the strip
  returns to the first visible item as today. Content height and bounds still let the last
  card/row be fully revealed.
- **R3 — Strip behavior unchanged.** Selection, L2/R2 category switching, horizontal overflow
  with ‹ › glyphs, chip focus ring (fully visible, not clipped by the new container), category
  memory and empty-category handling behave as before. Changing category starts the list at
  its top with the strip still pinned.
- **R4 — No collateral change.** Library, Home, Mapset detail, setup, sheets, pickers and
  toasts keep their current layout and scrolling. Pointer/touch clicks on chips and on
  partially scrolled cards still reach the right target; content hidden beneath the strip
  must not receive clicks.
- **R5 — Evidence.** Focused offline regressions for: strip position constant across offsets,
  focused item never overlapping the strip (Cards and List, first/last item, reversal),
  Up/Down between strip and list, and click routing at the strip boundary. Off-screen
  production renders (software renderer, PNG) of Discover Cards and List at the top and
  scrolled deep, with focus on a chip and on a card/row. No full title matrix, network e2e or
  SDK build; run the focused tests, fmt and the Clippy scope justified by the changed code.

## Ownership and acceptance

One supervised Codex UI worker owns `crates/ddl/src/runtime.rs`, `crates/ddl/ui/app-window.slint`
(and `tokens.slint` only if a background token is needed), the focused headless/runtime tests
and `docs/reports/111-pinned-discover-categories.md`. No catalog, status, journal, decisions or
user-guide edits. Merge `main` before final checks; commit small changes; no push, tag or
release. Never open visible windows or capture the screen — headless renders only.

Acceptance: scrolling Discover through more than two viewports in both layouts keeps the strip
at the same pixels, the focused item is always fully visible beneath it, and the renders show
clean separation with no content bleeding over the strip. Physical feel on the Deck LCD is
**Needs Deck Validation**.

## Owner decisions during review (2026-10-23)

- Discover cards are **340px** tall (from 410px), three per row at 384px; Library cards stay 410px.
- Card tags use **one compact row** (14px text, 26px chips, 6px gap); a second tag that does
  not fit is omitted rather than wrapped. The freed row gives the screenshot 96px of height.

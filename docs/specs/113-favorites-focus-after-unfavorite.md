# 113 — Favorites category: focus after unfavoriting

**Status:** Ready · **Depends on:** 072, 102, 111, 112
**Origin:** found while implementing spec 112 (report 112, open question); owner asked for the
fix on 2026-10-23.

## Problem

In Discover's Favorites category, unfavoriting the focused card (X) removes it from the list
(membership is the favorite mark), but `focus` keeps pointing at the removed card's id. The
card is no longer drawn, so nothing shows focus and the next D-pad press starts from a card
that is not on screen. The same happens when the mapset is unfavorited on Mapset detail and
the player returns to Favorites.

## Requirements

- **R1 — Neighbour focus.** When the focused Discover item leaves the current category's list,
  focus moves to the item that now occupies its position in the displayed order (the next
  one); if it was the last item, to the new last item; if the list becomes empty, to the
  Favorites chip in the category strip, with the existing empty-Favorites panel shown. Cards
  and List behave the same.
- **R2 — Return from detail.** Returning from Mapset detail to a category that no longer
  contains the selected mapset applies R1, using the mapset's former position.
- **R3 — Visible and stable.** The newly focused item is fully visible beneath the pinned
  strip (spec 111) without a page jump; re-favoriting the removed mapset elsewhere later does
  not move focus. No change when the focused item stays in the list (spec 112 moves keep
  focus on the same mapset).
- **R4 — No collateral change.** Other categories, Library, ordering (D-043), saved state and
  the favorite toggle's persistence are unchanged.
- **R5 — Evidence.** Focused runtime/headless tests for: unfavoriting a middle, the last and
  the only Favorites item, in Cards and List; returning from detail after unfavoriting there.
  Run the ordinary test command from `docs/orchestration.md` § Verification before finishing.

## Ownership and acceptance

One supervised worker owns `crates/ddl/src/runtime.rs`, the affected tests and
`docs/reports/113-favorites-focus-after-unfavorite.md`. No status, journal, decisions, catalog
or user-guide edits. Merge `main` before final checks; small commits; no push, tag or release.
Never open visible windows or capture the screen — headless renders only.

Acceptance: after unfavoriting in Favorites, a drawn item (or the Favorites chip when empty)
always has focus and is fully visible; the ordinary suite passes.

# 112 — Discover: installed and favorite mapsets first

**Status:** Ready · **Depends on:** 072, 090, 102, 111 · **Decision:** D-043
**Origin:** owner, 2026-10-23: items that are installed and favorite should come first; a
favorite that is not installed comes right after the installed ones — order by the tuple.

## Context

`Runtime::discover_items` (`crates/ddl/src/runtime.rs`) filters the snapshot by the open
category and sorts with `compare_browsing_order` only (chronological, then title and id).
Library (spec 102 R3) already puts installed favorites first.

## Requirements

- **R1 — Tuple order.** In every Discover category (including Favorites) and in both Cards and
  List, items sort by (installed, favorite), descending:
  1. installed and favorite;
  2. installed, not favorite;
  3. favorite, not installed;
  4. neither.
  Within each group keep `compare_browsing_order`. "Installed" means the same condition the
  card already shows as installed (an install manifest is present), whatever the base-game
  state. Keep the ordering logic small and unit-testable; it may live in `ddl-core` beside
  `compare_browsing_order` if that keeps the app code thin.
- **R2 — Live changes keep identity.** Toggling a favorite (X), finishing an install or an
  uninstall moves the item to its new group without losing or duplicating rows, and focus
  stays on the same mapset (it scrolls into view beneath the pinned strip, spec 111). Return
  from Mapset detail keeps the same selected mapset.
- **R3 — No collateral change.** Category membership, the category strip, Library order,
  Home, detail, saved state and catalog data are unchanged. No schema change.
- **R4 — Evidence.** Unit tests for the four-group order and the in-group tie-break; focused
  runtime/headless tests for R2 (favorite toggle, install and uninstall moving an item,
  focus identity) in Cards and List. Because this change reorders production nodes, run the
  **ordinary test command** from `docs/orchestration.md` § Verification before finishing, not
  only focused filters, and update any journey that hard-codes the old Discover order. One
  off-screen render per layout showing the four groups with real catalog recipes.

## Ownership and acceptance

One supervised worker owns `crates/ddl/src/runtime.rs`, `crates/ddl-core/src/catalog/order.rs`
(and its `mod.rs` export, if used), the affected tests and
`docs/reports/112-discover-installed-favorites-first.md`. No status, journal, decisions,
catalog or user-guide edits. Merge `main` before final checks; small commits; no push, tag or
release. Never open visible windows or capture the screen — headless renders only.

Acceptance: a Discover category with mixed items shows installed favorites, installed,
favorite-only and other mapsets in that order, chronological within each group, in Cards and
List; toggling a favorite or installing moves the item with focus intact; the ordinary suite
passes. Physical feel is **Needs Deck Validation**.

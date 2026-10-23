# 112 — Discover: installed and favorite mapsets first

## Result

Discover now leads every category, in Cards and List, with installed and favorite mapsets
(spec 112 R1, decision D-043). Items sort by the `(installed, favorite)` tuple descending —
installed favorites, installed mapsets, favorite-only mapsets, the rest — with the existing
chronological browsing order (`compare_browsing_order`: release date, then title and id)
unchanged inside each group. "Installed" is the card's installed state: an install manifest is
present, whatever the base-game state.

Category membership, the strip, Library order, Home, detail, saved state and catalog data are
untouched (R3). Favorites keeps its own membership rule; only its ordering changed.

## What changed

| File | Change |
| --- | --- |
| `crates/ddl-core/src/catalog/order.rs` | `compare_discover_order(a, a_installed, a_favorite, b, b_installed, b_favorite)` and the private `discover_group` rank helper; module docs updated. |
| `crates/ddl-core/src/catalog/mod.rs` | Exports `compare_discover_order`. |
| `crates/ddl/src/runtime.rs` | `Runtime::discover_items` sorts with the new comparator, passing `manifest.is_some()` and `favorite` per item. Cards and List share this method, so both layouts follow it. |
| `crates/ddl/src/headless_library.rs` | Focused headless tests for the four groups (with off-screen renders), the favorite toggle, the production install and the production uninstall. |
| `crates/ddl/src/headless_production.rs` | `categories_favorites_and_screenshots` journey updated: its category-order expectation uses `compare_discover_order`, and its "favoriting never moves a card" check now asserts the spec 112 move (favorite leads, focus kept, no lost/duplicated row). |

Row identity needed no new code: card/row ids are snapshot indexes and `replace_snapshot`
already remaps selection and card focus by recipe id (spec 090 R5). The ordering change
therefore moves a row without duplicating it, and focus scrolls it into view beneath the
pinned strip (spec 111). No new dependencies, no schema change, no catalog edit.

## R2 interpretation for uninstall (coordinator-confirmed)

A real uninstall keeps its spec 022 completion flow — Library, card focus, cleared history —
because Library excludes uninstalled mapsets and R3 forbids collateral change. Asked through
orchestration; the coordinator agreed that spec 112 R2 applies to the Discover ordering and row
identity for uninstalls, not to that navigation. The uninstall test therefore asserts the row
survives with no duplicate, the selection keeps the same recipe, and reopening Discover with
that mapset focused shows it once in its new group. Focus-identity checks stay with the
favorite toggle and install, where the reorder is visible in place.

## Verification

Commands run in the worktree (macOS arm64, offline):

| Command | Result |
| --- | --- |
| `cargo test -p ddl-core order` | 23 passed, 0 failed (includes the three new group/chronology tests). |
| `cargo test -p ddl --locked discover_installed_favorites_first_in_both_layouts` | 1 passed, 0 failed. |
| `cargo test -p ddl --locked discover_favorite_toggle_moves_the_row_and_keeps_focus` | 1 passed, 0 failed. |
| `cargo test -p ddl --locked discover_install_and_uninstall_move_the_item_and_keep_focus` | 1 passed, 0 failed. |
| `cargo test -p ddl --locked categories_favorites_and_screenshots` (excluded from the ordinary suite) | 1 passed, 0 failed (131 s). |
| `cargo test --workspace --locked -- --skip headless::journeys::title_fit_ --skip headless::journeys::categories_favorites_and_screenshots` | **1,008 passed, 0 failed**, exit 0. |
| `cargo fmt --all --check` | Clean. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Clean. |

Test coverage per requirement:

- **R1** — `catalog::order::tests::discover_leads_with_installed_and_favorite_groups`,
  `discover_keeps_chronology_inside_each_group` and `discover_groups_are_the_descending_tuple`
  cover the four groups, the in-group chronological tie-break (including unknown dates and the
  title/id fallback), and the descending rank.
- **R2** — `discover_favorite_toggle_moves_the_row_and_keeps_focus` runs the toggle in Cards and
  List, asserting the drawn order, no lost or duplicated row, focus on the same card, the saved
  favorite, and that returning from detail keeps the selection.
  `discover_install_and_uninstall_move_the_item_and_keep_focus` runs a real production install
  (loopback fixture, released package gate) in Cards and List — leaving Play setup returns to
  Discover focused on the same mapset, now in the installed group — then favorites it and runs a
  real uninstall, asserting its new group, single row and kept selection.
- **R3** — No source outside `runtime.rs`/`order.rs` ordering changed; the ordinary suite covers
  the unchanged screens.
- **R4** — Two off-screen renders with real catalog recipes in the `megawad` category, one mapset
  per group, produced by the render test. In Cards the first row is the installed favorite, the
  installed mapset and the favorite-only mapset, with the trailing group starting on the second
  row; in List the four groups appear in sequence.

Off-screen renders (software renderer, 1280×800, never a visible window):

- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/discover-installed-favorites-first/.scratch/112/discover_installed_favorites_first_in_both_layouts/112-discover-groups-cards.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/discover-installed-favorites-first/.scratch/112/discover_installed_favorites_first_in_both_layouts/112-discover-groups-list.png`

Reproduce with
`DDL_HEADLESS_DIR="$PWD/.scratch/112" cargo test -p ddl --locked discover_installed_favorites_first_in_both_layouts`.
`.scratch/` is gitignored, so the images stay out of the repository.

## Commits

On `vittau/discover-installed-favorites-first`:

- `604ba14` Sort Discover by installed and favorite mapsets
- `9ac60aa` Cover Discover grouping, focus and install flows

`main` was at the branch base `8c37fe0` when this was verified; no merge was needed. Nothing was
pushed, tagged or released.

## Open questions

- Physical feel on the Deck (how the reorder and focus-follow look at 800p, and whether an
  installed favorite leading every category feels right) is **Needs Deck Validation**.
- In the Favorites category, unfavoriting the focused card removes its row (membership rule,
  unchanged by R3) and focus then rests on a card that is no longer drawn, as before this spec.
  Picking a neighbour would be a separate behavior change; not done here.

# 113 — Favorites category: focus after unfavoriting

## Result

Unfavoriting the focused card in Discover's Favorites category no longer leaves focus on a card
that is no longer drawn. Focus moves to the card that now occupies the removed card's slot, else
to the new last card, else to the Favorites chip with its empty panel (R1). The same rule applies
when the mapset is unfavorited on Mapset detail and the player returns to Favorites, using the
mapset's former position (R2). The new focus is revealed beneath the pinned strip without a page
jump, and stays on the same mapset when that mapset is re-favorited later or when a category
keeps the item (R3). Cards and List share the rule.

Only the Favorites category loses an item when the favorite mark changes — membership in a tag
category is the recipe's tags — so the rule is guarded to `Category::Favorites` on Discover and
no other category, Library, ordering (D-043), saved state or the toggle's persistence changes
(R4).

## What changed

| File | Change |
| --- | --- |
| `crates/ddl/src/runtime.rs` | New `Runtime::settle_unfavorited_focus`, called from `sync` when Discover shows Favorites. It detects that the focused card index is absent from `discover_items`, rebuilds the Favorites order of the moment before the mark was removed (`former_favorites_position`: the item still counts as a favorite for `compare_discover_order`), and focuses the card now in that slot, else the last card, else the chip. Because the call sits in `sync`, one rule covers the optimistic toggle on Discover, the snapshot that replaces it when the worker finishes, and the return from detail (focus is restored from history before the same `sync`). |
| `crates/ddl/src/runtime.rs` | `mod favorites_focus_tests` (8 isolated headless tests): middle/last/only favorite in Cards and List, return from detail after unfavoriting there, a later re-favorite not moving focus, and a tag category keeping focus on the same mapset. |
| `crates/ddl/src/headless_production.rs` | `categories_favorites_and_screenshots` journey: after returning from detail where the only favorite was unfavorited, it now asserts the Favorites chip holds focus instead of only asserting that no card does. |

No new dependencies. No schema, catalog, copy or status change.

The former position is reconstructed rather than stored: the removed item is still in the
snapshot with `favorite == false`, and the Favorites order only differs from the previous one by
that single flag, so ordering it as a favorite reproduces the slot the next card inherited.

## Verification

Commands run in the worktree (macOS arm64, offline). `main` was at `a4f6a7c`, the branch base;
`git fetch origin main` showed nothing newer, so no merge was needed.

| Command | Result |
| --- | --- |
| `cargo test -p ddl favorites_focus_tests` | 8 passed, 0 failed. |
| `cargo test -p ddl --locked categories_favorites_and_screenshots` (excluded from the ordinary suite) | 1 passed, 0 failed (142 s). |
| `cargo test --workspace --locked -- --skip headless::journeys::title_fit_ --skip headless::journeys::categories_favorites_and_screenshots` | **1,016 passed, 0 failed**, exit 0 (ddl 256, ddl-core 651, integration 109). |
| `cargo fmt --all --check` | Clean. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Clean. |

Test coverage per requirement:

- **R1** — `unfavoriting_a_middle_card_in_cards_focuses_the_card_in_its_slot`,
  `unfavoriting_a_middle_card_in_list_focuses_the_card_in_its_slot`,
  `unfavoriting_the_last_card_in_cards_focuses_the_new_last_card`,
  `unfavoriting_the_last_card_in_list_focuses_the_new_last_card`,
  `unfavoriting_the_only_card_in_cards_focuses_the_favorites_chip`,
  `unfavoriting_the_only_card_in_list_focuses_the_favorites_chip`. Each presses X on a runtime
  whose stored state really favorites the fixture recipes (the favorite write runs through
  `Services`, so the returned snapshot matches), asserts the new focus, that the focused node is
  drawn and fully inside the scrolling plane below the pinned strip, and — for the middle case —
  that the offset did not change, because the replacement card inherits the slot. The only-card
  case asserts the chip is drawn in the strip, the empty-Favorites panel is shown and no card is
  drawn. Every case re-asserts the focus after the worker's snapshot has settled.
- **R2** — `returning_from_detail_after_unfavoriting_lands_on_the_former_position` opens detail on
  the middle of three favorites, unfavorites with X, waits for the saved snapshot, presses B and
  asserts Discover shows the card now in the former position, fully visible. The category
  journey also covers the empty case after returning from detail.
- **R3** — the same tests assert geometry (node inside the plane below the strip) and that the
  middle case keeps its scroll offset; `unfavoriting_a_middle_card...` also re-favorites the
  removed mapset through `replace_snapshot` and asserts focus still names the same recipe;
  `unfavoriting_in_a_tag_category_keeps_focus_on_the_same_mapset` covers an item that stays in
  the list while its group changes.
- **R4** — no other source changed; the ordinary suite covers the untouched screens, and the tag
  category test keeps its existing behaviour.

## Commits

On `vittau/favorites-focus-after-unfavorite`:

- `167d783` Keep a drawn neighbour focused after unfavoriting in Favorites

Nothing was pushed, tagged or released.

## Open questions

- Physical feel on the Deck (how the neighbour reveal lands under the strip, especially when the
  removed card was in the last row) is **Needs Deck Validation**; the headless checks only prove
  geometry through the same focus-follow path spec 111 established.
- R1 says "the current category's list"; the implementation is scoped to Favorites because the
  favorite mark is the only membership that changes from a Discover action. If a future category
  can lose items, the guard needs revisiting.

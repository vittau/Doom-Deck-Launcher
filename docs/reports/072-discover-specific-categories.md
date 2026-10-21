# 072 — Discover uses specific categories

## Scope and result

Discover no longer offers `All` or `Featured`. The strip is the owner's requested
specific set, in this fixed order, restricted to categories at least one active
recipe uses, followed by the personal `Favorites` filter:

`Cacowards` → `Megawads` → `Community Projects` → `Doomer Boards` → `Classics` →
`Episodes` → `Challenging` → `Favorites`.

`Doomer Boards` appears automatically once the 073 catalog supplies the vocabulary
and memberships; until then it is simply absent. `Featured`, `Beginner Friendly` and
any other vocabulary entry keep their catalog metadata but are not navigation chips.
No catalog, schema, compatibility, download or recipe file was changed by this task.

The empty-Favorites action is now **Browse mapsets** and selects the first available
requested specific category instead of the removed All view. When no specific
category exists (empty catalog), Favorites remains selected and the action is not
offered, so there is no dead button or empty-list index.

## Selection and recovery rules

- A new session chooses the first available requested category once a snapshot
  exists (Cacowards when populated).
- A category that disappears after a snapshot/catalog refresh falls back to the
  first available remaining category; card/chip focus is moved to that category's
  first card, or its chip when empty.
- `Favorites` is always the final list element, so the list is never empty.
- Category stepping is guarded against an empty list and still uses wrapping
  `rem_euclid`; L2/R2 wrap and D-pad/trigger navigation are preserved.
- Favorite sorting inside a category is unchanged.

## Reachability with a 90+ recipe catalog

Discover and Library cards are addressed by `CARD + snapshot_index`. The guard
previously accepted only ids `10..=99`, so the 91st and 92nd recipes —
the two Memento Mori recipes 073 adds — would have received ids 100 and 101 and
then failed activation and card classification. The range is now
`CARD..=CARD_LAST` (10–199), still entirely below the category strip's reserved
`CHIP` (200) and far from the Setup component range (100–199 is per-screen, never
on Discover/Library). Root actions, base-game and custom ids on other screens are
untouched, and no new generic id framework was introduced.

`CARD` and `CARD_LAST` are named constants; the render, activation, focus,
Back-recovery, favorite target and card-classification checks all use them.
`headless::journeys::catalog_card_ids_beyond_99_stay_reachable` grows the
production snapshot to 92 entries in memory, focuses the last card (id 101),
asserts it is drawn and still classified as a card, and confirms Confirm opens
that exact mapset in Detail.

## Coverage and tests

- `runtime::category_tests::requested_categories_keep_the_owner_order_and_omit_other_navigation`:
  exact owner order, only used requested ids, `featured`/`beginner-friendly`/`All`
  omitted, first choice Cacowards, empty list falls back to Favorites.
- `runtime::category_tests::every_embedded_mapset_is_reachable_from_a_requested_category`:
  every embedded recipe carries at least one requested specific tag.
- `headless::journeys::categories_favorites_and_screenshots`: production snapshot,
  no network. Asserts initial Cacowards selection, the full strip order/no All or
  Featured, every category's cards match its tag, chip focus and L2/R2 wrap, D-pad
  entry to the strip, empty Favorites with the Browse mapsets action leading to the
  first specific category, favorite sorting, category kept across Back/tab switches,
  fallback after a removed category with focus on a shown card, and an empty catalog
  with Favorites selected, safe chip focus and no browse action.
- `headless::journeys::catalog_card_ids_beyond_99_stay_reachable`: 92-entry
  in-memory snapshot, last card id 101 drawn, classified as a card and opened in
  Detail by Confirm.
- `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- No full unrelated headless suite was run.

## Screenshot producer changes

The existing journey capture that named the removed All view was renamed from
`015-discover-all-favorite.png` to `015-discover-specific-favorite.png`; the
`discover.png` mapping in `scripts/readme-screenshots.py` points at it. Other
captures keep their names. The five shared README/website PNGs were regenerated
from the four existing producers with the lossless script after the UI was final:

| Shared asset | Producer | Bytes |
| --- | --- | --- |
| `docs/screenshots/home.png` | `headless::journeys::library_favorite_and_home` | 276,406 |
| `docs/screenshots/discover.png` | `headless::journeys::categories_favorites_and_screenshots` (`015-discover-specific-favorite.png`) | 562,482 |
| `docs/screenshots/play-setup.png` | `headless::journeys::auger_options_and_components` | 278,961 |
| `docs/screenshots/mapset-detail.png` | `headless::journeys::categories_favorites_and_screenshots` (`015-detail-tags.png`) | 537,714 |
| `docs/screenshots/audio-settings.png` | `headless::audio::audio_section_renders_and_selects_default` | 217,704 |

The regeneration ran with `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2` before the card-id
range fix; that fix changes node ids only, not pixels, so the assets were not
regenerated a second time.

## Limits

Physical Deck validation of category hold-repeat feel, small header type readability
and real catalog-update timing remains for the owner. The catalog refresh paths are
covered with synthetic in-memory state, not a live network update. No push, tag,
release, workflow or coordinator-document change was made.

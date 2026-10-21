# 074 — Roomier mapset cards and base-game art

## Scope and result

Discover/Library mapset cards keep their 384×410 box, three-column grid and
410/438 geometry, but their content now sits inside a consistent 28 px inset
instead of touching the 20 px ornamental frame:

| Element | Before | After |
| --- | --- | --- |
| Art (mapset screenshot) | `x+20, y+20`, 344×142, flush with the frame | `x+28, y+28`, 328×128, 8 px inside the frame |
| Favorite mark | `x+312, y+28` | `x+316, y+36`, inside the art |
| Title | `x+24, y+174`, 336×68 | `x+28, y+164`, 328×60 |
| Author | `x+24, y+244` | `x+28, y+230` |
| Game and readiness | two lines at `y+270` and `y+300` | one line at `y+258`: `DOOM II · Base game missing` |
| Installed / profile count | `x+24, y+330` | `x+28, y+286` |
| Tag chips | `x+24, y+366` (second row overflowed) | `x+28, y+314`; two rows end at `y+382` |

Game and readiness are joined with the same `·` separator the Home screen already
uses; the information is unchanged and the line is 18 px so the longest label
(`Ultimate DOOM · Base game missing`) still fits instead of eliding. No mapset
card, grid, focus-ring, controller-target or sorting behaviour changed.

The bug this fixes is visible in the before render: a Doomer Boards recipe shows
two wide chips, so the second row was drawn at `y+488…518` — past the card's
`y+410` bottom, over the bottom frame ornament and into the next card row at
`y+522`. The after render keeps both rows inside the card with an 8 px gap to the
frame and 28 px to the card edge, for focused and unfocused, installed and
uninstalled cards.

### Base-game Library cards

The four original-game cards in Library were a separate, equally cramped
layout; `a648137` only changed the mapset cards. They now use a 24 px inset
(the frame is 20 px, so every element stays inside it) and a taller title slot
so the full game name wraps instead of eliding:

| Element | Before | After |
| --- | --- | --- |
| Card | 282×84 at `y=152` | 282×108 at `y=148` |
| Art | `x+10, y+164`, 96×60 | `x+24, y+24`, 80×60 (4:3, inside the frame) |
| Title | `x+116, y+162`, 156×64; drawn over the top frame | `x+112, y+24`, 146×60; two lines, full names |
| Section heading | `y=252` | `y=280`, clear of the focused outline |
| First mapset row | shifted `+202` | shifted `+226` |

`The Plutonia Experiment` now renders on two lines and `Ultimate Doom`, `DOOM II`
and `TNT: Evilution` stay on one, all at the readable fitted size. The focused
outline still extends 16 px outside the card; the first content is 24 px in.

### Discover catalog date (077 UI follow-up)

Discover's existing header status line (the small muted line under the heading)
now shows the effective catalog's source date, built from
`snapshot.catalog.serial` with `ddl_core::catalog::serial_datetime`. The format
is exactly `October 21, 2026 at 13:39 (UTC)`. Serial `0` (unknown) shows
`Catalog date unavailable`; the line never substitutes today's date. It is
recomputed on every sync, so a newer active catalog updates it without moving
the category strip or grid. The heading keeps its place and moves up 20 px only
as it already does whenever a status line is present; card space is unchanged.

## Evidence

Headless 1280×800 software renders, Discover at scroll top, focused Doomer Boards
card and two unfocused neighbours:

- before: `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/app-header-brand/target/headless/card_padding_before_capture/074-card-before.png`
- after: `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/app-header-brand/target/headless/cards_keep_roomy_insets_and_two_tag_rows/074-card-after.png`

Observed before bounds (page coordinates): card `12,84 384×410`; art `32,104
344×142` (0 px inside the frame); chips rows at `y 450` and `y 488`, the second
ending at `518` — 24 px below the card. Observed after bounds: card `12,84
384×410`; art `40,112 328×128`; title `40,248 328×60`; chips at `y 398` and
`y 436`, ending at `466` — 8 px inside the frame (`474`), 28 px above the card
edge (`494`).

The focused card's gold ring extends 16 px beyond the card; the first content is
28 px in, so neither the art nor the chips can cover the focus treatment.

Base-game evidence: the focused `Ultimate Doom` card with the four original
games and the `Mapsets and custom setups` heading:
`target/headless/base_games_library_page_setup_sources_and_recent_history/032-library-base-games.png`.
All four titles are complete and the art is clear of the frame on every card.

Discover date evidence:
`target/headless/discover_shows_the_catalog_source_date/077-discover-catalog-date.png`
shows the sample serial `1_792_589_940` as `October 21, 2026 at 13:39 (UTC)`.
The shared `docs/screenshots/discover.png` is captured from the same sample
state at scroll top with the category strip and first card row visible; the
sample serial is disclosed here and is not a live fetch time.

## Combined game/status baseline (integration fix)

Spec 074 R1 leaves every card's information unchanged, but the roomier layout
renders game and readiness, update or removal on one 18 px line:
`<game> · <status>`. The recipe-change journey still looked for the old
standalone `Update available` label (and later a standalone `No longer in the
catalog` on the Library card), so it failed on the merged checkpoint even though
the notice was visible. The journey now finds the intended installed card
(id `CARD`) and its status slot (`x+28, y+258`, 328×24), asserts the exact
combined text `DOOM II · Update available` or
`DOOM II · No longer in the catalog`, and checks that the line is inside the
800 px viewport. The detail screen keeps its own standalone removal copy.
Rendered evidence: `020-library-update.png` and `020-library-removed.png`.

## Verification

- `headless::journeys::recipe_changes_update_keep_and_removed`: the complete
  update, Keep current, explicit Update and removed-catalog journey passes, with
  the visible notice asserted on the intended card's combined status line.
- `headless::journeys::cards_keep_roomy_insets_and_two_tag_rows`: production
  snapshot, no network. Focuses an installed Doomer Boards card with two wide tag
  labels (so two chip rows), then asserts the exact art/title geometry, the two
  chip rows and the 28 px inset for every content node of the focused card; then
  repeats the inset assertion for every drawn card (unfocused, installed and
  uninstalled) and writes `074-card-after.png`.
- `runtime::local_ui::tests::base_games_library_page_setup_sources_and_recent_history`:
  asserts the four base-game cards are 282×108, each art is 80×60 at the 24 px
  inset, each title uses the 60 px multiline slot, and no content node enters the
  20 px frame; writes `032-library-base-games.png`.
- `headless::journeys::discover_shows_the_catalog_source_date`: sample serial
  `1_792_589_940` gives the exact UTC line, serial `0` gives
  `Catalog date unavailable`, and a newer serial updates the same line; writes
  `077-discover-catalog-date.png`.
- `headless::journeys::categories_favorites_and_screenshots`: full category,
  focus, favorites and screenshot journey still passes with the new layout and
  the sample catalog serial, and produces the scroll-top category capture.
- `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo test --workspace -- --test-threads=2` is the root's final run. The
  earlier full run's two failures are resolved in this tree: the recipe journey
  passes with the combined assertion, and the integrated backend at `3568c86`
  fixed the frozen-tags release guard
  (`services::release_state_tests::release_state_trees_remain_readable` now
  passes). The SIGIL resolver summary expectation was also updated for the new
  copy (076). No known UI-related full-suite failure remains.

## Limits

Physical Deck readability of the 18 px game/readiness line and the 24/28 px
insets is owner validation. The shared screenshot state uses a deterministic
sample catalog serial (`1_792_589_940`) so the date subtitle is visible; a real
build with an unknown serial shows the honest fallback instead. No visible
window was opened and no screen was captured. No push, tag, release or
coordinator-document change was made.

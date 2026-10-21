# 074 — Roomier mapset cards and base-game art

## Scope and result

Discover/Library mapset cards keep their 384×410 box, three-column grid and
410/438 geometry, but their content now sits inside a consistent 28 px inset
instead of touching the 20 px ornamental frame:

| Element | Before | After |
| --- | --- | --- |
| Art (base-game screenshot) | `x+20, y+20`, 344×142, flush with the frame | `x+28, y+28`, 328×128, 8 px inside the frame |
| Favorite mark | `x+312, y+28` | `x+316, y+36`, inside the art |
| Title | `x+24, y+174`, 336×68 | `x+28, y+164`, 328×60 |
| Author | `x+24, y+244` | `x+28, y+230` |
| Game and readiness | two lines at `y+270` and `y+300` | one line at `y+258`: `DOOM II · Base game missing` |
| Installed / profile count | `x+24, y+330` | `x+28, y+286` |
| Tag chips | `x+24, y+366` (second row overflowed) | `x+28, y+314`; two rows end at `y+382` |

Game and readiness are joined with the same `·` separator the Home screen already
uses; the information is unchanged and the line is 18 px so the longest label
(`Ultimate DOOM · Base game missing`) still fits instead of eliding. No card,
grid, focus-ring, controller-target or sorting behaviour changed.

The bug this fixes is visible in the before render: a Doomer Boards recipe shows
two wide chips, so the second row was drawn at `y+488…518` — past the card's
`y+410` bottom, over the bottom frame ornament and into the next card row at
`y+522`. The after render keeps both rows inside the card with an 8 px gap to the
frame and 28 px to the card edge, for focused and unfocused, installed and
uninstalled cards.

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

## Verification

- `headless::journeys::cards_keep_roomy_insets_and_two_tag_rows`: production
  snapshot, no network. Focuses an installed Doomer Boards card with two wide tag
  labels (so two chip rows), then asserts the exact art/title geometry, the two
  chip rows and the 28 px inset for every content node of the focused card; then
  repeats the inset assertion for every drawn card (unfocused, installed and
  uninstalled) and writes `074-card-after.png`.
- `headless::journeys::categories_favorites_and_screenshots`: full category,
  focus, favorites and screenshot journey still passes with the new layout.
- `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets -- -D warnings`: clean.

## Limits

Physical Deck readability of the 18 px game/readiness line and the 28 px inset is
owner validation. Screenshots were deliberately not regenerated: spec 074 R3
requires the final refresh after the integrated catalog and any further visual
changes, at a scroll-top Discover state that shows the category strip. No visible
window was opened and no screen was captured. No push, tag, release or
coordinator-document change was made.

# 071 — Application header matches the website brand

## Scope and result

The application header now uses the website's accepted compact arrangement from
`main` 0518500 instead of the single horizontal lockup:

- the existing approved stained-glass emblem sits at the header's old origin;
- the outlined DDL lettering above the smaller `DoomDeckLauncher` subtitle sits to
  its right, both lines sharing a left edge;
- the text block is vertically centered against the emblem;
- the tab strip, its focus ring and input mapping are unchanged.

The implementation is a small `Brand` component inside `crates/ddl/ui/app-window.slint`.
It composes existing approved assets only: `assets/logo/emblem-small.svg` and the
outlined DDL portion of `assets/logo/horizontal.svg`, cropped with a clipped
rectangle exactly as the website CSS crops the same 330×128 master. No new art, SVG
redraw, font, dependency or AI image was introduced.

## Measured layout

| Element | Position / size |
| --- | --- |
| Emblem box | x 40, y 24, 56×56 (visible SVG core starts at 42,26) |
| Emblem-to-copy gap | 12 px (website compact gap, scaled to the 56 px header) |
| DDL wordmark box | x 66.5 relative to the brand, 77×31.5, clipped from `horizontal.svg` |
| Subtitle | x 66.5, 10 px Barlow, one line below the wordmark with a 2 px box gap |
| Visible copy extent | x 107–194, y 29–73 in the 1280×800 render |
| First tab | x 214, unchanged; focus ring reaches x 206 |
| Header divider | y 102, unchanged |

The copy stays left of the tab focus-ring edge (194 < 206) and the emblem keeps the
old x40/y24 56 px header envelope. The previous tab `for label[index]` block, action
mapping and focus code were not touched.

## Verification

- `headless::app_header_brand_matches_the_compact_site_layout`: isolated off-screen
  Slint software render. Asserts emblem origin/height, 8–14 px emblem-to-copy gap,
  equal wordmark/subtitle left edges (≤2 px), compact line gap (≤8 px), text block
  vertical centering (≤6 px), copy clear of the tab ring, F1–F4 producing tab
  actions 20–23, and the Home tab's amber focus ring at its previous x206/x214
  bounds with the unchanged active surface. Writes `header-brand.png` and
  `header-tabs.png` under the isolated headless case directory.
- `headless::journeys::categories_favorites_and_screenshots`: the shared Discover
  journey renders the new header in every captured screen and exercises controller
  category navigation.
- `cargo test -p ddl --test ui_strings`: all visible Slint literals remain
  translatable (the subtitle uses `@tr`).
- `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- The five shared website/README screenshots were regenerated from the four
  existing headless producers after the header and Discover changes, using
  `scripts/readme-screenshots.py` and its lossless recompression (see 072 report
  for the asset table and final sizes).

## Limits

Rendering and input are verified headlessly at the Deck's 1280×800 logical size.
Physical Steam Deck readability of the small subtitle, controller glyph feedback
and panel scaling remain owner/Deck validation, not automated claims. No visible
window was opened and no screen was captured. No push, tag or release was made.

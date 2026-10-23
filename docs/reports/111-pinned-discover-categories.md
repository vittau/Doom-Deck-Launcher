# 111 — Pinned Discover category strip

## Final result and owner decision

**Confirmed:** Discover categories remain pinned in Cards and List. The independent
80px strip occupies y=230–310 at 1280×800; the scrolling plane clips painting and
pointer input below y=310. It uses the existing `Tokens.deep` background. Amber
continues to indicate focus only. Chips and overflow glyphs remain in the existing
navigation model, with their original selection, overflow and memory behavior.

After reviewing the initial 386px implementation and 340px alternative, the owner
chose **340px Discover cards as the production default**, compact single-row tag
chips, and returning the freed tag-row space to the screenshot. The former
review-only node transformation was removed; every new render uses production
geometry directly.

| Discover Cards element | Final geometry |
| --- | --- |
| Card | 384×340px, three columns, 368px vertical pitch |
| Frame content inset | 28px |
| Screenshot | 328×96px at relative y=28 |
| Title | Relative y=132, 60px slot, unchanged 26px font |
| Author | Relative y=198, unchanged 20px font |
| Base-game/status line | Relative y=226, unchanged 18px font |
| Installed state | Relative y=254, unchanged 20px font |
| Tag chips | One row at relative y=282, 26px high, 14px Barlow text |
| Tag spacing | 6px gap; 5px text inset on each side |

The 340px preview previously had 58px of screenshot height. Removing the second
tag row returns exactly 38px to that image, producing 96px of art while retaining
legible title/author/status sizes and 32px below the tag row. The initial card's
full focus ring spans y=314–686 at 1280×800.

Library cards remain 410px with their original 128px art, 18px tags and two-row
wrapping. Mapset detail retains its existing tag layout. List rows remain 74px
with an 82px pitch. The smaller chip text inset applies only to compact tags;
existing 18px chips retain their 10px inset. No dependencies, catalog files,
persisted schemas, tokens or protected coordinator-owned documents changed.

## Scrolling and input

Focus-follow calculations exclude pinned nodes and use scrolling-plane local
coordinates. Discover has a 422px scrolling viewport at 1280×800. List retains
minimum-distance movement and its existing 120ms retargetable interpolation.
Entering the strip preserves the presented offset; switching categories resets
the list from either a chip or a scrolled item. Bounds reveal the final focus
ring. Local pickers do not reserve strip space; other screens keep their previous
geometry and scroll functions.

The original implementation required the authorized mechanical `pinned: false`
initializer in `crates/ddl/src/shell.rs:1062`. This follow-up makes no shell change.

## Chip packing and evidence

Discover uses a separate single-row packing method. The existing shared
`Nodes::chips` and `chip_width` behavior remains unchanged for Library/detail.
Compact width is estimated as `ceil(character_count × 14 × 0.60 + 12)`; the wider
glyph allowance was verified against the real bundled Barlow font, rather than
assuming the legacy estimate was sufficient. The initial 15px/legacy estimate
failed the font check for “Megawads”; the final 14px estimate passes every label.

“Community Projects” and “Beginner Friendly” occupy 164px and 155px respectively:
with the 6px gap, their combined width is 325px inside the 328px content region.
Every ordered pair of distinct tags in the vocabulary is checked for containment,
identical y, full natural Barlow text width and height, and preserved lead order.
If a second label cannot fit, it is omitted; it never wraps or truncates the first.
A deliberately overlong second label tests this fallback.

The real-catalog scene also checks the actual production chip nodes for **every
embedded recipe** using Favorites, then checks the Community Projects category
with the open category leading. Its top/deep renders show real titles, artwork
and two-tag cards, including Memento Mori and Eviternity II. Their source recipes
are unchanged; the test marks recipes favorite in temporary state for the full
catalog check.

The pinned regression retains both layouts' byte-for-byte strip pixel comparison
across top/deep/arbitrary offsets, stick input, wheel input and Flickable drag;
full focus-ring bounds for every item forward and backward; repeated directional
reversals; Up/Down between the strip and first item; category reset from chips and
items; and real pointer clicks at y=305 (hidden content cannot activate), y=315
(visible content activates the correct item) and on the scrolled strip.

## Verification

`git merge main` returned **Already up to date** before final checks
(main `a160306272d6d3cb755cf29d77e3c6142245c62f`). All tests are offline. Every PNG
uses the production Slint software renderer; no visible window or screen capture
was opened.

| Exact command | Result |
| --- | --- |
| `DDL_HEADLESS_DIR="$PWD/.scratch/111" cargo test -p ddl compact_discover -- --nocapture` | 2 passed: natural font widths for all vocabulary pairs and real-catalog geometry/renders |
| `DDL_HEADLESS_DIR="$PWD/.scratch/111" cargo test -p ddl headless::library -- --nocapture` | 10 passed: pinned strip, compact production catalog, continuous Discover/Library motion, category/favorites navigation, Library scenes, neutral-scroll preservation and 1,000-item coverage |
| `cargo test -p ddl runtime::category_tests -- --nocapture` | 6 passed, including compact chip packing/font coverage and existing overflow/ordering tests |
| `DDL_HEADLESS_DIR="$PWD/.scratch/111" cargo test -p ddl compact_discover_catalog_scene -- --nocapture` | 1 passed again after adding an explicit zero-offset top-frame check and settling prior layout animation |
| `cargo fmt --all --check` | Passed |
| `cargo clippy -p ddl --all-targets -- -D warnings` | Passed, including after the final test-only adjustment |
| `git diff --check` | Passed |

The preceding implementation also passed the three focused scroll-math tests and
the empty-category/layout-toggle regression. This follow-up changes card/tag
geometry only; the focused headless suite rechecks the unchanged scroll math,
continuous List behavior and Library behavior. No full title matrix, full workspace
suite, network e2e or SDK build was needed.

## Fresh production renders

The following real-catalog frames were visually inspected. Top frames explicitly
use offset zero; deep frames focus the last applicable two-tag recipe. The smaller
Community Projects category has fewer than two List viewports of scroll travel;
the independent 30-item pinned regression still proves traversal beyond two
viewports in both layouts.

- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/compact_discover_catalog_scene/111-catalog-cards-top.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/compact_discover_catalog_scene/111-catalog-cards-deep.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/compact_discover_catalog_scene/111-catalog-list-top.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/compact_discover_catalog_scene/111-catalog-list-deep.png`

The original focused-chip/focused-item matrix was also regenerated with the final production geometry:

- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-cards-top-chip.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-cards-top-item.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-cards-deep-chip.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-cards-deep-item.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-list-top-chip.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-list-top-item.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-list-deep-chip.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-list-deep-item.png`

The old `111-cards-compact-340-review-only.png` is historical output only and is
superseded by these production frames; its transformation is no longer in the test.

## Remaining validation

No implementation questions or automated acceptance items remain open for R1–R5
or the owner's compact-card follow-up. **Needs Deck Validation:** physical Steam
Deck LCD readability and feel, including compact tag text and controller/stick/touch
comfort. No protected documentation changes are proposed by this implementation.

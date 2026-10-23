# 111 — Pinned Discover category strip

## Result

**Confirmed:** Discover categories remain pinned in Cards and List. The independent
80px strip occupies y=230–310 at 1280×800; the scrolling plane clips painting and
pointer input below y=310. It uses the existing `Tokens.deep` background, and amber
continues to indicate focus only. `Nodes::strip` marks chips and overflow glyphs as
pinned while retaining them in the existing navigation model.

Focus-follow calculations exclude pinned nodes and convert scrolling nodes into
the reduced viewport's local coordinates. Discover has 422px of scrolling height
at 1280×800. List retains minimum-distance movement and its existing 120ms
retargetable interpolation. Entering the strip preserves the presented offset;
category changes reset the list even if the chip keeps focus. Content bounds
include the final focus ring. Library and the remaining planes retain their
previous geometry, bounds and scroll functions; local pickers do not reserve a
pinned strip.

### Authorized geometry adjustment

The original 410px card plus its 32px focus ring cannot fit below a fully visible
category strip. The coordinator authorized **386px Discover cards**: screenshot
height decreases from 128px to 104px, lower content shifts up 24px, and vertical
row pitch decreases from 438px to 414px. Width remains 384px with three columns.
Library cards remain 410px; List rows remain 74px with an 82px pitch. At the initial
position, the first card's full ring spans y=314–732, below the strip.

The coordinator also authorized the mechanical `pinned: false` initializer in
`crates/ddl/src/shell.rs:1062`, required by the shared Slint node structure. No
other shell behavior changed. No dependencies, tokens, catalog data, persisted
schemas or coordinator-owned documents changed.

## Verification

`git merge main` returned **Already up to date** before the final checks
(main `a160306272d6d3cb755cf29d77e3c6142245c62f`). All tests run offline with synthetic
state and the production Slint software renderer; no desktop window or screen
capture was used.

| Exact command | Result |
| --- | --- |
| `DDL_HEADLESS_DIR="$PWD/.scratch/111" cargo test -p ddl headless::library -- --nocapture` | 9 passed (including the final rerun after the category-reset fix); includes the new pinned test, existing Discover/Library continuous motion, category/favorites navigation, Library scenes, setup/sheet neutral-scroll preservation and 1,000-item coverage |
| `cargo test -p ddl runtime::category_tests -- --nocapture` | 5 passed; category ordering/reachability, overflow window and glyphs, tag chips |
| `cargo test -p ddl runtime::scroll_tests -- --nocapture` | 3 passed; continuous minimum-distance bounds, reversals and existing card/section follow math |
| `cargo test -p ddl compact_toggle_contexts_empty_lists_and_save_failure_remain_usable -- --nocapture` | 1 passed; empty category, layout toggle and unrelated-screen behavior |
| `cargo fmt --all --check` | Passed |
| `cargo clippy -p ddl --all-targets -- -D warnings` | Passed; app scope matches the changed Rust/Slint code |
| `DDL_HEADLESS_DIR="$PWD/.scratch/111" cargo test -p ddl pinned_discover_categories -- --nocapture` | 1 passed again after simplifying equivalent branches in the review-only preview to satisfy Clippy |
| `git diff --check` | Passed |

The new regression covers both layouts equally:

- Byte-for-byte strip-region pixel equality at the top, deep scroll, arbitrary
  focus offsets, manual stick scrolling, native wheel input and a Flickable drag.
- Every focused item from first to last and back, with its complete focus ring
  bounded between y=310 and y=732; repeated directional movement and reversal
  preserve the selected identity. Deep navigation exceeds two scrolling viewports.
- Up from the first item to the selected chip without an offset jump, Down back
  to the first item, and category switching back to offset zero from both a
  focused chip and a deeply scrolled focused item.
- Real pointer events at y=305 (hidden content must receive no activation), y=315
  (the visible part of that same card/row activates the correct item), and on a
  chip while content is scrolled.
- Existing spec 102 regression now measures Discover in the reduced local viewport,
  preserving its timed animation, rapid reversal, manual takeover, neutral refresh,
  bounds, activation and detail-return checks. Library's 522px viewport is unchanged.

An initial compile identified the required shared-node initializer in `shell.rs`;
it was added after authorization. The first Clippy pass identified identical
branches in the optional preview transform; those branches were combined and the
final Clippy and affected test both passed. Final review also corrected a 12px
List category-reset offset when switching from an item, added regression coverage,
and reran all nine focused headless cases plus formatting and Clippy successfully.
No full workspace suite, title matrix,
network e2e or SDK build was needed for this focused UI change.

## Off-screen production PNGs

All nine frames below were visually inspected. The fixture uses 30 catalog-derived
mapsets with deterministic titles to exercise deep scrolling; actual production
nodes, artwork, fonts, tokens, input clipping and software rendering are used.
Each layout has top/deep frames with a focused chip and a focused item.

- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-cards-top-chip.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-cards-top-item.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-cards-deep-chip.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-cards-deep-item.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-list-top-chip.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-list-top-item.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-list-deep-chip.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-list-deep-item.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/pinned-discover-categories/.scratch/111/pinned_discover_categories/111-cards-compact-340-review-only.png`

### Review-only compact alternative

`111-cards-compact-340-review-only.png` shows 340px Discover cards beside the
386px default evidence (`111-cards-top-item.png`). It is generated only by the
headless test's transformation of production nodes: card height and row pitch
are reduced by another 46px, screenshot height becomes 58px, and lower text and
chips move up 46px. Width, three-column layout, text sizes, category strip and
List mode stay the same. The image reveals part of the next row and gives art
less prominence; it does not change the production default or add a preference.

## Open questions and remaining validation

No implementation questions or automated acceptance items remain open for R1–R5.
**Needs Deck Validation:** physical Steam Deck LCD feel, controller/stick and touch
comfort, visual density of the 386px default, and any owner choice to pursue the
340px alternative. No changes to the protected overview, decisions, status,
journal or user guide are proposed by this implementation.

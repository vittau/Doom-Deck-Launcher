# 102 — Continuous list scrolling and Library favorites first

## Outcome

**Confirmed:** implemented R1–R4 in app Rust/Slint and focused offline tests.
Discover and Library List reveal the focused row with the minimum movement,
keeping the existing offset whenever the row and its focus ring already fit.
Library installed favorites now precede other installed mapsets in Cards and List;
release chronology, unknown dates and title tie-breaks still apply within each group.
Discover retains its chronology and Favorites filter.

## Changes

- `runtime.rs`: compute list destinations from the currently presented position,
  clamp to the actual list extent, and retain manual offsets across neutral node
  rebuilds. Include selected row geometry in browsing anchors so reordering the
  same favorite also reveals it. Other screens retain their old anchor behavior.
- `app-window.slint`: keep destination/focus immediate and interpolate only the
  presented position for 120 ms with ease-out. New focus input retargets the
  current interpolation. Stick and native wheel input interrupt it at the current
  presentation. Neutral polls do not read a stale `content-y` back into the offset.
  List bounds use the real extent; Cards retain their previous whole-card landing
  and trailing viewport, and setup/sheet/picker planes keep immediate scrolling.
- `headless_library.rs`: add a small edge/bounds/interpolation journey for both
  lists and a favorite grouping/identity regression for both layouts. The former
  exercises repeated movement across more than two viewports, rapid reversal,
  action/return identity, neutral refresh, manual stick and native wheel offsets,
  interruptions during animation and both manual bounds. The latter checks known
  chronology, equal-date title ordering, unknown dates, no lost/duplicated rows,
  immediate favorite reorder, identity across layout changes, favorite persistence,
  Discover's Favorites chronology and access to base/local rows. Existing synthetic
  installs are intentionally snapshot-only; real service favorite persistence is
  checked separately from the optimistic reorder before that refresh.
- `headless.rs`: the timed-frame test opts into a mock-only clock in its isolated
  process, so PNG/software-render wall time cannot consume the animation. Other
  headless journeys retain their existing real-plus-mock clock.
- Update the existing compact journey's old first-row/top-alignment expectation
  to require a fully visible row; the new behavior intentionally retains context.

No dependencies, state schema, catalog formats, recipes or screenshot assets changed.
No visible windows were opened and no desktop screen capture was taken.

## Verification

Integrated local `main` (`2e8bb33`) before final focused checks with
`git merge main --no-edit`; Git reported already up to date. Implementation commits:
`c22028b` and `e8bdfe7`; final test/report follow-up is recorded in this branch.

All Cargo checks used the shared warm target via
`CARGO_TARGET_DIR=/Users/vitor/orca/workspaces/Doom-Deck-Launcher/flatpak-metainfo-version/target`.

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo test -p ddl --bin ddl headless::library --locked` | 8 passed |
| `cargo test -p ddl --bin ddl scroll_tests --locked` | 3 passed, including unchanged Cards/sheet policy regressions |
| `cargo test -p ddl --bin ddl compact_layout_survives_stale_snapshot --locked` | 1 passed; selected identity remaps across snapshot reordering |
| `cargo test -p ddl --bin ddl library_favorites_group --locked` | 1 passed after strengthening the fixture to distinguish chronological ordering from equal-date title tie-breaks |
| `cargo clippy -p ddl --bin ddl --tests --locked -- -D warnings` | Passed; limited to the changed app binary and its tests |
| `git diff --check` | Passed |

Logs: `.scratch/102-checks/{fmt,library,scroll,identity,favorites,clippy}.log`.
The eight headless Library cases include the existing Cards scene, base-game
navigation, neutral plane polling, compact category/detail/layout identity and
1,000-item repeated-layout/model-release regressions. No full workspace suite,
title-fit matrix, network e2e, Linux SDK build or release build was run, as requested.

## Off-screen evidence

Review page, with both before/during/after triptychs:

`/Users/vitor/orca/workspaces/Doom-Deck-Launcher/continuous-scroll/.scratch/102-review/index.html`

The six original production software frames are at:

`target/headless/continuous_lists_retarget_preserve_manual_offsets_and_render_motion/`

- `102-discover-before.png`, `102-discover-during.png`, `102-discover-after.png`
- `102-library-before.png`, `102-library-during.png`, `102-library-after.png`

Before is settled on the last fully visible mapset; during is 40 ms into a
120 ms interpolation after Down; after is sampled at +200 ms. Discover retains
Mapsets 01–05 while revealing 06. Library retains the base-game/mapset transition
while revealing Mapset 01 after Mapset 00. The focused row is fully visible at
the destination and the middle frame is strictly between the starting and final
offsets. All six frames were visually reviewed off-screen. The review directory
contains lossless `sips` re-encodes of those PNGs for image-viewer compatibility;
no pixel/layout edits were applied. Reproduce the originals with the focused
`continuous_lists_retarget_preserve_manual_offsets_and_render_motion` test.

## Remaining validation

**Needs Deck Validation:** physical controller held-repeat feel, whether 120 ms
feels appropriately unobtrusive in Gaming Mode, real wheel/trackpad/touch-drag
interaction with native Flickable inertia, and the owner's actual saved library.
Headless checks establish logic, native synthetic wheel handling and rendering,
not physical input feel or Linux/Deck performance.

No implementation requirements are deferred. Coordinator visual acceptance and
physical Deck validation remain; no push, tag or release was performed.

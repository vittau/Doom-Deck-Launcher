# 061 — Library frame rails and focus-follow scrolling

## Result and scope

Restore a clean Library landing with the coordinator's explicit clarification:
focus-follow aligns the section heading and whole focused item together when
they fit, otherwise the item's top. Manual right-stick scrolling remains free.
The shared main plane, Play setup summary and sheets use this policy. No new
dependencies, button styles, focus ranking or platform behavior changes.

The owner-like headless fixture contains seven installed catalog mapsets, four
base games and two custom setups at 1280×800. Its Library landing shows three
384 px cards across the available width, with “Mapsets and custom setups” at
the top and no base-game bottom rails below the header. It also renders the
absolute top, later rows, custom setups and manual partial-row scrolling.

The additional base-game focus reset discovered during fixture construction was
fixed at the coordinator's request. `local_nodes` now validates focus after all
Library cards have been appended, preserving base-game focus while retaining the
existing default mapset/custom/base fallback order. No remaining navigation issue
from that finding is deferred.

## Investigation

**Confirmed (headless/source):** the frame-rail symptom already exists at
`716bee5` (`v0.0.7-deck`). The first mapset is at y=298, height=410; the old
bottom-based focus-follow formula picks offset 214. The preceding four base-game
cards end near that offset, so their bottom rails are legitimately inside the
clipped viewport. This is not an SVG clipping failure. The same owner-like scene
has three columns in the old version and main: a single installed mapset naturally
occupies only the left column, rather than indicating a narrowed grid.

A bounded history comparison renders the same fixture at `716bee5`, `60cca92^`,
`60cca92`, and pre-fix main `a6c8a19`. There is no first-bad commit between these
endpoints for the pre-existing rails. The separate scrolling regression was
introduced by `60cca92` (`Scroll focused views and align controller navigation
defaults`): its new `ScrollPlane.scroll()` reads `content-y` before Slint has
updated the binding after Rust assigns `scroll-offset`. The gamepad timer invokes
it even for `(0, 0)`, overwriting the new focus-follow position with the previous
position. Its parent has no such scroll function. The fix computes the next
vertical position from `offset` directly for all shared ScrollPlane instances.
Neutral vertical ticks are no-ops: they also preserve the whole-card landing of
a final row whose top lies beyond the normal manual-scroll bound.

The clean first-mapset landing now uses offset 256: the preceding card frames
are fully above the viewport, the section heading sits at its normal inset,
and the full first row fits. Later rows align whole cards, with room for their
focus rings. The last row also needs a trailing viewport allowance in the
Flickable content extent: otherwise Flickable clamps the requested top alignment
back to its ordinary bottom bound during rendering, exposing the previous row's
bottom again. The regression checks this after rendering and event-loop updates,
as well as checking the immediate Rust property. The existing viewport clips all
vector frame pieces during free manual scrolling. No frame assets or grid
geometry needed changes.

## Regression coverage

- `headless::library::library_owner_scene_renders`: real service-created custom
  setups and registered synthetic base IWAD, seven snapshot-installed mapsets,
  four base games; heading position, three column geometry and actual rendered
  occupancy in each column. Compares the under-header band against the same wall
  with content removed, so any rendered frame pixel is detected independently of
  the SVG palette. Exercises deeper focus, manual scroll persistence across sync,
  and exact pixel equality outside the viewport after scrolling. Captures settle
  Slint change handlers before rendering; final-row alignment is rechecked after
  rendering to catch Flickable's own bounds adjustment.
- `headless::library::neutral_stick_tick_preserves_each_plane_offset`: assigns a
  focus offset then immediately invokes the neutral timer tick before a render,
  on main, setup summary and sheet planes. A nonzero manual delta must also
  start from the newly assigned main-plane offset.
- `runtime::scroll_tests`: section heading plus card fits/does not fit; an item
  already visible; later rows; oversized items; sheet/summary-style buttons.
- `headless::library::library_base_game_focus_survives_refresh_and_dpad_navigation`:
  coordinator-authorized scope addition. Base-game focus survives node rebuilds
  and a replacement service snapshot with installed mapsets present; Down enters
  the row, Right/Left reach all four cards, and Down/Up connect it to the mapsets
  and New custom setup action. Its focus-preservation assertion fails before
  the fix (focus 10 instead of 6300); log: `target/headless/061-before-base-focus.log`.
- The existing Discover carved-rail comparison now checks y=238–261, the new
  whole-card landing, instead of y=298–321, which lies inside the screenshot
  after the layout fix. The focused/unfocused rail pixel equality assertion is
  retained. `discover_and_detail` passes with that updated expected geometry.
- Existing spec 057 controller tests retain axis injection, no focus movement,
  manual refresh persistence, routing, and navigation checks for the other screens.

## Headless artifacts

All paths are relative to the repository; these are software-rendered PNGs,
never desktop captures. Run the corresponding tests to regenerate them.

Before/history, each directory contains `061-library-rest.png`,
`061-home-rest.png`, and `061-discover-rest.png`:

- `target/headless/061-history/716bee5/library_owner_history/`
- `target/headless/061-history/60cca92-parent/library_owner_history/`
- `target/headless/061-history/60cca92/library_owner_history/`
- `target/headless/061-history/a6c8a19/library_owner_history/`

After, under `target/headless/library_owner_scene_renders/`:

- `061-library-rest.png` — first mapset landing, no old rails, three columns.
- `061-library-top.png` — absolute top with the first base game focused.
- `061-library-row-two.png`, `061-library-row-three.png` — deeper focus.
- `061-library-stick.png` — partial rows after manual scroll.
- `061-library-custom-setup.png` — custom setup focus.
- `061-home-rest.png`, `061-discover-rest.png` — same owner-like snapshot.

## Verification

The heading and neutral-stick regression assertions were exercised in an
isolated archive of pre-fix main `a6c8a19`. Both fail there: the heading is 38 px below the origin instead of -4
(the node renderer adds its normal 8 px inset), and a neutral event-loop poll
reverts a newly assigned offset of 234 to 214. The failure log is
`target/headless/061-before-regressions.log`; the before Library PNG is
`target/headless/061-before/library_owner_scene_renders/061-library-rest.png`.
The archive endpoint/parent matrix logs are `/tmp/ddl061-history/*.log`.

Merged main (`2a38b7c`) in `e412231`; only the coordinator's status entry came
through that merge. `scripts/readme-screenshots.py` regenerated all four README
images byte-for-byte unchanged. The targeted Library/focus/neutral-stick tests (3),
focus-follow unit tests (2), and existing spec 057 controller tests (3) pass.
Final checks on macOS arm64, after the coordinator-added base-game focus fix:

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed |
| `cargo test --workspace` | Passed: 770 tests across all groups, including 163 binary tests and 565 core unit tests |
| `python3 scripts/readme-screenshots.py --check` | Passed; all four committed images are byte-for-byte unchanged |

Implementation commits: `7ca2c5f` (focus-follow policy and owner-like scene),
`d1a7609` (final-row bounds and neutral ticks), `4531bb1` (base-game focus
preservation), and `24648f2` (existing Discover rail assertion).

Final logs: `target/headless/061-final/0.log` (three Library regressions),
`1.log` (fmt), `2.log` (clippy), `3.log` (workspace), `4.log` (README check).
No visible windows were opened and no screen capture was taken.

## Open work

**Needs Deck Validation:** physical controller scroll feel and visual confirmation
with the owner's persisted library. No requested layout/scroll behavior is deferred.

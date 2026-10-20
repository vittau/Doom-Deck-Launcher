# Report 048 — Vector panel frames and ornaments

Branch: `vittau/vector-frames`. Contract: spec 048, D-020 and D-021.

## Change

Original, script-authored SVG replaces the atlas-cut frame and four ornament
PNGs. The design retains the 28 px frame, 96×28 px top/bottom ornaments and
28×48 px side ornaments. Dark stepped mouldings, four narrow bronze rails,
an oxblood inset, heavy corner castings and pointed gothic crests follow the
Ember Cathedral palette. All four corners and rails derive from one perimeter
profile in `crates/ddl/ui/assets/ember/author_frame.py`; ornaments have transparent
grounds so the underlying rail remains continuous even at fractional fit.

`frame.slint` composes four fixed-size corner SVGs, four rails stretched only
along their constant longitudinal profile, and four fixed-size ornament SVGs.
The production `Frame` uses that component and preserves the old opaque
`Tokens.surface` centre, including on sheets. The painted wall, covers, layout,
focus and input behaviour are unchanged by the frame work.

## Slint 1.18 verification (R2)

**Confirmed (pinned source):** this repository actually uses **1.18.1** (both
`slint` and `slint-build`). SVG images can be wrapped in nine-slice; the API
places any image in `NineSliceImage`, without excluding SVG
([`image.rs`, lines 1040–1060](https://github.com/slint-ui/slint/blob/v1.18.1/internal/core/graphics/image.rs#L1040-L1060)).
Both renderers dispatch nine-slice through `fit9slice`, whose border dimensions
are multiplied by the window scale factor
([shared fitting](https://github.com/slint-ui/slint/blob/v1.18.1/internal/core/graphics/image.rs#L1558-L1623),
[femtovg](https://github.com/slint-ui/slint/blob/v1.18.1/internal/renderers/femtovg/itemrenderer.rs#L1306-L1341),
[software](https://github.com/slint-ui/slint/blob/v1.18.1/internal/renderers/software/lib.rs#L2745-L2770)).

**Confirmed (source + off-screen experiment):** accepting SVG nine-slice does
not preserve the required carving in the software renderer. It unwraps the
SVG for each slice, then passes that slice's target size to `render_to_buffer`
([software rasterization](https://github.com/slint-ui/slint/blob/v1.18.1/internal/renderers/software/lib.rs#L2370-L2423)).
The SVG renderer scales the *entire SVG* to fit that size
([SVG rendering](https://github.com/slint-ui/slint/blob/v1.18.1/internal/core/graphics/image/svg.rs#L54-L90)).
For a 128×128 master and 28×28 corner, the whole frame is rasterized to 28×28
before the corner is sampled and enlarged: the corner carving loses most of
its detail and no longer matches native ornament grounds. The initial probe
showed this at 1×; increasing the fit factor does not fix the underlying ratio.
Femtovg follows a different path: `is_svg()` does not unwrap `NineSlice`, so
its texture uses intrinsic resolution
([classification](https://github.com/slint-ui/slint/blob/v1.18.1/internal/core/graphics/image.rs#L535-L550),
[texture sizing](https://github.com/slint-ui/slint/blob/v1.18.1/internal/renderers/femtovg/itemrenderer.rs#L1232-L1260)).
A single SVG nine-slice therefore fails the requirement to work consistently
on **both** renderers. Separate intrinsic canvases are the specified fallback.

**Confirmed (software test):** the composed SVG component preserves every
corner/rail and ornament joint pixel-exactly at 1× (the 1280×800 Deck fit) and
1.35× (the 1920×1080 fit from spec 029). Two panel sizes exercise all eight
corner joints and all eight ornament edges per panel, across every pixel of
the 28 px profile. No colour tolerance or golden-image update hides a step. Each sampled profile
must also contain at least twelve colours, so an empty frame cannot pass.

**Needs Deck Validation:** actual femtovg presentation and handheld appearance.
The femtovg path was checked in source; no visible GL window or screen capture
was used on the owner's machine.

## Verification / acceptance

- **A1:** `python3 crates/ddl/ui/assets/ember/author_frame.py --check` checks
  byte-identical regeneration with Python's standard library and rejects tags
  or references outside the SVG-only element allowlist. The SVGs contain
  paths, circles, transforms and linear gradients; no raster or filter.
  `cargo test -p ddl --test vector_frames` renders the shared production frame
  off-screen and checks all joints at both scales and two sizes.
- **A2:** owner ("Melhor") and coordinator approved the revised dark gothic
  design before production wiring. Final four-screen comparisons on the fixed
  wall exist and were inspected. The owner approved all four final renders and
  the 4× corner on 2026-10-20; the coordinator relayed that approval.
- **A3:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --
  -D warnings` and `cargo test --workspace` pass after merging `c4b6380`. The
  workspace suite reports **582 passed**, including 140 DDL unit/headless tests,
  408 ddl-core unit tests, integration tests and three doctests.
  `python3 scripts/readme-screenshots.py --check` also passes: all four committed
  PNGs match freshly rendered and recompressed output byte-identically.

Design artifacts (uncommitted, under `target/vector-frames/`):
`design-1x.png`, `design-fit.png`, `design-1x-corner-4x.png`,
`design-fit-corner-4x.png`, `home-design.png`, `home-design-before-after.png`.
The last two are explicitly design mockups: the existing headless Home image
with candidate perimeters rendered over it by the software renderer. The
production screen comparisons are separate final artifacts below.

Targeted production journeys: Home and AUGER setup/components pass, including
native-size opaque SVG crest pixels. A pixel comparison against the old Home
render found every pixel outside its two 28 px frame perimeters byte-identical.
The centre remains opaque `Tokens.surface`, preserving sheet backgrounds too.

Frame asset bytes: **23,645 B** (five PNGs) → **25,163 B** (twelve SVGs),
+1,518 B embedded source. Each SVG contains only its visible profile/casting,
with no off-canvas copies of the other corners. Before/after optimization renders
are pixel-identical at both 1× and 1.35×; the approved artwork is unchanged.

## Dependencies and follow-up

No new Cargo or runtime dependencies. Authoring uses Python's standard library.
Existing Pillow 12.3.0 was used only to assemble review PNGs; existing resvg
0.47.0 was used to inspect the source art, not to ship rasterized frame assets.

Owner approval ("Melhor") and coordinator design approval were relayed through
the design gate on 2026-10-19; production wiring followed that approval.

Merged green `main` (`68c4e5e`, including 045) in `72107ea`; the sole conflict
was resolved by retaining 045's `--wall-only` path and the vector-frame removal.
After final owner approval, merged main `305d644` (049 band fade) cleanly in
`0bc6562`. With the build slot granted, merged main `c4b6380` (038 catalog
TOML and documentation, plus the intervening main changes) cleanly in `38e4ca4`
before regenerating the README images. No protected source-of-truth files were
manually edited.

Proposal outside this task's ownership: update the obsolete `CONTRIBUTING.md`
UI-art exception that says the frame and ornaments remain PNG. They are now
script-authored SVGs; historical spec 035 reports should retain their history.


## Coordinator-authorized offline fixture audit

The coordinator expanded this task after the pre-merge categories journey
fetched a live catalog (13 embedded recipes versus 65 runtime cards). The main
fix adjusted count assertions, but `Environment { test: None }` still enabled
Runtime's production automatic update. No production behaviour was changed.

A `#[cfg(test)]` fixture helper persists `SetCatalogAutoUpdate(false)` through
the existing Services command before a Runtime can start. All five test
constructors without a test seam use it. The categories journey now asserts
updates are disabled, the catalog was not replaced, and `last_check` is `None`.

| Construction audited in `crates/ddl/src/` | Network isolation |
| --- | --- |
| `headless_production.rs::categories_journey` | Scratch state disables automatic updates before `attach_environment`; asserts no check was attempted. |
| `local_ui_tests.rs::environment` | Same fixture helper before any Runtime/service operation. |
| `services.rs::tests::env` | Same fixture helper. The cancellation test deliberately removes the seed to preserve its absent-file precondition; its cancelled direct Services command cannot dispatch Runtime's automatic check. The schema-migration test deliberately writes an old schema and still verifies the production default is on. |
| `services.rs::steam_wiring_tests` | Same fixture helper; startup only scans synthetic local IWADs. |
| `services.rs::store_wiring_tests::home_env` | Same fixture helper; startup scans synthetic local store records. |
| `headless_production.rs::scene`, `recipe_change_tests.rs::{setup,catalog_env}`, `services.rs::catalog_update_tests::environment`, `cli_exec.rs::tests::fixture` | Existing `TestEnvironment` uses canned release responses, fake engine runners and explicit loopback package/catalog endpoints; automatic production checks are disabled by the seam. |
| `runtime.rs::attach`, `cli_exec.rs::environment` | Production entry points, not invoked by the unit/headless test constructors; unchanged. |

`rg` audit covered every `Environment` construction, `test: None`,
`attach_environment` and catalog/engine HTTP dispatch in `crates/ddl/src/`.
Intentional HTTP transport tests use synthetic loopback servers, never public
upstream endpoints. Targeted Services suite: **36 passed**, including default
schema migration and the unchanged cancellation/no-write contract. The seven
local UI tests also pass with the offline fixture.


## Final production review on the seamless wall

Before images are the committed, headless-generated README baseline from green
main `68c4e5e`. After images are fresh Slint software-renderer PNGs from the
three production journeys, under `target/vector-frames/final-main/`; every
journey passed. All four screens were visually inspected. Home's pixels outside
the two 28 px frame perimeters are byte-identical to the main baseline.

| Screen | Before/after artifact (before left, after right) |
| --- | --- |
| Home | `target/vector-frames/review/home-before-after.png` |
| Discover | `target/vector-frames/review/discover-before-after.png` |
| Mapset detail | `target/vector-frames/review/mapset-detail-before-after.png` |
| Play setup | `target/vector-frames/review/play-setup-before-after.png` |
| Corner, nearest-neighbour 4× | `target/vector-frames/review/corner-before-after-4x.png` |

Native 1280×800 after images: `target/vector-frames/after/`.
The complete targeted output is `target/vector-frames/targeted-final.log`.
These review PNGs and logs are intentionally uncommitted.


## Final README refresh and validation

After merging `c4b6380`, `python3 scripts/readme-screenshots.py` passed all
three production journeys and regenerated the four losslessly compressed,
1280×800 README PNGs. All four refreshed images were inspected headlessly;
they combine the approved vector frames with main's 049 band fade and current
catalog. The earlier approved before/after artifacts remain under `target/`.

| README image | Bytes |
| --- | ---: |
| `docs/screenshots/home.png` | 288,661 |
| `docs/screenshots/discover.png` | 534,613 |
| `docs/screenshots/mapset-detail.png` | 478,384 |
| `docs/screenshots/play-setup.png` | 277,157 |

The final shell commands use `set -o pipefail` and propagate each command's
exit status. Logs are kept under `target/vector-frames/`: `final-readme.log`,
`final-fmt.log`, `final-clippy.log`, `final-workspace.log` and
`final-readme-check.log`. No visible window or screen capture was used.

All spec 048 acceptance criteria are complete; no implementation items remain
open. Actual Deck/femtovg presentation remains the hardware validation noted
above, with renderer compatibility established from pinned upstream source.

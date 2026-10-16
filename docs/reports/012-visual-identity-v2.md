# Spec 012 phase B — Ember Cathedral, intensified

Local implementation and headless review complete; physical Deck validation remains open.
This report covers R6–R7 and A3. A4 (Deck LCD readability / rendering speed, S3) is **Needs Deck
Validation**. No visible windows were opened and no screen was captured.

## Production changes

| Screen / state | Result |
| --- | --- |
| Shared shell | Bone Pirata One headings, Barlow body and controls, carved oxblood wall, 28 px bronze/stone nine-slice frames, bronze control bevels, separated amber focus rings, matching tabs and controller badges. Original broken-circle emblem and legible Barlow DDL wordmark replace the small blackletter logo. |
| Home | Cover and top art band; actual installed engine version and registry-backed base-game checks, explicit pending gameplay verification. Recently played shows up to two actual profile histories, or an honest empty state. Empty Home has its own subtitle; the readiness heading does not claim missing prerequisites are ready. |
| Discover / Library | Three original mapset covers, dedicated art band, opaque framed cards and short status messages that fit. Empty Library now has an opaque framed forward-action panel. No favorites or category features were added. |
| Mapset detail | Named title (`Details / SIGIL`), independent purpose subtitle, original cover and top art band, framed requirements/source area. Existing install/setup actions retained. |
| Play setup | Named title, art band, opaque component rows, top-aligned compact Will launch, singular/plural file counts, actual base-game state and engine version. Required order, advanced confirmation, profile persistence and independent options scrolling remain intact. |
| Install | Named subject in the page heading and panel, stage sequence, actual stage plus explanatory subtitle, IEC byte units, real transfer fraction and static indeterminate bar. Engine order is Download → Verify → Extract → Install; mapset order is Download → Extract → Verify → Install, matching existing service events. Failed installs stop claiming a download is in progress. |
| Loading / first run / IWAD review | Matching frames, fonts and opaque surfaces; first-run subtitle no longer repeats the engine panel. Review keeps candidate selection and content identification intact. |
| Directory / import browser | Case-insensitive names, visible directories before files, hidden entries last, stable ties; hidden names use readable muted text. Long breadcrumbs retain their destination with leading ellipsis. Shortcuts and existing navigation remain intact. |
| Settings | Matching framed navigation; folder, config and diagnostics panels size to their content. Empty folders explain the next action. Baseline schema has a meaningful label. Update copy is neutral before the explicit check, with no automatic request. |
| Toasts / dialogs / Playing | Opaque bronze-edged toasts, carved confirmation sheets, readable body copy and clear safe-default focus. Playing has an independent subtitle and continues to suppress input/hints while the child owns the session. |

`docs/design/ui.md` §2 now documents the Ember tokens and typography. Other interaction sections,
core behavior, protected product/state documents and dependencies were not changed.

## Assets, provenance and licences

All production PNGs live under `crates/ddl/ui/assets/ember/`. Source is the original phase-A
Ember atlas, retained in `docs/design/visual-v2/ember-cathedral/art-atlas.png`; its complete
built-in generation prompt remains in the adjacent `prompt.txt`. No new online generation or
third-party artwork was needed. These images are original concept art, not official covers,
actual map screenshots or compatibility evidence. Dedicated cover/band regions display the
“Original concept art” plaque; tiny recent thumbnails accompany a labelled main cover.

`build_assets.py` is the deterministic offline production compositor (Pillow 12.3.0, design-time
only). It crops the approved source, mirrors wall quarters to make matching opposite edges,
pre-darkens the wall by 38%, tiles the wallpaper, makes an opaque frame center and renders the
legible DDL lettering from Barlow. It asserts both tile seams and the PNG budget. Regenerate:

```sh
/tmp/ddl-ember-venv/bin/python crates/ddl/ui/assets/ember/build_assets.py
```

| PNG | Dimensions | Bytes | Composition |
| --- | --- | ---: | --- |
| `auger-band.png` | 400×88 | 64,315 | Offline composite from original art / bundled Barlow |
| `auger.png` | 512×586 | 436,684 | Original atlas crop |
| `emblem.png` | 64×64 | 8,837 | Original atlas crop |
| `frame.png` | 256×220 | 44,341 | Offline composite from original art / bundled Barlow |
| `logotype.png` | 116×56 | 1,051 | Offline composite from original art / bundled Barlow |
| `sigil-band.png` | 400×88 | 60,920 | Offline composite from original art / bundled Barlow |
| `sigil-ii-band.png` | 400×88 | 53,244 | Offline composite from original art / bundled Barlow |
| `sigil-ii.png` | 512×586 | 366,197 | Original atlas crop |
| `sigil.png` | 512×586 | 370,823 | Original atlas crop |
| `wall-tile.png` | 512×438 | 378,362 | Offline composite from original art / bundled Barlow |
| `wall.png` | 1280×632 | 568,501 | Offline composite from original art / bundled Barlow |

**PNG total: 2,353,275 bytes (2.35 MB)**, below the 6 MB target. The precomposited wallpaper
avoids tiling many runtime elements; the reusable seam-safe source tile is also retained.
Fonts plus notices add 169,188 bytes, for 2,522,463 added asset bytes excluding the compositor.
Existing Inter/furnace assets are retained for the earlier fixture history.

**Barlow Regular** (104,068 bytes) and **Pirata One Regular** (56,316 bytes) are unmodified copies
from `docs/design/visual-v2/fonts/`. Both use SIL OFL 1.1; separate Barlow/Pirata One notices are
bundled and now copied into the release tarball alongside Inter's notice. Verified Google Fonts
revision: `47d54ca1ebb14fd864695f1d39dc734f39017c33`:
[Barlow source and OFL](https://github.com/google/fonts/tree/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/barlow),
[Pirata One source and OFL](https://github.com/google/fonts/tree/47d54ca1ebb14fd864695f1d39dc734f39017c33/ofl/pirataone).
Font-family names are exempted from the UI-copy lint like the existing Inter family; visible
English copy still uses Slint translation resources. No Cargo dependency was added.

## Contrast and rendering

WCAG sRGB relative luminance, calculated with the retained phase-A compositor's `ratio` function:

| Foreground | Panel `#211719` | Raised `#352527` | Deep `#120C0D` |
| --- | ---: | ---: | ---: |
| Bone `#F8EAD5` | 14.75:1 | 12.26:1 | 16.35:1 |
| Muted `#D0BCA7` | 9.52:1 | 7.91:1 | 10.55:1 |
| Amber `#FFD078` | 12.11:1 | 10.06:1 | 13.42:1 |
| Success `#88D9B0` | 10.49:1 | 8.72:1 | 11.62:1 |
| Danger `#FFA69C` | 9.31:1 | 7.74:1 | 10.32:1 |

Primary-button dark text / amber: **13.42:1**. Every normal body pair exceeds 4.5:1, including
hidden folders. Decorative bronze lines are not text or the focus indicator. Text uses opaque
panels, rows, plaques and chrome; the wall is visible around them. Dialog dimming only affects
inactive background content. Focus retains its separate 4 px dark gap and 4 px amber ring.

Every production capture asserts `draw_if_needed` declines another frame when nothing changes.
There is no continuous animation. Production materials are pre-rendered PNGs and the built-in
Slint nine-slice path, without layered blur effects. `012-render-metrics.tsv` records software
render time per fresh 1280×800 RGB frame, excluding PNG encoding and comparisons. These are
unoptimized macOS arm64 host measurements, not a Deck benchmark; see final evidence below.

## Requirement evidence and visual review

- R6: shared Slint renderer applies the identity to all production states; original raster
  provenance and font licences are reviewed directly rather than by an automated originality
  claim. Headless rendering verifies complete state layouts and no idle repaint.
- R7 titles / plural / readiness / pre-check: production journey asserts `Details / SIGIL`,
  `Play setup / SIGIL`, `Play setup / AUGER;ZENITH`, `1 content file`, `2 content files`,
  real found/missing base games, Recently played and neutral update copy before any API request.
- R7 browser: service test `browser_sorts_without_case_and_places_hidden_entries_last` checks
  mixed-case names, visible folders, WAD files and hidden folders. Headless journey asserts
  hidden entries form the final group and render with muted tone. Unicode-safe leading path
  ellipsis and B/KiB/MiB/GiB boundaries have unit tests.
- R7 density / art / progress: regenerated production PNGs cover the actual shared renderer;
  stage sequence and byte labels have headless assertions. Visual inspection checks nonrepeating
  subtitles, opaque text surfaces, compact summaries, distinct art plaques and content-sized
  Settings panels. These visual requirements are inspected rather than approximated by pixel
  equality tests.

All `010-*.png` production renders and both `comparison-ember-*.png` images were inspected
through image tools, never via desktop capture. Review caught and fixed the clipped Discover
missing-game message, empty Library text over texture, repeated first-run/Playing subtitles,
stale receiving copy after install failure, and the folder-confirmation capture being taken
just after dismissal. The earlier fixture journey and focus/key assertions remain enabled.

Artifacts regenerate under `target/headless/` (or `DDL_HEADLESS_DIR`) with `cargo test -p ddl`.
The two Ember comparisons put production on the left and the approved Home / AUGER setup mockup
on the right at 2560×800. Production state names are listed in the final evidence below.

The service journey uses scratch XDG roots, generated ZIP members, loopback HTTP and a fake
engine runner. Discover's three genuine catalog entries and AUGER's ready comparison use clearly
marked render-only snapshots; AUGER borrows the fake SIGIL summary and synthetic base readiness
only for visual review, restores live state immediately, and never runs or claims to verify
AUGER gameplay. Fast Verify/Extract/Commit frames similarly inject stage notifications while
the test transfer is gated. Other service assertions continue exercising real adapters.

## Differences from the mockup

- More visible wall, thicker 28 px carved frames and top art bands follow D-021's intensification.
  The main layout remains the same header/content/footer and two-column setup skeleton.
- Barlow DDL lettering replaces the unreadable small blackletter wordmark; Pirata One stays on
  large titles and labels. Rounded production control/focus geometry preserves the existing
  Slint interaction implementation rather than substituting raster controls.
- Home uses actual history and prerequisite state, so the test has one recent SIGIL session and
  correctly says DOOM II is missing; the mockup's illustrative extra library items are not
  invented. “Readiness checks” avoids claiming an incomplete machine is ready.
- AUGER add-ons remain existing separate controller rows, preserving recipe-driven interaction
  and exclusivity. Advanced/gameplay rows scroll instead of replacing these with the mockup's
  compact segmented selector. Will launch stays fixed while options scroll.
- No illustrative transfer speed, completion estimate or gameplay-verification badge was added.
  Bytes and install stages derive from the existing events, including their distinct order.

## Remaining gates

**Needs Deck Validation:** S3 readability at handheld distance on the 1280×800 LCD, contrast in
real lighting, and software-renderer speed / idle CPU on the actual Deck. Local pixels and
no-repaint assertions cannot establish physical readability, GPU/backend behavior or power use.
No Linux release build, push, tag or release was performed. Font-notice packaging was inspected
in the workflow; executing the Linux release tarball pipeline remains CI-owned.

Mapset/base-game identities and proposed compatibility settings retain the existing pinned
research evidence in [SIGIL reference](../research/reference-sigil.md),
[upstream sources](../research/upstream-sources.md) and
[DSDA research](../research/dsda-engine.md). No engine flags, IWAD data, compatibility facts or
content hashes were introduced. No new open product question or dependency remains.

## Final local checks

Final macOS arm64 checks passed after the last code change: **309 tests**, zero failures.

`cargo fmt --all --check` — exit 0, no output.

`cargo clippy --workspace --all-targets -- -D warnings` — exit 0:

```text
Compiling ddl v0.0.2 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-012-ember/crates/ddl)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.02s
```

`cargo test --workspace` — exit 0, result lines:

```text
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 51.30s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 263 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Final tail:

```text
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

   Doc-tests ddl_core

running 3 tests
test crates/ddl-core/src/downloads/mod.rs - downloads (line 22) - compile ... ok
test crates/ddl-core/src/library/mod.rs - library (line 10) - compile ... ok
test crates/ddl-core/src/profiles/mod.rs - profiles (line 10) - compile ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.55s; merged doctests compilation took 0.30s
```

Final rendering sample: 33 production states; minimum 56.53 ms, median 70.53 ms, mean 71.38 ms, maximum 94.51 ms per fresh full frame. All 33 idle-repaint assertions passed. These unoptimized host timings are recorded for comparison, with no Deck performance claim.

Inspected production artifacts (all 1280×800):

`010-loading.png`, `010-first-run.png`, `010-home-empty.png`, `010-library-empty.png`, `010-quit-dialog.png`, `010-first-run-engine-install.png`, `010-cancel-dialog.png`, `010-directory-browser.png`, `010-iwad-ambiguous.png`, `010-iwad-found.png`, `010-discover.png`, `010-mapset-detail.png`, `010-install-progress.png`, `010-install-verify.png`, `010-install-extract.png`, `010-install-commit.png`, `010-play-setup-ready.png`, `010-play-setup-auger.png`, `010-compat-dialog.png`, `010-import-browser.png`, `010-play-setup-options-scroll.png`, `010-playing.png`, `010-engine-failed-toast.png`, `010-library.png`, `010-home-continue.png`, `010-play-setup-blocked.png`, `010-settings.png`, `010-settings-folders.png`, `010-settings-config.png`, `010-settings-diagnostics.png`, `010-reset-dialog.png`, `010-remove-folder-dialog.png`, `010-install-error.png`.

Inspected Ember comparisons (2560×800): `comparison-ember-home.png` and `comparison-ember-play-setup-auger.png`.

A3 is complete. A4 remains open for owner Deck checks; no local implementation item remains open.

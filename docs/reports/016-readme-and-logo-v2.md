# Spec 016 phase A — README, guide, screenshots and logo concepts

## Delivered scope

R1, R3, R4 and R5 only. No app behaviour, compiled artwork, Steam artwork or current
README logo changed. No new Rust or Python package dependency was added. The concept
export uses the same design-time resvg 0.47.0 tool as the existing logo workflow.

- [Cinder Key](../design/logo-v2/cinder-key/contact-sheet.png): pointed bronze escutcheon
  and bone keyhole; a strong cathedral silhouette, with a possible human-figure reading.
- [Rose Compass](../design/logo-v2/rose-compass/contact-sheet.png): octagonal rose window
  and four petals around an amber ember; strongest departure from Ember Gate, but can
  suggest a navigation app. Petals lose definition at tiny sizes.
- [Ember Folio](../design/logo-v2/ember-folio/contact-sheet.png): open carved map atlas
  with an amber bookmark; broad, readable library metaphor, with a possible reading-app
  association. Fine page details soften at 32 px.

Each has hand-authored filled-path emblem and horizontal SVG masters plus a vector
contact sheet directly exported to PNG. Sheets show both emblem and lock-up on dark and
light at 32, 64 and 256 px emblem heights. DDL lettering is newly drawn upright geometry;
the full name reuses the existing Barlow outlines under their bundled SIL OFL notice.
No image generation, franchise marks, embedded raster or external fonts were used.
See the [comparison and usage note](../design/logo-v2/README.md) for sizes and reproduction.

The [README](../../README.md) is 66 lines, with the current Ember Gate, one-sentence
pitch, four screenshots, six feature bullets, six install steps and the original licence,
trademark/non-affiliation and credit substance. The stale placeholder status was replaced
with a source-versus-release note and an explicit Needs Deck Validation statement.

The [Steam Deck guide](../steam-deck-guide.md) preserves the detailed install, manual Steam
artwork, controller/gyro, strict demo caveat, sensitivity, build, CLI and troubleshooting
instructions. Controller steps and tables were compared against the original README and
are verbatim except repaired relative links. FAQ, explicit logs/XDG guidance, pinned engine
source and a screenshot maintainer note were added. Deck checklist references now link to
its moved instructions; CONTRIBUTING had no moved-section links to repair.

## Verification approach

- R1: SVG element/reference checks, export dimensions and byte-equal regeneration;
  visual inspection of all three contact sheets at native scale. These are design choices,
  so recognisability and owner preference require visual judgement rather than unit tests.
- R3/R4: line, feature and install-step counts; preserved controller text comparison;
  local Markdown/HTML relative-link and GitHub heading-anchor checks. Source links remain
  pinned where DSDA facts are asserted; existing hardware caveats remain open.
- R5: existing real Slint software renderer, using scratch state, loopback content fixtures
  and a fake engine. Four 1280×800 RGB exports show Home, populated Discover with category
  strip/tags/favorite, Play setup and detail with tags/favorite. Compression changes no
  pixels. A fresh `--check` run compares the committed PNG bytes.

Screenshots are sample UI state, not gameplay evidence or actual map screenshots. No
visible windows were opened and no desktop screen was captured. No external downloads
are part of the screenshot or workspace test workflow.

## Commands and evidence

```text
cargo fmt --all --check
(exit 0; no output)

cargo clippy --workspace --all-targets -- -D warnings
    Checking slint v1.18.1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 28s

python3 scripts/check-readme-links.py
PASS: 32 local links/anchors in README and Steam Deck guide

python3 docs/design/logo-v2/export.py --resvg /tmp/ddl-logo-tools/bin/resvg --check
cinder-key: clean vector masters, 1440x1280, 82,472 bytes
rose-compass: clean vector masters, 1440x1280, 71,647 bytes
ember-folio: clean vector masters, 1440x1280, 67,758 bytes
```

Workspace test results (all exited successfully):

```text
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 113.12s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.23s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 298 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo test --workspace` tail:

```text
   Doc-tests ddl_core

running 3 tests
test crates/ddl-core/src/downloads/mod.rs - downloads (line 22) - compile ... ok
test crates/ddl-core/src/library/mod.rs - library (line 10) - compile ... ok
test crates/ddl-core/src/profiles/mod.rs - profiles (line 10) - compile ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 1.26s; merged doctests compilation took 1.05s
```

Fresh screenshot regeneration also passed byte equality:

```text
python3 scripts/readme-screenshots.py --check
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 58 filtered out; finished in 151.76s
docs/screenshots/home.png: 1280x800, 418,896 bytes
docs/screenshots/discover.png: 1280x800, 504,303 bytes
docs/screenshots/play-setup.png: 1280x800, 340,295 bytes
docs/screenshots/mapset-detail.png: 1280x800, 570,322 bytes
```

A3 is complete. Formatting, clippy with warnings denied, workspace tests, relative links,
concept exports and screenshot reproduction all passed.

All eight external URLs in README and guide also returned HTTP 200 in a separate
read-only request check. The pinned DSDA v0.30.1 configuration source was fetched and
checked for the controller enable key. Local verification is repeatable with the committed
link checker; it does not require a network connection.

| Screenshot | Dimensions | Optimised bytes |
| --- | --- | ---: |
| Home | 1280×800 | 418,896 |
| Discover | 1280×800 | 504,303 |
| Play setup | 1280×800 | 340,295 |
| Mapset detail | 1280×800 | 570,322 |

`git merge main` reported `Already up to date` at `3a4b916`. No app source was edited.

## Remaining owner/coordinator work

Owner choice remains pending. A1 is ticked for the assigned phase A concepts, with that
pending choice stated directly in the spec. The coordinator must record the choice in
`docs/decisions.md`; phase B applies it under R2/A2. No edit to decisions, status, journal,
OVERVIEW or other specs was made. Physical Deck checks for input handoff, gyro, strict
recording, crop/readability and gameplay remain Needs Deck Validation.


# Spec 016 phase B — apply Rose Compass

## Result

**Confirmed:** D-023's Rose Compass replaces Ember Gate in all production masters
under `docs/design/logo/` and byte-identical compiled SVG copies. The emblem is
byte-identical to the selected phase-A concept; the DDL contours are that concept's
upright broad-counter wordmark. Existing lock-up sizes and Barlow full-name outlines
remain. Header, loading and About references and embedded window-icon wiring already
point to these paths, so no UI code or behaviour change is needed.

The 256/512 px icons, portrait/wide capsules, transparent Steam logo, README top image
and 1280×640 social preview use Rose Compass. Original capsule/social scenery and
composition dimensions are preserved; the logo-free hero SVG and PNG are byte-identical.
The unchanged release workflow packages the six `docs/design/logo/steam/*.png` files
into `ddl/art/`; a scratch install check verified identical bytes and mode 0644.
No local release was built or published.

The usage note documents clear space, minimum sizes and production provenance.
Native 32/64/256 px contact-sheet inspection on dark/light showed four separated bone
petals and the amber centre; no alternate small-size emblem is needed. The production
154×56 px header lock-up retains the same emblem. Rose Compass concept masters remain
in `logo-v2/` as phase-A evidence; production geometry now lives in `logo/author.py`.
Old Ember Gate production vectors are retained only in Git history. The original raster
exploration and prompt in `logo/concepts/` remain for the historical spec 013 report.
The coordinator authorized wording-only guide updates to remove stale branding and
pending-choice statements. No new runtime, Cargo or Python dependency was introduced;
authoring uses the existing pinned design-time fonttools 4.60.1 and resvg 0.47.0.

## Verification

R2/A2 reuse the existing asset dimension/alpha/master-copy integration tests, embedded
window-icon assertions and headless loading/About journey assertions. The production
exporter additionally verifies byte equality against the selected Rose Compass emblem.
Visual originality and legibility require human inspection, rather than a unit test.
All verification uses off-screen Slint rendering; no visible window or screen capture.

Both author scripts reproduced all SVG bytes. Both production and concept export
`--check` commands passed. `python3 scripts/check-readme-links.py` passed all 32
local links/anchors. The social preview, portrait/wide capsules and 32 px/contact-sheet
exports were inspected directly. Original scenery groups were compared with the parent
commit, including exact byte equality for the unchanged hero PNG.

`cargo fmt --all --check` — exit 0, no output.

`cargo clippy --workspace --all-targets -- -D warnings` — exit 0, tail:

```text
   Compiling ddl v0.0.4 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-016b-logo/crates/ddl)
    Checking slint v1.18.1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 59s
```

`git merge main` reported `Already up to date` at `de61219`.


`cargo test --workspace` — exit 0, **398 tests passed, zero failures**:

```text
test result: ok. 65 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 115.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 307 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Final test tail:

```text

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s

   Doc-tests ddl_core

running 3 tests
test crates/ddl-core/src/library/mod.rs - library (line 10) - compile ... ok
test crates/ddl-core/src/downloads/mod.rs - downloads (line 22) - compile ... ok
test crates/ddl-core/src/profiles/mod.rs - profiles (line 10) - compile ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 1.21s; merged doctests compilation took 0.97s
```

Headless `target/headless/010-loading.png` and `013-about.png` were inspected at
1280×800: Rose Compass appears in the shared header, central loading panel and
About lock-up without clipping. All four regenerated README screenshots were also
inspected at native resolution; the header mark and DDL letters remain distinct.
`python3 scripts/readme-screenshots.py` exited 0 and regenerated the four 1280×800
images via the documented command:

| Screenshot | Bytes |
| --- | ---: |
| Home | 416,023 |
| Discover | 503,955 |
| Play setup | 336,048 |
| Mapset detail | 571,649 |


Fresh `python3 scripts/readme-screenshots.py --check` — exit 0:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 64 filtered out; finished in 90.97s
docs/screenshots/home.png: 1280x800, 416,023 bytes
docs/screenshots/discover.png: 1280x800, 503,955 bytes
docs/screenshots/play-setup.png: 1280x800, 336,048 bytes
docs/screenshots/mapset-detail.png: 1280x800, 571,649 bytes
```

All four screenshot exports reproduced byte-for-byte. `git diff --check` passed.
A2 is checked; all assigned phase-B work is complete and no product question remains.
No app behaviour, UI source, engine integration, dependencies, protected decisions or
coordinator status files were changed. Actual Steam custom-artwork crop, Gaming Mode
appearance and hardware input remain **Needs Deck Validation** (spec 013 A4).

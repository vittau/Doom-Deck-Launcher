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

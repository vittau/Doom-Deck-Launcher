# 013 — Ember Gate logo, icons and artwork

## Result and rationale

**Confirmed:** original clean SVG masters, compiled app usage, exported Steam artwork,
README branding and GitHub social preview implement R1–R5. The chosen **Ember Gate**
is a pointed bronze threshold surrounding one bone-lit ember, with an oxblood recess.
Broad contours and open negative space survive 32 px; inset rails, feet and diamond
engraving add detail at larger sizes. The upright slab DDL letters were drawn as
original contours, with a ceremonial ember underline; the full product name is
outlined from the existing OFL Barlow face. It avoids the poor small blackletter DDL
and keeps the established Pirata One screen titles.

The built-in image generation tool explored three ideas: cathedral gate, astrolabe
and basalt reliquary. The astrolabe lost clarity at small sizes; the reliquary felt
like a closed monument rather than an entrance. The gate expresses launching into
community worlds. The generated sheet and exact prompt are retained in
`docs/design/logo/concepts/`; it is exploration only. The final vectors were drawn
with new, simplified geometry in `author.py`, without tracing raster pixels or
using game art, franchise marks, screenshots, controllers or DOOM-derived fonts.

## Sources and exports

Masters under `docs/design/logo/`: `emblem.svg` (256×256), `horizontal.svg` (330×128),
`stacked.svg` (480×400), `full-name.svg` (700×160). Slint compiles identical copies
under `crates/ddl/ui/assets/logo/`. Usage, clear space, minimum sizes and light/dark
background guidance are in `docs/design/logo/README.md`.

Every delivered raster logo/art asset is a direct SVG export. The only generated
raster concept is explicitly segregated in `concepts/`; headless UI evidence comes
from Slint's off-screen software renderer, as required by A2. No deliverable PNG is
hand-edited. `contact-sheet.svg` → `contact-sheet.png` shows the exact same emblem
at 32/64/256 px on dark and light surfaces. `social-preview.svg` →
`social-preview.png` is 1280×640 for the owner to upload in GitHub settings.

Reproduction (offline after tool installation):

```sh
source ~/.cargo/env
cargo install resvg --version 0.47.0 --locked --root /tmp/ddl-logo-tools
python3 docs/design/logo/export.py --resvg /tmp/ddl-logo-tools/bin/resvg
python3 docs/design/logo/export.py --resvg /tmp/ddl-logo-tools/bin/resvg --check
```

`--check` re-renders to scratch, compares every PNG byte, verifies dimensions and
RGBA format, verifies the compiled SVG copies, and rejects text, images, external
references, fonts, filters and any element other than svg/g/path in the masters.
Authoring with `author.py` uses design-time `fonttools==4.60.1` and the existing
bundled Barlow font; exports need only Python's standard library and resvg. There
are **no new Cargo or runtime dependencies**. Original vector art uses the repo's
GPL-3.0-or-later licence. Barlow outlines retain the existing SIL OFL provenance
and notice at `crates/ddl/ui/assets/Barlow-OFL.txt`, already shipped in releases.

## App and recipe artwork

The shared Slint header uses `horizontal.svg` directly, the loading panel uses
`stacked.svg`, and Settings → **Diagnostics / About** uses `full-name.svg` beside
the application version. The existing diagnostics section remains the About
surface rather than adding a new navigation destination. Window `icon` binds to
an embedded 256×256 PNG exported from the emblem. All assets are compiled in; no
file lookup, online generation or runtime export is required.

The existing per-recipe art selector keeps SIGIL / SIGIL II / AUGER;ZENITH and adds
`eviternity` → ivory cathedral plus an oxblood generic threshold for every unknown
ID. Both have a **512×586 cover** and **400×88 top band**, exported from original
SVG scenes. The band uses the existing baked transition into the dark title area.
These are woodcut-style mood illustrations, explicitly labelled Original concept
art in the UI, not official covers or claims about map content. No recipe was
created or changed by this task. As requested by the coordinator, main was merged
(conflict-free merge `60ba5c2`), bringing in the real Eviternity recipe from
`fba5100` and updated recipe-count tests. The real embedded recipe selects art 3;
the production Discover capture includes its ivory art without substituting data.

## Steam packaging and size evidence

**Confirmed:** Valve's [Library Assets documentation](https://partner.steamgames.com/doc/store/assets/libraryassets)
(retrieved 2026-10-16) specifies capsule **600×900**, Library Header **920×430**,
hero **3840×1240**, and transparent logo up to **1280 wide and/or 720 tall**.
Valve's [Community and Client Icons documentation](https://partner.steamgames.com/doc/store/assets/community)
specifies shortcut icons **256×256 or 512×512**, PNG or ICO. These are intended
asset dimensions, not a claim that the custom-artwork file picker enforces those
sizes. DDL supplies a transparent 1280×720 logo canvas within those bounds.

`.github/workflows/release.yml` installs the six committed PNGs from
`docs/design/logo/steam/` to **`ddl/art/`** with mode 0644. The README's
**Make it look native in Steam** section explains manual shortcut icon, portrait
and Recent Games artwork, background and transparent logo selection. DDL never
writes Steam configuration. The hero contains no text; the logo is separate.
Linux release execution remains CI-owned; no local Linux release, push or tag
was attempted.

| Delivered file | Dimensions | Bytes |
| --- | --- | ---: |
| `crates/ddl/ui/assets/logo/icon.png` | 256×256 | 10,175 |
| `steam/icon-256.png` | 256×256 | 10,175 |
| `steam/icon-512.png` | 512×512 | 21,646 |
| `steam/library-capsule.png` | 600×900 | 40,470 |
| `steam/wide-capsule.png` | 920×430 | 37,198 |
| `steam/hero.png` | 3840×1240 | 129,576 |
| `steam/logo.png` | 1280×720 | 32,536 |
| `social-preview.png` | 1280×640 | 46,148 |
| `contact-sheet.png` | 900×720 | 37,885 |
| `ember/eviternity.png` | 512×586 | 31,275 |
| `ember/eviternity-band.png` | 400×88 | 9,233 |
| `ember/fallback.png` | 512×586 | 30,959 |
| `ember/fallback-band.png` | 400×88 | 8,793 |

Steam/social/contact files are relative to `docs/design/logo/`; embedded art files
are relative to `crates/ddl/ui/assets/`. The six Steam files total **271,601 bytes**.

## Verification and limits

- R1–R2 / A1: visual inspection of the contact sheet, complete lock-ups, capsules,
  social preview, ivory cover and both bands; exporter validates reproducibility
  and clean SVG structure. Originality and visual clarity are reviewed through
  provenance and inspection, not an automated copyright assertion.
- R3 / A2: the headless harness checks the embedded window icon is 256×256 and the
  Window icon property binds to it. Production journey asserts loading/About
  logo nodes and renders actual header/loading/About without visible windows.
- Recipe art: unit test verifies all four known IDs and several unknown IDs;
  production journey checks the real bundled Eviternity card, then substitutes render-only community/Eviternity IDs and asserts
  the selected art, renders both cards/bands, then restores the real snapshot.
  The substituted cards are labelled render fixtures, not an additional bundled recipe or install.
- R4–R5 / A3: `logo_assets` decodes every exported PNG, checks exact dimensions,
  opaque artwork and truly transparent icon/logo corners, and byte-identical
  compiled master/icon copies. Workflow packaging and manual README steps are
  inspected. No network is used in asset tests; the pre-existing production
  service journey uses a loopback fake server and scratch XDG state.

Headless evidence regenerates in `target/headless/` with `cargo test -p ddl`:
`010-loading.png`, `010-discover.png` (header), `013-about.png`,
`013-discover-fallback.png`, and `013-discover-eviternity.png`. The shared renderer
still checks no repaint at idle and preserves existing navigation/focus tests.
These images were inspected through image tools; no window was opened and no
screen was captured. A1–A3 are local gates; **A4 remains Needs Deck Validation**
for actual Steam custom artwork, crop, icon and Gaming Mode appearance. No other
open product question or dependency remains. Protected source-of-truth documents
were not authored by this task; their coordinator-owned main updates arrived
through the requested merge.

## Final post-merge command evidence

macOS arm64, pinned repository toolchain, after merging main and asserting the
real Eviternity recipe: **313 tests passed, zero failures**. A1–A3 are checked;
A4 is intentionally unchecked. `git diff --check` also passed.

`cargo fmt --all --check` — exit 0, no output.

`cargo clippy --workspace --all-targets -- -D warnings` — exit 0, tail:

```text
Compiling ddl-core v0.0.2 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-013-logo/crates/ddl-core)
    Checking ddl v0.0.2 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-013-logo/crates/ddl)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.35s
```

`cargo test --workspace` — exit 0, result lines:

```text
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 59.31s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 263 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Final test tail:

```text
test smoke::smoke_test_kills_a_child_that_outlives_the_timeout ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

   Doc-tests ddl_core

running 3 tests
test crates/ddl-core/src/downloads/mod.rs - downloads (line 22) - compile ... ok
test crates/ddl-core/src/library/mod.rs - library (line 10) - compile ... ok
test crates/ddl-core/src/profiles/mod.rs - profiles (line 10) - compile ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.42s; merged doctests compilation took 0.22s
```

The pinned export `--check` passed after the final SVG spacing change; all 13
exports reproduced byte-for-byte. A scratch packaging smoke check used the same
`install -m 0644` inputs as the workflow, archived them as `ddl/art/`, and verified
exactly the six expected PNGs with mode 0644. It did not build or ship a release.

All five named headless evidence images were inspected after the merge. In
`010-discover.png`, the real Eviternity recipe and authors accompany the ivory
cover; substituted fixture captures instead display “Render fixture — not catalog
metadata” to avoid implying official metadata or installation verification.

## Cover repaint

**Confirmed (2026-10-16):** the built-in `image_gen` tool painted new Eviternity
and generic fallback covers using the existing SIGIL / SIGIL II paintings only
as style references. Eviternity depicts an imagined ivory-and-gold celestial citadel
above hellfire; the fallback depicts a carved basalt/bronze gate onto a smoky
hellscape. These are original mood illustrations, not official title artwork,
map screenshots or claims about map content (D-020 / D-021). The existing UI's
**Original concept art** plaques and art-selection code remain unchanged.

The exact generation prompts (including the rejected initial fallback) are retained as `COVER_PROMPTS` in
`crates/ddl/ui/assets/ember/build_assets.py`. Generated 1172×1342 RGB originals
were fitted centrally with Pillow 12.3.0 / Lanczos to the existing **512×586**
portrait contract. The committed cover PNGs are now raster masters; generation
is nondeterministic, while offline asset builds preserve these masters and
reproduce **400×88** bands from a continuous `ImageOps.fit` crop centered at
`(0.5, 0.35)`, with the existing 144 px smooth dark entry fade. Regenerate with:

```sh
python3 -m venv /tmp/ddl-cover-venv
/tmp/ddl-cover-venv/bin/pip install Pillow==12.3.0
/tmp/ddl-cover-venv/bin/python crates/ddl/ui/assets/ember/build_assets.py
```

| Repainted PNG | Dimensions | Bytes |
| --- | --- | ---: |
| `eviternity.png` | 512×586 | 567,388 |
| `eviternity-band.png` | 400×88 | 63,556 |
| `fallback.png` | 512×586 | 566,351 |
| `fallback-band.png` | 400×88 | 64,159 |

`sips` confirmed dimensions; the compositor asserts the 600,000-byte budget
for each painted cover and band and the existing 6 MB overall budget. No new
Cargo/runtime dependency was added. The four old cover SVG sources were removed
after removing their only references in `author.py` / `export.py` (coordinator
approved scope extension). Logo, Steam, social and contact-sheet SVGs stay;
`export.py --check` with resvg 0.47.0 still reproduces all nine remaining PNGs
byte-for-byte. This section supersedes the vector-cover provenance and four
cover byte counts above; the earlier vector-logo provenance still applies.

Headless `cargo test -p ddl` passed (28 unit tests and 5 integration tests).
Inspected `target/headless/013-discover-eviternity.png`,
`013-discover-fallback.png` and `010-discover.png`: painterly architectural
texture, smoke, orange light and deep shadows read as one family with SIGIL,
SIGIL II and AUGER;ZENITH; the ivory citadel stays distinct and the generic gate
remains legible in the existing card crop. Both bands blend into their title
panels and all Original concept art plaques remain readable. No visible windows
or screen captures were used. Originality/style acceptance is a visual and
provenance review, not something a unit test can prove. Existing headless tests
cover ID selection and real/fixture Discover states. Rebuilding the assets twice
also reproduced all 20 Ember PNGs byte-for-byte.

`cargo fmt --all --check` — exit 0, no output.

`cargo clippy --workspace --all-targets -- -D warnings` — exit 0, tail:

```text
    Checking slint v1.18.1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 33s
```

`cargo test --workspace` — exit 0, **313 tests passed, zero failures**; result lines:

```text
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 68.65s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 263 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Final tail:

```text
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s

   Doc-tests ddl_core

running 3 tests
test crates/ddl-core/src/downloads/mod.rs - downloads (line 22) - compile ... ok
test crates/ddl-core/src/library/mod.rs - library (line 10) - compile ... ok
test crates/ddl-core/src/profiles/mod.rs - profiles (line 10) - compile ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.76s; merged doctests compilation took 0.56s
```

`git diff --check` passed. No repaint task item or open question remains;
physical Steam Deck appearance remains **Needs Deck Validation**, as before.


### Fallback review correction

Coordinator review found the initial fallback's tall gate slabs read as hard
vertical collage seams in the Discover card and band. The built-in `image_gen`
tool regenerated the entire fallback as **one continuous 1172×1342 painting**,
using SIGIL only as a style reference. The replacement shows a complete small
carved arch on a diagonal causeway in a continuous smoky basalt landscape;
short tapered supports replace the tall dividing slabs. The current prompt is
`COVER_PROMPTS['fallback']`; the rejected one is retained as
`COVER_PROMPTS['fallback-initial-rejected']`. Eviternity is unchanged.

The cover is a single central Lanczos fit to 512×586, and the 400×88 band is one
continuous crop from it with the existing title fade. No tiling, stitching or
side-by-side composition was used for either fallback asset. Inspected the
cover and band at full size: clouds, terrain and lighting continue across the
width without panel divisions. The table above records the replacement sizes,
both below 600,000 bytes. The UI's Original concept art plaques remain in place.

`cargo test -p ddl` passed again (28 unit tests + 5 integration tests; headless
journey 73.19 s). Inspected the regenerated `013-discover-fallback.png` at
1280×800: the Community map crop reads as one continuous landscape, with no
hard vertical panel divisions, and the top band has matching continuous depth.
The Original concept art plaques remain readable beside the unchanged Eviternity
and SIGIL II art. Full-size cover/band inspection and deterministic rebuild
checks passed; all 20 Ember PNGs reproduced byte-for-byte.

Review-fix formatting and Clippy both exited 0; formatting produced no output.
Clippy tail:

```text
   Compiling ddl v0.0.3 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/art-covers/crates/ddl)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.66s
```

Review-fix `cargo test --workspace` — exit 0, **313 tests passed, zero failures**:

```text
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 84.40s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 263 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.88s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Final tail:

```text
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

   Doc-tests ddl_core

running 3 tests
test crates/ddl-core/src/downloads/mod.rs - downloads (line 22) - compile ... ok
test crates/ddl-core/src/library/mod.rs - library (line 10) - compile ... ok
test crates/ddl-core/src/profiles/mod.rs - profiles (line 10) - compile ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 1.07s; merged doctests compilation took 0.86s
```

`git diff --check` also passed. No review-fix task items remain open; actual
Steam Deck appearance remains **Needs Deck Validation**.

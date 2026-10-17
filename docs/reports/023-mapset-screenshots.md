# 023 — Mapset screenshots, phase A (assets, recipe field, lint, embedding) — worker report

**Status:** A1 (R1, R2, R4) and A3 (fmt/clippy/tests) complete. A2 (R3, the Slint UI) is a
separate worker; A4 needs the physical Deck. No UI markup was touched: this branch delivers the
bundled images, the recipe link, the lint and the Rust seam the UI worker consumes.

Per the coordinator's revised plan, screenshots are **bundled** (D-024), not downloaded at
runtime: the earlier research/fetch/cache design was dropped before any runtime code was written.
`origin/main` (spec 022 uninstall, D-025, the Rose Window social preview) was merged before
finishing; the merge was clean.

## What changed

- **R1 — assets.** All **13 recipes** have bundled screenshots under
  `crates/ddl/ui/assets/screenshots/<recipe-id>/`: 2 each for eleven recipes and 1 each for
  `scythe` and `sigil-ii` (23 files, 1.1 MB total). Every file is a **640×360 JPEG** card crop
  (≤ 73 KB, well under the 250 KB target) with a `SOURCES.toml` beside it recording source URL,
  source page, credit, original size, retrieval date and the permission text found. No image
  file was committed outside that directory, and no full-size original was committed.
- **R2 — recipe field + lint.** New optional root array `[[screenshots]]` with `asset` (path under
  the screenshots directory) and `credit` (caption), schema-1 compatible and absent on recipes
  that have none. Lint (offline) checks the reference structurally (safe unique relative path,
  accepted extension, nonempty credit) and, when the screenshots directory is present, checks the
  file on disk: PNG/JPEG/WebP by **magic bytes**, ≤ 256 KiB, ≥ 640×360, pixel cap 4096×4096, and a
  matching `SOURCES.toml` entry with a credit and source URL. `crates/ddl-core/src/catalog/image.rs`
  reads only image headers (no pixel decode, no new dependency).
- **Build embedding / UI exposure.** `crates/ddl/build.rs` now also writes a
  `(recipe id, asset paths)` index to `$OUT_DIR/screenshots.rs`; `crates/ddl/src/screenshots.rs`
  exposes it as `for_recipe(id)` / `card(id)`. Slint's existing
  `embed_resources(EmbedResourcesKind::EmbedFiles)` embeds every file the `.slint` references with
  `@image-url`, so the images ship in the binary and show offline.
- **R4 — CONTRIBUTING.md.** New “Screenshots” section: how to source, crop (16:9 640×360 JPEG
  ≤ 250 KB), record `SOURCES.toml`, and reference the file from `[[screenshots]]`, plus the schema
  table row.
- **Research.** A “Screenshots (spec 023, D-024)” section appended to each
  `docs/research/reference-<id>.md` (new `reference-auger-zenith.md` and `reference-sigil-ii.md`,
  which had no reference file of their own).
- **New dependencies: none.** Image-header parsing is ~120 lines of bounded Rust; JPEG/WebP are
  accepted by lint and the committed assets are JPEG.

## Per-recipe screenshots

Every recipe has at least one; none had to be left out.

| Recipe | Files | Where the image came from | Credit |
| --- | --- | --- | --- |
| `alien-vendetta` | `01.jpg`, `02.jpg` | official site `doom2.net/av/screens.html` (direct) | Alien Vendetta team |
| `ancient-aliens` | `01.jpg`, `02.jpg` | release thread `doomworld.com/vb/thread/87784`, images on `i.imgur.com` (Wayback) | Paul "skillsaw" DeBruyne |
| `auger-zenith` | `01.jpg`, `02.jpg` | Doomer Boards thread `doomer.boards.net/thread/2176`, images on `i.imgur.com` (Wayback) | Doomer Boards Project 37 |
| `back-to-saturn-x-e1` | `01.jpg`, `02.jpg` | author site `esselfortium.net/images/btsx/e1_*.png` (direct) | Back to Saturn X team |
| `back-to-saturn-x-e2` | `01.jpg`, `02.jpg` | author site `esselfortium.net/images/btsx/e2_*.png` (direct) | Back to Saturn X team |
| `eviternity` | `01.jpg`, `02.jpg` | official shots `dfdoom.com/.../evi-post-*.jpg` (direct) | Eviternity team |
| `eviternity-ii` | `01.jpg`, `02.jpg` | official site `eviternity.dfdoom.com/.../promo-map*.jpg` (direct) | Eviternity II team |
| `going-down` | `01.jpg`, `02.jpg` | release thread `doomworld.com/vb/thread/65955`, images on `mediafire.com` (Wayback) | Cyriak Harris (mouldy) |
| `scythe` | `01.jpg` | Doom Wiki `MAP01: Get Going (Scythe)` start-screen capture (Wayback) | Erik Alm |
| `sigil` | `01.jpg`, `02.jpg` | official page `romero.com/sigil`, images on `images.squarespace-cdn.com` (direct) | John Romero |
| `sigil-ii` | `01.jpg` | Doom Wiki `E6M1: Cursed Darkness (SIGIL II)` start-screen capture (Wayback) | John Romero |
| `sunlust` | `01.jpg` | release thread `doomworld.com/forum/topic/68089`, image on `i.imgur.com` (Wayback) | Ribbiks & Dannebubinga |
| `valiant` | `01.jpg`, `02.jpg` | release thread `doomworld.com/vb/thread/71704`, images on `i.imgur.com` (Wayback) | Paul "skillsaw" DeBruyne |

Sourcing notes: Doomworld, DoomWiki, Doomworld's image host and ModDB all return 403 to automated
clients (Cloudflare); the release/wiki pages were read from Wayback Machine snapshots and the
**images themselves were fetched from their original URLs** (`i.imgur.com`, `mediafire.com`,
`esselfortium.net`, `dfdoom.com`, `doom2.net`, `images.squarespace-cdn.com`). Doomday/DoomWiki
pages that were not archived (e.g. the DBP37 DoomWiki map pages) were skipped in favour of the
archived ones. Every chosen image was visually checked to be a real first-person in-game view —
title screens, logos, box mock-ups and level-editor screenshots were rejected (an Eviternity
`ukiro.com` design-notes image turned out to be a GZDoom Builder screenshot and was replaced with
the official `evi-post-*.jpg` shots). Nothing is invented: every URL, size and hash-adjacent fact
was computed from the downloaded bytes.

## UI-facing API (for the spec 023 R3 worker)

1. **Where the images are.** `crates/ddl/ui/assets/screenshots/<recipe-id>/01.jpg`, `02.jpg`, …
   (640×360 JPEG). The `.slint` asset root is `crates/ddl/ui/`, exactly like the painted covers in
   `crates/ddl/ui/assets/ember/`, so a reference is
   `@image-url("assets/screenshots/<recipe-id>/01.jpg")`.
2. **Which image and what caption.** `ddl_core::catalog::Recipe::screenshots: Vec<Screenshot>`
   (`Screenshot { asset, credit }`), already part of the embedded catalog. `asset` is the path
   under `crates/ddl/ui/assets/screenshots/` (so the `@image-url` path is
   `assets/screenshots/<asset>`); `credit` is the caption, to be shown as
   `Screenshot: <credit>` instead of “Original concept art”. The first entry is the card image.
3. **Rust seam** (deterministic, no directory walking at runtime): `crates/ddl/src/screenshots.rs`
   (generated by `build.rs`) exposes `for_recipe(id) -> &'static [&'static str]` and
   `card(id) -> Option<&'static str>` with paths relative to the `ui/` asset root. Its test asserts
   that the index, the on-disk files and every recipe's `[[screenshots]]` list agree, so the R3
   wiring cannot drift from the catalog. (The module is `#[allow(dead_code)]` until R3 consumes it.)
4. **Fallback.** A recipe with no `[[screenshots]]` keeps the painted cover and its current
   “Original concept art” plaque; with the current catalog every recipe has a screenshot, so the
   painted art stays only for a future recipe that has none.
5. **Crop behaviour.** R3 owns the rendering (`image-fit: cover`-style crop with a centred focal
   point). The committed images are already 16:9, so a 16:9 card or band crop needs no further
   letterboxing; the 400×88 band in `build_assets.py` crops vertically from them.

## Verification

- `cargo fmt --all --check` — pass.
- `cargo clippy --workspace --all-targets -- -D warnings` — pass (no new dependency, no allow
  beyond the documented `#![allow(dead_code)]` on the pending-R3 module).
- `cargo test --workspace` — all my tests and the whole suite pass **except one pre-existing failure
  on `main`** (see below). Totals excluding it: 82 + 2 + 2 + 1 + 1 + 322 + 16 + 3 tests, 0 failures.
  New tests: header parsing (PNG/JPEG/WebP, truncated/unknown), four structural screenshot lint
  cases in the mutated-recipe table, a filesystem test (valid, missing, oversize, too-small,
  uncredited), and `crates/ddl` index-vs-catalog consistency.
- `cargo run -p ddl -- catalog lint catalog/` → `Catalog lint passed (offline).`
- `cargo run -p ddl -- catalog lint --fetch catalog/` → downloads, verifies the SHA-256 of every
  package and named member of all 13 recipes (including the 121 MiB Eviternity II archive) and
  passed.

```text
$ cargo run -q -p ddl -- catalog lint catalog/
Catalog lint passed (offline).

$ cargo run -q -p ddl -- catalog lint --fetch catalog/
Catalog lint passed (fetched packages verified).
```

## Pre-existing failure on main (not from this branch)

`cargo test --workspace` has one unrelated red test on `origin/main`:
`crates/ddl/tests/logo_assets.rs::steam_artwork_and_social_preview_have_contract_dimensions`
panics because `docs/design/logo/social-preview.png` is an **Indexed** PNG while the test requires
**Rgba**. That file and test were committed on `main` by `5f41f3d` ("Compose Rose Window social
preview", an ancestor of `origin/main`); my branch never touched either. Fixing it means
re-exporting that asset from whoever owns the social preview. Everything else is green:
`cargo test --workspace -- --skip steam_artwork_and_social_preview_have_contract_dimensions`
passes.

## Left open

- **A2 (R3, Slint UI):** cards, Discover, detail and the art band still render the painted covers;
  the credit caption and the off-screen renders belong to the UI worker. The seam above is ready.
- **A4 (Needs Deck Validation):** whether the 640×360 JPEGs look sharp on the Deck panel.
- **Sourcing depth:** `scythe`, `sigil-ii` (and any recipe with one file) would benefit from a
  second shot; `going-down`'s only public stills are 515×269, so its card is a mild upscale.
- **Permission:** the `SOURCES.toml` files record that no explicit image licence was found for
  most sources; D-024's “removed on the author's request” applies. If the owner wants stronger
  licence footing for a specific mapset, that recipe's entry is the place to record it.

## Open questions for the coordinator

1. `crates/ddl/build.rs` and `crates/ddl/src/main.rs` gained one line each (`mod screenshots;`) —
   a trivial conflict against the spec 022 worker at worst.
2. I ticked **A1** and **A3** in `docs/specs/023-mapset-screenshots.md`; **A2** (UI) and **A4**
   (Deck) are left for their owners.

## Phase B — show bundled screenshots in the UI

**Status:** R3 / A2 complete; final fmt, clippy and workspace tests pass. A4 remains open.

### Implementation (R3)

- Home (hero and history), Discover and Library cards, mapset detail, and the art band now
  resolve the recipe's **first declared screenshot** and show `Screenshot: <credit>`. Images
  use `image-fit: cover` with explicit horizontal/vertical centre alignment. No runtime
  network or filesystem image loading is needed. Detail keeps the same first image; a
  carousel or additional controller action would add little value here.
- Bone text on opaque `Tokens.deep` plates, with bronze borders, protects the caption from
  bright skies and busy imagery. Caption plates size themselves to the wrapped text, including
  the longer credits on the narrower Home hero. Home history uses a separate two-line plate below
  each small thumbnail; its layout was tightened to keep the plate inside the carved frame.
- Missing screenshot metadata, unknown/unbundled assets, and future recipes without bundled
  files retain the painted cover and `Original concept art` caption. Existing catalog updates
  cannot turn an arbitrary remote asset path into a filesystem image load.
- Both caption strings live in `ui/wiring-strings.slint` and use `@tr`; the screenshot credit
  is formatted through the existing Rust `wiring_copy::value` path. The temporary module-wide
  `allow(dead_code)` in `screenshots.rs` is removed.
- `docs/design/ui.md` describes the screenshot and fallback treatment. No catalog, screenshot
  JPEG, source manifest, Settings or catalog-update implementation was edited.

### Embedding choice

`build.rs` extends phase A's sorted directory index with an **OUT_DIR-generated Slint global**,
imported as `@screenshots` using Slint's library-path configuration. Its deterministic lookup
maps the indexed asset paths to literal `@image-url` expressions. Slint's existing
`EmbedResourcesKind::EmbedFiles` embeds and decodes the images using the support already in the
application; Rust sends the checked asset key and translated credit through `ViewNode` and the
band properties. Recipe metadata, rather than directory order, chooses the first image. The
lookup includes the remaining bundled shots so reordering a recipe works without another
hand-maintained mapping. Generated files are only rewritten when their contents change to
avoid retriggering Cargo through the generated Slint import.

**New dependencies: none.** In particular, no additional image-decoding crate was added.

### Requirement coverage

The existing off-screen software-renderer journey now asserts that all thirteen embedded
recipes resolve to a 640×360 Slint image, the exact translated caption, and matching card,
detail and art-band properties. It writes one focused Discover render per recipe, covering
all scroll positions, plus bright-sky, long Home credit and metadata-absent fallback fixtures.
It also checks recipe screenshot reordering and an unbundled asset fallback. Existing Home
and Library journeys assert their image nodes carry screenshots and captions. These are
render-only recipe mutations; no live catalog or user state is changed.

### Headless visual review (A2)

Off-screen PNGs are under `target/headless/` (not committed). I inspected all thirteen cards
across the focused Discover renders, including these full rows:

- `023-discover-ancient-aliens.png` — Alien Vendetta, Ancient Aliens, AUGER;ZENITH.
- `023-discover-back-to-saturn-x-e1.png` — both Back to Saturn X episodes, Eviternity.
- `023-discover-eviternity-ii.png` — Eviternity II, Going Down, Scythe.
- `023-discover-sigil.png` — SIGIL, SIGIL II, Sunlust.
- `023-discover-valiant.png` — Valiant.

I also inspected the detail/band compositions (`010-mapset-detail.png`,
`023-detail-ancient-aliens.png`, `023-detail-auger-zenith.png`), Home and Library cards
(`010-home-continue.png`, `010-library.png`), and `023-discover-fallback.png`. The explicit
`023-discover-white-sky.png` / `023-detail-white-sky.png` fixture reorders Ancient Aliens' two
shots so its white-cloud image stress-tests the same plate; shipped recipes still use their
first declared shot. `023-home-long-credit.png` checks the longer Back to Saturn X credit on
both the narrow hero and the history row. Captions remain readable against the bright sky,
neon/fire and dark scenes, and the complete long credit stays inside its plate.

No visible window was opened and no screen was captured. All images came from Slint's
software renderer. A2 is ticked; A4 remains **Needs Deck Validation**.

### Final verification

Validated on macOS arm64. `main` at `aca466b` was merged cleanly and checked again before
completion (already up to date). Linux CI and the physical Deck were not run here. No new
dependencies were added.

All **430 tests** pass: 82 launcher tests, 7 launcher integration tests, 322 core tests,
16 process/smoke tests and 3 doctests. The existing headless journey contains the new screenshot
assertions and render fixtures, so no second UI platform runs concurrently in the test process.
Command tails:

```text
$ cargo fmt --all --check
(exit 0; no output)

$ cargo clippy --workspace --all-targets -- -D warnings
    Checking ddl v0.0.4 (.../crates/ddl)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.62s

$ cargo test --workspace
...
test result: ok. 82 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 122.59s
...
test result: ok. 322 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.87s
...
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
...
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

README images were regenerated with the documented spec 016 R5 command and all four were
visually inspected. They also byte-match the optimized PNGs from the independent final
workspace run (`DDL_HEADLESS_DIR="$PWD/target/headless/workspace" cargo test --workspace`):

```text
$ python3 scripts/readme-screenshots.py
...
docs/screenshots/home.png: 1280x800, 337,259 bytes
docs/screenshots/discover.png: 1280x800, 487,551 bytes
docs/screenshots/play-setup.png: 1280x800, 330,100 bytes
docs/screenshots/mapset-detail.png: 1280x800, 542,884 bytes
```

### Remaining observations

- Physical Deck sharpness/readability is still A4, outside this phase's acceptance.
- Existing long mapset titles can clip in Home/Discover text slots (visible in the Back to
  Saturn X render fixture); the screenshot credits themselves wrap fully. This predates R3
  and was left outside the screenshot change.
- Proposed documentation follow-up outside this task's ownership: the guide's FAQ
  (`docs/steam-deck-guide.md`, “Are the screenshots actual gameplay?”) still calls the card
  images original mood art. It should distinguish the off-screen launcher UI captures from
  their now-real, credited in-game mapset images.

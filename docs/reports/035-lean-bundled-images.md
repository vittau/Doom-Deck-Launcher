# Report 035 — Lean bundled images: JPEG, at most 1280×800

Branch `vittau/lean-images`. Spec: `docs/specs/035-lean-bundled-images.md`.

## What changed

- **Lint (R1).** `crates/ddl-core/src/catalog/image.rs`, `lint.rs`, `mod.rs`. Bundled screenshots must now be
  JPEG, 640×360 up to **1280×800**, and at most **192 KiB (196 608 bytes)**. A PNG or WebP body is rejected
  ("must be JPEG") even under a `.jpg` name; `.png` / `.webp` asset names are rejected by the recipe rule;
  the old `MAX_SCREENSHOT_PIXELS` bound is replaced by the explicit 1280×800 bound (≈1 Mpixel, so the
  decompression-bomb guard stays). Messages point to `scripts/lean-image.sh`.
- **Byte cap, and why 192 KiB.** It is 1.5 bytes per pixel of a full 1280×800 frame, a quarter below the old
  256 KiB. Measured with `sips` at quality 60: DOOM screenshots enlarged to 1280×720 encode to 70–160 KB
  (from the committed 640×360 files; real 1280-wide sources are sharper, so these are lower bounds), and the
  densest textured art in the repo (the 1280×632 Ember wall) to 181 KB. A file over 192 KiB was saved above
  quality 60 or is unusually noisy; the contributor lowers the quality.
- **UI art (R2).** All opaque rasters became JPEG q60: 5 covers (`sigil`, `sigil-ii`, `auger`, `eviternity`,
  `fallback`), their 5 `-band` strips and `wall` (`ember/*.jpg`); `app-window.slint` paths updated (the only
  `.slint` with image references). Alpha was checked per file with `sips -g hasAlpha`: only `logotype.png`
  had real alpha, and it was unused.
- **Removed (unreferenced in every `.slint` and Rust file):** `furnace-art-sheet.png` (2.6 MB),
  `ember/wall-tile.png`, `ember/emblem.png`, `ember/logotype.png`. `docs/design/**` has its own copy of the
  furnace sheet and was not touched; README screenshots untouched.
- **Kept as lossless PNG (deviation from R2, agreed by the coordinator):** `ember/frame.png` (9-slice) and
  the four `ember/ornament-*.png`, 24 KB in total, no alpha. JPEG would save only about 12 KB, would blur
  their 1-pixel bevels, and `headless_production.rs` asserts the ornament renders pixel-exact against the
  PNG. The exception is stated in `CONTRIBUTING.md`.
- **`ember/build_assets.py`** now writes JPEG q60 for the opaque outputs and no longer emits the unused
  files. It needs Pillow, which is not installed here, so it was only byte-compiled, not re-run.
- **Docs.** `CONTRIBUTING.md` Screenshots section (rule, one-line command, UI-art paragraph) and a
  checklist line in `.github/pull_request_template.md`.
- **Existing 23 screenshots** (640×360 JPEG, 29–65 KB) already satisfy the rule and were not re-encoded
  (a second lossy pass only adds loss).

## Tooling (R4)

`scripts/lean-image.sh <input> <output.jpg> [quality=60]`: macOS `sips`, falling back to ImageMagick on
Linux. Shrinks to fit 1280×800 (never enlarges), keeps aspect, strips the colour profile, flattens alpha.
Checked on a 1536×1024 PNG (→ 1200×800), a 3840×1240 PNG (→ 1280×413) and a 1280×632 PNG (unchanged size).
It was also used for the conversion above. The ImageMagick branch is untested here (not installed).

## Size (R5)

| | before | after |
| --- | --- | --- |
| `crates/ddl/ui/assets/` (all files, git) | 8 337 459 B | 2 798 765 B (−66 %) |
| bytes embedded in the binary (referenced files: art, screenshots, fonts, svg, icon) | 5 272 210 B | 2 760 166 B (−2.51 MB, −48 %) |
| of which opaque UI art | 6 215 044 B PNG (incl. 4 unreferenced) | 652 920 B JPEG + 23 645 B PNG |

`build.rs` uses `EmbedResourcesKind::EmbedFiles`, so the release binary shrinks by the embedded delta
(≈2.5 MB). The release binary itself was **not** built before/after: it would cost two release builds on a
shared machine. Fonts (1.04 MB) are now the largest remaining item.

## Acceptance

- **A1:** `screenshots_lint_enforces_jpeg_1280x800_and_the_byte_cap` (PNG, oversize JPEG on each axis and at
  1920×1080, one byte over the cap are rejected; exactly 1280×800 and exactly 192 KiB pass),
  `screenshots_lint_rejects_non_jpeg_asset_names`, plus the existing lint tests. `cargo run -p ddl --
  catalog lint` → "Catalog lint passed (offline)" with all 13 recipes.
- **A2:** the headless journeys (`cargo test -p ddl headless`, 256 PNGs at 1280×800) were rendered before
  and after. I looked at `013-discover-fallback` and `010-mapset-detail` before/after side by side:
  indistinguishable at 100 %. Numeric diff over 9 key screens (discover, detail, fallback and Eviternity
  covers, first run, empty home/library, play setup, settings): PSNR 39.3–44.3 dB, 99.9 % of sampled
  bytes within ±24 levels. The worst is the fallback discover render (39.3 dB, max |Δ| 72 on a few
  painted-edge pixels). That is compression noise only; layout is identical. The ornament native-size
  pixel assertion still passes. Renders are in the session scratchpad, not committed.
- **A3:** script documented in `CONTRIBUTING.md` and its header; numbers above.
- **A4:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` (139 ddl + 400 ddl-core + integration) all green, after merging `main`
  (spec 034 included).

## New dependencies

None.

## Open questions / left open

- Quality: q60 was chosen (one step below the spec's start of 70). q50 also looked clean on the fallback
  cover and would save about 150 KB more across the UI art; I stayed conservative for the Ember look.
- The binary-size win is derived from the embedded asset delta, not from two release builds.
- Proposal for `docs/decisions.md` (not edited): record "bundled images: JPEG q60, ≤ 1280×800, screenshots
  ≤ 192 KiB; alpha art and the 9-slice/ornament sprites stay PNG".
- For spec 037: follow the new rule — `scripts/lean-image.sh <original> crates/ddl/ui/assets/screenshots/<id>/01.jpg`,
  final cap **192 KiB**, JPEG only.

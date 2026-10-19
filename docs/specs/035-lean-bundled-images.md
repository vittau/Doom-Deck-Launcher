# 035 — Lean bundled images: JPEG, at most 1280×800

**Status:** Ready (after 031, 032)  ·  **Milestone:** M3  ·  **Depends on:** 012, 023
**Origin:** owner, 2026-10-19: "As screenshots podem ser otimizadas, salvando em max 1280x800 …
e usar uma qualidade relativamente baixa JPEG … isso reduzirá o bundle quando forem muitos
assets." Every image in `crates/ddl/ui/assets/` is embedded in the `ddl` binary, and the catalog
grows by community PRs (D-010), so per-image cost multiplies.

## Facts (measured 2026-10-19)

- `crates/ddl/ui/assets/` is 8.1 MB. The 23 mapset screenshots are already 640×360 JPEGs
  (~45 KB each, 1.1 MB total). The rest is UI art: `furnace-art-sheet.png` 1536×1024 / 2.5 MB,
  `ember/*.png` 512×586 / 360–550 KB each, `ember/wall.png` 1280×632 / 555 KB, fonts, logo.
- Catalog lint (`crates/ddl-core/src/catalog/image.rs`) accepts PNG, JPEG or WebP up to 256 KB.

## Requirements

- **R1** — **Screenshot rule:** bundled mapset screenshots are JPEG, at most 1280×800 pixels, at a
  relatively low quality (start at quality 70 and go lower while a 1280×800 headless render on
  the card and detail still looks clean). Catalog lint enforces JPEG, the 1280×800 bound and a
  lower byte cap derived from that quality (state the number and why), so community PRs follow
  it. Update `CONTRIBUTING.md` and the PR template with the rule and a one-line command to
  produce such a file.
- **R2** — **UI art:** every raster in `crates/ddl/ui/assets/` that has no real transparency
  becomes a JPEG at most 1280×800 at the same quality; images that need alpha stay PNG but are
  reduced to at most 1280×800 and palette-quantized or recompressed losslessly. Unused assets
  (check every reference in `.slint` and Rust) are removed. Update `.slint` image paths.
- **R3** — **No visual regression:** before/after headless renders of every root screen, Mapset
  detail, Play setup and first run; differences must be compression-level only.
- **R4** — **Tooling:** a small script under `scripts/` (or a `ddl` dev subcommand) that converts
  an input image to the rule, using tools already available (macOS `sips` or an existing Rust
  dependency); no new crate unless justified in the report.
- **R5** — **Report the win:** release binary size (or the embedded asset total) before and after.

## Out of scope

`docs/design/**` mockups and README screenshots are not shipped in the binary; leave them.

## Acceptance criteria

- [ ] **A1** (R1) — lint tests: a PNG, an oversize JPEG and an over-cap JPEG are rejected; all 13
  recipes pass.
- [ ] **A2** (R2, R3) — renders inspected; asset directory size before/after in the report.
- [ ] **A3** (R4, R5) — script documented; size numbers in the report.
- [ ] **A4** — fmt, clippy `-D warnings`, tests green.

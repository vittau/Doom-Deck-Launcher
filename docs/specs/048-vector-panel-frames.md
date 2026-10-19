# 048 — Vector panel frames and ornaments

**Status:** Ready (after 045)  ·  **Milestone:** M3  ·  **Depends on:** 012, 035, 045, D-020, D-021
**Origin:** owner, 2026-10-19, on a panel corner where the bronze corner piece does not meet its
rails: "Essas tiles também não dão match. Melhor refazer isso em SVG pra garantir consistência."
Today every panel draws `assets/ember/frame.png` as `nine-slice(28)` (128×128, cut from the
painted atlas by `build_assets.py`, whose corners and stretched rails come from different spots of
the painting) plus four raster ornaments (`ornament-{top,bottom,left,right}.png`)
(`crates/ddl/ui/app-window.slint` lines 19–23).

## Requirements

- **R1** — **Vector frame:** replace `frame.png` and the four ornament PNGs with vector art
  authored by a script (one source of truth, like `docs/design/logo/author.py`): carved bronze
  rails, corners and the top/bottom/side ornaments of the Ember Cathedral look (D-021), drawn so
  every corner meets its rails exactly — same profile, same bevel, same gradient direction — at
  any panel size. SVG only (gradients allowed; no embedded raster, no filters that Slint's
  renderers do not support).
- **R2** — **How Slint draws it:** confirm from Slint 1.18 docs/source (cite, pinned) whether
  `nine-slice` works on an SVG `@image-url`, on both the femtovg and the software renderer and at
  the Deck's fit scale (spec 029). If it does, use one `frame.svg` with `nine-slice`; if not,
  compose the frame from corner SVGs and stretchable rail SVGs (or Slint `Path`s) so nothing is
  scaled non-uniformly where the profile would distort. Same frame thickness (28 px) and ornament
  sizes as today, so layouts do not move.
- **R3** — **Look:** at least as rich as today's frame (D-020: detailed, tasteful). Produce a
  side-by-side before/after headless render of Home, Discover, Mapset detail and Play setup plus a
  4× crop of one corner, for coordinator review; the coordinator shows them to the owner before
  merge.
- **R4** — Remove the frame/ornament code from `build_assets.py` and the unused PNGs; refresh the
  README screenshots.

## Acceptance criteria

- [ ] **A1** (R1, R2) — the authoring script regenerates the SVGs byte-identically; a test or
  script checks the corner/rail seams (pixel rows across each joint in a render at 1× and at the
  Deck fit scale show no step or colour jump).
- [ ] **A2** (R3) — the before/after renders and the corner crop exist (under `target/`, not
  committed) and the coordinator and the owner approved them.
- [ ] **A3** — fmt, clippy `-D warnings`, tests green; `readme-screenshots.py --check` passes.

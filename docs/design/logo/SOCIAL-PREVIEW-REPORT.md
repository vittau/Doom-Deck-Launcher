# Social preview v2 — worker report

> Historical (spec 013/016, D-023 mark). Spec 039 re-composed all three previews with the
> v3 Stained Glass logo; the scenes, layout and copy described below are unchanged.

## Result

Replaced the geometric border with **Rose Window**, an original carved oxblood
wall and bronze octagonal recess that gives the production Rose Compass a
deliberate architectural setting. The upright DDL wordmark, outlined product
name and two-line tagline explain the launcher at link-card size:
“Curated DOOM mapsets for Steam Deck.”

Explored and retained two other original directions under
`social-preview-variants/`: **Threshold**, a portal toward a distant cathedral,
and **Cartographer's Reliquary** (`atlas`), a relief map with bronze routes and
bridges. Rose Window was selected because the background reinforces the
specific brand emblem rather than depicting a generic fantasy destination.

All changes are inside `docs/design/logo/`. The production identity contours,
compiled assets and Steam artwork are unchanged. No runtime dependencies were
added. Pillow 12.3.0 is a design-time export dependency for deterministic palette
encoding and contrast checks; fonttools 4.60.1 remains the authoring dependency.
The generated scenes are unaltered source PNGs, produced with the built-in
image_gen tool without input images. Full prompts and provenance are committed
in `social-preview-layers/prompts.json`; they contain no borrowed franchise or
third-party mapset assets. All lettering and logo geometry are SVG paths.

## Verification

| Direction | Output | Dimensions | Bytes | Tagline contrast lower bound |
| --- | --- | --- | ---: | ---: |
| Rose Window — chosen | `social-preview.png` | 1280×640 | 458,141 | 10.62:1 |
| Threshold | `social-preview-variants/threshold.png` | 1280×640 | 427,772 | 8.36:1 |
| Cartographer's Reliquary | `social-preview-variants/atlas.png` | 1280×640 | 475,533 | 6.90:1 |

- `export.py --resvg /tmp/ddl-logo-tools/bin/resvg --check` passed with
  resvg 0.47.0 and Pillow 12.3.0, including every existing export and master-copy
  comparison. Checks cover byte equality, dimensions, ≤1,000,000 bytes per social
  PNG, exact brand colours, production contours, safe local source references,
  and contrast over the complete tagline region.
- Re-running `author-social.py` reproduced all three SVGs byte for byte.
- Inspected the final PNGs from files and a temporary 640×320-per-preview
  contact sheet. The 34 px tagline reads at 17 px in the half-scale versions.
  No visible windows were opened and no screen capture was used.
- Python sources compile; `git diff --check` passed. Rust tests were not run:
  this task changes design artifacts and offline export tools only, and its
  requirements are covered by the export checks and visual inspection.

## Remaining work

No task items or open questions remain. The README documents reproduction and
the selected direction. The owner can upload `social-preview.png` in GitHub's
repository social-preview settings; nothing was pushed or published.
Model generation itself is not byte reproducible; exports from the committed
source layers are reproducible offline.

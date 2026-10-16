# Phase A verification

All checks below ran headlessly on macOS arm64. No window opened and no screen captured.

## Dimensions (actual sips output)

```text
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-012-visual/docs/design/visual-v2/bastion-terminal/home.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-012-visual/docs/design/visual-v2/bastion-terminal/play-setup-auger.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-012-visual/docs/design/visual-v2/bastion-terminal/style-sheet.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-012-visual/docs/design/visual-v2/cinder-1993/home.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-012-visual/docs/design/visual-v2/cinder-1993/play-setup-auger.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-012-visual/docs/design/visual-v2/cinder-1993/style-sheet.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-012-visual/docs/design/visual-v2/ember-cathedral/home.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-012-visual/docs/design/visual-v2/ember-cathedral/play-setup-auger.png
  pixelWidth: 1280
  pixelHeight: 800
/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-012-visual/docs/design/visual-v2/ember-cathedral/style-sheet.png
  pixelWidth: 1280
  pixelHeight: 800
```

## PNG sizes

All 12 PNG files, including source atlases, are below 2,000,000 bytes. `sips` verified PNG dimensions; filesystem byte counts verify file size.

| File | Bytes |
| --- | ---: |
| bastion-terminal/art-atlas.png | 977,999 |
| bastion-terminal/home.png | 374,382 |
| bastion-terminal/play-setup-auger.png | 276,482 |
| bastion-terminal/style-sheet.png | 361,700 |
| cinder-1993/art-atlas.png | 1,101,338 |
| cinder-1993/home.png | 424,043 |
| cinder-1993/play-setup-auger.png | 307,888 |
| cinder-1993/style-sheet.png | 408,052 |
| ember-cathedral/art-atlas.png | 1,061,838 |
| ember-cathedral/home.png | 384,772 |
| ember-cathedral/play-setup-auger.png | 288,731 |
| ember-cathedral/style-sheet.png | 375,114 |

## Reproduction and layout

Nine deliverables regenerated with Pillow 12.3.0: all nine SHA-256 hashes identical before/after. Shared rendering functions and layout coordinates are used for every direction. Source atlases are never overwritten by the renderer. Text-width assertions and output-size assertions passed. WCAG body pairs exceed 4.5:1 and display pairs exceed 3:1; measured values are in README.

All nine mockups were visually inspected with image tools; spec 010 Home/setup production renders and all six v1 screens were inspected for layout/visual comparison. No clipping, missing glyph, overlapping add-on text or obscured focus remains.

## Reference tests

`cargo test -p ddl`: 27 tests passed (24 unit/headless, 2 catalog integration, 1 UI-string integration), including the headless fixture/production journey that regenerated `target/headless/010-*.png`. These validate the unchanged production reference, not this unimplemented theme. No Rust changes were made; workspace fmt/clippy/full core tests are not applicable to design-only artifacts.

## Requirement accounting

R1–R2: exact dimensions, common layout, strings/control badges and three style sheets checked by rendering assertions plus visual inspection. R3: generated prompts/no reference inputs and original imagery reviewed manually; no automated copyright test is claimed. R4: five actual source licence files checked and bundled. R5: computed opaque colour pairs checked against WCAG thresholds. A1 is delivered; A2 owner selection, A3 implementation and A4 Deck readability/performance remain outside phase A.

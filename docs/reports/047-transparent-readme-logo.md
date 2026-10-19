# Spec 047 — Transparent README logo

## Changes

- Added `readme-logo-dark.svg/.png` and `readme-logo-light.svg/.png` from the same
  `author.py` composition: unchanged D-029 emblem, lettering contours, placement,
  padding and 760×200 SVG / 1520×400 PNG dimensions; removed the background tablet.
  The dark-theme name remains bone `#F8EAD5`; the light-theme name uses the existing
  dark ink token `#120C0D`.
- Changed only README's first image line to the specified theme-aware `<picture>`,
  preserving the alt text and width 700. Updated usage notes and both contact-sheet
  examples, and replaced the old filename in the logo asset test with both variants.
- Extended the local-link checker to inspect `srcset` candidates, so a missing
  dark-theme image fails validation too.
- Removed the old `readme-logo.svg/.png`: no active consumer remains. References in
  `docs/reports/039-apply-logo-v3.md` are historical records of that task, not consumers.
- Added exporter assertions for emblem equality, name contours/fill/placement,
  fully transparent edges, presence of opaque name pixels, and theme contrast.
  No application or Rust behavior changed; only the asset enumeration changed.

## Verification

Merged `main` through `b369b7b` before final checks; README merged cleanly and retains
the coordinator's AI-agents note. All rendering and inspection
were headless; no windows were opened or screens captured.

### A1 — Reproducible exports, alpha and contrast

Passed with the existing pinned resvg 0.47.0, Pillow 12.3.0 and fonttools 4.60.1:

```sh
/tmp/ddl-logo-047-venv/bin/python docs/design/logo/author.py
/tmp/ddl-logo-047-venv/bin/python docs/design/logo/export.py --resvg /tmp/ddl-logo-tools/bin/resvg
/tmp/ddl-logo-047-venv/bin/python docs/design/logo/export.py --resvg /tmp/ddl-logo-tools/bin/resvg --check
```

Relevant `--check` output:

```text
readme-logo-dark.svg: corners alpha=[0, 0, 0, 0]; all edges alpha=0; name #F8EAD5 on #0d1117 contrast=15.97:1
docs/design/logo/readme-logo-dark.png  1520x400  81147 bytes
readme-logo-light.svg: corners alpha=[0, 0, 0, 0]; all edges alpha=0; name #120C0D on #ffffff contrast=19.37:1
docs/design/logo/readme-logo-light.png  1520x400  81108 bytes
```

Contrast uses linearized sRGB relative luminance of the flat name fill against each
specified background, excluding antialiased boundary pixels. The exporter also checks
all other committed exports, master copies and source geometry. A negative check with
one opaque corner in a scratch copy correctly failed the transparent-edge assertion.

### A2 — Composite previews

Saved and visually inspected all four combinations in this worktree (not committed):

- `target/readme-logo-previews/dark-on-dark.png`
- `target/readme-logo-previews/dark-on-light.png`
- `target/readme-logo-previews/light-on-light.png`
- `target/readme-logo-previews/light-on-dark.png`
- `target/readme-logo-previews/comparison.png` (four-up inspection sheet)

Each correct theme pairing has clear lettering and the same detailed emblem with no
background rectangle. Cross-theme previews demonstrate why theme selection is needed:
light-theme ink recedes on dark and bone recedes on white. The updated production contact
sheet was also inspected and has no clipping or overlapping labels.

### A3 — Links and asset contract

`python3 scripts/check-readme-links.py` passed:

```text
PASS: 33 local links/anchors in README and Steam Deck guide
```

A scratch README with an existing light image but a missing dark `srcset` correctly
failed with `missing-dark.png`. `git diff --check` passed.

The targeted `cargo test -p ddl --test logo_assets` passed both tests:

```text
test compiled_svg_masters_match_documented_sources ... ok
test steam_artwork_and_social_preview_have_contract_dimensions ... ok
test result: ok. 2 passed; 0 failed
```
No full-workspace build, clippy or test run is needed under spec 047 A3.

## Dependencies and remaining work

No new project or design-time dependencies; reused the existing pinned design tools.
No open questions, deferred spec requirements or proposed product decisions.

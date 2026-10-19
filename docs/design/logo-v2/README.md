# Logo v2 — Rose Compass selected

Three original directions in the Ember Cathedral palette: oxblood recesses, bronze
carving, bone highlights and one amber focal point. All use a newly hand-drawn upright
DDL wordmark with generous counters. The full name reuses the existing Barlow outlines
([SIL OFL notice](../../../crates/ddl/ui/assets/Barlow-OFL.txt)); original artwork is
GPL-3.0-or-later. No franchise marks, game imagery or franchise-derived lettering.
The owner selected **Rose Compass** on 2026-10-17 (D-023). Production masters,
compiled assets and Steam exports now live under [logo/](../logo/README.md);
these concept sheets remain as the phase-A comparison record.

| Concept | Idea and strength | Small-size risk |
| --- | --- | --- |
| [Cinder Key](cinder-key/contact-sheet.png) | A pointed escutcheon protects a bright keyhole: an invitation to unlock the map library. Strong vertical silhouette, closest to the carved cathedral mood. | Keyhole can read as a person; secondary bronze notches disappear at 32 px. |
| [Rose Compass](rose-compass/contact-sheet.png) | An octagonal rose window becomes a compass around a single ember: discovery in four directions. Most distinct silhouette from the current gate. | Petals merge at tiny sizes; can suggest a general navigation app. |
| [Ember Folio](ember-folio/contact-sheet.png) | An open stone map atlas with an amber bookmark: collecting and choosing maps. Broad, recognisable silhouette with no monogram in the emblem. | Page chevrons soften at 32 px; a book can suggest a reading app. |

Each folder contains `emblem.svg`, `horizontal.svg` (emblem + DDL + full name),
`contact-sheet.svg` and its direct PNG export. Sheets have **dark above, light below**;
labelled columns from left to right are **32, 64 and 256 px** emblem heights. In each row,
the emblem is above its lock-up. Lock-ups use the same emblem height, so widths are
96, 192 and 768 px. Bronze ticks below each column encode one tick per 32 px.
Inspect the PNG at native scale to judge the tiny specimens.

## Usage and reproduction

Emblem minimum: 32 px. Horizontal lock-up minimum: 192 px wide for DDL, 768 px
for reading the full name comfortably. The 96 px specimen intentionally exposes
failure of the full wordmark at tiny sizes; prefer the emblem there. Leave at least
one eighth of the emblem height clear. Keep proportions and colours. Each wordmark
includes its own dark plaque for contrast on light; put emblems on plain backgrounds.
No gradients, filters, raster content, strokes or live fonts: all marks and type are paths.

From the repository root (design-time tool, not a project dependency):

```sh
cargo install resvg --version 0.47.0 --locked --root /tmp/ddl-logo-tools
python3 docs/design/logo-v2/author.py
python3 docs/design/logo-v2/export.py --resvg /tmp/ddl-logo-tools/bin/resvg
python3 docs/design/logo-v2/export.py --resvg /tmp/ddl-logo-tools/bin/resvg --check
```

The author script stores the hand-authored contours and reuses the committed Barlow
name outlines without consulting system fonts. The exporter checks SVG element types,
absence of font/image references, PNG dimensions and byte equality in `--check` mode.
No image generation was used. Rose Compass application is tracked in spec 016 R2/A2. Production exports use
`../logo/export.py`; this exporter preserves the three historical comparison sheets.

## Production history

`production-d023/` keeps the D-023 production masters (emblem, horizontal, stacked,
full-name and the Steam compositions) as they shipped until spec 039 replaced them with
logo v3 · Stained Glass (D-029). Nothing in the app or release uses them; Git history has
the matching PNG exports.

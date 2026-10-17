# Rose Compass

Original DDL identity selected by the owner in D-023: an octagonal rose window
becomes a compass around a single amber ember. Four bone petals and a bronze rim
frame an oxblood recess. The upright, hand-drawn DDL lettering is the selected
concept's broad-counter wordmark. Nothing derives from franchise marks, game
textures or map screenshots. Original artwork uses GPL-3.0-or-later; Barlow
outlines retain the bundled SIL OFL notice.

Use `emblem.svg` alone at **32 px minimum**, `horizontal.svg` (short DDL lock-up)
at **144 px minimum width**, `full-name.svg` at **350 px**, and `stacked.svg` at
**240 px**. Leave at least one eighth of the emblem's height clear on every side.
Preserve proportions and colours. Use on near-black or a plain light surface;
the dark outer silhouette provides separation on light. Busy art needs a dark
plaque. The transparent Steam logo is a dedicated stacked composition.

## Reproduce

All deliverable PNGs are direct exports from committed SVGs; no raster editing.
The generated historical exploration sheet is explicitly **not** a deliverable master.

```sh
source ~/.cargo/env
cargo install resvg --version 0.47.0 --locked --root /tmp/ddl-logo-tools
python3 docs/design/logo/export.py --resvg /tmp/ddl-logo-tools/bin/resvg
python3 docs/design/logo/export.py --resvg /tmp/ddl-logo-tools/bin/resvg --check
```

`--check` renders to scratch and verifies byte equality, dimensions, transparency,
clean SVGs, and the compiled master copies. No network is used by exports or tests.
To edit authored geometry / re-outline the name, use Python 3 with
`fonttools==4.60.1` in a virtual environment and run `author.py` with that
environment's Python before exporting. It uses the bundled,
unmodified OFL Barlow font, with no system-font dependency. The original Ember Gate exploration and prompt remain in `concepts/` for the
historical spec 013 report. Its production masters have been replaced; Git history
preserves the old vectors. The phase-A Rose Compass concept is retained under
`../logo-v2/rose-compass/`; `author.py` now contains the identical emblem contours
and DDL wordmark and builds the production lock-ups and compositions here.
The export script uses only these production SVG masters.

The native 32 px dark/light specimens in `contact-sheet.png` retain separated
bone petals and a visible central ember. No small-size variant is needed. The
header uses the same emblem inside its 154×56 px short lock-up; headless renders
verify that use. Do not use the phase-A full-name concept lock-up at header size.

Existing Steam capsule/social compositions keep their dimensions and scenery.
The hero has no mark or text and remains byte-identical; release packaging still
installs `steam/*.png` into `ddl/art/`.

Eviternity's ivory cathedral and the generic oxblood threshold are original
woodcut-style mood art, not official covers or representations of actual maps.
Their cover and faded band SVGs use the same offline exporter. Existing SIGIL,
SIGIL II and AUGER artwork retains its spec 012 provenance.

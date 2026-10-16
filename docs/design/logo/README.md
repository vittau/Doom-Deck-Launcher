# The Ember Gate

Original DDL identity: an open cathedral threshold protects a single ember. The
symmetrical bronze gate and bone lintel remain readable at 32 px; the engraving
and oxblood recess become visible at larger sizes. The upright, hand-drawn DDL
slab lettering avoids small blackletter counters. Nothing derives from franchise
marks, game textures or map screenshots. Original artwork uses the repository's
GPL-3.0-or-later licence; Barlow outlines retain the bundled SIL OFL notice.

Use `emblem.svg` alone at **32 px minimum**, `horizontal.svg` (short DDL lock-up)
at **144 px minimum width**, `full-name.svg` at **350 px**, and `stacked.svg` at
**240 px**. Leave at least one eighth of the emblem's height clear on every side.
Preserve proportions and colours. Use on near-black or a plain light surface;
the dark outer silhouette provides separation on light. Busy art needs a dark
plaque. The transparent Steam logo is a dedicated stacked composition.

## Reproduce

All deliverable PNGs are direct exports from committed SVGs; no raster editing.
The generated exploration sheet is explicitly **not** a deliverable master.

```sh
source ~/.cargo/env
cargo install resvg --version 0.47.0 --locked --root /tmp/ddl-logo-tools
python3 docs/design/logo/export.py --resvg /tmp/ddl-logo-tools/bin/resvg
python3 docs/design/logo/export.py --resvg /tmp/ddl-logo-tools/bin/resvg --check
```

`--check` renders to scratch and verifies byte equality, dimensions, transparency,
clean SVGs, and the compiled master copies. No network is used by exports or tests.
To edit authored geometry / re-outline the name, use Python 3 with
`fonttools==4.60.1` and run `author.py` before exporting. It uses the bundled,
unmodified OFL Barlow font, with no system-font dependency. The three exploratory
ideas and exact generation prompt are retained in `concepts/`; only the pointed
gate was developed, with substantially simplified, newly drawn vector geometry.

Eviternity's ivory cathedral and the generic oxblood threshold are original
woodcut-style mood art, not official covers or representations of actual maps.
Their cover and faded band SVGs use the same offline exporter. Existing SIGIL,
SIGIL II and AUGER artwork retains its spec 012 provenance.

# Spec 054 — Steam artwork v2 (detailed vector scenes)

## Changes

- **New scene generator** [`docs/design/logo/scene.py`](../design/logo/scene.py): procedural,
  original Ember Cathedral interiors (D-021) for the three Steam compositions. Components:
  clustered piers (core and four engaged shafts with cylindrical gradients, foliate
  capitals, moulded bases), pointed arches with carved orders and bronze dogtooth, lancet
  windows with splayed mouldings, mullions, lead quarries and a quatrefoil oculus, a
  sixteen-petal rose window, carved spandrel roundels, stone courses with staggered and
  weathered blocks and lit arrises, perspective flagstones, bronze tripod braziers with
  layered flames and sparks, light shafts, floating embers, vault gloom and a vignette.
  Palette: the v3 constants (oxblood, bronze ramp, ember, amber, bone) plus an oxblood
  stone ramp. Soft light and shade are stacked translucent shapes (`fill-opacity`), split
  finely enough to stay below one 8-bit step, so no filter, mask or `stop-opacity` is
  needed. Seeds are fixed and coordinates use v3's two-decimal `num()`, so rebuilds are
  byte-identical. Nothing derives from franchise marks, game textures or screenshots.
- **`author.py`**: removed the woodcut `scene()`; the Steam compositions now take their
  scenery from `scene.py` (loaded like `../logo-v3/author.py`, by path) and set the
  emblem, wordmark and name directly on it, with **no plate**. The capsule lettering
  geometry and scale are unchanged; the wide capsule's emblem moved 16 px right for more
  left margin (owner/coordinator polish request), with the wordmark and name moved by the
  same amount and the scene's apse shifted 14 px right to keep the name clear of the pier.
- **Contact sheet**: a third band shows the library capsule (1/3), the wide capsule (1/2)
  and the hero (7/24), each in a nested `svg` viewport that clips its scene. The sheet is
  now 1200×2020.
- **`crates/ddl/tests/logo_assets.rs`**: one test-data tuple, `contact-sheet.png` 1200×1200 →
  1200×2020, approved by the coordinator (Rust is otherwise outside this task's ownership).
- **`export.py`** (new checks, all in `--check`):
  - Steam compositions: canvas size; capsule emblem, wordmark and name groups equal to
    `stacked.svg`'s contours; the name's placement is the one the contrast box assumes;
    the hero carries no emblem gradient (`gl-*`), i.e. no mark.
  - **No plate**: no opaque, flat-coloured, axis-aligned rectangle in a scene covers more
    than 5% of the frame (walls and floor are gradients).
  - **Name contrast**: the composition minus its lettering is rendered with resvg and every
    pixel under the name box is measured; bone `#F8EAD5` must keep ≥ 4.5:1 (D-020).
  - **SteamGridDB limits**: library and wide capsule ≤ 15 MB, hero ≤ 24 MB, logo ≤ 15 MB.
  - `logo.png` keeps a fully transparent corner and transparent pixels.
  - `ET.register_namespace` for the default SVG namespace, so the bare render is plain SVG.
- **README** (`docs/design/logo/README.md`): replaced the "woodcut hero unchanged" note
  with a Steam artwork section (scene, no-plate rule, hero safe area, logo position, size
  limits, checks), updated the SVG rules (`fill-opacity`, nested viewports in the sheet),
  the contact-sheet row and the reproduce command (`python -B` keeps `__pycache__` out of
  the tree).
- **Exports regenerated**: `steam/library-capsule.png`, `steam/wide-capsule.png`,
  `steam/hero.png`, `contact-sheet.png` and their SVGs. Every other export (icons,
  `logo.png`, social, README logos, compiled app masters) is byte-identical.

## Verification

All renders were headless (resvg to PNG, Pillow); no window was opened and no screen was
captured. Tools: resvg 0.47.0, Pillow 12.3.0, fonttools 4.60.1 (the README's pins),
installed under the worktree's `target/`.

### A1 — Design round, owner approval

Round 1 renders (1× and 50%) of both capsules and the hero, plus a 16:9 centre crop of the
hero (2204×1240 → 1920×1080) with the current `logo.png` overlaid at Steam's bottom-left
position and centred, were sent to the coordinator. **Owner approved round 1 as is** on
2026-10-20, with: keep `logo.png`, recommend bottom-left, optional wider left margin for
the wide-capsule emblem (applied, see above). The hero exported is byte-identical to the
approved round-1 render (1,017,287 bytes); the library capsule differs only by one ember
removed from the name box.

### A2 — One source, byte-identical, `--check` passes from a clean tree

`git archive HEAD` extracted twice into scratch; `author.py` run in one, then
`diff -rq` against the other: **no difference**. `export.py --check` then passed in the
same clean tree, and the tree was still identical afterwards. Relevant output:

```text
library-capsule.svg: lettering unchanged, no flat plate, name contrast >= 14.04:1
wide-capsule.svg: lettering unchanged, no flat plate, name contrast >= 8.53:1
hero.svg: no mark or text; no flat plate
docs/design/logo/steam/library-capsule.png	600x900	257749 bytes
docs/design/logo/steam/wide-capsule.png	920x430	202745 bytes
docs/design/logo/steam/hero.png	3840x1240	1017287 bytes
docs/design/logo/steam/logo.png	1280x720	90932 bytes
docs/design/logo/contact-sheet.png	1200x2020	722599 bytes
```

All SVGs pass the `../logo-v3/export.py` allowlist (`svg`, `defs`, `g`, `path`,
`linearGradient`, `radialGradient`, `stop`; `fill-opacity`; no raster, filters, strokes,
masks, fonts or external references).

### A3 — Sizes, transparency, file-size limits

| File | Size | Bytes | Limit | Source of limit |
| --- | --- | ---: | ---: | --- |
| `library-capsule.png` | 600×900, RGBA, opaque | 257,749 | 15 MB (grid) | SteamGridDB upload dialog |
| `wide-capsule.png` | 920×430, RGBA, opaque | 202,745 | 15 MB (grid) | SteamGridDB upload dialog |
| `hero.png` | 3840×1240, RGBA, opaque | 1,017,287 | 24 MB (hero) | SteamGridDB upload dialog |
| `logo.png` (unchanged) | 1280×720, RGBA, transparent corner | 90,932 | 15 MB (logo) | SteamGridDB upload dialog |

- **Confirmed** — SteamGridDB static upload limits: grids 15 MB, heroes 24 MB, logos 15 MB;
  accepted grid sizes include 600×900 and 920×430, hero sizes include 3840×1240. Source:
  the upload dialog's instructions in the
  [site bundle](https://www.steamgriddb.com/static/assets/233e475efea66025d29a.js), as
  recorded in [`docs/research/steam-shortcuts.md` §5.4](../research/steam-shortcuts.md#54-size-format-and-style-rules)
  (read 2026-10-19). The API docs' older "1MB" grid limit is stale (same section).
- **Confirmed** — Steam's hero guidance: 3840×1240, no text, "At the center of the
  template is a 'safe area' of 860px x 380px … critical content should be within the
  safe area"; the logo is "placed on top of the hero graphic" at a position the publisher
  chooses: "left bottom corner, centered top, centered middle, and centered bottom"
  ([Steamworks: Library Assets](https://partner.steamgames.com/doc/store/assets/libraryassets),
  read 2026-10-20).
- **Proposed** — the hero's focal apse (rose, lancet triplet, braziers) spans the centre;
  its brightest part lies inside the centred 860×380 box at full size and, a fortiori,
  inside a 16:9 or any narrower centred crop. Place `logo.png` **bottom left**: there it
  sits over the shaded side bay and floor; centred middle it covers the bright apse and
  reads worse (both checked in the round-1 crops). `logo.png` is unchanged.
- **Needs Deck Validation** — how Steam on the Deck crops and darkens the hero at its own
  aspect ratios, and the final logo size/position, can only be seen after upload.

### Rust asset test (targeted only)

`cargo test -p ddl --test logo_assets`: 2 passed (dimensions, RGBA, opacity of the
capsules/hero/sheet, transparency of `logo.png`, compiled masters unchanged). The full
workspace checks were not run, as agreed: the coordinator runs them at merge.

## New dependencies

None. resvg, Pillow and fonttools are the existing design-time pins from the logo README.

## Open questions

- None. (The contact-sheet size asserted by `logo_assets.rs` was resolved with the
  coordinator: the tuple now says 1200×2020.)

## Spec items left open

- None in R1–R5. Deck-side appearance after upload is **Needs Deck Validation** (A3).

## Proposed coordinator-owned updates

- `docs/status.md` / `docs/journal.yml`: spec 054 done; Steam capsules and hero are now
  vector Ember Cathedral scenes with no plate behind the lettering; recommend the
  bottom-left logo position when uploading to SteamGridDB/Steam.

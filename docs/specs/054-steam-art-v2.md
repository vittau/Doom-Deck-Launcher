# 054 — Steam artwork v2: detailed vector scenes

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 013, 039, D-020, D-021, D-029
**Origin:** owner, 2026-10-19, on the Steam capsules and hero before uploading them to
SteamGridDB: "estão muito ruins os assets. Os capsules precisam não ter o fundo preto no texto,
fica mt feio. E em todos, o fundo está feio. Faça com SVGs mais detalhados, como foi feito com o
logo."

Today `library-capsule.svg`, `wide-capsule.svg` and `hero.svg` (in `docs/design/logo/`) put the
logo on a flat black rectangle over a coarse "woodcut" scenery of flat-colour spires with repeated
window slits. `hero.png` still carries that scenery unchanged from before D-029.

## Requirements

- **R1** — **No plate behind the lettering:** the capsules set the emblem, the DDL wordmark and
  the name `DoomDeckLauncher` directly on the scene. Neither capsule has a black or flat rectangle,
  tablet or box behind them. The scene's own composition keeps the name readable: a darker
  recess, falling light or a vignette where the lettering sits, and the wordmark's existing
  shadow. Emblem and wordmark geometry come unchanged from `../logo-v3/author.py`; this task
  draws only the scenery.
- **R2** — **Detailed vector scenery:** replace the woodcut scenery in all three compositions with
  an original, detailed Ember Cathedral interior (D-021) at least as finely worked as the emblem.
  Possible elements: clustered gothic columns, ribbed vaults, pointed arches with tracery, a
  rose window or lancet glass glowing behind, braziers or candle light, stone courses, and depth
  from layered gradients and atmospheric falloff. Use the brand palette (oxblood, bronze, ember
  amber, bone) and avoid flat colour fields. The art is original: nothing derives from franchise
  marks, game textures or map screenshots.
- **R3** — **One source of truth:** a script generates the scenes (procedural paths and
  gradients), beside `author.py` or called by it, so `author.py` rebuilds the three
  compositions byte-identically. The SVG allowlist of `../logo-v3/export.py` still applies:
  paths, `linearGradient`/`radialGradient`/`stop`, `fill-opacity`, with no raster, filters,
  strokes, masks or fonts. If the look truly needs an element outside the allowlist, ask the
  coordinator first.
- **R4** — **Steam fit:** keep the sizes of spec 013 R4: library capsule 600×900, wide capsule
  920×430, hero 3840×1240 and transparent logo 1280×720.
  - The hero has no mark and no text. Steam draws `logo.png` over it and crops the hero at
    different aspect ratios, so the focal detail must survive a centred crop. Cite Steam's or
    SteamGridDB's published guidance for the hero safe area and the logo position.
  - Check `logo.png` over the new hero. Keep `logo.png` unless it reads poorly there; if so,
    propose a change.
  - Each PNG stays within SteamGridDB's upload limits. Cite the limits.
- **R5** — **Exports and docs:** `export.py` regenerates the PNGs and `--check` passes.
  - Update `docs/design/logo/README.md`, which today says the woodcut hero is unchanged, and the
    contact sheet.
  - Release packaging still installs `steam/*.png` into `ddl/art/`.

## Acceptance criteria

- [ ] **A1** (R1, R2) — design round first. Render both capsules and the hero as PNGs at 1× and at
  50%. Also render one hero centre crop at 16:9 with `logo.png` overlaid. The coordinator shows
  these to the owner, and the owner approves them before anything is wired into exports.
- [ ] **A2** (R3, R5) — `author.py` then `export.py --check` pass from a clean tree, with the same
  bytes.
- [ ] **A3** (R4) — sizes, transparency of `logo.png` and file sizes are checked, and the limits
  are cited in the report.

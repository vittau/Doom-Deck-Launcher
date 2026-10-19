# 036 — Logo v3: a richer Rose Compass

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 013, 016 (D-023)
**Origin:** owner, 2026-10-19: iterate on the project logo, adding more detail and making it more
interesting; it must stay SVG with no raster content, but it may have more detail and more
colours. The coordinator reviews the result critically. Run on Claude Opus 5.5, effort high
(owner's explicit routing for this task).

## Context

The current mark is **Rose Compass** (D-023, `docs/design/logo/`, concept record in
`docs/design/logo-v2/`): an octagonal rose window that becomes a compass around a single ember,
with the hand-drawn upright DDL wordmark, Ember Cathedral palette (oxblood, bronze, bone, one
amber focal point). v2 rules were flat fills only — "no gradients, filters, raster content,
strokes or live fonts".

## Requirements

- **R1** — **Evolve, don't replace:** v3 stays recognisably the Rose Compass (octagonal rose
  window, compass reading, single ember focus, DDL wordmark), and adds craft: tracery, carved
  bevels, layered glass, metal and ember tones, finer ornament — the DOOM-flavoured "carved
  cathedral" mood of D-020 without franchise marks, game imagery or franchise lettering.
- **R2** — **SVG rules for v3:** paths and shapes only; no `<image>`, no embedded raster, no
  external or live fonts (text as outlines), no scripts. Linear and radial gradients are allowed;
  avoid filters unless the Slint software renderer draws them identically (prove it with a
  headless render) — prefer layered shapes for glow. Keep files small (state the byte size).
- **R3** — **Scale tiers:** detail must not turn into noise. Deliver the full emblem for ≥ 128 px
  and, if needed, a simplified small-size emblem for 32–64 px that keeps the same silhouette; the
  app icon and window icon must read at 32 and 48 px.
- **R4** — **Directions:** produce 2–3 distinct v3 variants (e.g. different amounts of ornament or
  colour), each with `emblem.svg`, `emblem-small.svg` (if used), `horizontal.svg`, and a contact
  sheet exported by a committed, reproducible script (pinned design-time `resvg`, like
  `docs/design/logo-v2/export.py`): emblem at 32, 64, 128, 256, 512 px on dark and light, the
  lock-up, and the emblem shown in a 1280×800 mock of the app header. Each variant gets an honest
  strengths / risks note.
- **R5** — **Self-critique:** before finishing, review your own variants against R1–R3 at native
  scale and iterate at least once; record what you changed and why.
- **R6** — **No production swap:** do not touch `docs/design/logo/`, `crates/ddl/ui/assets/logo/`
  or Steam art. The owner picks a variant after the coordinator's review; applying it is a
  follow-up task.

## Acceptance criteria

- [ ] **A1** (R2, R4) — SVGs in `docs/design/logo-v3/<variant>/`, export script with a `--check`
  mode that verifies element types, absence of raster/font references and reproducible PNGs.
- [ ] **A2** (R3, R4) — contact sheets inspected at native scale; the 32 px specimen is legible.
- [ ] **A3** (R5) — iteration notes in the report.

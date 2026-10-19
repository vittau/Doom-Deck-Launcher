# 045 — UI touch-ups: wall seam and the "Verification pending" row

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 012, 030, 035
**Origin:** owner, 2026-10-19, on the `v0.0.6`/main UI: "O fundo tem essa quebrinha, tem como
resolver?" (a dark horizontal line across the background) and "tire o item 'Verification pending
- needs deck validation'. Me parece informação inútil em geral." Also: "Se for cortar, use o PNG
original para evitar compor artefatos JPEG."

## Requirements

- **R1** — **Seamless wall:** `crates/ddl/ui/assets/ember/wall.jpg` has a dark line at y≈216
  (and its repeat) because `build_assets.py` crops the wall quarter as `a.crop((0, 0, 512, 438))`
  from `docs/design/visual-v2/ember-cathedral/art-atlas.png`, and that crop includes the atlas'
  dark cell border (rows 434–437, mean luma ≈ 2 vs ≈ 37; columns 510–511 likewise). Mirroring
  doubles the border inside the tile. Crop inside the border (e.g. `(0, 0, 510, 433)`), keep the
  mirrored-quarter construction, and regenerate **only from the original PNG atlas** — never by
  editing or re-encoding the existing JPEG. Other assets stay byte-identical.
- **R2** — **No "Verification pending" in the player UI:** remove the Home readiness row
  "Verification pending · Needs Deck Validation" (`WiringStrings.verification`,
  `Text::Verification`, `runtime.rs`) and let the remaining checks lay out without a gap. Audit
  every other player-visible string for project-process labels ("Verification pending",
  "needs Deck validation", "Needs Deck Validation") and remove them from the UI; they belong in
  docs. Mapset verification evidence that the catalog really records (spec 002 `[verification]`
  status) may stay only where it tells the player something actionable — list each kept or
  removed string in the report.
- **R3** — **Docs:** refresh the README screenshots (`readme-screenshots.py`) so they show the
  seamless wall and the new Home.

## Acceptance criteria

- [ ] **A1** (R1) — a row/column luma check (test or script output in the report) shows no dark
  line in `wall.jpg`; a headless render of Home shows the seamless background; the build script
  is the only source of the change.
- [ ] **A2** (R2) — headless Home render without the row; tests updated; the audit table in the
  report.
- [ ] **A3** — fmt, clippy `-D warnings`, tests green; `readme-screenshots.py --check` passes.

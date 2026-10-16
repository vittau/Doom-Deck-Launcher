# 011 — README: manual Steam Input setup (Classic, Modern + Gyro)

**Status:** Done (A3 needs Deck)  ·  **Milestone:** M3 (drafted early)  ·  **Depends on:** —
**Policy:** OVERVIEW §8.2 and §11: Steam Input is documented, never automated. No code.

## Goal

A player configures controls for DDL and DSDA-Doom entirely from Gaming Mode using only the
README: two suggested layouts, why each setting matters, and how Steam-side and DSDA-side
sensitivity interact — honest about what still needs testing on hardware.

## Requirements

- **R1** — `README.md` section **Recommended Steam Deck Controller Setup**: controller-only steps
  in Gaming Mode (open the DDL non-Steam shortcut → controller icon → Edit Layout → assign →
  gyro/trackpad → save/apply). Mark Steam UI names as "as of <date>, may change".
- **R2** — Two **suggested** layouts as tables (OVERVIEW §8.2): **Classic** (no gyro) and
  **Modern + Gyro** (gyro to mouse, ideally enabled while touching the right stick), mapping to
  DSDA-Doom's defaults from DDL's baseline (`use_game_controller 1`, `use_mouse 1`) and DSDA's
  default bindings listed in `docs/research/dsda-engine.md` §4 (e.g. right trigger fires by
  default). Never invent a binding DSDA does not have.
- **R3** — **Double input:** explain virtual controller vs mouse emulation, why both are enabled
  in DDL's DSDA baseline, and how to avoid binding the same action twice (Steam layout vs DSDA
  bindings). Explain that DDL's own UI ignores pointer motion (D-009).
- **R4** — **Sensitivity:** where to tune gyro/trackpad (Steam) versus stick look speed (DSDA
  menu, which DDL preserves across launches — spec 005).
- **R5** — **Strict demo recording caveat** (research §5): mixing gamepad and mouse is blocked
  while strict recording; suggest a keyboard + mouse emulation layout for demo runners.
- **R6** — A short **"Not yet verified on hardware"** note listing what the owner still has to
  confirm (child-process layout handoff, gyro activation via stick touch), linking
  `docs/deck-testing.md` I1–I4.
- **R7** — Keep the README friendly and scannable; this section links from the install section.

## Acceptance criteria

- [x] **A1** — every binding named in the tables exists in `research/dsda-engine.md` §4 (list the
  key names in the report).
- [x] **A2** — no instruction asks the user to edit `.vdf` files, import community layouts or use
  a wizard.
- [ ] **A3** — *Needs Deck Validation:* the owner follows the steps in Gaming Mode.

# 056 — DSDA template v4 (VSync, look sensitivity) and the Deck gyro guide

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 005, 029, 034, 011
**Origin:** owner's Deck test of `v0.0.7-deck` (2026-10-20):
- "O jogo está com taxa de FPS máxima ilimitada, o que consome excessivamente no Deck. Mude a
  config [para] ligar o VSYNC, mantendo os FPS ilimitados."
- "Nas configs do controle, coloque tanto a sensibilidade horizontal quanto vertical em 5 por
  padrão."
- Gyro recommendation: Steam Input **Gyro to Mouse**, **Gyro Angles to Mouse Pixels** 25000 px,
  **Gyro Speed Deadzone** 0, **3DOF to 2D Conversion Style** Yaw.

## Requirements

- **R1 — VSync on, uncapped interpolation kept:** template v4 turns `render_vsync` on and keeps
  uncapped (interpolated) frame rate on, with no FPS limit key forcing a cap. Confirm every key,
  type and range in DSDA-Doom **v0.30.1** source and cite it (pinned links) in
  `docs/research/dsda-engine.md`; record how DSDA combines `render_vsync`, `uncapped_framerate`
  and `dsda_fps_limit` (does VSync cap the uncapped rate at the refresh rate in the SDL/OpenGL
  path the Deck uses?). If the source shows VSync has no effect in some renderer/video mode DDL
  uses, say so and stop for a decision.
- **R2 — Sensitivity 5/5:** the owner means DSDA's in-game **horizontal and vertical
  sensitivity** settings. Coordinator reading: these are the Mouse options
  (`mouse_sensitivity_horiz`, default 10; `mouse_sensitivity_vert`, default 1), which drive the
  Deck's gyro-as-mouse; the analog-stick keys (`right_analog_sensitivity_x/y`, defaults
  1536/768) use a different scale. Verify against the v0.30.1 menu source which settings the menu
  labels "Horizontal/Vertical Sensitivity" and set those two to `5`. If the labels belong to the
  analog keys instead, stop and ask (do not guess a scale).
- **R3 — Migration:** template v3 → v4 uses the existing three-way migration: a key the player
  changed from the v3 value keeps the player's value; untouched keys take the v4 value; the
  migration is recorded like the 029/034 ones. Bump the template version.
- **R4 — Gyro guide (docs only, no Steam Input code):** update `docs/steam-deck-guide.md`'s
  controller section (and README only if it repeats it) with the owner's gyro settings above,
  explained in one line each, plus that DSDA's mouse sensitivity is preset to 5/5 by DDL.
  Use Steam's own setting names; check them against current Steam documentation or screenshots
  and label anything unverified **Needs Deck Validation**.

## Acceptance criteria

- [ ] **A1** (R1, R2) research section with pinned DSDA links for each key; template v4 values
  under test.
- [ ] **A2** (R3) migration tests: untouched v3 → v4 values; player-edited sensitivity/vsync kept.
- [ ] **A3** (R4) guide updated; no Steam Input code anywhere (grep in the report).
- [ ] **A4** fmt, clippy `-D warnings`, `cargo test --workspace` green.
- [ ] **Needs Deck Validation:** DSDA runs with VSync (no tearing, lower power draw than 0.0.7
  at the same scene), still smooth; gyro aim with the guide's settings feels right.

# 034 — The owner's DSDA-Doom controller layout as the Deck default

**Status:** Ready (after 029)  ·  **Milestone:** M3  ·  **Depends on:** 005, 029
**Origin:** Deck test 2026-10-19: the controller works in DSDA-Doom (E5 ✅) but is not the owner's
preferred layout. Owner's layout, 2026-10-19:

| Action | Deck button (Steam's default gamepad layout) |
| --- | --- |
| Automap | Select (View) |
| Automap zoom in | D-pad Up |
| Automap zoom out | D-pad Down |
| Use / open | A |
| Toggle run (autorun) | B |
| Next weapon | D-pad Right |
| Previous weapon | D-pad Left |
| Quick save | L1 |
| Quick load | R1 |
| Quick turn (180°), if DSDA has it | X |

## Requirements

- **R1** — **Research:** from DSDA-Doom **v0.30.1** source (pin the tag; cite file and line), find
  how game-controller buttons are bound: the config keys, the button codes (SDL GameController
  names or indices), whether one button may hold two actions (D-pad Up/Down are both automap
  zoom *and* whatever DSDA binds them to in play), and whether automap zoom keys work while the
  map is open with the same buttons used for movement or weapons otherwise. Find whether DSDA has
  a 180° turn action and its key; if it has none, say so and leave X at DSDA's default. Record
  everything in `docs/research/dsda-engine.md` labelled Confirmed / Proposed. Never invent keys.
- **R2** — **Template:** the Deck template binds the table above, keeps DSDA's defaults for every
  action the table does not name (fire, strafe, movement, look, menu), and bumps
  `template_version` so spec 005's three-way migration keeps bindings the player changed in
  DSDA's menu.
- **R3** — **Conflicts:** if a requested binding collides with a default that matters in play
  (for example D-pad Up/Down outside the automap), the report says what DSDA does and which
  choice was made; ask the coordinator before dropping any requested binding.
- **R4** — **Guide:** `docs/steam-deck-guide.md` shows the in-game controller layout as a table.

## Acceptance criteria

- [ ] **A1** (R1) — research section with links to source lines.
- [ ] **A2** (R2) — template tests: every binding is a known key with a valid code; a migration
  from the previous template keeps a user-changed binding while adding the new ones.
- [ ] **A3** — fmt, clippy `-D warnings`, tests green. Deck re-check: E5 with the new layout
  (Needs Deck Validation).

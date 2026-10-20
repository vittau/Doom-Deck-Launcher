# 060 — Focus clarity: leftmost landing in rows, and a Play button that does not look focused

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 012, 030, 057
**Origin:** owner, 2026-10-20, on the 057 build:
- "DPAD pra baixo no Advanced está indo pra opção Pistol start e não pra Skill como pedi, fica
  muito confuso. Tem que ir pro botão à esquerda. O pistol start é acessível estando em Skill, e
  apertando DPAD RIGHT."
- "O botão Play amarelo sempre é muito confuso, parece que está selecionado."

## Requirements

- **R1 — Rows are entered at their leftmost item.** When D-pad Up/Down moves focus from an item
  into a row of several side-by-side items (Skill | Pistol start, add-on chips, dialog button
  rows, any other), focus lands on the row's **leftmost** item whenever the source spans more than
  one of the row's items (a full-width card such as Advanced, or a wider row). Only when the source
  sits in a single column that lines up with exactly one target (e.g. moving between two
  two-button rows with matching columns) does focus keep that column. Left/Right move within the
  row as today. Replace 057's "nearest by overlap" choice where it produces the reported jump.
  Apply the rule on every screen; the 057 focus-walk tests must assert the landing item, not
  only the visiting order.
- **R2 — One focus language; amber means focus only.** Today the primary action (Play, and any
  other "primary" button such as first-run Install) is filled amber — the same colour as the
  focus highlight — so it reads as selected when it is not, and some focused items are shown by an
  amber fill while others only get a ring. Define and apply one rule across the UI:
  - focus is shown the same way on every focusable element: the amber focus ring (keep the
    existing ring), plus at most a subtle brightening of that element's own fill;
  - no unfocused element uses the amber fill;
  - primary actions get their own unfocused style that cannot be mistaken for focus — an ember/
    blood-red fill with light text from the Ember Cathedral palette (D-020), legible on the Deck;
    when focused they show the same amber ring as everything else;
  - selected-state items (a chosen add-on chip, a toggle that is on) must also not use the focus
    fill; give "selected" its own mark (e.g. a check or a coloured edge) that differs from focus.
  Record the rule in `docs/design/ui.md` (states table: default, focused, primary, selected,
  disabled), and keep the first-run pulsing emphasis of spec 030 on the new primary style.

## Constraints

Never open visible windows or capture the screen — headless Slint renders only. Keep the Ember
Cathedral identity; contrast ≥ 4.5:1 for button text. Spec 059 runs in parallel (sandbox code,
an import dialog, a Diagnostics line); merge `main` before your final checks.

## Acceptance criteria

- [ ] **A1** (R1) tests: Down from Advanced lands on Skill; Up from Recipe load order lands on
  Skill; Right from Skill reaches Pistol start; the same leftmost rule on every audited row
  (list in the report).
- [ ] **A2** (R2) headless before/after renders of Play setup (Play unfocused and focused, Skill
  focused), Home first run (Install), a dialog, and the add-on chips with a selected chip; a
  contrast table in the report; the states table in `docs/design/ui.md`.
- [ ] **A3** README renders refreshed (`readme-screenshots.py --check`); fmt, clippy
  `-D warnings`, `cargo test --workspace` green.
- [ ] **Needs Deck Validation:** Play no longer looks selected; focus is obvious on every screen.

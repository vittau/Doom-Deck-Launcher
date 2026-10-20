# 057 — Controller navigation touch-ups: right-stick scroll, Play setup order, skills, tabs, B to quit

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 008, 030, 018, 031
**Origin:** owner's Deck test of `v0.0.7-deck` (2026-10-20), quoted per requirement.

## Requirements

- **R1 — Right stick scrolls.** "Algumas das telas do launcher têm scroll … e não há como scrollar
  no Deck exceto usando a touchscreen. Seria bom que o stick direito fizesse scroll nessas
  telas." On every scrollable view (screens, panels, dialogs, long error text, Settings pages,
  mapset detail, Discover/Library lists that scroll) the right stick scrolls the view under focus
  vertically (and horizontally where a view scrolls horizontally), with a deadzone and speed that
  scales with deflection; it never moves focus. Read the axis through the existing gilrs input
  path (spec 008). Inventory every scrollable view in the report.
- **R2 — Play setup D-pad order.** "Quando clico DPAD DOWN no item 'Advanced', ele pula direto pra
  'Recipe load order', pulando a linha com os itens 'Skill' e 'Pistol start'." Down from
  Advanced goes to the Skill/Pistol start row, then to Recipe load order; Up mirrors it. Audit the
  vertical focus order of every screen with multi-item rows (Play setup, Mapset detail, base-game
  page, Settings, custom setup) and fix any other skip; a test walks each screen top→bottom and
  bottom→top and asserts the order.
- **R3 — Skill names and default.** "Na opção de skill, na 4 coloca (UV) do lado (de
  Ultra-Violence), e transforme essa skill na padrão. Na maior, coloque (Nightmare)." Labels:
  skill 4 shows "(UV)" next to its name, skill 5 shows "(Nightmare)" (follow the existing label
  style; fit test still passes). `DEFAULT_SKILL` becomes 4 for **new** profiles and new custom
  setups; existing saved profiles keep their stored skill (decision D-032). Update fixtures,
  resolver/plan expectations and docs that print the default.
- **R4 — Tab order.** "Coloque a aba Library à direita de Home (segunda posição), empurrando
  Discover para a terceira posição." Tabs: Home, Library, Discover, Settings (L1/R1 and every
  index-based mapping, first-run emphasis of spec 030, hint footer, README screenshots).
- **R5 — B at the root opens Quit.** "Apertar B na raiz da navegação deve abrir a modal de sair. B
  de novo fecha a modal. A opção padrão (esquerda) da modal, porém, deve ser o Quit." At a root
  screen (a tab's top level with no dialog or sub-screen open), B opens the existing quit
  dialog; inside it, B closes it (cancel). The dialog's left button is **Quit** and has initial
  focus; A on it quits. B anywhere else keeps its current "back" meaning.

## Constraints

Never open visible windows or capture the screen — headless Slint renders only (AGENTS.md).
Spec 055 runs in parallel and edits the state-error path, `services.rs` and Settings →
Diagnostics; coordinate by merging `main` before your final checks.

## Acceptance criteria

- [ ] **A1** (R1) inventory table; tests inject right-stick axis events and assert the scroll
  offset changes and focus does not.
- [ ] **A2** (R2) focus-walk tests for each audited screen.
- [ ] **A3** (R3) headless render of the skill picker; tests for the new default and that an
  existing profile with skill 3 still loads as 3.
- [ ] **A4** (R4, R5) headless renders: tab bar in the new order; quit dialog with Quit focused;
  key tests for B at root → dialog, B → closed, A on default → quit requested.
- [ ] **A5** README renders refreshed (`readme-screenshots.py --check` passes); fmt, clippy
  `-D warnings`, `cargo test --workspace` green.
- [ ] **Needs Deck Validation:** right-stick scroll feel, Play setup order, B/quit flow.

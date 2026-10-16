# 008 — UI shell: design system, navigation, gamepad input, screens on fixture data

**Status:** Done (A6 needs Deck)  ·  **Milestone:** M1  ·  **Depends on:** 001; design in `docs/design/ui.md`,
`docs/design/visual/*.png`; decisions D-008, D-009, D-018

## Goal

Replace the placeholder menu with the real DDL interface from the design contract, fully
operable with the Steam Deck controller, rendering the main screens from **fixture data** (the
three reference mapsets) so the look, navigation and focus behaviour can be built, tested
headlessly and tried on the Deck before the core modules are wired in (later spec).

## Requirements

- **R1** — **Design tokens and assets:** Slint globals for the colour, typography, spacing,
  radius and elevation tokens in `ui.md` §2; Inter (OFL, from `docs/design/assets/`) bundled
  into the binary with its license file shipped in the release tarball; the DDL logotype as a
  vector (SVG) or crisp PNG asset. Concept art from `docs/design/assets/` may be used as
  placeholder cover art, labelled as in the mockups.
- **R2** — **Shell:** top tab bar (Home, Discover, Library, Settings), content area, and the
  persistent bottom button-hint bar whose legends change per screen/focus (`ui.md` §3). L1/R1
  switch tabs; B = Back (never Menu); Start = Menu (quit confirmation for now).
- **R3** — **Focus system:** spatial D-pad/stick navigation between focusable elements that
  matches the on-screen layout; the focus ring from `ui.md` is always visible on the focused
  element; focus is restored per screen when navigating back; no hover behaviour; pointer motion
  without a click is ignored and the cursor is hidden (D-009).
- **R4** — **Gamepad input via `gilrs`** (D-018): a small input module polls gilrs on a timer and
  translates D-pad, left stick (with deadzone and repeat delay/rate), A, B, X, Y, L1, R1, Start
  into the app's navigation actions; keyboard equivalents (arrows, Enter, Escape, Tab/Shift+Tab,
  F1–F4) stay available for development. Hot-plug is handled; no gamepad is not an error.
- **R5** — **Screens on fixture data** (`crates/ddl/src/fixtures.rs` or similar, clearly marked
  as fixtures): Home/Continue, Discover (cards for SIGIL, SIGIL II, AUGER;ZENITH), Mapset Detail,
  **Play setup** with the "Will launch" panel (base-game status, numbered load order, component
  rows — required components shown as **locked** rows, optional ones as toggles, exclusive
  groups as a choice, user-supplied ones as "not provided — import", complevel, skill, pistol
  start), the **Play blocked** state (missing IWAD with remediation text), and a Settings screen
  with static rows. Layout and copy follow `ui.md` §4–§6 and the visual mockups. Play/Install
  actions show a "not wired yet" toast in this spec.
- **R6** — **Strings** all go through Slint `@tr` (D-008).
- **R7** — **Window:** fullscreen by default, `--windowed` keeps 1280×800 for desktop testing;
  readable at 1280×800.
- **R8** — **Headless verification:** extend the existing headless test (`crates/ddl/src/headless.rs`)
  to render every screen in R5 at 1280×800 with the software renderer to PNGs under
  `target/headless/`, and to drive navigation with synthetic key events (tab switch, focus
  movement, back, focus restore, locked-row skip/behaviour, toggle). **Never open a window or
  capture the screen** (AGENTS.md).

## Out of scope

Real data and actions (catalog, downloads, engine, IWAD library, launching) — the next spec wires
them in. On-screen keyboard. Localization files beyond the `@tr` markup.

## Acceptance criteria

- [x] **A1** (R1, R2, R5, R7) — headless PNGs for every screen; compare them side by side with
  `docs/design/visual/*.png` and list deviations in the report.
- [x] **A2** (R3, R8) — navigation tests: tab cycle, spatial focus on a grid and a list, back
  restores focus, ring always present on the focused element.
- [x] **A3** (R4) — unit tests of the gilrs-to-action mapper (deadzone, repeat timing) using
  synthetic events; no physical device needed.
- [x] **A4** (R6) — no user-visible string literal outside `@tr` (a test or lint script).
- [x] **A5** — `cargo fmt`, `clippy -D warnings`, `cargo test` green; release build size noted.
- [ ] **A6** — *Needs Deck Validation:* the controller navigates every screen in Gaming Mode, no
  double input with the default Steam layout; text readable (`docs/deck-testing.md`).

## Tasks

- [x] Tokens, fonts, logo, shell and hint bar
- [x] Focus system and gilrs input
- [x] Screens on fixtures
- [x] Headless renders and navigation tests

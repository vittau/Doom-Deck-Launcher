# 030 — Navigation and feedback polish from the first Deck test

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 008, 010, 012, 015, 019
**Origin:** Deck test 2026-10-19 on `v0.0.6-local`, owner feedback in Gaming Mode.

## Goal

On the Deck, the player always knows where the controller can go and what each button does,
and feedback never sticks on screen.

## Requirements

- **R1** — **Tab bar shows when it works:** on every screen where L1/R1 and D-pad-up cannot reach
  the tabs (first run, Mapset detail, Play setup, progress, dialogs and sheets — today
  `root()` is false or a sheet is open), the tab bar is visibly disabled: greyed, no active
  underline, not focusable. On Home, Discover, Library and Settings it looks and works as
  today. Owner: "dá impressão que sempre dá pra navegar ali, é confuso."
- **R2** — **First run points at the next step:** on the first-run screen, **Install DSDA-Doom**
  is the clear primary action: the Ember accent red (D-020/D-021 tokens) and a subtle,
  slow pulse while it is focused and not yet done. The pulse stops once the engine is
  installed and never runs during gameplay.
- **R3** — **Button hints:** a compact hint row shows the buttons that act on the current screen
  (e.g. A Select · B Back · L1/R1 Tabs · L2/R2 Category · X Favorite · Start Menu), using the
  Deck's button names, and changes with the screen and with an open dialog. It must not
  crowd the 1280×800 layout.
- **R4** — **Header:** remove the "DoomDeckLauncher" text at the top right of the header (window
  title stays). Owner: too much information in the header.
- **R5** — **Discover spacing:** a clear margin between the header ("Curated worlds. Choose your
  next descent") and the card list, kept while the list scrolls so cards never butt against
  the header.
- **R6** — **Category strip:** holding L2 or R2 repeats the category step after about 0.5 s, then
  steps continuously; L2/R2 and D-pad left/right on the strip wrap around at both ends.
  Reuse the input mapper's repeat timer (`crates/ddl/src/input.rs`) rather than gilrs repeats.
- **R7** — **Toasts dismiss themselves:** every toast (e.g. "Catalog is up to date") stays about
  3 s and fades out; a newer toast replaces an older one; changing tab does not keep a stale
  one. Errors that need a decision stay dialogs, not toasts.

## Constraints

- Ember Cathedral look (D-020, D-021, `design/visual/`) stays; keep the 1280×800 layout.
- Headless renders only; never open windows.

## Acceptance criteria

- [ ] **A1** (R1) — navigation tests: on first run and on Mapset detail, L1/R1 and Up never focus
  a tab; the tab bar renders disabled (PNG); on the four root screens behaviour is unchanged.
- [ ] **A2** (R2, R3, R4, R5) — headless renders of first run, Home, Discover (scrolled) and a
  dialog, inspected and attached to the report.
- [ ] **A3** (R6) — mapper tests: hold L2 → one step, nothing until ~0.5 s, then repeats; release
  stops; wrap at both ends.
- [ ] **A4** (R7) — a test advances time and asserts the toast is gone after ~3 s and does not
  survive a tab change.
- [ ] **A5** — fmt, clippy `-D warnings`, tests green; README renders refreshed if they change.
  Deck re-check: S4, S2 (Needs Deck Validation).

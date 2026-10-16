# 010 — Wire the UI to the core: first run, install, play, settings

**Status:** Draft — becomes Ready when 008 and 009 merge  ·  **Milestone:** M1/M2 exit
**Depends on:** 004, 005, 007, 008, 009

## Goal

The first build that actually plays: a fresh Steam Deck user installs DSDA-Doom from DDL, points
DDL at their IWADs, picks SIGIL, SIGIL II or AUGER;ZENITH, installs it, chooses components, and
plays — controller only (OVERVIEW §3, Milestone 1–2 exit criteria).

## Requirements

- **R1** — **App services** in `crates/ddl` (thin; logic stays in `ddl-core`): on startup resolve
  XDG paths, provision the DSDA config (005), load the catalog (009), the IWAD registry (007),
  engine state (004) and profiles (009). All slow work (downloads, hashing, extraction, discovery)
  runs off the UI thread and reports back with `slint::invoke_from_event_loop`; the UI never
  blocks.
- **R2** — **First run** (`ui.md` First-run): if no engine is installed, offer **Install
  DSDA-Doom** (reviewed v0.30.1) with progress and cancel; then **Find IWADs**: Steam libraries
  automatically, plus **Add folder** via a controller-friendly directory browser (home, Steam
  libraries, `/run/media/*`; D-pad navigation, no text entry needed). Show what was identified,
  and ask the user to pick when identification is ambiguous.
- **R3** — **Discover / Detail / Install:** real catalog entries with base-game status (D-010);
  Install downloads and extracts with progress, cancel and typed, actionable errors (`ui.md` §6).
- **R4** — **Play setup:** the real `Resolution::summary()` (009) drives the "Will launch" panel;
  component toggles, exclusive choices, skill and pistol start persist to the profile;
  user-supplied components offer **Import** through the same directory browser; Play is disabled
  with the blocker's explanation and a next step when `resolve()` returns a `Blocker`.
- **R5** — **Play:** acquire the run guard, prepare dirs, run DSDA via 003's runner on a worker
  thread; while it runs DDL shows a "Playing…" state and ignores input; on exit it restores focus
  to the Play button, updates *last played*, and shows a non-blocking error with the log path if
  the engine failed (exit 255 / signal).
- **R6** — **Home / Library:** Continue uses the last played profile; Library lists installed
  mapsets.
- **R7** — **Settings:** engine version and attribution (license, source link), check for updates
  (explicit only), rollback, IWAD folders (add/remove/rescan), DSDA config **Reset to
  recommended** with confirmation, open log location text, app version.
- **R8** — **Headless tests** drive the real services with fakes (fake HTTP server, fake engine
  runner, temp dirs) through first-run → install → play → return, asserting UI state.
  **Never open windows or capture the screen.**

## Acceptance criteria

- [ ] **A1** (R1–R8) — headless end-to-end test as in R8 passes; headless PNGs of each state.
- [ ] **A2** — fmt, clippy `-D warnings`, tests green; CI green.
- [ ] **A3** — *Needs Deck Validation:* the full journey on the Deck (`docs/deck-testing.md`
  E1–E5, C1–C4, M1–M6).

# 029 — Deck display: the UI fits the screen and DSDA-Doom starts widescreen

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 005, 010, 017
**Origin:** Deck test 2026-10-19 on `v0.0.6-local`: (1) in Gaming Mode DDL rendered at about 2×,
so only the Home and Discover tabs fit (check S3 ❌); the Steam launch option
`SLINT_SCALE_FACTOR=1 %command%` fixed it. (2) DSDA-Doom opened in 4:3 (check F4 ⚠️), so the
widescreen status bars of spec 017 had no room. Owner: "Faça uma config padrão que seja boa
para o Deck."

## Facts

- **Confirmed (source):** on X11, winit 0.30 takes the scale factor from
  `WINIT_X11_SCALE_FACTOR`, else `Xft.dpi`, else the XRandR millimetre size of the monitor
  ([winit 0.30.13 `src/window.rs`, `scale_factor` docs](https://docs.rs/winit/0.30.13/winit/window/struct.Window.html#method.scale_factor)).
  Gaming Mode runs DDL under gamescope's Xwayland, and the 7" 1280×800 panel gives a high DPI.
- **Confirmed (source):** Slint 1.18.1's winit backend reads `SLINT_SCALE_FACTOR` and then ignores
  the reported factor (`i-slint-backend-winit-1.18.1/winitwindowadapter.rs:601` and `:1544`).
- **Confirmed (Deck):** `SLINT_SCALE_FACTOR=1` gives the intended 1280×800 layout.
- DDL's DSDA template (`crates/ddl-core/templates/dsda/steam-deck-default.toml`) sets only
  controller and mouse keys; every video key is DSDA's default.

## Requirements

- **R1** — **Fit, not DPI:** in fullscreen the UI's scale factor comes from the window's physical
  size, never from the reported DPI: `min(width / 1280, height / 800)`, so the Deck panel gets
  exactly 1.0 and a larger screen scales the 1280×800 design up without cropping (letterbox the
  spare axis with the background). `--windowed` keeps a 1280×800 logical window. A user-set
  `SLINT_SCALE_FACTOR` still wins. Linux-specific code stays behind `cfg` or a trait so macOS
  builds and tests.
- **R2** — **Diagnostics:** `ddl.log` records, at startup and on change, the reported scale factor,
  the window's physical size and the factor DDL applied.
- **R3** — **DSDA video research:** from DSDA-Doom **v0.30.1** source (pin the tag, cite file and
  line), find the config keys and values for: resolution, fullscreen mode (desktop vs
  exclusive), aspect ratio / widescreen rendering, status-bar and HUD scaling, and any key the
  doom_widehud status bars need to show. Record them in `docs/research/dsda-engine.md`, each
  labelled Confirmed or Proposed. Do not invent keys.
- **R4** — **Deck video defaults:** the template sets a Deck-good baseline from R3: native
  1280×800, desktop fullscreen, 16:10 widescreen rendering, a status bar and HUD scaled for the
  7" screen, and the widescreen status bar visible. Bump `template_version` so existing installs
  migrate through spec 005's three-way merge: values the player changed in DSDA's menu are kept.
- **R5** — **Guide:** `docs/steam-deck-guide.md` no longer needs a launch option for scaling;
  keep one troubleshooting line about `SLINT_SCALE_FACTOR`.

## Acceptance criteria

- [ ] **A1** (R1) — headless tests: a 1280×800 physical window that reports scale 2.25 lays out
  exactly like the 1280×800 / 1.0 render (all four tabs visible, same focus geometry); a
  1920×1080 window renders the design scaled, uncropped; `SLINT_SCALE_FACTOR` overrides both.
  PNGs inspected.
- [ ] **A2** (R2) — a test asserts the three values reach the log.
- [ ] **A3** (R3, R4) — template tests: the new keys parse, sit in DSDA's documented ranges, and a
  migration from template v1 keeps a user-changed value while adding the new defaults.
- [ ] **A4** — fmt, clippy `-D warnings`, tests green. Deck re-check: S3 without a launch option,
  F4 widescreen with the status bar (Needs Deck Validation).

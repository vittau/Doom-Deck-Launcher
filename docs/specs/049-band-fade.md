# 049 — Fade from the title panel into the art band

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 012, 033
**Origin:** owner, 2026-10-19, on the hard vertical edge where the dark title panel meets the art
band image at the top of content screens: "Na transição do header pra screen, bota um fadezinho do
black pra imagem."

## Requirements

- **R1** — In `crates/ddl/ui/app-window.slint` (title panel `Rectangle` at y 120 and the
  `art-band` image beside it, lines ~327–331), the image's left edge fades from the title panel's
  colour (`Tokens.deep`) to fully transparent over about 120 px, so there is no hard edge. Use a
  Slint `@linear-gradient` overlay (no new raster asset). The title text keeps its place and
  contrast; screens without an art band are unchanged.
- **R2** — Refresh the README screenshots.

## Acceptance criteria

- [ ] **A1** — headless renders of Mapset detail and one screen with each band source (screenshot
  and painted fallback) show the fade; a pixel row across the joint has no step.
- [ ] **A2** — fmt, clippy `-D warnings`, tests green; `readme-screenshots.py --check` passes.

# 013 — Project logo, app icon and Steam artwork

**Status:** Ready  ·  **Milestone:** M2  ·  **Depends on:** 012
**Evidence:** D-020, D-021 (Ember Cathedral identity), `docs/design/visual-v2/`.

## Goal

DDL has one original, memorable logo in the Ember Cathedral style that reads well from a 32 px
icon up to a 1920 px banner, used consistently in the app, the release and the repository.
Owner request (2026-10-16): "um logo para o projeto, e usá-lo onde apropriado", including in
the binary where possible.

## Requirements

- **R1** — **Logo design:** an original emblem + "DDL" / "DoomDeckLauncher" wordmark lock-ups
  (emblem only, horizontal, stacked), generated/drawn for DDL. No id Software / Bethesda marks,
  no DOOM-logo imitation (D-020). Legible at 32 px (emblem) and on the dark header.
- **R2** — **SVG master (owner request):** the logo is a hand-authored, clean **SVG** (paths
  and fills, no embedded raster, no external fonts — text converted to outlines), under
  `docs/design/logo/` in the lock-ups from R1, plus a short usage note (clear space, minimum
  size, dark/light backgrounds). Raster image generation may be used for concepts only; the
  final mark is vector. Every PNG in this spec is exported from the SVG by a committed,
  reproducible script (e.g. `resvg`/`rsvg-convert`), never hand-edited.
- **R3** — **In the binary:** the app header, loading screen and Settings → About use the SVG
  directly (Slint renders SVG), and the window icon is set from an embedded PNG exported from
  it — all compiled in.
- **R4** — **Steam artwork shipped in the release tarball** (`ddl/art/`): square icon
  (256×256 and 512×512), library capsule 600×900, wide capsule 920×430, hero 3840×1240 and logo
  1280×720 (transparent), at the sizes Steam's custom-artwork dialog accepts — cite the source
  for those sizes. README gains a short "Make it look native in Steam" section: set the
  shortcut icon and the custom artwork by hand (DDL never writes Steam configuration).
- **R5** — **Repository:** the README shows the logo at the top; a 1280×640 social preview PNG
  is committed (`docs/design/logo/social-preview.png`) for the owner to upload in GitHub
  settings.

## Out of scope

Writing Steam's `shortcuts.vdf` or grid folder (never, OVERVIEW §8). Desktop-file installation.

## Acceptance criteria

- [ ] **A1** (R1–R2) — SVG masters, export script and usage note committed; the emblem is shown at 32, 64, 256 px
  on dark and light in one contact-sheet PNG.
- [ ] **A2** (R3) — headless renders of header, loading and About show the logo; a test asserts
  the window icon is set.
- [ ] **A3** (R4–R5) — release workflow packages `ddl/art/*`; README section and logo present;
  fmt, clippy `-D warnings`, tests green.
- [ ] **A4** — *Needs Deck Validation:* the custom artwork and icon look right in the Steam
  library and Gaming Mode.

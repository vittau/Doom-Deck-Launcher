# 039 — Apply logo v3 (Stained Glass) everywhere

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 013, 016, 036 (D-029)
**Origin:** owner approved logo v3 B Stained Glass on 2026-10-19 ("Está ótimo, pode seguir").

## Requirements

- **R1** — **Production masters:** `docs/design/logo/` holds the v3 masters (emblem, small-tier
  emblem, horizontal, stacked, full-name and every Steam/hero/capsule/social composition spec
  013/016 defined), authored from `docs/design/logo-v3/` (reuse its `author.py` geometry; one
  source of truth, no hand-edited copies). Keep the v2 production files only as history in
  `docs/design/logo-v2/` (or a `previous/` folder); nothing ships the old mark.
- **R2** — **Exports:** `docs/design/logo/export.py` regenerates every PNG (window icon from the
  **small tier**, Steam art in `ddl/art/` sizes from spec 013 R4, README logo, 1280×640 social
  preview) with the pinned resvg; `--check` passes and proves byte-identical reproduction.
- **R3** — **In the binary:** header, loading screen, Settings → About and the window icon use the
  v3 assets (`crates/ddl/ui/assets/logo/`); the header uses the small-tier lock-up at its size.
- **R4** — **Repository:** README top logo and `docs/design/logo/README.md` usage notes updated;
  credit/licence text unchanged (original art GPL-3.0-or-later, Barlow per OFL).
- **R5** — **Social preview:** regenerate the 1280×640 PNG; the owner uploads it to GitHub by hand
  (note it in the report).

## Acceptance criteria

- [ ] **A1** (R1, R2) — `export.py --check` passes; a contact sheet of the production set is
  committed and inspected.
- [ ] **A2** (R3) — headless renders of header, loading and About show the v3 logo; the window-icon
  test passes; README renders refreshed.
- [ ] **A3** — fmt, clippy `-D warnings`, tests green.

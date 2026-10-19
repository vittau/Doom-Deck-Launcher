# 047 — Transparent README logo for light and dark themes

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 039, D-029
**Origin:** owner, 2026-10-19: "Ajusta o readme-logo.png pra ter transparência." Today
`docs/design/logo/author.py` `readme_logo()` draws the full name on a dark tablet so the bone
lettering reads on both GitHub themes; a transparent background alone would hide the name on the
light theme.

## Requirements

- **R1** — **Two transparent variants** from `author.py` (one source of truth, D-029 geometry
  unchanged): `readme-logo-dark.svg/.png` (for GitHub's dark theme: today's bone name, no tablet)
  and `readme-logo-light.svg/.png` (for the light theme: the same lock-up with the name in the
  logo's dark ink, `#120C0D` or the nearest existing brand token). Background fully transparent
  (alpha 0 outside the artwork); the emblem is unchanged in both. Exported by `export.py` with
  the pinned resvg; `--check` passes. Remove the old tablet `readme-logo.*` or keep it only if
  something else references it (say which in the report).
- **R2** — **README** shows the right variant with
  `<picture><source media="(prefers-color-scheme: dark)" srcset="…-dark.png"><img src="…-light.png" alt="…" width="700"></picture>`;
  keep the alt text and width. `check-readme-links.py` passes. Update `docs/design/logo/README.md`
  usage notes and the contact sheet.

## Acceptance criteria

- [ ] **A1** — `export.py --check` passes; an alpha check (script output in the report) shows
  transparent corners and edges in both PNGs; contrast of the name against GitHub's light
  (`#ffffff`) and dark (`#0d1117`) backgrounds is ≥ 4.5:1 for its variant.
- [ ] **A2** — composite previews of both variants on both GitHub backgrounds, inspected and
  saved under the worktree's `target/` (not committed).
- [ ] **A3** — `check-readme-links.py` passes; no Rust changes, so fmt/clippy/tests are not
  required beyond `cargo test -p ddl --test logo_assets` if it covers these files.

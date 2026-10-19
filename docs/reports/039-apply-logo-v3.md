# 039 — Apply logo v3 (Stained Glass) everywhere

## What changed

- **One authoring source (R1).** `docs/design/logo/author.py` now imports the geometry, palette and
  outlined lettering from `docs/design/logo-v3/author.py` and only composes the masters and Steam
  compositions. `emblem.svg`, `emblem-small.svg` and `horizontal.svg` are byte-identical to
  `logo-v3/stained-glass/`. `stacked.svg`, `full-name.svg`, `steam-logo.svg`, both capsules and the
  social SVGs use the full-tier emblem and the approved v3 wordmark. The woodcut scenery code is
  unchanged, so `steam/hero.png` is byte-identical to before.
- **History.** The D-023 production SVGs moved (`git mv`) to `docs/design/logo-v2/production-d023/`
  (noted in `logo-v2/README.md`). Their PNGs live in Git history only. Nothing ships the old mark.
- **Exports (R2).** `docs/design/logo/export.py` reuses the v3 validator (element/attribute allowlist,
  local gradient ids only, production silhouette) and adds the checks listed below. Window icon
  (`crates/ddl/ui/assets/logo/icon.png`) and `steam/icon-256.png` come from the **small tier**;
  `icon-512.png` from the full tier.
- **In the binary (R3).** No Slint/Rust code needed to change: the header already uses
  `horizontal.svg` (small-tier lock-up at 154×56), loading uses `stacked.svg`, About `full-name.svg`
  and the window icon `icon.png`. Compiled copies now include `emblem-small.svg`. The test
  `logo_assets.rs` also covers it and the new PNG sizes.
- **Repository (R4).** README top logo is the new `readme-logo.png`; `docs/design/logo/README.md`
  rewritten (tiers, minimum sizes, exports table, reproduction); `logo-v3/README.md`,
  `SOCIAL-PREVIEW-REPORT.md` and the Steam guide mention D-029. `logo-v3/author.py` got a two-line
  refactor: the comparison sheet reads the D-023 emblem from `logo-v2/rose-compass/`, so the v3
  comparison stays byte-identical. Licence/credit text is unchanged.
- **README screenshots** regenerated with `scripts/readme-screenshots.py` (header shows the v3 lock-up).
- **Social preview (R5).** `docs/design/logo/social-preview.png` regenerated (1280×640, 455,896 B,
  tagline contrast 10.62:1). The two alternative previews were regenerated too.
  **The owner uploads `social-preview.png` to GitHub settings by hand.**

## Acceptance criteria

- **A1.** `export.py --check` passes (also run after a clean `author.py`: tree stays clean, so it
  is deterministic) with resvg 0.47.0 (`/tmp/ddl-logo-tools`) and Pillow 12.3.0. It asserts byte equality of all
  PNGs, dimensions, RGBA/indexed type, the SVG allowlist, silhouette, the three v3 masters equal to
  their `logo-v3` sources, compiled copies equal to masters, unchanged logo contours in the social
  compositions, tagline contrast, the 1 MB social ceiling and the brand colours. `logo-v3/export.py
  --check` still passes. `docs/design/logo/contact-sheet.png` (1200×1200) is committed. I opened
  every exported PNG: contact sheet, icon-256/512, library and wide capsules, Steam logo, hero,
  README logo, the three social previews. All render correctly.
- **A2.** Headless renders (Slint software renderer, `DDL_HEADLESS_DIR` scratch, no window, no
  screen capture) of Home/Discover (header lock-up), `010-loading.png` (stacked) and
  `013-about.png` (full-name) were read and show the v3 logo. The window-icon assertion in
  `headless.rs` (256×256, `icon: app-icon;`) passes. README renders and links: `readme-screenshots.py
  --check` and `check-readme-links.py` pass.
- **A3.** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace` are green (main already merged, nothing to resolve).

## New dependencies

None in the project. Design-time only: resvg 0.47.0 (pinned), Pillow 12.3.0 and fonttools 4.60.1
(already pinned by the existing scripts).

## Open questions / for the coordinator

1. **Icon tiers.** The spec fixes the window icon to the small tier. I also used the small tier for
   the 256 px Steam icon (it shows at list size) and the full tier for 512 px. Say so if you want
   both Steam icons from the full tier; `test icon.png == steam/icon-256.png` would then change.
2. **README logo.** `full-name.svg` sets the name in bone on a transparent ground, which is unreadable
   on GitHub's light theme (also true of the D-023 file). I added `readme-logo.svg/.png`
   (full-name on a dark tablet) and pointed the README at the PNG. `full-name.svg` and `stacked.svg`
   themselves are unchanged in role, because the app draws them on dark panels.
3. **Social preview variants** (Threshold, Atlas) were regenerated with the lock-up; they show the
   dark tablet behind DDL on the scene, as the approved lock-up defines.
4. `docs/design/logo/` SVGs are bigger (contact-sheet.svg ≈ 280 KB, others 17–37 KB); PNG sizes are
   in the README table's source commands' output (steam art ≤ 130 KB each).

## Spec items left open

None. A4 of spec 013 (Deck validation of the Steam artwork) stays open and now applies to the new art.

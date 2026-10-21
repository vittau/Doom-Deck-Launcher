# Spec 081 — README branding matching the website

## Changes

- Added `readme-header-dark.svg/.png` and `readme-header-light.svg/.png`: the website
  header composition displayed at 370×160 (twice the site's 185×80 header slot), built
  from the existing masters and fonts. It scales the site's `.brand` geometry 2×: the
  small-tier emblem at 128 px, the production DDL wordmark at `152,30` and the product name
  outlined from the bundled Barlow at 22 px. No new identity, geometry, typeface or colour
  token was introduced.
- The dark variant keeps the stained-glass wordmark's bone face and ruby carved side with a
  bone subtitle. The light variant swaps only the face and subtitle to the existing dark
  ink `#120C0D`, keeping the ruby side, so the mark reads on white without a background
  tablet.
- `author.py` now composes both variants; `export.py` exports them at 740×320 (2× the
  displayed size) and validates size, contours, theme fills, transparent edges and
  contrast. The production contact sheet now shows the README header in both theme rows.
- README's first line is a centered `<picture>` linked to `https://doomdecklauncher.com/`,
  with the dark PNG in a `prefers-color-scheme: dark` source and the light PNG as fallback,
  displayed at `width="370"`. Alt text describes the emblem, wordmark and product name.
  All other README content is untouched.
- `docs/design/logo/README.md` documents the new assets, their theme fills, export sizes
  and reproduction. The older `readme-logo-*` full-name lock-ups are retained as reference
  marks (still used by the existing PNG contract test and historical records) but are no
  longer referenced by README.

## Verification

Merged `origin/main` through `783e3fc` before final checks (merge commit `22023c0`); the
README change merged cleanly and the later checks ran on the merged tree.

Reproduced with the pinned design tools (no new dependencies):

```sh
/tmp/ddl-logo-081-venv/bin/python docs/design/logo/author.py
/tmp/ddl-logo-081-venv/bin/python docs/design/logo/export.py --resvg /tmp/ddl-logo-tools/bin/resvg
/tmp/ddl-logo-081-venv/bin/python docs/design/logo/export.py --resvg /tmp/ddl-logo-tools/bin/resvg --check
```

`export.py --check` passed. Relevant output:

```text
readme-header-dark.svg: compact header, small-tier emblem and DDL contours unchanged; corners alpha=[0, 0, 0, 0]; all edges alpha=0; ink #F8EAD5 on #0d1117 contrast=15.97:1
docs/design/logo/readme-header-dark.png	740x320	46323 bytes
readme-header-light.svg: compact header, small-tier emblem and DDL contours unchanged; corners alpha=[0, 0, 0, 0]; all edges alpha=0; ink #120C0D on #ffffff contrast=19.37:1
docs/design/logo/readme-header-light.png	740x320	46183 bytes
```

The check also asserts that the emblem paths are byte-equal to `emblem-small.svg` and that
all six DDL paths keep the `horizontal.svg` contours, changing only the theme fills. Both
PNGs are RGBA, 740×320, with fully transparent corners and edges.

Offscreen previews, inspected in this worktree (not committed, under the git-ignored
`target/`):

- `target/readme-brand-preview/dark-on-dark.png` — dark variant on GitHub dark `#0d1117`
- `target/readme-brand-preview/light-on-light.png` — light variant on `#ffffff`
- `target/readme-brand-preview/comparison.png` — four-up sheet, including both
  cross-theme pairings that demonstrate why the theme split is needed
- `target/readme-brand-preview/readme-mock-light.png`
- `target/readme-brand-preview/readme-mock-dark.png` — the image at its declared
  370×160 beside an H1 and body copy in GitHub-like colours

Results: the correct pairing keeps the small-tier emblem's stained glass, the DDL face and
the subtitle legible in both themes; the cross-theme pairings visibly lose the DDL face or
subtitle, which is why README selects by `prefers-color-scheme`. The compact lock-up reads
as the site header and leaves the H1 as the page's first heading. Root visual review
approved both theme compositions and asked for the 370×160 README display size, so the
185×80 site-header artwork is displayed at 2× without changing its geometry.

Asset resolution and existing checks:

```text
python3 scripts/check-readme-links.py
PASS: 38 local links/anchors in README and Steam Deck guide
```

`git diff --check` passed. No Rust tests were needed for this documentation/artwork change;
the existing `logo_assets` PNG contract and its `readme-logo-*` entries are unchanged. No
visible windows were opened and no screen was captured; all renders were headless.

## Open questions and remaining work

None. Spec 081 is fully covered on the README, asset and documentation side.

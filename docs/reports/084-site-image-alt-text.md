# 084 — Alt text on every website image

## Outcome

Owner-reported follow-up: Bing Webmaster Tools flagged images on `doomdecklauncher.com` as
missing an `alt` attribute. The templates described five images with `alt=""` — the header
emblem, the header wordmark, the hero key art, the Donate cup and the footer emblem — because
they were treated as decorative. `alt=""` is the WCAG-correct way to hide a decorative image
([W3C WAI, decorative images](https://www.w3.org/WAI/tutorials/images/decorative/)), but Bing's
scan reports an empty `alt` as missing, so the pages satisfied accessibility and still failed the
crawler check. All five now carry a description, while the images that only repeat adjacent text
stay out of the accessibility tree.

No visual, layout, asset, renderer, workflow, JSON-LD, analytics or Rust change: the templates
only gained attribute values, and `credits.html` and `404.html` contain no `<img>` at all.

## Changes

| File | Change |
| --- | --- |
| `packaging/flatpak/site/index.html` | Five `alt=""` values replaced with descriptions; footer emblem marked `aria-hidden` |
| `packaging/flatpak/site/test_render.py` | New `test_images_carry_alt_text_without_duplicate_accessible_names`; the shared image assertions now reject empty `alt` |
| `packaging/flatpak/site/test_discovery.py` | Responsive-image check now requires non-blank `alt` instead of merely present |

### Alt text added

| Image | `alt` | Assistive technology |
| --- | --- | --- |
| Header emblem (`emblem-small.svg`) | `DoomDeckLauncher emblem` | Ignored: the link has `aria-label="DoomDeckLauncher home"` |
| Header wordmark (`horizontal.svg`) | `DoomDeckLauncher wordmark` | Ignored: inside `<span class="brand-wordmark" aria-hidden="true">` |
| Hero key art (`hero.png` / WebP derivative) | `DoomDeckLauncher’s Rose Compass emblem in the window of an Ember Cathedral interior` | Ignored: the hero art layer is already `aria-hidden="true"` |
| Donate cup (`bmc-logo.svg`) | `Buy Me a Coffee` | Announced with the visible `Donate` label: "Buy Me a Coffee Donate" |
| Footer emblem (`emblem-small.svg`) | `DoomDeckLauncher emblem` | Hidden with `aria-hidden="true"`, so the link name stays `DoomDeckLauncher` and is not read twice |

The accessible names above were the reason the images used empty `alt` in the first place. Where
the description would be announced twice, the image is now removed from the accessibility tree
(`aria-hidden`) instead of relying on an empty attribute; where an ancestor or the link label
already hides it, the wrapper was left untouched. Every image keeps its `width`/`height`, lazy
loading and `fetchpriority` behaviour.

## Verification

- `python3 -m unittest discover -s packaging/flatpak/site` — 41 tests, OK (the new one included).
- `python3 -m unittest discover -s scripts/flatpak` — 23 tests, OK.
- Rendered the page locally with `scripts/flatpak/render-site.py` into a scratch directory and
  parsed the output: **10 `<img>`, 0 empty `alt`**, and no `<img>` on `assets/credits.html` or
  `404.html`. Alt values are the five above plus the five screenshot descriptions that already
  existed.
- The only template delta is attribute text, so there is no layout or paint change to compare. No
  visible window was opened and no screen was captured.

## Open questions and owner-only work

- **Publication:** the fix must reach `main`; the `Project website` workflow then republishes the
  page (this branch was not pushed).
- **Bing:** the warning clears on the next crawl/scan, not on deploy. Re-run the Site Scan or the
  SEO report after publication; a stale "missing alt" entry for an already fixed URL usually
  needs a new scan before it disappears. Bing Webmaster Tools was not accessed from this machine,
  so the reported URLs were not compared with the five images fixed here.
- The wordmark, hero art and footer emblem remain decorative for screen readers by design; they
  keep text descriptions only so crawlers see a non-empty `alt`. If the owner prefers strict
  semantics over the crawler warning, those three can revert to `alt=""` while the header
  emblem, Donate cup and descriptions stay.

# 083 — Improve search discovery and website delivery

## Outcome

Implemented the repository-side recommendations from report 080 for the project website and
added one owner follow-up request (R7): root `robots.txt` and `sitemap.xml`, accurate
`WebSite`/`SoftwareApplication` JSON-LD and social metadata, a factual product paragraph plus six
question-shaped answers, responsive WebP delivery for the hero and screenshots, credits-page
metadata, a small branded 404, and the official Donate widget moved out of the header to below the
hero release line. Both publication paths now allow and retain the new root files. No Rust code,
shared PNG, README branding, coordinator document, signed repository artifact, workflow secret,
console or external account was touched, and nothing was pushed, tagged or released.

All 37 offline website tests pass (24 pre-existing, 13 new), as do the 23 `scripts/flatpak` tests
and the `served_urls.sh` test. Three headless local Lighthouse runs move total lab transfer from
**3 226 KiB to 758 KiB** (median) with performance **74 → 95**; these are single-machine lab
results, not field data, and nothing here promises indexing, rankings or citations.

## Changes

| File | Change |
| --- | --- |
| `scripts/flatpak/render-site.py` | Renders `robots.txt`, `sitemap.xml`, JSON-LD (serialized, script-boundary safe), the 404 page and the WebP derivative copies; still standard library only |
| `scripts/flatpak/generate-site-images.py` | New reproducible `cwebp` generator/checker for committed website derivatives |
| `packaging/flatpak/site/images/` | 18 committed WebP derivatives + `manifest.json` (encoder, source hashes, dimensions, quality, output hashes/bytes) |
| `packaging/flatpak/site/index.html` | JSON-LD, social metadata, product lead sentence, six Q&A articles, responsive `<picture>` delivery, Donate moved below the release line |
| `packaging/flatpak/site/credits.html` | Meta description and self-referencing canonical |
| `packaging/flatpak/site/404.html` | New small branded 404 with absolute asset URLs |
| `packaging/flatpak/site/style.css` | Hero `<picture>` layering, Q&A grid, Donate placement/spacing, 404 layout |
| `packaging/flatpak/site/test_discovery.py` | New discovery, JSON-LD, image-delivery and publication-preservation tests |
| `packaging/flatpak/site/test_render.py`, `test_identity.py` | Updated Donate placement contract and new allowlist/cleanup expectations |
| `.github/workflows/project-site.yml` | Root allowlist accepts `robots.txt`, `sitemap.xml`, `404.html`; verification compares the served discovery files |
| `.github/workflows/flatpak-remote.yml` | Root cleanup retains the discovery files and 404; verification compares them too |

The R7 Donate move was requested in coordinator follow-up `msg_372c25a9a081`; the updated spec text
was not present on `origin/main` (`5bb62b9`) when this branch merged it, so the follow-up body is
the recorded scope for that item. `origin/main` was merged again before final checks (already
up to date).

## Requirement evidence

### R1 — robots.txt and sitemap.xml

`scripts/flatpak/render-site.py` writes both files from `--site-base` on every render and refresh:

```robots
# DoomDeckLauncher project website
User-agent: *
Allow: /
Disallow: /repo/

Sitemap: https://doomdecklauncher.com/sitemap.xml
```

The policy is one wildcard group: no search or AI crawler is named or blocked, and `/repo/` is the
only exclusion. `sitemap.xml` is a schema-valid `urlset` with exactly the homepage and
`https://doomdecklauncher.com/assets/credits.html`; it deliberately has no `lastmod`, because the
only available date is the app release date, which is not the page's source-modification
provenance. Google documents `lastmod` as optional and only useful when consistently accurate
([sitemaps overview](https://developers.google.com/search/docs/crawling-indexing/sitemaps/overview)).

Both publication paths were updated in the same change:

- `project-site.yml` now allows `index.html|robots.txt|sitemap.xml|404.html|assets/*|CNAME` in its
  page-only guard (it previously rejected any other root file), and its post-deploy verification
  fetches and byte-compares `robots.txt` and `sitemap.xml`.
- `flatpak-remote.yml`'s `find … -delete` cleanup now keeps `robots.txt`, `sitemap.xml` and
  `404.html` alongside the existing root files, and its verification step byte-compares them too.

Tests: `test_robots_is_policy_neutral_and_declares_the_canonical_sitemap`,
`test_sitemap_lists_the_homepage_and_credits_without_invented_lastmod`,
`test_page_only_allowlist_accepts_discovery_and_derivative_files`,
`test_refresh_regenerates_discovery_files_and_keeps_release_metadata`, plus the extended
`test_signed_cleanup_retains_cname_and_new_descriptor`.

### R2 — structured data and social metadata

The homepage now carries one JSON-LD block built in Python from the rendered `SITE_BASE` and
`APP_ID`:

- `WebSite`: `name` DoomDeckLauncher, `alternateName` DDL, canonical `url`, `inLanguage` en.
- `SoftwareApplication`: name/url, `applicationCategory` GameApplication, `operatingSystem`
  Linux (the published artifact is a Linux Flatpak), the GPL-3.0-or-later SPDX license URL,
  `isAccessibleForFree`, `offers` with numeric `price: 0`, the canonical `.flatpakref` download
  URL, and the visible requirement sentence (“A Steam Deck and legally owned DOOM base-game files
  (IWADs). DoomDeckLauncher never bundles or downloads commercial base games.”).

No `aggregateRating`, `review`, `author`, `publisher`, `organization`, `brand` or extra platform
is asserted. That is deliberate: Google's software-app documentation still lists `name`,
`offers.price` **and** a rating/review as required for that rich result, so this markup is for
entity clarity and is **not** claimed to be rich-result eligible ([software app structured
data](https://developers.google.com/search/docs/appearance/structured-data/software-app)).
`WebSite` structured data is the primary site-name preference Google documents on the home page
([site names](https://developers.google.com/search/docs/appearance/site-names)).

Generated values are serialized with `json.dumps` and then `<`, `>`, `&` are replaced with JSON
escape sequences, so the block stays valid JSON and cannot terminate the `script` element; HTML
attribute escaping for the rest of the page is unchanged. `test_structured_data_survives_unsafe_generated_values`
renders a base containing a quote and `</script>` and reparses the block.

Social metadata added: `og:site_name`, `og:locale` (`en_US`), `og:image:type` (`image/png`, which
matches the real 1280×640 preview), `twitter:title` and `twitter:description`. Claims in the
Twitter description are also visible on the page.

### R3 — visible product description and Q&A

The approved hero headline and artwork are unchanged. The hero lead now opens with the factual
product sentence (“DoomDeckLauncher is a free, open-source, controller-first launcher for Steam
Deck…”) and the launcher section intro names the product again. A “Questions, answered.” section
adds six concise `h3` questions covering: what DDL is; price and GPL-3.0-or-later license; whether
DOOM or mapsets are included; DSDA-Doom as the only source port; installation and update flow
(signed remote, not Flathub, tarball alternative); and Steam Deck focus including “no Windows or
macOS builds are published” and the still-pending physical-Deck validation. There is no
`FAQPage`/`QAPage` markup and no keyword block: Google restricts FAQ rich results to well-known
government and health sites ([FAQ changes](https://developers.google.com/search/blog/2023/08/howto-faq-changes)).

Visible words grow from 733 to 1 051 (`index.html` raw 18 335 → 26 426 bytes, gzip 5 315 → 7 138),
a content increase that is mostly the Q&A answers.

### R4 — image delivery

`scripts/flatpak/generate-site-images.py` encodes committed derivatives with `cwebp 1.6.0 /
libsharpyuv 0.4.2` and writes `packaging/flatpak/site/images/manifest.json`, pinning each shared
PNG's SHA-256, dimensions and output hash. `--check` compares the current PNG inputs and committed
WebP bytes without invoking the encoder, and the renderer copies the committed files unchanged —
the website runtime stays standard-library only, with no image library or CDN.

| Source PNG (unchanged) | PNG bytes | 640w WebP | 960w WebP | 1280w WebP | 2×-width saving |
| --- | ---: | ---: | ---: | ---: | ---: |
| `hero.png` (3840×1240) | 1 017 287 | 23 718 (960w) | 69 366 (1920w) | 97 746 (2560w) | 90.4 % |
| `discover.png` | 534 370 | 48 962 | 87 188 | 143 708 | 73.1 % |
| `home.png` | 276 406 | 24 894 | 43 462 | 64 796 | 76.6 % |
| `play-setup.png` | 278 961 | 29 094 | 51 852 | 78 106 | 72.0 % |
| `mapset-detail.png` | 537 697 | 42 816 | 76 312 | 125 326 | 76.7 % |
| `audio-settings.png` | 217 704 | 25 664 | 43 682 | 65 134 | 70.1 % |

Hero candidates are 960/1920/2560 (`sizes="100vw"`); screenshots are 640/960/1280 with per-layout
`sizes`. A 1440 px desktop at 1× fetches the 1920 hero plus five 640 stills: **240 796 bytes**
instead of the previous **2 862 425 bytes** of PNGs (−91.6 %). Screenshot quality is 88, chosen
after comparing a magnified text crop with the PNG original: the UI labels and controller prompts
stay crisp, and every anchor still opens the untouched full-size PNG. `fetchpriority="high"` is
set only on the hero artwork and the above-fold Discover preview; the four below-fold screenshots
keep `loading="lazy" decoding="async"`. Hero and screenshot images keep explicit width/height, and
`<picture>` always includes a PNG fallback plus a versioned WebP `srcset`.

Staleness is guarded in three places: the manifest records source and output hashes,
`generate-site-images.py --check` fails on drift, and
`test_webp_derivatives_are_current_and_cache_versioned` / `test_stale_derivative_manifest_is_rejected`
fail CI when a shared PNG changes without regenerated derivatives.

The official Buy Me a Coffee embed keeps its exact owner configuration (slug `vittau`, `#FFDD00`,
Cookie font, official script, `<noscript>` link). No `defer`/preconnect was added: local runs did
not reproduce report 080's Google-Fonts render block, so changing the embed would have been an
unmeasured branding edit.

### R5 — credits page and branded 404

`assets/credits.html` now has a unique meta description and a self-referencing canonical
(`https://doomdecklauncher.com/assets/credits.html`), stays indexable, and remains in the sitemap.
No `noindex` was added.

The optional 404 was included because it is small (1 646 bytes raw, 705 gzip) and both publishers
now write root files: it carries the site stylesheet/icon through the canonical apex, a noindex
directive, no JavaScript, and links home, to install steps, credits and the repository. Absolute
URLs are intentional — GitHub Pages keeps the missing path in the address bar, so relative asset
paths would break on nested misses. Both workflow allowlists/cleanup were updated in R1.

### R6 — tests and verification

New offline coverage in `test_discovery.py`:

- robots/sitemap topology, wildcard-only policy, canonical sitemap, no invented `lastmod`, URLs
  resolving to rendered pages, credits indexability;
- JSON-LD parses, only documented facts, canonical agreement, no ratings/reviews/author/org, no
  Windows/macOS platform claim, safe serialization of unsafe generated values;
- social metadata values and real social-preview dimensions;
- derivative manifest truth (18 files, source hashes, dimensions, byte counts), stale-manifest
  rejection, `<picture>`/`srcset` candidates and cache versions, PNG fallbacks and full-size links,
  above-fold priority vs below-fold lazy;
- page-only allowlist acceptance/rejection and cleanup retention of discovery files;
- Q&A/product copy surviving with every script removed (GA stays initialized once per page via the
  existing 082 test).

Headless responsive verification used the installed Chrome through Playwright, full-page, with no
visible windows and no screen capture:

| Viewport | Page height | `scrollWidth` | Selected images |
| --- | ---: | ---: | --- |
| 390×844 | 8 737 px | 390 = `clientWidth` | hero 960w, Discover 640w; below-fold lazy images not fetched |
| 768×1024 | 5 689 px | 768 = `clientWidth` | hero 960w, all stills 640w |
| 1440×900 | 4 907 px | 1440 = `clientWidth` | hero 1920w, all stills 640w |

The approved hero artwork, headline and hierarchy read the same as the pre-change render. An
earlier screenshot taken with a short virtual-time budget showed clipped headline text before the
Pirata One font loaded; the font-settled pre- and post-change renders both wrap correctly and have
no horizontal overflow, so that was a screenshot-timing artifact, not a layout defect.

Lighthouse 12.8.2, mobile emulation, headless, six local runs against a localhost copy of the
rendered tree (three pre-change, three final). These are lab numbers from one machine:

| Metric | Pre-change (3 runs) | Final (3 runs) |
| --- | --- | --- |
| Performance | 72–74 (median 74) | 95 / 95 / 95 |
| Accessibility / SEO / Best practices | 100 / 100 / 100 | 100 / 100 / 100 |
| FCP | 957–1 509 ms (median 1 507) | 1 955 ms |
| LCP | 10 431–14 652 ms (median 10 805) | 2 705 ms |
| TBT | 66–192 ms (median 71) | 27–34 ms |
| CLS | 0.001 | 0.008 |
| Total transfer | 3 226 KiB | 758 KiB |
| Responsive-image savings | 1 396 265 B | 171 667 B |
| Modern-format savings | 2 228 307 B | 0 B |

The final three runs scored identically; only TBT moved, so treat the score as stable **for this
localhost setup only**. Report 080's public-network runs varied far more (performance 70–71, LCP
5.4–10.3 s), and a single Lighthouse run is not a ranking or field-data signal. The remaining
responsive estimate is the gap between the 960w candidate and mobile DPR×slot sizing; a 720w tier
was considered and skipped (below). CLS rose from 0.001 to 0.008, still far inside the “good”
threshold, after the donation button moved into the hero; no layout shift was visible in the
renders.

## Tests run

```sh
python3 -m unittest discover -s packaging/flatpak/site      # 37 tests, OK
python3 -m unittest discover -s scripts/flatpak            # 23 tests, OK
bash scripts/flatpak/test_served_urls.sh                   # OK
python3 scripts/check-readme-links.py                      # PASS: 38 links/anchors
python3 scripts/flatpak/generate-site-images.py --check    # PASS: 18 derivatives, 1 141 826 bytes
git diff --check                                           # clean
```

No Rust build or test was run; no Rust file changed. `git diff` against `696dc7d` confirms no
change under `docs/screenshots/`, `docs/design/`, `README.md`, the 080 report or coordinator
documents.

## R7 follow-up — Donate placement

The single official widget now renders in `.hero-copy` immediately after the version/publication
date paragraph and before the “See the setup steps” link; the header contains three navigation
links only. Install remains the primary button above it. The embed attributes, yellow `#FFDD00`,
official logo, Cookie font, 44 px compact height and `<noscript>` fallback are unchanged, and
`test_render.py` now pins one instance outside the header with this order. Desktop and 390 px
renders confirm the widget fits, stays on-brand and leaves sensible spacing. Moving it out of the
header also took the third-party embed out of the initial render-critical path; the final
Lighthouse runs above were taken after this change (same tree before it scored 78–90 with LCP
3.6–5.9 s).

## Publication and handoff

Publication remains root-owned and independent: this branch only changes the source templates,
renderer, workflows, committed derivatives and tests. After merge, the page-only workflow refreshes
the site without a signed Flatpak release, and the next signed publication regenerates the same
root files. No console submission, credential use, outreach, push, tag or release happened here.

Owner-only work that remains:

1. Submit `https://doomdecklauncher.com/sitemap.xml` in Search Console and Bing Webmaster Tools
   once the changed site is live; no submission is claimed.
2. Optionally run the Rich Results Test / URL Inspection on the homepage to confirm Google parses
   the JSON-LD. Expect entity understanding, not a software-app rich result (no genuine ratings).
3. Confirm the Search Console “Search generative AI” control stays on the default *Include* choice
   (carried over from report 080; not verifiable from this repository).
4. Keep earning real citations in DOOM/Steam Deck community spaces; markup is not a substitute.

## Excluded recommendations, with reasons

- **`llms.txt`** — not documented as a requirement or ranking/citation lever by any crawler vendor
  read for report 080 (F12); omitted rather than adding an unverified convention.
- **`FAQPage`/`QAPage` markup** — explicitly excluded by the spec; Google limits FAQ rich results
  to well-known government/health sites, and QAPage is for user-generated Q&A.
- **A 720w screenshot tier** — would shave part of the remaining 171 KB responsive estimate but
  adds five more committed assets for a narrow DPR band; the 640/960/1280 ladder covers Deck 1×,
  desktop 1× and 2×.
- **Preconnect / deferred donation embed** — the owner-approved embed works as configured and the
  local runs did not show the Google-Fonts render block report 080 measured on the public network;
  no speculative change to branding behavior.
- **`lastmod` in the sitemap** — the app release date is not page-modification provenance, and the
  spec forbids inventing it.
- **Credits `noindex`** — the spec keeps the credits page indexable and in the sitemap; it also
  answers real attribution queries.
- **IndexNow / console work / repository About topics** — owner-side (key file, accounts,
  outreach), outside this repository's publication and credential scope; About metadata is already
  root-owned and updated.
- **Retired install-descriptor shim, training-crawler policy, on-site Steam Deck guide** — open
  owner decisions from report 080 §9, not repository-side discovery work.

## Sources

Read directly or carried from report 080's same-date audit (2026-10-21); the software-app and
site-name pages were re-checked while writing this report:

| Source | Used for |
| --- | --- |
| [Google — Software app structured data](https://developers.google.com/search/docs/appearance/structured-data/software-app) | Required `name`, `offers.price` and rating/review for the rich result; `price: 0` example |
| [Google — Site names](https://developers.google.com/search/docs/appearance/site-names) | `WebSite` data on the home page is the primary preference; `og:site_name`/title are secondary |
| [Google — Sitemaps overview](https://developers.google.com/search/docs/crawling-indexing/sitemaps/overview) | Discovery aid, optional `lastmod`, no crawl/index guarantee |
| [Google — robots.txt introduction](https://developers.google.com/search/docs/crawling-indexing/robots/intro) | robots is crawl management, not removal |
| [RFC 9309](https://www.rfc-editor.org/rfc/rfc9309.html) | A 4xx robots.txt means crawlers may access any resource |
| [Google — FAQ/HowTo rich-result changes](https://developers.google.com/search/blog/2023/08/howto-faq-changes) | Why no FAQ markup was added |
| [MDN — `<picture>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/picture) | Responsive WebP with PNG fallback |
| [web.dev — `fetchpriority`](https://web.dev/articles/fetch-priority) | Above-fold image priority |
| [SPDX — GPL-3.0-or-later](https://spdx.org/licenses/GPL-3.0-or-later.html) | License identifier URL used in JSON-LD |

## Preview artifacts (git-ignored, in this worktree)

- `target/site-083/` — final rendered site; `target/pre083/site/` — pre-change render.
- `target/preview-083/final-desktop-1440.png` (1440×4907), `final-tablet-768.png` (768×5689),
  `final-narrow-390.png` (390×8737), `final-credits.png`, `final-404-local.png`.
- `target/preview-083/r7-desktop-hero.png`, `r7-narrow-donate.png` — Donate placement.
- `target/preview-083/lh-old-{1,2,3}.json`, `lh-final-{1,2,3}.json` — Lighthouse lab runs.
- `target/preview-083/measure.js` — viewport/`scrollWidth`/selected-image measurement used above.

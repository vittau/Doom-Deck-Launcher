# 069 — Website favicon, current previews and DDL header

## Scope and result

Bounded presentation correction under spec 065, expanded by the owner through the
coordinator to refresh website/README screenshots and match the application header.
Merged local `main` (`d1a39cc`) before final verification. No application code,
workflows, signed repository artifacts or coordinator-owned documents changed.

- Both project pages explicitly declare the existing approved 256×256 stained-glass
  PNG favicon, with `?v=<first 16 SHA-256 characters>` matching the stylesheet cache
  convention. Credits is rendered as a template so its hash updates too.
- Relative links resolve to `/Doom-Deck-Launcher/assets/icon.png` from both index and
  nested credits. No domain-root favicon is replaced.
- The website header reuses the application's exact `horizontal.svg` symbol + DDL
  artwork, with a smaller DoomDeckLauncher subtitle aligned below the DDL wordmark.
  Its home link retains the full accessible product name.
- Website and README share refreshed Home, Discover, Play setup, mapset detail and
  Audio screenshots. Every website screenshot thumbnail and full-size link carries
  its own content hash. The existing two-column gallery accommodates detail/Audio
  without a new layout system. Credits describe real Slint renders with sample state
  and distinguish original DDL interface art from community map imagery.
- The existing screenshot regeneration script now includes the Audio headless test.

## Live HTTP finding (before correction)

Read-only `curl` inspection of the approved HTTPS site on 2026-10-21:

| Resource | Response | Finding |
| --- | --- | --- |
| `/Doom-Deck-Launcher/` | 200 HTML | Explicit `assets/icon.png`, without a content hash. |
| `/Doom-Deck-Launcher/assets/icon.png` | 200 `image/png` | Byte-identical to current approved app icon; 256×256 RGBA. |
| `/Doom-Deck-Launcher/favicon.ico` | 404 | No project-local implicit ICO. |
| `/favicon.ico` | 200 `image/vnd.microsoft.icon` | Separate domain-level multi-size ICO. |

The live icon SHA-256 is
`e3992435bcefde15bd9aec4467353d347e71ad25121f2d4ef28dcaf4ade22ae4`, identical
to `crates/ddl/ui/assets/logo/icon.png`. Both live icon responses advertised
`cache-control: max-age=86400` and Cloudflare cache hits. The credits template had
no favicon declaration. **Confirmed:** missing credits declaration and unversioned
index icon URL; **Proposed explanation:** retained browser favicon state or fallback
to the domain icon. HTTP evidence alone cannot identify the owner's cached tab icon.
No incorrect new artwork was found in the currently served project PNG.

## Asset provenance

Approved identity: D-029 / spec 039, documented in `docs/design/logo/README.md`.
Favicon source is `crates/ddl/ui/assets/logo/icon.png`; header source is
`crates/ddl/ui/assets/logo/horizontal.svg`, also used by `app-window.slint`.
Both are original GPL-3.0-or-later stained-glass project exports. No new art or AI
image generation, new fonts, or runtime dependencies were introduced.

The coordinator supplied freshly regenerated root `target/headless` captures from
the integrated 067/068 source. Inspected all five off-screen captures before copying;
no personal paths are visible. Used the existing lossless `optimized_png` function,
retaining the real 1280×800 RGB scanlines:

| Published source in `docs/screenshots/` | Headless capture/test |
| --- | --- |
| `home.png` | `library_favorite_and_home/010-home-continue.png` |
| `discover.png` | `categories_favorites_and_screenshots/015-discover-all-favorite.png` |
| `play-setup.png` | `auger_options_and_components/010-play-setup-auger.png` |
| `mapset-detail.png` | `categories_favorites_and_screenshots/015-detail-tags.png` |
| `audio-settings.png` | `audio_section_renders_and_selects_default/067-audio-settings.png` |

These are local production UI renders with explicit sample state, not physical Deck
captures. Audio shows Default selected and Arachno not installed; no installed
soundfont or playback claim is implied. Community map image provenance remains in
the existing credits/source records.

## Verification

- `python3 -m unittest discover -s packaging/flatpak/site`: 12 tests pass.
  Added assertions for exact approved icon/header bytes, both pages' favicon MIME,
  size, project-subpath URL resolution, content hashes, changed-export refresh,
  retained app release, all five shared screenshot sources/dimensions, and matching
  cache-versioned thumbnail/full-size links. Existing signed-artifact preservation,
  metadata, download links and stylesheet tests pass.
- Headless Chromium at 1280×844, 390×844 and 320×844: no horizontal overflow, all
  local images decoded, no failed local HTTP responses, official donation embed
  present, exact hashed favicon link on index and credits. Desktop/narrow header
  images visually inspected. Browser-only tooling installed in ignored `.scratch`;
  used the existing cached headless-shell executable, with no visible windows.
- Local evidence: `.scratch/favicon/browser-results.json`, `header-{width}.png`,
  `desktop-{width}.png`, `check.cjs`, HTTP headers/live assets and screenshot-check
  log. These review artifacts are intentionally uncommitted.
- `git diff --check`: passes.
- All five committed screenshot files match the existing optimizer's output from
  the coordinator-provided root captures byte for byte. A supplementary four-case
  `scripts/readme-screenshots.py --check` run was stopped after the Home journey
  passed, at the coordinator's request to avoid redundant UI work; no full suite
  was started. The coordinator had already regenerated the full current UI suite
  (870 passing tests). This report does not claim the stopped regeneration check
  completed. Reproduction now includes `headless::audio::audio_section_renders_and_selects_default`.

## Limitations and remaining publication

The owner's browser cache state cannot be inspected remotely. Coordinator must
review, publish using the independent page workflow and purge the project prefix;
then verify deployed hashed URLs. No push, tag, release or deployment was performed.
Physical Deck installation, gameplay and MIDI audio remain outside this presentation
change. No open implementation items remain for spec 065's bounded correction.

## Owner alignment correction

After publication the owner identified excessive space and vertical misalignment between DDL,
its subtitle and the emblem. The coordinator corrected this bounded layout: a separate existing
emblem sits beside a centered two-line text block, reusing the outlined DDL lettering from the
unchanged app horizontal SVG. The text lines share a left edge and a 2 px gap; no new artwork.
Twelve offline renderer tests pass. Headless Chromium at 1280, 390 and 320 px asserts matching
emblem/text vertical centers (within 0.2 px), matching text left edges, gap at most 3 px and no
horizontal overflow. Header PNGs were visually checked at `/tmp/ddl-brand-{width}.png`.

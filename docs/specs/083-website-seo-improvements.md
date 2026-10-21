# 083 — Improve search discovery and website delivery

**Status:** Ready. **Origin:** owner requests review and implementation of report 080 using OpenCode.

## Reviewed scope

Implement the useful repository-side recommendations from report 080. Current canonical domain,
HTTPS redirects, Search Console setup and signed publication are already working. Preserve the
approved Ember Cathedral design, compact header, official Buy Me a Coffee embed/logo/Cookie font,
GA4 configuration, primary Flatpak installation flow and independent website publication.

- R1: Generate root robots.txt with a policy-neutral allow for search/AI crawlers, exclude only
  the OSTree /repo/ tree and declare the canonical sitemap. Generate valid sitemap.xml containing
  the homepage and credits URL. Do not invent lastmod dates from the app-release date: omit
  optional lastmod unless genuine source modification provenance is available. Update both
  publication paths' root-file allowlists/cleanup, so neither rejects nor deletes discovery files.
- R2: Add accurate WebSite (name DoomDeckLauncher, alternateName DDL, canonical URL) and
  SoftwareApplication structured data from actual visible product facts, plus useful social
  metadata (site name, locale, image type, Twitter title/description). Never invent ratings,
  reviews, organization, author identity or supported platforms. Keep JSON-LD valid after
  rendering and safe for generated values. Free launcher/GPL may be described accurately;
  commercial DOOM base-game ownership is still required. No rich-result/indexing promises.
- R3: Improve visible factual product description and add concise useful questions/answers
  about price/license, required DOOM IWADs, DSDA-Doom, installation/updates and Steam Deck focus.
  Preserve the existing hero headline/artwork and overall hierarchy; add a natural product-name
  description instead of replacing the approved hero for keywords. Do not claim Windows/macOS
  support, Flathub listing, GZDoom support or physical Deck verification. Avoid filler and keyword
  stuffing; no FAQPage/QAPage rich-result markup.
- R4: Optimize the large hero and screenshot delivery with reproducible local WebP derivatives
  and appropriate responsive variants, fallback PNG links and explicit dimensions. Keep shared
  README/source PNGs unchanged; screenshots must retain crisp text and original full-size links.
  Hash/version derivatives so stale assets are not reused. Give the above-fold preview sensible
  loading priority; below-fold screenshots stay lazy. Prefer build-time committed derivatives,
  no runtime image library or external CDN. Preserve official donation embed behavior/branding;
  optional preconnect only if useful, no defer change that breaks its document-write behavior.
- R5: Add accurate description/canonical metadata to the credits page. Keep it indexable and
  in the sitemap; no speculative noindex policy. A branded 404 is optional only if small and
  safely supported by both publishers; it is lower priority than R1–R4.
- R6: Run existing offline website tests and add meaningful checks for robots/sitemap topology,
  JSON-LD truth/validity, local links/assets and preservation across both publication paths.
  Verify responsive headless renders at desktop and narrow widths (no visible windows or screen
  capture). Compare current/new Lighthouse lab results and actual bytes; label single-run
  variability and never guarantee rankings, citations or index inclusion. Keep content usable
  without JavaScript and GA initialized once. Cite current primary docs for technical claims.

## Ownership and handoff

Own website templates/CSS, renderer, narrow page/publication workflow changes, optimized website
assets and their reproducible generator/tests, plus docs/reports/083-website-seo-improvements.md.
Do not edit README branding, shared PNGs, Rust/application behavior, signed repository/key/ref
logic, coordinator status/journal/decisions/overview, or historical audit findings. No new runtime
project dependencies, credential use, external account changes, console submissions, outreach,
push, tag or release. Root handles repository About metadata and publication. Merge main before
final checks. Report each requirement's evidence, asset byte comparison, preview paths, tests,
remaining owner-only work and any excluded recommendation with its reason.

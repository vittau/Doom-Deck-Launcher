# 080 — SEO and AI-search discovery audit of `doomdecklauncher.com`

**Task:** research audit of conventional search and AI-search discoverability of the public
project site, with evidence, prioritized proposals and limitations. **No code, publication,
DNS, Pages, credential or console change was made.** This report is the only tracked change.

Evidence was collected on **2026-10-21 between 23:26 and 23:32 UTC** (20:26–20:32
America/Sao_Paulo) from this macOS arm64 host and the public internet. Live responses were
treated as untrusted data: they were fetched, parsed as text and never executed.

## Outcome

The site is technically clean and easy to crawl — static HTML with all text in the response,
correct HTTPS/HSTS-free TLS for both hosts, `www` and the retired project URL redirected to the
apex, healthy outbound links, accessibility 100 in Lighthouse, and no user-agent cloaking for
any crawler tested. The discoverability problem is not crawling; it is that **the live page is
the pre-migration render**: its canonical, `og:url`, `og:image`, brand links and install links
all still point at `https://www.vitormach.dev/Doom-Deck-Launcher/` and the retired application
ID. That single fact dominates the audit, and the fix is already in source but is **gated behind
the signed publication of the new Flatpak channel**. Beyond that: there is no `robots.txt`, no
sitemap, no structured data, no site-name signal, no HTTPS enforcement on plain HTTP, and only
~760 words of indexable text about the product.

## Method

- Live HTTP: `curl` with explicit user agents (browser, `Googlebot`, `GPTBot`, `OAI-SearchBot`,
  `ClaudeBot`, `PerplexityBot`, `curl`) and `--resolve` where the local stub resolver had a stale
  negative cache; headers, byte bodies and hashes compared.
- DNS/TLS: `dig` against the system resolver, `1.1.1.1`, `8.8.8.8` and the authoritative
  Cloudflare nameservers; `openssl s_client` for the certificate.
- Source: branch `vittau/seo-audit` merged with `verify/071-073` and then `origin/main`
  (`7772da8`). The renderer was executed locally into a scratch directory and diffed against the
  live bytes.
- Performance/accessibility: Lighthouse 12.8.2 run headlessly (no visible window, no screenshot)
  against the live URL. Google PageSpeed Insights API was attempted first and refused with
  `429 ... Quota exceeded ... Queries per day` for anonymous requests, so its scores are **not**
  reported here.
- Crawler and search requirements: vendor primary documentation only (section 12), read on the
  same date.

**What this audit cannot do:** it has no access to Google Search Console, Bing Webmaster Tools,
Cloudflare analytics or server logs, so it cannot state index coverage, impressions, crawl
frequency or AI-answer citations. Nothing here promises indexing, ranking or citation.

## 1. Live snapshot

### 1.1 Hosting, DNS, TLS, redirects

| Check | Observed |
| --- | --- |
| `https://doomdecklauncher.com/` | `200`, `server: GitHub.com`, `content-type: text/html; charset=utf-8`, `content-length: 18064`, `cache-control: max-age=600`, gzip with `Accept-Encoding: br, gzip` |
| `https://www.doomdecklauncher.com/` | `301` → `https://doomdecklauncher.com/` |
| `https://vittau.github.io/Doom-Deck-Launcher/` | `301` → `https://doomdecklauncher.com/` |
| `https://www.vitormach.dev/Doom-Deck-Launcher/` | `301` → `https://doomdecklauncher.com/` |
| `https://vitormach.dev/Doom-Deck-Launcher/` | `301` → `https://www.vitormach.dev/Doom-Deck-Launcher/` → `301` → apex (two hops) |
| `http://doomdecklauncher.com/` | **`200`** with the same HTML — **no HTTP→HTTPS redirect**, no `Strict-Transport-Security` header |
| `http://www.doomdecklauncher.com/` | `301` → `http://doomdecklauncher.com/` (stays on plain HTTP) |
| `https://doomdecklauncher.com` (no slash) | `200` (both variants served; canonical declares the slash form) |
| DNS | apex `A` = `185.199.108–111.153` and `AAAA` = `2606:50c0:8000::153`–`…8003::153`; `www` = `CNAME vittau.github.io.`; NS `luke.ns.cloudflare.com`, `zara.ns.cloudflare.com` |
| TLS | Let's Encrypt `CN=doomdecklauncher.com`, SAN `doomdecklauncher.com` + `www.doomdecklauncher.com`, valid `2026-10-21 22:22:42` → `2027-01-19` |
| Cloudflare | apex responses carry **no** `cf-ray`/`cf-cache-status` (DNS-only in front of Pages); the legacy `vitormach.dev` host does carry them |

The redirect topology is correct and consolidates every retired URL on the apex. The one
anomaly is plain HTTP: GitHub Pages is serving `200` instead of redirecting, which means the
`Enforce HTTPS` option appears not to be active even though HTTPS itself works
(**Confirmed** observation; the setting itself is owner-side and was not inspected).

### 1.2 Crawl controls and discovery files

| URL | Status |
| --- | --- |
| `/robots.txt` | **`404`** (GitHub 404 page, 9 379 bytes) |
| `/sitemap.xml` | **`404`** |
| `/favicon.ico` | `404` (harmless: both pages declare `<link rel="icon" href="…/icon.png">`) |
| `/site.webmanifest` | `404` |
| `/llms.txt` | `404` |
| `/assets/credits.html` | `200`, `4 249` bytes |
| `/repo/summary`, `/repo/summary.sig` | `200` (`application/octet-stream`, `application/pgp-signature`) |
| `/repo/` | `404` (expected for an OSTree remote without a directory index) |

No response carries `X-Robots-Tag`, and neither page carries a `robots` meta tag, so nothing is
excluded from indexing today. A missing `robots.txt` is not an indexing blocker: per RFC 9309
§2.3.1.3 an unavailable (4xx) `robots.txt` means a crawler **MAY** access any resource, and
Google's own introduction says the file is for crawl management, not removal.

### 1.3 Head metadata (live, verbatim)

```html
<title>DoomDeckLauncher — Community DOOM maps on Steam Deck</title>            <!-- 52 chars -->
<meta name="description" content="Own DOOM on Steam? Discover, install and play community
  mapsets on Steam Deck with DoomDeckLauncher. Download the experimental signed Flatpak.">
                                                                                <!-- 142 chars -->
<link rel="canonical" href="https://www.vitormach.dev/Doom-Deck-Launcher/">
<meta property="og:type" content="website">
<meta property="og:title" content="DoomDeckLauncher — Your next DOOM adventure">
<meta property="og:description" content="Your DOOM library. A world of community maps. A
  controller-first launcher for Steam Deck.">
<meta property="og:url" content="https://www.vitormach.dev/Doom-Deck-Launcher/">
<meta property="og:image" content="https://www.vitormach.dev/Doom-Deck-Launcher/assets/social-preview.png">
<meta property="og:image:width" content="1280">
<meta property="og:image:height" content="640">
<meta property="og:image:alt" content="DoomDeckLauncher’s original Rose Compass identity …">
<meta name="twitter:card" content="summary_large_image">
<meta name="theme-color" content="#120c0d">
```

- **Canonical and `og:url` point at a URL that 301s to the page itself.** Google documents
  redirects as a *strong* canonicalization signal and `rel="canonical"` as a *hint*, so this
  chain is not fatal, but it is a stale declaration on a host the project is retiring, and every
  social unfurl of the apex still advertises the old URL.
- `og:image` is accurate: the served `assets/social-preview.png` is really 1280×640.
- **Missing:** `og:site_name`, `og:locale`, `og:image:type`, `twitter:title`,
  `twitter:description`, `<link rel="apple-touch-icon">`, and any `application/ld+json`
  (**`0` occurrences** in the live document).
- `assets/credits.html` has its own `<title>` but **no** meta description and **no** canonical.

### 1.4 Content and structure

| Signal | Live value |
| --- | --- |
| `<html lang>` | `en` |
| Headings | 1 × `h1` — “Your DOOM. A whole new world.”; 3 × `h2`; 8 × `h3`; no `h4` |
| Visible words | **759** |
| Keyword counts (case-insensitive) | doom 24, flatpak 10, steam deck 8, mapset 6, dsda 5, wad 2, iwad 2, “source port” 0, gzdoom 0, prboom 0 |
| Images | 8 `<img>`, **0** missing `alt` |
| Links | 32 total: 20 external, 5 in-page anchors, remainder relative assets |
| Landmarks | `header`/`nav`/`main`/`section`/`article`/`figure`/`figcaption`/`footer`, skip link, `aria-labelledby` per section |
| Outbound link health | all 20 external URLs return `200` (following redirects) |
| JS-dependence | none for content: all copy is in the HTML; only the donation widget is scripted, with a `<noscript>` fallback link |

Weaknesses: the `h1` contains no product name, no “Steam Deck” and no topic term, so it does not
match the title; the page never states in one plain sentence *what the software is, who it is
for, what it costs and what it requires*; there is no requirements/FAQ/technical section, so the
only fetchable pages about the project are 759 words here plus a credentials page.

### 1.5 Performance and accessibility (Lighthouse 12.8.2, mobile emulation, headless)

| Category | Score | Metric | Value |
| --- | --- | --- | --- |
| Performance | 70 | FCP | 3.5 s |
| Accessibility | 100 | LCP | 5.4 s |
| SEO | 100 | TBT / CLS | 0 ms / 0 |
| — | — | Speed Index / TTI | 5.1 s / 5.4 s |
| — | — | Total transfer | **2 871 KiB** |

| Audit | Finding |
| --- | --- |
| `modern-image-formats` | **2 119 KiB** estimated savings; six PNGs, largest `hero.png` 1 017 287 B, `discover.png` 529 775 B |
| `uses-responsive-images` | **1 315 KiB** estimated savings (1280×800 screenshots sent at full size) |
| `render-blocking-resources` | **909 ms** from `https://fonts.googleapis.com/css?family=Cookie&display=swap`, pulled in by the Buy Me a Coffee widget |
| `third-party-summary` | `buymeacoffee.com` 166 ms main-thread, 117 ms blocking; Google Fonts 17.9 ms |
| `uses-rel-preconnect` | ~400 ms available for `fonts.googleapis.com` / `fonts.gstatic.com` |
| `uses-text-compression`, `font-display`, `unused-css-rules`, `color-contrast`, `image-alt`, `link-name`, `heading-order`, `meta-description`, `crawlable-anchors`, `is-crawlable`, `link-text`, `canonical` | pass |

Lighthouse's SEO category passing is not a quality verdict — it only checks that a canonical
exists, not where it points. Accessibility 100 is automated coverage only; it does not replace
manual checks (focus order in the `<details>` disclosure, contrast on the hero art).

### 1.6 Publication and install path (why the page is stale)

| URL | Status today |
| --- | --- |
| `/io.github.vittau.Doom-Deck-Launcher.flatpakref` (linked from the live page) | `200` |
| `/com.doomdecklauncher.DoomDeckLauncher.flatpakref` (what source now links) | **`404`** |
| `/ddl.flatpakrepo` | `200` (old identity) |
| `gh-pages` root | contains `CNAME` = `doomdecklauncher.com`, `.nojekyll`, the **old** descriptor, and no `robots.txt`/`sitemap.xml` |

Running the page-only publication guard against the current `gh-pages` tree is decisive:

```console
$ python3 scripts/flatpak/render-site.py --check-publication --output pub \
    --app-id com.doomdecklauncher.DoomDeckLauncher --site-base https://doomdecklauncher.com
exit=1
$ python3 scripts/flatpak/render-site.py --check-publication --output pub \
    --app-id io.github.vittau.Doom-Deck-Launcher --site-base https://www.vitormach.dev/Doom-Deck-Launcher
exit=0
```

`Project website` (`project-site.yml`) gates every step — render, push, Pages build, purge,
verification — on that guard. So a page-only refresh **cannot** correct the canonical today: the
new signed channel must be published first (the coordinator reports a signed tag is imminent).
The full publisher (`flatpak-remote.yml`) regenerates the page in the same run, so there should be
no window where the page links a descriptor that does not exist.

Re-checked at **23:33 UTC** while finishing this report: still `18 064` bytes, still the old
canonical, new descriptor still `404`. Every live observation in §1 therefore describes one
consistent pre-publication state.

## 2. Source state on this branch

Merged `verify/071-073` and then `origin/main` (`7772da8`, “Clarify current domain and verified
release checkpoint”) into `vittau/seo-audit`. Rendering the current source into scratch and
diffing against the live bytes shows the live page is **exactly** the pre-migration render:

```console
$ python3 scripts/flatpak/render-site.py --output ./render --version 0.0.10 --date 2026-10-20 \
    --app-id com.doomdecklauncher.DoomDeckLauncher --site-base https://doomdecklauncher.com
$ diff <(curl -sS https://doomdecklauncher.com/) render/index.html
<     <link rel="canonical" href="https://www.vitormach.dev/Doom-Deck-Launcher/">
>     <link rel="canonical" href="https://doomdecklauncher.com/">
<               href="https://www.vitormach.dev/Doom-Deck-Launcher/io.github.vittau.Doom-Deck-Launcher.flatpakref"
>               href="https://doomdecklauncher.com/com.doomdecklauncher.DoomDeckLauncher.flatpakref"
<                 <code>run io.github.vittau.Doom-Deck-Launcher</code>
>                 <code>run com.doomdecklauncher.DoomDeckLauncher</code>
…
```

Already correct in source (`packaging/flatpak/site/index.html`, `scripts/flatpak/render-site.py`,
`.github/workflows/project-site.yml`, `.github/workflows/flatpak-remote.yml`): apex `SITE_BASE`,
apex canonical/`og:url`/`og:image`/brand links, new descriptor name, `CNAME` written on both
publication paths. Still absent **in source too**: robots, sitemap, structured data,
`og:site_name`, 404 page, image format/responsive variants, and on-site help content.

Two source details matter for any proposal that adds root files:

- `project-site.yml` line 76 allows only `index.html|assets/*|CNAME` to change in a page-only run.
- `flatpak-remote.yml` lines 330–332 delete every root file except `index.html`, `.nojekyll`,
  `CNAME`, `${APP_ID}.flatpakref` and `ddl.flatpakrepo`.

A new `robots.txt`/`sitemap.xml` would therefore be **rejected** in the first path and
**deleted** in the second unless both allowlists are updated in the same change.

## 3. What already works

1. Server-rendered, JavaScript-free content: every crawler (including AI fetchers that do not
   execute JS) receives all 759 words, the install steps and the disclaimer in the HTML.
2. URL consolidation: `www`, the GitHub Pages project URL and the retired
   `vitormach.dev/Doom-Deck-Launcher/` path all `301` to the apex, matching the single canonical
   intent of spec 079 / D-039.
3. No cloaking or bot-specific blocking: the page body is byte-identical (`sha256` prefix
   `de2cc989eb52809d`, 18 064 B) for browser, `Googlebot`, `GPTBot`, `OAI-SearchBot`,
   `ClaudeBot`, `PerplexityBot` and `curl` user agents.
4. Accurate, honest metadata where it exists: title and description are unique, specific and
   within typical display lengths; `og:image` dimensions match the real file; alt text on all 8
   images; semantic landmarks and a skip link; `lang="en"`.
5. Trust/accuracy signals that matter for AI answers: explicit “experimental/pre-alpha”, “not a
   Flathub listing”, “never bundles or downloads commercial base games”, GPL-3.0-or-later, and a
   full trademark/non-affiliation disclaimer.
6. Healthy internal/external links (all 20 outbound URLs `200`) and a valid TLS certificate for
   apex and `www`.
7. GitHub is already a second discovery surface with a real description, README screenshots and
   a website link in the README.

## 4. Findings, prioritized

Labels: **Confirmed** = observed here; **Proposed** = recommendation; **Owner** = cannot be done
from this repository.

### P0 — correctness, ship with the imminent signed publication

**F1. The live page still declares the retired domain and application ID.**
Canonical, `og:url`, `og:image`, brand link and both install buttons point to
`www.vitormach.dev/Doom-Deck-Launcher/` and `io.github.vittau.Doom-Deck-Launcher`.
*Cause:* the page-only workflow defers until the new signed channel exists (§1.6). *Fix:* already
in source; it lands with the publication. *Validation:*

```sh
curl -sS https://doomdecklauncher.com/ | grep -o 'rel="canonical" href="[^"]*"'
curl -sS https://doomdecklauncher.com/ | grep -c 'vitormach.dev\|io\.github\.vittau\.Doom-Deck-Launcher'
# expect: canonical = https://doomdecklauncher.com/  and the count = 0
curl -sS -o /dev/null -w '%{http_code}\n' https://doomdecklauncher.com/com.doomdecklauncher.DoomDeckLauncher.flatpakref
```

**F2. Plain HTTP is served, not redirected.** `http://doomdecklauncher.com/` returns `200` with
the full page (`server: GitHub.com`, no HSTS), and `http://www…` redirects to the **http** apex.
Two schemes of every URL are crawlable. *Fix (Owner):* enable `Enforce HTTPS` in the repository's
Pages settings; GitHub then issues the `301` to `https` and sends HSTS. *Validation:* the two
`curl -I http://…` calls above must return `301` with a `Location:` to `https://`.

### P1 — cheap, high-leverage source changes (need approval; no code written)

**F3. No `robots.txt`.** Not a blocker (RFC 9309: 4xx = allow), but it is the only place to
declare sitemap location and to state the intended AI-crawler policy explicitly instead of by
omission. *Proposed:* generate it in `scripts/flatpak/render-site.py` (values from `SITE_BASE`)
and allow the file in `.github/workflows/project-site.yml:76` and
`.github/workflows/flatpak-remote.yml:330-332`. A minimal, policy-neutral body:

```robots
User-agent: *
Allow: /
Disallow: /repo/
Sitemap: https://doomdecklauncher.com/sitemap.xml
```

`Disallow: /repo/` keeps crawlers out of the OSTree object tree (never user-facing) without
touching any AI or search agent. Whether to *also* name `GPTBot`/`ClaudeBot` explicitly is a
policy question, not an SEO requirement — see §6.

**F4. No sitemap.** With one real page plus the credits page, a sitemap is small but genuinely
useful for a brand-new domain with almost no inbound links, because it is the one discovery
mechanism you control. Google states it improves discovery but **does not guarantee** crawl or
index. *Proposed:* render `sitemap.xml` from `render-site.py` with the two URLs
(`/`, `/assets/credits.html`) and `lastmod` = publication date; reference it from `robots.txt` and
submit it in Search Console/Bing Webmaster Tools (**Owner**). *Validation:* `curl` both files,
then `xmllint --noout` on the fetched XML.

**F5. No structured data and no site-name signal.** Zero `application/ld+json`. Google's
site-names documentation says `WebSite` structured data is the strongest preference it accepts,
with `og:site_name` and the title as secondary inputs — the page currently has none of the
structured form and no `og:site_name`. *Proposed (accuracy-first, no invented facts):*

- `WebSite`: `name` “DoomDeckLauncher”, `alternateName` “DDL”, `url` apex.
- `Organization` with `name`, `url`, `sameAs` → GitHub repository, `logo` → `/assets/icon.png`.
- `SoftwareApplication`: `name`, `operatingSystem` “Linux”, `applicationCategory`
  “GameApplication”, `offers` `{ "@type": "Offer", "price": 0, "priceCurrency": "USD" }`,
  `license` GPL-3.0-or-later URL, `downloadUrl` → the `.flatpakref`, `softwareRequirements` →
  “legally owned DOOM IWAD”.
  **Accuracy caveat:** Google's Software-app rich result requires `name`, `offers.price` **and**
  a `rating`/`review` or `aggregateRating`; the project has no genuine ratings, so this markup
  would be valid and useful for entity understanding but **must not** be expected to produce a
  rich result. Do not add a fabricated rating to chase one.
- Add `og:site_name`, `og:locale`, `og:image:type`, `twitter:title`, `twitter:description`.

*Validation:* parse the JSON-LD in `packaging/flatpak/site/test_render.py` (assert the fields and
that `price` is `0`), then the Rich Results Test manually once, and confirm every claim matches
visible page text.

**F6. Page weight is dominated by PNG screenshots and a third-party font request.**
2.87 MB transfer, LCP 5.4 s, 2.1 MB of image-format savings and 1.3 MB of responsive savings
available; the donation widget drags in a render-blocking `fonts.googleapis.com` stylesheet
(909 ms) that the project does not otherwise need — it already self-hosts Barlow and Pirata One
with `font-display: swap`. *Proposed (in order of value/effort):* (a) replace the inline Buy Me a
Coffee `<script>` in `packaging/flatpak/site/index.html` (nav, lines ~51–59) with a static styled
link to `https://buymeacoffee.com/vittau`, keeping the `<noscript>` path as the only path — removes
both third-party origins and the render-blocking font request; (b) commit WebP derivatives of
`docs/screenshots/*.png` and `docs/design/logo/steam/hero.png` and serve them via `<picture>` /
`srcset` with the PNGs as fallback, adding `fetchpriority="high"` to the hero; (c) `preconnect`
only if third parties stay. *Validation:* re-run the same Lighthouse invocation and compare LCP,
transfer size and `modern-image-formats`; assert new asset names/hashes in `test_render.py` so a
missing derivative fails CI.

**F7. Thin, entity-poor copy.** The page describes the *experience* well but never states the
product in one citable sentence, and the `h1` contains no product or topic term. For ordinary SEO
and for retrieval-augmented answers, that sentence and a requirements block are the highest-value
content addition. *Proposed:* keep the artwork, and make the `h1` carry the product name (visible
change, needs owner/design approval), e.g. “DoomDeckLauncher — community DOOM maps on Steam
Deck”; add a short “What is DoomDeckLauncher?” paragraph and a “Requirements” list (Steam Deck or
Linux desktop, Flatpak, legally owned DOOM IWADs, DSDA-Doom downloaded in-app, experimental
channel); add 4–6 question-shaped `h2`/`h3` sections answering real owner questions (free? open
source? includes DOOM? Windows/macOS? which source port? GZDoom compatibility? how do updates
work?). Do **not** add `FAQPage` markup: since August 2023 Google shows FAQ rich results only for
well-known authoritative government and health sites, and `QAPage` is for user-generated Q&A.

**F8. `assets/credits.html` is under-described.** Add a meta description, a self-referencing
canonical, and keep the existing back-link (already `../index.html`); consider
`<meta name="robots" content="noindex, follow">` if the owner prefers not to compete with the
landing page for “DoomDeckLauncher credits”.

**F9. No branded 404.** Missing paths return GitHub's generic 404. A small `404.html` with the
header and a link home costs nothing and keeps stray crawler hits on-site.

### P2 — discovery beyond the page (mostly Owner)

**F10. There are no inbound citations yet.** The repository was created `2026-10-14`, the domain
and certificate are hours old, the GitHub repository shows `homepage: null`, `topics: []`,
0 stars/forks, and a probe of the connected search integration for the exact term
`DoomDeckLauncher` returned no results. Without external mentions, no engine — classic or
AI — has a reason to discover, let alone cite, the site. *Proposed:* set the repository **About**
website to `https://doomdecklauncher.com/` and add topics (`steam-deck`, `doom`, `flatpak`, `wad`,
`launcher`, `rust`, `slint`, `dsda-doom`); then earn real citations from the DOOM/Steam Deck
community (Doomworld thread, r/SteamDeck, DSDA-Doom discussions, UZDeck/GZDeck comparison
mentions, SteamGridDB, Flatpak community spaces). This is outreach, not markup — it cannot be
automated from this repository, and it is the single biggest lever.

**F11. Consoles are unverified (Owner).** No Search Console property, no Bing Webmaster Tools
site, hence no coverage data and no way to request indexing of the new domain. See §7.

**F12. `llms.txt` is not a requirement.** None of the crawler documents read for this audit
(OpenAI, Anthropic, Perplexity, Google) mentions it, and no vendor documents a ranking or citation
benefit. Adding one is optional and harmless at most; it is not on the critical path and this
report does not recommend it as anything more than optional.

## 5. What each AI surface actually needs

Every claim below is from the vendor's own documentation read on 2026-10-21 (links in §12). The
distinction that matters: **search crawling**, **model training**, and **user-initiated fetching**
are separate permissions, and `robots.txt` is the control for the first two.

| Surface | Index/search agent | Training agent | User-fetch agent | What this site needs |
| --- | --- | --- | --- | --- |
| ChatGPT search | `OAI-SearchBot` — “Sites that are opted out of OAI-SearchBot will not be shown in ChatGPT search answers, though can still appear as navigational links” | `GPTBot` | `ChatGPT-User` (not an automatic crawler; robots may not apply) | Allow `OAI-SearchBot`; already implicitly allowed (no robots.txt) |
| Claude | `Claude-SearchBot` — disabling it “prevents our system from indexing your content for search optimization” | `ClaudeBot` | `Claude-User` — disabling “prevents our system from retrieving your content in response to a user query” | Allow all three (or decide per policy); Anthropic also honors `Crawl-delay` |
| Perplexity | `PerplexityBot` — “designed to surface and link websites in search results”, not training | — | `Perplexity-User` — “generally ignores robots.txt rules” because the user requested it | Allow `PerplexityBot`; note user fetches bypass robots by design |
| Bing / Copilot | `bingbot` → Bing index | — | — | Indexing via Bing Webmaster Tools + IndexNow; sitemap |
| Google Search + AI features | `Googlebot` | `Google-Extended` — a robots **token only**, “doesn’t have a separate HTTP request user agent string” | — | Indexable page + Search Console; `Google-Extended` only if training must be excluded |

Consequences for DDL, in plain terms:

- **Nothing is currently blocked for any of them** — no `robots.txt` at all, no `noindex`, no WAF,
  and byte-identical responses to every agent tested. That is the right starting state.
- **Do not “optimise” for AI by adding a robots policy unless the owner wants one.** The default
  (allow) is what maximizes presence in ChatGPT search, Claude search, Perplexity and Google AI
  features. If DDL ever wants training excluded but search allowed, that is expressible
  (disallow `GPTBot`/`ClaudeBot`/`Google-Extended` while allowing the search agents) — a decision
  for the owner, not an SEO defect. GPL-3.0-or-later already invites reuse.
- **AI crawlers do not need JavaScript here**, which is a real advantage over most launcher
  landing pages.
- **“GEO/AEO” claims should be treated with suspicion.** Google's own guide says optimizing for
  generative AI is still SEO and warns that creating content per fan-out query violates its
  scaled-content policy; it also states plainly that meeting every requirement does not mean
  Google will crawl, index or serve the content.

## 6. Search-indexing requirements, and what is provable from here

**Google (Confirmed from primary docs):** a page must be indexed and snippet-eligible to appear
in generative AI features; on top of the Search technical requirements, Google's current guide
states that “a site must be included in Search generative AI features in Search Console to be
eligible for display in generative AI features on Google Search” — the owner should inspect that
Search Console setting. Indexing and serving are not guaranteed. Canonicalization prefers
redirects over `rel="canonical"`. Sitemaps help discovery but do not guarantee crawling.

**Bing (Confirmed from primary docs):** IndexNow is the documented push mechanism and is
supported by Microsoft Bing (plus Naver, Seznam.cz, Yandex, Yep); it requires a key and an
HTTP ping per changed URL, and Google is not a participant. Conventional discovery still runs
through the Bing index and Bing Webmaster Tools, and a sitemap remains the baseline. Copilot's web
grounding is served from Microsoft's search stack, so Bing indexation is the actionable lever —
**Proposed inference**, not a vendor statement about Copilot specifically.

**Sitemaps and robots:** both are discovery/control aids, never ranking controls; the honest
expectation is “faster, more predictable discovery”, not visibility.

## 7. Owner-only actions (cannot be done from this repository)

1. **Pages → Enforce HTTPS** — observed plain-HTTP `200` (§1.1).
2. **Search Console**: add a *domain* property for `doomdecklauncher.com` (DNS verification) and
   an URL-prefix property for the apex; submit `sitemap.xml` once it exists; use URL Inspection
   on `https://doomdecklauncher.com/` after publication; review the generative-AI-features
   setting named in Google's guide.
3. **Bing Webmaster Tools**: add the site, submit the sitemap, request indexing. Optionally adopt
   IndexNow (key file at the publication root — a small source change, and it intersects the two
   allowlists in §2).
4. **GitHub repository About**: set the website URL and topics (F10).
5. **Community citations**: Doomworld / r/SteamDeck / DSDA-Doom / UZDeck-GZDeck threads (F10).
   This is the only route to the inbound links that both classic and AI search rely on.
6. **Decision** on whether to keep the OAI-SearchBot/ClaudeBot/PerplexityBot/GPTBot policy open
   (recommended default) and whether to keep the retired descriptor alive for cached pages
   (open question §9).

## 8. Limitations

- No access to Search Console, Bing Webmaster Tools, Cloudflare analytics or GitHub traffic:
  **index coverage, crawl rate and AI citations are unknown and unmeasured.**
- Anonymous SERP probes were inconclusive and are not used as evidence: DuckDuckGo's HTML
  endpoint returned `202` (bot challenge, 0 results); Bing's `site:` and quoted-brand queries
  returned unrelated localized results; the connected search integration returned no results for
  `DoomDeckLauncher`. None of these can distinguish “not indexed” from “probe blocked”.
- PageSpeed Insights API refused anonymous requests with a daily-quota `429`; the performance
  numbers come from a single local Lighthouse 12.8.2 mobile run (lab data, not field/CrUX data —
  the domain is far too new to have field data).
- Lighthouse accessibility 100 means automated checks pass, not that the page is accessible.
- The report describes one host, one network vantage point and one moment; DNS/certificate state
  changed within hours of the audit and will change again with the imminent publication.
- Behaviour of the actual signed installation, Discover, Gaming Mode and the launcher itself is
  out of scope for this audit (**Needs Deck Validation** elsewhere).
- Cloudflare purge/proxy behaviour was not tested (the apex is DNS-only from this vantage point),
  and no credential, console or API mutation was attempted.

## 9. Open questions

1. **H1 copy:** is changing the visible `h1` to include “DoomDeckLauncher” (and keeping the
   artwork line as an eyebrow) acceptable design-wise? It is the cheapest on-page relevance fix.
2. **Donation widget:** is a static “Donate” link acceptable instead of the Buy Me a Coffee
   script, dropping both third-party origins and the render-blocking Google Font? (Current
   `test_official_donation_embed_matches_owner_configuration` pins the embed — that test would
   change too.)
3. **Retired install URL after publication:** the full publisher deletes
   `io.github.vittau.Doom-Deck-Launcher.flatpakref` from the root, while browsers, social caches
   and search indexes may reference it for days (Cloudflare cache is only 600 s, but social cards
   are not). Owner already waived migration; should a temporary shim that installs the **new**
   channel be served at the old path, or is a clean 404 preferred?
4. **Content ownership:** should the Steam Deck guide (currently `docs/steam-deck-guide.md` on
   GitHub) get a canonical on-site page, or stay on GitHub with the site linking out? Publishing
   both in full would split signals; the recommendation is one canonical home, decided by the
   owner.
5. **Training policy:** keep the current “everything allowed” default, or exclude
   training crawlers while keeping search agents?

## 10. Sources checked (2026-10-21)

| Source | Used for |
| --- | --- |
| [OpenAI — Overview of OpenAI Crawlers](https://platform.openai.com/docs/bots) | `OAI-SearchBot` (search), `GPTBot` (training), `ChatGPT-User` (user actions; robots may not apply); search opt-out wording |
| [Anthropic — Does Anthropic crawl data from the web…](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler) | `ClaudeBot` / `Claude-SearchBot` / `Claude-User` roles, robots + `Crawl-delay` |
| [Perplexity — Perplexity Crawlers](https://docs.perplexity.ai/guides/bots) | `PerplexityBot` (search) vs `Perplexity-User` (generally ignores robots), WAF/IP guidance |
| [Google — Optimizing for generative AI features](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide) | “still SEO”, RAG/query fan-out, “indexed and snippet-eligible”, Search Console generative-AI eligibility, no indexing guarantee |
| [Google — Intro to robots.txt](https://developers.google.com/search/docs/crawling-indexing/robots/intro) | robots.txt is crawl management, not removal |
| [RFC 9309](https://www.rfc-editor.org/rfc/rfc9309.html) §2.3.1.3 | 4xx robots.txt ⇒ crawler MAY access any resource |
| [Google — Sitemaps overview](https://developers.google.com/search/docs/crawling-indexing/sitemaps/overview) | discovery aid; no crawl/index guarantee |
| [Google — Consolidate duplicate URLs](https://developers.google.com/search/docs/crawling-indexing/consolidate-duplicate-urls) | redirects = strong signal, `rel=canonical` = hint |
| [Google — Software app structured data](https://developers.google.com/search/docs/appearance/structured-data/software-app) | required `name`, `offers.price`, rating/review; co-type `VideoGame`; no guaranteed display |
| [Google — Site names](https://developers.google.com/search/docs/appearance/site-names) | `WebSite` structured data is the strongest preference; `og:site_name`/title secondary |
| [Google — Google crawlers list](https://developers.google.com/search/docs/crawling-indexing/google-common-crawlers) | `Google-Extended` is a robots token with no separate HTTP user agent |
| [Google — Changes to HowTo and FAQ rich results](https://developers.google.com/search/blog/2023/08/howto-faq-changes) | FAQ rich results limited to well-known government/health sites |
| [IndexNow documentation](https://www.indexnow.org/documentation) / [home](https://www.indexnow.org/) | key + per-URL ping; supported by Bing, Naver, Seznam.cz, Yandex, Yep |
| [GitHub Pages — Managing a custom domain](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site) | apex/`www` records, Enforce HTTPS, propagation time (already cited in report 079) |

## 11. Appendix — reproducible evidence

```sh
# Headers, redirects, scheme handling
curl -sS -o /dev/null -D - https://doomdecklauncher.com/
curl -sS -o /dev/null -D - https://www.doomdecklauncher.com/
curl -sS -o /dev/null -D - --resolve doomdecklauncher.com:80:185.199.108.153 \
  http://doomdecklauncher.com/            # 200, no Location, no HSTS

# Discovery files
for p in robots.txt sitemap.xml favicon.ico llms.txt site.webmanifest \
         com.doomdecklauncher.DoomDeckLauncher.flatpakref \
         io.github.vittau.Doom-Deck-Launcher.flatpakref; do
  curl -sS -o /dev/null -w "%{http_code} $p\n" "https://doomdecklauncher.com/$p"
done

# No cloaking: identical bytes for every agent
for ua in "Googlebot/2.1" "GPTBot/1.4" "OAI-SearchBot/1.4" "ClaudeBot/1.0" "PerplexityBot/1.0"; do
  curl -sS -A "Mozilla/5.0 (compatible; $ua)" https://doomdecklauncher.com/ | shasum -a 256
done

# DNS / TLS
dig @1.1.1.1 doomdecklauncher.com A +short
dig @luke.ns.cloudflare.com doomdecklauncher.com AAAA +short
echo | openssl s_client -connect doomdecklauncher.com:443 -servername doomdecklauncher.com 2>/dev/null \
  | openssl x509 -noout -subject -dates -ext subjectAltName

# Publication gate (against a checkout of origin/gh-pages)
git archive origin/gh-pages | tar -x -C pub
python3 scripts/flatpak/render-site.py --check-publication --output pub \
  --app-id com.doomdecklauncher.DoomDeckLauncher --site-base https://doomdecklauncher.com   # exit 1 today

# Performance / accessibility (headless, no visible window)
npx --yes lighthouse@12 https://doomdecklauncher.com/ \
  --only-categories=performance,accessibility,seo \
  --output=json --output-path=lh.json \
  --chrome-flags="--headless=new --no-sandbox --disable-gpu" --quiet
```

## 12. Deliverables and next step

Only `docs/reports/080-seo-and-ai-discovery-audit.md` was added, committed on
`vittau/seo-audit`. No source, workflow, DNS, Pages, credential or console change was made, and
nothing was pushed, tagged or published. The recommended next step is **not** a fix: it is to let
the imminent signed publication refresh the page (which resolves F1 by itself), confirm the
canonical with the `curl` in §4 F1, and then have the coordinator decide which P1 items are worth
a follow-up task — robots+sitemap and the donation-widget/image-format performance work being the
two best value-per-risk candidates.

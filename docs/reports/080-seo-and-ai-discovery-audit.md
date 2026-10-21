# 080 — SEO and AI-search discovery audit of `doomdecklauncher.com`

**Task:** research audit of conventional search and AI-search discoverability of the public
project site, with evidence, prioritized proposals and limitations. **No code, publication,
DNS, Pages, credential or console change was made.** This report is the only tracked change.

Evidence was collected on **2026-10-21 between 23:26 and 23:38 UTC** (20:26–20:38
America/Sao_Paulo) from this macOS arm64 host and the public internet. The signed publication
happened inside that window, so the report keeps the pre- and post-publication evidence separate
(§1). Live responses were treated as untrusted data: they were fetched, parsed as text and never
executed.

## Outcome

The site is technically clean and easy to crawl: static HTML with all text in the response,
working HTTPS on both hosts with a valid certificate (plain HTTP redirects; §1.1), `www` and the
retired project URL redirected to the apex, healthy outbound links, accessibility 100 in
Lighthouse, and no `User-Agent`-based cloaking at the origin. **The one dominant problem fixed
itself during the audit:** the signed publication (≈23:35 UTC, §1.6) replaced the pre-migration
render, so the live canonical, `og:url`, `og:image`, brand and install links are now on
`https://doomdecklauncher.com/` and the retired descriptor is gone. What remains is not a
crawling defect: no `robots.txt`, no sitemap, no structured data or site-name signal, ~751 words
of indexable copy, PNG-dominated page weight, and consoles/citations this audit cannot verify.

## Method

- Live HTTP: `curl` with explicit user agents (browser, `Googlebot`, `GPTBot`, `OAI-SearchBot`,
  `ClaudeBot`, `PerplexityBot`, `curl`) and `--resolve` where the local stub resolver had a stale
  negative cache; headers, byte bodies and hashes compared. User-agent strings were only ever used
  as request headers — that is origin-level evidence, not crawler-IP or WAF verification (§8).
- DNS/TLS: `dig` against the system resolver, `1.1.1.1`, `8.8.8.8` and the authoritative
  Cloudflare nameservers; `openssl s_client` for the certificate.
- Source: branch `vittau/seo-audit` merged with `verify/071-073` and then `origin/main`
  (`7772da8`). The renderer was executed locally into a scratch directory and diffed against the
  live bytes.
- Performance/accessibility: Lighthouse 12.8.2 run headlessly twice — once before and once after
  the publication — against the live URL (no visible window, no screenshot). Google PageSpeed
  Insights API was attempted first and refused with
  `429 ... Quota exceeded ... Queries per day` for anonymous requests, so its scores are **not**
  reported here.
- Crawler and search requirements: vendor primary documentation only (sources in §10), read on the
  same date.

**What this audit cannot do:** it has no access to Search Console or Bing Webmaster Tools data,
Cloudflare analytics or server logs (Search Console is configured by the owner — §4 F11 — but its
private data is not available here), so it cannot state index coverage, impressions, crawl
frequency or AI-answer citations. Nothing here promises indexing, ranking or citation.

## 1. Live snapshot

### State change during the audit (read this first)

The audit started **before** the signed publication and finished **after** it, so two live states
exist in the evidence:

| Time (UTC) | Live state |
| --- | --- |
| 23:26–23:32 | Pre-publication render: `18 064`-byte page, canonical/`og:url`/`og:image` and both install links on `www.vitormach.dev/Doom-Deck-Launcher/` with the retired `io.github.vittau…` ID, new descriptor `404`; the page-only refresh guard rejected a page update (§1.6) |
| 23:33–23:34 | `Enforce HTTPS` visible at the edge: `http://` → `301` (§1.1) |
| **23:36** | **Post-publication**: `17 954`-byte page (v0.0.11, published 2026-10-21), canonical and every link on `https://doomdecklauncher.com/`, new descriptor `200`, retired descriptor `404` |

§1.1–§1.5 describe the **current post-publication** state; §1.6 keeps the pre-publication evidence
and the guard transcript that explains why the page was stale for the first part of the audit.
Robots, sitemap and structured data were re-checked after publication and are still absent.

### 1.1 Hosting, DNS, TLS, redirects

| Check | Observed |
| --- | --- |
| `https://doomdecklauncher.com/` | `200`, `server: GitHub.com`, `content-type: text/html; charset=utf-8`, `content-length: 17954` (post-publication), `cache-control: max-age=600`, gzip with `Accept-Encoding: br, gzip` |
| `https://www.doomdecklauncher.com/` | `301` → `https://doomdecklauncher.com/` |
| `https://vittau.github.io/Doom-Deck-Launcher/` | `301` → `https://doomdecklauncher.com/` |
| `https://www.vitormach.dev/Doom-Deck-Launcher/` | `301` → `https://doomdecklauncher.com/` |
| `https://vitormach.dev/Doom-Deck-Launcher/` | `301` → `https://www.vitormach.dev/Doom-Deck-Launcher/` → `301` → apex (two hops) |
| `http://doomdecklauncher.com/` | `301` → `Location: https://doomdecklauncher.com/` (re-checked 23:34 UTC from all four Pages `A` addresses) |
| `http://www.doomdecklauncher.com/` | `301` → `http://doomdecklauncher.com/`, then `301` → `https://doomdecklauncher.com/` (two hops; the first names the plain-HTTP apex, which immediately upgrades) |
| `https://doomdecklauncher.com` (no slash) | `200` (both variants served; canonical declares the slash form) |
| DNS | apex `A` = `185.199.108–111.153` and `AAAA` = `2606:50c0:8000::153`–`…8003::153`; `www` = `CNAME vittau.github.io.`; NS `luke.ns.cloudflare.com`, `zara.ns.cloudflare.com` |
| TLS | Let's Encrypt `CN=doomdecklauncher.com`, SAN `doomdecklauncher.com` + `www.doomdecklauncher.com`, valid `2026-10-21 22:22:42` → `2027-01-19` |
| Cloudflare | apex responses carry **no** `cf-ray`/`cf-cache-status` (DNS-only in front of Pages); the legacy `vitormach.dev` host does carry them |

The redirect topology is correct and consolidates every retired URL on the apex. Two observations
about **when** it is correct:

- **Snapshot timing and edge discrepancy (corrected).** This audit first sampled
  `http://doomdecklauncher.com/` at **23:27:55 UTC** and again with an explicit `--resolve` to
  `185.199.108.153` at **23:28:25 UTC**, and both returned `200` with the full HTML and no
  redirect, from the `brazilsouth` edge. An independent coordinator check at **23:33:42 UTC** and
  this audit's re-check at **23:34:00 UTC** both get `301` → `https://doomdecklauncher.com/` from
  **all four** `185.199.108–111.153` addresses. The Pages API (`https_enforced: true`, certificate
  approved) confirms the intended state. So the earlier `200` was a per-edge/cached response
  during propagation of the Pages setting, **not** an owner-side misconfiguration; the `301` is
  the current, correct behaviour. The lesson recorded in §8 is that scheme/edge samples taken
  minutes apart can disagree.
- **No HSTS header observed** at 23:34 UTC on the HTTPS response. `Enforce HTTPS` produces the
  redirect; `Strict-Transport-Security` is not sent by GitHub Pages here, so no HSTS claim is made
  in this report. The redirect alone already removes the duplicate-scheme crawl path.

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
excluded from indexing today. A missing `robots.txt` is **not a crawl or indexing blocker**: per
RFC 9309 §2.3.1.3 an unavailable (4xx) `robots.txt` means a crawler **MAY** access any resource,
and Google's own introduction says the file is for crawl management, not removal. Its absence
only means that no sitemap is advertised and no crawler policy is stated explicitly.

Re-checked at **23:36 UTC after the signed publication**: `robots.txt` and `sitemap.xml` are still
`404` (the publication only refreshed the page, descriptor and repository), while the new
descriptor is now `200` (§1.6).

### 1.3 Head metadata (live after publication, verbatim)

```html
<title>DoomDeckLauncher — Community DOOM maps on Steam Deck</title>            <!-- 52 chars -->
<meta name="description" content="Own DOOM on Steam? Discover, install and play community
  mapsets on Steam Deck with DoomDeckLauncher. Download the experimental signed Flatpak.">
                                                                                <!-- 142 chars -->
<link rel="canonical" href="https://doomdecklauncher.com/">
<meta property="og:type" content="website">
<meta property="og:title" content="DoomDeckLauncher — Your next DOOM adventure">
<meta property="og:description" content="Your DOOM library. A world of community maps. A
  controller-first launcher for Steam Deck.">
<meta property="og:url" content="https://doomdecklauncher.com/">
<meta property="og:image" content="https://doomdecklauncher.com/assets/social-preview.png">
<meta property="og:image:width" content="1280">
<meta property="og:image:height" content="640">
<meta property="og:image:alt" content="DoomDeckLauncher’s original Rose Compass identity in an Ember Cathedral interior">
<meta name="twitter:card" content="summary_large_image">
<meta name="theme-color" content="#120c0d">
```

- Canonical, `og:url` and `og:image` are now exactly the serving URL and its assets; the page
  contains **zero** occurrences of the retired host or application ID (pre-publication values are
  preserved in §1.6).
- `og:image` is accurate: the served `assets/social-preview.png` is really 1280×640.
- **Still missing:** `og:site_name`, `og:locale`, `og:image:type`, `twitter:title`,
  `twitter:description`, `<link rel="apple-touch-icon">`, and any `application/ld+json`
  (**`0` occurrences** in the live document).
- `assets/credits.html` still has its own `<title>` but **no** meta description and **no**
  canonical.

### 1.4 Content and structure

| Signal | Live value |
| --- | --- |
| `<html lang>` | `en` |
| Headings | 1 × `h1` — “Your DOOM. A whole new world.”; 3 × `h2`; 8 × `h3`; no `h4` |
| Visible words | **751** (759 pre-publication) |
| Keyword counts (case-insensitive, post-publication) | doom 28, flatpak 10, steam deck 8, mapset 6, dsda 5, wad 2, iwad 2, “source port” 0, gzdoom 0, prboom 0 |
| Images | 8 `<img>`, **0** missing `alt` |
| Links | 32 total: 20 absolute (4 apex, 16 third-party), 5 in-page anchors, remainder relative assets |
| Landmarks | `header`/`nav`/`main`/`section`/`article`/`figure`/`figcaption`/`footer`, skip link, `aria-labelledby` per section |
| Outbound link health | every third-party URL returns `200` (following redirects) |
| JS-dependence | none for content: all copy is in the HTML; only the donation widget is scripted, with a `<noscript>` fallback link |

Weaknesses: the `h1` contains no product name, no “Steam Deck” and no topic term, so it does not
match the title; the page never states in one plain sentence *what the software is, who it is
for, what it costs and what it requires*; there is no requirements/FAQ/technical section, so the
only fetchable pages about the project are 751 words here plus a credentials page.

### 1.5 Performance and accessibility (Lighthouse 12.8.2, mobile emulation, headless)

Two runs on the live URL — **23:29:55 UTC** (pre-publication page) and **23:36:37 UTC**
(post-publication page). The assets are nearly identical; the spread between the runs shows how
variable a single public-network lab measurement is, so the timings are reported as a range and
only the weight/format findings are treated as stable.

| Category / metric | Run 1 (23:29:55) | Run 2 (23:36:37) |
| --- | --- | --- |
| Performance / Accessibility / SEO | 70 / 100 / 100 | 71 / 100 / 100 |
| FCP | 3.5 s | 1.1 s |
| LCP | 5.4 s | 10.3 s |
| TBT | 0 ms | 230 ms |
| CLS | 0 | 0.001 |
| Speed Index | 5.1 s | 2.4 s |
| Total transfer | 2 871 KiB | 2 936 KiB |

Both runs identify the same LCP element: the `div.hero-art` CSS background, i.e. `assets/hero.png`
(1 017 287 B served). Stable findings across runs:

| Audit | Finding |
| --- | --- |
| `modern-image-formats` | **2 119 KiB** (run 1) / **2 176 KiB** (run 2) estimated savings across six PNGs |
| `uses-responsive-images` | **1 315 KiB** / **1 364 KiB** estimated savings (1280×800 screenshots sent at full size) |
| Largest assets | `hero.png` 1 017 287 B; `mapset-detail.png` 537 697 B; `discover.png` 534 370 B; `home.png` 276 406 B |
| `render-blocking-resources` | **909 ms** from `https://fonts.googleapis.com/css?family=Cookie&display=swap` (pulled in by the Buy Me a Coffee embed) in run 1; none measured in run 2 with the same requests made |
| `third-party-summary` | `buymeacoffee.com` 117 ms main-thread blocking in run 1, 0 ms in run 2; Google Fonts 14 KB of font fetched in run 2 |
| Passing | `uses-text-compression`, `font-display`, `unused-css-rules`, `color-contrast`, `image-alt`, `link-name`, `heading-order`, `meta-description`, `crawlable-anchors`, `is-crawlable`, `link-text`, `canonical` |

The third-party cost is real but variable: the same requests (`cdnjs.buymeacoffee.com` 6 043 B,
`fonts.googleapis.com` CSS 755 B, a Cookie `.woff2` 14 463 B) appeared in both runs, while
Lighthouse attributed blocking time in only one of them.

Lighthouse's SEO category passing is not a quality verdict — it only checks that a canonical
exists, not where it points. Accessibility 100 is automated coverage only; it does not replace
manual checks (focus order in the `<details>` disclosure, contrast on the hero art).

### 1.6 Publication and install path (why the page was stale, and when it changed)

The audit's first half measured the pre-publication tree; the second half measured the publication
that replaced it. Both are recorded here because the difference explains the whole P0 item.

| URL / artifact | Pre-publication (23:26–23:32 UTC) | Post-publication (23:36 UTC) |
| --- | --- | --- |
| `/io.github.vittau.Doom-Deck-Launcher.flatpakref` | `200` — and it was what the page linked | **`404`** (root cleanup removed it) |
| `/com.doomdecklauncher.DoomDeckLauncher.flatpakref` | `404` | **`200`** (`[Flatpak Ref] Name=com.doomdecklauncher.DoomDeckLauncher`, `Url=https://doomdecklauncher.com/repo/`) |
| `/ddl.flatpakrepo` | `200`, old identity | `200`, `Url`/`Homepage` on the apex |
| `/repo/summary`, `/repo/summary.sig` | `200` | `200` |
| Page | `18 064` B, retired canonical | `17 954` B, v0.0.11 (published 2026-10-21), apex canonical |
| `gh-pages` root | `CNAME`, `.nojekyll`, old descriptor | same plus the new descriptor; still **no** `robots.txt`/`sitemap.xml` |

Before publication, the page-only workflow refused to run. Running its own guard against that
tree is decisive:

```console
$ python3 scripts/flatpak/render-site.py --check-publication --output pub \
    --app-id com.doomdecklauncher.DoomDeckLauncher --site-base https://doomdecklauncher.com
exit=1
$ python3 scripts/flatpak/render-site.py --check-publication --output pub \
    --app-id io.github.vittau.Doom-Deck-Launcher --site-base https://www.vitormach.dev/Doom-Deck-Launcher
exit=0
```

`Project website` (`project-site.yml`) gates every step — render, push, Pages build, purge,
verification — on that guard, so **until the signed channel existed the canonical could not be
fixed by a page-only run**. The full publisher (`flatpak-remote.yml`) regenerated the page in the
same run, which is why the new page, new descriptor and removal of the old descriptor appeared
together at ≈23:35 UTC with no window where the page linked a missing file.

Consequences worth keeping:

- The publication job was the signed repository run `39693631806` (coordinator evidence), and the
  owner's Cloudflare purge for the new `doomdecklauncher.com/` prefix was accepted at 23:35:05 UTC
  — so the new canonical was not served from a stale edge cache; this audit's 23:36 UTC fetch
  confirms it.
- The retired install URL is now a hard `404` while browsers, social caches and search indexes can
  still hold the pre-publication page for a while (open question §9.3).
- The page-only workflow can now refresh content, which is what makes the P1 page changes
  (robots, sitemap, structured data, markup and content) straightforward to publish separately.

## 2. Source state on this branch

Merged `verify/071-073` and then `origin/main` (`7772da8`, “Clarify current domain and verified
release checkpoint”) into `vittau/seo-audit`. Two diffs against the source render bracket the
publication:

- **Before** it, the live page was the pre-migration render, so the diff exposed exactly the
  migration delta:

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

- **After** it (23:36 UTC), the live page and the current source render differ only in the release
  line (`v0.0.11 · Published 2026-10-21` live versus the local placeholder), which is expected
  because the version and date are publication inputs.

Already correct in source (`packaging/flatpak/site/index.html`, `scripts/flatpak/render-site.py`,
`.github/workflows/project-site.yml`, `.github/workflows/flatpak-remote.yml`) and now live: apex
`SITE_BASE`, apex canonical/`og:url`/`og:image`/brand links, new descriptor name, `CNAME` written
on both publication paths. Still absent **in source and live**: robots, sitemap, structured data,
`og:site_name`, 404 page, image format/responsive variants, and on-site help content.

Two source details matter for any proposal that adds root files:

- `project-site.yml` line 76 allows only `index.html|assets/*|CNAME` to change in a page-only run.
- `flatpak-remote.yml` lines 330–332 delete every root file except `index.html`, `.nojekyll`,
  `CNAME`, `${APP_ID}.flatpakref` and `ddl.flatpakrepo`.

A new `robots.txt`/`sitemap.xml` would therefore be **rejected** in the first path and
**deleted** in the second unless both allowlists are updated in the same change.

## 3. What already works

1. Server-rendered, JavaScript-free content: every crawler (including AI fetchers that do not
   execute JS) receives all 751 words, the install steps and the disclaimer in the HTML.
2. URL consolidation: `www`, the GitHub Pages project URL and the retired
   `vitormach.dev/Doom-Deck-Launcher/` path all `301` to the apex, matching the single canonical
   intent of spec 079 / D-039.
3. No user-agent-based cloaking: the page body is byte-identical (`sha256` prefix
   `7d86f9e9b99b2bd9`, 17 954 B after publication; `de2cc989eb52809d`, 18 064 B before it) for
   browser, `Googlebot`, `GPTBot`, `OAI-SearchBot`, `ClaudeBot`, `PerplexityBot` and `curl`
   user-agent strings. This shows the origin does not vary content by `User-Agent`; it does
   **not** prove that real crawler IP ranges, WAF rules at any intermediary, or requests from
   other networks reach the site (§8).
4. Accurate, honest metadata where it exists: title and description are unique, specific and
   within typical display lengths; `og:image` dimensions match the real file; alt text on all 8
   images; semantic landmarks and a skip link; `lang="en"`.
5. Trust/accuracy signals that matter for AI answers: explicit “experimental/pre-alpha”, “not a
   Flathub listing”, “never bundles or downloads commercial base games”, GPL-3.0-or-later, and a
   full trademark/non-affiliation disclaimer.
6. Healthy internal/external links (16 third-party targets, all `200`) and a valid TLS certificate
   for apex and `www`.
7. GitHub is already a second discovery surface with a real description, README screenshots and
   a website link in the README.

## 4. Findings, prioritized

Labels: **Confirmed** = observed here; **Proposed** = recommendation; **Owner** = cannot be done
from this repository.

### P0 — the correction that mattered, now verified resolved

**F1. Retired domain and application ID on the live page — RESOLVED 2026-10-21 ≈23:35 UTC.**
For the first half of the audit, canonical, `og:url`, `og:image`, brand link and both install
buttons pointed to `www.vitormach.dev/Doom-Deck-Launcher/` and
`io.github.vittau.Doom-Deck-Launcher`. *Cause:* the page-only workflow deferred until the new
signed channel existed (§1.6). *Resolution:* the signed publication regenerated the page
(v0.0.11), so at 23:36 UTC the canonical is `https://doomdecklauncher.com/`, the page contains
**0** occurrences of the retired host or ID, the new descriptor returns `200` and the retired
descriptor returns `404`. *Re-verify at any time:*

```sh
curl -sS https://doomdecklauncher.com/ | grep -o 'rel="canonical" href="[^"]*"'
curl -sS https://doomdecklauncher.com/ | grep -c 'vitormach.dev\|io\.github\.vittau\.Doom-Deck-Launcher'
# expect: canonical = https://doomdecklauncher.com/  and the count = 0
curl -sS -o /dev/null -w '%{http_code}\n' https://doomdecklauncher.com/com.doomdecklauncher.DoomDeckLauncher.flatpakref
```

Scope of that fix: **F1 only.** The publication did not change robots, sitemap, structured data,
`og:site_name`, image formats, copy or consoles — those remain the P1/P2 items below.

**Corrected during review — plain HTTP (no longer a finding).** An earlier draft listed
`http://doomdecklauncher.com/` returning `200` as an owner action item. That sample was taken at
23:27:55/23:28:25 UTC during Pages propagation; the current behaviour is `301` →
`https://doomdecklauncher.com/` from every Edge address (23:34 UTC), consistent with
`https_enforced: true` and an approved certificate, as independently confirmed by the root review
at 23:33:42 UTC. See §1.1. There is no HTTPS action item for the owner, and the observation must
not be treated as a current blocker. (No HSTS header is sent, which is normal for GitHub Pages and
not required.)

### P1 — cheap, high-leverage source changes (need approval; no code written)

*(Identifiers are kept stable for traceability: **F2** was the retracted plain-HTTP finding and now
appears as the “Corrected during review” note above.)*

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
policy question, not an SEO requirement — see §5.

**F4. No sitemap.** With one real page plus the credits page, a sitemap is small but genuinely
useful for a brand-new domain with almost no inbound links, because it is the one discovery
mechanism you control. Google states it improves discovery but **does not guarantee** crawl or
index. *Proposed:* render `sitemap.xml` from `render-site.py` with the two URLs
(`/`, `/assets/credits.html`) and `lastmod` = publication date; reference it from `robots.txt` and,
**after it exists**, submit it in Search Console/Bing Webmaster Tools (**Owner**; no submission is
claimed here). *Validation:* `curl` both files, then `xmllint --noout` on the fetched XML.

**F5. No structured data and no site-name signal.** Zero `application/ld+json`. Google's
site-names documentation says `WebSite` structured data is the strongest preference it accepts,
with `og:site_name` and the title as secondary inputs — the page currently has none of the
structured form and no `og:site_name`. *Proposed (accuracy-first, no invented facts):*

- `WebSite`: `name` “DoomDeckLauncher”, `alternateName` “DDL”, `url` apex.
- `SoftwareApplication`: `name`, `operatingSystem` “Linux”, `applicationCategory`
  “GameApplication”, `offers` `{ "@type": "Offer", "price": 0, "priceCurrency": "USD" }`,
  `license` GPL-3.0-or-later URL, `downloadUrl` → the `.flatpakref`, `softwareRequirements` →
  “legally owned DOOM IWAD”.
  **Accuracy caveat:** Google's Software-app rich result requires `name`, `offers.price` **and**
  a `rating`/`review` or `aggregateRating`; the project has no genuine ratings, so this markup
  would be valid and useful for entity understanding but **must not** be expected to produce a
  rich result. Do not add a fabricated rating to chase one.
- **Do not invent an `Organization`.** DDL is an individual-authored project, not a company or
  registered entity; adding an `Organization` node would assert a corporate identity that does not
  exist. If the owner wants the author attributed, use a `Person` node with the same `sameAs`
  repository link and only when a public author name is already on the page.
- Add `og:site_name`, `og:locale`, `og:image:type`, `twitter:title`, `twitter:description`.

Note the scope honestly: Google states that structured data **is not required** for generative AI
search and that there is no special schema.org markup to add — it helps eligibility for classic
rich results and machine-readable entity clarity, nothing more.

*Validation:* parse the JSON-LD in `packaging/flatpak/site/test_render.py` (assert the fields and
that `price` is `0`), then the Rich Results Test manually once, and confirm every claim matches
visible page text.

**F6. Page weight is dominated by PNG screenshots; a third-party font request is secondary.**
2.87 MB transfer, LCP 5.4 s, 2.1 MB of image-format savings and 1.3 MB of responsive savings
available. The donation widget also causes one render-blocking `fonts.googleapis.com` stylesheet
(909 ms) because the owner-approved Buy Me a Coffee embed requests the Cookie font
(`data-font="Cookie"`); otherwise the page self-hosts Barlow and Pirata One with
`font-display: swap`. *Proposed:* (a) the image work is the big win — commit WebP derivatives of
`docs/screenshots/*.png` and `docs/design/logo/steam/hero.png` and serve them via `<picture>` /
`srcset` with the PNGs as fallback, adding `fetchpriority="high"` to the hero; (b) for the widget,
**preserve the approved branding** and pick one of: `preconnect` to `fonts.googleapis.com` and
`fonts.gstatic.com` plus `defer` on the embed, or a local equivalent that keeps the same logo and
Cookie-font look without a third-party request. Removing the widget is **not** required and is the
owner's call, not an audit finding. *Validation:* re-run the same Lighthouse invocation and
compare LCP, transfer size and `modern-image-formats`; assert new asset names/hashes in
`test_render.py` so a missing derivative fails CI, and keep
`test_official_donation_embed_matches_owner_configuration` green (or update it deliberately if the
owner changes the embed).

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

**F10. Inbound citations: unknown, and likely thin.** What is verifiable: the repository was
created `2026-10-14`, the domain and certificate are hours old, the GitHub repository shows
`homepage: null`, `topics: []` and 0 stars/forks (public API), and the anonymous search probes in
§8 found nothing conclusive. **This is not proof that there are no inbound citations** — neither
an index nor a backlink source was available, so the honest label is *unknown*. The direction is
still right: organic discovery of a week-old project with no listed website field and no topics is
unlikely, and it is the one lever that no amount of markup replaces. *Proposed:* set the
repository **About** website to `https://doomdecklauncher.com/` and add topics (`steam-deck`,
`doom`, `flatpak`, `wad`, `launcher`, `rust`, `slint`, `dsda-doom`); then earn real citations from
the DOOM/Steam Deck community (Doomworld thread, r/SteamDeck, DSDA-Doom discussions, UZDeck/GZDeck
comparison mentions, SteamGridDB, Flatpak community spaces). This is outreach, not markup, and it
cannot be automated from this repository.

**F11. Search Console and Bing consoles: partly solved, partly unknown.** The owner confirmed on
2026-10-21 that **Search Console is configured**, and that is corroborated publicly: a
`google-site-verification` TXT record is now present at the apex (none existed at 23:28 UTC), which
is the DNS method for a *domain* property. Record that as **owner-confirmed, verified by DNS** —
not as “unverified”. What remains genuinely unknown: index coverage, impressions, crawl stats and
whether a sitemap was ever submitted (it cannot be, until a sitemap exists — do not claim
otherwise). For **Bing Webmaster Tools** there is still no public artifact (`BingSiteAuth.xml`
`404`, no `msvalidate.01` meta), so its state is unknown rather than absent. See §7.

**F12. `llms.txt` is not a requirement.** None of the crawler documents read for this audit
(OpenAI, Anthropic, Perplexity, Google) mentions it, and no vendor documents a ranking or citation
benefit. Adding one is optional and harmless at most; it is not on the critical path and this
report does not recommend it as anything more than optional.

## 5. What each AI surface actually needs

Every claim below is from the vendor's own documentation read on 2026-10-21 (links in §10). The
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

- **Nothing is blocked by the origin for any of them** — no `robots.txt` at all, no `noindex`, and
  no `User-Agent`-dependent responses. That is the right starting state. (Byte-identical test
  requests to spoofed user-agent strings do not by themselves prove that real crawler IP ranges or
  any intermediary WAF reach the site; see §8.)
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

**Google (Confirmed from primary docs, exact text checked):** a page must be indexed and
snippet-eligible to appear in generative AI features. `ai-optimization-guide` adds, literally:
“In addition to the technical requirements for Search, a site must be included in Search
generative AI features in Search Console to be eligible for display in generative AI features on
Google Search.” That sentence is ambiguous on its own, so the linked Search Console help page was
read too: there is a **Search generative AI control** under *Settings → Search generative AI*,
rolled out to all websites worldwide as of **2026-08-31**, with three choices — *Include my site's
links and content in Search generative AI features* (**the default for all properties**),
*Exclude…*, and *Inherit control from parent*. So the actionable reading is narrow and is **not**
an “enable a setting” task: confirm the control is on the default *Include* (or that an inherited
parent choice is not *Exclude*). Indexing and serving are still not guaranteed. Canonicalization
prefers redirects over `rel="canonical"`. Sitemaps help discovery but do not guarantee crawling.

**Bing (Confirmed from primary docs):** IndexNow is the documented push mechanism and is
supported by Microsoft Bing (plus Naver, Seznam.cz, Yandex, Yep); it requires a key and an
HTTP ping per changed URL, and Google is not a participant. Conventional discovery still runs
through the Bing index and Bing Webmaster Tools, and a sitemap remains the baseline. Copilot's web
grounding is served from Microsoft's search stack, so Bing indexation is the actionable lever —
**Proposed inference**, not a vendor statement about Copilot specifically.

**Sitemaps and robots:** both are discovery/control aids, never ranking controls; the honest
expectation is “faster, more predictable discovery”, not visibility.

## 7. Owner-only actions (cannot be done from this repository)

1. **Search Console** (*owner-confirmed configured; DNS verification visible*): nothing to set up.
   After a sitemap is published, submit it — do **not** record any submission before that exists.
   Optionally run URL Inspection on `https://doomdecklauncher.com/` and confirm
   *Settings → Search generative AI* is on the default *Include* choice (§6). Coverage and
   impression data stay owner-side and were not available to this audit.
2. **Bing Webmaster Tools** (*state unknown*): add/confirm the site, submit the sitemap once it
   exists, request indexing. Optionally adopt IndexNow (key file at the publication root — a small
   source change, and it intersects the two allowlists in §2).
3. **GitHub repository About**: set the website URL and topics (F10).
4. **Community citations**: Doomworld / r/SteamDeck / DSDA-Doom / UZDeck-GZDeck threads (F10).
   This is the only route to the inbound links that both classic and AI search rely on.
5. **Decision** on whether to keep the OAI-SearchBot/ClaudeBot/PerplexityBot/GPTBot policy open
   (recommended default) and whether to keep the retired descriptor alive for cached pages
   (open question §9).

No HTTPS action is listed: `Enforce HTTPS` is already effective (§1.1 and the P0 note in §4).

## 8. Limitations

- No access to Search Console data, Bing Webmaster Tools, Cloudflare analytics or GitHub traffic,
  and no backlink source: **index coverage, crawl rate, impressions, inbound links and AI citations
  are unknown and unmeasured.** Search Console itself is configured (owner-confirmed, and its DNS
  verification TXT record is public), but its private data was not available here.
- Anonymous SERP probes were inconclusive and are not used as evidence: DuckDuckGo's HTML
  endpoint returned `202` (bot challenge, 0 results); Bing's `site:` and quoted-brand queries
  returned unrelated localized results; the connected search integration returned no results for
  `DoomDeckLauncher`. None of these can distinguish “not indexed”, “not cited” or “no inbound
  links” from “probe blocked”.
- **User-agent strings are not crawler verification.** Sending `Googlebot`/`GPTBot`/… as a
  `User-Agent` from an ordinary host shows only that the origin does not branch on that string. It
  says nothing about the real crawler's IP ranges, any WAF or bot-management layer, or reachability
  from other networks and regions; verifying that requires server logs or vendor tooling.
- **Edge/propagation timing matters.** The HTTP→HTTPS check returned `200` at 23:27–23:28 UTC and
  `301` at 23:33–23:34 UTC from the same host, so a single scheme/header sample can be stale by
  minutes during a Pages setting change (§1.1).
- PageSpeed Insights API refused anonymous requests with a daily-quota `429`; the performance
  numbers come from a single local Lighthouse 12.8.2 mobile run (lab data, not field/CrUX data —
  the domain is far too new to have field data).
- Lighthouse accessibility 100 means automated checks pass, not that the page is accessible.
- The report describes one host, one network vantage point and a moving target: the certificate was
  hours old and the signed publication changed the live page mid-audit (§1 state table). Anything
  here can be outdated after the next deployment; re-run the commands in §11 before acting.
- Behaviour of the actual signed installation, Discover, Gaming Mode and the launcher itself is
  out of scope for this audit (**Needs Deck Validation** elsewhere).
- Cloudflare purge/proxy behaviour was not tested (the apex is DNS-only from this vantage point),
  and no credential, console or API mutation was attempted.

## 9. Open questions

1. **H1 copy:** is changing the visible `h1` to include “DoomDeckLauncher” (and keeping the
   artwork line as an eyebrow) acceptable design-wise? It is the cheapest on-page relevance fix.
2. **Donation widget:** the embed is an explicit owner-approved logo/font/script choice, so removal
   is not a recommendation. If the 909 ms render block is worth addressing, is a `preconnect` +
   `defer` variant enough, or would the owner accept a local equivalent that keeps the same
   Buy Me a Coffee logo and Cookie-font look without the third-party request? (Current
   `test_official_donation_embed_matches_owner_configuration` pins the approved embed.)
3. **Retired install URL after publication:** the full publisher has now deleted
   `io.github.vittau.Doom-Deck-Launcher.flatpakref` from the root (observed `404` at 23:36 UTC),
   while browsers, social caches and search indexes may reference it for days (Cloudflare cache is
   only 600 s, but social cards are not). Owner already waived migration; should a temporary shim
   that installs the **new** channel be served at the old path, or is a clean `404` preferred?
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
| [Google — Optimizing for generative AI features](https://developers.google.com/search/docs/fundamentals/ai-optimization-guide) | “still SEO”, RAG/query fan-out, “indexed and snippet-eligible”, Search generative AI control, structured data not required, `llms.txt` named as unnecessary, no indexing guarantee |
| [Search Console Help — Search generative AI control](https://support.google.com/webmasters/answer/16908024) | *Settings → Search generative AI*; default *Include*; Exclude/Inherit; worldwide rollout 2026-08-31 |
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
# Headers, redirects, scheme handling (re-checked 23:34 UTC: 301 from every Edge address;
# an earlier 23:27 UTC sample returned 200 from one Edge during propagation — see §1.1)
curl -sS -o /dev/null -D - https://doomdecklauncher.com/
curl -sS -o /dev/null -D - https://www.doomdecklauncher.com/
curl -sS -o /dev/null -D - http://doomdecklauncher.com/       # 301 -> https://doomdecklauncher.com/
for ip in 185.199.108.153 185.199.109.153 185.199.110.153 185.199.111.153; do
  curl -sS -o /dev/null -w "%{http_code} " --resolve doomdecklauncher.com:80:$ip \
    http://doomdecklauncher.com/; echo "$ip"
done

# Discovery files
for p in robots.txt sitemap.xml favicon.ico llms.txt site.webmanifest \
         com.doomdecklauncher.DoomDeckLauncher.flatpakref \
         io.github.vittau.Doom-Deck-Launcher.flatpakref; do
  curl -sS -o /dev/null -w "%{http_code} $p\n" "https://doomdecklauncher.com/$p"
done

# No User-Agent-based cloaking at the origin (NOT a crawler-IP or WAF test — see §8)
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
  --app-id com.doomdecklauncher.DoomDeckLauncher --site-base https://doomdecklauncher.com
# exit 1 before the signed publication (23:26 UTC); exit 0 afterwards (23:36 UTC)

# Performance / accessibility (headless, no visible window) — run twice, values vary
npx --yes lighthouse@12 https://doomdecklauncher.com/ \
  --only-categories=performance,accessibility,seo \
  --output=json --output-path=lh.json \
  --chrome-flags="--headless=new --no-sandbox --disable-gpu" --quiet
```

## 12. Deliverables and next step

Only `docs/reports/080-seo-and-ai-discovery-audit.md` was added, then corrected after review, on
`vittau/seo-audit`. No source, workflow, DNS, Pages, credential or console change was made, and
nothing was pushed, tagged or published. **The signed publication landed during the audit and
closed F1**: the live page now carries the apex canonical, apex `og:` URLs and the new descriptor,
verified at 23:36 UTC with the commands in §4 F1, after the owner's prefix purge was accepted at
23:35:05 UTC. What is left is a decision, not a defect: pick which P1 items get a follow-up task —
best value-per-risk are `robots.txt` + `sitemap.xml` (remembering both publication allowlists) and
the image-format/responsive work — while the owner handles the remaining console work (Bing, plus
submitting a sitemap **after** one exists), repository About/topics, community citations and the
open questions in §9. Search Console itself is configured and DNS-verified; only its private data
is unavailable here. The donation embed stays an owner branding choice, not a finding.

### Corrections applied in this review pass

- Live publication recorded: the page changed between 23:32 and 23:36 UTC, so §1 now carries a
  before/after table, §1.3/§1.4/§1.5 were re-measured, F1 is marked **resolved and verified**, and
  the “next step” no longer asks anyone to wait for the publication. Added the publication run
  (`39693631806`) and the accepted prefix purge (23:35:05 UTC) as coordinator evidence.
- Search Console corrected from “unverified” to **owner-confirmed configured**, corroborated by the
  `google-site-verification` TXT record that appeared at the apex during the audit; no sitemap is
  claimed submitted, and Bing remains unknown (§4 F11, §7, §8).
- Plain-HTTP `200` retracted: recorded the 23:27–23:28 UTC versus 23:33–23:34 UTC timing and Edge
  discrepancy, restored the `301` as current evidence, and removed the obsolete owner HTTPS
  action (§1.1, §4, §7, §11).
- Search Console generative-AI requirement quoted exactly and resolved against the Search Console
  help page, which shows a default-*Include* control rather than an action to enable (§6, §10).
- Removed `Organization` from the structured-data proposal: DDL is an individual project, not an
  entity to invent (§4 F5).
- Reframed the donation widget as an owner-approved branding choice; removal is not recommended,
  and the embed test is expected to stay green (§4 F6, §9).
- Renamed unmeasurable conclusions to *unknown*: inbound citations and console/verification state
  (§4 F10/F11, §7, §8), with the observed DNS/file/meta evidence stated separately.
- Qualified spoofed-`User-Agent` testing as origin-only evidence, not crawler-IP or WAF proof
  (§3, §5, §8, §11), and kept “missing robots is not a blocker” explicit (§1.2, §4 F3).
- Performance reported as two runs instead of one score, with timings as a range and only weight,
  format and LCP-element findings treated as stable (§1.5).
- Fixed section references (`§5`/`§10`) and narrowed the publication's benefit to F1 only.

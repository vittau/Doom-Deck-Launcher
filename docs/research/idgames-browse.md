# /idgames browse research — a data source for the unverified tier

Scope: what DDL can use to browse and search the `/idgames` archive **without** the blocked
Doomworld API, how good and how fresh that data is, what "unverified" must mean in the UI and on
launch, how untrusted archives are handled, and where the browse UX lives. This document is the
evidence for spec 041, R1–R4; the proposed build specs and owner questions are in
`docs/reports/041-idgames-browse-research.md`.

All claims are labelled **Confirmed** (observed in this session or quoted from a primary source),
**Proposed** (a DDL decision, not an upstream fact) or **Needs Deck Validation** (cannot be verified
on the macOS arm64 dev host).

## Method and evidence

- Research date: **2026-10-19**, roughly 18:20–18:45 UTC. Host: macOS arm64. Tools: `curl`,
  `python3` (stdlib only), `shasum`, `zstd`.
- Everything was read-only. No archive was downloaded in full and nothing was executed. Scratch
  files live in `target/research-041/` (git-ignored, never committed).
- Request budget: **≈ 150 HTTP requests**, mostly small text files and HEADs; one 419 KB index and
  one 445 KB index were downloaded. No bulk crawl was run. The first full index build described in
  §4 is an estimate from a 40-file sample, not a completed crawl.
- One 40-file sample of per-file `.txt` sidecars was drawn with a fixed seed, stratified by era
  (10 files per 1994–2000, 2001–2010, 2011–2018, 2019–2026), from `levels/doom` and
  `levels/doom2` (excluding `/deathmatch/`). It is a sample, not a census; percentages are
  estimates. The sample list is in `target/research-041/sample-meta.json` (not committed).
- Wayback snapshots are used for `doomworld.com`, which blocks automated clients; the
  snapshot URL is written with `id_` so the cited page is the original payload, not the Wayback
  wrapper.

---

## 1. Summary and recommendation

**Confirmed (this session):** the Doomworld `/idgames` frontend and API are Cloudflare-challenged
from this host: `https://www.doomworld.com/idgames/?search=auger` and
`https://www.doomworld.com/idgames/api/api.php?action=search&query=scythe&out=json` both returned
**HTTP 403, 5 KB, `<title>Just a moment...</title>`** on 2026-10-19. This reproduces
`docs/research/upstream-sources.md` §3.1 (2026-10-16).

**Confirmed:** the archive mirrors themselves are reachable and publish two kinds of data:

| source | what it is | measured |
| --- | --- | --- |
| `ls-laR.gz` / `fullsort.gz` on every mirror | archive-generated file listing | 419,389 B gzipped, 2.79 MB raw, 43,741 files in 461 directories, refreshed daily (mtime 2026-10-19 05:07 UTC) |
| per-file `.txt` sidecars on every mirror | the author's own text file, same stem as the `.zip` | 3.2 KB mean, 0.64 s mean fetch, 10,877/10,885 DOOM-family single-player zips have one |

**Recommendation (Proposed):** build the tier on **Option C — a DDL-built index published like the
catalog bundle (spec 019)** — using the mirror index (A) and the `.txt` sidecars (B) as build
inputs. The app then:

1. fetches one small JSON index (≈ 0.6–2 MB) from a CI branch, at most once per 24 h, and caches it
   (`Indexed` metadata only);
2. fetches one `.txt` (≈ 3 KB) on demand when the player opens an entry, and keeps it in the local
   cache for the author's own credits/permission text;
3. downloads the `.zip` from an allowlisted mirror with a pinned **size** (not a known hash), and
   records a trust-on-first-download SHA-256.

This is the only option that gives title/author search, instant offline browse and a bounded
per-user download without making every player crawl 10 885 files (Option B alone) or showing
filenames without titles (Option A alone). It also keeps the app independent of the blocked API,
which D-015 already requires.

**Why not the API (Confirmed):** it is blocked, its fields are rich but it publishes
**no hashes** (`docs/research/upstream-sources.md` §3.2, from the API documentation captured
[2025-08-16](https://web.archive.org/web/20250816213631id_/https://www.doomworld.com/idgames/api/)),
and its terms are only a blanket "© 2012–2016 Doomworld.com All Rights Reserved" line. The frontend
itself says it is *"NOT AN OFFICIAL /IDGAMES MIRROR … a community service provided by Linguica and
Doomworld"* — a third party, not the archive — so its data should not become a hard dependency of a
GPL launcher.
[Browse snapshot, 2026-09-26](https://web.archive.org/web/20260926022352id_/https://www.doomworld.com/idgames/).

**Confirmed (snapshot):** Doomworld's only readable site-wide terms are its **forum registration
terms** ([Wayback, 2025-06-22](https://web.archive.org/web/20250622192657id_/https://www.doomworld.com/terms/));
they say nothing about API use, bulk metadata or hotlinking. The API page adds only the
"All Rights Reserved" line, so Doomworld's terms for this data remain an **open question** (report,
question 5–6); using the mirrors avoids the question for the product.

---

## 2. The archive index files (Option A)

### 2.1 What they are and how big they are

**Confirmed.** `https://www.gamers.org/pub/idgames/ls-laR.gz` is a gzip of a Unix `ls -laR` of the
whole archive (fetched 2026-10-19):

| quantity | value |
| --- | --- |
| compressed | **419,389 B** |
| uncompressed | **2,788,935 B** |
| lines | 45,126 |
| directories | 461 |
| regular files | 43,741 |
| `.zip` / `.txt` | 21,424 / 21,181 |
| `levels/` zips / txt | 15,063 / 15,069 |
| `levels/` zip bytes | **31.84 GB** (all games) |
| SHA-256 | `502a7ccd3195452fb4955f077a078d0167bf80b721535be8c59e1daa269c2603` |
| `Last-Modified` | **Mon, 19 Oct 2026 05:07:08 GMT** |
| download (gamers.org) | 1.27 s, ≈ 331 KB/s |

**Confirmed:** the same file is **byte-identical on seven mirrors** — gamers.org, youfailit.net,
ftp.fu-berlin.de, mirrors.lug.mtu.edu, lethe.chinstrap.org, ftpmirror.infania.net and
mirror.braindrainlan.nu all return 419,389 B and the same SHA-256 (checked 2026-10-19). Mirror
`Last-Modified` values differ by a few hours because each host syncs on its own schedule
(youfailit reported 09:07 GMT).

**Confirmed:** `https://www.gamers.org/pub/idgames/fullsort.gz` (445,245 B at HEAD, 445,273 B at GET
minutes later) is a second archive-generated index with **one record per line**, sorted
newest-first, of the form `YYYY/MM/DD <size> <path>`:

```text
2026/10/18   10370840  levels/doom2/Ports/megawads/tistartan.zip
2021/07/26   12771737  levels/doom2/Ports/megawads/dbp37_augzen.zip
1993/02/03     260606  historic/doom0_2.zip
```

It has 43,264 dated records (after dropping the `fullpath` prologue/epilogue). It also lists
`incoming/` and `newstuff/` copies, so the same upload appears two or three times.

### 2.2 What fields it gives

**Confirmed.** `ls-laR` records permissions, link count, owner, group, size, a month/day/year (or
month/day/time for files newer than about six months) and the file name, grouped by directory:

```text
./levels/doom2/Ports/megawads:
-rw-rw-r--  1 idgmaint idgames      6212 Jul 26  2021 dbp37_augzen.txt
-rw-rw-r--  1 idgmaint idgames  12771737 Jul 26  2021 dbp37_augzen.zip
```

**Confirmed:** it has **no title, author, IWAD, port or complevel**. It gives the path (and thus the
archive category), size and an archive date. `fullsort.gz` gives the same three fields with
unambiguous ISO dates but no directories.

**Confirmed:** the archive's own naming policy makes the path useful: uploads must be lowercase,
≤ 12 characters, ASCII letters/numbers/dashes/underscores, and are filed under
`/pub/idgames/levels/<game>/…` ([`README.INCOMING`, updated 28 Jan 2023](https://www.gamers.org/pub/idgames/README.INCOMING)).
Files are moved to `newstuff/` **and** their final home at the same time and the `newstuff` copy is
deleted after 10 days, so only `levels/` paths are durable.

### 2.3 Size distribution and the download cap

**Confirmed**, from the 15,063 `levels/` zips in `ls-laR.gz`:

| percentile / threshold | bytes |
| --- | --- |
| p50 | 118,817 |
| p90 | 3,813,990 |
| p99 | 38,837,295 |
| max | **426,662,505** (`levels/doom2/Ports/s-u/slipgate.zip`) |
| > 10 MB | 669 (4.44 %) |
| > 50 MB | 67 (0.44 %) |
| > 100 MB | 16 (0.11 %) |
| > 200 MB | 6 (0.04 %) |

The archive's upload guidance says files should not exceed 100 MB without prior arrangement
(confirmed in `README.INCOMING`), yet 16 are larger. A practical unverified-tier hard cap is
**512 MiB** with a warning above 100 MB (Proposed, §7.1).

### 2.4 Freshness

**Confirmed:** the master index was regenerated at **05:07 UTC on the day of research** and already
contained a file uploaded the previous day (`tistartan.zip`, 2026-10-18 11:48). The rolling lists
confirm the update rate:

| file | zips listed |
| --- | --- |
| `LAST.24hours` | 2 (both still in `incoming/`) |
| `LAST.7days` | 14 |
| `LAST.30days` | 71 |

So the steady-state rate is **≈ 2–3 new archives per day**; a daily index refresh is plenty.
`fullsort.gz` changed size between a HEAD and a GET minutes apart, so it is regenerated more than
once a day; `ls-laR.gz` had the same 419,389 B content on every mirror during the session (only the
per-mirror `Last-Modified` values differ).
Freshness is "same day" but not real-time: uploads after the 05:07 snapshot wait for the next day
(Confirmed from the `incoming/` entries in `LAST.24hours` that are not yet in `levels/`).

### 2.5 Terms

- **Confirmed (quoted):** the archive asks clients to spread load:
  *"ATTENTION: To reduce the load on this machine … please check the following list for a
  mirroring site closer to you and USE it."* ([`README`](https://www.gamers.org/pub/idgames/README),
  fetched 2026-10-19, itself last modified 2026-07-29).
- **Confirmed (quoted):** `README.legal` allows redistribution but forbids selling the archive:
  *"The material at this ftp site is not placed on CDROM or sold for profit … Do not upload
  anything to this site that has distribution restrictions that prohibit this."*
  ([`README.legal`](https://www.gamers.org/pub/idgames/README.legal), unchanged since 2005).
- **Confirmed (observed):** [`robots.txt`](https://www.gamers.org/robots.txt) disallows
  `/pub/incoming/`, the duplicate symlink `/pub/games/idgames/`, and a list of named crawlers
  (including `GPTbot`, `claudebot`, `Claude-SearchBot`, `PerplexityBot`, `SemrushBot`, `MJ12bot`);
  it does **not** disallow `/pub/idgames/levels/`. A DDL crawler must use its own descriptive
  `User-Agent` (DDL already sends `DoomDeckLauncher/<version>`, spec 006) and must not masquerade
  as a browser or a blocked bot.
- **Proposed:** treat the index files as a public archive listing to be read politely, not as a
  dataset to redistribute wholesale: DDL stores derived fields, not the `ls` file itself.

### 2.6 Failure modes (measured)

- **Confirmed:** the **master rate-limits bursts**. After ≈ 20–30 requests to `www.gamers.org` in
  about three minutes, every subsequent connection timed out for roughly ten minutes; a single
  retry later returned 200. The mirrors did not do this.
- **Confirmed (observed):** some mirrors are behind Cloudflare and block automation:
  `https://files.xvertigox.com/idgames/…` returns **403** with
  `<title>Attention Required! | Cloudflare</title>`; `https://planetzdoom.com/idgames/ls-laR.gz`
  returns a **"Just a moment…"** challenge while its file paths 302-redirect to
  `idgames.planetzdoom.com` (see the next bullet).
- **Confirmed:** mirror base paths are **not** uniform. The published mirror list says
  `/pc/games/idgames/` for ftp.fu-berlin.de, `/idgames/` for lethe, mtu and planetzdoom, and
  `/pub/idgames/` for gamers.org (and, by observation, youfailit, infania and braindrainlan)
  ([`README`](https://www.gamers.org/pub/idgames/README)). Assuming `<host>/pub/idgames` gives 404s.
- **Confirmed:** `planetzdoom.com/idgames/...` **302-redirects to `idgames.planetzdoom.com`**, a
  new host that is not in `catalog/hosts.toml`; spec 006 re-checks every redirect target, so the
  host must be allowlisted explicitly or planetzdoom dropped from the rotation.
- **Confirmed:** one transient 404 from `ftpmirror.infania.net` for a file that returned 200 seconds
  later; mirrors can lag or hiccup.
- **Confirmed:** there is **no checksum** anywhere in the index or beside the files — only size and
  date.
- **Confirmed:** parsing `ls -laR` dates is lossy: files older than about six months print
  `Jul 26  2021`, recent ones print `Oct 18 11:48` with no year. The year for recent files must be
  inferred (the archive is generated "now"), and the timestamps are server-local, so only the day
  is trustworthy. `fullsort.gz` avoids this with ISO dates.
- **Confirmed:** `ls-laR.gz` also contains `incoming/`, `incoming/holding/`, `newstuff/` and other
  non-level trees. Indexing them would create duplicates and dead paths.

### 2.7 Verdict

**Proposed:** usable as the **file inventory** (path, size, date) and as the build input for
Option C. Not usable on its own for search: with no titles, the player would browse 10 885
filenames such as `dbp37_augzen.zip`, which is exactly the "thousands of unverified entries"
OVERVIEW §9 warns against presenting as a catalog.

---

## 3. Per-file `.txt` sidecars (Option B)

### 3.1 Guarantee and coverage

**Confirmed (quoted):** every upload must ship a same-named `.TXT` **outside** and **inside** the
zip: *"All files must be ZIPped and have a same-named .TXT file uploaded with the ZIP, as well as a
copy of the same .TXT inside of the ZIP."*
([`README.INCOMING`](https://www.gamers.org/pub/idgames/README.INCOMING), requirement 1).
That policy is why the sidecars are near-universal.

**Confirmed:** in `levels/` there are 15,063 zips and 15,069 txts; **15,054 zips have a same-stem
sidecar (99.94 %)**, 9 do not, and 15 txts have no matching zip. For the DOOM-family
single-player subset (`levels/doom` + `levels/doom2`, excluding `/deathmatch/`): 10,885 zips,
**10,877 with a sidecar**, 8 without. The 8 missing ones are mostly paired uploads that share one
txt (`24hour_a`/`24hour_b`, `cleim20a`/`cleim20b`, `ssatc1`/`ssatc2`, `marx1_1`/`marx1_2`).

**Confirmed:** the sidecar is the same bytes as the copy inside the zip, at least for the reference
release: the sidecar's SHA-256, `674c33641557b18aa17d10628ba3d567970025b2d916dad3a87d1e9eb5dc3e90`,
is exactly the hash recorded for `DBP37_AUGZEN.txt` inside `dbp37_augzen.zip` in
`docs/research/upstream-sources.md` §5.2 (one sample; the upload policy says they are copies).

Example: <https://youfailit.net/pub/idgames/levels/doom2/Ports/megawads/dbp37_augzen.txt>
(6,212 B, `Last-Modified: 2021-07-26`).

### 3.2 Size, timing and transport

**Confirmed** (40-file sample, fetched from youfailit.net on 2026-10-19):

| quantity | value |
| --- | --- |
| successes | 40/40 (HTTP 200) |
| total bytes | 128,433 |
| mean size | 3,210 B |
| min / max | 604 B / 13,663 B |
| mean latency | **0.64 s** (sequential, 0.15 s pause) |
| max latency | 1.73 s |
| `Content-Length` vs index size | exact match, 40/40 |

**Confirmed:** the mirror answers with `Content-Length`, `Last-Modified`, a strong `ETag` and
`Accept-Ranges: bytes`; conditional GETs are therefore possible. youfailit also advertises
`x-ws-ratelimit-limit: 1000` (remaining decremented per request and replenished within 65 s in this
session), but the mirror's actual window is undocumented.

**Confirmed:** all seven working mirrors served the newest upload (`tistartan.txt`, 2026-10-18)
with the same 5,442 B length within a day of upload. Byte-identity was not hashed on every mirror;
the one sample above was.

### 3.3 What fields it gives (40-file sample)

The modern template is consistent enough to parse; older files vary. **Confirmed (sample counts):**

| field | present | notes |
| --- | --- | --- |
| `Title:` | 38/40 | two free-form files (1996, 2003) use no `Title:` label |
| `Author:` | 38/40 | same two |
| `Game:` | 27/40 | plus 8 with the older `Episode and Level #:` → **35/40 (87.5 %)** declare a game |
| `Advanced engine needed:` | 19/40 | often `None`, `Na` or absent (unknown) |
| `Primary purpose:` | 23/40 | |
| `Map #:` | 25/40 | |
| `Other files required:` | 19/40 | |
| `Tested With:` | 12/40 | evidence of testing, **not** a requirement |
| `May Not Run With…:` | 20/40 | strongest negative signal |
| any `complevel` mention | 3/40 | and two of those are traps, see below |

**Confirmed:** the archive path supplements the txt: `levels/doom2/…` vs `levels/doom/…` gave the
base game for **100 %** of the sample (32 doom2, 5 doom, plus 2 heretic and 1 hexen, which DDL's
Doom-only base games cannot install).

**Confirmed examples that matter for gating:**

- ZDoom/GZDoom required:
  [`blkfortx.txt`](https://youfailit.net/pub/idgames/levels/doom2/Ports/a-c/blkfortx.txt)
  (`Advanced engine needed: ZDoom`, `May Not Run With...: Only works with ZDoom!`);
  [`3dent.txt`](https://youfailit.net/pub/idgames/levels/doom2/Ports/0-9/3dent.txt)
  (`GZDoom 3.2.0 and up`, `May Not Run With: Non-GZDoom Ports`).
- DSDA-friendly evidence: [`tistartan.txt`](https://youfailit.net/pub/idgames/levels/doom2/Ports/megawads/tistartan.txt)
  (`Advanced engine needed: MBF21+UMAPINFO`, `Tested With: … DSDA-Doom 0.30.0`);
  [`abyspe65.txt`](https://youfailit.net/pub/idgames/levels/doom2/Ports/a-c/abyspe65.txt)
  (`Boom`, tested with PrBoom+/ZDoom/GZDoom);
  [`ph_25yoe.txt`](https://youfailit.net/pub/idgames/levels/doom2/Ports/megawads/ph_25yoe.txt)
  (`limit-removing`, "safe to use in prboom+ complevel 2").
- Unknown/absent: [`catacomb.txt`](https://youfailit.net/pub/idgames/levels/doom2/a-c/catacomb.txt)
  (1995, no engine line, no game line) and
  [`jbase1.txt`](https://youfailit.net/pub/idgames/levels/doom2/j-l/jbase1.txt)
  (`Advanced engine needed: None`).

**Confirmed parsing traps:**

- Free-text `complevel` is not a target. `ccpr.txt` says
  *"You will need -complevel 17 when recording demos"* — a demo-recording hint, not a requirement;
  `ph_25yoe.txt` mentions both "complevel 2" (the target) and a lighting note. A text scraper that
  takes the first match gets it wrong.
- `Tested With` is not a requirement: `abyspe65.txt` lists GZDoom among tested ports while its
  `Advanced engine needed` is Boom.
- `Advanced engine needed: None` / `Na` / `None!` means "no special engine", not a missing field.
- 1 of 40 files was **not UTF-8** (byte `0xFC`, a Latin-1/CP437 `ü`). The parser needs a
  UTF-8 → Windows-1252/CP437 fallback and must not fail the build.
- Descriptions wrap across lines with irregular indentation; a field value is not always one line.
- 2 of 40 use free-form headers with no `Title:`/`Author:` labels; the filename is the only
  reliable fallback (one of them still contains a title in prose).

### 3.4 Freshness

**Confirmed:** a sidecar's `Last-Modified` is the upload/mirror time (2021-07-26 for AUGER;ZENITH,
which the archive dates 2021-07-26). Unlike the daily index, a sidecar is available as soon as the
file is; all mirrors had the 2026-10-18 upload's txt the next day. It is therefore the freshest
metadata source, but only per file — there is no searchable list.

### 3.5 Terms

- **Confirmed:** the archive's `README.legal` requires uploads to be freely redistributable and the
  archive not to be sold; the upload policy requires the txt next to and inside each zip.
- **Confirmed:** each txt carries the author's own `Copyright / Permissions` section (AUGER;ZENITH's
  is quoted in `upstream-sources.md` §4.3). Those permissions vary; DDL must not republish the txt
  wholesale in its own index.
- **Proposed:** the index stores **facts** only — title, author, date, size, inferred game/port
  flag — and never the description, credits, reviews or the txt body. The txt itself is fetched to
  the user's own cache for display and kept with the install as the credits file, exactly like a
  browser cache.
- **Proposed:** always link the canonical `/idgames` page for attribution and show the author
  name; do not present the data as DDL's own.

### 3.6 Failure modes

- 8/10,885 DOOM-family zips have no sidecar → title/author unavailable, filename fallback.
- Wrong base path → 404; Cloudflare-protected mirrors → 403; mirror lag → 404 then 200.
- A per-user crawl of 10,885 files would be ~35 MB and ~2 h at the measured rate, and 10 885
  requests per player against a volunteer mirror is unacceptable (Proposed). This is the decisive
  argument against Option B as a **runtime** strategy.
- Parsing quality: ≈ 5 % missing title/author, ≈ 12.5 % missing game declaration, ≈ 52 % missing
  engine line (sample). Unknowns must stay unknown.

### 3.7 Verdict

**Proposed:** excellent **detail source** and the input to the index build; unacceptable as the
only runtime source.

---

## 4. DDL-built index (Option C)

### 4.1 Shape

**Proposed** schema (JSON, published like the spec 019 bundle):

```json
{
  "schema": 1,
  "generated_at": "2026-10-19T05:07:08Z",
  "source": {
    "index": "https://youfailit.net/pub/idgames/ls-laR.gz",
    "index_sha256": "502a7ccd…",
    "mirror": "youfailit.net"
  },
  "min_ddl_version": "0.1.0",
  "entries": [
    ["levels/doom2/Ports/megawads", "dbp37_augzen", "AUGER;ZENITH",
     "Doomer Boards Community", "2021-07-26", 12771737, "doom2", "unknown", 1]
  ]
}
```

- `path` is `levels/<game>/…/<stem>`, the stable key; the `.txt` and `.zip` URLs are
  `<mirror-base>/<path>.{txt,zip}`.
- `game` ∈ `doom | doom2 | unknown`; `port` ∈ `vanilla | limit-removing | boom | mbf | mbf21 |
  zdoom | other | unknown`; the last integer is `txt_available` (0/1).
- No description, rating, votes, screenshots or txt body. `min_ddl_version` lets a future schema
  be rejected by older binaries, like the catalog bundle.
- **Confirmed estimate basis:** 10,885 entries with sampled title/author lengths took **2.04 MB**
  as JSON objects, **0.62 MB** grouped by directory as arrays, and **0.12–0.18 MB** gzipped. Adding
  date/flags keeps it well under a **2 MiB** plain-JSON cap; the cap should still be enforced.
  (Label: estimate, not a built artefact.)

### 4.2 Build and update cost

**Proposed first build:** 10,885 sidecars at 0.64 s sequential ≈ **1.9 h**; with two connections
per mirror ≈ 1 h, with four ≈ 30 min. A GitHub Actions job (6 h limit) can do it once with
`workflow_dispatch`; after that, an incremental build only fetches archives added or changed since
the last index. **Confirmed:** the archive adds ≈ 2–3 zips/day (71 in 30 days), so the daily job is
seconds to a minute, not an hour.

**Proposed politeness rules for the builder:** one mirror at a time, at most 2–4 requests in
flight, a descriptive UA with the project URL, `If-Modified-Since`/ETag on every txt, stop and keep
the previous index on 403/429/5xx, and never use the master (`www.gamers.org`) for the bulk pass —
the mirror list explicitly asks clients to use mirrors.

### 4.3 Publishing and consumption

**Proposed:** a dedicated orphan branch `idgames-index` (not the `catalog` branch, whose job triggers
only on `catalog/**` and whose serial semantics are per recipe release) with a single
`index.json`, built by `.github/workflows/idgames-index.yml` on a daily schedule plus
`workflow_dispatch`; `contents: write` only for that job; ordinary commits, never force-pushed, last
good index kept on failure. The app reads
`https://raw.githubusercontent.com/vittau/Doom-Deck-Launcher/idgames-index/index.json` — HTTPS,
already an allowlisted host, the same trust model as D-025 (no signature; signing is later
hardening).

**Proposed app behaviour:** fetch at most once per 24 h in the background (same shape as spec 019),
manual "Refresh archive list" in the browse screen, keep the previous index on any failure, and
render from cache first. The index is *not* part of the catalog bundle: it is 10× larger, changes
daily and must not invalidate recipes.

### 4.4 Terms

**Proposed:** the index is DDL's own derived data (facts: path, size, date, title, author, inferred
flags), rebuilt from public archive listings and author txt files; it republishes no txt body and no
description, credits the archive and the author, and links the canonical page. This needs owner
sign-off because it is a new public artefact derived from community content (open question 6). The
same robots/politeness rules as §2.5 apply to the crawler.

### 4.5 Failure modes

- CI crawl fails or is blocked → the published index stays at the last good build; the app shows its
  `generated_at` and keeps working offline.
- New uploads appear with up to a day's delay (the index snapshot is daily).
- One bad txt must not fail the build: skip the entry's title/author, keep the file with a filename
  fallback, and report a count in the build log.
- A schema change needs `schema` + `min_ddl_version`, exactly like the catalog bundle.
- The hosted index is a single point of failure; the app must degrade to "archive list unavailable"
  rather than an empty screen, and install/download of anything already known must keep working.
- No hashes: the index cannot verify downloads (see §7.2).

### 4.6 Verdict

**Proposed:** recommended. It combines the inventory (A) and the author metadata (B), costs one
small download per day per user, works offline after the first fetch, and keeps the blocked API out
of the product.

---

## 5. Comparison

| | A: `ls-laR.gz` only | B: `.txt` only | C: DDL index |
| --- | --- | --- | --- |
| fields | path, size, date | title, author, game, port evidence + size/date | all of A+B, derived |
| search by title/author | no | only by crawling | yes, local |
| per-user first-use cost | 419 KB | ≈ 35 MB, ≈ 2 h, 10 885 requests | ≈ 0.6–2 MB |
| freshness | daily | immediate | daily (CI) |
| offline browse after first use | yes (raw list, no titles) | no | yes |
| depends on Cloudflare-blocked API | no | no | no |
| hosting/maintenance | none | none | one CI branch + one job |
| licensing risk | lowest | author text kept locally | derived facts only; needs sign-off |
| failure blast radius | parse errors only | 10 885 requests per player | stale/broken index |

**Recommendation (Proposed): C for browse/search, B fetched on demand for the entry page, A as the
build input and as a fallback freshness check. No runtime use of the API and no per-user crawl.**

---

## 6. What "unverified" means (R2)

### 6.1 Status vocabulary

**Confirmed:** DDL's recipe model already has the right stage:
`VerificationStatus::{Indexed, Inspected, Verified, Partial, Incompatible}`
(`crates/ddl-core/src/catalog/model.rs`), and OVERVIEW §7 defines **Indexed** as *"Found in external
metadata; browse/details; launch configuration not guaranteed"*.

**Proposed:**

- An `/idgames` entry is **Indexed**. It never gets the catalog's verified badge, tester, tested
  engine version or compatibility claim.
- Player-facing label: **"Unverified"** chip plus one line: *"From the /idgames archive. DDL has not
  tested this mapset."* Do not use process jargon ("Needs Deck Validation") — spec 045 removed that
  from the player UI.
- The entry page may say **"Inspected"** only after DDL has actually opened the zip and recorded the
  member list/hashes locally; that is still not "Verified" and must not imply playability.
- The canonical page stays `/idgames` (`https://www.doomworld.com/idgames/<dir>/<stem>`) for
  attribution; the download goes to a mirror.

### 6.2 IWAD inference

**Proposed, evidence-based:**

1. **Archive path** is the primary signal: `levels/doom2/… → doom2`, `levels/doom/… → doom`
   (**Confirmed**, 100 % of the sample). `levels/heretic`, `hexen`, `strife`, `hacx`, `doom64`
   are out of scope for DDL's `Game` enum (`doom | doom2 | tnt | plutonia`) and should not be
   indexed in v1.
2. The txt `Game:` / `Episode and Level #:` corroborates (35/40). If it names **TNT** or
   **Plutonia**, require `tnt`/`plutonia` instead of `doom2` (rare; do not auto-download those
   IWADs, they are user-supplied like all IWADs).
3. If path and txt disagree, show **"IWAD unclear"** and let the player pick; never silently choose.
4. If the required IWAD is missing, Play is blocked and the UI says which IWAD and how to add it
   (D-010). The archive never contains IWADs, so this is always a user-supplied dependency.

**Needs Deck Validation:** that a `levels/doom2` entry labelled `doom2` actually launches with every
known DOOM II IWAD variant DDL accepts; sampling on the Deck is required before the tier ships.

### 6.3 Complevel and compatibility evidence

**Confirmed:** DSDA-Doom reads the `COMPLVL` lump when `-complevel` is omitted; if no lump wins, its
`default_compatibility_level` is **21 (MBF21)**, not "auto vanilla"
(`docs/research/dsda-engine.md` §1, upstream `prboom2/src/dsda/settings.c:60–170`).

**Proposed policy — evidence, never a silent numeric guess (and D-030's aliases):**

1. **COMPLVL present** in the downloaded WAD(s) → omit `-complevel`, show *"Compatibility declared
   by the WAD (COMPLVL)"*. This is the only authoritative signal a mirror gives us, and it requires
   the file; the launch spec must read the WAD lump directory after extraction.
2. **Author-declared port family** in the txt's `Advanced engine needed` field → map it exactly as
   **D-030** does for recipes: `vanilla`/`limit-removing` → `vanilla` (2 on Doom II, 3 on
   Doom/Ultimate), `boom` → `boom` (9), `mbf` → `mbf` (11), `mbf21` → `mbf21` (21). Pass the
   **alias word**; show it with the resolved number (e.g. *"Vanilla (2) · from the author's TXT"*)
   and the quoted field. D-030 makes this convention a decision, not a DDL guess, and it removed
   `complevel_status` — do not reintroduce a verification label.
3. **No family evidence** → omit `-complevel` and show *"Compatibility unknown — DSDA-Doom will use
   its default (MBF21) unless the WAD declares one"*. Offer an advanced override with D-030's
   aliases and numbers, clearly marked as a player choice. Do not print a complevel as if DDL knew
   it.
4. **Never** scrape free text for `complevel`: the sample contains a `-complevel 17` demo hint that
   is not the map's target, and a "complevel 2" mentioned inside a lighting note (Confirmed).
5. `Partial`/`Incompatible` are reserved for entries DDL has actually inspected and found broken,
   not for "no engine line".

**Needs Deck Validation:** that DSDA's default-21 launch is sane for a handful of unknown-metadata
WADs, and that each alias resolves and plays as expected with the tier's launch plan.

### 6.4 Port requirement gating

**Proposed mapping from the txt (sample evidence):**

| evidence | classification | UI / action |
| --- | --- | --- |
| `Advanced engine needed` names only DSDA-Doom, PrBoom+, Boom, MBF, MBF21, "limit removing", "vanilla", Crispy/Doom Retro | **try** | Download & Try enabled |
| `Advanced engine needed` names ZDoom/GZDoom/LZDoom/UZDoom/EDGE/Doomsday/Eternity/Skulltag/Zandronum | **other port required** (blocked) | Play disabled; show the quoted requirement; offer the source page |
| no engine line, or `None`/`Na` | **unknown** | Download & Try enabled with "compatibility unknown" |
| path under `levels/…/Ports/…` | **hint only**: likely a source-port mapset | never blocks on its own |
| txt `May Not Run With` names DSDA-Doom/PrBoom+ | **likely blocked** | block with the quote (author's own words) |
| txt `May Not Run With` names only vanilla/Chocolate | **port required, not DSDA-hostile** | allow Try, show the quote; do not treat as proof |

**Confirmed** examples: ZDoom-only `blkfortx`/`3dent`; Boom `abyspe65`; MBF21 `tistartan`. **Do
not** use `Tested With` as a gate (Confirmed: `abyspe65` lists GZDoom but is Boom).

**Proposed:** entries marked "other port required" stay **browsable** so discovery is broad, but
**Download & Play is disabled**. This is the M4 exit criterion — broad discovery without pretending
everything runs.

### 6.5 What the player sees

**Proposed entry page:**

- **Shown:** title, author, `YYYY-MM-DD` archive date, size (human-readable), canonical /idgames
  link, author's permission/credits text (fetched on demand), base game and whether the player has
  it, the engine requirement quote, the "Unverified" chip, the download source/mirror.
- **Not shown:** rating, votes, review count, screenshots (unless the txt/zip itself ships one — it
  does not as a rule), recommended skill, recommended complevel, add-ons/components, "verified" or
  "Deck tested" badge, curated tags.
- **Disabled:** Install & Play for known-incompatible entries; pistol-start (always off and not
  offered — DDL has not verified it); profile presets/skill recommendations; "Continue" until the
  first launch.
- **Warning sheet before the first download:** *"This mapset is not reviewed. DDL will download it
  from the /idgames archive, record its hash, extract only WAD/DEH/TXT files, and launch it with
  unknown compatibility. Files are never executed."* with Download / Cancel.
- **After install:** it appears in Library with the same "Unverified" chip, separate from the
  verified catalog; a later catalog recipe with the same path can offer *"Now verified by DDL —
  switch to the reviewed recipe"* (Proposed).

**Needs Deck Validation:** that the warning copy and the disabled states are understandable on a
1280×800 screen at couch distance (headless renders first, then the owner's Deck).

---

## 7. Safety when no hash is pinned (R3)

### 7.1 Download

**Proposed:**

- URL = `<allowlisted mirror base>/<path>.zip`, size from the index used as `expected_size`
  (spec 006 R1); hard cap **512 MiB** (max observed 426.7 MB, §2.3), warning above 100 MB.
- At most 2–3 mirror bases, tried on network/5xx failure; never on a successful download. Each
  redirect target re-checked against `catalog/hosts.toml`, with the exact-base-path table of §2.6.
- `Content-Length` must equal the index size; a mismatch aborts before reading the body where the
  server provides it, otherwise the streamed byte count is checked at the end.
- Same pipeline as spec 006: HTTPS only, timeouts, progress, cancel, temp file, atomic move,
  SHA-256 while streaming, cache keyed by hash.

### 7.2 Trust-on-first-download (TOFU)

**Proposed:**

- Identity key: `<path>` + index size + archive date. The page shows the canonical page and the
  mirror URL; the index date/size are displayed as "as listed on the archive".
- First successful download: compute SHA-256, store it with the entry and the source URL, show
  **"Hash recorded (first download) — not verified against the author"**. It is useful for
  detecting later changes, not for proving authenticity.
- Subsequent downloads: if size/date are unchanged, require the stored SHA-256; a mismatch aborts,
  keeps the existing install and reports both hashes. If the archive date/size changed, treat it as
  a new archive version and ask before replacing (never silently overwrite a working install).
- Mirror disagreement on a first download (two mirrors, different hashes) aborts and reports; the
  player can retry later. With a stored hash, the mirror that matches wins and the other is
  quarantined.
- Do not treat TOFU as "Verified". The install manifest records it as `unverified` provenance with
  the hash and date.

### 7.3 Extraction and what gets loaded

**Proposed, on top of spec 006 R4–R5 (which already reject traversal, absolute paths, symlinks,
zip bombs and unknown methods):**

- List the zip central directory first (no extraction) and select members by extension:
  - `.wad` (PWAD): candidate maps. If one → main. If several → the largest is the main candidate;
    the rest are optional toggles, **off by default**, never auto-loaded. If zero → incompatible
    ("no WAD in the archive").
  - `.deh`/`.bex`: passed as `-deh`, ordered by name; optional ones off unless the txt names them.
  - `.txt`/`.nfo`/`.doc`: kept with the install as the credits/permission file; never loaded.
  - everything else (`.exe`, `.bat`, `.sh`, `.dll`, `.pk3`, `.lmp`, …): **not extracted** (or
    listed as unsupported). No executable bit is ever set; nothing is executed (spec 006 R5).
- Enforce the spec 006 limits (member count, per-member size, total uncompressed size, compression
  ratio, nesting depth) because the archive is untrusted even though it is curated.
- After extraction, hash each kept member and store it in the install manifest; this is the "what
  did DDL actually run" record.
- Multi-WAD order is the biggest unverified risk: show the proposed load order in the confirm
  sheet and require confirmation when there is more than one WAD or any DEH.
- **Never** load a WAD just because it is in the zip (OVERVIEW §7: "never auto-load every file from
  an archive").

### 7.4 Promotion to a catalog recipe by PR

**Proposed:**

1. DDL can emit a **recipe draft** for an indexed entry: `ddl idgames draft <path>` (desktop CLI)
   writes a schema-valid `catalog/<id>.toml` with the computed package size/SHA-256, `idgames_path`,
   the chosen members and their hashes, the inferred IWAD and the evidence quotes;
   `verification.status = "indexed"`, `date` = build date, `engine_version` = the supported DSDA
   version, `tester = "idgames import (automated)"` and `notes` saying what is automated and
   unverified (the lint requires a tester, a date, a semver engine version and non-empty notes —
   `crates/ddl-core/src/catalog/lint.rs`). The UI must not present that automated string as a human
   tester.
2. The contributor tests on a Deck, fills `tester`, `date`, `engine_version`, `limitations` and the
   complevel evidence, and opens a PR touching only that file (D-010).
3. CI lints it (spec 002); a maintainer reviews the evidence and promotes it to `inspected` or
   `verified`. Only `verified` gets Install & Play from the catalog.
4. The browse entry can show **"Has a DDL recipe"** by matching its path against the installed
   catalog, so a promoted mapset does not appear twice with different labels.
5. The draft must never be auto-merged or auto-published: promotion is a human-reviewed PR, exactly
   like every other recipe.

**Needs Deck Validation:** a full promotion dry-run for one entry (e.g. a small Boom mapset) from
browse → draft → PR → recipe, including the computed hashes.

---

## 8. UX sketch in words (R4)

### 8.1 Placement

**Proposed:** keep the four root tabs (Home, Discover, Library, Settings — spec 008) and put the
archive behind a section at the bottom of **Discover**: a card *"Browse the /idgames archive —
unreviewed, thousands of mapsets"* with a **Browse** action. It opens a full-screen screen pushed
over Discover (tab bar disabled, B = Back, like Mapset detail per spec 030). No fifth tab: an
unverified tier must not look like a peer of the curated catalog.

### 8.2 Search and filters (controller only)

- **Y = Search** opens the Steam on-screen keyboard (spec 031 R3; the `steam://open/keyboard`
  helper is **Needs Deck Validation**); if it cannot open, the field still accepts input from a
  physical keyboard and shows a one-line hint.
- Search is **local over the cached index** (no network per keystroke): case- and
  diacritic-insensitive substring over title, author and filename, minimum 2 characters.
- **Filters that need no typing:** base game (Doom / Doom II), era (1994–1999, 2000–2009,
  2010–2019, 2020–now), "runs on DSDA-Doom" (hides other-port-required), and sort
  (newest, title, size). These are the controller-friendly path; typing is the exception.
- Show *"<n> results · archive list from <date>"*; a stale index is labelled, not hidden.

### 8.3 Paging and focus

- 50 entries per page, L1/R1 (or the D-pad at the list edge) for previous/next page; a "Load more"
  action is also focusable. L1/R1 are free here because the tab bar is disabled on this screen
  (spec 030). The list is virtualised; 10 885 rows are never laid out at once.
- Focus is restored to the same row when coming back from an entry, and to the search field after a
  query.
- Empty/error states: no index yet ("Connect once to load the archive list" + Retry); no results
  ("No matches — try fewer words"); index older than 14 days ("This list may be out of date").

### 8.4 Offline

- With a cached index: browse, search, filters and the entry's basic page work offline; the txt
  detail shows if it was viewed before, otherwise "Connect to load the author's text".
- Download/Play require the engine and the archive host; if the engine is missing, the existing
  first-run flow takes over. If the WAD is already installed, Play works offline (the install
  manifest is local).
- The index refresh is silent on failure and never blocks the UI.

### 8.5 Entry page and copy

Layout mirrors a catalog card/detail but with muted art and no badge row: title, author, date,
size, source, base game, engine evidence, warning. Primary action **"Download & Try"**; secondary
**"Open /idgames page"** (external; Gaming Mode behaviour **Needs Deck Validation**). The warning
sheet is the only place that uses stronger language, and it must name the archive as the source.

**Needs Deck Validation:** OSK invocation, focus return after the keyboard, readability at
1280×800, and whether an external browser link is usable in Gaming Mode.

---

## 9. Open questions

The owner-facing list with recommendations is in
`docs/reports/041-idgames-browse-research.md` §"Open questions". In short: whether to publish a
derived index at all and on which branch; whether to scope to 10 885 DOOM-family single-player
entries; whether "other port required" entries may still be downloaded; the default-21 launch
behaviour; crawl permission/politeness; legal sign-off for derived metadata; player-facing wording;
and the allowlist/base-path changes (`idgames.planetzdoom.com`, per-host base paths, dropping
`files.xvertigox.com` from rotation).

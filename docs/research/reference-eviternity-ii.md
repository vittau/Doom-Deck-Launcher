# Reference research — Eviternity II (Dragonfly et al)

**Status:** desk research for a D-010 catalog recipe · **Gathered:** 2026-10-16 (UTC)
**Author workstream:** Content Catalog (catalog addition)
**Scope:** facts needed to author a DDL recipe for **Eviternity II**. No code is produced here.

Labels used throughout:

- **Confirmed** — verified from an official source, upstream source code, or a downloaded archive
  (evidence recorded below).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

The canonical Doomworld page and the authors' release thread were **not readable** from this host:
Doomworld returns Cloudflare HTTP 403 ("Just a moment…") to non-browser clients, the same
limitation recorded in `upstream-sources.md` §4.2; the Internet Archive was temporarily offline
during this session, so no Wayback snapshot was available. Facts therefore come from the files the
authors shipped with the release (the accompanying TXT, credits and changelog, read without
executing anything) and from the direct `/idgames` mirror path.

Archives were downloaded to a scratch directory (never committed), hashed with `shasum -a 256`,
inspected with Python `zipfile` and a read-only WAD lump reader; nothing was executed.

```sh
curl -sSL -A "DoomDeckLauncher catalog review" \
  -o eviternityii.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/eviternityii.zip"
shasum -a 256 eviternityii.zip
```

## Quick reference

| | Eviternity II |
| --- | --- |
| Release | **Final version, 2024-09-30** (initial release 2023-12-10) |
| Author | Joshua "Dragonfly" O'Sullivan et al (see credits) |
| Archive | `levels/doom2/Ports/megawads/eviternityii.zip` |
| Archive size / SHA-256 | 121,166,110 bytes / `0fb738712c9007e8c238b6a88fb57842148ef3d6036a17f9a82c9949f131e9c9` |
| Main WAD | `Eviternity II.wad` (MAP01–MAP37) |
| IWAD | `doom2.wad` |
| Format / complevel | **MBF21 → complevel 21** (the WAD's own `COMPLVL` lump says `mbf21`) |
| Embedded patch | one MBF21 `DEHACKED` lump (1,439,098 bytes, DECOHack-authored) |
| `COMPLVL` lump | **present: `mbf21`** |
| Other lumps | `UMAPINFO` (episodes, 11,348 B); ZDoom-only `MAPINFO`/`ZMAPINFO`/`GAMEINFO` (inert) |
| Optional components | none in the official package |
| Licence | No reuse as a base/resource without written consent; redistribution via /idgames only |

---

## 1. Official sources

- **Canonical `/idgames` page:**
  <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/eviternityii> — filename
  `levels/doom2/Ports/megawads/eviternityii.zip`; the archive listing dates the file
  **2024-11-24**. Title "Eviternity II" (Final version). **Confirmed (URL is canonical; the page
  itself was 403 from this host — see Method).**
- **Archive directory listing (fetched 2026-10-16):**
  <https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/> — `eviternityii.txt` and
  `eviternityii.zip`, both dated `2024-11-24 13:49` (116 MB). **Confirmed**
- **Release text (`eviternityii.txt`, inside the archive):** `Advanced engine needed : DSDA-Doom
  or GZDoom. Use latest versions.`; `Release date : September 30th 2024` (initial release
  December 10th, 2023); `Author : Dragonfly et al`; `Game : Doom 2`; `Map # : MAP01 - MAP37`;
  `New levels : 37`; `Dehacked/BEX Patch : Yes`; `Tested With : DSDA-Doom v0.27.4 and above.
  GZDoom v4.11.3 and above.` **Confirmed (quoted).**
- **Credits (`Credits.txt`) and changelog (`EVITERNITY II - Final Version - CHANGELOG.txt`)**
  ship inside the archive; the changelog is titled "EVITERNITY II - Final CHANGELOG". **Confirmed**

## 2. Canonical archive and byte-identical mirrors

The `/idgames` path is `levels/doom2/Ports/megawads/eviternityii.zip`; every mirror serves that
`<dir>/<filename>` under its own base (`upstream-sources.md` §3.4). DDL uses direct mirror URLs,
never the `/idgames` API (D-015). Three allowlisted hosts were checked and **downloaded in full**
(121 MB each); all three serve the exact same bytes:

| Host (allowlisted) | URL | bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/eviternityii.zip` | 121,166,110 | `0fb73871…31e9c9` |
| `youfailit.net` | `https://youfailit.net/pub/idgames/levels/doom2/Ports/megawads/eviternityii.zip` | 121,166,110 | `0fb73871…31e9c9` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/Ports/megawads/eviternityii.zip` | 121,166,110 | `0fb73871…31e9c9` |
| `planetzdoom.com` (extra check) | `https://planetzdoom.com/idgames/levels/doom2/Ports/megawads/eviternityii.zip` | 121,166,110 | `0fb73871…31e9c9` |

**Confirmed:** all four are byte-identical (same length and same SHA-256 computed here), so they
may share one package record (D-015). The recipe lists the master plus two mirrors.

## 3. Archive contents (ZIP, 121,166,110 bytes)

Everything is listed as a member with its exact size and SHA-256 computed here. Nothing else is in
the archive; there is no separate music pack or alternate WAD.

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `Eviternity II.wad` | **293,081,691** | `3127fbc96579960c2fcadb22b09bdd50693357d864674ef5ea271120f461759a` | maps |
| `Credits.txt` | 8,248 | `51bbdeb3bb74f9384d90b59a48bb4ae786686923906f5e6ef2cabb44ae90dead` | credits (keep) |
| `EVITERNITY II - Final Version - CHANGELOG.txt` | 44,358 | `67af7bdde67c1358d80ee4301e9ba51e56603fca72a2c028fc3f6b39a0df5d34` | changelog (keep) |
| `eviternityii.txt` | 6,477 | `977ede13d7558fa3e848f708ce57ed59d4512d8d3bd0889d24e75295b29cfeb3` | readme/credits (keep) |

**Confirmed.**

### Size cap (DDL change accompanying this recipe)

The WAD member is **293,081,691 bytes (≈279.5 MiB)**, over the former 256 MiB catalog cap. The cap
was raised to **512 MiB** in two constants that are documented to stay in sync — `MAX_BYTES` in
`crates/ddl-core/src/catalog/lint.rs` (package + member validation and the `--fetch` aggregate
decompression budget in `catalog/fetch.rs`) and `MAX_MEMBER_BYTES` in
`crates/ddl-core/src/library/install.rs` (import, download-request and installed-member bounds).
Extraction was already bounded by `downloads::extract::ExtractLimits` (2 GiB per member, 4 GiB
total), so no change was needed there. **Proposed (DDL policy):** the cap is only an abuse /
zip-bomb bound; a member's exact **size + SHA-256** remain the integrity check, so raising it does
not weaken verification.

## 4. WAD inspection (read-only, no execution)

`Eviternity II.wad` is a **PWAD**, 293,081,691 bytes, **9,460 lumps**, with 37 `MAPxx` markers
(MAP01–MAP37) and `DEMO1`–`DEMO4`. **Confirmed.**

- **`COMPLVL` lump present (5 bytes): `mbf21`.** DSDA-Doom recognises exactly this word and maps it
  to complevel 21
  ([`prboom2/src/w_wad.c:360–418`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/w_wad.c#L360-L418)).
  So the authors themselves declare **MBF21 / complevel 21**. **Confirmed**
- **`UMAPINFO` lump present (11,348 bytes)** with `episode = "M_EPI1", "Withered", "1"` and per-map
  entries. DSDA-Doom v0.30.1 implements UMAPINFO
  ([`prboom2/src/dsda/mapinfo/u.c:544–546`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/mapinfo/u.c#L544-L546)
  iterates the `UMAPINFO` lump and calls `ParseUMapInfo`; the `episode` key is handled at
  [`prboom2/src/umapinfo.cpp:331`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/umapinfo.cpp#L331)),
  so the chapter episodes appear in the engine's own episode menu. **Confirmed**
- **One embedded MBF21 `DEHACKED` lump (1,439,098 bytes)** beginning
  `Patch File for DeHackEd v3.0 … Created with DECOHack v0.31.0 … Doom version = 2021 / Patch
  format = 6`. **Confirmed**
- ZDoom-only lumps (`ZMAPINFO` 20,000 B, `MAPINFO` 16,414 B, `GAMEINFO` 110 B, plus `DECORATE`/
  `ZSCRIPT`) are present but inert under DSDA-Doom: ZDoom `MAPINFO` support was removed in
  v0.30.0. **Confirmed**

## 5. DSDA-Doom v0.30.1 support (pinned tag)

- **MBF21 (complevel 21) is a supported compatibility level.** The enum maps `21` to MBF21
  ([`prboom2/src/doomtype.h:113–140`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomtype.h#L113-L140),
  [`prboom2/src/dsda/settings.c:115–155`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L115-L155)),
  and the accepted word `mbf21` resolves to 21
  ([`prboom2/src/dsda/settings.c:60–170`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L60-L170)).
  **Confirmed**
- **The recipe passes `-complevel 21` explicitly** (D-011); even without it, DSDA would read the
  winning `COMPLVL` lump (`mbf21`) when not playing a demo
  ([`prboom2/src/w_wad.c:360–418`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/w_wad.c#L360-L418)).
  Passing it explicitly makes the launch reproducible and matches DDL's contract. **Confirmed**
- **The authors name DSDA-Doom as a first-class target** (`Advanced engine needed : DSDA-Doom or
  GZDoom`; `Tested With : DSDA-Doom v0.27.4 and above`), so v0.30.1 is inside the tested window.
  **Confirmed (quoted).**

### What DSDA-Doom v0.30.1 does with the embedded DEHACKED under DDL's `-noautoload`

DDL launches with `-noautoload` (D-011) to skip **directory** autoload
([`prboom2/src/dsda/args.c:491–505`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L491-L505),
[`prboom2/src/d_main.c:2059–2070`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2059-L2070)).
`-noautoload` does **not** disable DEHACKED-in-a-WAD: `d_main.c` processes `DEHACKED` lumps from
explicitly loaded PWADs whenever `-nodeh` is absent, independent of the autoload flag
([`prboom2/src/d_main.c:2147–2156`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2147-L2156)).
So a launch `… -iwad doom2.wad -file …/Eviternity II.wad -complevel 21 -noautoload` applies the
embedded MBF21 patch automatically, and the recipe must **not** add a separate `-deh` entry.
**Confirmed** (upstream source at the pinned tag; `dsda-engine.md` §1 and `reference-eviternity.md`
§4 state the same rule).

## 6. Licence and redistribution

The release TXT states (**Confirmed (quoted)**):

```text
* Copyright / Permissions *

Without prior written consent, you may NOT use any of this WAD or it's
assets as a base or resource for your own projects.

If you wish to use any data found within Eviternity II, please contact
Dragonfly first, who will be able to advise you of the permissions
surrounding each asset as required.
```

The text does not restate the CC BY-NC-ND terms of the original Eviternity; it is strictly a
**no-reuse** clause. The file is nonetheless distributed through `/idgames`, whose archive rules
require uploaded files to be freely redistributable (`upstream-sources.md` §4.2). **Proposed (DDL
policy):** DDL never modifies or repackages the archive; it downloads the original ZIP from its
canonical `/idgames` mirror, keeps the readme, credits and changelog (`keep = true`) unmodified,
and makes no claim of a reuse licence.

## 7. Composition (recipe data — matches `catalog/eviternity-ii.toml`)

- One package `free` (the ZIP above), one required `main` component loading `Eviternity II.wad` as
  `maps`. No optional components: the official package ships nothing else.
- Keep `eviternityii.txt`, `Credits.txt` and the final changelog as `keep = true`.
- `base_game = doom2` (no variant restriction); `compatibility.family = "mbf21"`,
  `complevel = 21`; engine `=0.30.1`, `inspected`.
- **`complevel_status = "proposed"`.** The *value* is confirmed by the WAD's own `COMPLVL` lump
  and by the authors' port statement, but DDL's convention (`CONTRIBUTING.md`, "Complevel
  proposals stay proposed until tested") keeps it Proposed until a Deck run; that run also covers
  the UMAPINFO episode menu and the MBF21 patch.

## 8. Open questions

- **Complevel is author-declared, not yet Deck-tested.** The `COMPLVL` lump makes 21 the correct
  value; whether the episode menu, MBF21 behaviour and audio are right on the Deck is
  **Needs Deck Validation**.
- **Licence reading.** The no-reuse clause is stricter than the original Eviternity's CC BY-NC-ND;
  DDL's redistribute-from-source model relies on the author's `/idgames` upload. Flagged for the
  owner, not changed here.
- **Canonical page/thread unread.** The Doomworld entry and release thread were Cloudflare-blocked
  from this host; a contributor with browser access could confirm there is no newer official build.
- **Memory during `--fetch`.** The lint reads the single 279.5 MiB member into memory (bounded by
  the 512 MiB budget); fine for a CLI check, to be kept in mind if lint ever runs under a tight
  memory cap.


## Screenshots (spec 023, D-024)

Bundled under `crates/ddl/ui/assets/screenshots/eviternity-ii/`; the `SOURCES.toml` beside the
images records each file's source URL, source page, credit, original size, retrieval date
and any permission text found. Retrieved 2026-10-17; each file is a 640x360 JPEG card crop
of the original below.

Official promotional screenshots on the mapset's own site `eviternity.dfdoom.com`, reachable directly.

| Asset | Source image URL | Source page | Credit | Original |
| --- | --- | --- | --- | --- |
| `01.jpg` | <https://eviternity.dfdoom.com/wp-content/uploads/2023/12/promo-map04.jpg> | <https://eviternity.dfdoom.com/> | Eviternity II team | 1050x1050 (`promo-map04.jpg`) |

**Why representative:** The `promo-map04` and `promo-map19` shots are in-game views of the mapset's chapters, published by the team as the representative screens.

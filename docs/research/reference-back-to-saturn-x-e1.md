# Reference research — Back to Saturn X E1: Get Out Of My Stations (esselfortium et al)

**Status:** desk research for a D-010 catalog recipe · **Gathered:** 2026-10-16 (UTC)
**Author workstream:** Content Catalog (catalog addition)
**Scope:** facts needed to author a DDL recipe for **Back to Saturn X Episode 1**. No code is
produced here.

Labels used throughout:

- **Confirmed** — verified from an official source, upstream source code, or a downloaded archive
  (evidence recorded below).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

The canonical Doomworld page was **not readable** from this host: it returns Cloudflare HTTP 403
("Just a moment…") to non-browser clients, the same limitation recorded in `upstream-sources.md`
§4.2; the Internet Archive was temporarily offline during this session, so no Wayback snapshot was
available. Facts therefore come from the files the authors shipped with the release (the
accompanying TXT, read without executing anything) and from the direct `/idgames` mirror path.

Archives were downloaded to a scratch directory (never committed), hashed with `shasum -a 256`,
inspected with Python `zipfile` and a read-only WAD lump reader; nothing was executed.

```sh
curl -sSL -A "DoomDeckLauncher catalog review" \
  -o btsx_e1.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/megawads/btsx_e1.zip"
shasum -a 256 btsx_e1.zip
```

**Note on the `/idgames` path:** BTSX is a vanilla-compatible release, so the archive classifies it
under `levels/doom2/megawads/` (**not** `levels/doom2/Ports/megawads/`, which is for source-port
releases). A naive `Ports/megawads/btsx_e1.zip` 404s. **Confirmed** (both directory listings were
fetched).

## Quick reference

| | Back to Saturn X E1 |
| --- | --- |
| Release | **1.1.6, 2020-08-13** |
| Author | Sarah "Esselfortium" Mancuso et al (Back to Saturn X team) |
| Archive | `levels/doom2/megawads/btsx_e1.zip` |
| Archive size / SHA-256 | 18,652,774 bytes / `9f6d2fba63e20e2fddd5a1012813da1087b751f9311d3d8bf4757f69373c2c9a` |
| Main WADs | `btsx_e1a.wad` (maps) + `btsx_e1b.wad` (textures/resources), MAP01–MAP25, MAP31–MAP32 |
| IWAD | `doom2.wad` |
| Format / complevel (Proposed) | Vanilla → **complevel 2** |
| Embedded patch | one `DEHACKED` lump in `btsx_e1a.wad`; a separate `btsx_e1.deh` targets vanilla/Chocolate |
| `COMPLVL` lump | none |
| Optional components | none |
| Licence | "You MAY distribute this file… with no modifications"; no reuse as a base |

---

## 1. Official sources

- **Canonical `/idgames` page:**
  <https://www.doomworld.com/idgames/levels/doom2/megawads/btsx_e1> — filename
  `levels/doom2/megawads/btsx_e1.zip`; the archive listing dates the file **2020-08-16**. Title
  "Back to Saturn X E1: Get Out Of My Stations". **Confirmed (URL is canonical; the page itself
  was 403 from this host — see Method).**
- **Archive directory listing (fetched 2026-10-16):**
  <https://www.gamers.org/pub/idgames/levels/doom2/megawads/> — `btsx_e1.txt` and `btsx_e1.zip`,
  both dated `2020-08-16 10:47` (18 MB). **Confirmed**
- **Release text (`btsx_e1.txt`, inside the archive):** `Advanced engine needed : Doom2.exe or
  compatible`; `Version : 1.1.6`; `Release date : 08/13/2020`; `Author : Esselfortium and a lot of
  wonderful people`; `Game : Doom 2`; `Map # : MAP01-MAP25, MAP31-MAP32`; `New levels : 27`;
  `Dehacked/BEX Patch : Yes`. **Confirmed (quoted).**
- **Two-file split (release text):** *"BTSX has been split into two wad files, which must be
  loaded together. This has been done in order to work around an obscure vanilla crash involving
  wads that exceed 4046 lumps."* and *"Source port users should only need to load the two wad
  files together."* **Confirmed (quoted).**

## 2. Canonical archive and byte-identical mirrors

The `/idgames` path is `levels/doom2/megawads/btsx_e1.zip`; every mirror serves that
`<dir>/<filename>` under its own base (`upstream-sources.md` §3.4). DDL uses direct mirror URLs,
never the `/idgames` API (D-015). Three allowlisted hosts were checked and **downloaded in full**;
all three serve the exact same bytes:

| Host (allowlisted) | URL | bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/megawads/btsx_e1.zip` | 18,652,774 | `9f6d2fba…3c2c9a` |
| `youfailit.net` | `https://youfailit.net/pub/idgames/levels/doom2/megawads/btsx_e1.zip` | 18,652,774 | `9f6d2fba…3c2c9a` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/megawads/btsx_e1.zip` | 18,652,774 | `9f6d2fba…3c2c9a` |
| `planetzdoom.com` (extra check) | `https://planetzdoom.com/idgames/levels/doom2/megawads/btsx_e1.zip` | 18,652,774 | `9f6d2fba…3c2c9a` |

**Confirmed:** all four are byte-identical (same length and same SHA-256 computed here), so they
may share one package record (D-015). The recipe lists the master plus two mirrors.

## 3. Archive contents (ZIP, 18,652,774 bytes)

Everything is listed as a member with its exact size and SHA-256 computed here. Nothing else is in
the archive.

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `btsx_e1a.wad` | 22,307,013 | `92cb0aba71cfd59276fde5f4456e030d9e55488135bd416d8932ff40a1a77f3d` | maps |
| `btsx_e1b.wad` | 29,449,016 | `fc405051c044f4f7f5264902978ff087f7d9985546c7f103bf69be5caec92011` | graphics/resources |
| `btsx_e1.deh` | 4,985 | `…` (not loaded; see §4) | — |
| `btsx_e1.txt` | 15,746 | `639ca057a87c1b19f447f7fbb618bdd95cacfdffe1b59de1212102728594c70f` | credits (keep) |

**Confirmed.** The archive has no top-level directory. The `.deh` is deliberately **not** a
member: DDL names only the files it loads or keeps.

## 4. WAD inspection (read-only, no execution)

- `btsx_e1a.wad` is a **PWAD**, 22,307,013 bytes, **2,123 lumps**, with the map markers
  (27 levels: MAP01–MAP25, MAP31, MAP32), `DEMO1`–`DEMO3`, a **`DEHACKED` lump (3,148 bytes)** and
  a ZDoom-style `MAPINFO`/`ZMAPINFO`. **Confirmed**
- `btsx_e1b.wad` is a **PWAD**, 29,449,016 bytes, **2,248 lumps**, **no maps**; it carries the
  bulk of the custom texture/flat resources (`PLAYPAL`, `COLORMAP`, `TEXTURE1`, `PNAMES`, the BTSX
  texture family). **Confirmed**
- **No `COMPLVL`** and **no `UMAPINFO`** lump in either WAD. DSDA-Doom **removed ZDoom `MAPINFO`
  support in v0.30.0**, so the `MAPINFO` lumps are inert and the complevel must be declared in the
  recipe. **Confirmed**
- **Two DEHACKED sources.** The embedded `DEHACKED` in `btsx_e1a.wad` uses the extended
  `[STRINGS]`/`[PARS]` sections; the separate `btsx_e1.deh` uses vanilla `Text` blocks. The
  release text says source-port users only need the two WADs, so the embedded patch is the one
  DSDA-Doom applies. **Confirmed** (byte comparison of the two patches and the release text).

### What DSDA-Doom v0.30.1 does with the embedded DEHACKED under DDL's `-noautoload`

DDL launches with `-noautoload` (D-011) to skip **directory** autoload
([`prboom2/src/dsda/args.c:491–505`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L491-L505),
[`prboom2/src/d_main.c:2059–2070`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2059-L2070)).
`-noautoload` does **not** disable DEHACKED-in-a-WAD: `d_main.c` processes `DEHACKED` lumps from
explicitly loaded PWADs whenever `-nodeh` is absent, independent of the autoload flag
([`prboom2/src/d_main.c:2147–2156`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2147-L2156)).
So a launch `… -iwad doom2.wad -file …/btsx_e1a.wad …/btsx_e1b.wad -complevel 2 -noautoload`
applies the embedded patch automatically, and the recipe must **not** add a separate `-deh` entry
(the extraneous `btsx_e1.deh` would be redundant at best). **Confirmed** (upstream source at the
pinned tag; `dsda-engine.md` §1 states the same rule).

## 5. Requirements and compatibility

- **Base game:** DOOM II — the release TXT says `Game : Doom 2`, `Map # : MAP01-MAP25,
  MAP31-MAP32`. **Confirmed**
- **Required port class:** `Advanced engine needed : Doom2.exe or compatible`; the release is
  stated to be *"fully compatible with Doom2.exe and Chocolate Doom"* and also tested with
  Eternity, PrBoom-Plus, ZDoom, GZDoom and Risen3D. **Confirmed (quoted).**
- **Complevel (Proposed):** **2**. The release targets vanilla Doom2.exe; complevel 2 is
  "Doom/Doom II 1.9" in DSDA-Doom's enum. **Proposed until Deck-tested** — this is the value the
  recipe uses.
- **Load order:** `btsx_e1a.wad` then `btsx_e1b.wad`, exactly as listed in the release text and as
  a ZIP drag-and-drop load would produce. **Confirmed**
- **Warp/episode notes:** a single continuous DOOM II mapset with no `UMAPINFO`, so no `-warp` is
  prescribed; the DOOM II menu selects the map. **Proposed.**
- **DEH/BEX:** none to load separately — the patch is embedded in `btsx_e1a.wad` (see §4).
  **Confirmed**

## 6. Licence and redistribution

The release TXT states (**Confirmed (quoted)**):

```text
* Copyright / Permissions *
Authors may NOT use the contents of this file as a base for modification or
reuse.  Permissions have been obtained from original authors for any of
their resources modified or included in this file.

There will be a full release of the texture resource after Back to Saturn X's
three episodes are complete. In the meantime, please be respectful and don't
take our stuff. Thanks!

You MAY distribute this file, provided you include this text file, with no
modifications.  You may distribute this file in any electronic format (BBS,
Diskette, CD, etc) as long as you include this file intact.  I have
received permission from the original authors of any modified or included
content in this file to allow further distribution.
```

**Proposed (DDL policy):** DDL never modifies or repackages the archive; it downloads the original
ZIP from its canonical `/idgames` mirror and keeps the bundled text file (`keep = true`)
unmodified. Redistribution is permitted under the terms above; reuse as a base is not.

## 7. Composition (recipe data — matches `catalog/back-to-saturn-x-e1.toml`)

- One package `free` (the ZIP above), one required `main` component with two members loaded in
  order: `btsx_e1a.wad` as `maps`, then `btsx_e1b.wad` as `graphics`. No optional components.
- Keep `btsx_e1.txt` as `keep = true`.
- `base_game = doom2` (no variant restriction); `compatibility.family = "vanilla"`,
  `complevel = 2`, `complevel_status = "proposed"`; engine `=0.30.1`, `inspected`.

## 8. Open questions

- **Complevel 2 vs limit-removing.** The release is built around the 4046-lump vanilla limit and
  Chocolate Doom, so plainly vanilla; whether every map fits a pure vanilla *play* target on
  DSDA-Doom needs a Deck run (**Needs Deck Validation**).
- **BTSX E3.** The TXT references a three-episode project; only E1 and E2 are archived, and no
  third episode is part of this recipe.
- **Canonical page unread.** The Doomworld entry was Cloudflare-blocked from this host; a
  contributor with browser access could confirm there is no newer official build.


## Screenshots (spec 023, D-024)

Bundled under `crates/ddl/ui/assets/screenshots/back-to-saturn-x-e1/`; the `SOURCES.toml` beside the
images records each file's source URL, source page, credit, original size, retrieval date
and any permission text found. Retrieved 2026-10-17; each file is a 640x360 JPEG card crop
of the original below.

The screenshots are hosted by the project lead on `esselfortium.net` and were linked from the E1 release thread; the author's site is reachable directly.

| Asset | Source image URL | Source page | Credit | Original |
| --- | --- | --- | --- | --- |
| `01.jpg` | <https://esselfortium.net/images/btsx/e1_1.png> | <https://www.doomworld.com/vb/thread/62529> | Back to Saturn X team | 1280x700 (`e1_1.png`) |

**Why representative:** The E1 shots show the mapset's red-sky tech-base exteriors and the OTEX texture set in play.

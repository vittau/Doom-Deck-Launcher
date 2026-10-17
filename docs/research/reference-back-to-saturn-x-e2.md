# Reference research — Back to Saturn X E2: Tower in the Fountain of Sparks (esselfortium et al)

**Status:** desk research for a D-010 catalog recipe · **Gathered:** 2026-10-16 (UTC)
**Author workstream:** Content Catalog (catalog addition)
**Scope:** facts needed to author a DDL recipe for **Back to Saturn X Episode 2**. No code is
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
accompanying TXT and the nested patch README, read without executing anything) and from the direct
`/idgames` mirror path.

Archives were downloaded to a scratch directory (never committed), hashed with `shasum -a 256`,
inspected with Python `zipfile` and a read-only WAD lump reader; nothing was executed.

```sh
curl -sSL -A "DoomDeckLauncher catalog review" \
  -o btsx_e2.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/megawads/btsx_e2.zip"
shasum -a 256 btsx_e2.zip
```

**Note on the `/idgames` path:** BTSX is a vanilla-compatible release, so the archive classifies it
under `levels/doom2/megawads/` (**not** `levels/doom2/Ports/megawads/`). **Confirmed** (directory
listing fetched).

## Quick reference

| | Back to Saturn X E2 |
| --- | --- |
| Release | **1.0.1, 2020-08-13** |
| Author | Sarah "Esselfortium" Mancuso et al (Back to Saturn X team) |
| Archive | `levels/doom2/megawads/btsx_e2.zip` |
| Archive size / SHA-256 | 27,988,469 bytes / `2365b9ff5884027523553519830884cc85106c483fdf19c0f2bbe75155b60331` |
| Main WADs | `btsx_e2a.wad` (maps) + `btsx_e2b.wad` (textures/music/resources), MAP01–MAP27, MAP31 |
| IWAD | `doom2.wad` |
| Format / complevel (Proposed) | Vanilla → **complevel 2** (authors state this explicitly) |
| Embedded patch | one `DEHACKED` lump in `btsx_e2a.wad`; a separate `btsx_e2.deh` targets vanilla/Chocolate |
| `COMPLVL` lump | none |
| Official optional component | `map20_for_old_ports.zip` (not offered; see §5) |
| Licence | "You MAY distribute this file… with no modifications"; no reuse as a base |

---

## 1. Official sources

- **Canonical `/idgames` page:**
  <https://www.doomworld.com/idgames/levels/doom2/megawads/btsx_e2> — filename
  `levels/doom2/megawads/btsx_e2.zip`; the archive listing dates the file **2020-08-16**. Title
  "Back to Saturn X E2: Tower in the Fountain of Sparks". **Confirmed (URL is canonical; the page
  itself was 403 from this host — see Method).**
- **Archive directory listing (fetched 2026-10-16):**
  <https://www.gamers.org/pub/idgames/levels/doom2/megawads/> — `btsx_e2.txt` and `btsx_e2.zip`,
  both dated `2020-08-16 10:47` (27 MB). **Confirmed**
- **Release text (`btsx_e2.txt`, inside the archive):** `Advanced engine needed : Doom2.exe or
  compatible` followed by the crucial line **`PRBOOM USERS USE COMPLEVEL 2 OR MAP20 WILL BREAK`**;
  `Version : 1.0.1`; `Release date : 08/13/2020`; `Author : Esselfortium and the Back to Saturn X
  team`; `Game : Doom 2`; `Map # : MAP01-MAP27, MAP31`; `New levels : 27`; `Dehacked/BEX Patch :
  Yes`. **Confirmed (quoted).**
- **Two-file split (release text):** *"BTSX has been split into two wad files which must be loaded
  together… to work around an obscure vanilla crash involving wads that exceed 4046 lumps."* and
  *"Source port users should only need to load the two wad files together."* **Confirmed (quoted).**

## 2. Canonical archive and byte-identical mirrors

The `/idgames` path is `levels/doom2/megawads/btsx_e2.zip`; every mirror serves that
`<dir>/<filename>` under its own base (`upstream-sources.md` §3.4). DDL uses direct mirror URLs,
never the `/idgames` API (D-015). Three allowlisted hosts were checked and **downloaded in full**;
all three serve the exact same bytes:

| Host (allowlisted) | URL | bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/megawads/btsx_e2.zip` | 27,988,469 | `2365b9ff…b60331` |
| `youfailit.net` | `https://youfailit.net/pub/idgames/levels/doom2/megawads/btsx_e2.zip` | 27,988,469 | `2365b9ff…b60331` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/megawads/btsx_e2.zip` | 27,988,469 | `2365b9ff…b60331` |
| `planetzdoom.com` (extra check) | `https://planetzdoom.com/idgames/levels/doom2/megawads/btsx_e2.zip` | 27,988,469 | `2365b9ff…b60331` |

**Confirmed:** all four are byte-identical (same length and same SHA-256 computed here), so they
may share one package record (D-015). The recipe lists the master plus two mirrors.

## 3. Archive contents (ZIP, 27,988,469 bytes)

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `btsx_e2a.wad` | 27,856,614 | `f3151768ee30d8c04a3c5e9ac091bbfd99dfe2271bfc6885b1ef1bde710a158d` | maps |
| `btsx_e2b.wad` | 40,471,797 | `f3dc59270166d5c5f5b1fcb7911aef2b5f672df74372a5cd2b49b34b2cc7b551` | graphics/music/resources |
| `btsx_e2.deh` | 6,708 | `…` (not loaded; see §4) | — |
| `btsx_e2.txt` | 16,003 | `8066832552e33a0d4ba372dfd1377083ec0b787d7fc26ecc48e9e6fd48146d5e` | credits (keep) |
| `map20_for_old_ports.zip` | 524,287 | `…` (official optional patch; see §5) | — |

Nested inside `map20_for_old_ports.zip`: `m20patch.wad` (1,604,511 B, `82be584c628cb2b1fda035c63c576ecab06c3f1578cfc7be8debc90f90684393`) and `README.txt`
(1,050 B, `0fd1ca5c3c4b8d5189e310859745a004d528e9c61070e972f69bf3c82fb399d1`). **Confirmed**

## 4. WAD inspection (read-only, no execution)

- `btsx_e2a.wad`: **PWAD**, 27,856,614 bytes, **2,822 lumps**, **30 map markers**: `MAP01`–`MAP28`,
  the secret `MAP31`, and the co-op alternate `MAP16C` (the release text lists `Map # :
  MAP01-MAP27, MAP31` and `New levels : 27`; see §8 about `MAP27`/`MAP28`). It also has
  `DEMO1`–`DEMO3`, a **`DEHACKED` lump (4,426 bytes)**, a ZDoom-style `MAPINFO`/`ZMAPINFO` and a
  `GAMEINFO`. **Confirmed**
- `btsx_e2b.wad`: **PWAD**, 40,471,797 bytes, **2,047 lumps**, **no maps**; it carries the custom
  texture resources, `PLAYPAL`/`COLORMAP` and 31 `D_*` music lumps. **Confirmed**
- **No `COMPLVL`** and **no `UMAPINFO`** lump. DSDA-Doom **removed ZDoom `MAPINFO` support in
  v0.30.0**, so the `MAPINFO` lumps are inert and the complevel must be declared in the recipe.
  **Confirmed**
- **Two DEHACKED sources.** The embedded `DEHACKED` in `btsx_e2a.wad` uses the extended
  `[STRINGS]`/`[PARS]` sections; the separate `btsx_e2.deh` uses vanilla `Text` blocks. The
  release text says source-port users only need the two WADs, so the embedded patch is the one
  DSDA-Doom applies. **Confirmed**

### What DSDA-Doom v0.30.1 does with the embedded DEHACKED under DDL's `-noautoload`

DDL launches with `-noautoload` (D-011) to skip **directory** autoload
([`prboom2/src/dsda/args.c:491–505`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L491-L505),
[`prboom2/src/d_main.c:2059–2070`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2059-L2070)).
`-noautoload` does **not** disable DEHACKED-in-a-WAD: `d_main.c` processes `DEHACKED` lumps from
explicitly loaded PWADs whenever `-nodeh` is absent, independent of the autoload flag
([`prboom2/src/d_main.c:2147–2156`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2147-L2156)).
So a launch `… -iwad doom2.wad -file …/btsx_e2a.wad …/btsx_e2b.wad -complevel 2 -noautoload`
applies the embedded patch automatically, and the recipe must **not** add a separate `-deh` entry.
**Confirmed** (upstream source at the pinned tag; `dsda-engine.md` §1 states the same rule).

## 5. Requirements and compatibility

- **Base game:** DOOM II — the release TXT says `Game : Doom 2`, `Map # : MAP01-MAP27, MAP31`.
  **Confirmed**
- **Required port class:** `Advanced engine needed : Doom2.exe or compatible`; *"fully compatible
  with Doom2.exe and Chocolate Doom"*, also tested with Eternity, PrBoom-Plus, ZDoom, GZDoom and
  Risen3D. **Confirmed (quoted).**
- **Complevel (Proposed):** **2** — the authors state it directly: *"PRBOOM USERS USE COMPLEVEL 2
  OR MAP20 WILL BREAK"*. In DSDA-Doom's enum complevel 2 is "Doom/Doom II 1.9". **Proposed until
  Deck-tested** (the value itself is the authors' own); this is the value the recipe uses.
- **Load order:** `btsx_e2a.wad` then `btsx_e2b.wad`. The release also notes an alternate co-op
  MAP16 (`MAP16C`) is included inside the main WAD, not as a separate file. **Confirmed**
- **Warp/episode notes:** a single continuous DOOM II mapset with no `UMAPINFO`, so no `-warp` is
  prescribed. The release text names the alternate co-op map (`MAP16C`) as the only non-linear
  slot. **Proposed.**
- **The bundled `map20_for_old_ports.zip` (genuine official optional component):** its README
  states *"DO NOT LOAD THIS PATCH IN VANILLA!"* and *"This patch should ONLY be used if your source
  port doesn't support the optimized blockmap used in the mainline version of MAP20."* It is needed
  by ZDoom 2.8.1, ZDaemon, Zandronum, PrBoom(-Plus) **without** a vanilla complevel, and other old
  Boom/ZDoom-derived ports; it is **not** needed by (and is not vanilla-compatible in) Vanilla,
  Chocolate Doom, Eternity, **PrBoom(-Plus) with complevel 2/3/4**, Crispy Doom, Doom Retro,
  GZDoom or LZDoom. **Confirmed (quoted).**
  **Decision (Proposed):** DDL always launches with complevel 2, so the patch is not needed and must
  not be offered — loading it would replace MAP20 with a non-vanilla variant. It is therefore
  documented here but not modelled as a component, following the AUGER;ZENITH EDGE-only precedent
  (D-016 / `upstream-sources.md` §5.4)).

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

## 7. Composition (recipe data — matches `catalog/back-to-saturn-x-e2.toml`)

- One package `free` (the ZIP above), one required `main` component with two members loaded in
  order: `btsx_e2a.wad` as `maps`, then `btsx_e2b.wad` as `graphics`. No optional components (see
  §5 for why the MAP20 patch is not offered).
- Keep `btsx_e2.txt` as `keep = true`.
- `base_game = doom2` (no variant restriction); `compatibility.family = "vanilla"`,
  `complevel = 2`, `complevel_status = "proposed"`; engine `=0.30.1`, `inspected`.

## 8. Open questions

- **`MAP27`/`MAP28` markers.** The WAD carries `MAP01`–`MAP28` while the release text lists
  `MAP01-MAP27, MAP31` and `New levels : 27`; `MAP27` and `MAP28` appear only as map markers in
  `btsx_e2a.wad` and are not described in the release map listing. They are inside the single
  WAD that the recipe loads, so the recipe is unaffected; a contributor could confirm their status
  from the authors.
- **Complevel 2 vs limit-removing.** The release targets vanilla Doom2.exe/Chocolate Doom;
  whether every map is safe in a pure vanilla *play* target on DSDA-Doom needs a Deck run
  (**Needs Deck Validation**).
- **BTSX E3.** The TXT references a three-episode project; only E1 and E2 are archived, and no
  third episode is part of this recipe.
- **Canonical page unread.** The Doomworld entry was Cloudflare-blocked from this host; a
  contributor with browser access could confirm there is no newer official build.


## Screenshots (spec 023, D-024)

Bundled under `crates/ddl/ui/assets/screenshots/back-to-saturn-x-e2/`; the `SOURCES.toml` beside the
images records each file's source URL, source page, credit, original size, retrieval date
and any permission text found. Retrieved 2026-10-17; each file is a 640x360 JPEG card crop
of the original below.

The screenshots are hosted by the project lead on `esselfortium.net` and were linked from the E2 release thread; the author's site is reachable directly.

| Asset | Source image URL | Source page | Credit | Original |
| --- | --- | --- | --- | --- |
| `01.jpg` | <https://esselfortium.net/images/btsx/e2_1.png> | <https://www.doomworld.com/vb/thread/69960> | Back to Saturn X team | 1280x800 (`e2_1.png`) |

**Why representative:** The E2 shots show the mapset's ruined temple-and-water vistas, the 'Tower in the Fountain of Sparks' theme.

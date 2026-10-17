# Reference research — Eviternity (Dragonfly et al)

**Status:** desk research for a D-010 catalog recipe · **Gathered:** 2026-10-16 (UTC)
**Author workstream:** Content Catalog (catalog addition)
**Scope:** facts needed to author a DDL recipe for **Eviternity** (the original 2018/2019 release,
not Eviternity II). No code is produced here.

Labels used throughout:

- **Confirmed** — verified from an official source, upstream source code, or a downloaded archive
  (evidence recorded below).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

The canonical page and release thread were fetched through the Internet Archive's Wayback Machine
because Doomworld returns Cloudflare HTTP 403 ("Just a moment…") to non-browser clients from this
host — the same limitation already recorded in `upstream-sources.md` §4.2. Snapshots used:

- `/idgames` entry: <https://web.archive.org/web/20241119051741id_/https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/eviternity>
- release thread (page 1): <https://web.archive.org/web/20210421212759id_/https://www.doomworld.com/forum/topic/103425-final-release-eviternity/>

Archives were downloaded to a scratch directory (never committed), hashed with `shasum -a 256`,
inspected with Python `zipfile` and a read-only WAD lump reader; nothing was executed.

```sh
curl -sSL -A "DoomDeckLauncher catalog review" \
  -o eviternity.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/eviternity.zip"
shasum -a 256 eviternity.zip
```

## Quick reference

| | Eviternity |
| --- | --- |
| Release | **Final version (v1.0)**, released 2019-02-10 |
| Author | Joshua "Dragonfly" O'Sullivan et al (guest mappers, see credits) |
| Archive | `levels/doom2/Ports/megawads/eviternity.zip` (id **19304**) |
| Archive size / SHA-256 | 49,938,319 bytes / `ad2f8b73108df7e4d3843d46a9e903238ac7e321ae9281f8c61f67fb92983656` |
| Main WAD | `Eviternity.wad` (MAP01–MAP32) |
| IWAD | `doom2.wad` |
| Format / complevel (Proposed) | MBF → **complevel 11** |
| Embedded patch | one `DEHACKED` lump in the WAD (no separate DEH/BEX) |
| `COMPLVL` lump | none |
| Optional components | none in the official package |
| Licence | **CC BY-NC-ND 4.0**, original presentation with the credits file intact |

---

## 1. Official sources

- **Release thread (authors' primary source, owner-provided):**
  <https://www.doomworld.com/vb/thread/103425> redirects to the canonical
  <https://www.doomworld.com/forum/topic/103425-final-release-eviternity/>. **Confirmed**
- **Opening post** — Dragonfly, 2018-12-10 (edited 2019-02-12),
  `?do=findComment&comment=1943244`: *"Eviternity is primarily an MBF format project, intended to be
  played in the following ports: PrBoom+, GZDoom, Eternity & Doom Retro; and is supported in
  Zandronum when playing multiplayer."* It links three official downloads: `/idgames`, ModDB and
  DFDoom.com. **Confirmed**
- The post's `Download Now` links resolve to
  <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/eviternity>,
  <https://moddb.com/mods/eviternity> and <https://www.dfdoom.com/download/1147/> (the last two are
  third-party hosts, not used by DDL). **Confirmed**
- **Recommended post** — Dragonfly, `?do=findComment&comment=1962502`: *"If you're going to record
  a speedrun, I would recommend doing your runs in PrBoom+ (not GlBoom+) using complevel 11."*
  **Confirmed**
- **Canonical `/idgames` page:** <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/eviternity>.
  Title "Eviternity (Final version)", filename `levels/doom2/Ports/megawads/eviternity.zip`,
  size 47.62 MB, date 02/11/19, author "Dragonfly et al". **Confirmed**

## 2. Canonical archive and byte-identical mirrors

The `/idgames` path is `levels/doom2/Ports/megawads/eviternity.zip`; every mirror serves that
`<dir>/<filename>` under its own base (`upstream-sources.md` §3.4). DDL uses direct mirror URLs,
never the `/idgames` API (D-015). Three allowlisted hosts were checked and **downloaded in full**;
all three serve the exact same bytes:

| Host (allowlisted) | URL | bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/eviternity.zip` | 49,938,319 | `ad2f8b73…83656` |
| `youfailit.net` | `https://youfailit.net/pub/idgames/levels/doom2/Ports/megawads/eviternity.zip` | 49,938,319 | `ad2f8b73…83656` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/Ports/megawads/eviternity.zip` | 49,938,319 | `ad2f8b73…83656` |

- **Confirmed:** all three are byte-identical (same length and same SHA-256 computed here), so
  they may share one package record (D-015).
- **Confirmed (excluded mirror):** `DSDArchive` (`https://dsdarchive.com/files/wads/doom2/1345/eviternity.zip`)
  serves a **different** 45,724,964-byte ZIP (`sha256=568ececc…6cd69e`). Its `Eviternity.wad` is
  byte-identical (`21b7e134…025062`) but its `Eviternity.txt` differs (14,838 vs 14,999 bytes).
  Because the package hashes differ it **cannot be listed as a mirror** for this recipe — the same
  "mirrors must be byte-identical" trap as the Romero vs `/idgames` SIGIL ZIPs (`CONTRIBUTING.md`).
- Other mirrors/paths exist (e.g. `ftp.fu-berlin.de`, `ftpmirror.infania.net`) but were not needed
  once two byte-identical mirrors were confirmed.

## 3. Archive contents (ZIP, 49,938,319 bytes)

Everything is listed as a member with its exact size and SHA-256 computed here. Nothing else is in
the archive; there is no separate MIDI pack or alternate WAD.

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `Eviternity.wad` | 130,864,796 | `21b7e13435f79d9d24e3c4fab3c257785c176c14f4d9ed217ea3288a73025062` | maps |
| `Eviternity.txt` | 14,999 | `3e1c5f1a08fb0b1d924fc6a832ac22ec78f98e0a536fe75bb8024b99a283209c` | credits (keep) |
| `Eviternity_Changelog.txt` | 24,284 | `d467780f0291dc51bd94b4a5f222bb44930be42419855aa1681cc28c7eaa9978` | changelog (keep) |

**Confirmed.** The changelog header reads "EVITERNITY v1.0 Changelog — Changes since RC1", and it
records that RC1's second, empty `DEHACKED` lump was removed for the final build.

## 4. WAD inspection (read-only, no execution)

`Eviternity.wad` is a **PWAD**, 130,864,796 bytes, **5,406 lumps**. **Confirmed.**

- **No `COMPLVL` lump** and **no `UMAPINFO`/`MAPINFO` lump** are present (there are GZDoom-only
  `ZMAPINFO`/`GAMEINFO`/`DECORATE`/`ZSCRIPT` lumps). The complevel therefore cannot come from a lump
  and must be declared in the recipe. **Confirmed**
- **One embedded `DEHACKED` lump** (23,239 bytes) beginning
  `Patch File for DeHackEd v3.0 … Doom version = 21 / Patch format = 6` (WhackEd4-authored,
  MBF-style). **Confirmed**
- It ships four `DEMO1`–`DEMO4` lumps of its own, consistent with an MBF-format release. **Confirmed**

### What DSDA-Doom v0.30.1 does with the embedded DEHACKED under DDL's `-noautoload`

DDL launches with `-noautoload` (D-011) to skip **directory** autoload
([`prboom2/src/dsda/args.c:491–505`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L491-L505),
[`prboom2/src/d_main.c:2059–2070`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2059-L2070)).
`-noautoload` does **not** disable DEHACKED-in-a-WAD: `d_main.c` processes `DEHACKED` lumps from
explicitly loaded PWADs whenever `-nodeh` is absent, independent of the autoload flag
([`prboom2/src/d_main.c:2147–2156`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2147-L2156)).
So a launch `… -iwad doom2.wad -file …/Eviternity.wad -complevel 11 -noautoload` applies Eviternity's
embedded patch automatically, and the recipe must **not** add a separate `-deh` entry (it would
double-apply). **Confirmed** (upstream source at the pinned tag; `dsda-engine.md` §1 already states
the same rule).

## 5. Requirements and compatibility

- **Base game:** DOOM II — the release TXT says `Game : Doom 2`, `Map # : MAP01 - MAP32`. **Confirmed**
- **Required port class:** `Advanced engine needed : PrBoom+ or GZdoom`; `Tested With : Primary
  engines: GZDoom 3.7.1, PrBoom+ 2.5.1.5`, *"Should also work: Eternity 4.00, Doom Retro 2.8;
  NOTE: GLBoom breaks skies"*. **Confirmed**
- **Format (authors):** "primarily an MBF format project" (opening post, comment 1943244). **Confirmed**
- **Complevel (Proposed):** **11 (MBF)**, from the authors' own recommendation in comment 1962502.
  `family = "mbf"`. **Proposed until Deck-tested** — this is the value the recipe uses.
- **Warp/episode notes:** Eviternity is a single continuous DOOM II mapset of six thematic 5-map
  "chapters" plus MAP31/MAP32; the chapters are not engine episodes and there is no `UMAPINFO`, so
  no `-warp` is prescribed. The DOOM II menu selects the map. **Proposed.**
- **DEH/BEX:** none to load separately (see §4). **Confirmed.**

## 6. Licence and redistribution

The release TXT states (**Confirmed (quoted)**):

```text
* Copyright / Permissions *
This work is licensed under the Creative Commons
Attribution-NonCommercial-NoDerivatives 4.0 International License.
Please do not redistribute the material in any form other than its original
presentation, with this credits file intact and unedited.
If you remix, transform, or build upon the material, you may not distribute
the modified material.
```

**Proposed (DDL policy):** DDL never modifies or repackages the archive; it downloads the original
ZIP from its canonical `/idgames` mirror and keeps both bundled text files (`keep = true`)
unmodified. The `permission` recipe field records the CC BY-NC-ND 4.0 terms verbatim in substance.
This is a stricter licence than SIGIL's "redistribute freely" text and is compatible with DDL's
fetch-from-source, keep-credits-intact model.

## 7. Composition (recipe data — matches `catalog/eviternity.toml`)

- One package `free` (the ZIP above), one required `main` component loading `Eviternity.wad` as
  `maps`. No optional components: the official package ships nothing else.
- Keep `Eviternity.txt` and `Eviternity_Changelog.txt` as `keep = true`.
- `base_game = doom2` (no variant restriction); `compatibility.family = "mbf"`,
  `complevel = 11`, `complevel_status = "proposed"`; engine `=0.30.1`, `inspected`.

## 8. UI cover-art note (reported, not changed here)

DDL does **not** yet render a recipe's `[[images]]`. Cover art is a hardcoded map from recipe id to a
cell of the bundled sprite sheet: `fn art(id)` in `crates/ddl/src/runtime.rs` returns index 0 for
`sigil`, 1 for `auger-zenith`, 2 for `sigil-ii`, and **falls back to 0 for every other id**.
`crates/ddl/ui/assets/furnace-art-sheet.png` is 1536×1024 — exactly three 512-wide cells — and each
cell is captioned "Original concept art". Consequently an `eviternity` recipe (which has no
`[[images]]`, like every current recipe) would display **SIGIL's cell (index 0)**, i.e. the wrong
art. Fixing this needs a new sheet cell plus an `art("eviternity")` arm, both outside this task's
ownership; `[[images]]` remain lint-only metadata today.

## 9. Open questions

- **Complevel 11 vs the port defaults.** The authors recommend complevel 11 for demo/speedrun
  recording and call the project MBF-format; DSDA's default is 21 (MBF21). Whether complevel 11 is
  the best *play* default on DSDA-Doom and whether the embedded DEHACKED behaves identically to
  PrBoom+ needs a Deck run (**Needs Deck Validation**).
- **DSDArchive divergence.** Why the DSDArchive copy carries a shorter `Eviternity.txt` is unknown;
  DDL excludes it as a mirror, but a future review could document the difference.


## Screenshots (spec 023, D-024)

Bundled under `crates/ddl/ui/assets/screenshots/eviternity/`; the `SOURCES.toml` beside the
images records each file's source URL, source page, credit, original size, retrieval date
and any permission text found. Retrieved 2026-10-17; each file is a 640x360 JPEG card crop
of the original below.

The official release screenshots are hosted on the project lead's site `www.dfdoom.com` and were linked from the mapset's release thread (`doomworld.com/vb/thread/103425`); dfdoom.com is reachable directly.

| Asset | Source image URL | Source page | Credit | Original |
| --- | --- | --- | --- | --- |
| `01.jpg` | <https://www.dfdoom.com/wp-content/uploads/2018/12/evi-post-04.jpg> | <https://www.doomworld.com/vb/thread/103425> | Eviternity team | 1896x1042 (`evi-post-04.jpg`) |

**Why representative:** The shots are in-game views of Eviternity's golden-lit tech base, not the title screen or box art.

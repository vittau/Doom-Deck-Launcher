# Reference research — Scythe II, Plutonia 2 and TNT: Revilution

**Status:** desk research for D-010 catalog recipes · **Gathered:** 2026-10-22 (UTC)
**Workstream:** spec 087 worker A (Scythe II, Plutonia 2) and spec 088 worker A (TNT: Revilution)
**Scope:** the `/idgames` releases named below, the archive bytes, release text, embedded and
external patches, and the evidence needed for each recipe. No archive content was executed and no
IWAD was downloaded.

Labels used throughout:

- **Confirmed** — verified from the downloaded archive, its release text, or pinned DSDA-Doom
  v0.30.1 source/release notes; the evidence is recorded below.
- **Proposed** — a DDL interpretation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

All downloads went to `.scratch/087-a/` (git-ignored, never committed). ZIP members were read as
data with Python's `zipfile`; WAD directories and DEHACKED/MAPINFO lumps were parsed read-only with
`struct`. Nothing was extracted to a loadable directory implicitly, and nothing was executed.

```sh
curl -fsSL --max-time 300 --max-filesize 60000000 -o scythe2.zip \
  https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/scythe2.zip
curl -fsSL --max-time 300 --max-filesize 60000000 -o pl2.zip \
  https://www.gamers.org/pub/idgames/levels/doom2/megawads/pl2.zip
curl -fsSL --max-time 300 --max-filesize 60000000 -o tntr.zip \
  https://www.gamers.org/pub/idgames/levels/doom2/megawads/tntr.zip
shasum -a 256 *.zip
```

`youfailit.net` timed out (75 s, no connection) for both archives and is **not listed** in the
recipes. `ftp.fu-berlin.de` and `mirrors.lug.mtu.edu` were downloaded in full and are byte-identical
to the archive master for all three packages.

## Quick reference

| | Scythe II | Plutonia 2 | TNT: Revilution |
| --- | --- | --- | --- |
| Recipe id | `scythe-ii` | `plutonia-2` | `tnt-revilution` |
| Release | Version 2, 2009-07-10 (first release 2005-06-07) | Version 1.0, 2008-12-31 | Version 1.20, 2017-07-08 (first release 2017-05-29) |
| Author(s) | Erik Alm | Plutonia 2 Team (22 contributors credited) | TNT: Revilution Team (community project) |
| `/idgames` path | `levels/doom2/Ports/megawads/scythe2.zip` | `levels/doom2/megawads/pl2.zip` | `levels/doom2/megawads/tntr.zip` |
| Package size / SHA-256 | 10,584,383 / `0d5c1ab9…0cd8d` | 5,375,859 / `bb925d13…a0c66` | 8,619,240 / `aa64d4f3…6cba8` |
| Main WAD | `scythe2.wad`, 31,285,404 B | `PL2.WAD`, 18,131,932 B | `TNTR.wad`, 28,776,797 B |
| IWAD | DOOM II | **The Plutonia Experiment** | **TNT: Evilution** |
| Format / complevel | limit-removing → `vanilla` (2 on Doom II) | vanilla → `vanilla` (4 on Plutonia) | vanilla → `vanilla` (4 on TNT) |
| Maps | 32 (`MAP01`–`MAP32`) | 32 reachable (`MAP01`–`MAP32`) + `MAP33` defined only in ZDoom MAPINFO | 32 (`MAP01`–`MAP32`) |
| Patch | embedded `DEHACKED` (5,812 B) | embedded `DEHACKED` (7,357 B); external `PL2.DEH` (7,359 B) not loaded | embedded `DEHACKED`/BEX (8,218 B); external `TNTR.deh` (10,372 B) not loaded |
| Music | embedded `D_*` lumps | embedded `D_*` lumps; MAPINFO names tracks | 35 embedded `D_*` lumps |
| Licence | no reuse as a base; distribute with text intact; non-commercial | no reuse as a base; distribute with text intact; team obtained permissions | reuse permitted; distribute with text intact; team obtained permissions |

---

## 1. Scythe II (`scythe-ii`)

### 1.1 Release identity

- **Canonical page:** <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/scythe2>,
  filename `levels/doom2/Ports/megawads/scythe2.zip`. **Confirmed.**
- `scythe2.txt` title block: *"Title: Scythe 2. Version 2"*, *"Release date: June 7, 2005,
  Rereleased July 10, 2009"*, *"Author: Erik Alm"*. **Confirmed (quoted).**
- The text says *"32 new levels"* and *"Map #: Map01-32"*, and the changelog says *"maps 28-30
  added"* in version 2. The current archive is therefore the complete 2009 version 2 release, not
  the 2005 partial release. **Confirmed.**
- The author labels six episodes in the map list (MAP01–05, 06–10, 11–15, 16–20, 21–25, 26–27 plus
  MAP28–30 and MAP31/32). These are text chapters, not engine episodes: there is no UMAPINFO or
  MAPINFO lump. **Confirmed.**

### 1.2 Archive and byte-identical mirrors

| Host (allowlisted) | URL | Bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/scythe2.zip` | 10,584,383 | `0d5c1ab9d0152f5f1cf7a0d4ca2f41f5a6ac2986fa6ee850504d1d49e020cd8d` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/Ports/megawads/scythe2.zip` | 10,584,383 | same |
| `ftp.fu-berlin.de` | `https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/Ports/megawads/scythe2.zip` | 10,584,383 | same |

**Confirmed:** all three downloaded in full, identical bytes; `/idgames` publishes no checksum, so
the SHA-256 was computed here.

### 1.3 Archive contents (ZIP, 10,584,383 bytes)

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `scythe2.wad` | 31,285,404 | `157fa27eed11a872c41263ff24b5677aeb123514a6352f040f7873435ad09ee1` | maps (main) |
| `scythe2.txt` | 9,052 | `1902b55fb824fde4d9dffe8e15b29c7d8bebec62450636f0a4cd25a721c9835f` | credits/permission (keep) |

**Confirmed.** Flat layout, two Deflate entries, no symlinks, no traversal, no nested ZIP.

### 1.4 WAD and patch inspection (read-only)

`scythe2.wad` — **PWAD**, 31,285,404 bytes, **1,243 lumps**, 32 map markers `MAP01`–`MAP32`, each a
standard ten-lump group. **Confirmed.**

- **Embedded `DEHACKED` lump, 5,812 bytes.** It is a DeHackEd v3.0 / patch format 6 patch that
  changes things (SS Nazi/Commander Keen slots), frames and codepointers for the mapset's custom
  monsters. It is applied automatically by DSDA-Doom from the loaded PWAD; `-noautoload` does not
  disable embedded patches, and DDL does not pass `-nodeh`. **Confirmed** by reading the lump and
  the pinned DSDA-Doom v0.30.1 behaviour in `docs/research/dsda-engine.md` §1.
- **No `COMPLVL`, `UMAPINFO` or `MAPINFO` lump.** **Confirmed.**
- The WAD carries its own `SS_START`/`FF_START` graphics and `D_*` music lumps; no separate music
  file exists in the archive. **Confirmed.**

### 1.5 Compatibility reasoning

- **Base game: DOOM II.** The release text says *"Game: DOOM2"* and *"Map #: Map01-32"*.
  **Confirmed.**
- **Family: limit-removing.** The text says *"Advanced engine needed: Any limit removing port"*
  and *"Doom2.exe. Don't even try it. The new version is however compatible with pure limit
  removing ports."* **Confirmed (quoted).**
- **Complevel: `vanilla`.** The author states no complevel, so the limit-removing community
  convention in CONTRIBUTING applies (limit-removing → `vanilla`). On DOOM II the alias resolves
  to 2 (D-030). **Proposed by convention; Needs Deck Validation.**
- **Runtime:** a single continuous 32-map set with no UMAPINFO, so the recipe prescribes no
  `warp`; the map menu selects the map. **Confirmed.**
- **Engine evidence:** DSDA-Doom v0.30.1 accepts the `vanilla` alias and applies embedded
  DEHACKED; the recipe pins `=0.30.1` with `inspected` status (not gameplay-verified).

### 1.6 Permission and credits

`scythe2.txt` says **Confirmed (quoted):**

> Authors may NOT use the contents of this file as a base for modification or reuse unless written
> permission is obtained from the author. You MAY distribute this file, provided you include this
> text file, with no modifications. … This file may not be used for any commercial purposes without
> the author's agreement.

Additional credits in the text: testers Qwerty, Vile, Vincent, Khaoscythe, Anima Zero and others;
the GothicDM/Gothic2 crew for textures; Kristian Aro for compiling Ruinbros resources; Espi for
monster sprites; Kim "Mutator" Malde for level-name graphics; Julian for intermission/title
graphics; Fredrik Johansson for textures; Drew "stx-Vile" Devore for music selection and testing.
The recipe keeps `scythe2.txt` (`keep = true`).

---

## 2. Plutonia 2 (`plutonia-2`)

### 2.1 Release identity

- **Canonical page:** <https://www.doomworld.com/idgames/levels/doom2/megawads/pl2>, filename
  `levels/doom2/megawads/pl2.zip`. **Confirmed.**
- `PL2.TXT` title block: *"Title: Plutonia 2"*, *"Release: Version 1.0, December 31, 2008"*,
  *"Author: Plutonia 2 team"*. **Confirmed (quoted).**
- `PL2.TXT` description: *"An add-on sequel for Final DOOM's 'The Plutonia Experiment'. It emulates
  Plutonia's design and gameplay styles and follows from where it left off."* **Confirmed.**
- `PL2INFO.TXT` is the authors' manual: compatibility, resource credits, music track notes and a
  32-map information section. **Confirmed.**

### 2.2 Archive and byte-identical mirrors

| Host (allowlisted) | URL | Bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/megawads/pl2.zip` | 5,375,859 | `bb925d13119e6d5dfea69a5ffe17d8954cdc3f2da0a6128a58f6ff6a366a0c66` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/megawads/pl2.zip` | 5,375,859 | same |
| `ftp.fu-berlin.de` | `https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/megawads/pl2.zip` | 5,375,859 | same |

**Confirmed**, downloaded in full and byte-identical.

### 2.3 Archive contents (ZIP, 5,375,859 bytes)

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `PL2.WAD` | 18,131,932 | `9495a54ecfd5ab2dde4fed22eb4a1dd3e70296899df835f5aa74d8a8cc6f253e` | maps (main) |
| `PL2.TXT` | 3,915 | `c19728b15488d00ca1f86973ad4508506c99a996617e7c66b1ac164a298f218d` | credits/permission (keep) |
| `PL2INFO.TXT` | 66,591 | `e79112f2be34943f249a68e960bb6cf577253d670e1d7d28773b5b423ba4028c` | manual/credits (keep) |
| `PL2.DEH` | 7,359 | `f025d00c9276fb612078efe976c582518fa8dd782629b18a1ca49ddda4c86589` | external DeHackEd — **not declared** (§2.4) |
| `PL2.ICO` | 22,486 | `80957236f5310a3fe90a0abadafaca2240476dae6234b24297cd0a373533c294` | Windows icon — **not declared** |

**Confirmed.** Flat layout, five Deflate entries, no symlinks, no traversal. `PL2.ICO` is not a
game file and is outside DDL's model.

### 2.4 WAD and patch inspection (read-only)

`PL2.WAD` — **PWAD**, 18,131,932 bytes, **761 lumps**. It has **33 map markers**: `MAP01`–`MAP32`
plus `MAP33`, and each group is a standard ten-lump map. **Confirmed.**

- **`MAP33` "Chocolate" is a bonus map defined only in the ZDoom `MAPINFO` lump** (5,038 B);
  `PL2.TXT` still describes the release as *"New levels: 32 (full megawad)"*. **Confirmed.**
- **Embedded `DEHACKED` lump, 7,357 bytes**, with level names, intermission/story text and music
  strings; it has 79 `Text` entries and the 32 `level n:` intermission names. **Confirmed** by
  reading the lump.
- **External `PL2.DEH`, 7,359 bytes**, is a plain DeHackEd v3.0 patch with the same kind of text
  changes. It is **not byte-identical** to the embedded lump: at the MAP03 entry the embedded lump
  says `aztec3: Emerald Eve` (matching the MAPINFO and the manual's map list) while the external
  file says `aztec3: Skull Island`, and one `Text 14` length differs (14 vs 15). **Confirmed.**
- **Decision — do not load both.** `PL2INFO.TXT` §1.4 says: *"The DeHackEd patch (PL2.DEH) only
  contains text information … It is optional, but recommended. The patch is included as a lump in
  the WAD for source ports that support a DEHACKED lump."* DSDA-Doom supports the embedded lump
  and applies it automatically, so the external file is for engines without internal support; the
  recipe declares no `dehacked` component and does not apply the older external strings twice.
  **Confirmed by inspection; Needs Deck Validation for the visible names.**
- **No `COMPLVL` or `UMAPINFO` lump.** **Confirmed.**

### 2.5 Compatibility reasoning

- **Base game: The Plutonia Experiment.** `PL2.TXT` says *"Game: The Plutonia Experiment"*.
  `PL2INFO.TXT` §3.2 is explicit: *"PLUTONIA.WAD as the IWAD. Using DOOM2.WAD will cause errors
  during play, or fail to work at all"*; it also notes the resources were *"converted it to run
  with Plutonia as the IWAD."* The `/idgames` path is a Doom II directory but is **not** evidence
  for the IWAD. **Confirmed (quoted).**
- **Family: vanilla.** `PL2.TXT` says *"Designed to run with DOOM2.EXE"* and *"Tested With: Doom2
  v1.9f, ZDoom v2.1+, PrBoom v2.4+"*; `PL2INFO.TXT` describes playing *"in 'vanilla' form with
  the DeHackEd patch"*. **Confirmed (quoted).**
- **Complevel: `vanilla`.** With Plutonia as the IWAD the alias resolves to 4 (D-030). **Proposed
  by the release's vanilla target; Needs Deck Validation.**
- **ZDoom MAPINFO is inert.** DSDA-Doom removed ZDoom MAPINFO support in v0.30.0
  (<https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.0>), so DSDA-Doom ignores the map
  titles/episode block and cannot reach `MAP33`. **Confirmed** by the pinned release note; the
  visible consequence is **Needs Deck Validation**.
- **Runtime:** single continuous `MAP01`–`MAP32` progression with no UMAPINFO; no `warp` default.

### 2.6 Permission and credits

`PL2.TXT` says **Confirmed (quoted):**

> Authors may NOT use the contents of this file as a base for modification or reuse. Permissions
> have been obtained from original authors for any of their resources modified or included in this
> file. You MAY distribute this file, provided you include this text file, with no modifications.
> … The team has received permission from the original authors of any modified or included content
> in this file to allow further distribution.

`PL2INFO.TXT` §6.1 credits 22 contributors: Thomas van der Velden (T.V./Rabotik), Gusta, Kira,
Pipicz, PerOxyd, Joe Pallai, Fredrik, Angus, Tango, Erik Alm, Alexander S., Eric "The Green
Herring" Baker, James Paddock (Jimmy), Stuart Rynn, Miguel Folatelli, Jamie Robertson, Julian
Aubourg, Sam Woodman (Metabolist), Pedro A. Gomez Blanco, Kristian Käll, Paul Corfiatis and Virgil.
**Confirmed** from the manual. The recipe keeps `PL2.TXT` and `PL2INFO.TXT`.

---

## 3. TNT: Revilution (`tnt-revilution`)

### 3.1 Release identity

- **Canonical page:** <https://www.doomworld.com/idgames/levels/doom2/megawads/tntr>, filename
  `levels/doom2/megawads/tntr.zip`. **Confirmed.**
- `TNTR.txt` title block: *"Title: TNT: Revilution"*, *"Version: 1.20"*, *"Release date:
  05/29/2017"*, *"Updated: 07/08/2017"*, *"Author: Various"*. **Confirmed (quoted).**
- The text describes a *"fan made community project that acts as a sequel to TeamTNT's
  Evilution"*. **Confirmed (quoted).**
- Project history: developed as *TNT2: Devilution* from 2009-08-28, renamed *TNT: Revilution* on
  2014-02-19, led by Eris Falling from 2015-08-22 and Jaws In Space from 2016-01-31. **Confirmed
  (quoted).**

### 3.2 Archive and byte-identical mirrors

| Host (allowlisted) | URL | Bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/megawads/tntr.zip` | 8,619,240 | `aa64d4f3e713e4c7349c86d02731428c19e1f403f87e7d589d22e663a446cba8` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/megawads/tntr.zip` | 8,619,240 | same |
| `ftp.fu-berlin.de` | `https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/megawads/tntr.zip` | 8,619,240 | same |

**Confirmed**, downloaded in full and byte-identical.

### 3.3 Archive contents (ZIP, 8,619,240 bytes)

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `TNTR.wad` | 28,776,797 | `3e842f3172fac2d2008cbf0dfe40b89af2ea34ece07c0df75f8992359a79af86` | maps (main) |
| `TNTR.txt` | 10,182 | `52f724f2e9562a7d68bb3635fff758bf1ee5e1d268c151a59959be5c06fae6a3` | credits/permission (keep) |
| `TNTR.deh` | 10,372 | `96cc84645a61865f38a8c08386d5eabf73f07bc7bf47e9107979d6d5bee2f257` | external DeHackEd — **not declared** (§3.4) |

**Confirmed.** Flat layout, three Deflate entries, no symlinks, no traversal.

### 3.4 WAD and patch inspection (read-only)

`TNTR.wad` — **PWAD**, 28,776,797 bytes, **1,585 lumps**, 32 map markers `MAP01`–`MAP32`.
**Confirmed.**

- **Embedded `DEHACKED`/BEX lump, 8,218 bytes.** It defines the custom things (Laser Turret,
  Holo-Icon, Fake Cube, Rocket Turret), 50 frame edits, `[CODEPTR]` assignments, `[PARS]` par times
  for all 32 maps and a `[STRINGS]` block with 32 `THUSTR_n` map names, the mapset name, six story
  texts and six background flats. **Confirmed** by reading the lump.
- **External `TNTR.deh`, 10,372 bytes.** It is a plain DeHackEd v3.0 file with the same changes in
  `Text`/`Pointer` form (49 `Text` entries, including the 32 `level n:` names). It is **not
  byte-identical** to the embedded patch because the two formats encode the same strings
  differently. **Confirmed.**
- **Decision — do not load both.** The WAD's embedded patch is complete for DSDA-Doom, which
  applies embedded DEHACKED automatically; the external file is the fallback for engines without
  embedded support (the authors' text lists Chocolate Doom, Crispy Doom and PrBoom among tested
  ports). Loading the external file after the embedded one would re-apply the same changes rather
  than add missing data. **Confirmed by inspection; Needs Deck Validation for the visible result.**
- **ZDoom `MAPINFO` lump, 4,633 bytes**, is inert under DSDA-Doom v0.30.1 (ZDoom MAPINFO support
  removed in v0.30.0). **Confirmed.** **No `COMPLVL` or `UMAPINFO` lump. Confirmed.**
- **Music:** the WAD carries 35 `D_*` lumps, matching the readme's per-map MIDI list. No separate
  music WAD exists. **Confirmed.**

### 3.5 Compatibility reasoning

- **Base game: TNT: Evilution.** `TNTR.txt` says *"Game: TNT"* and *"For Demos use TNT.wad"*, and
  the project is a sequel to *TNT: Evilution*. The `/idgames` Doom II directory is not evidence.
  **Confirmed (quoted).**
- **Family: vanilla.** `TNTR.txt` says *"Advanced engine needed: No. For Demos use TNT.wad"* and
  *"Tested With: Chocolate Doom, Crispy Doom, Doom Retro, CRL, PRBoom, GZDoom, ZDAEMON."*
  **Confirmed (quoted).**
- **Complevel: `vanilla`.** With TNT as the IWAD the alias resolves to 4 (D-030). **Proposed by
  the release's vanilla target; Needs Deck Validation.**
- **Known vanilla limits.** `TNTR.txt` warns that *"On Map 16 & Map 22 Arch-Viles can launch the
  player into positions that can cause Vanilla to crash due to Visplanes"*, lists savegame
  overflows on several maps, and says MAP30 requires a telefrag compatibility option in some
  ports. **Confirmed (quoted); Needs Deck Validation.**
- **Runtime:** single continuous `MAP01`–`MAP32` progression with no UMAPINFO; no `warp` default.

### 3.6 Permission and credits

`TNTR.txt` says **Confirmed (quoted):**

> Authors MAY use the contents of this file as a base for modification or reuse. Permissions have
> been obtained from original authors for any of their resources modified or included in this file.
> You MAY distribute this file, provided you include this text file, with no modifications. … I
> have received permission from the original authors of any modified or included content in this
> file to allow further distribution.

Credits in `TNTR.txt`: project founders Kyka, Ragnor "Devalaous" Berntsen and Megamur; project
leaders Micah "Jaws In Space" Petersen, Kyka and Eris Falling; quality control Dobu Gabu Maru,
Steve D, Demon of the Well, Ragnor, Megamur, rdwpa and AD_79; music by Bucket, Eradrop, Eris
Falling, Jeremy Doyle and Viscra Maelstrom; DeHackEd by Obsidian; story by Jaws In Space; graphics
and textures by the long list in the text. **Confirmed** from the release text. The recipe keeps
`TNTR.txt`.

---

## 4. Tag evidence (R3)

| Tag | Scythe II | Plutonia 2 | TNT: Revilution |
| --- | --- | --- | --- |
| `cacowards` | 2005 Top Ten winner — <https://www.doomworld.com/cacowards/2005/> | 2009 Top Ten winner and Mordeth Award — <https://www.doomworld.com/cacowards/2009/> | 2017 runner-up — <https://www.doomworld.com/cacowards/2017/> |
| `megawad` | 32 maps (readme + WAD) | 32 maps (readme + WAD; MAP33 is MAPINFO-only) | 32 maps (readme + WAD) |
| `classics` | 2005 release, Cacoward recognition | 2008 release, Cacoward recognition | 2017 release, Cacoward recognition |
| `community-projects` | — (single author) | 22 contributors in PL2INFO.TXT §6.1 | Multi-author community project, credited in TNTR.txt |
| `challenging` | Readme: "This wad is harder than the original Scythe"; "you WILL run into trouble on skill 4" | — | — |

The Cacowards rows come from the archived inventory in `docs/research/cacowards.md` (2005 line for
Scythe 2, 2009 Top Ten and Mordeth rows for Plutonia 2, 2017 runner-up row for TNT: Revilution),
which records the exact Wayback captures it read. **Confirmed from that archived inventory;
doomworld.com blocks automated fetches, so the live pages were not read in this session.** The
recipes say so in their notes rather than claiming a direct live read.

No Cacowards claim is made for gameplay quality beyond the award itself; the recipes stay
`inspected`, not `verified`.

## 5. Artwork (R4) — unavailable source, fallback documented

No compliant in-game screenshot was obtained for the three titles, so no screenshot directory or
`SOURCES.toml` was committed and the recipes carry no `[[screenshots]]` entries. The existing
painted fallback art remains. Evidence of the attempts, all on 2026-10-22:

- **DoomWiki direct:** `https://doomwiki.org/w/images/...` and the MediaWiki API returned
  Cloudflare HTTP 403 ("Just a moment…"); DoomWiki blocks automated fetches. **Confirmed.**
- **Scythe II DoomWiki page via Wayback:** the archived `Scythe_2` page was fetched and parsed; it
  shows a title picture (`Scythe2_titlepic.png`) and two monster sprites (`Scythe2-marine.PNG`,
  `Scythe2-afrit.PNG`), but no in-game map view. The title picture and sprites are excluded by
  D-024/CONTRIBUTING (not a recognisable map view), so they were not used. **Confirmed.**
- **Plutonia 2 DoomWiki page via Wayback:** the archived page shows only the title picture
  (`Plutonia_2_title.png`); no in-game map view. **Confirmed.**
- **Later Wayback CDX/direct requests:** the service began returning "Internet Archive:
  Temporarily Offline" and connection resets, so the remaining map-page images could not be
  sourced within the bounded network budget. TNT: Revilution had no reachable candidate page.
- **Doomworld and the authors' sites:** Doomworld is Cloudflare-blocked for automation, and the
  old official `rabotik.nl/pluto2.html` returns HTTP 404. **Confirmed.**

The recipes omit `[[screenshots]]`; this is recorded in `docs/reports/087-a.md` as required by
spec 087 R4/088 R3. No image was synthesized, upscaled from a title screen or captured from a
window.

## 6. Verification commands and results

Run from the repository root on the macOS arm64 dev host, with
`CARGO_TARGET_DIR=/Users/vitor/git/Doom-Deck-Launcher/target` and `CARGO_BUILD_JOBS=2`:

```sh
/Users/vitor/git/Doom-Deck-Launcher/target/debug/ddl catalog lint \
  catalog/scythe-ii.toml catalog/plutonia-2.toml catalog/tnt-revilution.toml
# Catalog lint passed (offline).

/Users/vitor/git/Doom-Deck-Launcher/target/debug/ddl catalog lint --fetch \
  catalog/scythe-ii.toml catalog/plutonia-2.toml catalog/tnt-revilution.toml
# Catalog lint passed (fetched packages verified).

CARGO_TARGET_DIR=… CARGO_BUILD_JOBS=2 \
  cargo test -p ddl-core --test catalog_087_a -- --test-threads=2
# 5 passed; 0 failed.
```

The integration test freezes the package and member sizes/hashes, case-exact paths, one main
component in order, the base game and `vanilla` alias (including the resolved 2/4/4 level), the
tag sets and the cited Cacowards year, and proves the three assigned recipes appear exactly once in
`Catalog::embedded` with no skipped entries. Formatting and core clippy results are recorded in
`docs/reports/087-a.md`.

## 7. Open questions / Deck checks

- **Needs Deck Validation (all three):** IWAD selection (Doom II, Plutonia, TNT), actual complevel
  behaviour, embedded DEHACKED application (custom monsters, level names, story text), load order,
  audio, map selection and progression.
- **Plutonia 2:** confirm the external `PL2.DEH` is not needed on DSDA-Doom and that the embedded
  lump's `Emerald Eve` / story text appear; confirm `MAP33` is unreachable as expected and whether
  that limitation is acceptable for the release.
- **TNT: Revilution:** confirm the embedded patch's custom monsters and level names, and test the
  README's MAP16/MAP22 visplane risk, savegame overflow and MAP30 telefrag compatibility note.
- **Scythe II:** confirm the embedded patch's custom monsters and that the six text episodes play
  as one continuous 32-map run without UMAPINFO.
- **Artwork:** no in-game image is committed for any of the three titles; a later manual/owner-run
  capture under D-024 can fill the fallback without changing the recipes.

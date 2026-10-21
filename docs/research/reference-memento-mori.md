# Reference research — Memento Mori and Memento Mori II

**Status:** desk research for two D-010 catalog recipes (spec 073) · **Gathered:** 2026-10-21 (UTC)
**Author workstream:** Content Catalog · **Scope:** facts needed to author DDL recipes for
**Memento Mori** (the complete/updated release) and **Memento Mori II**. No code is produced here.

Labels used throughout:

- **Confirmed** — verified from an upstream file, a downloaded archive or a primary statement
  (evidence recorded below).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

All downloads went to `.scratch/mm/` (git-ignored, never committed). Nothing was executed: ZIPs were
read with Python's `zipfile`, and the WADs only parsed as data by a read-only lump-directory reader.
Doomworld answers HTTP 403 from this host, so idgames metadata comes from the archive master's
directory listings (`ls-laR.gz`) and the sidecar `.txt` files; DoomWiki pages were read from Wayback
snapshots (2024/2026 captures).

```sh
curl -sSL -A "DoomDeckLauncher catalog review" -o mm_allup.zip \
  "https://www.gamers.org/pub/idgames/themes/mm/mm_allup.zip"
curl -sSL -A "DoomDeckLauncher catalog review" -o mm2.zip \
  "https://www.gamers.org/pub/idgames/themes/mm/mm2.zip"
shasum -a 256 mm_allup.zip mm2.zip
```

**Where the two releases live.** `/idgames` keeps both under the dedicated `themes/mm/` directory
(confirmed by the directory listing, 2026-10-21). Memento Mori's *complete* form is
`themes/mm/mm_allup.zip` — the archive idgames ids 4855 links as "Complete version" (DoomWiki,
§External links) and the one the crew's own text calls "MM_ALL.ZIP … This ZIP contains all files".
The segmented three-floppy form (`mm1_3up.zip` … `mm3_3up.zip`, each with `INSTALL.EXE`/`MM.EXE`)
is deliberately **not** used: it carries DOS executables. Memento Mori II's complete form is
`themes/mm/mm2.zip` (MM2.TXT: "MM2.ZIP – WADs to play Memento Mori 2"; the segmented form is
`mm2_1/2/3.zip`). **Confirmed.**

## Quick reference

| | Memento Mori | Memento Mori II |
| --- | --- | --- |
| Release | First release 1995-12-10; updated version February 1996 (`/idgames` file dated **1996-02-21**) | **1996-07-27** (MM2.TXT header and file date) |
| Course | "The Memento Mori Crew" — 21 map authors coordinated by Denis and Thomas Möller (The Innocent Crew) | "MM2 Crew" — 24 contributors; coordinated by the same Möllers |
| Archive | `themes/mm/mm_allup.zip` (`/idgames` id 4855) | `themes/mm/mm2.zip` (`/idgames` id 6562) |
| Archive size / SHA-256 | 2,950,178 bytes / `c8607a7a32d4f6547d492e0f734324320b955e2b11691b02b107de09ac2e7bfb` | 4,061,135 bytes / `f4e7a10b035b123647f631348d63fc46392d8d8e21fab8a5158cd9ef35027451` |
| Main WAD | `MM.WAD`, 7,825,282 B (MAP01–MAP32) | `MM2.WAD`, 11,453,000 B (MAP01–MAP32) |
| Music WAD | `MMMUS.WAD`, 885,387 B, 35 `D_*` lumps | `MM2MUS.WAD`, 1,075,404 B, 35 `D_*` lumps |
| IWAD | `doom2.wad` | `doom2.wad` |
| Format / complevel | Vanilla Doom II v1.9 → **complevel 2** | Vanilla Doom II v1.9 → **complevel 2** |
| Load order | `doom2 -file mm.wad mmmus.wad` (MMHELP.TXT Q4) | `doom2 -file mm2.wad mm2mus.wad` (MM2.BAT) |
| `DEHACKED` / `COMPLVL` / `UMAPINFO` | none / none / none | none / none / none |
| Licence | distribute only with the text file intact; per-map reuse needs the author's agreement | free; CD sales need prior permission; songs may not be extracted |

---

## 1. Official sources

- **Memento Mori.** `/idgames` path `themes/mm/mm_allup.zip`; sidecar `themes/mm/mm_allup.txt`
  (2,893 bytes) is byte-identical to the archive's own `MM.TXT` (SHA-256 `33f67e46…` on both).
  DoomWiki (Wayback capture 20260213005328 of the article) gives release **1995-12-10**, an
  **updated release in February 1996**, IWAD Doom II, port Vanilla Doom, and the "Top 100 WADs of
  All Time" 1996 listing (where it is erroneously filed as a 1996 WAD). **Confirmed** (file bytes)
  / **Confirmed** (DoomWiki secondary for the dates).
- **Memento Mori II.** `/idgames` path `themes/mm/mm2.zip`; sidecar `themes/mm/mm2.txt` is 440
  bytes and byte-identical to the archive's `FILE_ID.DIZ` (SHA-256 `c3994fdd…`). DoomWiki (capture
  20250124061149) gives release **1996-07-27**, IWAD Doom II, port Vanilla Doom, and lists it as
  #2 of the 1996 Top 100 WADs. **Confirmed** (file bytes) / **Confirmed** (DoomWiki secondary).
- **Version note (Memento Mori).** `AUTHORS.TXT` and `CHANGES.TXT` are headed "updated version,
  Feb 95" — a typo in the authors' own text, since the first release was December 1995 and the
  `/idgames` file is dated 1996-02-21; `MMHELP.TXT` is dated `2/10/96` and `CHANGES.TXT` lists the
  bug-fixes (maps 2, 5, 7, 16, 21, 28, 32, demo bug, MAP21 name bug, map-title order). DDL pins
  this updated build. **Confirmed** (quoted text and file dates) / **Proposed** (that "Feb 95" is
  a typo for Feb 96, on the two independent dates above).

## 2. Canonical archives and byte-identical mirrors

Both files were downloaded in full from all four allowlisted `/idgames` hosts
(`docs/research/upstream-sources.md` §3.4); each host serves the exact same bytes, so each release
is one package record (D-015).

| File | `www.gamers.org` | `youfailit.net` | `ftp.fu-berlin.de` | `mirrors.lug.mtu.edu` |
| --- | --- | --- | --- | --- |
| `themes/mm/mm_allup.zip` | 2,950,178 B `c8607a7a…e7bfb` | same bytes | same bytes | same bytes |
| `themes/mm/mm2.zip` | 4,061,135 B `f4e7a10b…27451` | same bytes | same bytes | same bytes |

**Confirmed.** `/idgames` publishes no checksum; both SHA-256 values were computed here from the
downloaded bytes (2026-10-21).

## 3. Archive contents

Sizes and hashes are of the uncompressed member bytes. Every entry is a regular file with no path
separator, no symlink and no traversal; there is no nested archive. Nothing else is present.

### `mm_allup.zip` — 2,950,178 bytes

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `MM.WAD` | 7,825,282 | `3e34522f61e40e508fbbcf1f32c3de3f9dc9ec5749e2a411bbe646d7ae35a22e` | maps |
| `MMMUS.WAD` | 885,387 | `3e1618f79d6e045ce18f7f72f8a329595e475b1d61b19264bc6b73d615345e73` | music |
| `MM.TXT` | 2,893 | `33f67e460161ae8153fe0c06735d9c6d827736b716ace24a3519db0f0da45bee` | credits (keep) |
| `AUTHORS.TXT` | 3,018 | `0b74c28fc556094dfa4a62a97440a7eb244ec9933f10211e861c05507e734e44` | per-map credits (keep) |
| `MMHELP.TXT` | 4,868 | `19e2cba7843b9fcf37dd5f3de46cd2f80a17a4ba5ba92dbc5a1f782d0b89bc51` | help/load-order (keep) |
| `CHANGES.TXT` | 1,149 | `e3627920cbda386da7317b5299ee28db380301c9c8439b8ba5087c1e1917e82e` | version note (keep) |
| `FILE_ID.DIZ` | 507 | `c82c6d59343dfbb966c34adfed5289f3b10b5bde8fb26b24b17ec8a5e8771b12` | BBS blurb |

### `mm2.zip` — 4,061,135 bytes

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `MM2.WAD` | 11,453,000 | `810c3eb9eb38cd86a5b6ba0bc032d8a9a7a29a58072acbb68d4bb824a8de859b` | maps |
| `MM2MUS.WAD` | 1,075,404 | `d09350612e953c1dc7055dbcb05228c698f7267bdf38d6cb1de11966c7b922d5` | music |
| `MM2.TXT` | 42,580 | `8706785377fb8be6e832c745952c8ab9ecec58736d57667d717329e5b1c354e5` | readme, credits, briefings (keep) |
| `FILE_ID.DIZ` | 440 | `c3994fdd553a22c86b8261a2bdd475b42d8387ce4611ccfce3725dee75781d7e` | BBS blurb (keep) |
| `MM2.BAT` | 32 | `c59a90761c2159f26dbd98958d0b15be43b203436e5bf56733aad55beb56eebb` | DOS one-line launcher (not a member) |

`MM2.BAT` content is exactly `doom2 -file mm2.wad mm2mus.wad`. It is a DOS batch file: DDL never
extracts it as a component and never executes anything from an archive (hard rule).

## 4. WAD inspection (read-only, no execution)

| WAD | Magic | Lumps | Map markers | Other notable lumps |
| --- | --- | --- | --- | --- |
| `MM.WAD` | PWAD | 450 | `MAP01`–`MAP32` (all 32) | `TEXTURE1`, `PNAMES`, `M_DOOM`, `INTERPIC`, `ENDOOM`, `DEMO1`–`DEMO3` |
| `MMMUS.WAD` | PWAD | 35 | none | 35 `D_*` music lumps replace the whole soundtrack |
| `MM2.WAD` | PWAD | 572 | `MAP01`–`MAP32` (all 32) | `TEXTURE1`, `PNAMES`, `M_DOOM`, `TITLEPIC`, `INTERPIC`, `STBAR`, `ENDOOM`, `DEMO1`–`DEMO3` |
| `MM2MUS.WAD` | PWAD | 35 | none | 35 `D_*` music lumps |

- **No `DEHACKED`, `COMPLVL`, `UMAPINFO` or `MAPINFO` lump in any of the four WADs.** The complevel
  therefore cannot come from a lump and must be declared by the recipe. **Confirmed.**
- **Music is not embedded in the map WADs.** Both map WADs carry no `D_*` lumps at all, so the music
  WAD is what supplies the soundtrack and must load after the map WAD (see §5). **Confirmed.**
- **Custom textures are self-contained:** each map WAD carries its own `TEXTURE1`/`PNAMES`, so no
  external texture pack is needed. **Confirmed.**
- **Directory-name padding artifact (Memento Mori).** 167 of `MM.WAD`'s 450 16-byte directory
  entries pad the 8-byte lump name with stray bytes after a NUL — e.g. `MAP32` is stored as
  `4D 41 50 33 32 00 00 53` (`"MAP32\0\0S"`), and the same happens to `DEMO3`, `THINGS`, `SEGS`,
  `NODES` and `REJECT` in the maps. `MM2.WAD` has one such entry (`STBAR`). **Confirmed** (raw
  bytes) / **Proposed**: C library string comparisons stop at the NUL, so the trailing bytes are
  never compared and lump lookup is unaffected — which is the only reading consistent with these
  WADs being playable at all, including every map's `THINGS`/`SEGS`/`NODES`. DDL does not rewrite
  or "fix" the archive.

### DSDA-Doom v0.30.1 handling (pinned)

- No `DEHACKED` lump, so no patch is applied automatically and the recipe must not add `-deh`.
- No `COMPLVL`, so the complevel comes from the recipe's `complevel = "vanilla"` (2 on Doom II,
  `docs/research/dsda-engine.md` §1, D-017); DSDA-Doom's omitted-flag default is 21.
- Launch expectation: `-iwad doom2.wad -file …/MM.WAD …/MMMUS.WAD -complevel 2`.

## 5. Requirements and compatibility

- **Base game.** `MM.TXT`: "Game : DOOM II", "Episode and Level # : MAP01-MAP32"; `MMHELP.TXT`
  Q1: "you'll need the original version of Doom II installed … Version 1.9 is best"; Q6 warns the
  game breaks without v1.9. `MM2.TXT`: "Memento Mori 2 brings you 32 exciting levels", "Doom ][,
  Copyright (C) id Software", install into the Doom 2 directory; `MM2.BAT` runs `doom2`.
  **Confirmed.**
- **Port class.** Vanilla Doom II 1.9 for both (author statements above; DoomWiki lists both as
  Vanilla Doom). `family = "vanilla"`, `complevel = "vanilla"` (cl2). **Confirmed** for the
  author's own target; the resulting DSDA-Doom complevel is **Proposed** until a Deck run.
- **Load order (author-stated).** `MMHELP.TXT` Q4: "use the command line to start Doom II:
  `doom2 -file mm.wad mmmus.wad`". `MM2.BAT`: `doom2 -file mm2.wad mm2mus.wad`. The music WAD
  comes **second**. **Confirmed (quoted).**
- **Warp/episode notes.** One continuous DOOM II mapset each (MAP01–MAP32, no `UMAPINFO`); the menu
  selects the map, so no warp is prescribed. **Proposed.**
- **Single-player.** Both are co-op-first by design but fully playable alone (`MM.TXT`: "Most
  levels were made for co-op play, all levels are playable alone as well"; `MMHELP.TXT` repeats
  it). **Confirmed (quoted).**

## 6. Licence and redistribution

`MM.TXT` / `mm_allup.txt` (**Confirmed (quoted)**):

```text
Authors (MAY/may NOT) use this level as a base to build additional
levels.

-> This depends on every single level, so check out the MMINFO
-> stuff and contact the original author of the map before doing
-> ANYTHING with it. Basically Memento Mori is a set of levels and
-> nobody wants them to be split.

You MAY distribute this WAD, provided you include this file, with
no modifications.  You may distribute this file in any electronic
format (BBS, Diskette, CD, etc) as long as you include this file
intact.  Copyright, The Memento Mori Crew, 1995.
```

`MM2.TXT` (**Confirmed (quoted)**): "Guess what, MM2 is FREE, yes - free to everyone!"; commercial
CD distribution needs prior permission ("write us FIRST. Expect to ship 24 free copies of your CD,
IF we agree"); and the soundtrack terms: "All entries (songs) in the MM2 Music PWAD are original
compositions and are not to be reproduced (extracted) in any way without the consent of the two
persons named above."

**Proposed (DDL policy):** download the unmodified archive and keep the credits/help/changelog
texts (`keep = true`) so the "include this file intact" clause holds; never extract the songs,
never execute `MM2.BAT`, never re-host. Redistribution stays a download from the canonical
`/idgames` mirrors.

## 7. Composition (recipe data — matches `catalog/memento-mori*.toml`)

- **Memento Mori** — one package `free` (`mm_allup.zip`); the required `main` component loads
  `MM.WAD` as `maps` and then `MMMUS.WAD` as `music`, in the order the help file's launch line
  uses. Texts `MM.TXT`, `AUTHORS.TXT`, `MMHELP.TXT`, `CHANGES.TXT` kept.
- **Memento Mori II** — one package `free` (`mm2.zip`); the required `main` component loads
  `MM2.WAD` as `maps` and then `MM2MUS.WAD` as `music`, as `MM2.BAT` does. Texts `MM2.TXT` and
  `FILE_ID.DIZ` kept; `MM2.BAT` is not a member.
- Both: `base_game = doom2`, `compatibility.family = "vanilla"`, `complevel = "vanilla"`,
  engine `=0.30.1` `inspected`, verification `inspected` (desk research only).

## 8. Music credits

- **Memento Mori** — the whole soundtrack is Mark Klem's own MIDI work (DoomWiki soundtrack table,
  secondary; `MMMUS.WAD`'s 35 `D_*` lumps are the shipped form; `MM.TXT`: "NEW music"). Several
  tracks are reused across maps (e.g. "The Wind" on MAP10/MAP24/MAP32). **Confirmed.**
- **Memento Mori II** — `MM2.TXT` "MUSICAL SCORE CREDITS": intro and storyline music by David P.
  Shaw, intermission by Mark A. Klem; odd maps by Klem, even maps (and MAP32) by Shaw. Matches
  DoomWiki's per-map table. **Confirmed (quoted).**

## 9. Category evidence (Classics)

Both are 30-year-old releases with documented community recognition: Doomworld's "Top 100 WADs of
All Time" 1996 feature lists Memento Mori first and Memento Mori II second (DoomWiki pages and the
`doomworld.com/10years/bestwads/1996.php` link they cite; Doomworld itself 403s from this host, so
this is **secondary** evidence, as recorded for Scythe). No Cacoward exists for 1995/1996 releases
(the feature began in 2004), so neither recipe is tagged `cacowards`. Both are 32-map releases →
`megawad`; both are multi-author collaborations → `community-projects` (not `doomer-boards`).

## 10. Left open

- **Deck validation** of both: IWAD selection, complevel 2, `-file` order (map WAD then music WAD),
  single-player gameplay, music playback and map selection are **Needs Deck Validation**.
- **Music loudness/`-nomusic` behaviour** and the MIDI device are **Needs Deck Validation**.
- **Memento Mori II top-secret levels** (MAP01/MAP02 replacements) live inside the separate DOS
  infopack `mm2info.zip` (`MM2INFO.DAT`), not in `mm2.zip`; out of scope, and DDL does not run DOS
  programs.
- **Memento Mori segmented release** (`mm1_3up`…`mm3_3up`, `INSTALL.EXE`, `MM.EXE`) and the
  infopack `mminfoup.zip` are DOS-executable packaging; not used.
- **Screenshots.** None added. The task allows bundling only verified, licensed upstream map
  images; the 1995/96 releases ship no in-game screenshots inside their archives (text-only
  credits) and the crew's homepage is long gone, so there is no verifiable licensed image source
  for this pass. The UI falls back to the painted concept art for these recipes.

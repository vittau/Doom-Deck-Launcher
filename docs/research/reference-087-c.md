# Reference research — Arrival, Vanguard, Lunatic and Overboard (spec 087 worker C, extended by spec 088)

Scope: the four catalog recipes this worker owns — `arrival`, `vanguard`, `lunatic` and
`overboard`. Every identity below was read from the downloaded bytes; nothing was executed or
extracted to disk. Labels: **Confirmed** (observed in this session or quoted from a primary
source), **Proposed** (a DDL inference), **Needs Deck Validation** (cannot be checked on the
macOS arm64 dev host).

Research date: **2026-10-22**. Host: macOS arm64. Tools: `curl`, `python3` (stdlib `zipfile`,
`struct`), `shasum`, `sips`, `scripts/lean-image.sh`. Scratch downloads live in
`.scratch/087c/` (git-ignored, never committed).

## 1. Method and sources

- **Mirrors.** The `/idgames` mirrors `youfailit.net` and `www.gamers.org` timed out or rate-limited
  from this host on the research date, so the packages were fetched from
  `ftp.fu-berlin.de/pc/games/idgames/…` first and then re-downloaded from
  `mirrors.lug.mtu.edu/idgames/…` and `lethe.chinstrap.org/idgames/…`. All three mirrors returned
  **byte-identical archives** for all four packages (SHA-256 compared, §6). **Confirmed.**
- **Case-exact members and hashes.** Member names, uncompressed sizes and SHA-256 come from the ZIP
  central directory plus the decompressed bytes, never from the sidecar `.txt`. **Confirmed.**
- **What was inspected without executing.** ZIP listings; WAD header and lump directory; per-map
  `LINEDEFS`/`SECTORS` records; embedded `DEHACKED`, `MAPINFO`, `EMAPINFO` lumps; every author text
  file. No archive member was run, no IWAD was fetched, no window was opened.
- **Engine evidence.** DSDA-Doom **v0.30.1** source at the pinned tag:
  [`prboom2/src/p_spec.c`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/p_spec.c),
  [`prboom2/src/p_spec.h`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/p_spec.h)
  (generalized-line bases, lines 99–106; generalized dispatch, `p_spec.c:1636`;
  `P_SpawnVanillaExtras`, `p_spec.c:3449–3465` and `p_spec.c:3689`) and
  [`prboom2/src/dsda/mapinfo/`](https://github.com/dsda-org/dsda-doom/tree/v0.30.1/prboom2/src/dsda/mapinfo),
  which contains only `legacy.c`, `u.c` (UMAPINFO) and `hexen.c`. `legacy.c` is the **hardcoded**
  Doom/Doom II progression, not a lump parser: `dsda_LegacyNextMap` holds the static `doom2_next[33]`
  table at lines 128–135 (`MAP05 → 06`, `MAP13 → 14`, `MAP15 → 31`, `MAP31 → 32`, `MAP30 → 01`).
  There is **no ZDoom MAPINFO reader** in v0.30.1. **Confirmed** as source reading, not gameplay.
- **Cacowards pages.** `doomworld.com` blocks automated clients, so the award pages were read from
  raw Wayback `id_` snapshots. The 2011 edition still uses the legacy `/18years/` paths; the 2021
  and 2022 pages use `/cacowards/<year>/bestN/`. **Confirmed.**
- **DoomWiki and the release threads were not used as evidence**: Doomworld/DoomWiki block
  automated fetches. Where a fact could not be read from a primary source it is marked
  **Needs Deck Validation** instead of being filled in.

## 2. Arrival (id `arrival`)

| Field | Value | Evidence |
| --- | --- | --- |
| Title | Arrival | `arrival.txt` |
| Version / date | 1.1.1, 2021-04-16 | `arrival.txt` (`Version`, `Release date`) |
| Authors | Walker "Pavera" Wright & Brayden "AD_79" Hart | `arrival.txt` |
| Game | Doom 2 | `arrival.txt` (`Game: Doom 2`) |
| Maps | 11 slots: MAP01–MAP10 + MAP32 | `arrival.txt` (`MAP01-MAP10, MAP32`) and the WAD directory (**Confirmed**) |
| Advanced engine | `Limit-removing`; `May Not Run With: Vanilla Doom` | `arrival.txt` |
| Music | Yes (AD_79); graphics like Mek's skies | `arrival.txt` |
| DEHACKED | Readme says **No**; the WAD embeds a `DEHACKED` lump (strings only) | WAD inspection (**Confirmed**) |
| `/idgames` | `levels/doom2/Ports/a-c/arrival.zip` | archive path; canonical page <https://www.doomworld.com/idgames/levels/doom2/Ports/a-c/arrival> |
| Cacowards | 2021 Top Twelve, page 3 — winner | <https://www.doomworld.com/cacowards/2021/best3/> (Wayback capture `20211210073539`) |

**Package bytes (Confirmed).** `arrival.zip` 2,552,025 B,
SHA-256 `3907dbe4dfb39129ef2ff3b957640ddea7a7cad4e7d00e4a7b04adaa431992aa`. Members:

| Member | Size | SHA-256 | Loaded / kept |
| --- | --- | --- | --- |
| `arrival.wad` | 7,820,717 | `6e499ff4c0ca7afdd94bf9696be9aaa7594cc767331c56dd8f24d0c9f66acda9` | maps |
| `arrival.txt` | 4,063 | `a343d200cde1e70669df39a5b528ae18ece39f832c3c12d5bfd6f8350a66870d` | kept, unmodified |

The idgames sidecar `arrival.txt` is byte-identical to the member (`a343d200…`), consistent with
the archive upload policy.

**WAD inventory (Confirmed).** `PWAD`, 177 lumps, map markers `MAP01`–`MAP10` and `MAP32`. One
`DEHACKED` lump (806 B) containing a `[STRINGS]` block with `HUSTR_*` map names plus `C1TEXT`
story text — the readme's `Dehacked/BEX Patch: No` line is inaccurate, but nothing is loaded
separately because the patch is embedded. No `COMPLVL`, `UMAPINFO`, `MAPINFO` or ZDoom-only lumps.

**Compatibility reasoning (Proposed, from the author's statement).** The author states
`Limit-removing` and forbids Vanilla Doom, so `family = "limit-removing"` and, by the D-030
convention, `complevel = "vanilla"` (2 on Doom II). Inspection adds one nuance: ten of the eleven
maps place **line special 271** (regular sky transfer), 103 lines in total, and no Boom generalized
linedef types. DSDA-Doom v0.30.1 applies 271 in every complevel — `P_SpawnVanillaExtras` documents
it as *"allow MBF sky transfers in all complevels"* (`p_spec.c:3449–3465`) and is the vanilla-mode
path (`p_spec.c:3689`) — so the custom skies still render at complevel 2. **Needs Deck Validation:**
that the skies render as intended on the Deck.

**Metadata and progression (Confirmed).** No `COMPLVL`, `MAPINFO`, `EMAPINFO` or `UMAPINFO`
exists. The hardcoded Doom II table in DSDA-Doom's `legacy.c` (`MAP10 → 11`) therefore continues
from the custom MAP10 into the base IWAD's stock MAP11 unless the player quits. **Needs Deck
Validation:** the post-MAP10 transition.

**Tags (Confirmed citations).** `cacowards` = 2021 Top Twelve page 3 winner (page linked above;
the blurb's "9 maps" is the Cacowards author's count, while the release and WAD have eleven slots).
`classics` = released 2021-04-16, more than five years before this review, plus that recognition.
Not `episode`: eleven slots exceed the nine-map rule in `catalog/tags.toml`.

**Artwork.** One in-game screenshot from the Cacowards 2021 feature
(`static.doomworld.com/pages_media/10_arrival.png`, 1280×960, original, archived on
2025-04-30). Provenance in `crates/ddl/ui/assets/screenshots/arrival/SOURCES.toml`.

## 3. Vanguard (id `vanguard`)

| Field | Value | Evidence |
| --- | --- | --- |
| Title | Vanguard | `Vanguard.txt` |
| Date | 2011-05-11 | `Vanguard.txt`; mirror `Last-Modified: Wed, 11 May 2011 20:18:52 GMT` |
| Author | Paul DeBruyne (skillsaw) | `Vanguard.txt` |
| Game | Doom 2 | `Vanguard.txt` |
| Maps | 13: MAP01–MAP13 (12 main + a bonus thirteenth "for laughs") | `Vanguard.txt` (`MAP01-MAP13`, `Map #: 1-13`) and the WAD directory (**Confirmed**) |
| Advanced engine | `Boom compatible, limit removing ports`; description says `12 Boom format levels for PrBoom+, ZDoom, or Eternity` | `Vanguard.txt` |
| DEHACKED / MAPINFO | Readme: `Dehacked/BEX Patch: Yes; level names only`. WAD embeds `DEHACKED` and a **ZDoom-syntax** `MAPINFO` (not a Boom format) | WAD inspection (**Confirmed**) |
| `/idgames` | `levels/doom2/Ports/v-z/vanguard.zip` | canonical page <https://www.doomworld.com/idgames/levels/doom2/Ports/v-z/vanguard> |
| Cacowards | 2011 Top Ten, page 2 — winner | <https://www.doomworld.com/cacowards/2011/>; the edition page is <https://www.doomworld.com/18years/best2.php> (Wayback capture `20111211085659`) |

**Package bytes (Confirmed).** `vanguard.zip` 9,910,540 B,
SHA-256 `d37b5bd01b7c9902be97f1d42c1c1f68cf4aaeb9ac97ab8cce88fcd1b459dbef`. Members:

| Member | Size | SHA-256 | Loaded / kept |
| --- | --- | --- | --- |
| `Vanguard.wad` | 34,897,578 | `52807ec27756d23dedc388c8c39b953797fa2da1c8339303de63141a0cebc465` | maps |
| `Vanguard.txt` | 4,223 | `ceab9fcf5532157aa65fa5f3a69143d2a8711c86ef5bd6dd066340eca047bdf2` | kept, unmodified |

**WAD inventory (Confirmed).** `PWAD`, 3,208 lumps, thirteen map markers. One `DEHACKED` lump
(1,189 B, `HUSTR_*` level names only, `Engine config = boom.cfg`). One `MAPINFO` lump (1,302 B) in
**ZDoom syntax** (`defaultmap`, `map MAP01 "…"` blocks with `titlepatch`, `next`, `secretnext`,
`music`, `map07special`) — not a Boom format. No `COMPLVL` or `UMAPINFO`.

**Compatibility reasoning (Confirmed by inspection, not only the shorthand).** The readme's
headline `Boom compatible, limit removing ports` is shorthand; its own description names the
format directly — *"12 Boom format levels"* — and the bytes agree: Boom extended / generalized
specials appear in **all thirteen maps** (407 generalized lines in total: 213 floor, 88 locked-door,
43 door, 42 lift, 20 ceiling, 1 stairs). DSDA-Doom only recognizes those types outside demo
compatibility (`p_spec.c:1636`, bases in `p_spec.h:99–106`), and vanilla Doom does not implement
them at all. `family = "boom"`, `complevel = "boom"` (9).

**Metadata and progression (Confirmed).** The `MAPINFO` lump is ZDoom syntax and DSDA-Doom v0.30.1
has no ZDoom MAPINFO reader (`prboom2/src/dsda/mapinfo/` holds only `legacy.c`, `u.c`, `hexen.c`),
so its titles and per-map music do not apply. With no `UMAPINFO`/`COMPLVL`, progression is the
hardcoded Doom II table in `legacy.c` (`MAP13 → 14`): after the custom MAP13 the engine loads the
base IWAD's stock MAP14 unless the player quits. **Needs Deck Validation:** the titles/music actually
heard and the post-MAP13 transition.

**Tags.** `cacowards` = 2011 Top Ten page 2 winner (links above). `classics` = 2011-05-11, over five
years old, plus that recognition. Not `episode`/`megawad`: thirteen slots.

**Artwork.** No obtainable image: the 2011 Cacowards pages carry only the award icon, and the
release thread is not fetchable from this host. The painted fallback art stays; this report records
the limitation.

## 4. Lunatic (id `lunatic`)

| Field | Value | Evidence |
| --- | --- | --- |
| Title | Lunatic | `Lunatic.txt` |
| Date | 2011-08-11 | `Lunatic.txt`; mirror `Last-Modified: Thu, 11 Aug 2011 21:19:25 GMT` |
| Author | Paul DeBruyne (skillsaw) | `Lunatic.txt` |
| Game | Doom 2 | `Lunatic.txt` |
| Maps | 5: MAP01–MAP05 | `Lunatic.txt` (`MAP01-MAP05`) and the WAD directory (**Confirmed**) |
| Advanced engine | `Boom compatible, limit removing ports`; description: `Lunatic includes 5 Boom-format maps` | `Lunatic.txt` |
| DEHACKED / MAPINFO | Readme: `Dehacked/BEX Patch: Yes`, `Other: MAPINFO/EMAPINFO`. WAD embeds `DEHACKED`, a **ZDoom-syntax** `MAPINFO` and an **Eternity** `EMAPINFO` | WAD inspection (**Confirmed**) |
| `/idgames` | `levels/doom2/Ports/j-l/lunatic.zip` | canonical page <https://www.doomworld.com/idgames/levels/doom2/Ports/j-l/lunatic> |
| Cacowards | 2011 Top Ten, page 1 — winner | <https://www.doomworld.com/cacowards/2011/>; edition page <https://www.doomworld.com/18years/best1.php> (Wayback capture `20111210235724`) |

**Package bytes (Confirmed).** `lunatic.zip` 4,058,779 B,
SHA-256 `01e393282b23af6e854d8925f958ad7fb769f004cdfaa97b815bb0d9b999e616`. Members:

| Member | Size | SHA-256 | Loaded / kept |
| --- | --- | --- | --- |
| `Lunatic.wad` | 18,160,632 | `4ef88cf44d7d15effdf634811e3ae21fceaee21336870879c95af75b08783c9f` | maps |
| `Lunatic.txt` | 3,579 | `93f0b9a52f1006c49fef41c341d86c5d3d83992418cafdd12fb00beb290f73cf` | kept, unmodified |

**WAD inventory (Confirmed).** `PWAD`, 2,452 lumps, five map markers. `DEHACKED` (3,161 B) changes
the SS Nazi into a modified Keen type, edits several frames and sets `HUSTR_*` level names
(`Engine config = boom.cfg`). One `MAPINFO` lump (544 B) in **ZDoom syntax** (`defaultmap`,
`sky1 ZDOOMSKY`, `map MAP01 "…"` blocks with `titlepatch`/`next`/`secretnext`/`music`) and one
`EMAPINFO` lump (584 B) in **Eternity's** `[MAP01] levelname = …` format. Neither is a Boom format;
they are two different ports' metadata dialects carried for convenience. No `COMPLVL` or
`UMAPINFO`.

**Compatibility reasoning (Confirmed by inspection).** *"5 Boom-format maps"* plus the bytes:
generalized line types occur in **all five maps** (162 lines: 104 floor, 24 ceiling, 21
locked-door, 10 door, 3 lift) alongside Boom extended specials 213 (floor light), 253 (line
scroll) and 261 (ceiling light). Vanilla compatibility does not implement the generalized types
(`p_spec.c:1636`), so `family = "boom"`, `complevel = "boom"` (9).

**Metadata and progression (Confirmed).** DSDA-Doom v0.30.1 reads neither ZDoom `MAPINFO` nor
Eternity `EMAPINFO` (`prboom2/src/dsda/mapinfo/` holds only `legacy.c`, `u.c`, `hexen.c`), so the
custom map titles and per-map music do not apply. With no `UMAPINFO`/`COMPLVL`, progression is the
hardcoded Doom II table in `legacy.c` (`MAP05 → 06`): after the custom MAP05 the engine loads the
base IWAD's stock MAP06 unless the player quits. **Needs Deck Validation:** the titles/music heard
and the post-MAP05 transition.

**Tags.** `cacowards` = 2011 Top Ten page 1 winner. `classics` = 2011-08-11, over five years old,
plus that recognition. `episode` = **five maps**, counted from the WAD directory and the author's
map list (the nine-or-fewer rule).

**Artwork.** No obtainable image (same 2011 page limitation); the painted fallback art stays.

## 5. Overboard (id `overboard`, added by spec 088)

| Field | Value | Evidence |
| --- | --- | --- |
| Title | Overboard | `overboard.txt` |
| Date | 2022-04-08 (readme); idgames file dated 2022-09-27, an update "to fix multiplayer issues" | `overboard.txt`; mirror `Last-Modified: Tue, 27 Sep 2022 00:28:31 GMT` |
| Author | mouldy (Cyriak Harris) | `overboard.txt` (`Author: mouldy`); public author identity as in `catalog/going-down.toml` |
| Game | Doom 2 | `overboard.txt` |
| Maps | 15 slots: MAP01–MAP14 + MAP31 (six main, a game-over map, New Game Plus, MAP31) | `overboard.txt` (`New levels: 15`, `MAP01-14 & MAP31`) and the WAD directory (**Confirmed**) |
| Advanced engine | `Boom`; description: `6 maps … (boom format, complevel 9)`; `May Not Run With: non-boom ports` | `overboard.txt` |
| DEHACKED | Yes, embedded (`level names`, pickup strings, a few frames) | readme + WAD inspection (**Confirmed**) |
| `/idgames` | `levels/doom2/Ports/megawads/overboard.zip` | canonical page <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/overboard> |
| Cacowards | 2022 Top Ten, page 3 — winner | <https://www.doomworld.com/cacowards/2022/best3/> (Wayback capture `20221210051456`) |
| Licence | CC BY-NC-ND 4.0 | `overboard.txt` (`Copyright / Permissions`) |

**Package bytes (Confirmed).** `overboard.zip` 7,728,312 B,
SHA-256 `9dddda5bb754192c97507646c65f214a378206015d5fa1995bf75269bb02356a`. Members:

| Member | Size | SHA-256 | Loaded / kept |
| --- | --- | --- | --- |
| `overboard.wad` | 25,265,537 | `608218886e8a61086040d28045d81644801c33d1265f343cd6cf62d36f7b3e95` | maps |
| `overboard.txt` | 3,135 | `b4cf202073939a044f361fa1180e23b789becf6b9c82f42996fd96a5d37f3ce2` | kept, unmodified |

**WAD inventory (Confirmed).** `PWAD`, fifteen map markers (`MAP01`–`MAP14`, `MAP31`), one
`DEHACKED` lump (1,412 B) with frame edits, `GOTBLUECARD`/`PD_BLUEO` pickup text and `HUSTR_*`
level names. No `MAPINFO`, `UMAPINFO` or `COMPLVL` — the author's text is the only compatibility
statement.

**Compatibility reasoning (Confirmed author value + inspection).** The author states
`complevel 9` explicitly, *"boom format, complevel 9"*. Generalised Boom line types appear in **all
fifteen maps** (1,061 lines: 637 floor, 207 door, 164 lift, 29 ceiling, 11 locked-door, 11 stairs,
2 crusher), which vanilla compatibility does not implement (`p_spec.c:1636`). `family = "boom"`,
`complevel = "boom"` (9), the alias that names 9.

**Metadata and progression (Confirmed).** No `COMPLVL`, `MAPINFO`, `EMAPINFO` or `UMAPINFO`
exists. The embedded DEHACKED only relabels maps and pickups. The author's "game over" map and New
Game Plus are ordinary custom maps; the hardcoded Doom II table in `legacy.c` (`MAP14 → 15`) then
continues from the custom MAP14 into the base IWAD's stock MAP15 unless the player quits. **Needs
Deck Validation:** the New Game Plus flow and the post-MAP14 transition.

**Tags.** `cacowards` = 2022 Top Ten page 3 winner (link above). Not `episode`: fifteen slots, even
though the Cacowards blurb says "6 maps" (the six-map core is not the release's slot count). Not
`classics`: released 2022-04-08, less than five years before this review. Not `challenging`: the
author calls the difficulty "average".

**Artwork.** One in-game screenshot from the Cacowards 2022 feature
(`static.doomworld.com/pages_media/13_overboard.png`, 1280×960, archived 2024-12-14). Provenance in
`crates/ddl/ui/assets/screenshots/overboard/SOURCES.toml`.

## 6. Headless Chicken (id `headless-chicken`, added by spec 089)

The DSDA identity page (<https://dsdarchive.com/wads/hdlschkn>) names Luca 'Chookum' Pavone and
lists 33 maps; its demo table records `DSDA-Doom …cl9` for every map. The **original archive
release** is the idgames upload `levels/doom2/Ports/megawads/hdlschkn.zip` (its readme asks the
archive to place it in `levels/doom2/ports/megawads`), not DSDA's separate archive copy
(`/files/wads/doom2/5131/hdlschkn.zip`, 35,557,932 B, a different RC that was inspected and
discarded). **Confirmed.**

| Field | Value | Evidence |
| --- | --- | --- |
| Title | Headless Chicken | `hdlschkn.txt` |
| Release | RC1 2023-09-12, RC13 2025-10-25 (readme); idgames file dated 2026-02-02 | `hdlschkn.txt`; mirror `Last-Modified` |
| Author | Luca 'Chookum' Pavone | `hdlschkn.txt`; DSDA identity page |
| Game | Doom 2 | `hdlschkn.txt` |
| Maps | 33: MAP01–MAP33 in five UMAPINFO episodes | `hdlschkn.txt` ("33 levels") and the WAD directory (**Confirmed**) |
| Advanced engine | `Boom (-complevel "9") or GZDoom/Zandro for extras`; COMPLVL lump `boom` | `hdlschkn.txt`; WAD lump inspection (**Confirmed**) |
| DEHACKED | Yes, embedded; changes weapon timings, projectiles and map names | `hdlschkn.txt`; WAD inspection |
| `/idgames` | `levels/doom2/Ports/megawads/hdlschkn.zip` | canonical page <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/hdlschkn> |

**Package bytes (Confirmed).** `hdlschkn.zip` 38,085,270 B,
SHA-256 `de8926c8d3c22dc739718eb06c18da994f3a9fcd12c0f404cbc4c342ec5c4659`. Members:

| Member | Size | SHA-256 | Loaded / kept |
| --- | --- | --- | --- |
| `HDLSCHKN.wad` | 126,358,308 | `5b7620307c6c8f0203508c1e380a563dda0d62172e65f32406166c3e427380ab` | maps |
| `hdlschkn.txt` | 27,158 | `1a09ef102856ba673bc4b5dcce954b630be97dca086259ef0490e7ee0a1d3ca3` | kept, unmodified |

**WAD inventory (Confirmed).** `PWAD`, 8,243 lumps, 33 map markers. `COMPLVL` is the four bytes
`boom`. `UMAPINFO` (10,823 B) declares five episodes (`episode = "E1TITLE", "Sticks 'n' Stones"` …
`"That Pesky Heck"`), per-map names/music/sky, `MAP30 endcast = true`, and secret returns
`MAP31/MAP32/MAP33 → MAP16`. `DEHACKED` is embedded. The ZDoom-syntax `MAPINFO` (13,241 B,
`gameinfo { … }`), Eternity `EMAPINFO` (5,468 B) and the GZDoom `DECORATE`, `ZSCRIPT`, `SNDINFO`,
`LANGUAGE`, `GLDEFS` lumps are the author's "extras" and are inert under DSDA-Doom v0.30.1.

**Compatibility reasoning (Confirmed).** The author states Boom/complevel 9, the WAD's own
`COMPLVL` says `boom`, DSDA demo entries record cl9, and every one of the 33 maps uses Boom
generalized line types (2,022 lines: 937 floor, 610 lift, 285 locked-door, 98 door, 92 ceiling),
which vanilla compatibility does not implement (`p_spec.c:1636`). `family = "boom"`,
`complevel = "boom"`. **Needs Deck Validation:** UMAPINFO episode menu, `endcast`, secret returns,
DEHACKED effects and performance.

**Tags.** `megawad` = 33 maps, counted from the WAD and the readme. No Cacowards entry was found
for this release in `docs/research/cacowards.md`, so `cacowards` is not claimed.

**Artwork.** No obtainable source: the release is not covered by the Cacowards inventory and the
Doomworld release thread is not fetchable from this host. The painted fallback art stays.

## 7. Micro-Slaughter Community Project (id `micro-slaughter-community-project`, added by spec 089)

| Field | Value | Evidence |
| --- | --- | --- |
| Title | Micro-Slaughter Community Project | `MSCP_v1b.txt` |
| Version / date | 1b, 2021-01-27 (readme); idgames file dated 2026-09-07 | `MSCP_v1b.txt`; mirror `Last-Modified` |
| Authors | Bauul (project lead) + 14 credited mappers | `MSCP_v1b.txt` map credits; UMAPINFO `author` fields |
| Game | DOOM2 | `MSCP_v1b.txt` |
| Maps | 19: MAP01–MAP17, MAP31, MAP32 | `MSCP_v1b.txt` (`Map01 - Map17, Map31, Map32`) and the WAD directory (**Confirmed**) |
| Advanced engine | `PRBoom+ compatible`; `Target format: PRBoom+ complevel 9`; `May Not Run With: Chocolate Doom, Crispy, or anything that doesn't support PRBoom+ maps` | `MSCP_v1b.txt` |
| DEHACKED | Yes, embedded | readme + WAD inspection (**Confirmed**) |
| `/idgames` | `levels/doom2/Ports/megawads/mscp_v1b.zip` | canonical page <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/mscp_v1b> |

**Package bytes (Confirmed).** `mscp_v1b.zip` 25,051,502 B,
SHA-256 `0e708e0e871a583d522b09c45ba2f00466a66c957f408aad5f710e19fb1a09cc`. Members:

| Member | Size | SHA-256 | Loaded / kept |
| --- | --- | --- | --- |
| `MSCP_v1b.wad` | 86,441,134 | `c28e7d90342abd2bab37a243f1078b44dc8bce22e2ee6490f8009d5502f02cf6` | maps |
| `MSCP_v1b.txt` | 5,455 | `57048764375bd4e8129c6a4cd2609854d17498535186c4e1b861fe7ef6073a4e` | kept, unmodified |

**WAD inventory (Confirmed).** `PWAD`, 2,554 lumps, 19 map markers. `UMAPINFO` (4,036 B) sets each
map's title, `next`, par time (300 s), music and author, and marks `MAP17` ("Epilogue Tour") with
`endgame = true`; `MAP31`/`MAP32` return to `MAP16`. `DEHACKED` is embedded. `ZMAPINFO` (ZDoom) and
`DECORATE` are inert for DSDA-Doom. No `COMPLVL`.

**Compatibility reasoning (Confirmed).** The author states `PRBoom+ complevel 9` and lists DSDA
0.29.4 among the tested engines; 18 of the 19 maps use Boom generalized line types (1,002 lines:
595 floor, 307 ceiling, 56 lift, 30 locked-door, 14 door), which vanilla compatibility does not
implement (`p_spec.c:1636`). `family = "boom"`, `complevel = "boom"`. The author *strongly
recommends* pistol-starting each map, so the recipe records `pistol_start = true` as a profile
suggestion only.

**Tags.** `community-projects` = fifteen credited mappers led by Bauul (readme map credits and
UMAPINFO `author` fields). Not `megawad` (19 slots) or `episode` (more than nine). No Cacowards
entry was found in the researched inventory, so `cacowards` is not claimed.

**Artwork.** No obtainable source was found within the bounded research budget (the release is not
in the Cacowards inventory, and Doomworld/DoomWiki block automated fetches). The painted fallback
art stays.

## 8. Mirror verification

All six archives were downloaded from three allowlisted HTTPS mirrors and hashed. Every mirror
returned the exact size and SHA-256 of §2–§7. **Confirmed** on 2026-10-22.

| Package | ftp.fu-berlin.de | mirrors.lug.mtu.edu | lethe.chinstrap.org |
| --- | --- | --- | --- |
| `arrival.zip` | 2,552,025 B / `3907dbe4…92aa` | match | match |
| `vanguard.zip` | 9,910,540 B / `d37b5bd0…dbef` | match | match |
| `lunatic.zip` | 4,058,779 B / `01e39328…e616` | match | match |
| `overboard.zip` | 7,728,312 B / `9dddda5b…356a` | match | match |
| `hdlschkn.zip` | 38,085,270 B / `de8926c8…4659` | match | match |
| `mscp_v1b.zip` | 25,051,502 B / `0e708e0e…09cc` | match | match |

`www.gamers.org` rate-limits bursts and `youfailit.net` timed out from this host on the research
date; they are not listed as mirrors in the recipes. `ftpmirror.infania.net` returned 404 for the
`/idgames/…` base path used here, so it is not listed either. No host was added to
`catalog/hosts.toml`.

## 9. Verification commands and results

Run from the worktree root on 2026-10-22, with `CARGO_TARGET_DIR=/Users/vitor/git/Doom-Deck-Launcher/target`
and `CARGO_BUILD_JOBS=2`:

| Command | Result |
| --- | --- |
| `ddl catalog lint catalog/{arrival,vanguard,lunatic,overboard,headless-chicken,micro-slaughter-community-project}.toml` | `Catalog lint passed (offline).` |
| `ddl catalog lint --fetch …` (same six paths) | `Catalog lint passed (fetched packages verified).` |
| `cargo test -p ddl-core --test catalog_087_c -- --test-threads=2` | 9 passed, 0 failed |
| `cargo clippy -p ddl-core --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |

## 10. Open checks (all **Needs Deck Validation**)

- IWAD detection and launch with the six recipes on a real Deck (DOOM II, no variants).
- Complevel application: `vanilla` (2) for Arrival looks right for skies because DSDA-Doom applies
  special 271 in every complevel; confirm the custom skies and that no Boom-only behaviour is
  expected. `boom` (9) for Vanguard, Lunatic, Overboard, Headless Chicken and MSCP; Headless
  Chicken also ships `COMPLVL boom`.
- Embedded DEHACKED application (map names, Lunatic's modified Keen/SS behaviour, Overboard's
  pickup strings, Headless Chicken's weapon timings) and UMAPINFO application for Headless Chicken
  (five episodes, `endcast`, secret returns) and MSCP (titles, par times, `endgame`).
- ZDoom `MAPINFO`/`EMAPINFO`/`ZMAPINFO` and GZDoom `DECORATE`/`ZSCRIPT` lumps are inert in
  DSDA-Doom v0.30.1: confirm the map titles, music and any extras actually seen on the Deck.
- Progress and endings: Arrival MAP10 + MAP32, Lunatic MAP05 and Vanguard MAP13 (the hardcoded
  table continues into the base IWAD's MAP06/MAP14 — quit from the intermission to stop),
  Overboard's New Game Plus and post-MAP14 fallback, Headless Chicken's episodes and `endcast`,
  MSCP's MAP17 `endgame` and MAP31/MAP32 returns.
- Performance (unmeasured): Headless Chicken is 126 MB across 33 maps with 8,243 lumps; MSCP is
  slaughter combat with 300-second par times. No framerate, memory or battery figures are claimed.
- Audio (every release ships a MIDI soundtrack) and controller navigation of the map/episode menus.
- Artwork: Vanguard, Lunatic, Headless Chicken and MSCP have no bundled screenshot (fallback art);
  confirm the fallback cards look acceptable on the Deck.

# Reference research — Valiant (Paul "skillsaw" DeBruyne)

**Status:** desk research for a D-010 catalog recipe · **Gathered:** 2026-10-16 (UTC)
**Author workstream:** Content Catalog (catalog batch A)
**Scope:** facts needed to author a DDL recipe for **Valiant** (the canonical `/idgames` release).
No code is produced here.

Labels used throughout:

- **Confirmed** — verified from an official source, upstream source code, or a downloaded archive
  (evidence recorded below).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

All downloads went to `.scratch/` (git-ignored, never committed). Nothing was executed; the ZIP was
read by `zipfile` and `Valiant.wad` only parsed as data by a read-only lump-directory reader.
Doomworld returns Cloudflare HTTP 403 from this host, so page metadata is taken from the `/idgames`
search index and the archive master's listing; the bundled `Valiant.txt` is the author's primary
statement.

```sh
curl -sSL -A "DoomDeckLauncher catalog review" -o valiant.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/valiant.zip"
shasum -a 256 valiant.zip
```

## Quick reference

| | Valiant |
| --- | --- |
| Release | First released 2015-02-07; canonical `/idgames` build 2015-05 (`Valiant.wad` dated 2015-05-16) |
| Author | Paul "skillsaw" DeBruyne |
| Archive | `levels/doom2/Ports/megawads/valiant.zip` |
| Archive size / SHA-256 | 25,605,504 bytes / `067d649f2a845386ea20ad20b458125143ead3e6f1dbfb4e161ca0eb8f159121` |
| Main WAD | `Valiant.wad` (MAP01–MAP32) |
| IWAD | `doom2.wad` |
| Format / complevel | MBF → **complevel 11** (author-stated) |
| Embedded patch | one MBF `DEHACKED` lump (26,657 B); ZDoom `MAPINFO`/`DECORATE`/… (inert under v0.30.1) |
| `COMPLVL` lump | none |
| Optional components | none in the official package |
| Licence | distribute with `Valiant.txt` intact; no base for reuse |

---

## 1. Official sources

- **Canonical `/idgames` page:** <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/valiant>.
  Title "Valiant", filename `levels/doom2/Ports/megawads/valiant.zip`, author "Paul \"skillsaw\"
  DeBruyne". **Confirmed.**
- **Release text `Valiant.txt`** (inside the ZIP, 12,479 bytes): `Advanced engine needed : MBF
  Compatible`; `Release date : February 7th, 2015`; *"Valiant is a MBF megawad for Doom II featuring
  32 new maps spanning 5 themed episodes."*; `Game : Doom 2`, `Map # : MAP01-MAP32`; the port notes
  recommend *"PrBoom-plus (complevel 11)"*; `May Not Run With…` warns it needs MBF code pointers or
  DECORATE and ZDBSP extended nodes. **Confirmed (quoted).**
- **Version (dated observations):** the ZIP as served by the archive carries `Valiant.wad` dated
  2015-05-16 (the May update of the February release); the archive listing dates the file
  2015-05-30. The Doom Speed Demo Archive keeps "Valiant (2015-04)" and "Valiant (2015-05)" builds,
  matching this second build. **Confirmed.**
- **No newer `/idgames` build.** In 2024 skillsaw published an MBF21 update on Bethesda.net /
  Dropbox, and there is a separate *Valiant: Vaccinated Edition* (its own `/idgames` entry,
  `valve.zip`, 2015-12). Neither is part of `valiant.zip`; the MBF21 update is **not** on `/idgames`
  nor on an allowlisted host, so this recipe uses the canonical `/idgames` release. **Confirmed.**

## 2. Canonical archive and byte-identical mirrors

The `/idgames` path is `levels/doom2/Ports/megawads/valiant.zip`; every mirror serves that
`<dir>/<filename>` under its own base (`upstream-sources.md` §3.4). DDL uses direct mirror URLs,
never the `/idgames` API (D-015). Three allowlisted hosts were **downloaded in full**; all serve
the exact same bytes:

| Host (allowlisted) | URL | bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/valiant.zip` | 25,605,504 | `067d649f…59121` |
| `youfailit.net` | `https://youfailit.net/pub/idgames/levels/doom2/Ports/megawads/valiant.zip` | 25,605,504 | `067d649f…59121` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/Ports/megawads/valiant.zip` | 25,605,504 | `067d649f…59121` |
| `ftp.fu-berlin.de` | `https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/Ports/megawads/valiant.zip` | 25,605,504 | `067d649f…59121` |

**Confirmed:** byte-identical, so they share one package record (D-015). `/idgames` supplies no
checksum; the SHA-256 was computed here from the canonical bytes.

## 3. Archive contents (ZIP, 25,605,504 bytes)

Everything in the archive is listed with its exact size and SHA-256 computed here. Nothing else is
present; there is no separate music pack or alternate WAD.

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `Valiant.wad` | 96,578,759 | `33f079a83d8b7f5706d5995917c5b0f657da0587df497c352d58b47762bd8c51` | maps |
| `Valiant.txt` | 12,479 | `97c817937ce5cf5a6638696ec215fd7e9af757f6572de3925221c014a85c2090` | credits (keep) |

**Confirmed.** Flat layout, ASCII names, both entries Deflate, no symlinks, no traversal.

## 4. WAD inspection (read-only, no execution)

`Valiant.wad` is a **PWAD**, 96,578,759 bytes, **5,185 lumps**, map markers `MAP01`–`MAP32` (stored
in the author's build order). **Confirmed.**

- **One embedded `DEHACKED` lump** (26,657 bytes), `Patch File for DeHackEd v3.0`, authored with
  WhackEd4 1.0.1, `Doom version = 21`, `Patch format = 6`. It implements the custom monsters and
  weapons in **MBF** format ("an extensive MBF format DeHackEd patch", `Valiant.txt`).
  **Confirmed.**
- ZDoom/Eternity-only lumps are present: `MAPINFO` (6,364 B), `EMAPINFO` (6,874 B), `DECORATE`
  (17,596 B), `GLDEFS` (550 B), `SNDINFO` (105 B), `LANGUAGE` (133 B). **Confirmed.**
- **No `COMPLVL` lump** and **no `UMAPINFO` lump** are present. The complevel cannot come from a
  lump and must be declared in the recipe. **Confirmed.**

### DSDA-Doom v0.30.1 handling (pinned)

DDL launches with `-noautoload` (D-011), which skips directory autoload but not embedded patches:

- The embedded MBF `DEHACKED` lump is applied automatically because `-nodeh` is absent
  ([`prboom2/src/d_main.c:2147–2156`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2147-L2156)).
  The recipe must **not** add a separate `-deh` entry (it would double-apply). This is the patch
  that supplies Valiant's new monsters/weapons; DSDA-Doom supports MBF DeHackEd.
- **ZDoom `MAPINFO` support was removed in v0.30.0** ("Removed ZDoom MAPINFO support",
  <https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.0>). Valiant's `MAPINFO` (episode
  definitions) and `EMAPINFO`/`DECORATE`/`GLDEFS`/`SNDINFO`/`LANGUAGE` are ZDoom/Eternity-only and
  **inert** under v0.30.1. The author documents the consequence: *"In PrBoom-plus, Valiant plays
  like a conventional 32 level megawad with a death exit at the end of each episode."* and the
  final boss "will graphically glitch in ports without DECORATE support but this does not impact
  gameplay". **Confirmed.**
- **`COMPLVL` absent**, so the complevel comes from the recipe's explicit `-complevel 11`.

**Confirmed.** The launch expectation is `-iwad doom2.wad -file …/Valiant.wad -complevel 11` with no
`-deh` argument.

## 5. Requirements and compatibility

- **Base game:** DOOM II — `Valiant.txt` says `Game : Doom 2`, `Map # : MAP01-MAP32`. **Confirmed.**
- **Required port class:** MBF — `Advanced engine needed : MBF Compatible`; requires MBF code
  pointers (or DECORATE) and ZDBSP extended nodes. DSDA-Doom supports all of these. **Confirmed.**
- **Complevel:** **11 (MBF)** — from the author's `PrBoom-plus (complevel 11)` recommendation,
  `family = "mbf"`. Recorded as `proposed` because it has not been Deck-tested (CONTRIBUTING.md
  step 5). **Proposed.**
- **Pistol start (context):** *"Each map is designed to be played from a pistol start, but some
  effort has been made to support continuous play as well."* Left to the player profile; the recipe
  prescribes no `pistol_start`. **Confirmed (context) / Proposed (not encoded).**
- **Warp/episode notes:** single continuous DOOM II mapset (MAP01–MAP32; the five "episodes" are
  not engine episodes under DSDA-Doom). No `-warp` is prescribed. **Proposed.**
- **DEH/BEX:** the patch is embedded and automatic (see §4). **Confirmed.**

## 6. Licence and redistribution

`Valiant.txt` states (**Confirmed (quoted)**):

```text
Authors may NOT use the contents of this file as a base for modification or
reuse.  Permissions have been obtained from original authors for any of
their resources modified or included in this file.
...
You MAY distribute this file, provided you include this text file, with no
modifications.  You may distribute this file in any electronic format (BBS,
Diskette, CD, etc) as long as you include this file intact.  I have
received permission from the original authors of any modified or included
content in this file to allow further distribution.
```

**Proposed (DDL policy):** DDL downloads the unmodified ZIP and keeps `Valiant.txt` (`keep = true`),
so the "include this file intact" clause is satisfied. The file lists extensive texture, sprite,
music and sound credits (id Software, Raven Software, 3D Realms, and many community authors).

## 7. Composition (recipe data — matches `catalog/valiant.toml`)

- One package `free` (the ZIP above), one required `main` component loading `Valiant.wad` as `maps`.
  No optional components: the official package ships nothing else.
- Keep `Valiant.txt` as `keep = true`.
- `base_game = doom2`; `compatibility.family = "mbf"`, `complevel = 11`,
  `complevel_status = "proposed"`; engine `=0.30.1`, `inspected`.

## 8. Open questions

- **Complevel 11.** Not declared by any lump; proposed from the author's statement and needs a Deck
  run (**Needs Deck Validation**).
- **`MAPINFO` inertness.** Episodes and splash screens are lost under DSDA-Doom (by design of
  v0.30.0). The continuous-play flow with per-episode death exits must be confirmed acceptable on
  the Deck (**Needs Deck Validation**).
- **Alternate official builds not used:** the 2024 MBF21 update (Bethesda.net/Dropbox) and the
  Vaccinated Edition (`valve.zip`) are separate releases; the latter could be its own recipe.


## Screenshots (spec 023, D-024)

Bundled under `crates/ddl/ui/assets/screenshots/valiant/`; the `SOURCES.toml` beside the
images records each file's source URL, source page, credit, original size, retrieval date
and any permission text found. Retrieved 2026-10-17; each file is a 640x360 JPEG card crop
of the original below.

The screenshots are the author's own in the mapset's release thread (`doomworld.com/vb/thread/71704`); Doomworld blocks automation, so the thread was read from its Wayback Machine snapshot and the images fetched from their original `i.imgur.com` URLs.

| Asset | Source image URL | Source page | Credit | Original |
| --- | --- | --- | --- | --- |
| `01.jpg` | <https://i.imgur.com/8bLhSJ5.png> | <https://www.doomworld.com/vb/thread/71704> | Paul "skillsaw" DeBruyne | 1920x1080 (`8bLhSJ5.png`) |

**Why representative:** The shots are in-game views of Valiant's tech-base maps, not the title screen.

# Reference research — Scythe (Erik Alm)

**Status:** desk research for a D-010 catalog recipe · **Gathered:** 2026-10-16 (UTC)
**Author workstream:** Content Catalog (catalog batch A)
**Scope:** facts needed to author a DDL recipe for **Scythe** (the 2003 original megawad, not
Scythe 2 or Scythe X). No code is produced here.

Labels used throughout:

- **Confirmed** — verified from an official source, upstream source code, or a downloaded archive
  (evidence recorded below).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

All downloads went to `.scratch/` in this worktree (git-ignored, never committed). Nothing was
executed from the archives; the ZIP was read by `zipfile` and the WAD only parsed as data by a
read-only lump-directory reader. The canonical page is Cloudflare-protected from this host (HTTP
403), so page metadata was taken from the Doomworld search result and the `/idgames` directory
listing at the archive master.

```sh
curl -sSL -A "DoomDeckLauncher catalog review" -o scythe.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/megawads/scythe.zip"
shasum -a 256 scythe.zip
```

## Quick reference

| | Scythe |
| --- | --- |
| Release | Released 2003-04-08; uploaded to `/idgames` 2003-04-10 |
| Author | Erik Alm, with MAP31 by Kim "Torn" Bach |
| Archive | `levels/doom2/megawads/scythe.zip` (`/idgames` id 11944) |
| Archive size / SHA-256 | 2,086,863 bytes / `a38246e005b4675d1f91474cd8dd93f4da016c4a22b1169dcede2608d72f6e99` |
| Main WAD | `SCYTHE.WAD` (MAP01–MAP32) |
| IWAD | `doom2.wad` |
| Format / complevel (Proposed) | Vanilla → **complevel 2** |
| Embedded patch | none (no `DEHACKED`, no `COMPLVL`, no `UMAPINFO`/`MAPINFO`) |
| Optional components | none in the official package |
| Licence | distribute with `scythe.txt` intact; no commercial use without agreement |

---

## 1. Official sources

- **Canonical `/idgames` page:** <https://www.doomworld.com/idgames/levels/doom2/megawads/scythe>.
  Title "Scythe", filename `levels/doom2/megawads/scythe.zip`, author "Erik Alm, with guest mapper
  Kim \"Torn\" Bach", date **04/10/03**, size 1.99 MB, `/idgames` id **11944**. **Confirmed**
  (Doomworld search result + `/idgames` directory listing).
- **Release text** `scythe.txt` (inside the ZIP, 5,093 bytes): *"Advanced engine needed : none, runs
  with doom2.exe v1.9 or any port"*; `Game : DOOM2`; `Map # : Map01-32`; `Dehacked/BEX Patch : No`.
  **Confirmed (quoted).**
- The readme leaves `Release date` blank; Doomwiki records the original release as **2003-04-08**
  (<https://doomwiki.org/wiki/Scythe>) and the archive master's `scythe.zip` is dated 2003-04-10.
  **Confirmed (dated observations).**
- No official release thread was located as a primary source; the bundled `scythe.txt` is the
  authors' authoritative statement for this recipe. **Confirmed (absence of a newer official
  build: `Scythe 2` and `Scythe X` are sequels with their own `/idgames` entries, not re-uploads
  of `scythe.zip`).**

## 2. Canonical archive and byte-identical mirrors

The `/idgames` path is `levels/doom2/megawads/scythe.zip`; every mirror serves that
`<dir>/<filename>` under its own base (`upstream-sources.md` §3.4). DDL uses direct mirror URLs,
never the `/idgames` API (D-015). Three allowlisted hosts were **downloaded in full**; all three
serve the exact same bytes:

| Host (allowlisted) | URL | bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/megawads/scythe.zip` | 2,086,863 | `a38246e0…f6e99` |
| `youfailit.net` | `https://youfailit.net/pub/idgames/levels/doom2/megawads/scythe.zip` | 2,086,863 | `a38246e0…f6e99` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/megawads/scythe.zip` | 2,086,863 | `a38246e0…f6e99` |
| `ftp.fu-berlin.de` | `https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/megawads/scythe.zip` | 2,086,863 | `a38246e0…f6e99` |

- **Confirmed:** all four are byte-identical (same length and same SHA-256 computed here), so they
  may share one package record (D-015). The recipe lists the archive master first, then three
  mirrors.
- The `/idgames` archive provides **no checksum** (`upstream-sources.md` §3.4); the SHA-256 above
  was computed at review time from the canonical bytes.

## 3. Archive contents (ZIP, 2,086,863 bytes)

Everything in the archive is listed with its exact size and SHA-256 computed here. Nothing else is
present; there is no separate music pack, alternate WAD or credits file.

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `SCYTHE.WAD` | 6,215,556 | `7d0ee3f9ac017255d2903504dab2d0ed32bfd39482b2cc5e102d26c4858871ff` | maps |
| `scythe.txt` | 5,093 | `65ed2c995c3504f6f0c9de3d4c34cc0b3e7ec713670927b19fca0f08c73267fa` | credits (keep) |

**Confirmed.** Flat layout, ASCII names, both entries Deflate, no symlinks, no traversal.

## 4. WAD inspection (read-only, no execution)

`SCYTHE.WAD` is a **PWAD**, 6,215,556 bytes, **511 lumps**, 32 map markers `MAP01`–`MAP32`. Each
map is the standard ten-lump vanilla group (`THINGS … BLOCKMAP`). **Confirmed.**

- **No `COMPLVL` lump**, **no `DEHACKED` lump**, **no `UMAPINFO`/`MAPINFO`** and no ZDoom-only
  lumps. The readme states `Dehacked/BEX Patch : No`. **Confirmed.**
- The complevel therefore cannot come from a lump and must be declared in the recipe. **Confirmed.**

### DSDA-Doom v0.30.1 handling (pinned)

There is nothing for DSDA-Doom to apply automatically here: no `COMPLVL` (so the CLI complevel the
recipe declares is used; omitting the flag would fall back to the configured default of 21, per
`dsda-engine.md` §1) and no embedded `DEHACKED`/`UMAPINFO`/`MAPINFO` lumps. The recipe must
**not** add a separate `-deh` entry. **Confirmed.**

## 5. Requirements and compatibility

- **Base game:** DOOM II — the readme says `Game : DOOM2`, `Map # : Map01-32`. **Confirmed.**
- **Required port class:** `Advanced engine needed : none, runs with doom2.exe v1.9 or any port`;
  `May Not Run With… : Unreal 2` (a joke). Doomwiki lists the port as **Vanilla Doom**. **Confirmed.**
- **Complevel (Proposed):** **2 (Doom II 1.9)**, `family = "vanilla"`. This is the vanilla DOOM II
  compatibility level and matches the authors' "doom2.exe v1.9" statement. **Proposed until
  Deck-tested.**
- **Warp/episode notes:** a single continuous DOOM II mapset (MAP01–MAP32, three loose "episodes"
  of ten maps plus two secret maps that are not engine episodes). No `UMAPINFO`, so no `-warp` is
  prescribed; the DOOM II menu selects the map. **Proposed.**
- **DEH/BEX:** none to load (see §4). **Confirmed.**

## 6. Licence and redistribution

The release text states (**Confirmed (quoted)**):

```text
* Copyright / Permissions *
Authors (may NOT) use the contents of this file as a base for
modification or reuse.
You MAY distribute this file, provided you include this text file, with
no modifications.  You may distribute this file in any electronic
format (BBS, Diskette, CD, etc) as long as you include this file
intact. This file may not be used for any commercial purposes without the
author's agreement.
```

**Proposed (DDL policy):** DDL never modifies or repackages the archive; it downloads the original
ZIP from its canonical `/idgames` mirror and keeps `scythe.txt` (`keep = true`) unmodified, so the
"include this file intact" clause is satisfied. Credit: id Software (base game), Fredrik Johansson
(textures), ZarcyB (titlepic), Chris "Ryback" Ratcliff (MAP30 idea).

## 7. Composition (recipe data — matches `catalog/scythe.toml`)

- One package `free` (the ZIP above), one required `main` component loading `SCYTHE.WAD` as `maps`.
  No optional components: the official package ships nothing else.
- Keep `scythe.txt` as `keep = true`.
- `base_game = doom2` (no variant restriction); `compatibility.family = "vanilla"`,
  `complevel = 2`, `complevel_status = "proposed"`; engine `=0.30.1`, `inspected`.

## 8. Open questions

- **Complevel 2.** Not declared by any lump; proposed from the authors' "doom2.exe v1.9" statement.
  Whether DSDA-Doom's vanilla DOOM II compatibility behaves as expected on every map (e.g. the
  MAP28 voodoo-doll effect the readme calls out) needs a Deck run (**Needs Deck Validation**).
- **Cover art.** Like every current recipe, this one carries no `[[images]]`; DDL's hardcoded cover
  map falls back to SIGIL's cell until a new cell and `art()` arm are added (outside this task).


## Screenshots (spec 023, D-024)

Bundled under `crates/ddl/ui/assets/screenshots/scythe/`; the `SOURCES.toml` beside the
images records each file's source URL, source page, credit, original size, retrieval date
and any permission text found. Retrieved 2026-10-17; each file is a 640x360 JPEG card crop
of the original below.

Scythe has no surviving author site; the Doom Wiki map article is the accessible source. DoomWiki blocks automation, so the image was read from its Wayback Machine snapshot.

| Asset | Source image URL | Source page | Credit | Original |
| --- | --- | --- | --- | --- |
| `01.jpg` | <https://doomwiki.org/w/images/thumb/5/5c/Scythe-map01-start.png/320px-Scythe-map01-start.png> | <https://doomwiki.org/wiki/MAP01:_Get_Going_(Scythe)> | Erik Alm | 320x240 (`Scythe-map01-start.png`) |

**Why representative:** The Doom Wiki captured the MAP01 start view itself: a first-person in-game shot of the map, not an automap or title screen.

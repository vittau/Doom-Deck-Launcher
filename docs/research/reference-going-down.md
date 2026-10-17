# Reference research — Going Down (mouldy / Cyriak Harris)

**Status:** desk research for a D-010 catalog recipe · **Gathered:** 2026-10-16 (UTC)
**Author workstream:** Content Catalog (catalog batch A)
**Scope:** facts needed to author a DDL recipe for **Going Down** (the final `/idgames` release,
the third bug-fix). No code is produced here.

Labels used throughout:

- **Confirmed** — verified from an official source, upstream source code, or a downloaded archive
  (evidence recorded below).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

All downloads went to `.scratch/` (git-ignored, never committed). Nothing was executed; the ZIP was
read by `zipfile` and `gd.wad` only parsed as data by a read-only lump-directory reader. Doomworld
returns Cloudflare HTTP 403 from this host, so page metadata is taken from the `/idgames` search
index and the archive master's listing; the bundled `gd.txt` is the author's primary statement.

```sh
curl -sSL -A "DoomDeckLauncher catalog review" -o gd.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/gd.zip"
shasum -a 256 gd.zip
```

## Quick reference

| | Going Down |
| --- | --- |
| Release | First full release 2014-09-14; `/idgames` entry 2014-11-02; current file is the **third bug-fix** (built 2015-08-21) |
| Author | Cyriak Harris (mouldy) |
| Archive | `levels/doom2/Ports/megawads/gd.zip` (`/idgames` id 17917) |
| Archive size / SHA-256 | 8,832,905 bytes / `48accac85af296534565e4c6cbc4cb7312e70756012c327b3d7c1c14c9872dc9` |
| Main WAD | `gd.wad` (MAP01–MAP32) |
| IWAD | `doom2.wad` |
| Format / complevel | Boom → **complevel 9** (author-stated) |
| Embedded patch | one `DEHACKED` lump (5,695 B); one ZDoom-style `MAPINFO` lump (inert under v0.30.1) |
| `COMPLVL` lump | none |
| Optional components | none in the official package |
| Licence | distribute with `gd.txt` intact; no base for reuse |

---

## 1. Official sources

- **Canonical `/idgames` page:** <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/gd>.
  Title "Going Down", filename `levels/doom2/Ports/megawads/gd.zip`, author "mouldy", date
  **11/02/14**, `/idgames` id **17917**. **Confirmed** (search result + archive listing).
- **Release text `gd.txt`** (inside the ZIP, 4,634 bytes): `Archive Maintainer : This is a THIRD
  bug-fix to update the existing wad file`; `Update to : levels/doom2/Ports/megawads/gd.zip`;
  *"This is a 32 map Doom 2 megawad - boom format, complevel 9."*; `Release date : 02/11/14`;
  `Author : mouldy`; `Dehacked/BEX Patch : Yes`; `May Not Run With… : non-boom ports`.
  **Confirmed (quoted).**
- **Version history (dated observations):** the 2013-10-22 and 2014-04-03 partial releases and the
  first complete 32-map release on 2014-09-14 precede the `/idgames` upload (2014-11-02). The
  current canonical ZIP is the **third bug-fix** (its `gd.txt` is dated 2015-08-21 in the ZIP
  directory). Doomwiki records the same final `/idgames` upload and a separate 2015 update.
  **Confirmed.**
- **No newer official build of `gd.zip`:** *Going Down: Turbo* (2024) is a distinct remaster with
  its own `/idgames` entry, not a re-upload of `gd.zip`. **Confirmed.**

## 2. Canonical archive and byte-identical mirrors

The `/idgames` path is `levels/doom2/Ports/megawads/gd.zip`; every mirror serves that
`<dir>/<filename>` under its own base (`upstream-sources.md` §3.4). DDL uses direct mirror URLs,
never the `/idgames` API (D-015). Three allowlisted hosts were **downloaded in full**; all serve
the exact same bytes:

| Host (allowlisted) | URL | bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/gd.zip` | 8,832,905 | `48accac8…72dc9` |
| `youfailit.net` | `https://youfailit.net/pub/idgames/levels/doom2/Ports/megawads/gd.zip` | 8,832,905 | `48accac8…72dc9` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/Ports/megawads/gd.zip` | 8,832,905 | `48accac8…72dc9` |
| `ftp.fu-berlin.de` | `https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/Ports/megawads/gd.zip` | 8,832,905 | `48accac8…72dc9` |

**Confirmed:** byte-identical, so they share one package record (D-015). `/idgames` supplies no
checksum; the SHA-256 was computed here from the canonical bytes.

## 3. Archive contents (ZIP, 8,832,905 bytes)

Everything in the archive is listed with its exact size and SHA-256 computed here. Nothing else is
present; there is no separate music pack or alternate WAD.

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `gd.wad` | 25,359,070 | `b352de5b133156a4af0a3f55e2647fa85c69795e94eb2e28adc2f1c58576b9b6` | maps |
| `gd.txt` | 4,634 | `3d3fae1b348e2e59258935b879f33ecf9f3c7b41e223b643455bf9a52fc6da11` | credits (keep) |

**Confirmed.** Flat layout, ASCII names, both entries Deflate, no symlinks, no traversal.

## 4. WAD inspection (read-only, no execution)

`gd.wad` is a **PWAD**, 25,359,070 bytes, **544 lumps**, map markers `MAP01`–`MAP32` (the map lumps
are stored in the author's build order, not sorted). **Confirmed.**

- **One embedded `DEHACKED` lump** (5,695 bytes), `Patch File for DeHackEd v3.0`, authored with
  WhackEd4 1.0.2, `Doom version = 21`, `Patch format = 6`. The `gd.txt` lists `Dehacked/BEX
  Patch : Yes`; the patch adds the megawad's custom behaviour/text. **Confirmed.**
- **One `MAPINFO` lump** (5,851 bytes) in **ZDoom syntax** (`defaultmap`, `map MAP01
  "Going Up"`, `cluster`, `music D_*`). **Confirmed.**
- **No `COMPLVL` lump** and **no `UMAPINFO` lump** are present. The complevel therefore cannot come
  from a lump and must be declared in the recipe. **Confirmed.**

### DSDA-Doom v0.30.1 handling (pinned)

DDL launches with `-noautoload` (D-011), which skips directory autoload but not embedded patches:

- The embedded `DEHACKED` lump is applied automatically because `-nodeh` is absent
  ([`prboom2/src/d_main.c:2147–2156`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2147-L2156)).
  The recipe must **not** add a separate `-deh` entry (it would double-apply).
- **ZDoom `MAPINFO` support was removed in v0.30.0** ("Removed ZDoom MAPINFO support",
  <https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.0>), so the embedded `MAPINFO` is
  **inert** under v0.30.1; the mapset plays as a conventional 32-map BOOM megawad. **Confirmed.**
- **`COMPLVL` absent**, so the complevel comes from the recipe's explicit `-complevel 9`
  (`dsda-doom`'s omitted-flag default is 21, per `dsda-engine.md` §1). **Confirmed.**

**Confirmed.** The launch expectation is `-iwad doom2.wad -file …/gd.wad -complevel 9` with no
`-deh` argument.

## 5. Requirements and compatibility

- **Base game:** DOOM II — `gd.txt` says `Game : Doom 2`, 32 maps. **Confirmed.**
- **Required port class:** Boom — `Advanced engine needed : Boom compatible`; the description adds
  *"boom format, complevel 9"*; `May Not Run With… : non-boom ports`. **Confirmed.**
- **Complevel:** **9 (Boom)** — stated verbatim by the author, `family = "boom"`. Recorded as
  `proposed` because it has not been Deck-tested (CONTRIBUTING.md step 5). **Proposed.**
- **Warp/episode notes:** single continuous DOOM II mapset (MAP01–MAP32, no `UMAPINFO`); no `-warp`
  is prescribed. **Proposed.**
- **DEH/BEX:** the `DEHACKED` patch is embedded and automatic (see §4). **Confirmed.**

## 6. Licence and redistribution

`gd.txt` states (**Confirmed (quoted)**):

```text
Authors may NOT use the contents of this file as a base for modification or
reuse.  Permissions have been obtained from original authors for any of
their resources modified or included in this file.

You MAY distribute this file, provided you include this text file, with no
modifications.  You may distribute this file in any electronic format (BBS,
Diskette, CD, etc) as long as you include this file intact.  I have
received permission from the original authors of any modified or included
content in this file to allow further distribution.
```

**Proposed (DDL policy):** DDL downloads the unmodified ZIP and keeps `gd.txt` (`keep = true`), so
the "include this file intact" clause is satisfied. Music credits (author's own compositions) and
graphics credits (Plums: city sky texture; obsidian: dehacked help) are listed in `gd.txt`.

## 7. Composition (recipe data — matches `catalog/going-down.toml`)

- One package `free` (the ZIP above), one required `main` component loading `gd.wad` as `maps`.
  No optional components: the official package ships nothing else.
- Keep `gd.txt` as `keep = true`.
- `base_game = doom2`; `compatibility.family = "boom"`, `complevel = 9`,
  `complevel_status = "proposed"`; engine `=0.30.1`, `inspected`.

## 8. Open questions

- **Lump-declared complevel.** None exists; 9 is taken from the author's own statement and needs a
  Deck run to confirm behaviour (**Needs Deck Validation**).
- **`MAPINFO` inertness.** The embedded ZDoom `MAPINFO` sets music and cluster names. Under
  DSDA-Doom v0.30.1 it is ignored by design; whether the in-game music selection matches the
  author's intent without it must be confirmed on the Deck (**Needs Deck Validation**).
- **Going Down: Turbo (2024).** A separate official remaster with its own `/idgames` entry; it
  would be its own recipe, out of scope here.


## Screenshots (spec 023, D-024)

Bundled under `crates/ddl/ui/assets/screenshots/going-down/`; the `SOURCES.toml` beside the
images records each file's source URL, source page, credit, original size, retrieval date
and any permission text found. Retrieved 2026-10-17; each file is a 640x360 JPEG card crop
of the original below.

The screenshots are the author's own in the mapset's release thread (`doomworld.com/vb/thread/65955`); Doomworld blocks automation, so the thread was read from an earlier Wayback Machine snapshot and the images fetched from their original `mediafire.com` URLs.

| Asset | Source image URL | Source page | Credit | Original |
| --- | --- | --- | --- | --- |
| `01.jpg` | <http://www.mediafire.com/convkey/24f9/new6i797i9dcwcs4g.jpg> | <https://www.doomworld.com/vb/thread/65955> | Cyriak Harris (mouldy) | 515x269 (`new6i797i9dcwcs4g.jpg`) |

**Why representative:** They show recognisable Going Down interiors (the office/library start of the descent).

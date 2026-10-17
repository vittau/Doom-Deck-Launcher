# Reference research — Ancient Aliens (skillsaw et al)

**Status:** desk research for a D-010 catalog recipe · **Gathered:** 2026-10-16 (UTC)
**Author workstream:** Content Catalog (catalog addition)
**Scope:** facts needed to author a DDL recipe for **Ancient Aliens** (the v1.2 update). No code is
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
available. Facts therefore come from the files the author shipped with the release (the
accompanying TXT, read without executing anything) and from the direct `/idgames` mirror path.

Archives were downloaded to a scratch directory (never committed), hashed with `shasum -a 256`,
inspected with Python `zipfile` and a read-only WAD lump reader; nothing was executed.

```sh
curl -sSL -A "DoomDeckLauncher catalog review" \
  -o aaliens.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/aaliens.zip"
shasum -a 256 aaliens.zip
```

## Quick reference

| | Ancient Aliens |
| --- | --- |
| Release | **v1.2, 2023-03-16** (v1.1 2022-12-30; v1 2016-05-08) |
| Author | Paul "skillsaw" DeBruyne (guest mappers: AD_79, esselfortium, Joshy, lupinx-Kassman, Pinchy, stewboy, Tarnsman) |
| Archive | `levels/doom2/Ports/megawads/aaliens.zip` |
| Archive size / SHA-256 | 39,885,868 bytes / `1c9d1e256a44bab531c8017e5a590cdcc51ac4411f154446c694589054ec2884` |
| Main WAD | `aaliens_v1_2.wad` (MAP01–MAP32) |
| IWAD | `doom2.wad` |
| Format / complevel (Proposed) | Boom → **complevel 9** |
| Embedded patch | one `DEHACKED` lump in the WAD (no separate DEH/BEX) |
| `COMPLVL` lump | none |
| Optional components | none in the official package |
| Licence | "You MAY distribute this file… with no modifications"; no reuse as a base |

---

## 1. Official sources

- **Canonical `/idgames` page:**
  <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/aaliens> — filename
  `levels/doom2/Ports/megawads/aaliens.zip`; the archive listing dates the file **2023-03-18**.
  **Confirmed (URL is canonical; the page itself was 403 from this host — see Method).**
- **Archive directory listing (fetched 2026-10-16):**
  <https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/> — `aaliens.txt` and
  `aaliens.zip`, both dated `2023-03-18 11:01` (38 MB). **Confirmed**
- **Release text (`aaliens.txt`, inside the archive):** title "Ancient Aliens", filename
  `aaliens.zip`, author "Paul \"skillsaw\" DeBruyne", release dates v1 2016-05-08, v1.1
  2022-12-30, **v1.2 2023-03-16**, `Advanced engine needed : Boom compatible`, `Game : Doom 2`,
  `Map # : MAP01-MAP32`, `Dehacked/BEX Patch : Yes`. Version 1.1 updated textures; version 1.2
  fixed a MAP24 crash in Eternity caused by rebuilt nodes. **Confirmed (quoted).**
- **Port differences (release text):** *"ZDoom-based ports and Eternity use MAPINFO for episode
  selection and story screens"* and *"in Zdoom-based ports and Eternity, ACS is used to play sound
  effects occasionally"*. **Confirmed (quoted).**

## 2. Canonical archive and byte-identical mirrors

The `/idgames` path is `levels/doom2/Ports/megawads/aaliens.zip`; every mirror serves that
`<dir>/<filename>` under its own base (`upstream-sources.md` §3.4). DDL uses direct mirror URLs,
never the `/idgames` API (D-015). Three allowlisted hosts were checked and **downloaded in full**;
all three serve the exact same bytes:

| Host (allowlisted) | URL | bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/aaliens.zip` | 39,885,868 | `1c9d1e25…c2884` |
| `youfailit.net` | `https://youfailit.net/pub/idgames/levels/doom2/Ports/megawads/aaliens.zip` | 39,885,868 | `1c9d1e25…c2884` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/Ports/megawads/aaliens.zip` | 39,885,868 | `1c9d1e25…c2884` |

**Confirmed:** all three are byte-identical (same length and same SHA-256 computed here), so they
may share one package record (D-015). The recipe lists the master plus two mirrors.

## 3. Archive contents (ZIP, 39,885,868 bytes)

Everything is listed as a member with its exact size and SHA-256 computed here. Nothing else is in
the archive; there is no separate MIDI pack or alternate WAD.

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `aaliens_v1_2.wad` | 133,408,915 | `1a30ce65fa511510bbbc35d1d432309e3621ad447cd8652130bace8fa1c03adc` | maps |
| `aaliens.txt` | 15,881 | `13ef4e78b1caf56191247868b7e089a1df63e1d44e70fd715cc282959f845aff` | credits (keep) |

**Confirmed.** The archive has no top-level directory; the WAD name carries the version, so the
recipe pins v1.2 explicitly.

## 4. WAD inspection (read-only, no execution)

`aaliens_v1_2.wad` is a **PWAD**, 133,408,915 bytes, **8,726 lumps**, with 32 `MAPxx` markers and
`DEMO1`–`DEMO3`. **Confirmed.**

- **No `COMPLVL` lump** and **no `UMAPINFO` lump** are present. A ZDoom-style `MAPINFO` (5,770 B)
  exists (episode/story screens, as the TXT says), but DSDA-Doom **removed ZDoom `MAPINFO` support
  in v0.30.0**, so it is inert here and the complevel cannot come from a lump; it must be declared
  in the recipe. **Confirmed**
- **One embedded `DEHACKED` lump** (7,787 bytes) beginning
  `Patch File for DeHackEd v3.0 … Doom version = 21 / Patch format = 6` (WhackEd4-authored).
  **Confirmed**

## 5. Requirements and compatibility

- **Base game:** DOOM II — the release TXT says `Game : Doom 2`, `Map # : MAP01-MAP32`. **Confirmed**
- **Required port class:** `Advanced engine needed : Boom compatible`. **Confirmed**
- **Complevel (Proposed):** **9 (Boom)**. The release is described as Boom-compatible and ships no
  `COMPLVL` lump; Boom compatibility is the standard target. **Proposed until Deck-tested** — this
  is the value the recipe uses.
- **Warp/episode notes:** a single continuous DOOM II mapset (MAP01–MAP32) with no `UMAPINFO`, so
  no `-warp` is prescribed; the DOOM II menu selects the map. The episode/story screens are
  ZDoom/Eternity-only and inert under DSDA-Doom. **Proposed.**
- **DEH/BEX:** none to load separately — the patch is an embedded `DEHACKED` lump (see §4).
  **Confirmed**

### What DSDA-Doom v0.30.1 does with the embedded DEHACKED under DDL's `-noautoload`

DDL launches with `-noautoload` (D-011) to skip **directory** autoload
([`prboom2/src/dsda/args.c:491–505`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L491-L505),
[`prboom2/src/d_main.c:2059–2070`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2059-L2070)).
`-noautoload` does **not** disable DEHACKED-in-a-WAD: `d_main.c` processes `DEHACKED` lumps from
explicitly loaded PWADs whenever `-nodeh` is absent, independent of the autoload flag
([`prboom2/src/d_main.c:2147–2156`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2147-L2156)).
So a launch `… -iwad doom2.wad -file …/aaliens_v1_2.wad -complevel 9 -noautoload` applies the
embedded patch automatically, and the recipe must **not** add a separate `-deh` entry.
**Confirmed** (upstream source at the pinned tag; `dsda-engine.md` §1 states the same rule).

## 6. Licence and redistribution

The release TXT states (**Confirmed (quoted)**):

```text
* Copyright / Permissions *
Authors may NOT use the contents of this file as a base for modification or
reuse.  Permissions have been obtained from original authors for any of
their resources modified or included in this file. Use aa-tex as a resource
instead.

You MAY distribute this file, provided you include this text file, with no
modifications.  You may distribute this file in any electronic format (BBS,
Diskette, CD, etc) as long as you include this file intact.  I have
received permission from the original authors of any modified or included
content in this file to allow further distribution.
```

The TXT also carries a GothicDM/Gothic2 texture disclaimer (permission to include those select
textures/flats with credit). **Proposed (DDL policy):** DDL never modifies or repackages the
archive; it downloads the original ZIP from its canonical `/idgames` mirror and keeps the bundled
text file (`keep = true`) unmodified.

## 7. Composition (recipe data — matches `catalog/ancient-aliens.toml`)

- One package `free` (the ZIP above), one required `main` component loading `aaliens_v1_2.wad` as
  `maps`. No optional components: the official package ships nothing else (the bandcamp OST is not
  a WAD and is not bundled).
- Keep `aaliens.txt` as `keep = true`.
- `base_game = doom2` (no variant restriction); `compatibility.family = "boom"`, `complevel = 9`,
  `complevel_status = "proposed"`; engine `=0.30.1`, `inspected`.

## 8. Open questions

- **Complevel 9 vs an MBF-family engine.** The TXT only says "Boom compatible" and ships no
  `COMPLVL`; the embedded `DEHACKED` targets "Doom version = 21" (a WhackEd authoring setting, not
  an engine complevel). Whether 9 is the best *play* default on DSDA-Doom needs a Deck run
  (**Needs Deck Validation**).
- **Which version idgames serves.** The `aaliens.txt`/`aaliens.zip` on the archive are the v1.2
  update (dated 2023-03-18); older mirrors or repacks may ship v1/v1.1, which must not be mixed
  with these hashes.
- **Canonical page unread.** The Doomworld entry was Cloudflare-blocked from this host; a
  contributor with browser access could confirm there is no newer official build.


## Screenshots (spec 023, D-024)

Bundled under `crates/ddl/ui/assets/screenshots/ancient-aliens/`; the `SOURCES.toml` beside the
images records each file's source URL, source page, credit, original size, retrieval date
and any permission text found. Retrieved 2026-10-17; each file is a 640x360 JPEG card crop
of the original below.

The images are the author's own screenshots in the mapset's release thread (`doomworld.com/vb/thread/87784`); Doomworld blocks automated clients, so the thread was read from its Wayback Machine snapshot and the images fetched from their original `i.imgur.com` URLs.

| Asset | Source image URL | Source page | Credit | Original |
| --- | --- | --- | --- | --- |
| `01.jpg` | <https://i.imgur.com/5WCF715.png> | <https://www.doomworld.com/vb/thread/87784> | Paul "skillsaw" DeBruyne | 1920x1080 (`5WCF715.png`) |

**Why representative:** The shots are first-person views of the mapset's signature sunset-coloured temple courtyards, the look Ancient Aliens is known for.

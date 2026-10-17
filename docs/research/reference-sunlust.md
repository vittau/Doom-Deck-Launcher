# Reference research — Sunlust (Ribbiks & Dannebubinga)

**Status:** desk research for a D-010 catalog recipe · **Gathered:** 2026-10-16 (UTC)
**Author workstream:** Content Catalog (catalog addition)
**Scope:** facts needed to author a DDL recipe for **Sunlust**. No code is produced here.

Labels used throughout:

- **Confirmed** — verified from an official source, upstream source code, or a downloaded archive
  (evidence recorded below).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

The canonical Doomworld page and the authors' release thread were **not readable** from this host:
`https://www.doomworld.com/idgames/...` returns Cloudflare HTTP 403 ("Just a moment…") to
non-browser clients, the same limitation already recorded in `upstream-sources.md` §4.2, and the
Internet Archive was temporarily offline during this session, so no Wayback snapshot was
available. Facts therefore come from the files the authors themselves shipped with the release
(the accompanying TXT, read without executing anything) and from the direct `/idgames` mirror
path.

Archives were downloaded to a scratch directory (never committed), hashed with `shasum -a 256`,
inspected with Python `zipfile` and a read-only WAD lump reader; nothing was executed.

```sh
curl -sSL -A "DoomDeckLauncher catalog review" \
  -o sunlust.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/sunlust.zip"
shasum -a 256 sunlust.zip
```

## Quick reference

| | Sunlust |
| --- | --- |
| Release | **2015-08-07** (idgames file date 2015-08-09) |
| Author | Ribbiks & Dannebubinga (franckFRAG: `M_DOOM` graphic) |
| Archive | `levels/doom2/Ports/megawads/sunlust.zip` |
| Archive size / SHA-256 | 24,388,254 bytes / `46bc1fe72abec5ad06739b91657f3ed2fed1fda967cd38e6faa4a1701999d64a` |
| Main WAD | `sunlust/sunlust.wad` (MAP01–MAP32) |
| IWAD | `doom2.wad` |
| Format / complevel (Proposed) | Boom → **complevel 9** |
| Embedded patch | one `DEHACKED` lump in the WAD (no separate DEH/BEX) |
| `COMPLVL` lump | none |
| Optional components | none in the official package |
| Licence | "You MAY distribute this file… with no modifications"; no reuse as a base |

---

## 1. Official sources

- **Canonical `/idgames` page:**
  <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/sunlust> — filename
  `levels/doom2/Ports/megawads/sunlust.zip`; the archive listing dates the file **2015-08-09**.
  **Confirmed (URL is canonical; the page itself was 403 from this host — see Method).**
- **Archive directory listing (fetched 2026-10-16):**
  <https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/> — `sunlust.txt` and
  `sunlust.zip`, both dated `2015-08-09 03:31`. **Confirmed**
- **Release text (`sunlust.txt`, inside the archive):** title "Sunlust", filename
  `sunlust.wad`, release date "Aug 7, 2015", author "Ribbiks & Dannebubinga", `Game : Doom 2`,
  `Map # : 01-32`, `Advanced engine needed : Boom-compatible`, `Dehacked/BEX Patch : Yes`.
  **Confirmed (quoted).**
- **`sunlust_extra_info.txt` (inside the archive):** per-map commentary and a large additional
  credits list; it repeats *"designed to be played from pistol start"*. **Confirmed (quoted).**
- No author release thread could be read (see Method), so no forum claim is used.

## 2. Canonical archive and byte-identical mirrors

The `/idgames` path is `levels/doom2/Ports/megawads/sunlust.zip`; every mirror serves that
`<dir>/<filename>` under its own base (`upstream-sources.md` §3.4). DDL uses direct mirror URLs,
never the `/idgames` API (D-015). Three allowlisted hosts were checked and **downloaded in full**;
all three serve the exact same bytes:

| Host (allowlisted) | URL | bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/sunlust.zip` | 24,388,254 | `46bc1fe7…9d64a` |
| `youfailit.net` | `https://youfailit.net/pub/idgames/levels/doom2/Ports/megawads/sunlust.zip` | 24,388,254 | `46bc1fe7…9d64a` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/Ports/megawads/sunlust.zip` | 24,388,254 | `46bc1fe7…9d64a` |
| `planetzdoom.com` (extra check) | `https://planetzdoom.com/idgames/levels/doom2/Ports/megawads/sunlust.zip` | 24,388,254 | `46bc1fe7…9d64a` |

**Confirmed:** all four are byte-identical (same length and same SHA-256 computed here), so they
may share one package record (D-015). The recipe lists the master plus two mirrors.

## 3. Archive contents (ZIP, 24,388,254 bytes)

Everything is listed as a member with its exact size and SHA-256 computed here. Nothing else is in
the archive; there is no separate MIDI pack or alternate WAD.

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `sunlust/sunlust.wad` | 65,966,252 | `6a90becf56040896fd2876a29f60a7a528d9802b081e0a1cc7d7cc65d546a964` | maps |
| `sunlust/sunlust.txt` | 2,952 | `e5c3e93ee5b188a4263f6f3f04649c5713141c464f41fe78b1611088982b9b53` | credits (keep) |
| `sunlust/sunlust_extra_info.txt` | 23,602 | `8cb41747aea2f95f735742a0f7504947bcd931eff945248f4f6f5eb2799c3bdb` | extra info / credits (keep) |

**Confirmed.** The archive has a top-level `sunlust/` directory, so member paths carry that prefix.

## 4. WAD inspection (read-only, no execution)

`sunlust/sunlust.wad` is a **PWAD**, 65,966,252 bytes, **2,788 lumps**, with the map markers
`MAP01`…`MAP32` and `DEMO1`–`DEMO3`. **Confirmed.**

- **No `COMPLVL` lump** and **no `UMAPINFO` lump** are present. A ZDoom-style `MAPINFO` (4,464 B)
  exists, but DSDA-Doom **removed ZDoom `MAPINFO` support in v0.30.0**, so it is inert here and the
  complevel cannot come from a lump; it must be declared in the recipe. **Confirmed**
  (`host-name` [v0.30.0 release notes] and the same rule recorded in `reference-eviternity.md` §4).
- **One embedded `DEHACKED` lump** (2,687 bytes) beginning
  `Patch File for DeHackEd v3.0 … Doom version = 21 / Patch format = 6` (WhackEd4-authored).
  **Confirmed**

## 5. Requirements and compatibility

- **Base game:** DOOM II — the release TXT says `Game : Doom 2`, `Map # : 01-32`. **Confirmed**
- **Required port class:** `Advanced engine needed : Boom-compatible`. **Confirmed**
- **Complevel (Proposed):** **9 (Boom)**. The release is described as Boom-compatible and ships no
  `COMPLVL` lump; Boom compatibility is the standard target. **Proposed until Deck-tested** — this
  is the value the recipe uses.
- **Pistol start:** the TXT states the set is *"designed to be played from pistol start"*.
  **Confirmed** → the recipe records `pistol_start = true` as a **profile suggestion only**
  (the schema's `compatibility.runtime.pistol_start`). It is not a launch flag the recipe forces.
- **Warp/episode notes:** a single continuous DOOM II mapset (MAP01–MAP32) with no `UMAPINFO`, so
  no `-warp` is prescribed; the DOOM II menu selects the map. **Proposed.**
- **DEH/BEX:** none to load separately — the patch is an embedded `DEHACKED` lump
  (see §4). **Confirmed**

### What DSDA-Doom v0.30.1 does with the embedded DEHACKED under DDL's `-noautoload`

DDL launches with `-noautoload` (D-011) to skip **directory** autoload
([`prboom2/src/dsda/args.c:491–505`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L491-L505),
[`prboom2/src/d_main.c:2059–2070`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2059-L2070)).
`-noautoload` does **not** disable DEHACKED-in-a-WAD: `d_main.c` processes `DEHACKED` lumps from
explicitly loaded PWADs whenever `-nodeh` is absent, independent of the autoload flag
([`prboom2/src/d_main.c:2147–2156`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2147-L2156)).
So a launch `… -iwad doom2.wad -file …/sunlust.wad -complevel 9 -noautoload` applies Sunlust's
embedded patch automatically, and the recipe must **not** add a separate `-deh` entry.
**Confirmed** (upstream source at the pinned tag; `dsda-engine.md` §1 states the same rule).

## 6. Licence and redistribution

The release TXT states (**Confirmed (quoted)**):

```text
* Copyright / Permissions *
Authors may NOT use the contents of this file as a base for modification or
reuse.  Permissions have been obtained from original authors for any of their
resources modified or included in this file.

You MAY distribute this file, provided you include this text file, with no
modifications.  You may distribute this file in any electronic format (BBS,
Diskette, CD, etc) as long as you include this file intact.  I have received
permission from the original authors of any modified or included content in
this file to allow further distribution.
```

**Proposed (DDL policy):** DDL never modifies or repackages the archive; it downloads the original
ZIP from its canonical `/idgames` mirror and keeps both bundled text files (`keep = true`)
unmodified. Redistribution is permitted under the terms above; reuse as a base is not.

## 7. Composition (recipe data — matches `catalog/sunlust.toml`)

- One package `free` (the ZIP above), one required `main` component loading
  `sunlust/sunlust.wad` as `maps`. No optional components: the official package ships nothing else.
- Keep `sunlust/sunlust.txt` and `sunlust/sunlust_extra_info.txt` as `keep = true`.
- `base_game = doom2` (no variant restriction); `compatibility.family = "boom"`,
  `complevel = 9`, `complevel_status = "proposed"`; engine `=0.30.1`, `inspected`;
  `runtime.pistol_start = true` (suggestion only, from the authors' text).

## 8. Open questions

- **Complevel 9 vs MBF.** The TXT only says "Boom-compatible" and ships no `COMPLVL`; some
  Sunlust demos were recorded at Boom (9). Whether 9 is the best *play* default on DSDA-Doom and
  how the embedded DEHACKED behaves needs a Deck run (**Needs Deck Validation**).
- **Canonical page/thread unread.** The Doomworld entry and forum thread were Cloudflare-blocked
  from this host; a contributor with browser access could add the thread's complevel guidance if
  any exists.
- **`M_DOOM`/`extra_info` credit chain.** `sunlust_extra_info.txt` references "gothictx or
  cc4-tex" and other resource credits; the recipe keeps the file so those credits ship intact.

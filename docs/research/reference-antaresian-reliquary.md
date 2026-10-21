# Reference research — Antaresian Reliquary (Seongbae Park / antares031)

**Status:** desk research for a D-010 catalog recipe · **Gathered:** 2026-10-21 (UTC)
**Author workstream:** Content Catalog (catalog addition)
**Scope:** facts needed to author a DDL recipe for **Antaresian Reliquary**. No code is produced
here.

Labels used throughout:

- **Confirmed** — verified from an official source, upstream source code, or a downloaded archive
  (evidence recorded below).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

The canonical Doomworld page and the DoomWiki page returned Cloudflare HTTP 403 ("Just a
moment…") to this host, and the release thread was also 403 directly. The Internet Archive
Wayback Machine served the DoomWiki article and the release thread, and those snapshots are cited
below. Facts otherwise come from the files the author shipped with the release (read without
executing anything) and from direct `/idgames` mirrors.

Archives were downloaded to a scratch directory (never committed), hashed with `shasum -a 256`,
listed with `unzip -l`/`unzip -v`, and inspected with a read-only Python WAD lump reader; nothing
was executed. The two screenshots were downloaded from the URLs recorded in their `SOURCES.toml`
and converted with `scripts/lean-image.sh`.

```sh
curl -sSL --max-time 120 -A "DoomDeckLauncher catalog review" \
  -o anta_req.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/anta_req.zip"
shasum -a 256 anta_req.zip
unzip -l anta_req.zip
```

## Quick reference

| | Antaresian Reliquary |
| --- | --- |
| Release | **2020-08-24** (author TXT and archive timestamps) |
| Author | Seongbae Park (antares031) |
| Archive | `levels/doom2/Ports/a-c/anta_req.zip` |
| Archive size / SHA-256 | 12,793,564 bytes / `f81e4a3e818b308438d322a1a716c53c43327ffa8e1ed04ab880a5221ea37f2e` |
| Main WAD | `ANTA_REQ.wad` (46,829,703 bytes) |
| IWAD | `doom2.wad` |
| Format / complevel | Boom → **complevel 9** (`boom`) |
| Embedded patch | one `DEHACKED` lump (24,232 bytes); no separate DEH/BEX file |
| `COMPLVL` / `UMAPINFO` lumps | none |
| Map slots | MAP01–MAP07, MAP31, MAP32 (nine slots) |
| Optional components | none in the official package |
| Licence | CC BY-NC 4.0; bundled credits text kept |

---

## 1. Official sources

- **Canonical `/idgames` page:**
  <https://www.doomworld.com/idgames/levels/doom2/Ports/a-c/anta_req> — filename
  `levels/doom2/Ports/a-c/anta_req.zip`; the DoomWiki article links the legacy id `?id=19641`.
  **Confirmed (URL is canonical; the page itself returned 403 from this host — see Method).**
- **Release text (`ANTA_REQ.txt`, inside the archive; 10,216 bytes):** title "Antaresian
  Reliquary", filename `ANTA_REQ`, release date `2020/08/24`, author `antares031`, `Advanced
  engine needed : Boom`, `Game : Doom 2`, `Map # : 7`, `Dehacked/BEX Patch : Yes`, `Tested With :
  PrBoom+ 2.5.1.5, GZDoom 4.4.2, Eternity 4.00, ZDaemon 1.10.12`. **Confirmed (quoted).**
- **Release thread (archived):**
  <https://web.archive.org/web/20250521075320/https://www.doomworld.com/vb/thread/114478> — the
  author's first post calls the WAD "boom-compatible", says seven levels were selected from
  community projects, and notes each level has a death exit for forced pistol starts. The post
  also records the final idgames update dated August 24, 2020 (DECORATE-only fixes). **Confirmed
  (archived snapshot; direct Doomworld access was 403).**
- **DoomWiki (archived):**
  <https://web.archive.org/web/20251119110641/https://doomwiki.org/wiki/Antaresian_Reliquary> —
  calls it a nine-level Boom-compatible Doom II PWAD, lists MAP01–MAP07, MAP31 and MAP32, and says
  two new levels were added to prevent entry to regular IWAD levels. **Confirmed as secondary
  evidence; direct DoomWiki access was 403.**
- **Cacowards 2020, Top Ten page 1:**
  <https://www.doomworld.com/cacowards/2020/best1/> (archived copy used:
  <https://web.archive.org/web/20201210111058id_/https://www.doomworld.com/cacowards/2020/best1/>)
  — lists "Antaresian Reliquary - @antares031 / Doom 2, Boom-compatible, 7 maps". **Confirmed
  (archived page read).** The 7-map label matches the author TXT; the WAD's nine map slots are
  described below.

## 2. Canonical archive and byte-identical mirrors

The `/idgames` path is `levels/doom2/Ports/a-c/anta_req.zip`; mirrors serve that
`<dir>/<filename>` under their own base (`upstream-sources.md` §3.4). DDL uses direct mirror URLs,
never the `/idgames` API (D-015). Four allowlisted hosts were downloaded in full and checked:

| Host (allowlisted) | URL | bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/anta_req.zip` | 12,793,564 | `f81e4a3e…a37f2e` |
| `dsdarchive.com` | `https://dsdarchive.com/files/wads/doom2/4089/anta_req.zip` | 12,793,564 | `f81e4a3e…a37f2e` |
| `youfailit.net` | `https://youfailit.net/pub/idgames/levels/doom2/Ports/a-c/anta_req.zip` | 12,793,564 | `f81e4a3e…a37f2e` |
| `mirrors.lug.mtu.edu` (extra check) | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/Ports/a-c/anta_req.zip` | 12,793,564 | `f81e4a3e…a37f2e` |
| `ftp.fu-berlin.de` (extra check) | `https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/Ports/a-c/anta_req.zip` | 12,793,564 | `f81e4a3e…a37f2e` |

**Confirmed:** all five downloads are byte-identical (same length and SHA-256 computed here), so
they may share one package record (D-015). The recipe lists the master, dsdarchive and
youfailit.

## 3. Archive contents (ZIP, 12,793,564 bytes)

The archive has no top-level directory and exactly two members. Sizes and SHA-256 were computed
from the downloaded bytes:

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `ANTA_REQ.wad` | 46,829,703 | `a34447b2aeb021f09fdad4d153fe8177833b0dbf41fb74e6b8266f02959f3ab9` | maps and resources |
| `ANTA_REQ.txt` | 10,216 | `adfa6a1e6dfae70a499eef4342e6c554d3cd87fd079c1ab38303580153dec754` | credits/permission (keep) |

**Confirmed.** There is no separate `.deh`/`.bex` member; the archive contains only the WAD and
its TXT. The TXT is UTF-8 with CRLF line endings and was read without modifying it.

## 4. WAD inspection (read-only, no execution)

- `ANTA_REQ.wad` is a **PWAD**, 46,829,703 bytes, **1,499 lumps**, **nine map markers**:
  MAP01–MAP07, MAP31, MAP32. **Confirmed.**
- **Embedded `DEHACKED` lump, 24,232 bytes**, `Doom version = 21`, `Patch format = 6`, created
  with WhackEd4. It changes the player/monster/weapon behaviour and adds the custom Antaresia,
  Former Ranger and Chaingun Elite actors. DSDA-Doom loads a WAD-embedded DEHACKED patch
  automatically; `-noautoload` does not disable it, and no `-deh` entry is needed. **Confirmed.**
- **No `COMPLVL` lump and no `UMAPINFO` lump.** `ZMAPINFO` (2,440 bytes), `DECORATE` (5,289
  bytes) and `LANGUAGE` (214 bytes) are ZDoom-only. DSDA-Doom **removed ZDoom MAPINFO support in
  v0.30.0** (`https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.0`, "Removed ZDoom MAPINFO
  support"), so the author's ZMAPINFO map names, next-map links and MAP31 ending text are inert
  under v0.30.1. The recipe therefore declares the complevel explicitly. **Confirmed.**
- **Complevel:** the author's TXT says `Advanced engine needed : Boom`, the release post says
  "boom-compatible", and the author tested PrBoom+ 2.5.1.5. D-030 uses the DSDA alias `boom`
  (complevel 9). **Confirmed from author text; no gameplay test performed.**
- **Map names in the bundled ZMAPINFO (inert in DSDA):** MAP01 Moonstriker's Outpost, MAP02 A
  Forlorn Outpost, MAP03 Harvester's Reminiscence, MAP04 Purgatorial Technologies, MAP05 Alpha
  Scorpii Supercluster, MAP06 The Kerberos Complex, MAP07 Druidia Borealis, MAP31 The Eclipse of
  Betelgeuse, MAP32 Cheonsanggye. **Confirmed from the lump.**
- The TXT lists six compilation maps plus MAP31 (MAP01–MAP06, MAP31) and calls the release "7
  maps"; the WAD additionally contains MAP07 and MAP32. DoomWiki describes nine levels and says
  two new levels were added to prevent entry to regular IWAD levels. The recipe describes nine
  map slots and does not invent a different map count. **Confirmed archive shape; the "two added
  levels" framing is DoomWiki's secondary description.**
- **No `COMPLVL`/`UMAPINFO` means DSDA falls back to vanilla Doom II progression.** MAP31/MAP32
  are secret/bonus slots and are not reached by the normal MAP01–MAP07 sequence; they need
  `-warp`/IDCLEV until DDL exposes level selection. **Needs Deck Validation** for the exact
  end-of-MAP07/end-of-MAP32 behavior and for whether a clean ending appears without the author's
  ZMAPINFO.

## 5. Launch contract and components

- IWAD: **Doom II** (`doom2.wad`), no variants.
- Load exactly one file: `ANTA_REQ.wad` via `-file`; no `-deh` list (the WAD's `DEHACKED` lump is
  automatic).
- Explicit `-complevel boom` (D-030 alias); no COMPLVL discovery.
- The author recommends forced pistol starts (death exits), so the recipe's runtime default is
  `pistol_start = true`; it is a profile suggestion only.
- No optional components are published in the official package. The TXT's credits list third-party
  sprite/texture/music/sound sources but ships no separate add-on WAD.
- Keep the bundled TXT in the install (`keep = true`) to preserve the CC BY-NC 4.0 text and full
  credits.

## 6. Screenshots and provenance

Two representative in-game views were posted by the author in the release thread and bundled under
`crates/ddl/ui/assets/screenshots/antaresian-reliquary/`:

| File | Source | Original | Committed |
| --- | --- | --- | --- |
| `01.jpg` | <https://i.imgur.com/Qbj6Qyd.png> (MAP05 view) | 800×600 PNG | 116,603 bytes JPEG |
| `02.jpg` | <https://i.imgur.com/hkRdHBV.png> (hellish interior) | 800×600 PNG | 79,627 bytes JPEG |

Source page for both: <https://www.doomworld.com/vb/thread/114478> (archived snapshot above).
Credit: `antares031`. The originals were downloaded on 2026-10-21 and converted with
`scripts/lean-image.sh` (JPEG quality 60, no enlargement); both are 800×600 and below the 192 KiB
limit. No separate image licence text was found; they are bundled under D-024 and removed on the
author's request. **Confirmed image files; no gameplay/Deck test is claimed from them.**

```sh
scripts/lean-image.sh Qbj6Qyd.png \
  crates/ddl/ui/assets/screenshots/antaresian-reliquary/01.jpg
scripts/lean-image.sh hkRdHBV.png \
  crates/ddl/ui/assets/screenshots/antaresian-reliquary/02.jpg
```

## 7. Tags, verification and owner request

- `cacowards`: 2020 Top Ten page 1, archived page read (link above). **Confirmed.**
- `episode`: nine map slots, matching the tag's nine-or-fewer rule and DoomWiki's "nine-level"
  description. **Confirmed for map count; the release is a compilation rather than a single
  authored episode.**
- `classics`: released 2020-08-24, more than five years before this 2026 review, with the
  Cacowards citation in the recipe notes. **Confirmed dates/citation.**
- **Owner-requested MAP31 warning (2026-10-21):** the player-facing description states that MAP31
  contains a massive number of enemies and performs poorly on Steam Deck. This wording and
  provenance come from the owner's request and knowledge (spec 066 R5), not from a DDL
  measurement; the recipe does not invent FPS, enemy counts or a benchmark. Reference:
  <https://doomwiki.org/wiki/MAP31:_The_Eclipse_of_Betelgeuse_(Antaresian_Reliquary)>.
- Verification status is **inspected** (archive, members, WAD lumps and author TXT read; nothing
  executed). No gameplay or Deck claim is made.

## 8. Limitations / open questions

- **Needs Deck Validation:** IWAD/complevel, load order, gameplay, audio and the MAP31 warning's
  practical effect. The warning itself is an owner decision, not a DDL benchmark.
- **Needs Deck Validation:** MAP31/MAP32 reachability and the MAP07/MAP32 endings under DSDA-Doom
  without the author's ZMAPINFO. The bundled ZMAPINFO is inert under v0.30.1, so the author's
  custom map names/next-map links and MAP31 ending text do not appear.
- DoomWiki and Doomworld blocked direct automation; their facts here are labelled as Wayback
  snapshots or secondary evidence, while the recipe's hashes and member facts come from the
  downloaded bytes.
- No new hosts or dependencies were introduced.

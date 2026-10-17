# Reference research — Alien Vendetta (Anders Johnsen et al)

**Status:** desk research for a D-010 catalog recipe · **Gathered:** 2026-10-16 (UTC)
**Author workstream:** Content Catalog (catalog batch A)
**Scope:** facts needed to author a DDL recipe for **Alien Vendetta** (the final `/idgames`
release, the 2002 Compet-N edition). No code is produced here.

Labels used throughout:

- **Confirmed** — verified from an official source, upstream source code, or a downloaded archive
  (evidence recorded below).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

All downloads went to `.scratch/` (git-ignored, never committed). Nothing was executed from the
archive; the ZIP was read by `zipfile`, and `AV.WAD`/`AVMOVFIX.WAD` and `AV.DEH` were only parsed as
data by a read-only lump/lump-header reader. Doomworld returns Cloudflare HTTP 403 to non-browser
clients from this host, so its page metadata is taken from the `/idgames` directory listing and the
search index; the bundled `AV.TXT` is the authors' primary statement.

```sh
curl -sSL -A "DoomDeckLauncher catalog review" -o av.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/megawads/av.zip"
shasum -a 256 av.zip
```

## Quick reference

| | Alien Vendetta |
| --- | --- |
| Release | First release 2001-12-24; final **Compet-N edition** dated 24.12.2002, `/idgames` 2002-12-26 |
| Authors | Anders Johnsen (project lead) and 12 further level authors; see §1 |
| Archive | `levels/doom2/megawads/av.zip` |
| Archive size / SHA-256 | 9,041,333 bytes / `5c5382cc50b565946b6725b3058cc1fdefe7367ee3067808068900117aabf8fc` |
| Main WAD | `AV.WAD` (MAP01–MAP32) |
| IWAD | `doom2.wad` |
| Format / complevel (Proposed) | Vanilla → **complevel 2** |
| Patch | external `AV.DEH` (automap names + story/intermission text); **no** embedded `DEHACKED` |
| Optional component | `AVMOVFIX.WAD` (Compet-N demo variant of MAP20), bundled |
| Licence | distribute `AV.WAD`/`AV.TXT` intact, no base for new levels, no commercial use |

---

## 1. Official sources

- **Canonical `/idgames` page:** <https://www.doomworld.com/idgames/levels/doom2/megawads/av>.
  Filename `levels/doom2/megawads/av.zip`, author "Anders Johnsen et al". **Confirmed.**
- **Release text `AV.TXT`** (inside the ZIP, 55,068 bytes): title block reads *"ALIEN VENDETTA
  … THE COMPET-N EDITION 24.12.2002 releas[e]"*; `[1.3 Compatibility]` — *"AV does not require any
  port to run. It was constructed with vanilla Doom2 v1.9 exe in mind."*; `[6.1 Level authors]` lists
  the 13 contributors below; `[7.2 Levels]` carries the redistribution terms. **Confirmed (quoted).**
- **Two releases (Confirmed):** the original 2001-12-24 release and the **Compet-N edition** of
  2002-12-24, which changed almost every map and swapped MAP24/MAP25. The `/idgames` ZIP is the
  2002 Compet-N edition (mirror `Last-Modified`/listing date 2002-12-26; `AV.TXT` date 2002-12-24),
  so it is the final canonical build. No newer official build exists.
- **`/idgames` upload date:** the archive master lists `av.zip` at 2002-12-26. **Confirmed (dated
  observation).**

### Level authors (`AV.TXT` §6.1, quoted)

Mattias Berggren, Vincent Catalaa, Pablo Dictter, Yashar "GeminI" Garibzadeh, Madani El Hariri,
Martin Aalen Hunsager, Jan Endre Jansen, **Anders Johnsen** (project lead), Kristian Käll,
Kim André "Mutator" Malde, Anthony Soto, Brad Spencer, Lee Szymanski. **Confirmed.**

## 2. Canonical archive and byte-identical mirrors

The `/idgames` path is `levels/doom2/megawads/av.zip`; every mirror serves that `<dir>/<filename>`
under its own base (`upstream-sources.md` §3.4). DDL uses direct mirror URLs, never the `/idgames`
API (D-015). Three allowlisted hosts were **downloaded in full**; all serve the exact same bytes:

| Host (allowlisted) | URL | bytes | SHA-256 |
| --- | --- | --- | --- |
| `www.gamers.org` (archive master) | `https://www.gamers.org/pub/idgames/levels/doom2/megawads/av.zip` | 9,041,333 | `5c5382cc…bf8fc` |
| `youfailit.net` | `https://youfailit.net/pub/idgames/levels/doom2/megawads/av.zip` | 9,041,333 | `5c5382cc…bf8fc` |
| `mirrors.lug.mtu.edu` | `https://mirrors.lug.mtu.edu/idgames/levels/doom2/megawads/av.zip` | 9,041,333 | `5c5382cc…bf8fc` |
| `ftp.fu-berlin.de` | `https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/megawads/av.zip` | 9,041,333 | `5c5382cc…bf8fc` |

**Confirmed:** byte-identical, so they share one package record (D-015). `/idgames` supplies no
checksum; the SHA-256 was computed here from the canonical bytes.

## 3. Archive contents (ZIP, 9,041,333 bytes)

Every member is listed with its exact size and SHA-256 computed here. The archive is a DOS-era
release: it also ships a Win16/DOS launcher `AV.EXE`, which DDL **never** executes and does not
declare as a member.

| Path (case-exact) | Bytes | SHA-256 | Role |
| --- | --- | --- | --- |
| `AV.WAD` | 23,819,412 | `fe5fd9ebaaafdf948d5c518ab4cb4d78411a176e2264a89fd0edefb23b190091` | maps (main) |
| `AV.DEH` | 7,403 | `9fb5ab09b7c3aad40eba090cff4d50e74fa8907457a210c08d275a06b089c77b` | dehacked (main) |
| `AVMOVFIX.WAD` | 1,816,944 | `1fa72a1eba15022b8f794cf362df042072614da92606a3aef02d1872a1b86ffd` | optional maps (MAP20 variant) |
| `AV.TXT` | 55,068 | `52aca80bef87884359cc3f671616e37c35a6e8f98012423ee9109e1a265ec5df` | credits/readme (keep) |
| `CHANGES.TXT` | 7,660 | `95b49036375644882e5ac7c9f47ba83a05b109635743398497422731e59db750` | changelog (keep) |
| `AV.EXE` | 709,905 | `cc12d5b77f34e7258b4d2e7c0dfb922e0db8010394ccc93a40bdcf155a942cbd` | DOS launcher — **never executed, not declared** |

**Confirmed.** Flat layout, ASCII names, all six entries Deflate, no symlinks, no traversal. The
DOS launcher `AV.EXE` is outside DDL's model (hard rule: never execute anything from a downloaded
archive) and is intentionally not a recipe member.

## 4. WAD and patch inspection (read-only, no execution)

`AV.WAD` — **PWAD**, 23,819,412 bytes, **1,175 lumps**, 32 map markers `MAP01`–`MAP32`. Every map
is the standard ten-lump vanilla group. **Confirmed.**

- **No `COMPLVL` lump**, **no `DEHACKED` lump** and **no `UMAPINFO`/`MAPINFO`** are present in
  `AV.WAD`. **Confirmed.**
- `AVMOVFIX.WAD` — **PWAD**, 1,816,944 bytes, **11 lumps**, a single `MAP20` group (the alternate
  MAP20). **Confirmed.**
- `AV.DEH` — a DeHackEd v3.0 patch, `Doom version = 19`, `Patch format = 6`, containing only
  `Text` replacements: the intermission/story texts and the 32 intermission level names
  (e.g. `L20: Misri Halek`). It changes **no** things, frames or codepointers. **Confirmed (read,
  not executed).**

### DSDA-Doom v0.30.1 handling (pinned)

DDL launches with `-noautoload` (D-011). `-noautoload` skips directory autoload but does **not**
disable `DEHACKED`-in-a-WAD, and it does not affect patches passed on `-deh`:

- Embedded `DEHACKED` lumps from explicitly loaded PWADs are applied whenever `-nodeh` is absent,
  so `AV.WAD`'s (absent) patch would be automatic
  ([`prboom2/src/d_main.c:2147–2156`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2147-L2156)).
- `-deh`/`-bex` patches are processed in list order **after** WAD-embedded patches
  ([`prboom2/src/d_main.c:2108–2192`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2108-L2192)).
  Because `AV.DEH` is external (not embedded), the recipe must load it through the `-deh` list.
- **ZDoom `MAPINFO` support was removed in v0.30.0** ("Removed ZDoom MAPINFO support",
  <https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.0>), so any `MAPINFO`/`DECORATE`-style
  lumps are inert under v0.30.1; `AV.WAD` contains none anyway.
- The complevel cannot come from a lump and is declared by the recipe.

**Confirmed.** The launch expectation is `-iwad doom2.wad -file …/AV.WAD -deh …/AV.DEH -complevel 2`.

## 5. Requirements and compatibility

- **Base game:** DOOM II — `AV.TXT` §1 describes 32 levels for Doom II; Doomwiki lists IWAD Doom II.
  **Confirmed.**
- **Required port class:** Vanilla — *"constructed with vanilla Doom2 v1.9 exe in mind"* (`AV.TXT`
  §1.3); it was one of the few PWADs allowed for Compet-N recordings. **Confirmed.**
- **Complevel (Proposed):** **2 (Doom II 1.9)**, `family = "vanilla"` — the vanilla DOOM II
  compatibility level. **Proposed until Deck-tested.**
- **Warp/episode notes:** single continuous DOOM II mapset (MAP01–MAP32, no `UMAPINFO`); no `-warp`
  is prescribed. **Proposed.**
- **DEH/BEX:** load the external `AV.DEH` (main); the optional `AVMOVFIX.WAD` is a MAP20-only
  alternate (see §7). **Confirmed.**

## 6. Licence and redistribution

`AV.TXT` §7.2 states (**Confirmed (quoted)**):

```text
[7.2 Levels]

Authors may NOT use these levels as a base to build additional levels.
You may distribute this WAD, provided you include this file, with no
modifications. You may distribute this file in any electronic format (BBS,
Diskette, CD, etc) as long as you include this textfile intact.
```

Plus §7.1: AV-specific graphics (status bar/titlepic/interpic/bossback/menu) may not be reused.
**Proposed (DDL policy):** DDL downloads the unmodified ZIP and keeps both bundled text files
(`keep = true`), so the "include this textfile intact" clause is satisfied.

## 7. Optional component — `AVMOVFIX.WAD`

`AV.TXT` §1.8 says (**Confirmed (quoted)**): *"we discovered a limitation regarding map20 and
demorecording … we included an altered version of map20 as avmovfix.wad, its[b]asically map20 w/o
the huge mountain area … Note that this one is only of interest for demorecorders in Compet-N
context, and should be ignored if using a port such as zdoom."*

- It contains only a `MAP20` group, so loading it after `AV.WAD` overrides MAP20. **Confirmed
  (lump analysis).**
- It is a genuine, official, bundled alternate version of a map — the kind of optional component
  D-010/SIGIL's `compat` choice models — so the recipe exposes it as a **default-off** optional
  component with an honest description. It is **not** recommended for ordinary play.
- It has no maps beyond MAP20, no `DEHACKED`, no `COMPLVL` and no ZDoom lumps, so it changes
  neither IWAD nor complevel. **Confirmed.**

## 8. Composition (recipe data — matches `catalog/alien-vendetta.toml`)

- One package `free` (the ZIP above). Required `main` component loads `AV.WAD` (`maps`) **and**
  `AV.DEH` (`dehacked`).
- Optional `movfix` component (position 2, default off) loads `AVMOVFIX.WAD` (`maps`) as an add-on.
- Keep `AV.TXT` and `CHANGES.TXT` as `keep = true`; do not declare `AV.EXE`.
- `base_game = doom2`; `compatibility.family = "vanilla"`, `complevel = 2`,
  `complevel_status = "proposed"`; engine `=0.30.1`, `inspected`.

## 9. Open questions

- **Complevel 2.** Not declared by any lump; proposed from the vanilla-1.9 statement. Needs a Deck
  run (**Needs Deck Validation**).
- **`AV.EXE`.** The DOS launcher is the only way the release "officially" applies the par times
  (`AV.DEH` omits them). DSDA-Doom cannot run the DOS executable, so DDL applies `AV.DEH` only and
  does not reproduce the exe's par times. Documented, not a blocker.
- **`Alien Vendetta: Black Label` (2016).** A separate official two-level expansion on the AV
  website, not part of this `/idgames` package and not on an allowlisted host, so it is out of
  scope for this recipe.

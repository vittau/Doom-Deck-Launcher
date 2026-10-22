# Reference research — Swim With The Whales (spec 092)

Scope: the single `swim-with-the-whales` recipe. Every identity below was read from the downloaded
bytes; nothing was executed or extracted to disk. Labels: **Confirmed** (observed in this session or
quoted from a primary source), **Proposed** (a DDL inference), **Needs Deck Validation** (cannot be
checked on the macOS arm64 dev host).

Research date: **2026-10-22**. Host: macOS arm64. Tools: `curl`, `python3` (stdlib `zipfile`,
`struct`), `shasum`, `sips`, `scripts/lean-image.sh`. Scratch downloads live in `.scratch/087c/`
(git-ignored, never committed).

## 1. Method and sources

- **Archive.** The `/idgames` package `levels/doom2/Ports/s-u/swtw.zip` was fetched from
  `ftp.fu-berlin.de/pc/games/idgames/…` and re-downloaded from `mirrors.lug.mtu.edu/idgames/…` and
  `lethe.chinstrap.org/idgames/…`; all three were byte-identical (SHA-256 compared, §2). The archive
  stores its files under a `swtw/` directory. **Confirmed.**
- **Author text.** `swtw/swtw.txt` inside the zip was read byte-for-byte; the idgames sidecar
  `swtw.txt` is byte-identical to the member. **Confirmed.**
- **Release thread.** `doomworld.com` blocks automated clients, so the 2013 release thread was read
  through a raw Wayback `id_` capture (`20131223064809`) of
  <https://www.doomworld.com/vb/wads-mods/66260-swim-with-the-whales-final-release/>. Its first post
  is dated `11-20-13` and it carries the author's screenshots and download links. **Confirmed.**
- **Cacowards.** The 2013 page was read through a raw Wayback capture (`20131219012018`) of
  <https://www.doomworld.com/20years/best3.php>; the year page is
  <https://www.doomworld.com/cacowards/2013/>. **Confirmed.**
- **Engine evidence.** DSDA-Doom **v0.30.1** at the pinned tag: `prboom2/src/p_spec.c` (generalized
  dispatch, line 1636; `P_SpawnVanillaExtras`, 3449–3465 and 3689), `prboom2/src/p_spec.h:99–106`
  (generalized-line bases) and `prboom2/src/dsda/mapinfo/`, which holds only `legacy.c` (hardcoded
  Doom/Doom II progression), `u.c` (UMAPINFO) and `hexen.c` — no ZDoom MAPINFO reader.
  **Confirmed** as source reading, not gameplay.
- **DoomWiki was not read** (HTTP 403 to automated clients); no claim here relies on its contents.

## 2. Package and member bytes

**Confirmed.** `swtw.zip` is 2,719,261 B,
SHA-256 `ebd2317a3006614ce4afbd1f8c8294f34949e872860cbc2a17ed8de59d73aef7`. Members (case-exact,
nested one directory deep):

| Member | Size | SHA-256 | Loaded / kept |
| --- | --- | --- | --- |
| `swtw/swtw.wad` | 8,868,645 | `7f081d591f2fce923d08fe1f86ed52cc5b8bd10061c2c96711506ea6286390d7` | maps |
| `swtw/swtw.txt` | 8,328 | `521cb605249c52bbe3015cd62c1419c047bfcd2d4409cc17cfda5a106cf6dba0` | kept, unmodified |

Mirror identity: the same size and SHA-256 on `ftp.fu-berlin.de`, `mirrors.lug.mtu.edu` and
`lethe.chinstrap.org`. The idgames file's `Last-Modified` is **2013-12-21**; the readme inside it
describes the set itself, so this is the selected archive identity, not a repackaging.

## 3. Release identity and dates

| Field | Value | Evidence |
| --- | --- | --- |
| Title | Swim With The Whales | `swtw.txt` |
| Author | Ribbiks | `swtw.txt` |
| Readme date | 2013-12-02 (`Release date: 12/02/2013`) | `swtw.txt`; format evidence below |
| First public release | 2013-11-20 | release-thread first post dated `11-20-13` (Wayback capture 2013-12-23) |
| idgames archive file | 2013-12-21 | mirror `Last-Modified` |
| Game | Doom 2 | `swtw.txt` (`Game: Doom 2`) |
| Maps | 3 main maps plus a hidden MAP31 joke map | `swtw.txt` map list and the WAD directory (**Confirmed**) |
| Advanced engine | `Boom-Compatible (tested in prb+ cl9, gzdoom)`; `May Not Run With: Any port that doesn't support boom linedefs / voodoo doll shennanigans` | `swtw.txt` |
| `/idgames` | `levels/doom2/Ports/s-u/swtw.zip` | canonical page <https://www.doomworld.com/idgames/levels/doom2/Ports/s-u/swtw> |
| Cacowards | 2013 runner-up | <https://www.doomworld.com/cacowards/2013/> (edition page `/20years/best3.php`, capture 2013-12-19) |

**Date format (Confirmed).** Ribbiks writes month/day. His Stardate 20X6 readme says
`Release date: 06/02/2013` and that archive's `Last-Modified` is **2013-06-02**, so the readme means
June 2, not February 6. Applied here, `12/02/2013` is **December 2, 2013**, which also fits the
2013-11-20 thread start and the 2013-12-21 archive upload. The February reading is not used anywhere.
The release thread predates the readme date, so the **original public release is 2013-11-20** and
the idgames build carries the author's 2013-12-02 revision.

## 4. WAD inventory and compatibility

**Confirmed.** `PWAD`, 320 lumps, four map markers: `MAP01`, `MAP02`, `MAP03` and the joke `MAP31`.
One embedded `DEHACKED` lump (513 B, `[STRINGS]` only: map names plus dark-blue/light-blue/black key
pickup text). One `MAPINFO` lump (625 B) in **ZDoom syntax** (`defaultmap`, `map MAP01 "…"` blocks
with `titlepatch`, `next`, `sky1`, `cluster`, `music`) — not a Boom format. No `COMPLVL`, `UMAPINFO`
or `EMAPINFO`.

**Compatibility reasoning (Confirmed).** The author requires a Boom-compatible engine and names
`prb+ cl9`; the maps agree: all four use Boom generalized line types, **134 lines** in total
(MAP01 5, MAP02 53, MAP03 64, MAP31 12; families: 118 floor, 6 lift, 6 ceiling, 2 locked-door,
1 door, 1 crusher). Vanilla compatibility does not implement those types (`p_spec.c:1636`), so
`family = "boom"` and `complevel = "boom"` (9).

**Metadata and progression (Confirmed).** DSDA-Doom v0.30.1 has no ZDoom MAPINFO reader
(`prboom2/src/dsda/mapinfo/` holds only `legacy.c`, `u.c`, `hexen.c`), so the lump's titles and
per-map music do not apply. With no `UMAPINFO`/`COMPLVL`, progression is the hardcoded Doom II
next-map table in `legacy.c`: after the custom `MAP03` the engine loads the base IWAD's stock
`MAP04`, and a normal exit from the custom `MAP31` goes to the base IWAD's `MAP32`. The player quits
from the intermission to stop after the set. **Needs Deck Validation:** the titles/music heard and
these transitions.

## 5. Original public date for spec 090

- **Best sourced original public date: 2013-11-20, day precision.** Source: the author's release
  thread first post, dated `11-20-13`, read through the Wayback capture of 2013-12-23.
- The readme date **2013-12-02** (month/day) is the author's later revision of the set; the idgames
  file (**2013-12-21**) is the archive upload of that revision.
- No earlier public build is evidenced; DoomWiki could not be read (403), so nothing is inferred
  from it.

## 6. Tags

| Tag | Evidence |
| --- | --- |
| `cacowards` | 2013 runner-up, <https://www.doomworld.com/cacowards/2013/> (edition page `/20years/best3.php`, capture 2013-12-19). |
| `classics` | Public since 2013-11-20, over five years before this review, with that Cacowards recognition. |
| `episode` | Four map slots (three main maps plus the hidden joke map), counted from the WAD and the author's map list; nine or fewer. |
| `challenging` | Author: "very challenging gameplay"; UV is "Not messing around!"; the title map is "Maybe the hardest level I've made"; he encourages even skilled players to start on HMP. |

Not tagged `megawad` (four slots). Not tagged `gimmick`: the owner's gimmick assignment covers The
10x10 Project and Linear Doom only.

## 7. Artwork

One in-game screenshot posted by the author in the release thread
(<https://i.imgur.com/9833QJQ.png>, 1024×768, archived thread capture 2013-12-23): a lit blue
fortress interior with the player's weapon and HUD visible. Provenance and permission text are in
`crates/ddl/ui/assets/screenshots/swim-with-the-whales/SOURCES.toml`. Two other author screenshots
were inspected but are too dark for a card; no synthesis was attempted.

## 8. Verification commands and results

Run from the worktree root on 2026-10-22 with
`CARGO_TARGET_DIR=/Users/vitor/git/Doom-Deck-Launcher/target` and `CARGO_BUILD_JOBS=2`:

| Command | Result |
| --- | --- |
| `ddl catalog lint catalog/swim-with-the-whales.toml` | `Catalog lint passed (offline).` |
| `ddl catalog lint --fetch catalog/swim-with-the-whales.toml` | `Catalog lint passed (fetched packages verified).` |
| `cargo test -p ddl-core --test catalog_092 -- --test-threads=2` | 8 passed, 0 failed |
| `cargo clippy -p ddl-core --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |

## 9. Open checks (all **Needs Deck Validation**)

- IWAD detection and launch with DOOM II.
- Complevel `boom` (9) behaviour and the Boom-only line/voodoo effects.
- Embedded DEHACKED strings (map names, key pickup text).
- Inert ZDoom `MAPINFO`: map titles and music fall back to engine behaviour.
- Progression: `MAP03` → stock `MAP04`, normal `MAP31` exit → stock `MAP32`; stop from the
  intermission after the set.
- Gameplay difficulty across HNTR/HMP/UV, audio (three MIDI tracks plus title/intermission music),
  performance of the three large abstract maps, and the author-documented slime trails with
  software rendering.
- Controller navigation of the map menu and the fallback card art for any title without art.

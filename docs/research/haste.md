# Reference research — Haste (spec 093)

Scope: the single `haste` recipe. Every identity below was read from the downloaded bytes; nothing
was executed or extracted to disk. Labels: **Confirmed** (observed in this session or quoted from a
primary source), **Proposed** (a DDL inference), **Needs Deck Validation** (cannot be checked on the
macOS arm64 dev host).

Research date: **2026-10-22**. Host: macOS arm64. Tools: `curl`, `python3` (stdlib `zipfile`,
`struct`), `shasum`, `sips`, `scripts/lean-image.sh`. Scratch downloads live in `.scratch/087c/`
(git-ignored, never committed).

## 1. Method and sources

- **Archive.** `levels/doom2/Ports/megawads/haste.zip` was fetched from
  `ftp.fu-berlin.de/pc/games/idgames/…` and re-downloaded from `mirrors.lug.mtu.edu/idgames/…` and
  `lethe.chinstrap.org/idgames/…`; all three were byte-identical (SHA-256 compared, §2).
  **Confirmed.**
- **Author text.** `Haste.txt` inside the zip was read byte-for-byte; the idgames sidecar is
  byte-identical to the member. **Confirmed.**
- **Cacowards.** The 2021 Top Twelve page 4 was read through a raw Wayback `id_` capture
  (`20211210073502`) of <https://www.doomworld.com/cacowards/2021/best4/>; the year page is
  <https://www.doomworld.com/cacowards/2021/>. **Confirmed.**
- **Engine evidence.** DSDA-Doom **v0.30.1** at the pinned tag: `prboom2/src/p_spec.c` (generalized
  dispatch, line 1636; `P_SpawnVanillaExtras`, 3449–3465 and 3689), `prboom2/src/p_spec.h:99–106`
  (generalized-line bases) and `prboom2/src/dsda/mapinfo/` (only `legacy.c`, `u.c` and `hexen.c`; no
  ZDoom MAPINFO reader). **Confirmed** as source reading, not gameplay.
- **DoomWiki was not read** (HTTP 403 to automated clients); no claim here relies on its contents.

## 2. Package and member bytes

**Confirmed.** `haste.zip` is 5,519,763 B,
SHA-256 `f71eb1623da6c5fc02c998499dc123b45db5c8f56ad3d8a5f92bbc800f033f12`. Members:

| Member | Size | SHA-256 | Loaded / kept |
| --- | --- | --- | --- |
| `Haste.wad` | 16,931,409 | `fac36c642771ccd7336814bbff443e093ddf09469b156017a5bf2d7312d01818` | maps |
| `Haste.txt` | 4,629 | `8bdd5194682b362b4f3884d74ae65564a05e27d2abd8d6c8a4f2918ca966368d` | kept, unmodified |

Mirror identity: same size and SHA-256 on `ftp.fu-berlin.de`, `mirrors.lug.mtu.edu` and
`lethe.chinstrap.org`. The idgames file's `Last-Modified` is **2022-01-08**.

## 3. Release identity and dates

| Field | Value | Evidence |
| --- | --- | --- |
| Title | Haste | `Haste.txt` |
| Project leads | Insane_Gazebo, Scotty | `Haste.txt` |
| Mappers | Insane_Gazebo, Scotty, Nirvana, Bemused, Ribbiks, Benjogami, Archi | `Haste.txt` map credits |
| Readme date | 2021-09-22 (`22/09/2021`, day-first and unambiguous) | `Haste.txt` |
| idgames archive file | 2022-01-08 | mirror `Last-Modified` |
| Game | DOOM2 | `Haste.txt` (`Game: DOOM2`) |
| Maps | 18 slots: MAP01–MAP17 plus secret MAP31 | `Haste.txt` (`New levels: 18`) and the WAD directory (**Confirmed**) |
| Advanced engine | `Boom`; tested with `DSDA-Doom 0.2.3`; `May Not Run With: zDaemon as several maps use extended nodes` | `Haste.txt` |
| `/idgames` | `levels/doom2/Ports/megawads/haste.zip` | canonical page <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/haste> |
| Cacowards | 2021 Top Twelve, page 4 — winner | <https://www.doomworld.com/cacowards/2021/best4/> (capture 2021-12-10) |

**Original public date (Confirmed, day precision): 2021-09-22.** The author's readme date is the
earliest release evidence found; the Cacowards write-up (published December 2021) describes it as a
2021 release, and the idgames upload is four months later (2022-01-08). DoomWiki was not readable,
so no earlier build is claimed or inferred.

## 4. WAD inventory, compatibility and progression

**Confirmed.** `PWAD`, 500 lumps, eighteen map markers (`MAP01`–`MAP17`, `MAP31`). One embedded
`DEHACKED` lump (593 B, `[STRINGS]` only: map names for all eighteen slots). One `MAPINFO` lump
(2,192 B) in **ZDoom syntax** (`defaultmap`, `map MAP01 "…"` blocks with `titlepatch`, `next`,
`sky1`, `cluster`, `music`) — not a Boom format. No `COMPLVL`, `UMAPINFO` or `EMAPINFO`.

**Compatibility reasoning (Confirmed).** The author requires Boom and lists DSDA-Doom 0.2.3 among
tested engines; 17 of the 18 maps use Boom generalized line types, **390 lines** in total
(267 floor, 71 lift, 22 door, 20 locked-door, 9 ceiling, 1 crusher). The one map without generalized
types is the MAP31 secret. Vanilla compatibility does not implement those types (`p_spec.c:1636`),
so `family = "boom"` and `complevel = "boom"` (9). Extended nodes are noted by the author; DSDA-Doom
supports them, but that stays a Deck check.

**Metadata and progression (Confirmed).** DSDA-Doom v0.30.1 has no ZDoom MAPINFO reader
(`prboom2/src/dsda/mapinfo/` holds only `legacy.c`, `u.c`, `hexen.c`), so the lump's titles and
per-map music do not apply. With no `UMAPINFO`/`COMPLVL`, progression is the hardcoded Doom II
next-map table in `legacy.c`: the author's `MAP17` is `Thanks for Playing`, after which the run would
continue into the base IWAD's `MAP18`; a normal exit from the secret `MAP31` leads to the base IWAD's
`MAP32`. The player quits from the intermission to stop after the ending map. **Needs Deck
Validation:** the titles/music heard and these transitions.

## 5. Tags

| Tag | Evidence |
| --- | --- |
| `cacowards` | 2021 Top Twelve page 4 winner, <https://www.doomworld.com/cacowards/2021/best4/> (capture 2021-12-10). |
| `community-projects` | Readme credits two project leads and seven named mappers. |
| `classics` | Released 2021-09-22, over five years before this review, with that Cacowards recognition. |

Not `megawad`: the archive directory is named `megawads`, but the WAD and readme have eighteen
slots, fewer than the thirty the tag requires. Not `episode` (more than nine). Not `challenging`:
the Cacowards write-up says the set was built to be approachable to players who do not live and
breathe hardcore gameplay, so no experienced-players claim is made. Not `gimmick`: the owner's
gimmick assignment covers The 10x10 Project and Linear Doom only.

## 6. Artwork

One in-game screenshot published with the Cacowards 2021 feature
(<https://static.doomworld.com/pages_media/10_haste.jpg?_cb=1638911191>, 1920×1080, read through the
2021-12-10 Wayback capture): a lit industrial-gothic hall with green sludge channels. Provenance and
permission text are in `crates/ddl/ui/assets/screenshots/haste/SOURCES.toml`.

## 7. Verification commands and results

Run from the worktree root on 2026-10-22 with
`CARGO_TARGET_DIR=/Users/vitor/git/Doom-Deck-Launcher/target` and `CARGO_BUILD_JOBS=2`:

| Command | Result |
| --- | --- |
| `ddl catalog lint catalog/haste.toml` | `Catalog lint passed (offline).` |
| `ddl catalog lint --fetch catalog/haste.toml` | `Catalog lint passed (fetched packages verified).` |
| `cargo test -p ddl-core --test catalog_093 -- --test-threads=2` | 8 passed, 0 failed |
| `cargo clippy -p ddl-core --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |

## 8. Open checks (all **Needs Deck Validation**)

- IWAD detection and launch with DOOM II; `boom` (9) behaviour, including the extended nodes.
- Embedded DEHACKED map names; inert ZDoom `MAPINFO` titles/music.
- Progression after the `MAP17` ending map and from a normal `MAP31` exit (stock `MAP18`/`MAP32`);
  stopping from the intermission.
- Gameplay, audio and performance of the compact slaughter maps; no framerate, memory or battery
  measurement was made.

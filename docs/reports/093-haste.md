# 093 — Haste

**Spec:** `docs/specs/093-haste.md` (coordinator `verify/087-pwads`) ·
**Branch:** `vittau/pwads-087-c` · **Date:** 2026-10-22

## What changed

- Added `catalog/haste.toml`: one DOOM II recipe for the community project, one `free` `/idgames`
  package with case-exact `Haste.wad` and `Haste.txt` (kept as credits), and one required main
  component.
- Added `crates/ddl/ui/assets/screenshots/haste/01.jpg` (in-game screenshot from the 2021 Cacowards
  feature) with provenance in `SOURCES.toml`.
- Added `docs/research/haste.md` (evidence) and this report.
- Added `crates/ddl-core/tests/catalog_093.rs`: eight offline tests covering embedding without
  skips, package/member identity, single-WAD load order with embedded DEHACKED, base
  game/complevel, tags (including the no-megawad and no-challenging decisions), product-copy rules,
  dates and the screenshot link.
- Also applied the narrow product-copy follow-up to the earlier six recipes: their root and
  component descriptions no longer say "Boom-compatible"/"limit-removing" and no longer list
  map-slot codes; the theme, map counts and campaign description stay, and every technical fact,
  pin and limitation is untouched.

## Compatibility, composition and dates

- **Base game:** DOOM II (`Game: DOOM2`). **Family/complevel:** boom (9). The readme says
  `Advanced engine needed: Boom`, lists DSDA-Doom 0.2.3 among tested engines, and warns that several
  maps use extended nodes; 17 of 18 maps use Boom generalized line types (390 lines), which vanilla
  compatibility does not implement.
- **Composition:** eighteen slots in the WAD (`MAP01`–`MAP17` plus secret `MAP31`); only
  `Haste.wad` is loaded. The embedded DEHACKED patch is inside that WAD (map names only), so no
  separate `-deh` member is declared.
- **Metadata:** the bundled `MAPINFO` is ZDoom syntax and DSDA-Doom v0.30.1 has no ZDoom MAPINFO
  reader, so titles/music fall back to engine behaviour. With no `UMAPINFO`/`COMPLVL`, the hardcoded
  Doom II table would continue from the `MAP17` "Thanks for Playing" map into the base IWAD's
  `MAP18`, and a normal `MAP31` exit leads to the base IWAD's `MAP32`; the player quits from the
  intermission to stop.
- **Dates:** the readme's `22/09/2021` is day-first and unambiguous → **2021-09-22** (original
  public date, day precision) for spec 090. The idgames file is the **2022-01-08** upload. No
  chronology field was added; worker D owns that schema.

## Tags

`cacowards` (2021 Top Twelve page 4 winner), `community-projects` (two project leads and seven
credited mappers) and `classics` (released 2021-09-22, over five years before this review). No
`megawad`: the `/idgames` directory says `megawads`, but the WAD and readme have eighteen slots,
fewer than thirty. No `episode` (more than nine). No `challenging`: the Cacowards write-up says the
set was built to be approachable to players who do not live and breathe hardcore gameplay. No
`gimmick` (the owner's assignment covers The 10x10 Project and Linear Doom only).

## Product copy

Root description: eighteen compact, high-impact maps moving from a gore-soaked prison tower and
phosphorescent burrows to a Plutonian floodplain and blazing temples, built for fast, fluid play in
ten-minute bursts, with an ending map and a hidden secret level. Component: the full campaign with
the ending map and secret level. No port-family names, map-slot codes, lump/patch details or engine
names; a test enforces the boundary and the evidence stays in verification.

## Artwork

One real in-game screenshot from the 2021 Cacowards feature
(`static.doomworld.com/pages_media/10_haste.jpg`, 1920×1080), resized to 1280×720 and saved as a
156,624 B JPEG (limit 192 KiB), with source URL, page, credit, original size, retrieval date and the
no-licence-found note in `SOURCES.toml`.

## Verification

| Check | Result |
| --- | --- |
| Offline lint (`ddl catalog lint catalog/haste.toml`) | `Catalog lint passed (offline).` |
| Fetch lint (`--fetch`) | `Catalog lint passed (fetched packages verified).` |
| `cargo test -p ddl-core --test catalog_093 -- --test-threads=2` | 8 passed, 0 failed |
| `cargo clippy -p ddl-core --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |

Shared `CARGO_TARGET_DIR=/Users/vitor/git/Doom-Deck-Launcher/target`, `CARGO_BUILD_JOBS=2`, test
threads 2. No full workspace/Slint run (coordinator does the combined pass). Main was merged before
the final checks; the combined recipe-count assertion remains coordinator-owned.

## Open questions / Needs Deck Validation

- IWAD detection and launch with DOOM II; `boom` (9) behaviour and extended-node handling.
- Embedded DEHACKED map names; inert ZDoom MAPINFO titles/music.
- Progression after the `MAP17` ending map and from a normal `MAP31` exit (stock `MAP18`/`MAP32`);
  stopping from the intermission.
- Gameplay, audio and performance of the compact maps; no framerate, memory or battery measurement
  was made.
- DoomWiki was not readable (403), so no claim depends on it.

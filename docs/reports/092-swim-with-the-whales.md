# 092 — Swim With The Whales

**Spec:** `docs/specs/092-swim-with-the-whales.md` (coordinator `verify/087-pwads`) ·
**Branch:** `vittau/pwads-087-c` · **Date:** 2026-10-22

## What changed

- Added `catalog/swim-with-the-whales.toml`: one DOOM II recipe for Ribbiks's release, one `free`
  `/idgames` package with the case-exact nested members `swtw/swtw.wad` and `swtw/swtw.txt` (kept
  as credits), and one required main component.
- Added `crates/ddl/ui/assets/screenshots/swim-with-the-whales/01.jpg` (author screenshot from the
  release thread) with provenance in `SOURCES.toml`.
- Added `docs/research/swim-with-the-whales.md` (evidence) and this report.
- Added `crates/ddl-core/tests/catalog_092.rs`: eight offline tests covering embedding without
  skips, package/member hashes and nested case-exact paths, single-WAD load order with the embedded
  DEHACKED, base game/complevel, tags, product-copy rules, date precision and the screenshot link.
- Earlier six recipes, shared tests, `tags.toml`, `hosts.toml`, runtime/UI and coordinator docs were
  not touched. No new dependency, host, engine or archive execution.

## Compatibility, composition and dates

- **Base game:** DOOM II (`Game: Doom 2`). **Family/complevel:** boom (9). The author states
  `Boom-Compatible (tested in prb+ cl9, gzdoom)` and forbids ports without Boom linedefs; all four
  maps use Boom generalized line types (134 lines), which vanilla compatibility does not implement.
- **Composition:** four map slots in the WAD (`MAP01`–`MAP03` plus the joke `MAP31`); the recipe
  loads only `swtw/swtw.wad`. The embedded DEHACKED patch (map names and key pickup text) is part of
  that WAD, so no separate `-deh` member is declared.
- **Metadata:** the bundled `MAPINFO` is ZDoom syntax and DSDA-Doom v0.30.1 has no ZDoom MAPINFO
  reader, so titles/music do not apply. With no `UMAPINFO`/`COMPLVL`, the hardcoded Doom II table
  continues from the custom `MAP03` into the base IWAD's `MAP04`, and a normal `MAP31` exit leads to
  the base IWAD's `MAP32`; the player quits from the intermission to stop.
- **Dates:** the readme says `12/02/2013`; Ribbiks writes month/day (his Stardate 20X6 readme
  `06/02/2013` matches that archive's 2013-06-02 upload), so the readme date is **2013-12-02**. The
  release thread's first post is dated **2013-11-20**, which is the sourced **original public
  release** (day precision) for spec 090; the idgames file is the **2013-12-21** upload of the
  author's revision. No chronology field was added here — worker D owns that schema.

## Tags

`cacowards` (2013 runner-up, [year page](https://www.doomworld.com/cacowards/2013/), edition page
`/20years/best3.php` read through the 2013-12-19 Wayback capture), `classics` (public since
2013-11-20), `episode` (four map slots, nine or fewer) and `challenging` (the author's own
"very challenging gameplay", UV "Not messing around!", HMP advice for skilled players). No `megawad`
(four slots). No `gimmick`: the owner's gimmick assignment covers The 10x10 Project and Linear Doom
only.

## Product copy

Root description: three maps through abstract blue otherworlds — sunken fortresses, cathedral-like
ruins and a waterlogged volcanic wasteland — with elaborate secrets, a hidden joke map, and the
author's advice that even veteran players start on HMP. Component: the three-map campaign plus the
hidden joke map. The descriptions contain no port-family names, map-slot codes, lump/patch
implementation or engine-source talk; those facts live in `verification.inspected`, `notes`,
`limitations` and the research. A test enforces the boundary.

## Artwork

One real in-game screenshot posted by the author in the release thread
(`i.imgur.com/9833QJQ.png`, 1024×768), cropped to 1024×576 and saved as a 79,689 B JPEG (limit
192 KiB), with source URL, page, credit, original size, retrieval date and the no-licence-found note
in `SOURCES.toml`. Two other author screenshots were inspected and rejected as too dark for a card;
nothing was synthesized.

## Verification

| Check | Result |
| --- | --- |
| Offline lint (`ddl catalog lint catalog/swim-with-the-whales.toml`) | `Catalog lint passed (offline).` |
| Fetch lint (`--fetch`) | `Catalog lint passed (fetched packages verified).` |
| `cargo test -p ddl-core --test catalog_092 -- --test-threads=2` | 8 passed, 0 failed |
| `cargo clippy -p ddl-core --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |

Shared `CARGO_TARGET_DIR=/Users/vitor/git/Doom-Deck-Launcher/target`, `CARGO_BUILD_JOBS=2`, test
threads 2. No full workspace/Slint run (coordinator does the combined pass). Main was merged before
the final checks; the combined recipe-count assertion remains coordinator-owned.

## Open questions / Needs Deck Validation

- IWAD detection and launch with DOOM II; `boom` (9) behaviour and the Boom-only line effects.
- Embedded DEHACKED strings, and the inert ZDoom MAPINFO (titles/music fall back to engine
  behaviour).
- Progression after `MAP03` and from a normal `MAP31` exit (stock `MAP04`/`MAP32`); stopping from the
  intermission.
- Gameplay difficulty across HNTR/HMP/UV, audio, performance of the three large abstract maps, and
  the author-documented slime trails with software rendering.
- No other open questions; DoomWiki was not readable (403), so no claim depends on it.

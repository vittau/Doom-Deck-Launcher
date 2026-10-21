# Report — 066 Antaresian Reliquary catalog entry

**Task:** implement `docs/specs/066-antaresian-reliquary.md` · **Branch:** `vittau/antaresian-reliquary`
**Date:** 2026-10-21 · **Worker:** Content Catalog

## What changed

- Added `catalog/antaresian-reliquary.toml`: one Doom II / Boom recipe for the official
  `levels/doom2/Ports/a-c/anta_req.zip` archive, with three byte-identical download URLs, exact
  archive and member sizes/SHA-256, the bundled credits text kept, an embedded-DEHACKED-only load
  order, `complevel = "boom"`, `pistol_start = true`, and no optional components.
- Added `crates/ddl/ui/assets/screenshots/antaresian-reliquary/` with two author-posted in-game
  JPEGs (`01.jpg`, `02.jpg`) and `SOURCES.toml` recording source URLs, source page, credit,
  original size, retrieval date and permission text.
- Added `docs/research/reference-antaresian-reliquary.md` with sources, mirror/hash evidence,
  archive and WAD inspection, map-slot findings, screenshots, labels and limitations.
- Added `antaresian-reliquary` to the embedded-catalog membership expectation in
  `crates/ddl-core/src/catalog/tests.rs` (the only test change; no core behaviour changed).
- Merged `origin/main` before final checks. That merge brought spec 066 R5 (owner-requested MAP31
  warning) and concurrent Flatpak work; only the spec/doc merge was relevant here.

The player-facing description carries the required R5 warning verbatim in spirit: “MAP31 contains
a massive number of enemies and performs poorly on Steam Deck.” Verification notes record that this
is an explicit owner request (2026-10-21) based on owner knowledge, not a DDL measurement.

## Evidence and findings

- **Release/author:** `2020-08-24`, Seongbae Park (`antares031`), per the bundled `ANTA_REQ.txt`
  and archive timestamps. DoomWiki calls it a nine-level Boom-compatible Doom II PWAD.
- **Archive:** 12,793,564 bytes,
  `f81e4a3e818b308438d322a1a716c53c43327ffa8e1ed04ab880a5221ea37f2e`. Downloaded in full from
  `www.gamers.org`, `dsdarchive.com`, `youfailit.net`, `mirrors.lug.mtu.edu` and
  `ftp.fu-berlin.de`; all five were byte-identical, so the recipe lists the first three.
- **Members (exactly two, no top-level directory):**
  - `ANTA_REQ.wad` — 46,829,703 bytes,
    `a34447b2aeb021f09fdad4d153fe8177833b0dbf41fb74e6b8266f02959f3ab9`.
  - `ANTA_REQ.txt` — 10,216 bytes,
    `adfa6a1e6dfae70a499eef4342e6c554d3cd87fd079c1ab38303580153dec754` (kept).
- **WAD:** PWAD, 1,499 lumps, nine map markers (MAP01–MAP07, MAP31, MAP32). One embedded
  `DEHACKED` lump (24,232 bytes, Doom version 21, patch format 6); no separate `.deh`/`.bex`
  member. No `COMPLVL` and no `UMAPINFO`; `ZMAPINFO` (2,440 bytes), `DECORATE` and `LANGUAGE` are
  ZDoom-only and inert under DSDA-Doom v0.30.1, which removed ZDoom MAPINFO support in v0.30.0.
- **Complevel:** `boom` from the author's `Advanced engine needed : Boom` and the release thread's
  "boom-compatible", tested by the author with PrBoom+ 2.5.1.5. No gameplay test was performed.
- **Map-count discrepancy:** the TXT and Cacowards page say “7 maps”; the WAD and DoomWiki list
  nine slots. The seven compilation maps are MAP01–MAP06 + MAP31; MAP07 and MAP32 are the two
  added levels. The recipe says nine map slots and explains the discrepancy in its notes.
- **Permission:** CC BY-NC 4.0, with full bundled credits; the recipe preserves the TXT. No
  commercial use is claimed or implied.
- **Screenshots:** author-posted 800×600 in-game views from the release thread, downloaded from
  `https://i.imgur.com/Qbj6Qyd.png` and `https://i.imgur.com/hkRdHBV.png`, converted with
  `scripts/lean-image.sh`; committed sizes are 116,603 and 79,627 bytes. No separate image licence
  text was found; the entries state D-024 bundling and removal on request.
- **R5 warning source:** owner request plus DoomWiki's MAP31 page; no FPS, enemy count or benchmark
  was invented.

## Commands and results

Archive/member verification (scratch directory, nothing committed or executed):

```sh
curl -sSL --max-time 120 -A "DoomDeckLauncher catalog review" \
  -o anta_req.zip \
  "https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/anta_req.zip"
shasum -a 256 anta_req.zip
unzip -l anta_req.zip
unzip -o -q anta_req.zip -d extract
shasum -a 256 extract/*
```

The other mirror URLs and the read-only WAD lump/map inspection were run the same way. Direct
DoomWiki/Doomworld pages returned HTTP 403; archived snapshots were used and labelled as such.

Recipe and catalog checks after merging main:

```sh
cargo run -q -p ddl -- catalog lint catalog/antaresian-reliquary.toml
# Catalog lint passed (offline).

cargo run -q -p ddl -- catalog lint catalog/antaresian-reliquary.toml --fetch
# Catalog lint passed (fetched packages verified).

cargo test -q -p ddl-core catalog::
# 100 passed; 0 failed.

cargo fmt --all --check
# no output, exit 0.

cargo clippy --workspace --all-targets -- -D warnings
# Finished dev profile; no warnings.

cargo test --workspace
# exit 0; all workspace, integration and doc tests passed.
```

## Spec coverage

- **R1:** recipe added; release, author, IWAD, engine, complevel, map slots and dependencies come
  from the bundled TXT and inspected bytes. The Cacowards path was verified, not trusted.
- **R2:** canonical `/idgames` path plus allowlisted byte-identical mirrors; archive/member
  SHA-256 and sizes computed from real bytes; credits text kept; one WAD loaded with its embedded
  DEHACKED only; nothing executed.
- **R3:** two real in-game screenshots with source/credit/provenance under the JPEG/size policy;
  tags are limited to evidence-backed `classics`, `cacowards` and `episode`; no gameplay/Deck claim.
- **R4:** research file uses Confirmed / Proposed / Needs Deck Validation labels and records the
  403/Wayback limitations.
- **R5:** player-facing MAP31 warning is present and concise; owner provenance recorded.

## Open questions / limitations

- **Needs Deck Validation:** IWAD/complevel, load order, gameplay, audio and the practical MAP31
  performance warning.
- **Needs Deck Validation:** MAP31/MAP32 reachability and the MAP07/MAP32 endings under DSDA-Doom.
  The author's `ZMAPINFO` is inert under v0.30.1; MAP31/MAP32 need `-warp`/IDCLEV until DDL exposes
  level selection. This is recorded in the recipe limitations and research, not silently “fixed”.
- DoomWiki/Doomworld direct access is Cloudflare-blocked from this host; those facts are from
  archived snapshots and are labelled secondary where appropriate.
- No new hosts and no new dependencies. No application behaviour, UI, launch/config or packaging
  files were changed.
- No push, tag, release or merge to `main`; the branch contains four small commits plus the
  `origin/main` merge commit.

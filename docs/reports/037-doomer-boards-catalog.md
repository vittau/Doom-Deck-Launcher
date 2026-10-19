# 037 — Doomer Boards Projects in the catalog (worker report)

**Spec:** `docs/specs/037-doomer-boards-catalog.md` · **Decision:** D-028 (extended 2026-10-19)
**Branch:** `vittau/doomer-boards` · **Date:** 2026-10-19

## What changed

- `docs/research/doomer-boards-projects.md` (**R1**) — the inventory of the Project Catalog:
  **77 projects, DBP 01–77**, one row each with the release thread, release date, map count,
  IWAD, target port/format, the /idgames path where it exists, the other published downloads
  and the DSDA-Doom verdict with its evidence link, plus per-project notes and the labels
  Confirmed / Proposed / Needs Deck Validation.
- `catalog/<slug>.toml` (**R2, R5**) — **76 new recipes, one per DBP** (DBP37 already had
  `auger-zenith`), written and committed in eight waves of roughly ten:
  01–10, 11–20, 21–30, 31–40, 41–52, 53–69, the Dropbox-only projects, 73–77.
  Each recipe follows spec 002 and `auger-zenith`: package and members with sizes and SHA-256
  taken from a real download, explicit load order, IWAD, complevel with its evidence, the
  bundled credits text kept, and tags from `catalog/tags.toml`.
- `catalog/hosts.toml` (**R3**) — added one host, `ia600903.us.archive.org`, the pinned download
  host of the archive.org item `dbp-wad-archive`. `www.gamers.org`, `dsdarchive.com`,
  `youfailit.net` and `ftp.fu-berlin.de` were already allowlisted. **No new tag**: the existing
  `community-projects` tag is exactly this family ("Multi-author community projects such as the
  Doomer Boards Projects"), so `tags.toml` is untouched.
- `crates/ddl/ui/assets/screenshots/<slug>/{01.jpg,SOURCES.toml}` (**R4**) — 67 bundled in-game
  screenshots, JPEG, quality 60, 640×360…1280×800, ≤ 192 KiB (the spec 035 rule), each with
  source URL, source page, credit, original size, retrieval date and permission text.
- `crates/ddl-core/src/catalog/tests.rs` — the embedded-catalog test enumerated the 13 original
  recipe ids; its list now names all 91 shipped recipes, sorted. This is the only Rust file
  touched, as the task's ownership note allows.

## Source selection (D-028)

| source | projects |
| --- | --- |
| /idgames (`www.gamers.org`, `youfailit.net`, `ftp.fu-berlin.de`) | **43** |
| DSDA archive mirror (`dsdarchive.com`) | **9** |
| archive.org `dbp-wad-archive` (`ia600903.us.archive.org`) | **24** |
| Dropbox | **0** (see below) |

Dropbox is unusable in practice, which contradicts D-028's premise: `www.dropbox.com` does not
redirect to `dl.dropboxusercontent.com` but to a **per-request** host such as
`uc6b74966005c2b22696e15c5150.dl.dropboxusercontent.com`, and three identical requests returned
three different hashes. The downloader's allowlist is an exact host comparison *by design*
(`hosts.rs`, test `does_not_match_subdomains_or_suffixes`), so neither `lint --fetch` nor an
install can authorise it. The 21 projects that publish only on Dropbox are therefore pinned from
the archive.org item (the same host the coordinator approved for DBP41/60/69); every hash and
member list was recomputed from those ZIPs, and several are different builds than the Dropbox
ones (e.g. DBP77 v1 is 6,361,417 bytes vs 6,036,661 on Dropbox). The Dropbox versions of those
21 recipes were generated first and are kept untouched in `target/dbp/hold/` (scratch, not
committed) in case a downloader suffix rule lands.

## How each acceptance criterion was verified

- **A1 (R1)** — the inventory covers every project the catalog page publishes. Verified by
  parsing the page's card list: `--imgdbpNN` background variables for **NN = 01…77, no gaps**;
  the stylesheet reserves `--imgdbp01`…`--imgdbp99`, which is where the spec's "01–99" comes
  from (**Confirmed**). All 77 release threads were read from Wayback snapshots for the release
  date, the port statement and the download links; every /idgames entry's readme was read for
  `Advanced engine needed`, `Tested With` and `Map #`.
- **A2 (R2, R3, R5)** — `ddl catalog lint --fetch <recipe>` was run for **every one of the 76 new
  recipes** and passed (archive size, SHA-256 and every named member verified from the real
  bytes). `ddl catalog lint` (offline, whole directory) and `ddl catalog bundle` pass;
  `cargo test -p ddl-core` and `cargo test -p ddl --test catalog_cli` are green; `cargo fmt
  --all --check` is clean; `cargo clippy -p ddl-core --all-targets -- -D warnings` is clean.
  The full-workspace clippy/test run is recorded at the end of this report.
- **A3 (R4)** — 67 of the 76 recipes carry a screenshot. The nine without one are listed in the
  inventory (§ Screenshots) with the reason — the release thread only posts a title screen, a
  logo or a movie still: **DBP08, DBP24, DBP31, DBP32, DBP41, DBP44, DBP55, DBP63, DBP69**.
  Four picks are multi-view collages posted by the authors (DBP19, DBP40, DBP43, DBP65) and four
  came from a Wayback copy of a dead image host (DBP59, DBP60, DBP61, DBP64); both facts are in
  the matching `SOURCES.toml`.
- **A4** — the coordinator runs the Linux e2e (`gh workflow run e2e.yml`) on the merge.

## Findings the spec and D-028 should absorb

1. **There is no DBP 78–99.** The page reserves `--imgdbp01`…`--imgdbp99` in CSS; the body holds
   **77** cards. DBP78 *"Ghost Riders of Christmas Past"* (RC1) exists only as an upload in the
   `dbp-wad-archive` item, with no card on the page — follow-up.
2. **D-028's Dropbox assumption is wrong in 2026** (per-request hashed redirect host, see above).
   Proposed wording: Dropbox links are only usable if the downloader learns a
   `*.dl.dropboxusercontent.com` suffix rule; until then a Dropbox-only release is not a usable
   source.
3. **`doomshack.org` is dead.** `http://doomshack.org/uploads/...` answers `302` with
   `Location: https://web.archive.org/web/20260103060906/https:/doomshack.org/...`, and the
   HTTPS endpoint does not connect. DBP41's bytes were pinned from the `dbp-wad-archive` copy
   (3,529,660 bytes, the same size as the Doomshack/Wayback copy), so `doomshack.org` is **not**
   in the allowlist: it is not used.
4. **The DSDA archive mirror is the best evidence of DSDA playability** for projects that are
   not on /idgames: its wad pages record demos with engine and complevel (e.g. DBP34 demos on
   `DSDA-Doom v0.28.3cl2`). Nine projects exist only there.
5. **Version drift between copies.** Each recipe pins exactly the copy it was verified against;
   the inventory notes the version in the filename where several exist (DBP12 v1.2, DBP23 v2,
   DBP40 RC1, DBP44 RC2-4, DBP46 RC2, DBP64 RC3, DBP65 v3, DBP68 RC1.4, DBP73 v3, DBP76 v1.1).
6. **Every DBP 01–77 is DSDA-playable**: all are Doom II limit-removing episodes except DBP33
   (Boom, cl9) and DBP42 (MBF21, cl21). None is GZDoom-only, UDMF or ZScript; DBP48 ships a
   GZDoom original *and* a limit-removing version and the catalog pins the limit-removing ZIP.

## New dependencies

None. The scratch helpers (thread parser, recipe generator, screenshot converter) lived in
`target/dbp/` and `scripts/tmp-*` and were removed before the final commit. `sips` and `ffmpeg`
were used only to inspect and resize screenshots.

## Open questions / left open

- Every recipe carries **Needs Deck Validation** for IWAD detection, complevel, load order,
  gameplay, audio and on-panel screenshot quality.
- The archive.org item serves at ~50 KB/s from this host, so the 24 recipes that depend on it
  install slowly. A Dropbox suffix rule (or a faster mirror) would fix that; it is a downloader
  change, outside this task's ownership.
- DBP78 and any later project need a follow-up inventory update.
- Proposed (coordinator-owned): note in D-028 that the archive.org `dbp-wad-archive` item is an
  accepted source for Doomer Boards releases that /idgames never carried, and that Dropbox
  shared links currently cannot be allowlisted.

## Final checks

<!-- filled in after the workspace checks -->

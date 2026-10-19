# 037 — Doomer Boards Projects in the catalog (worker report)

**Spec:** `docs/specs/037-doomer-boards-catalog.md` · **Decision:** D-028 (extended 2026-10-19)
**Branch:** `vittau/doomer-boards` · **Date:** 2026-10-19 · **Status:** in progress (see
"Dropped / left open")

## What changed

- `docs/research/doomer-boards-projects.md` (**R1**) — the inventory of the Project Catalog:
  **77 projects, DBP 01–77**, one row each with the release thread, release date, map count,
  IWAD, target port/format, the /idgames path where it exists, the other published downloads
  and the DSDA-Doom verdict with its evidence link, plus per-project notes and the labels
  Confirmed / Proposed / Needs Deck Validation.
- `catalog/<slug>.toml` (**R2, R5**) — one recipe per DBP, written in waves: **68 recipes**
  committed so far (DBP 01–36, 38–47, 49–52, 54–57, 60, 66, 67, 69); the 21 Dropbox-only
  projects are held (see "Dropped / left open"). Every recipe follows `auger-zenith`/spec 002:
  package + members with sizes and SHA-256 taken from a real download, load order, IWAD,
  complevel with its evidence, credits text kept, tags from `catalog/tags.toml`.
- `catalog/hosts.toml` (**R3**) — added `ia600903.us.archive.org` (the pinned download host of
  the archive.org item `dbp-wad-archive`, D-028 extension approved by the coordinator).
  `www.gamers.org`, `dsdarchive.com`, `youfailit.net` and `ftp.fu-berlin.de` were already there.
  **No new tag was needed**: `community-projects` already describes exactly this family.
- `crates/ddl/ui/assets/screenshots/<slug>/{01.jpg,SOURCES.toml}` (**R4**) — 67 bundled
  in-game screenshots, JPEG, quality 60, 640×360…1280×800, ≤ 192 KiB, each with source URL,
  source page, credit, original size, retrieval date and permission text.
- `crates/ddl-core/src/catalog/tests.rs` — the embedded-catalog test's id list (it enumerated
  the 13 original recipes) now names every shipped recipe id, sorted. This is the only Rust file
  touched, as allowed by the task's ownership note.

## How each acceptance criterion was verified

- **A1 (R1)** — the inventory lists every project the catalog page publishes. Verified by
  parsing the card list: `--imgdbpNN` variables for NN = 01…77, no gaps; the stylesheet's
  `--imgdbp01`…`--imgdbp99` is what makes the page *look* 99-wide (**Confirmed**). Each of the
  77 release threads was read from a Wayback snapshot for the release date, the port statement
  and the download links; each /idgames entry's readme was read for `Advanced engine needed`,
  `Tested With` and `Map #`.
- **A2 (R2, R3, R5)** — `ddl catalog lint --fetch <recipe>` was run for **every committed
  recipe** and passed (sizes, SHA-256 and named members verified from the real archives); the
  offline `ddl catalog lint` passes for the whole directory. `cargo test -p ddl-core` and
  `cargo test -p ddl --test catalog_cli` are green. A full-workspace `clippy`/`test` run was
  requested from the coordinator (build-slot rule).
- **A3 (R4)** — 67 of the 76 planned recipes have a screenshot; the nine without one are listed
  in the inventory (§ Screenshots) with the reason (the release thread only posts a title
  screen, a logo or a movie still): DBP08, DBP24, DBP31, DBP32, DBP41, DBP44, DBP55, DBP63,
  DBP69. Four are multi-view collages posted by the authors and four needed a Wayback copy of a
  dead image host; both are recorded per file in `SOURCES.toml`.
- **A4** — the coordinator runs the Linux e2e on the merge.

## Findings the spec and D-028 should absorb

1. **There is no DBP 78–99.** The page reserves `--imgdbp01`…`--imgdbp99` in CSS; the body
   holds **77** cards. DBP78 *"Ghost Riders of Christmas Past"* (RC1) exists only as an upload
   in the `dbp-wad-archive` item, with no card on the page — follow-up.
2. **D-028's Dropbox assumption is wrong in 2026.** `www.dropbox.com` does not redirect to
   `dl.dropboxusercontent.com`: it redirects to a **per-request** CDN host such as
   `uc6b74966005c2b22696e15c5150.dl.dropboxusercontent.com`, and three identical requests gave
   three different hashes. An exact-host allowlist cannot authorise a Dropbox install, so
   `lint --fetch` and a real install both fail. This is why 21 recipes are held; the ruling
   requested was (A) switch them to the archive.org item, (B) teach the downloader a
   `*.dl.dropboxusercontent.com` rule, (C) ship none of them.
3. **`doomshack.org` is dead.** `http://doomshack.org/uploads/...` answers `302` with
   `Location: https://web.archive.org/web/20260103060906/https:/doomshack.org/...` and the
   HTTPS endpoint does not connect. DBP41's bytes were therefore pinned from the
   `dbp-wad-archive` copy (3,529,660 bytes, identical size to the Doomshack/Wayback copy), so
   `doomshack.org` is **not** in the allowlist: it is not used.
4. **The DSDA archive mirror is the best evidence of DSDA playability** for projects that are
   not on /idgames: its wad pages record demos with engine and complevel (e.g. DBP34 demos on
   `DSDA-Doom v0.28.3cl2`). Nine projects exist only there.
5. **Version drift between copies.** The archive.org item is not byte-identical to the /idgames
   or Dropbox copies for several projects (DBP77: 6,361,417 vs 6,036,661 bytes), so each recipe
   pins exactly the copy that was downloaded and hashed.

## New dependencies

None. The recipes, screenshots and research use the repository's existing tooling; the scratch
helpers lived in `target/dbp/` and `scripts/tmp-*.py` and were removed before the final commit.

## Open questions / dropped work

- The 21 Dropbox-only projects (DBP32, 38, 48, 49, 53, 58, 59, 61–65, 68, 70–77) have no recipe
  until the redirect ruling lands; their inventory rows already say what is planned.
- `cargo clippy --workspace` / `cargo test --workspace` and `cargo fmt --all --check` results
  are recorded below once the build slot is granted.
- Every recipe still carries **Needs Deck Validation** for IWAD detection, complevel, load
  order, gameplay, audio and on-panel screenshot quality.
- Proposed wording changes (decisions are coordinator-owned): D-028 should say that Dropbox's
  redirect target is a per-request hashed CDN host, and — if option (A) is taken — that the
  archive.org `dbp-wad-archive` item is an accepted source for Doomer Boards releases that
  /idgames never carried.

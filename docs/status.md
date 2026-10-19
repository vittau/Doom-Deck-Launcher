# Status

**Milestone:** M3 — `main` has 13 mapsets with tags/favorites, multiple profiles, widescreen status bars, GOG IWAD discovery, recipe updates after install, the Stained Glass logo (v3, D-029), real mapset screenshots, catalog updates without a release, a download idle timeout and local content (any WAD/ZIP/DEH, player-set load order, extras on catalog mapsets); latest pre-release is `v0.0.6-local`; first Deck test done on `v0.0.6-local` (2026-10-19): fixes in specs 028–032  ·  **Updated:** 2026-10-19

| Item | Spec / output | State |
| --- | --- | --- |
| Repo foundation (license, AGENTS.md, docs) | `docs/` | Done |
| Workspace, CI, release pipeline | `specs/001-workspace-ci-release.md` | Done — CI green; `v0.0.1-skeleton` (S1–S6); **`v0.0.2-playable`** pre-release (tarball 13.96 MB) for Deck checks F1–F8 |
| DSDA-Doom v0.30.x CLI, config, pistol start, strict mode (desk research) | `research/dsda-engine.md` | Done |
| Upstream sources: DSDA releases/AppImage, `/idgames` API, AUGER;ZENITH | `research/upstream-sources.md` | Done |
| Reference mapsets SIGIL / SIGIL II | `research/reference-sigil.md` | Done |
| Prior art: UZDeck / GZDeck (incl. UI critique) | `research/prior-art.md` | Done |
| Recipe schema, catalog, lint, reference recipes | `specs/002-recipe-catalog.md` | Done — 3 reference recipes lint clean with real fetch |
| Launch plan and engine process runner | `specs/003-launch-plan.md` | Done (A6 needs Deck) |
| Engine manager (discover, download, verify, install, rollback) | `specs/004-engine-manager.md` | Done (A5 needs Deck) |
| Mapset install, profiles, launch resolution (core) | `specs/009-install-resolve.md` | Done |
| Downloads and safe ZIP extraction (nested) | `specs/006-downloads-extraction.md` | Done |
| DsdaConfigManager v1 | `specs/005-dsda-config-manager.md` | Done (A6 needs Deck) |
| UI contract v1 + wireframes (Codex) | `design/ui.md`, `design/mockups/` | Done |
| UI visual pass: hi-fi generated mockups, identity (Codex) | `design/visual/` | Done — note: required components must render as locked, not as a toggle |
| IWAD identification (hashes, lumps, Steam paths) | `research/iwad-identification.md` | Done |
| IWAD table, identifier, Steam discovery, canonical links | `specs/007-iwad-library.md` | Done (A6 needs Deck) |
| Eviternity recipe (owner request) | `catalog/eviternity.toml`, `research/reference-eviternity.md` | Done — complevel 11 Proposed; needs Deck |
| Wire UI to core: first run, install, play, settings | `specs/010-ui-wiring.md` | Done (A3 needs Deck) |
| Visual identity v2 (DOOM-flavoured, D-020) + production polish | `specs/012-visual-identity-v2.md` | Done (A4 needs Deck) — Ember Cathedral in the production UI |
| Project logo, app icon and Steam artwork | `specs/013-logo-and-icons.md` | Done (A4 needs Deck) — logo replaced by Rose Compass in spec 016 (D-023); social preview v2 "Rose Window" uploaded to GitHub (2026-10-17) |
| Headless CLI + real-network Linux e2e job | `specs/014-cli-and-linux-e2e.md` | Done — e2e green on real network (run 38983844913): AppImage direct + extraction, 4 mapsets, D-011 argv |
| Catalog batch: Scythe, Alien Vendetta, Sunlust, Ancient Aliens, Valiant, Going Down, BTSX E1/E2, Eviternity II | `catalog/*.toml` | **Done** — 13 recipes on `main`; catalog/member cap raised to 512 MiB for Eviternity II; e2e run 38987876930 |
| Recipe tags, Discover categories, favorites | `specs/015-tags-categories-favorites.md` | **Done** (A4 needs Deck): 8 tags in `catalog/tags.toml`, all 13 recipes tagged (Cacowards evidence secondary — doomworld blocks automation), Discover strip with L2/R2, X favorites, app state schema 2 |
| README revamp, Steam Deck guide, logo v2 | `specs/016-readme-and-logo-v2.md` | **Done**: README 66 lines with screenshots, `docs/steam-deck-guide.md`, **Rose Compass** logo (D-023) in app header, loading, About, window icon, Steam art, README and social preview (uploaded 2026-10-17) |
| README manual Steam Input guide | `specs/011-readme-steam-input.md` | Done (A3 needs Deck) |
| AppImage on Deck (FUSE vs extract) | `deck-testing.md`, `research/dsda-engine.md` §7 | Linux host: direct FUSE works (CI); SteamOS 3.8 ships every engine library (glibc 2.41, pipewire-jack); must not run under a Steam Compatibility tool. Deck check pending |
| UI shell: design system, navigation, gilrs gamepad, screens on fixtures | `specs/008-ui-shell.md` | Done (A6 needs Deck) |
| Steam Input handoff, gyro-as-mouse, strict-mode demos | — | Needs Deck Validation |

## Backlog — next major release (owner said go 2026-10-16; specs written)

| Item | Spec | State |
| --- | --- | --- |
| Widescreen status bars (doom_widehud, GPL-3.0; credited in About/README/recipes) | 017 | **Done** (A4 e2e run 39065323300 passed, A6 needs Deck): 5 mapset bars (Ancient Aliens, BTSX E1/E2, Eviternity, Valiant) + Doom/Doom II IWAD bar for SIGIL, SIGIL II, Going Down, Scythe; default on; cosmetic-only and `release` lint. Residual risk: the Eviternity bar derives from CC BY-NC-ND art (DDL only downloads it from upstream; research §4) |
| Multiple profiles per mapset (`deck-testing.md` C4) | 018 | **Done** (A5 needs Deck): picker on Play setup, new (copy/defaults), preset-name rename, guarded delete, remembered profile, Home/Library labels, `--profile` CLI; app state stays schema 2 |
| Catalog updates without a DDL release | 019 | **Done** (D-025): CI publishes `catalog.json` to the `catalog` branch; DDL checks every 24 h (opt-out in Settings) and on demand, validates against the embedded host allowlist, caches atomically; `ddl catalog status/update/bundle`. Deck: S2–S3 |
| Recipe changes after install | 020 | **Done**: typed change classification, silent metadata adoption, staged generations published by an atomic `library/<id>/active.json` rename, idempotent profile reconciliation, Update / Keep current (per recipe hash), removed-from-catalog badge, `ddl mapset status|update` |
| Uninstall a mapset (owner decision D-022: checkbox to also delete profiles and saves) | 022 | **Done** (A5 needs Deck): Uninstall on Mapset detail, confirm with "Also delete profiles and saves" (unchecked), crash-safe and path-contained, `ddl mapset uninstall <id> [--delete-profiles]`; also fixed a pre-existing duplicate "Play setup" button on detail |
| Real mapset screenshots on cards (owner request 2026-10-17; D-024: bundled with source and credit) | 023 | **Done**: 23 bundled in-game shots for all 13 recipes with SOURCES.toml, `[[screenshots]]` field + lint; cards, Home, Library, detail and art band show them with a compact "Screenshot: <credit>" caption; painted art only as fallback. Deck: S1 |
| GOG and other store IWAD paths | 021 | **Done** (A5 needs Deck): Heroic (native + Flatpak), minigalaxy, GOG offline default; opt-in standard Linux folders. Lutris stays manual (would need SQLite). GOG/id Anthology Final Doom hashes added to `data/iwads.toml`. |
| Download idle timeout (60 s without data → clear error) + fix the flaky engine-download test | 024 | **Done** (A2 partial): stalled downloads fail after 60 s without data with a "stopped receiving data" message (ureq per-read cap in the transport), cancel still wins; the flaky engine fixture raced the real 15 s header timeout and now uses a test-only timeout. A2 stress loop 6/20 passes (rest left open; too slow until the journey is split) |
| Long mapset titles fit their slots (found in 023 review) | 025 | **Done**: titles wrap → shrink one step → elide (full title on detail) in Home, Recently played, Discover, Library, Play setup, install progress and dialogs; component labels too (approved scope addition); fit test covers all 13 recipes plus a synthetic 60-character title and label; README renders refreshed |
| Local content: arbitrary user WADs/patches, player-set load order, extras on top of catalog mapsets (D-026) | 026 | **Done** (A5 needs Deck): core + CLI (WAD/ZIP/DEH/BEX by content, `.pk3` rejected per DSDA source, SHA-256 identity check, `ddl custom …`, `ddl profile extras …`) and UI (controller file picker with multi-select and SD-card shortcuts, grouped order editor with pick-up/move/drop, New custom setup in Library, custom setups in Home, Extra files on Play setup with locked recipe files, IWAD/compatibility pickers, missing/changed-file and delete dialogs). Also fixed a real state-lock race: `StateLock` now unlocks on drop (a spawned child briefly held the lock between fork and exec). Deck: L1–L3 |
| Lutris IWAD discovery | — | Deferred (owner, 2026-10-17: "não precisa ainda") |
| Split the headless production journey into smaller tests (owner, 2026-10-17: "esse teste tá muito longo") | 027 | **Done**: the 355 s journey is now 25 isolated, named headless tests (subprocess per test, own XDG root and artifact dir; worst 37.5 s); full workspace ~2 min warm, 3 parallel runs green; "Running UI tests" note in the guide |

## Deck test 2026-10-19 fixes (`v0.0.6-local`, results in `deck-testing.md`)

| Item | Spec | State |
| --- | --- | --- |
| Steam scan misses Doom II/TNT/Plutonia: app 2280 installs to `Ultimate Doom/` with `base/doom2/`, `base/tnt/`, `base/plutonia/`, `rerelease/` | 028 | **Done**: `Ultimate Doom/base/{,doom2,tnt,plutonia}` and `rerelease/` probed flat (no recursion), KEX resource WADs filtered by name, classic `base/` preferred; research §4 corrected. Deck: F2 |
| Gaming Mode renders at ~2× (DPI); DSDA opens 4:3 — fit-to-screen scale and Deck video defaults | 029 | **Done**: fullscreen scale = min(w/1280, h/800) from the physical size, re-applied if the backend re-reports the DPI, `SLINT_SCALE_FACTOR` still wins, values logged; DSDA template v2 (1280×800, desktop fullscreen, `render_aspect 2` 16:10, `render_stretch_hud 2`) with three-way migration; research §9. Deck: S3 without launch option, F4 widescreen |
| Tab bar disabled where it can't be used, first-run emphasis, button hints, header, Discover margin, L2/R2 hold+wrap, toasts fade | 030 | **Done**: tabs greyed and unfocusable off root screens and in dialogs, red pulsing Install on first run, per-screen hint footer, header text removed, 22 px Discover gap, L2/R2 500 ms hold-repeat + wrap, 3 s toasts with 300 ms fade cleared on tab change; README renders refreshed. Deck: S4, S2 |
| Profiles: summary on detail, free-text names with Steam keyboard, wrap and L1/R1 switching | 031 | **Done**: profile summary + Edit on Mapset detail, one profile list (A selects, L1/R1 wrap), free-text names (1–32 chars, case-insensitive unique), Steam keyboard via `steam steam://open/keyboard` with a Steam + X hint, preset-name code removed. Deck: C4 naming/keyboard |
| Play base games without a mapset (D-027); IWAD folders lists automatic sources | 032 | **Done**: Library "Base games" group (Ultimate Doom, Doom II, TNT, Plutonia; NERVE excluded as a PWAD), base-game page + Play setup with profiles/saves/extras under `base:<game>` ids, vanilla complevels 3/2/4/4, classic copy preferred and switchable (`iwad_path`), Home history, IWAD folders lists automatic sources without Remove, `ddl doctor`/`ddl launch base:<game>`. Finished by a Sonnet worker after Codex hit its limit. Deck: play Ultimate Doom and Doom II from Library |
| Drop decorative subtitles and the screenshot caption plates (credits move to detail text and About) | 033 | **Done**: headers are the title alone (audit of every subtitle in the report; state/next-step lines kept and shortened), tone rule in `docs/design/ui.md`, no caption plates on images, credits as plain text on Mapset detail and About, base-game pages without repeated "Original game" and with `~` paths |
| Owner's DSDA controller layout (Select map, D-pad zoom/weapons, A use, B autorun, L1/R1 quick save/load, X quick turn) | 034 | **Done**: template v3 binds the owner's layout (View map, D-pad ↑/↓ automap zoom, ←/→ weapons, B autorun, L1/R1 quick save/load, X `input_reverse` 180°; A use and B jump stay DSDA defaults), migration keeps player rebinds; research §4. Deck: E5 |
| Bundled images as JPEG ≤ 1280×800 at low quality, lint-enforced for community screenshots; UI art recompressed | 035 | **Done**: lint requires JPEG 640×360–1280×800, ≤ 192 KiB (q60); UI art without alpha → JPEG q60, 4 unused assets removed, ornaments/frame stay lossless PNG; assets 8.3 → 2.8 MB (embedded 5.3 → 2.8 MB); `scripts/lean-image.sh` |
| Logo v3: richer Rose Compass, SVG only, more detail and colour (owner routing: Opus 5.5 high) | 036 | **Done**: three directions, owner picked B Stained Glass after a second round (lead came, amber + red glass, jewels, smaller ember); small tier for 32–64 px; reproducible exporter with `--check`; D-029 |
| Every Doomer Boards Project (DBP 01–99) that DSDA-Doom can play, /idgames first, Dropbox fallback (D-028) | 037 | **Done**: the page lists 77 DBPs (01–77; 78–99 are only reserved CSS slots), all DSDA-playable; 76 new recipes (DBP37 was `auger-zenith`) — 43 from /idgames, 9 from dsdarchive, 24 from the archive.org `dbp-wad-archive` item (`ia600903.us.archive.org`); 67 screenshots; `lint --fetch` green for every recipe; catalog now 89 recipes. doomshack.org is dead (not allowlisted). Dropbox mirrors follow in 038 |
| Dropbox as a fast first mirror (narrow `*.dl.dropboxusercontent.com` redirect rule) for the 21 DBPs now on slow archive.org | 038 | Phase 1 **merged** (redirect rule: one-label host, only in a chain entering at `www.dropbox.com`, same-host second hop); phase 2 (mirrors) after 037 |
| Resumable downloads: keep pinned partials, `Range`/`If-Range` resume, 3 retries, bounded partial cache | 040 | Ready |
| Research: Cacowards inventory with /idgames availability and DSDA verdicts | 042 | Ready |
| Research: optional Steam shortcuts for installed mapsets (feasibility) | 043 | Ready |
| Weekly catalog link health (`ddl catalog check-links`, headers only, issue on failure) | 044 | Ready (after 040) |
| UI touch-ups: seamless wall background, no "Verification pending" / process labels in the player UI | 045 | In progress |
| Complevels as DSDA aliases (`vanilla`/`boom`/`mbf`/`mbf21`), no complevel verification status (D-030) | 046 | Ready (after 037 and 045) |
| Transparent README logo (light and dark variants via `<picture>`) | 047 | In progress |
| Vector panel frames and ornaments (corners meet their rails at any size) | 048 | Ready (after 045) |
| Fade from the title panel into the art band | 049 | In progress |
| Research: /idgames browse | 041 | **Done**: mirror `ls-laR.gz` (43,741 files, daily) + per-file txt; recommends a CI-built index; specs 050–053 written from its proposal; owner questions open (publish a derived index; courtesy note to the archive maintainers) |
| /idgames index build and publish | 050 | Ready (first publish waits for the owner) |
| /idgames browse core (fetch, cache, search) | 051 | Ready (after 050) |
| /idgames unverified install and launch | 052 | Ready (after 051) |
| /idgames browse UI and recipe-draft hand-off | 053 | Ready (after 052) |
| Apply logo v3 (D-029) in production: masters, header, loading, About, window icon, Steam art, social preview | 039 | **Done**: masters authored from `logo-v3/author.py` (v2 kept in `logo-v2/production-d023/`), every PNG regenerated with resvg 0.47.0 (`export.py --check`), small tier for the header and window icon, README logo and screenshots refreshed; social preview regenerated — **owner uploads it to GitHub by hand**. Deck: logo check |

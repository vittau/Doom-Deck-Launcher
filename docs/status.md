# Status

## Current checkpoint — 2026-10-28

**Available for Deck testing:** [v0.0.19-menu](https://github.com/vittau/Doom-Deck-Launcher/releases/tag/v0.0.19-menu),
immutable source `84ca7b533695e609df5c2b615f3858ff65b94a83`, published as a Linux tarball
and signed Flatpak at [doomdecklauncher.com](https://doomdecklauncher.com/).
Release 40735115609, signed Flatpak 40735115503 (site version 0.0.19) and CI 40733494293
succeeded on the tag source; the tarball matches its published SHA-256. Every game now opens
at DSDA-Doom's title and menu: DDL has no skill setting and passes neither `-skill` nor
`-warp` (114, D-044); pistol start is unchanged. Deck checks MM1–MM4 are pending.

**Per-mapset progress (116, D-045):** the owner approved the scope from report 115 —
completed maps need a recorded skill, total = DSDA's detected PWAD maps minus an optional recipe
`progress.exclude_maps`, a simple per-map table on Mapset detail without times, per selected
profile. Spec 116 is ready for implementation.

### Shipped

- **Install paths:** the per-release install folder is now a bounded slug of the release label
  (`library/headless-chicken/RC1-2023-12-09/free/HDLSCHKN.wad`) instead of the whole citation,
  so paths no longer carry spaces, commas or parentheses. Existing installs are not migrated
  (D-042): reinstall a mapset to adopt the short folder. The short path does not fix the
  owner's Headless Chicken crash, which is an engine defect (110).
- **Local content:** choosing a folder lists files from bounded inspections instead of hashing
  every byte; selected files are identified only on confirmation and that capture can be
  cancelled. The picker remembers the last usable folder, a missing file can be located in its
  existing slot or profile (silent repair for equal bytes, explicit confirmation otherwise),
  and new custom setups take the first content filename and can be renamed without touching
  profiles or saves. Physical feel, Flatpak folder visibility and SD-card behavior are
  Needs Deck Validation (LC1–LC7).
- **Catalog controls:** Show recipe variants replaces the old Advanced row. Mapsets without free
  recipe alternatives keep a visible, greyed-out row that focus skips; where alternatives exist
  the first one is revealed and focused, and replacing a selection still asks for confirmation.
  Catalog compatibility reads Set by recipe, while custom setups keep Auto/numeric choices.
  Contrast and scroll behavior are Needs Deck Validation (RV1–RV3).
- **Browsing:** shared persistent Cards/List choice on Select/V (095), continuous List
  scrolling and favorite-first installed mapsets in Library (102). Discover retains its
  category filters and chronological ordering. Actual controller feel remains Needs Deck Validation.
- **Catalog:** 118 recipes, including the player-supplied No Rest for the Living expansion.
  The published app bundles screenshots for 114 recipes; the remaining four lacked a verified
  usable image. Downloads, user-supplied components, profiles, base games and local custom
  setups are implemented.
- **Updates:** catalogs needing a newer client now ask for a DDL update rather than a connection
  retry (098). Flatpak AppStream/display versions and binary version are synchronized (099/100).
- **Upgrade guards:** real tagged-serializer state fixtures cover releases through
  `v0.0.15-local`, including audio, Cards/List and the optional local-picker preferences.
  This is automated evidence, not a substitute for upgrading the owner's actual Deck setup.
- **Website:** canonical domain and signed Flatpak identity are D-039. The owner confirmed
  Analytics reception of `donate_click` and `install_click`; D-040 limits them to intent.
  Owner reports Google search visibility; the live favicon is a reachable 256×256 PNG with
  an explicit home-page icon link and no observed robots block. Search-result icon display
  has not been confirmed; no favicon change is planned from that observation alone.

### After the published app tag

The published tag `9745d53` is verified by [Linux CI 39957952528](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39957952528):
fmt, denied-warning Clippy, ordinary offline tests and catalog lint succeeded. The ordinary
suite excludes the ten exhaustive render cases specified in 101; it is not a full-suite result.
The catalog publication job also succeeded without a catalog change. The daily
[/idgames index run 39931095281](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39931095281)
succeeded on 2026-10-23; the app archive browser remains unimplemented and deprioritized.

- **Recipe screenshots:** the published app bundles **114 of 118**. Missing:
  japanese-valentines, mindblood-genesis, spaceballs-the-quest-for-more-dbp-maps and
  tomatomania. No verified usable image was found for those four; none was generated or
  substituted from another mapset.
- **Headless Chicken:** **known broken on Linux with DSDA-Doom 0.30.1**, at any complevel and
  install path. The engine crashes while precaching the mapset's stereo Ogg sound effects:
  the official AppImage bundles SDL 2.0.20, whose SSE3 stereo-to-mono conversion makes an
  aligned load from DSDA's 8-byte-aligned buffer. `-nosound` is the only workaround and DDL
  does not expose it. Upstream: issue [dsda-doom#1049](https://github.com/dsda-org/dsda-doom/issues/1049),
  fix [dsda-doom#1050](https://github.com/dsda-org/dsda-doom/pull/1050) (opened by the owner
  2026-10-23, CI green, awaiting review). When a release contains it, add that engine to
  `engines.toml` and re-test the mapset (110).
- **Only fixture and documentation commits follow the tag.** The post-tag history adds the
  `v0.0.15-local` saved-state fixture, its guard and this status; it does not imply a newer
  binary than the published one.

- **Pinned Discover categories (111, published in 0.0.17):** the category strip stays fixed
  above the scrolling content in Cards and List; the focused item is always fully visible below
  it and switching category returns to the top. Owner-chosen compact Discover cards: 340px
  tall with 96px art and one row of smaller tag chips; Library cards unchanged. Sixteen focused
  tests, fmt and app Clippy passed; real-catalog off-screen renders reviewed. The owner
  validated PD1–PD3 on the Deck with 0.0.17 (strip fixed, navigation/touch, 14px tags readable).

- **Discover installed and favorites first (112, published in 0.0.18; D-043):** every
  Discover category, in Cards and List, lists installed favorites, installed mapsets,
  favorite-only mapsets, then the rest, chronological inside each group. Favorite toggles and
  installs move the item with focus kept; uninstall keeps its spec 022 navigation. Ordinary
  suite (1,008 tests), the category/favorites journey and CI 40030510101 passed. Spec 113
  (also in 0.0.18) fixes focus after unfavoriting inside the Favorites category. The owner validated DF1–DF2 on the Deck (2026-10-26).

### Immediate direction

The owner validated 0.0.18 on the Deck (DF1–DF2) and a fresh `.flatpakref` install, and
reports no further compatibility problem found. Keep outstanding hardware checks honest; do
not treat every historical pending checkbox as an observed defect.

**Outreach on hold (2026-10-26):** the owner posted on r/DoomMods and in Doomworld's Source
Ports subforum; r/DoomMods removed the post and Doomworld warned the account for "AI slop"
(posting restricted). Channel order, rules and post links live in [outreach.md](outreach.md);
the next channel is the owner's call. Prioritize the install/update → find owned games → install → play →
return journey, including retained data, physical input and Flatpak audio/file access.

**D-041:** `/idgames` browsing (051–053) is deferred without priority. The curated Discover
catalog is sufficient for the current direction; occasional other content uses the existing
manual custom-setup/extra-files path. Do not start archive-browser work by default.

The source-based UX study is [report 105](reports/105-curated-browsing-and-local-content-review.md).
The owner authorized the four manual-content improvements on 2026-10-23. Implemented and reviewed locally under [spec106](specs/106-local-content-polish.md):
lightweight folder listing without full-file hashes or a full snapshot refresh, durable last
folder, explicit slot/profile-preserving Locate file and recognizable/renameable custom
setups. Launch-time identity verification remains. Merge `95a55bb` matches checked production
source; fmt, denied-warning workspace Clippy and 995 ordinary offline tests passed with the
ten documented visual exclusions. The isolated synthetic host diagnostic fell from
1047–1122 ms to 0.16–0.25 ms; it is not complete UI latency or a Deck/SD result. See report106
and pending LC1–LC7 Deck checks.

The owner reported that the catalog Advanced row appeared active but did nothing for tested
mapsets. [Spec107](specs/107-catalog-variant-clarity.md) was reviewed and merged as
`b537833` and is part of the published 0.0.15 app. The row stays visible but greyed out, non-actionable and skipped by focus when
no recipe variants exist. Available Show/Hide recipe variants reveals and focuses the first
alternative; selecting one retains player confirmation with the safe action focused. This
also fixes alternatives previously appearing above the visible viewport. Catalog complevel
copy now says Set by recipe; custom setups retain Auto/numeric Compatibility. Eight focused
follow-up tests, fmt and denied-warning app Clippy passed; five headless states were reviewed.
The earlier 995-test ordinary suite was not repeated for this scoped presentation follow-up.

Both local-content and recipe-variant changes are published in `v0.0.15-local`, whose
publication, provenance and checksum are recorded in [report108](reports/108-publish-local-content-update.md).
No physical Deck result is claimed; LC1–LC7 and RV1–RV3 remain pending in
[the Deck checklist](deck-testing.md). Browsing filters/search remain without priority.
Source/check evidence lives in reports106/107.

## Historical milestone tables

The tables below retain earlier checkpoints and their original results. For the current
release, source delta and priority, use the checkpoint above and individual release reports.

**Milestone:** M3 — `main` has 13 mapsets with tags/favorites, multiple profiles, widescreen status bars, GOG IWAD discovery, recipe updates after install, the Stained Glass logo (v3, D-029), real mapset screenshots, catalog updates without a release, a download idle timeout and local content (any WAD/ZIP/DEH, player-set load order, extras on catalog mapsets); latest pre-release is `v0.0.9-focus`; first Deck test done on `v0.0.6-local` (2026-10-19): fixes in specs 028–032  ·  **Updated:** 2026-10-20

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
| Eviternity recipe (owner request) | `catalog/eviternity.toml`, `research/reference-eviternity.md` | Done — complevel `mbf` (11); needs Deck |
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
| Dropbox as a fast first mirror (narrow `*.dl.dropboxusercontent.com` redirect rule) for the 21 DBPs now on slow archive.org | 038 | **Done**: phase 1 redirect rule (one-label host, only in a chain entering at `www.dropbox.com`, same-host second hop); phase 2 puts Dropbox first in the 12 DBPs whose `?dl=1` file matches the pinned SHA-256 (2.34 MiB/s vs 0.023 MiB/s on archive.org), the 9 that differ stay archive.org-only; `lint --fetch` green. Linux e2e pending |
| Resumable downloads: keep pinned partials, `Range`/`If-Range` resume, 3 retries, bounded partial cache | 040 | **Done**: `<sha>.part` + JSON sidecar, 206 appends, 200/416/bad `Content-Range` restart once, retries after 2/5/10 s, cache bounded to 7 days / 2 GiB, "Resuming…" row |
| Research: Cacowards inventory with /idgames availability and DSDA verdicts | 042 | **Done**: 2004–2025 (no editions before 2004), 391 Top Ten/runner-up entries + 69 special-award mapsets; 180 playable, 60 not, 151 unknown; 14 proposed recipe waves (135 /idgames candidates, all on `www.gamers.org`); gap: `going-down` lacks the `cacowards` tag |
| Research: optional Steam shortcuts for installed mapsets (feasibility) + SteamGridDB | 043 | **Done**: README-only (no shortcut writing: Steam must be closed, unsupported format); SteamGridDB has no aliases, so the page and the shortcut name are `DoomDeckLauncher`; owner requests the page and uploads the existing art |
| Weekly catalog link health (`ddl catalog check-links`, headers only, issue on failure) | 044 | **Done**: header-only probes under the download allowlist/redirect rules, 4 in flight / 1 per host, retries (raw.githubusercontent.com answers ~50 % 503), weekly `catalog-health.yml` opens/closes one issue; `ddl catalog e2e-sample` cuts the e2e installs to 19 of 16 recipes; real run 222/222 URLs ok |
| UI touch-ups: seamless wall background, no "Verification pending" / process labels in the player UI | 045 | **Done**: wall crop inside the atlas border (regenerated from the PNG atlas), Home readiness row and every hard-coded process label removed (audit in the report), headless tests fixed for the 89-recipe catalog (main was red after 037) |
| Complevels as DSDA aliases (`vanilla`/`boom`/`mbf`/`mbf21`), no complevel verification status (D-030) | 046 | **Done**: 89 recipes rewritten to aliases, `complevel_status` removed (lint rejects it and numbers an alias names), argv passes the word, UI shows "Vanilla (2)"; old clients (v0.0.6) drop every new recipe and keep their embedded catalog, verified against the real v0.0.6 code. Follow-up: aliases in the custom-setup complevel override |
| Transparent README logo (light and dark variants via `<picture>`) | 047 | **Done**: `readme-logo-{dark,light}` transparent, name contrast 15.97:1 / 19.37:1, `<picture>` in README |
| Vector panel frames and ornaments (corners meet their rails at any size) | 048 | **Done**: script-authored SVG corners, rails and ornaments (`ember/author_frame.py`), seams exact at 1x and 1.35x, owner-approved renders; test fixtures no longer reach the public catalog (`without_catalog_auto_updates`). Deck: frame check |
| Fade from the title panel into the art band | 049 | **Done**: 120 px gradient from the panel colour, headless regression measures the joint |
| Steam artwork v2: detailed vector scenes, no plate behind the lettering | 054 | **Done**: procedural Ember Cathedral interiors (`docs/design/logo/scene.py`) for both capsules and the hero, owner-approved; `logo.png` unchanged (bottom-left); all five pieces uploaded to SteamGridDB (`/game/5558703`, pending site verification) |
| Research: /idgames browse | 041 | **Done**: mirror `ls-laR.gz` (43,741 files, daily) + per-file txt; recommends a CI-built index; specs 050–053 written from its proposal; owner questions open (publish a derived index; courtesy note to the archive maintainers) |
| /idgames index build and publish | 050 | **Done**: `ddl idgames index build/validate`, daily slow crawl (1 request at a time, ≥ 3 s, 1,500/run, back-off), publish to `idgames-index`; first index published 2026-10-20: 10,885 entries, 1.0 MB, 1,500 requests all 200 (min gap 3.0 s); the first try failed on a control character in a sidecar title (sanitized now) and a suffix-matching `ls-remote` (fixed for `catalog` too) |
| /idgames browse core (fetch, cache, search) | 051 | Ready (after 050) |
| /idgames unverified install and launch | 052 | Ready (after 051) |
| /idgames browse UI and recipe-draft hand-off | 053 | Ready (after 052) |
| Apply logo v3 (D-029) in production: masters, header, loading, About, window icon, Steam art, social preview | 039 | **Done**: masters authored from `logo-v3/author.py` (v2 kept in `logo-v2/production-d023/`), every PNG regenerated with resvg 0.47.0 (`export.py --check`), small tier for the header and window icon, README logo and screenshots refreshed; social preview regenerated — **owner uploads it to GitHub by hand**. Deck: logo check |

## Deck test 2026-10-20 fixes (`v0.0.7-deck`)

| Item | Spec | State |
| --- | --- | --- |
| Upgrade over 0.0.6 lost the saved setup (`app.json` recipe copies vs spec 046); recovery screen; Settings never empty | 055 | **Done**: v0.0.5/0.0.6/0.0.7 state trees made by each tag's own serializers load with nothing lost (release guard test), legacy recipe copies decoded without loosening catalog parsing, recovery screen (Retry / Start fresh keeps `*.unreadable-<time>` / Diagnostics), Settings panels never empty. Codex, finished by Sonnet. Deck: U1–U2 |
| DSDA template v4: VSync on with uncapped FPS, look sensitivity 5/5; gyro settings in the Deck guide (D-033) | 056 | **Done**: template v4 (`render_vsync 1`, `uncapped_framerate 1`, no FPS cap; `mouse_sensitivity_horiz/vert 5`), v3→v4 three-way migration, cited in research §10; gyro values in the Deck guide. Deck: U3–U4 |
| Right-stick scroll, Play setup D-pad order, skill 4 "(UV)" default, tabs Home/Library/Discover, B at root → Quit (D-032) | 057 | **Done**: right-stick scroll on every scrollable view (incl. Recovery), Play setup focus order, skill labels/default, tab order, B → Quit at roots. Codex, finished by Sonnet. Deck: U5–U9. Pre-existing nit: the Advanced card's subtitle touches its bottom edge |
| Research: Flatpak on Flathub/Discover and running the engine from the sandbox | 058 | **Done** (`research/flatpak.md`): extract-and-run the AppImage in the sandbox (no FUSE), runtime 26.08; Flathub's AI policy forbids AI-assisted manifests and agent-run submissions — channel choice waits for the owner |
| Flatpak on Flathub (D-034): app sandbox-ready — extract-first engine, paths, visibility messages, import of a tarball-era setup, portal keyboard, no self-update; the owner writes manifest/metainfo/desktop entry and submits by hand | 059 | **Done**: sandbox probe (`/.flatpak-info`), extract-first engine in the sandbox, Flatpak XDG paths, one-line "not visible to the Flatpak" notices, one-time import of a tarball setup (Import focused), `xdg-open steam://open/keyboard`, About notes Discover updates; no packaging files (D-034). The owner writes the manifest next. Deck: after the owner's first Flatpak build |
| Focus clarity: rows entered at their leftmost item (Advanced → Skill), amber only for focus, primary actions (Play, Install) in ember red, a distinct selected mark | 060 | **Done**: rows entered at the leftmost item, one amber focus ring, primary actions ember red (6.2:1), selected chips with a green edge, recipe-locked rows dimmed with a lock, unavailable actions in the disabled style. Deck: V1–V3 |
| Library tab broken in v0.0.8 (stray frames under the header, one-card row) — regression | 061 (brief; report `reports/061-library-regression.md`) | **Done**: the gamepad loop's per-tick `invoke_scroll(0,0)` no longer overwrites the offset; focus-follow aligns a section heading (or the item) to the top so no partial frames sit under the header (the rails already showed in 0.0.7); Library base-game focus no longer resets. Deck: V4–V5 |

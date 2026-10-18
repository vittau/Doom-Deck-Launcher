# Status

**Milestone:** M3 — `main` has 13 mapsets with tags/favorites, multiple profiles, widescreen status bars, GOG IWAD discovery, recipe updates after install, the Rose Compass logo, real mapset screenshots and catalog updates without a release; latest pre-release is `v0.0.5-screens.1`; Deck validation pending  ·  **Updated:** 2026-10-17

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
| Long mapset titles fit their slots (found in 023 review) | 025 | Finishing (2026-10-18, Codex): final checks, README PNGs and report on the existing branch; snapshot `refs/keep/spec-025-long-titles` |
| Local content: arbitrary user WADs/patches, player-set load order, extras on top of catalog mapsets (D-026) | 026 | **Phase A merged** (core + CLI: WAD/ZIP/DEH/BEX by content, `.pk3` rejected per DSDA source, custom setups, profile extras, identity check, `ddl custom …`, `ddl profile extras …`; API in `docs/reports/026-local-content.md`). phase B (R8–R11: file picker, order editor, entry points, badges) in progress (Codex, 2026-10-18) |
| Lutris IWAD discovery | — | Deferred (owner, 2026-10-17: "não precisa ainda") |
| Split the headless production journey into smaller tests (owner, 2026-10-17: "esse teste tá muito longo") | — | Backlog: the single `fixture_screens_navigation_and_focus_render_headlessly` journey takes ~2–5 min, longer under load; split it into independent per-screen/per-flow tests that can run in parallel |

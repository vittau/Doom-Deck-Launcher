# Status

**Milestone:** M2 — `v0.0.4-cli` pre-release (Ember UI, logo, 4 mapsets, Konsole CLI, engine diagnosis); Linux e2e green; awaiting Deck validation  ·  **Updated:** 2026-10-16

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
| Project logo, app icon and Steam artwork | `specs/013-logo-and-icons.md` | Done (A4 needs Deck) — Ember Gate SVG logo; owner uploads `docs/design/logo/social-preview.png` in GitHub settings |
| Headless CLI + real-network Linux e2e job | `specs/014-cli-and-linux-e2e.md` | Done — e2e green on real network (run 38983844913): AppImage direct + extraction, 4 mapsets, D-011 argv |
| Catalog batch: Scythe, Alien Vendetta, Sunlust, Ancient Aliens, Valiant, Going Down, BTSX E1/E2, Eviternity II | `catalog/*.toml` | **Done** — 13 recipes on `main`; catalog/member cap raised to 512 MiB for Eviternity II; e2e run 38987876930 |
| Recipe tags, Discover categories, favorites | `specs/015-tags-categories-favorites.md` | **Done** (A4 needs Deck): 8 tags in `catalog/tags.toml`, all 13 recipes tagged (Cacowards evidence secondary — doomworld blocks automation), Discover strip with L2/R2, X favorites, app state schema 2 |
| README revamp, Steam Deck guide, logo v2 | `specs/016-readme-and-logo-v2.md` | Phase A **done** (README 66 lines, `docs/steam-deck-guide.md`, reproducible screenshots, 3 logo concepts in `docs/design/logo-v2/`); owner picked **Rose Compass** (D-023); phase B (R2) applying it (Codex) |
| README manual Steam Input guide | `specs/011-readme-steam-input.md` | Done (A3 needs Deck) |
| AppImage on Deck (FUSE vs extract) | `deck-testing.md`, `research/dsda-engine.md` §7 | Linux host: direct FUSE works (CI); SteamOS 3.8 ships every engine library (glibc 2.41, pipewire-jack); must not run under a Steam Compatibility tool. Deck check pending |
| UI shell: design system, navigation, gilrs gamepad, screens on fixtures | `specs/008-ui-shell.md` | Done (A6 needs Deck) |
| Steam Input handoff, gyro-as-mouse, strict-mode demos | — | Needs Deck Validation |

## Backlog — next major release (owner said go 2026-10-16; specs written)

| Item | Spec | State |
| --- | --- | --- |
| Widescreen status bars (doom_widehud, GPL-3.0; credited in About/README/recipes) | 017 | **Done** (A4 e2e run 39065323300 passed, A6 needs Deck): 5 mapset bars (Ancient Aliens, BTSX E1/E2, Eviternity, Valiant) + Doom/Doom II IWAD bar for SIGIL, SIGIL II, Going Down, Scythe; default on; cosmetic-only and `release` lint. Residual risk: the Eviternity bar derives from CC BY-NC-ND art (DDL only downloads it from upstream; research §4) |
| Multiple profiles per mapset (`deck-testing.md` C4) | 018 | **Done** (A5 needs Deck): picker on Play setup, new (copy/defaults), preset-name rename, guarded delete, remembered profile, Home/Library labels, `--profile` CLI; app state stays schema 2 |
| Catalog updates without a DDL release | 019 | **Draft — waiting for the owner** (fetch policy, trust model, publishing; spec Q1–Q3) |
| Recipe changes after install | 020 | **Done**: typed change classification, silent metadata adoption, staged generations published by an atomic `library/<id>/active.json` rename, idempotent profile reconciliation, Update / Keep current (per recipe hash), removed-from-catalog badge, `ddl mapset status|update` |
| Uninstall a mapset (owner decision D-022: checkbox to also delete profiles and saves) | 022 | Ready — starting (opencode, Codex window spent) |
| GOG and other store IWAD paths | 021 | **Done** (A5 needs Deck): Heroic (native + Flatpak), minigalaxy, GOG offline default; opt-in standard Linux folders. Lutris stays manual (would need SQLite). Follow-up: add the published GOG Final Doom hashes to `data/iwads.toml`. |

# Status

**Milestone:** M0 — repository and feasibility spikes  ·  **Updated:** 2026-10-16

| Item | Spec / output | State |
| --- | --- | --- |
| Repo foundation (license, AGENTS.md, docs) | `docs/` | Done |
| Workspace, CI, release pipeline | `specs/001-workspace-ci-release.md` | In progress |
| DSDA-Doom v0.30.x CLI, config, pistol start, strict mode (desk research) | `research/dsda-engine.md` | Done |
| Upstream sources: DSDA releases/AppImage, `/idgames` API, AUGER;ZENITH | `research/upstream-sources.md` | Done |
| Reference mapsets SIGIL / SIGIL II | `research/reference-sigil.md` | Done |
| Prior art: UZDeck / GZDeck (incl. UI critique) | `research/prior-art.md` | Done |
| Recipe schema, catalog, lint, reference recipes | `specs/002-recipe-catalog.md` | Ready — waits for 001 |
| Launch plan and engine process runner | `specs/003-launch-plan.md` | Ready — waits for 001 |
| Engine manager (discover, download, verify, install, rollback) | `specs/004-engine-manager.md` | Ready — waits for 001, 006 |
| Downloads and safe ZIP extraction (nested) | `specs/006-downloads-extraction.md` | Ready — waits for 001 |
| DsdaConfigManager v1 | `specs/005-dsda-config-manager.md` | Ready — waits for 001 |
| UI contract v1 + wireframes (Codex) | `design/ui.md`, `design/mockups/` | Done |
| UI visual pass: hi-fi generated mockups, identity (Codex) | `design/visual/` | In progress |
| IWAD identification (hashes, lumps, Steam paths) | `research/iwad-identification.md` | Done |
| IWAD table, identifier, Steam discovery, canonical links | `specs/007-iwad-library.md` | Ready — waits for 001 |
| AppImage on Deck (FUSE vs extract) | `deck-testing.md` | Needs Deck Validation |
| Slint + controller backend on Deck (SDL3 vs gilrs) | — | Not started (needs a release build) |
| Steam Input handoff, gyro-as-mouse, strict-mode demos | — | Needs Deck Validation |

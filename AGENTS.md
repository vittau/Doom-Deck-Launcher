# AGENTS.md — DoomDeckLauncher (DDL)

Native, controller-first DOOM mapset launcher for Steam Deck. Rust + Slint UI; launches the
official DSDA-Doom AppImage. License: GPL-3.0-or-later.

## Sources of truth (read before working)

| File | Role |
| --- | --- |
| `OVERVIEW.md` | Product north star: vision, boundaries, architecture. Read the sections your task touches. |
| `docs/decisions.md` | Current decisions. They win over `OVERVIEW.md` when the two differ. |
| `docs/specs/NNN-*.md` | What to build: requirements and acceptance criteria. |
| `docs/research/*.md` | Verified facts about DSDA-Doom, Steam, upstream sources — with links. |
| `docs/status.md`, `docs/journal.yml` | Current state and changelog. Coordinator-owned. |

## Orchestration

Work is coordinated in Orca ADE: an orchestrator agent writes specs and briefs, dispatches one
worker per task (own worktree and branch), reviews, merges and releases.
- **Worker:** follow your injected brief and this file; report with `worker_done`.
- **Orchestrator, or taking over as one:** read `docs/orchestration.md` first.

## Workflow (spec-driven)

1. **No spec, no code.** Implement exactly what the spec or task asks. Behaviour the spec does not
   cover → ask; do not improvise.
2. Every requirement you implement gets a test or a stated reason why it can't have one.
3. Facts about DSDA-Doom, Steam Deck, AppImage or WAD content come from upstream source or docs,
   cited with a link (pin tags/commits). Label claims **Confirmed**, **Proposed** or
   **Needs Deck Validation**. Never invent CLI flags, config keys, complevels, hashes or
   compatibility data.
4. Finish with a report: what changed, how it was verified, open questions, spec items left open.

## Layout

```text
crates/ddl-core/    pure logic, no UI deps (catalog, recipes, resolver, downloads, library,
                    profiles, dsda_config, engines/dsda, platform, diagnostics)
crates/ddl/         binary `ddl`: Slint UI and app wiring
docs/               decisions, specs, research, status, journal
catalog/            one TOML recipe per mapset — community-contributed via PR (D-010)
.github/workflows/  CI and releases (built in a Steam Linux Runtime SDK container)
scripts/            developer and orchestration helpers
```

## Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The dev host is macOS arm64; releases are x86_64 Linux, built only in CI. Code must compile and
test on both: put Linux-only behaviour behind `cfg` or traits, and test it with fakes.

## Hard rules

- One engine: DSDA-Doom. **No Steam Input code** of any kind (README instructions only).
- Never bundle or download IWADs. Never execute anything from downloaded archives.
- Launch processes with `std::process::Command` and separate arguments — never a shell string.
- All remote data and archives are untrusted: bound sizes, reject path traversal and symlink
  escapes, verify hashes when known, time out network calls.
- User state: atomic writes, versioned schemas, XDG paths resolved in code (no hardcoded home).
- Formats: TOML for hand-authored data (recipes, catalog, templates), JSON for machine state.
  Do not use `serde_yaml` (unmaintained).
- Keep `ddl-core` free of UI dependencies. Prefer small DSDA-specific types over generic
  plugin frameworks.
- New dependencies: maintained crates only; name and justify each one in your report.
- Tests: unit tests beside the code, fixtures in `tests/fixtures/`, no network access.
- Repository text is in English.
- Do not edit `OVERVIEW.md`, `docs/decisions.md`, `docs/status.md` or `docs/journal.yml` unless
  your task says so; propose the change in your report instead.
- Commit small, imperative-subject commits on your task branch. Never push, tag or release.

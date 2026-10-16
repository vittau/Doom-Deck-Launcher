# 001 — Workspace skeleton, CI and release pipeline

**Status:** Ready  ·  **Milestone:** M0  ·  **Depends on:** —

## Goal

A buildable Rust/Slint workspace whose `ddl` binary opens a controller-style placeholder screen,
checked on every push, and released as a Steam-Deck-ready tarball when a `v*` tag is pushed. This
is the first artifact the owner downloads and runs on the Deck.

## Requirements

- **R1** — Cargo workspace per D-005: `crates/ddl-core` (library, no UI dependencies) and
  `crates/ddl` (binary named `ddl`, depends on `ddl-core` and Slint). Edition 2024,
  `rust-toolchain.toml` pinning an exact stable toolchain, workspace-level `rust-version`,
  committed `Cargo.lock`.
- **R2** — `ddl-core::platform::paths` resolves DDL's config, data, cache and state directories
  per the XDG Base Directory spec (`XDG_*_HOME` when absolute, else `$HOME/.config`,
  `$HOME/.local/share`, `$HOME/.cache`, `$HOME/.local/state`), each joined with `ddl`. The
  resolver takes its environment as input so tests need no real environment; missing `HOME`
  is an error, not a panic.
- **R3** — `ddl --version` prints `ddl <version>` and exits 0; `ddl --help` lists options;
  unknown options exit non-zero with a message. `--windowed` runs in a 1280×800 window instead
  of fullscreen.
- **R4** — With no options, `ddl` opens a fullscreen Slint window titled `DoomDeckLauncher` that
  shows the product name and a vertical menu of three placeholder items ("Play", "Settings",
  "Quit"). Up/Down move a clearly visible focus highlight; Enter/Return activates; "Quit" and
  Escape on the menu exit with status 0. No hover-dependent behaviour. Layout readable at
  1280×800. (Gamepad input arrives with the input spike; keyboard only here.)
- **R5** — Structured logging via `tracing` to stderr, level from `DDL_LOG` (default `info`).
- **R6** — `.github/workflows/ci.yml`, on push and pull request: `cargo fmt --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` on
  `ubuntu-latest`, plus a release-mode build of `ddl` inside the same Steam Linux Runtime SDK
  container as R7 that runs `ddl --version` and prints `ldd target/release/ddl`.
- **R7** — `.github/workflows/release.yml`, on tags `v*`: build `ddl` in release mode inside a
  Steam Linux Runtime SDK container image **pinned by digest**; package
  `ddl-linux-x86_64.tar.gz` holding a top-level `ddl/` directory with the `ddl` binary,
  `LICENSE` and `README.md`; write `ddl-linux-x86_64.tar.gz.sha256`; create the GitHub Release
  with both assets using the `gh` CLI and `GITHUB_TOKEN` (tags containing `-` are marked
  prerelease). Use only `actions/*` first-party actions; no third-party release actions.
- **R8** — The choice of runtime (e.g. Steam Linux Runtime 3.0 "sniper" vs a newer one), image
  reference and digest, Rust installation method inside it, and the Slint backend/renderer are
  documented in `docs/research/steam-runtime-build.md` with links and Confirmed / Proposed /
  Needs Deck Validation labels. Default to choices that minimize runtime library requirements
  on SteamOS; note every dynamic library the binary needs.
- **R9** — `README.md` for end users: what DDL is, pre-alpha status, install from a GitHub
  Release (extract, add `ddl` as a non-Steam game, launch in Gaming Mode), build from source,
  license, credits to UZDeck and GZDeck, and a non-affiliation trademark note
  (see `OVERVIEW.md` §18). No Steam Input section yet.

## Out of scope

Gamepad input, DSDA-Doom handling, any real screen content, Flatpak, auto-update.

## Acceptance criteria

- [x] **A1** (R1, R5) — `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
  and `cargo test --workspace` pass on macOS.
- [x] **A2** (R2) — unit tests cover: XDG var set and absolute, set but relative (ignored),
  unset (fallback), missing `HOME` (error).
- [x] **A3** (R3) — tests (or a scripted check in the report) show `--version`, `--help`,
  unknown-option behaviour.
- [x] **A4** (R4) — `cargo run -p ddl -- --windowed` on macOS shows the menu; focus moves with
  arrows; Quit exits 0 (describe what you observed, screenshot if possible).
- [ ] **A5** (R6, R7) — workflows are syntactically valid YAML and every step's command was
  reviewed; actual CI execution is verified by the coordinator after push.
- [x] **A6** (R8) — research doc exists with pinned image digest and cited sources.
- [ ] **A7** (R4, R7) — *Needs Deck Validation:* the release tarball runs in Desktop and Gaming
  Mode (added to `docs/deck-testing.md` by the coordinator).

## Tasks

- [x] Workspace, toolchain, crates, `paths` module + tests
- [x] CLI parsing, logging, Slint placeholder menu
- [x] Runtime/container research → `docs/research/steam-runtime-build.md`
- [x] `ci.yml`, `release.yml`
- [x] `README.md`

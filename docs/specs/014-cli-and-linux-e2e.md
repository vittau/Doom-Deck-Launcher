# 014 — Headless CLI and a real-network Linux end-to-end job

**Status:** Ready  ·  **Milestone:** M2  ·  **Depends on:** 004, 007, 009, 010
**Why now:** the owner cannot test on the Deck for a while. Exercising the real download,
verification, extraction, AppImage and launch-planning paths on a real Linux machine (GitHub's
Ubuntu runner and the Steam Runtime SDK container) catches most non-hardware failures before
the Deck test, and the same commands give the owner a Konsole diagnostic tool on the Deck.

## Goal

`ddl` gains a few non-UI subcommands that drive the same `ddl-core` services as the UI, and a
manually triggered CI workflow runs them end to end against the real upstream hosts.

## Requirements

- **R1** — **Subcommands** (same parser style as `ddl catalog lint`, `--help` for each, exit
  code 0 on success, non-zero with a one-line actionable error otherwise; never opens a window):
  - `ddl engine status` — installed versions, active one, mode (appimage/extracted).
  - `ddl engine install [--version 0.30.1] [--force-extract]` — the reviewed install path of
    spec 004 with progress on stderr; `--force-extract` takes the D-013 extraction fallback even
    when the AppImage would run, so the fallback can be validated on purpose (spec 004 R5,
    `deck-testing.md` E3).
  - `ddl iwad scan [PATH…]` — Steam discovery plus the given folders; prints each identified
    file with variant, game and SHA-256; ambiguous files are listed, never auto-chosen.
  - `ddl mapset list` / `ddl mapset install <id> [--with <component>…]` — spec 009 install
    with progress; components validated against the recipe.
  - `ddl launch <id> --dry-run [--with …] [--skill N] [--pistol-start]` — resolves exactly as
    Play does and prints the argv (one argument per line) and the working directory, or the
    typed `Blocker` with its explanation. Without `--dry-run` it launches through the same run
    guard as the UI (useful in Konsole on the Deck).
  - `ddl doctor` — one report: paths, engine state, IWAD registry, installed mapsets, DSDA
    config state, log location. Paste-friendly.
- **R2** — **Same state as the UI:** the subcommands use the XDG paths and state files the UI
  uses; a UI started afterwards sees what the CLI installed. Concurrent UI + CLI mutation is
  refused with a clear message (reuse the existing guards; add a small state lock only if one
  is missing, and justify it).
- **R3** — **Linux end-to-end workflow** `.github/workflows/e2e.yml`, `workflow_dispatch` only
  (manual; no schedule): builds `ddl` in the pinned Steam Runtime SDK container, then with real
  network:
  1. `ddl engine install` (AppImage path) and, in a fresh data dir, `ddl engine install
     --force-extract`; both must pass the `-v` smoke test.
  2. `ddl mapset install` for every catalog recipe with every free optional component (AUGER's
     add-ons one at a time, SIGIL's compat variant).
  3. Synthetic IWADs cannot stand in for real ones (never download IWADs), so `ddl launch
     --dry-run` is checked for the expected `MissingIwad` blocker per recipe; additionally
     a test-only seam (feature or env var, off in release builds) registers a fake IWAD of the
     right name so the dry-run argv can be asserted against D-011 (single `-file`, `-iwad`
     absolute canonical link, numeric `-complevel`, `-noautoload`, `-config`, per-profile
     dirs).
  4. Uploads `ddl doctor` output and the logs as a workflow artifact.
  Also run the same steps on `ubuntu-latest` outside the container, recording whether the
  AppImage runs directly there (FUSE) — informative, not a failure.
- **R4** — **Docs:** README "Troubleshooting from Konsole" lists the subcommands;
  `deck-testing.md` E3 uses `ddl engine install --force-extract`.

## Out of scope

Any change to the UI flows. Downloading IWADs (never). Scheduled e2e runs.

## Acceptance criteria

- [x] **A1** (R1, R2) — unit/integration tests for each subcommand with the existing fakes
  (loopback HTTP, fake engine, temp XDG), including `--force-extract` and the dry-run argv.
- [ ] **A2** (R3) — the e2e workflow ran once on `main` (coordinator triggers it) and passed;
  the report links the run and summarises the AppImage-direct result on plain Ubuntu.
- [x] **A3** (R4) — README and `deck-testing.md` updated; fmt, clippy `-D warnings`, tests green.
- [ ] **A4** — *Needs Deck Validation:* `ddl doctor` and `ddl launch` work from Konsole.

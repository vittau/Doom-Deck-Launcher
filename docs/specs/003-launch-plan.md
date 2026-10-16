# 003 — Launch plan and engine process runner

**Status:** Done  ·  **Milestone:** M1  ·  **Depends on:** 001

## Goal

Turn a fully resolved, typed description of "what to play" into the exact DSDA-Doom argv and
environment required by D-011, start it without a shell, and report how it ended. This is the
single choke point through which every game launch passes, so it must be impossible to build
an unsafe or ambiguous command line through it.

## Requirements

- **R1** — `ddl_core::engines::dsda::LaunchPlan`, a typed value holding: engine executable
  (absolute path), IWAD (absolute path; in practice DDL's canonical-name symlink, D-017), ordered PWAD/ZIP files, ordered DEH/BEX patches,
  complevel (a `Complevel` type accepting only the reviewed numbers 0–17 and 21; never the
  placeholders 18–20, never a word such as `vanilla`, never "omit" — words are resolved
  earlier from the identified IWAD, D-017),
  skill (1–5), optional warp (episode/map), pistol start flag, optional demo recording path,
  config file path, per-profile save/data/screenshot directories, working directory, and a
  typed list of extra `-assign key=value` overrides.
- **R2** — `LaunchPlan::to_command()` produces argv in this order: `-iwad`, `-file …` (once,
  only if non-empty), `-deh …` (once, only if non-empty), `-complevel`, `-skill`, `-warp`
  (if set), `-pistolstart` (if set), `-record` (if set), `-noautoload`, `-config`, `-save`,
  `-data`, `-shotdir`, `-assign …` (once, only if non-empty). Each value is its own argv token.
- **R3** — Validation before building, with typed errors: every path absolute; files exist and
  are regular files; no path value or `-assign` key/value starts with `-`; pistol start and
  recording are mutually exclusive; `-assign` keys match `[a-z0-9_]+` and come from an
  allowlist the caller supplies; directories exist or are created by an explicit
  `prepare_dirs()` step (DSDA does not create them).
- **R4** — `EngineRunner` trait with a real implementation over `std::process::Command`
  (no shell, environment inherited minus `DOOMWADDIR`, `DOOMWADPATH`, `DOOMDATADIR`,
  `DOOMSAVEDIR`, which DDL clears so they cannot override the plan) and a fake for tests. The
  real runner captures stdout/stderr into a per-launch log file under DDL's state directory,
  and returns `LaunchOutcome { exit_status, duration, log_path }`, distinguishing a clean exit,
  a non-zero exit (DSDA's `I_Error` exits with status 255) and death by signal.
- **R5** — `smoke_test(executable)` runs `<exe> -v` with a timeout (10 s default), succeeds
  only on exit 0 with stdout starting with `dsda-doom v`, and returns the reported version.
- **R6** — A human-readable `LaunchPlan::describe()` summary (IWAD, files in order, patches,
  complevel, skill, options) for the UI's "Will launch" preview and diagnostics, with
  home-directory paths shortened to `~` (no private absolute paths in exported logs).

## Out of scope

Resolving recipes into plans (resolver spec), engine download/installation (spec 004), DSDA
config provisioning (spec 005), UI.

## Acceptance criteria

- [x] **A1** (R1, R2) — table-driven tests: argv for minimal plan, multi-file plus patches,
  warp, pistol start, assigns; snapshot of exact token order.
- [x] **A2** (R3) — one test per validation error, including repeated-flag impossibility (no
  API can emit `-file` twice) and pistol start + record rejection.
- [x] **A3** (R4) — fake runner tests; a real-runner test on a harmless system binary
  (e.g. `/bin/sh -c 'exit 3'` is **not** allowed — use a small test helper binary built from the
  workspace) proving exit status, signal death and log capture; environment scrubbing tested.
- [x] **A4** (R5) — smoke test against a fake executable script fixture written by the test
  (exit 0 with version, wrong output, non-zero, timeout).
- [x] **A5** (R6) — `describe()` snapshot test, including `~` shortening.
- [ ] **A6** — *Needs Deck Validation:* launch the official AppImage with a generated plan on the
  Deck (tracked in `docs/deck-testing.md`).

## Tasks

- [x] Types and validation
- [x] argv builder and describe
- [x] Runner trait, real runner, fake, smoke test

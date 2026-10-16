# 005 — DsdaConfigManager v1

**Status:** Done  ·  **Milestone:** M1  ·  **Depends on:** 001; uses 003's
`LaunchPlan` config path

## Goal

Give every DDL user a private, controller-ready DSDA-Doom configuration that survives engine
upgrades and DDL baseline changes without losing anything the player changed in DSDA's own
menus, and that never touches a separately installed DSDA-Doom setup (OVERVIEW §8.1, D-012).
Facts come from `docs/research/dsda-engine.md` §3–§5.

## Requirements

- **R1** — **Parser/serializer** for DSDA's config format: `name value` lines; integers
  (decimal, octal, `0x` hex as DSDA's `%i` reads them); strings in double quotes with the first
  and last character stripped, no escapes; `#` comment lines; `input_*` binding lines with the
  three-profile grammar `keys mouse joy | keys mouse joy | keys mouse joy` (keys comma-separated,
  `0` = none, `-1` = no button). Unknown keys are preserved as opaque lines. Round-tripping a
  DSDA-generated file yields identical semantic content.
- **R2** — **Baseline template** `steam-deck-default`, stored as TOML in the repository
  (`crates/ddl-core/templates/dsda/steam-deck-default.toml` or similar) with `template_version`,
  the tested DSDA version range, and only the keys DDL intends to own. Minimum content:
  `use_game_controller 1` (DSDA's default is 0), `use_mouse 1`, and the analog/mouse values from
  OVERVIEW §8.1, each marked in a comment as an upstream default, not Deck tuning. Bindings are
  not changed in v1 except where research shows a gamepad action is unbound by default (list
  any such case in the report rather than inventing a layout).
- **R3** — **Provisioning:** on first use create `<config dir>/dsda/active.cfg` (DDL's XDG config
  dir, parent created) containing the template's values; store an exact copy of what was
  applied as `<config dir>/dsda/base/<template_version>.cfg`. Never read or write DSDA's own
  default config (`~/.dsda-doom/` or `$XDG_DATA_HOME/dsda-doom/`).
- **R4** — **Run guard:** an API that callers must use around engine runs (`begin_run()` /
  `end_run()` or an RAII guard) so that any provisioning, migration or reset attempted while a
  run is active fails with a typed error. Persist a lock file so a crashed DDL does not leave a
  stale lock forever (the lock records the PID; a dead PID clears it).
- **R5** — **Three-way merge** when the template version changes: for each key in the new
  template, compare old base (B), current active file (A) and new template (T): if A == B, take
  T (the user never changed it); if A != B and T == B, keep A (the user changed it, DDL did
  not); if A != B and T != B and A != T, it is a **conflict**: keep A and report it. Keys the
  user changed that are not in the template are always kept. Keys removed from the template
  revert to user ownership (kept as they are). Before writing, back up the active file to
  `<config dir>/dsda/backups/<timestamp>-<template_version>.cfg`. Writes are atomic
  (temp file + rename). Keep the last 10 backups.
- **R6** — **Reset to recommended:** replaces the active file with the template values plus
  DSDA-generated keys left untouched, after a backup; requires an explicit `confirm` argument.
- **R7** — **Per-profile overrides:** a typed set of scalar overrides rendered as transient
  `-assign key=value` tokens for 003's `LaunchPlan` (never written into the active file), limited
  to an allowlist of known keys from the research inventory.
- **R8** — **Diagnostics:** `state()` returns DSDA version range, template version, active file
  path (with `~` shortening for exported logs), last migration time, and unresolved conflicts.

## Out of scope

Any Steam Input handling (README only). Any tuning UI. Deck-calibrated values (they come from
on-device testing later).

## Acceptance criteria

- [x] **A1** (R1) — parser/serializer tests with a fixture produced from DSDA's documented
  defaults (build it from the research inventory; do not copy a real user file), including
  bindings, strings, hex, comments and unknown keys.
- [x] **A2** (R2, R3) — provisioning test in a temp dir; asserts DSDA default paths untouched.
- [x] **A3** (R4) — guard tests, including stale-lock recovery.
- [x] **A4** (R5) — merge tests for all five cases (untouched, user-only change, template-only
  change, conflict, removed key), backup creation and rotation, atomic write.
- [x] **A5** (R6, R7, R8) — reset with/without confirm; override allowlist; state snapshot.
- [ ] **A6** — *Needs Deck Validation:* change a setting in DSDA's menu, quit, relaunch from
  DDL: the setting persists (tracked in `docs/deck-testing.md`).

## Tasks

- [x] Format parser/serializer
- [x] Template + provisioning + run guard
- [x] Merge, backups, reset, overrides, state

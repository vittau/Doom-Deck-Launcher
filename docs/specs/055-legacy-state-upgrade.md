# 055 — Read state saved by older releases, and a usable "could not read saved setup" screen

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 046, 010, 020
**Origin:** owner's Deck test of `v0.0.7-deck` over a `v0.0.6-local` install (2026-10-20): DDL
opened on the first-run "Install DSDA-Doom" again; Install showed "Install stopped" and "Could not
read saved setup. Open Diagnostics to find the log location, then Retry". Settings → Diagnostics /
About, Engine, IWAD folders and DSDA config were all **empty panels**; Retry did nothing. Deleting
`~/.config/ddl` fixed it (engine, IWADs and mapsets were found again, AUGER;ZENITH played).

## Diagnosis (coordinator, Confirmed in code)

`crates/ddl/src/services.rs` `AppState` stores whole catalog recipes
(`recipes: BTreeMap<String, Recipe>`), and `Recipe` and its parts are `deny_unknown_fields`.
Spec 046 removed `complevel_status` and changed `complevel` to DSDA aliases, so a v0.0.6
`app.json` no longer deserializes and `AppState::parse` returns `Problem::State` for the whole
file. Spec 046 tested only the other direction (old clients reading the new catalog).

## Requirements

- **R1 — Audit:** list every file DDL persists (app state, library `active.json` and generation
  metadata, profiles, custom setups, catalog cache, DSDA config state, engine state, anything
  else) and diff each serialized type between the tags `v0.0.6-local` and `v0.0.7-deck`, and
  between `v0.0.5-screens.1` and `v0.0.6-local`. The report has the table: file, type, changed
  (yes/no), how this spec handles it.
- **R2 — Read old shapes:** every persisted file written by `v0.0.5-screens.1`, `v0.0.6-local` or
  `v0.0.7-deck` loads with no data loss that matters to the player: engine, IWAD folders,
  installed mapsets and their generations, profiles and their selection, favorites, kept recipes,
  custom setups, settings. Old recipe copies accept `complevel_status` (ignored) and a numeric
  `complevel` (mapped to the alias of spec 046 when one names it; otherwise kept as DSDA accepts
  it). Keep `deny_unknown_fields` for hand-authored catalog TOML; tolerance applies only to state
  DDL wrote itself. Prefer a typed legacy form plus conversion over loosening the current types.
  The next save writes the current schema atomically; bump the schema version only if a field
  meaning changes, and then keep a one-time backup `<file>.v<N>.bak` beside it.
- **R3 — Real fixtures:** add `tests/fixtures/state/v0.0.6-local/` (and `v0.0.5-screens.1/`,
  `v0.0.7-deck/`) — a complete config + data tree produced by **the real code at that tag**
  (build or run the tag's `ddl` CLI or its serializers in a scratch worktree; never hand-edit
  JSON), with one engine, two IWAD folders, two installed recipes (one with an optional
  component), two profiles on one recipe, a favorite and a custom setup. A test loads each tree
  with the current code and asserts the items above. Document in the report how the fixtures
  were made so the next release can add its own.
- **R4 — Guard:** a test fails when the current code can no longer read any fixture tree. From
  now on every release tag adds its own fixture tree; propose the release-checklist line in the
  report (the coordinator adds it to the docs).
- **R5 — Unreadable state is a screen, not first run:** when a state file still cannot be read
  (corrupt JSON, a future schema), DDL shows a dedicated recovery screen instead of the first-run
  "Install DSDA-Doom" flow. It names the file and a one-line reason, and offers:
  - **Retry** — re-reads the files; on another failure the screen says so visibly (it must never
    look like nothing happened);
  - **Start fresh** — after a confirm dialog, moves only the unreadable file(s) to
    `<name>.unreadable-<UTC timestamp>` (never delete) and continues; installed engine, mapsets
    and IWADs on disk are found again as on a clean first run;
  - **Diagnostics** — opens Settings → Diagnostics.
- **R6 — Settings work without state:** Settings → Diagnostics / About, Engine, IWAD folders and
  DSDA config never render as empty panels. With unreadable state, Diagnostics still shows
  version, log file path, config/data paths and the failing file; panels that need state show a
  one-line explanation instead of nothing. Find and fix why they were empty.
- **R7 — Controller-first:** the recovery screen and its dialog are fully usable with the D-pad,
  A and B; long text scrolls (the right-stick scroll of spec 057 will apply here too).

## Acceptance criteria

- [ ] **A1** (R1) audit table in the report.
- [ ] **A2** (R2, R3) fixture trees for three tags, made by the tagged code; load tests pass
  and assert engine, folders, installs, profiles, favorites, custom setup.
- [ ] **A3** (R4) the guard test; a deliberately broken field makes it fail (shown in the report).
- [ ] **A4** (R5, R6) headless renders (PNG, Slint software renderer) of the recovery screen,
  its confirm dialog, the "still unreadable" Retry result and Settings → Diagnostics with
  unreadable state; key-driven tests for Retry, Start fresh (file renamed, not deleted) and
  Diagnostics.
- [ ] **A5** fmt, clippy `-D warnings`, `cargo test --workspace` green.
- [ ] **Needs Deck Validation:** install `v0.0.6-local`, set it up, upgrade to the new build:
  everything is still there (coordinator adds the check to `docs/deck-testing.md`).

# 032 — Play the base games, and see where IWADs were found

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 007, 018, 021, 026, 028, 030
**Origin:** Deck test 2026-10-19 on `v0.0.6-local`. To play plain Ultimate Doom the owner had to
build an empty custom setup (L1 ⚠️): "tem que ser fácil de jogar só o base se a pessoa quiser."
After "Use this game", the IWAD folders screen said "No folders added." although Steam IWADs
were in use: "Seria bom que os IWADS adicionados automaticamente tb apareçam como folders."
Decision **D-027**.

## Requirements

- **R1** — **Base games:** every base game DDL identifies among the player's IWADs (Ultimate
  Doom / registered Doom, Doom II, TNT: Evilution, The Plutonia Experiment; others the IWAD
  table already identifies) appears as a playable **base game** entry — one per game, not one
  per file. It has no download, no install step and no catalog recipe.
- **R2** — **Where:** a "Base games" group in Library (above or beside installed mapsets), and base
  games appear in Home's recently played after they are played. Each entry's page shows the IWAD
  file in use and its source ("Steam", "GOG via Heroic", folder), and offers Play and Play setup.
- **R3** — **Correct defaults:** the complevel is DSDA's vanilla level for that IWAD — 3 for Doom 1
  / Ultimate Doom, 2 for Doom II, 4 for TNT and Plutonia (Confirmed,
  `docs/research/iwad-identification.md` §3, DSDA v0.30.1 `settings.c` L59–75). The IWAD pick
  prefers the classic `base/` copy over KEX `rerelease/` (spec 028 R3); the player may switch to
  another identified copy in Play setup.
- **R4** — **Profiles and extras:** base games get profiles, saves, skill, pistol start and Extra
  files exactly like mapsets (specs 018, 026), stored under a stable id per game
  (e.g. `base:doom2`) that cannot collide with recipe ids or custom setup ids. Badged as the
  original game, not "Custom — not verified".
- **R5** — **IWAD sources list:** Settings → IWAD folders lists every folder DDL reads IWADs from:
  automatic ones (Steam library paths, GOG/Heroic, minigalaxy, standard Linux folders when on)
  with their source label and the games found there, followed by folders the player added.
  Automatic entries have no Remove (a rescan would find them again). "No folders added." only
  appears when the player added none, and never as the only line when automatic folders exist.
- **R6** — **CLI:** `ddl doctor` lists base games; `ddl launch base:<game>` works.

## Constraints

- Never bundle or download IWADs. App state schema bump only if needed, with migration and a
  versioned schema. Ember look; 1280×800; headless renders only.

## Acceptance criteria

- [ ] **A1** (R1, R3) — core tests with the spec 028 fixture: four base games, correct complevels,
  `base/` copy chosen; a library with only `rerelease/doom2.wad` still yields Doom II.
- [ ] **A2** (R2, R4) — headless renders of Library with Base games, a base game page and its Play
  setup, inspected; a launch-plan test shows the argv for `base:doom2` (IWAD, `-complevel 2`, no
  `-file`); profiles and saves are separate per base game.
- [ ] **A3** (R5) — render and test of the IWAD folders screen with Steam entries and one added
  folder; no Remove on automatic entries.
- [ ] **A4** (R6) — CLI tests.
- [ ] **A5** — fmt, clippy `-D warnings`, tests green. Deck re-check: play Ultimate Doom and
  Doom II from Library (Needs Deck Validation).

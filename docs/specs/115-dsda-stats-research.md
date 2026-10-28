# 115 — Research: per-mapset progress from DSDA-Doom's own stats

**Status:** Ready · **Milestone:** M4 ("improved per-mapset progress", OVERVIEW §13) ·
**Depends on:** 003, 018, 114 (D-044) · **Type:** research only — no product code
**Origin:** owner, 2026-10-28: liked the idea of showing progress/statistics per mapset; wants a
worker to find out what is possible and to be told before anything is implemented.

## Goal

Find out, from pinned DSDA-Doom source and DDL's own launch layout, what progress data DDL can
read reliably after a game session, and propose a small, honest feature (or say it is not worth
it). The owner decides from this report whether an implementation spec is written.

## Questions (answer each, labelled Confirmed / Proposed / Needs Deck Validation)

- **Q1 — What DSDA-Doom records.** At tag `v0.30.1`: what `wad_stats.c` writes (file name,
  location, format and version marker, every field: completion, best skill, kills/items/secrets,
  times, max values, anything else), when it is written (level exit, quit, crash, death,
  menu New Game, load game) and when it is read back. Same for `split_tracker.c` and for
  `-levelstat` / `-analysis` output if relevant. Cite file and line ranges.
- **Q2 — Where it lands for DDL.** DDL passes `-data`, `-save`, `-config` and the rest per
  launch (`crates/ddl-core/src/engines/dsda/plan.rs`, resolver, profiles). With DSDA's data
  organizer (filename-stem based, up to eight PWAD stems, research `dsda-engine.md` §data
  organizer), which path holds the stats for a catalog mapset, a base game, a custom setup and a
  profile with extra files? Are profiles isolated from each other? What happens when two setups
  share stems?
- **Q3 — Interaction with DDL settings.** Pistol start, complevel, add-ons/extra files,
  menu-only launches (D-044, no `-skill`/`-warp`), Nightmare, saves loaded from the menu, `-nomonsters`
  style modifiers: which of these change or invalidate what is recorded?
- **Q4 — Denominator and names.** How DDL could know the total number of maps and their display
  names for a mapset (UMAPINFO, MAPINFO, DEHACKED strings, lump scan of `MAPxx`/`ExMy`, recipe
  data) — with the limits of each; whether stats already contain the full map list.
- **Q5 — Reading it safely.** Size bounds, partial/corrupt files, files written while DSDA runs,
  format changes across DSDA releases, Flatpak sandbox paths. DDL treats the file as untrusted
  local data.
- **Q6 — Proposal.** What DDL could show (e.g. "14/32 maps" on Library cards, a per-map table on
  Mapset detail with kills/secrets/time, overall time), with a rough size and the Deck checks it
  would need. State what is **not** possible or not reliable. Respect the hard rules: no
  telemetry, no network, no Steam integration, DSDA-Doom only, no writes to DSDA's files.

## Constraints

- No product code, no new dependencies, no catalog changes. A throwaway parser or probe under
  the worker's scratch area is fine if it helps; do not commit it into `crates/`.
- Never open visible windows or capture the screen. Running the DSDA-Doom AppImage is not
  possible on the macOS dev host; reason from source and existing fixtures, and mark runtime
  behaviour **Needs Deck Validation**.
- Pin every DSDA citation to `v0.30.1` (and to `master` only to flag a coming change, with the
  commit).

## Deliverables

- `docs/research/dsda-stats.md` — the evidence (Q1–Q5), with pinned links.
- `docs/reports/115-dsda-stats-research.md` — the proposal (Q6), open questions for the owner,
  and a draft requirement list an implementation spec could start from.

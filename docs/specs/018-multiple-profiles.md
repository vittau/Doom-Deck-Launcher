# 018 — Multiple profiles per mapset

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 009, 010, 015
**Backlog item:** "Multiple profiles per mapset" (`deck-testing.md` C4). The core already stores
any number of profiles per recipe (`ddl-core/src/profiles/`) with per-profile `-save`, `-data`
and `-shotdir` dirs (D-011); the UI only ever creates "Default".

## Goal

A player can keep separate runs of the same mapset — e.g. one UV pistol-start run and one casual
run with the add-on — each with its own saves, components, skill and pistol start.

## Requirements

- **R1** — **Profile picker on Play setup / Mapset detail:** shows the active profile name and
  lets the controller switch between the mapset's profiles; the last used profile is remembered
  per mapset and is what Continue launches (spec 010 R6).
- **R2** — **New profile:** creates "Profile 2", "Profile 3", … (first free number), copying the
  current profile's settings or starting from defaults (two actions). No text entry required.
- **R3** — **Rename (optional input):** a rename action offers a short list of preset names
  ("Default", "Casual", "UV", "Pistol start", "Co-op", "Speedrun", "Profile N") chosen with the
  D-pad — still no text entry (on-screen keyboard is out of scope).
- **R4** — **Delete profile:** requires a hold/confirm step, never deletes the last profile, and
  says plainly that the profile's saves and screenshots are deleted with it; the saves dir is
  removed only after the profile file is gone (atomic order, no orphan profile pointing to
  deleted saves).
- **R5** — **Library / Home:** a mapset with more than one profile shows the profile name on its
  Continue card; Library shows the count.
- **R6** — **CLI:** `ddl launch <id> [--profile <name|id>]` and `ddl doctor` lists profiles per
  mapset (spec 014 parser style).
- **R7** — Bindings follow `docs/design/ui.md`; document the new ones there. Ember style (D-021).

## Out of scope

Text entry / on-screen keyboard. Moving saves between profiles. Profile import/export.

## Acceptance criteria

- [x] **A1** (R1–R4) — core and service tests: create/copy/rename/delete, last-profile guard,
  delete order, remembered last profile, Continue uses it.
- [x] **A2** (R1, R5) — headless renders: picker with 2+ profiles, delete confirm, Home Continue
  with profile name; navigation tests drive the picker with keys.
- [x] **A3** (R6) — CLI tests for `--profile`.
- [x] **A4** — fmt, clippy `-D warnings`, tests green.
- [ ] **A5** — *Needs Deck Validation:* `deck-testing.md` C4 (saves stay separate).

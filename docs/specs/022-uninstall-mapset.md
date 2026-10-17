# 022 — Uninstall a mapset

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 009, 018, 020 · **Decision:** D-022
**Why:** DDL can install mapsets but never remove them (found while specifying 020). Owner
decision (2026-10-17): "ter um checkbox pra marcar se quiser também limpar perfis e saves."

## Goal

The player can remove an installed mapset from the controller, choosing whether its profiles and
saves go too, and nothing outside that mapset is ever touched.

## Requirements

- **R1** — **Action:** "Uninstall" on Mapset detail and Library (installed mapsets only, including
  ones no longer in the catalog, spec 020 R5). Not offered while the mapset is running.
- **R2** — **Confirm dialog with a checkbox (D-022):** names the mapset and its installed size;
  a controller-toggleable checkbox "Also delete profiles and saves (N profiles)" — **unchecked by
  default**; focus starts on Cancel. With the box unchecked only the install files go; with it
  checked the mapset's profiles and their save/data/shot dirs go too.
- **R3** — **Safe removal (core):** remove only paths under DDL's library dir for that recipe
  (every path canonicalised and checked to be inside it; symlinks never followed out). Order:
  mark the install inactive atomically first (so a crash leaves "not installed", never a
  half-install that launches), then delete files; leftovers from an interrupted delete are
  cleaned on next start. User-imported components (D-014) are part of the install and go too —
  the dialog says so when present. The shared download cache and IWAD links are untouched.
- **R4** — **Profiles kept:** when kept, profiles stay and reappear if the mapset is reinstalled
  (same recipe id). When deleted: profile files first, then their dirs (spec 018 order); the
  remembered-profile and favorite entries for the recipe are cleared from app state only when
  profiles are deleted (favorites stay — they are a catalog preference).
- **R5** — **CLI:** `ddl mapset uninstall <id> [--delete-profiles]` with the same rules; refuses
  while the UI holds the state lock (spec 014 R2).
- **R6** — Bindings documented in `docs/design/ui.md`; Ember style with existing dialog components.

## Out of scope

Bulk uninstall. Uninstalling the engine or IWADs. Clearing the download cache.

## Acceptance criteria

- [ ] **A1** (R3, R4) — core tests: path containment (traversal and symlink escapes rejected),
  crash between mark and delete, kept vs deleted profiles, reinstall finds kept profiles.
- [ ] **A2** (R1, R2) — headless renders: dialog unchecked and checked; navigation test toggles
  the checkbox with the controller; focus starts on Cancel.
- [ ] **A3** (R5) — CLI tests.
- [ ] **A4** — fmt, clippy `-D warnings`, tests green.
- [ ] **A5** — *Needs Deck Validation:* uninstall from Gaming Mode.

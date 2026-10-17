# 020 — Recipe changes after install

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 009, 018
**Backlog item:** "Catalog change after install". Today `install` refuses with
`InstallError::RecipeChanged` when the installed manifest's `recipe_sha256` differs from the
current recipe (`crates/ddl-core/src/library/install.rs`), and the UI has no flow for it. Recipes
already change between DDL releases (the embedded catalog, D-019), and would change more often
if spec 019 is accepted, so this needs a proper path.

## Goal

When the catalog's recipe for an installed mapset changes (new release, new mirror, new
component, fixed complevel) or disappears, the player sees it plainly and chooses, without ever
losing saves or breaking the installed copy.

## Requirements

- **R1** — **Classify the change** (core): compare the install manifest with the current recipe
  and report one of: *metadata only* (credits, mirrors, tags, notes — same files and launch),
  *launch change* (complevel, IWAD, load order, DEH), *content change* (different package hashes,
  members or release), *components changed* (added/removed optional components), or *removed from
  catalog*. A recipe change can carry several.
- **R2** — **Metadata only:** adopt silently — update the manifest's recipe identity without
  re-downloading.
- **R3** — **Other changes:** Library card and Mapset detail show "Update available" with a short
  human summary of what changes (e.g. "New release 1.1 · complevel 9 → 11 · new add-on"). Actions:
  **Update** (download into a staging dir, verify, then swap atomically; the old install stays
  playable until the swap succeeds) and **Keep current** (dismiss for this recipe hash; keeps
  launching from the manifest, D-019).
- **R4** — **Profiles survive:** saves, data and screenshot dirs are untouched by Update; profile
  component selections that no longer exist are dropped with a notice; new components get their
  recipe default. A content change warns that old saves may not load in the new release (DSDA
  saves are version-sensitive) — wording only, no blocking.
- **R5** — **Removed from catalog:** the mapset stays installed and playable, marked
  "No longer in the catalog". (DDL has no uninstall flow yet — backlog item, not part of this spec.)
- **R6** — **CLI:** `ddl mapset status [<id>]` prints the classification; `ddl mapset update <id>`
  runs the Update path.

## Out of scope

Automatic updates without confirmation (except R2). Migrating save files.

## Acceptance criteria

- [ ] **A1** (R1, R2) — classification tests for each kind, combinations, and metadata-only adoption.
- [ ] **A2** (R3–R5) — update tests with the loopback fake: staging + atomic swap, failure leaves the
  old install intact, saves untouched, dropped component notice; removed-recipe handling.
- [ ] **A3** — headless renders: Library "Update available", detail with the summary, removed
  badge; CLI tests.
- [ ] **A4** — fmt, clippy `-D warnings`, tests green.

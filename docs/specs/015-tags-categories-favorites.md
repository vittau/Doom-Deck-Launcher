# 015 — Recipe tags, Discover categories and favorites

**Status:** Ready  ·  **Milestone:** M2  ·  **Depends on:** 002, 010, 012
**Owner request (2026-10-16):** grow the catalog and bring back the category strip and favorites
that spec 010 left out for lack of recipe metadata.

## Goal

Discover becomes browsable as the catalog grows: each recipe carries reviewed tags, Discover
filters by category with the controller, and the player can mark favorites.

## Requirements

- **R1** — **Tag vocabulary** `catalog/tags.toml` (hand-authored, D-006): per tag `id`, `label`
  (English, translatable in the UI), short `description`, `order`. Initial set: `featured`,
  `beginner-friendly`, `classics`, `cacowards` (Cacoward winners, with the year in the recipe
  evidence), `community-projects` (multi-author projects such as Doomer Boards), `megawad`
  (≥ 30 maps), `episode` (one 9-map episode or less), `challenging` (aimed at experienced
  players). Unknown tags in a recipe are lint errors.
- **R2** — **Recipe field** `tags = ["…"]` (optional, schema 1 compatible: absent = no tags).
  Lint checks every tag exists; `cacowards` requires a citation in `[verification]` notes or
  `inspected` (the Doomworld Cacowards page for that year). CONTRIBUTING.md documents tags and
  how to propose a new one (edit `tags.toml` in the same PR).
- **R3** — **Tag the existing recipes** with evidence-backed tags (Cacowards years cited from
  doomworld.com/cacowards; map counts from the recipe research).
- **R4** — **Discover categories:** a horizontal category strip above the grid — "All", then the
  tags in `order` that at least one recipe uses, then "Favorites" — controller-navigable (D-pad up
  from the grid reaches the strip; L2/R2 or the strip itself switches; exact bindings follow
  `docs/design/ui.md`), remembering the last category for the session. Empty Favorites shows a
  friendly empty state explaining how to add one.
- **R5** — **Favorites:** X toggles favorite on a focused Discover card and on Mapset detail
  (hint bar shows it); persisted in the app state JSON (atomic, versioned, migrate old files);
  favorites sort first within any category. Library shows the favorite mark too.
- **R6** — **Cards show tags** compactly (at most two chips on a card, all on Mapset detail), in
  the Ember style.

## Out of scope

Search/text entry. Remote catalog updates. Per-user custom tags.

## Acceptance criteria

- [x] **A1** (R1–R3) — lint tests for unknown tags and the cacowards citation rule; all catalog
  recipes lint clean with tags; CONTRIBUTING.md updated.
- [x] **A2** (R4–R6) — headless renders: Discover "All", one tag category, Favorites empty and
  non-empty, detail with tags; navigation tests for the strip and X toggling; state migration test.
- [x] **A3** — fmt, clippy `-D warnings`, tests green.
- [ ] **A4** — *Needs Deck Validation:* category strip and favorites with the controller.

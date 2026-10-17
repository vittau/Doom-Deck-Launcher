# 023 — Representative screenshots on mapset cards

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 002, 012, 013 · **Decision:** D-024
**Owner request (2026-10-17):** "os cards dos PWADs devem mostrar alguma screenshot
representativa do mod, e não 'original concept art'."

## Goal

Each mapset card, detail page and art band shows a real screenshot of that mapset, credited to
its source, so players recognise what they are choosing. The painted concept art stays only as
the fallback when no screenshot is available or downloaded yet.

## Decision D-024 (Accepted, revised 2026-10-17)

The owner wants real screens in Discover without caution-driven gaps. Screenshots are
**bundled**: representative in-game shots from public official or community sources, resized
and committed to the repository with source and credit, compiled into the binary like the
covers. No runtime download. Concept art is only the fallback for a recipe that has none yet.

## Requirements

- **R1** — **Assets:** for each of the 13 recipes, 1–3 representative in-game screenshots (a
  recognisable map view, not a title screen or logo), resized/cropped for the card (and the art
  band / detail where it helps), JPEG or PNG, small (aim ≤ 250 KB each), under
  `crates/ddl/ui/assets/screenshots/<recipe-id>/`, with `SOURCES.toml` (or per-file entries)
  recording source URL, source page, author/credit, original dimensions, retrieval date and any
  permission text found. Every recipe gets at least one; a recipe left without one is listed in
  the report with what was tried.
- **R2** — **Recipe link:** recipes reference their screenshots by asset name (optional field,
  schema 1 compatible) so community PRs can add them; lint checks the referenced asset exists,
  is within size/dimension bounds and has a source/credit entry. The build embeds them.
- **R3** — **UI:** cards, Discover, detail and the art band show the screenshot (cropped to fill,
  focal point centred) with a small credit caption ("Screenshot: <credit>") instead of
  "Original concept art"; the painted fallback keeps its caption. Ember style, legible caption
  on a dark plate. Headless renders cover Discover with all 13 cards.
- **R4** — CONTRIBUTING.md documents how to add screenshots with source and credit.

## Out of scope

Runtime screenshot downloads. User-supplied images. Videos.

## Acceptance criteria

- [ ] **A1** (R1, R2, R4) — all 13 recipes have at least one screenshot with source/credit;
  lint tests for missing asset, oversize, missing credit; CONTRIBUTING updated.
- [ ] **A2** (R3) — headless renders: Discover (all cards with screenshots), detail and art band
  with the credit caption.
- [ ] **A3** — fmt, clippy `-D warnings`, tests green.
- [ ] **A4** — *Needs Deck Validation:* screenshots look sharp on the Deck panel.

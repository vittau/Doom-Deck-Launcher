# 117 — Progress polish: summary placement, custom-card bar, setup tables

**Status:** Ready · **Milestone:** M4 · **Depends on:** 116 · **Decision:** D-045
**Origin:** coordinator, 2026-11-03, from report 116's open questions after the owner validated
PG1–PG6 on the Deck (0.0.20); chosen by the coordinator as the recommended next step
after the owner asked to finish, commit and push (not an explicit owner pick of this scope).

## Context

Spec 116 shipped per-mapset progress in 0.0.20. [Report 116](../reports/116-mapset-progress.md)
§ Open questions left three presentation gaps:

1. On Mapset detail the progress block sits below the title/description/profile block, so at
   the default scroll the summary is near the fold and the table needs a right-stick scroll.
2. Custom-setup Library cards show a text-only `N/M maps`, without the R9 bar.
3. The base-game and custom-setup screens show no progress section; only catalog Mapset detail
   has the summary and per-map table.

The progress data, rules and states (`Recorded`, `NoStatsYet`, `Unavailable`) already exist
in `ddl-core::progress` and are already read for base games and custom setups
(`services.rs`: `base_progress`, `custom_progress_for`). This spec only changes presentation.

## Requirements

- **R1 — Summary visible without scrolling.** On catalog Mapset detail, the progress title and
  summary (the `N/M maps` count with its bar, or the single *no stats yet* / *unavailable*
  line) are fully visible at the default scroll position at 1280×800, for both short and long
  descriptions and for a profile with add-ons and extras. Move the progress block (or only its
  summary) above the description as needed; the per-map table may stay below and keep scrolling
  with the right stick. Nothing else on the page is removed.
- **R2 — Bar on custom-setup cards.** A custom-setup Library card with a *Recorded* count draws
  the spec 116 R9 bar beside its `N/M maps` text in the same row, with the same tone and rules
  as the catalog card bar (decoration, not focusable, empty at 0, full at M). No bar when there
  is no count. The card's height and insets stay within the existing layout; if the text and
  bar do not both fit legibly, say so in the report instead of shrinking text. Base-game cards
  keep their text-only count (no room at 282 px; report 116).
- **R3 — Progress section on base-game and custom-setup screens.** The screens reached from a
  base-game card and a custom-setup card show the same progress section as Mapset detail for
  the selected profile: title, summary with bar, and the per-map table with the spec 116 R10
  UV-Max mark and R11 Kills/Secrets columns; *no stats yet* and *unavailable* show one plain
  line each. The section is read-only: focus stays on the screen's actions, D-pad and Back
  behave as today, and the right stick scrolls a long table as on Mapset detail. Its summary
  follows R1 (visible at the default scroll).
- **R4 — One builder.** Mapset detail, the base-game screen and the custom-setup screen draw
  the progress section through one shared builder (position and width as parameters), not
  three copies of the table code.
- **R5 — Evidence.** Headless renders (Slint software renderer, PNG) of: Mapset detail at the
  default scroll with a long description and recorded progress (summary visible); Mapset detail
  scrolled to the end of a long table; a custom-setup Library card with its bar; the base-game
  screen and the custom-setup screen with a recorded table including a UV-Max row, and each in
  *no stats yet*. Headless tests assert R1's summary and R2's bar fall inside the visible area
  and that focus never lands on progress nodes on the three screens. This moves production UI
  nodes, so run the ordinary test command from `docs/orchestration.md` § Verification.

## Out of scope

- Any change to the progress rules, parser, path derivation, recipe field or engine gate.
- A bar on base-game cards; a "detected maps" qualifier; map titles; Discover cards.
- New strings beyond what the moved/reused section needs (reuse spec 116's strings).

## Acceptance criteria

- [ ] **A1** (R1, R5) — headless test and PNG: the summary is inside the viewport at the default
  scroll on Mapset detail with a long description.
- [ ] **A2** (R2, R5) — headless test and PNG: the custom-setup card shows `N/M maps` with a bar
  inside the card's inset.
- [ ] **A3** (R3, R4, R5) — headless tests and PNGs for both setup screens (recorded with UV-Max,
  no stats yet); focus skips progress nodes; one builder serves all three screens.
- [ ] **A4** — fmt, denied-warning workspace Clippy and the ordinary test command pass.
- [ ] **A5** — PNGs reviewed by the coordinator before merge.
- [ ] **A6** — on the Deck: the three screens read well at arm's length and the right stick
  scrolls the tables. *(Needs Deck Validation → `docs/deck-testing.md`.)*

# 117 — Progress presentation polish

## Changes

- **R1:** Mapset detail now places the progress title and summary above the description,
  independent of the description, tag, add-on and extra-file content below it. Existing
  metadata, actions, source/credits and table remain present on the scroll plane.
- **R2:** Custom-setup Library cards draw the same success-colored proportional bar as catalog
  cards, beside the existing count, at the existing 448 px card height and text size. Empty,
  partial and full counts use the existing bar rules; missing/unavailable counts omit it.
  Base-game cards retain text-only counts.
- **R3/R4:** One `Nodes::progress_section` builder serves Mapset detail, base-game detail
  (opened from a Library card), and custom-setup detail. Position, width and table start are
  parameters: the summary precedes metadata/actions and the table follows them. The setup
  screens read the selected-profile progress already supplied by the snapshot; no progress
  logic or persistence changed. All progress nodes remain non-actionable, preserving the
  action IDs, D-pad navigation and Back-to-card behavior. Tables use ordinary right-stick
  scrolling, the existing UV-Max mark and Kills/Secrets columns, and existing state strings.
- No dependencies added. No `ddl-core`, catalog, spec or coordinator-owned document edits;
  merging `main` imported its existing clarification to spec 117.

## Verification

All rendering uses the existing off-screen Slint software renderer at 1280×800; no desktop
windows or screen captures were used.

Focused commands:

- `cargo test -p ddl detail_ -- --nocapture` — passed, 4 tests.
- `cargo test -p ddl progress_polish -- --nocapture` — passed, 3 tests:
  - `headless::library::progress_polish_detail_summary_and_long_table`: short and very long
    descriptions, an add-on and an extra file; default-scroll title/summary viewport bounds;
    read-only focus; right-stick scroll to the final MAP18 row; no-stats and unavailable
    summaries with no bar.
  - `headless::library::progress_polish_custom_card_bar`: empty/partial/full fill, success tone,
    non-actionability, original card height, count/bar separation, card insets and viewport
    bounds, and omitted bars for no-stats/unavailable.
  - `headless::library::progress_polish_base_and_custom_sections`: opens both screens from
    their Library cards; verifies recorded summaries, UV-Max trophies and percentage columns,
    no-stats/unavailable lines, viewport bounds, scrolling, four-direction controller action
    traversal that only lands on existing actions, and Back restoring the original card.

Final verification after merging `main` (`dfba57b`):

- `cargo fmt --all --check` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed; log:
  `target/117-clippy.log`.
- `cargo test --workspace --locked -- --skip headless::journeys::title_fit_ --skip headless::journeys::categories_favorites_and_screenshots`
  — passed (exit 0; 265 DDL unit/headless tests, with the specified 10 cases filtered;
  all workspace integration, core and doc tests passed). Log: `target/117-tests.log`.

## PNG evidence (R5 / A5)

Paths below are relative to the worktree root
`/Users/vitor/orca/workspaces/Doom-Deck-Launcher/progress-polish`.

| View | PNG |
| --- | --- |
| Mapset detail, long description, default scroll | `target/headless/progress_polish_detail_summary_and_long_table/117-detail-long-description-default.png` |
| Mapset detail, end of long table | `target/headless/progress_polish_detail_summary_and_long_table/117-detail-table-end.png` |
| Custom Library card with bar | `target/headless/progress_polish_custom_card_bar/117-custom-card-bar.png` |
| Base game, recorded summary at default scroll | `target/headless/progress_polish_base_and_custom_sections/117-base-recorded-default.png` |
| Base game, recorded table with UV-Max | `target/headless/progress_polish_base_and_custom_sections/117-base-recorded-table.png` |
| Base game, no stats yet | `target/headless/progress_polish_base_and_custom_sections/117-base-no-stats-default.png` |
| Base game, unavailable | `target/headless/progress_polish_base_and_custom_sections/117-base-unavailable-default.png` |
| Custom setup, recorded summary at default scroll | `target/headless/progress_polish_base_and_custom_sections/117-custom-recorded-default.png` |
| Custom setup, recorded table with UV-Max | `target/headless/progress_polish_base_and_custom_sections/117-custom-recorded-table.png` |
| Custom setup, no stats yet | `target/headless/progress_polish_base_and_custom_sections/117-custom-no-stats-default.png` |
| Custom setup, unavailable | `target/headless/progress_polish_base_and_custom_sections/117-custom-unavailable-default.png` |

Worker visual review: all required PNGs inspected. The summaries fit at default scroll; the
custom card count/bar remain legible inside the original card; the scrolled tables show
UV-Max and aligned percentages without progress-content overlap. The base/custom recorded
screens have separate default and scrolled evidence because their tables follow existing
actions. **Coordinator PNG review remains required before merge (A5).**

## Open questions and remaining acceptance

- No implementation questions or unimplemented R1–R5 items.
- A1–A3 have headless evidence; A4 passed all required checks. A5 requires
  coordinator review of the listed PNGs.
- **Needs Deck Validation — A6:** at handheld distance, inspect the default-scroll summary
  on Mapset detail with a long description/add-ons/extras, and on the base-game and custom
  screens. Scroll each recorded table with the right stick to its final row; check UV-Max,
  Kills and Secrets readability. Confirm D-pad only visits actions, Back returns to the
  invoking card, and custom-card empty/partial/full bars remain legible without shrinking
  text. Check both no-stats and unavailable messages on setup screens.

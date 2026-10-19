# 030 — Navigation and feedback polish

## Changes

- R1: The header derives tab availability from root navigation, modal state and gameplay. Disabled tabs use a grey label, no selected surface or underline, and no focus ring. Runtime root navigation also excludes local views/dialogs; focus IDs are unchanged.
- R2: First-run Install DSDA-Doom uses an Ember action red with the existing amber focus ring. Its focused, unfinished state slowly alternates over 1.4-second transitions (2.8-second cycle); installation, busy operations, unfocus, dialogs and gameplay stop the animation. The red action/pulse tokens have calculated label contrast of 4.64:1–5.25:1 against the dark on-accent ink.
- R3: The reusable hint controls form a compact foreground footer. A Select and B Back stay concise; runtime hint flags choose favorite/details/import/local actions, tabs, categories and Start. Dialogs hide underlying screen actions and show A/B plus Start Confirm only where applicable.
- R4: Removed the top-right app-name text; the window title and logo stay.
- R5: Discover has a fixed 22-pixel gap below the header and a clipped scroll viewport. Scroll positioning reserves space for the complete focus ring above the footer.
- R6: L2/R2 share the mapper's held-action repeat timer: immediate step, 500 ms delay, then 100 ms repeats; release/suspend stops repetition. Triggers and strip left/right wrap both ends.
- R7: A shared show-toast callback replaces the message and restarts its three-second timer, including identical consecutive messages. Expiry fades for 300 ms; tab changes clear both current and displayed text immediately. Runtime and fixture-shell toast producers use the callback.

No dependencies added. Existing D-020/D-021 tokens, bundled fonts and Ember artwork remain the visual source; this checkout has no `design/visual/` directory. No protected coordinator documents were edited.

## Verification

- A1: `headless::journeys::navigation_feedback` drives PreviousTab, NextTab and Up on FirstRun and Detail and checks that tabs cannot receive focus; it checks enabled navigation on Home, Discover, Library and Settings. Disabled tabs are visible in the attached FirstRun and Detail renders.
- A2: The same production journey renders FirstRun, Home, scrolled Discover and a quit dialog at 1280×800. All five images below were opened and visually inspected. The test additionally counts foreground A/B hint pixels on Home after gameplay and in the dialog. Header-copy removal and the fixed scrolling margin are verified visually in these production renders. Pulse eligibility is tested after installation and during gameplay; subjective animation feel still needs the Deck re-check.
- A3: `input::tests::category_hold_delays_repeats_and_release_or_suspend_stops` checks both triggers at 0, 499, 500, 599 and 600 ms, release and suspension. The existing `categories_favorites_and_screenshots` production journey now checks left/right and L2/R2 wrap in both directions, including empty Favorites.
- A4: The production navigation test advances the headless platform clock, checks old/new/identical toast replacement, expiry after three seconds, intermediate fade opacity, final zero opacity and tab-change clearing. The clock seam is additive in `headless.rs`.
- A5: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` all pass on the final implementation (532 tests, including 125 launcher tests, with zero failures). `main` was merged before final checks (already up to date at `96fe9bd`). README screenshots were regenerated with the existing headless production journeys and lossless PNG compression. Their final copies (`docs/screenshots/{home,discover,play-setup,mapset-detail}.png`) were also opened and inspected. The previous fixed-coordinate rail comparison now samples its measured new rows, accounting for both the 20-pixel viewport shift and 16-pixel focus-ring scroll.

## Inspected PNGs

Generated originals are under `target/headless/navigation_feedback/`; losslessly compressed copies are attached here for durable review:

| View | Original inspected PNG | Attached copy |
| --- | --- | --- |
| First run | `target/headless/navigation_feedback/030-first-run.png` | [First run](../screenshots/030-first-run.png) |
| Home | `target/headless/navigation_feedback/030-home.png` | [Home](../screenshots/030-home.png) |
| Discover, scrolled | `target/headless/navigation_feedback/030-discover-scrolled.png` | [Scrolled Discover](../screenshots/030-discover-scrolled.png) |
| Dialog | `target/headless/navigation_feedback/030-dialog.png` | [Dialog](../screenshots/030-dialog.png) |
| Mapset detail | `target/headless/navigation_feedback/030-detail.png` | [Detail](../screenshots/030-detail.png) |

## Open items

**Needs Deck Validation:** re-check S4 and S2 in Gaming Mode, including trigger repeat feel, focused pulse subtlety and toast timing. No implementation requirements remain open; no DSDA-Doom, AppImage, Steam or WAD compatibility facts were introduced.

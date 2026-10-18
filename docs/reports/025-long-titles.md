# 025 — Long mapset titles fit their slots

## Result

R1–R3 implemented with the Ember Cathedral fonts, materials, frames, artwork and controller
navigation retained. All 13 embedded recipes and a synthetic 60-character title are exercised
at 1280×800 by the off-screen Slint software renderer. No visible window was opened and no
screen was captured. New dependencies: none.

## Audit (R1)

Every recipe was rendered in Home (hero and Recently played), Discover, Library, Mapset detail,
Play setup, install progress, the profile picker and uninstall confirmation: 112 before frames
and 112 after frames including the synthetic recipe, plus the component frames described below. The longest catalog title is
**Back to Saturn X E2: Tower in the Fountain of Sparks**, 52 characters. The synthetic title is
`ABCDEFGHIJKLMNOPQRSTUVWXYZ ABCDEFGHIJKLMNOPQRSTUVWXYZ ABCDEF` (60 characters), retaining the
Alien Vendetta author list to exercise a long credit independently of the title.

Confirmed by headless renders and the bundled-font measurements:

| Slot | Cases before the fix |
| --- | --- |
| Home hero title | Back to Saturn X E1 and E2: 93 px preferred wrapped height in a 60 px slot; the second line was cut through its glyphs. Synthetic: 139 px in 60 px. |
| Home hero author | Alien Vendetta, both Back to Saturn X episodes, Eviternity, Eviternity II, SIGIL II; multiline text was clipped in a 36 px slot. Synthetic inherits Alien Vendetta's author list. |
| Recently played title | Both Back to Saturn X episodes and the synthetic title require multiple lines in a single-line slot. All 13 recipes also had a 34 px preferred line in a 32 px slot; short-title glyphs mostly appeared intact, but the line box was undersized. |
| Discover / Library title | Both Back to Saturn X episodes: 67 px in 42 px; synthetic: 101 px in 42 px. |
| Play setup / install progress context heading | Back to Saturn X E2: 801 / 892 px natural width in 784 px. Synthetic detail / setup / progress: 986 / 1031 / 1122 px in 784 px. |
| Detail title | Current compact catalog titles fit, but parenthesized qualifiers were silently removed for Alien Vendetta, Ancient Aliens, AUGER;ZENITH, Eviternity and Eviternity II. Synthetic: 103 px in 60 px. |
| Uninstall title | Catalog titles fit. Synthetic confirmation needs 67 px in the original 64 px title slot. |
| Card/detail authors | Original character-count truncation could wrap to two lines on cards; no additional clipped catalog glyphs found in the old slots. Cards now use one measured, elided line. |
| Profile picker / art band | Neither contains a mapset title or author line. The picker shows the active profile name; the band carries the screenshot credit. Both were rendered for every recipe and retain their existing content. |

The raw baseline measurement list is `target/headless/025-before/025-audit.txt`; the supplemental
context-heading width measurements are `target/headless/025-after/025-fit.txt`. Preferred sizes
include line spacing, so a line-box shortfall is distinguished above from visibly severed glyphs.

## Fitting (R2)

- Shared Slint `FittedText` measures the actual bundled face at the assigned width, tries
  wrapping at the original scale, reduces by exactly one scale step if needed, then uses
  Slint's elision. Title slots cap their rendered height at one or two measured lines.
- Home hero: 36→26 px, two lines in 68 px. History: 26→22 px, one line in 34 px; short titles
  keep 26 px. Cards: 26→22 px, two lines in 68 px, followed by one 26 px author slot.
  Author data is no longer prematurely shortened by character count.
- Context headings: 36→26 px, one line. Mapset confirmation / profile sheet headings use
  26→22 px. Other confirmation sheets retain their existing multi-line space.
- Detail renders the **complete** recipe title, including parenthesized qualifiers, at
  44→36 px in a 96 px slot with no elision. The author and description move down only enough
  to accommodate it. The shorter description slot elides its last complete line, avoiding a
  new clipped line or overlap with tags.
- A 4 px horizontal glyph inset protects Pirata One's side bearings: the first pixel-bound
  test caught an initial `A` extending one pixel left of its nominal text origin.

## Regression coverage (R3)

`headless::fixture_screens_navigation_and_focus_render_headlessly` calls
`production::title_fit_journey` on the existing single headless platform. It uses production
runtime nodes and sheet composition, asserting the expected number of title/author slots,
one/two-line limits, space for at least one complete line, one-step scale bounds, title separation from the following row, the exact
full detail title and its measured preferred height, and actual software-rendered ink bounds.
The isolated pixel fixture uses the same `FittedText` component as production; it must paint
nonempty text and cannot paint outside its assigned rectangle. Catalog iteration is dynamic,
so new recipes join the fit test automatically. Sheet opening uses a render-only seam rather
than saving profiles or executing uninstall commands.

The obsolete character-count author helper and its unit test were replaced by measured-font
coverage of every recipe's author line. The fixture encoder uses the existing PNG crate's fast
lossless deflate backend, keeping adaptive-filter scanlines unchanged; all ten focused
synthetic PNGs byte-match after the README script's normalization against the previous encoder.
No new dependency or pixel change was introduced by that test-only optimization.

The first synthetic detail check failed at 88 px; increasing the slot to 96 px made the full
second line fit at the permitted 36 px step. This is an intrinsic-height assertion, independent
of the outer clipping guard.

Fast focused reruns (no windows):

```sh
DDL_TITLE_ONLY=1 DDL_HEADLESS_DIR="$PWD/target/headless/025-after" \
  cargo test -p ddl fixture_screens_navigation_and_focus_render_headlessly
# Optional: add DDL_TITLE_RECIPE=synthetic-60 to iterate on one recipe.
```

`DDL_TITLE_AUDIT=1` records intrinsic measurement failures without requiring fit assertions;
the baseline frames were generated with the original presentation before applying the fit.

## Visual evidence (A1)

Before/after files share the name `025-<screen>-<recipe-id>.png`, under
`target/headless/025-before/` and `target/headless/025-after/`. Screens are `home`, `discover`,
`library`, `detail`, `setup`, `progress`, `profiles`, `uninstall`. Examples for the clipped cases:

- `025-home-back-to-saturn-x-e1.png`, `025-home-back-to-saturn-x-e2.png`: hero, author and history.
- `025-home-alien-vendetta.png`, `025-home-eviternity.png`, `025-home-eviternity-ii.png`,
  `025-home-sigil-ii.png`: long author lines.
- `025-discover-back-to-saturn-x-e1.png`, `025-discover-back-to-saturn-x-e2.png` and the same
  `025-library-*` names: two-line card titles and single-line authors.
- `025-setup-back-to-saturn-x-e2.png`, `025-progress-back-to-saturn-x-e2.png`: context headings.
- `025-detail-auger-zenith.png` and the other formerly abbreviated titles: full qualifiers.
- `025-<screen>-synthetic-60.png`: the longer fixture in all eight screen/sheet compositions.

I inspected before/after crops for all 13 recipes plus the synthetic title in all eight
compositions (`025-montage-<screen>.png`, local render artifacts), full frames for the clipped
Back to Saturn X cases and synthetic detail, and all four regenerated README PNGs.
The contact sheets are crops of software frames, not desktop captures.

## Approved scope addition — component labels

The coordinator explicitly added independent recipe `Component.label` values to Play setup
coverage during the task. No recipe labels were changed. All **28** catalog component labels
are exercised, including hidden advanced replacements and exclusive-choice members; the
synthetic recipe also puts its 60-character string into its main component label.
The longest catalog component label is **AUGER;ZENITH (Doomer Boards Project 37)** (39 characters).
It fits its existing slot. The synthetic label required 53 px of wrapped Barlow text in the
row's 30 px title area, clipping both lines and approaching the locked-component explanation.

Component rows now opt into measured 22→20 px fitting on one line. Their current label area
cannot hold two complete body lines, so the final fallback is elision; row heights and the
separate explanation remain intact. Exclusive-choice labels use the fitter at the minimum
20 px scale, retaining vertical centering and a single elided line. Visual review caught blank
exclusive-choice labels during the first extension pass. The fix
keeps their choice rendering separate from body rows; the regression check now also requires
visible text pixels in each actual production row/button, alongside the isolated fitter check.
The shared test checks every component label's measured line and painted bounds and writes
`025-component-<recipe-id>-<node-id>.png` at each component's scroll position.

Inspected supplemental before/after evidence:
`target/headless/025-component-before/025-component-synthetic-60-100.png` and
`target/headless/025-component-after/025-component-synthetic-60-100.png`.
The original `025-before/025-setup-auger-zenith.png` records the longest shipped label;
final after renders include `025-component-auger-zenith-100.png`. There are 30 additional
component frames (28 catalog labels plus the synthetic recipe's two labels).

## Final verification (2026-10-18)

Merged then-current `main` (`14d50c2`, including spec 026 phase A) without conflicts in
`e905a87`. Phase A was already present through the earlier `1382b8a` merge; this final merge
only brought coordinator-owned status/journal history. No fitting code needed another change.
`main` subsequently advanced with spec 024 while the checks ran; that later merge is outside
this verified tree and can be reconciled by the coordinator during review.

Ran each required final check once, sequentially, on macOS arm64:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
DDL_HEADLESS_DIR="$PWD/target/headless/025-final" cargo test --workspace
```

All exited 0. Logs are local artifacts in `target/spec-025-final/{fmt,clippy,tests}.log`.
Formatting produced no output; clippy's tail was:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.76s
```

The workspace run passed **495 tests**, zero failures, including all 13 catalog recipes,
the synthetic title, and the component-label regression checks. Selected output tails:

```text
test headless::fixture_screens_navigation_and_focus_render_headlessly ... ok
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 222.27s
test result: ok. 371 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The README screenshots had already been regenerated using the documented
`python3 scripts/readme-screenshots.py` command in the prior implementation pass. All four
were inspected again during completion. Applying that script's `optimized_png` normalization
to the fresh final-run frames byte-matched the existing regenerated files: Home 361,489 bytes,
Discover 555,298 bytes, Play setup 344,934 bytes, Mapset detail 572,467 bytes. No additional
regeneration was needed after the documentation-only merge.

Completion review also re-inspected the Back to Saturn X E2 Home before/after frames, its
Discover after frame, synthetic full detail, detail/setup/uninstall contact sheets, and the
synthetic component before/after frames. The final run's fresh render evidence and
`025-fit.txt` are under `target/headless/025-final/`. These are off-screen software renders.

## Open items

A1–A3 are complete. No spec 025 requirements or scope questions remain open. No dependencies
were added. Linux CI and physical Deck validation were not performed in this macOS worktree;
this task verifies text fitting at the specified 1280×800 resolution headlessly.

## Review fix — default artifact directory (2026-10-18)

Review found that the text-report writer used a working-directory-relative
`target/headless`, unlike the PNG writer. The earlier verification with
`DDL_HEADLESS_DIR` set did not cover this default-path failure on a clean checkout.

Commit `924b95c` extracts `headless_dir()` from the existing PNG writer. Both PNGs and
`025-fit.txt` / `025-audit.txt` now use it: honor `DDL_HEADLESS_DIR`, otherwise resolve
`CARGO_MANIFEST_DIR/../../target/headless`, and create the directory before writing.
The spec 025 artifact-write audit found only the PNG path and this conditional report
writer; `025-before/` and `025-after/` are caller-selected output directories, not
additional hardcoded write paths. No dependencies were added.

Merged current `main` (`bcbcd83`, including spec 024) without conflicts in `eb4752e`.
Before verification, moved the prior `target/headless` tree to
`target/spec-025-review-fix/previous-headless` and asserted the original path did not exist.
Earlier local evidence paths in this report now live under that archive. This preserves the
old renders while providing the same absent-directory starting condition as deletion.

Final review-fix verification ran sequentially with `DDL_HEADLESS_DIR` explicitly unset:

```sh
env -u DDL_HEADLESS_DIR cargo fmt --all --check
env -u DDL_HEADLESS_DIR cargo clippy --workspace --all-targets -- -D warnings
env -u DDL_HEADLESS_DIR cargo test --workspace
```

All exited 0. Formatting was silent; clippy completed in 12.72s. The workspace passed
**499 tests**, zero failures, including the full headless journey in the default output
configuration. Selected tails (`target/spec-025-review-fix/{fmt,clippy,tests}.log`):

```text
test headless::fixture_screens_navigation_and_focus_render_headlessly ... ok
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 184.83s
test result: ok. 375 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Confirmed the run created a nonempty `target/headless/025-fit.txt` beside the PNGs and did
not write a crate-relative `crates/ddl/target/headless/025-fit.txt`. This full existing
journey covers the reported regression without environment mutation inside parallel tests;
the audit report uses the same helper and conditional write site. A1–A3 remain satisfied;
no review-fix items remain open.

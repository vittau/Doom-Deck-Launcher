# 107 — Clear recipe variants and recipe-controlled compatibility

## Outcome

**Confirmed in source and focused offline tests:** replaced catalog setup's ambiguous Advanced
row with Show/Hide recipe variants. Recipes without hidden alternatives retain a visible,
greyed-out row reading No recipe variants available, with no click handler or controller focus
stop. The available row expands actual recipe alternatives and focuses the first revealed
choice so it is immediately fully visible. Selecting a replacement retains the existing
player confirmation, initially focused on Keep current settings (the cancellation action).
No external approval or permissions process is involved.

The complevel value now says Set by recipe; blocked setup screens also explain that the
recipe controls it. Custom setup Compatibility still offers Auto and numeric choices.
The guide explains these separate flows. No recipe, component-selection policy, engine,
launch setting, dependency, downloaded file, historical state fixture or report106 changed.
No generic catalog complevel override was introduced.

## Evidence by requirement

| Requirement | Evidence |
| --- | --- |
| R1 — disabled available-state presentation | Shared `is_recipe_variant` predicate is exactly `replaces.is_some() && !user_supplied`, used for hiding components, row availability and activation. Runtime uses the effective `Item.recipe`, preserving the existing installed-snapshot selection. Disabled rows retain ordinary row rendering with `actionable = false`, which uses the existing greyed-out style, emits no TouchArea and excludes them from navigation. A guard also rejects a stale direct controller activation. The new headless test checks visibility, help, non-actionability, click/Confirm rejection and Up/Down skipping. |
| R2 — clear alternatives and confirmation | Translatable Show/Hide labels and Recipe alternatives / player-confirmation help replace Advanced. Removed the old forced 64-pixel height; the normal row reserves 80 pixels for its readable label and subtitle. Expansion focuses the first matching component, while collapse leaves focus on the toggle and preserves selected profile components. The new test checks the revealed row is completely inside the scrolling viewport, checks safe default focus, confirm-on-safe-button cancellation, Back cancellation and explicit acceptance. Existing production replacement/exclusivity journey also passes. |
| R3 — recipe-controlled complevel | Ready-state copy includes the resolved alias/number and Set by recipe. Blocked states include Complevel is set by the recipe. The synthetic installed/launchable production test asserts `Complevel: Vanilla (3) · Set by recipe` and renders it; the new blocked-state test asserts the explanatory note. Existing custom creation/IWAD/Auto/numeric compatibility journey passes unchanged. |
| R4 — guidance | The guide explains expansion, the safe confirmation action, disabled rows, separate purchased-file imports, recipe-controlled complevel and direct Auto/numeric choices in custom setups. No unsupported override or arbitrary-mod compatibility is promised. Physical checks added to the Deck checklist. |

**Confirmed current source inventory:** scanning `catalog/*.toml` with the same predicate finds
only `sigil/compat` as a hidden free replacement. `sigil/soundtrack` and
`sigil-ii/soundtrack` are user-supplied replacements, excluded from this toggle. This is a
source inventory, not a claim about compatibility or a hardcoded title allowlist. The test
also removes free alternatives from the effective SIGIL snapshot while leaving paid imports,
and verifies the row disables; thus availability follows data rather than a recipe ID.

## Verification

Created `vittau/recipe-variant-clarity` from main after spec106 was accepted and merged.
Merged main before final checks (already up to date). Shared warm target:
`/Users/vitor/git/Doom-Deck-Launcher/target`.

Passed:

- `cargo fmt --all --check`.
- `cargo clippy -p ddl --all-targets --locked -- -D warnings`.
- `cargo test -p ddl --bin ddl recipe_variants_107 --locked` — 1 focused journey.
- `cargo test -p ddl --bin ddl controller_057 --locked` — 3 navigation/scroll/root journeys,
  including Skill/Pistol start/load-order traversal.
- `cargo test -p ddl --bin ddl controller_060_rows_are_entered_at_their_leftmost_item --locked`
  — 1 row-entry journey, updated to skip Auger Zenith's disabled variant row.
- `cargo test -p ddl --bin ddl install_progress_and_play_setup --locked` — 1 synthetic ready-state
  production journey with actual complevel copy.
- `cargo test -p ddl --bin ddl play_setup_options_and_engine_exit --locked` — 1 existing production
  replacement/confirmation/exclusivity journey, using the fake engine runner.
- `cargo test -p ddl --bin ddl local_new_setup_iwad_compatibility_and_content_filtered_multiselect --locked`
  — 1 custom Auto/numeric compatibility regression journey.
- `git diff --check`; changed-document relative Markdown links resolve.

Eight affected test cases passed. No ordinary workspace suite replay: spec106's 995 passing
tests and exactly ten documented exclusions remain the authoritative broader evidence for
unchanged behavior. No title matrix, network e2e, SDK/package build or actual game launch.
The production fixtures use scratch XDG directories, synthetic content, loopback test responses
and a fake engine runner; no external service or game data was required.

## Reviewed headless frames

All five are 1280×800 production Slint software renders, inspected offscreen by the worker
and accepted by the coordinator. No visible window or desktop capture was used. The first
expanded frame left COMPAT above the viewport; it was corrected by focusing the revealed
alternative and then regenerated with an explicit viewport-bound assertion.

Absolute paths:

- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/local-content-polish/target/headless/recipe_variants_107_disabled_available_and_confirmed_states/107-no-variants.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/local-content-polish/target/headless/recipe_variants_107_disabled_available_and_confirmed_states/107-variants-collapsed.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/local-content-polish/target/headless/recipe_variants_107_disabled_available_and_confirmed_states/107-variants-expanded.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/local-content-polish/target/headless/recipe_variants_107_disabled_available_and_confirmed_states/107-variant-confirmation.png`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/local-content-polish/target/headless/install_progress_and_play_setup/107-recipe-complevel.png`

Raw focused-check logs: `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/local-content-polish/.scratch/107/`.

## Remaining work

**Needs Deck Validation:** physical controller feel, disabled-row contrast at LCD scale,
scroll/focus when opening SIGIL's alternatives, and readability of the recipe-controlled
complevel note. No implementation requirement remains open. No push, tag or release.

## Coordinator acceptance

Accepted and merged locally as `b537833` after source review and all five final production
frames. The expanded alternative is fully visible and focused; disabled rows are visibly
inactive and skipped; ready-state complevel wording fits. The merged app/core source matches
the worker's checked tree byte-for-byte, so focused checks were not replayed after this
conflict-free merge. Logs and frames are retained at
`/Users/vitor/git/Doom-Deck-Launcher/.scratch/107/evidence/`. No release was made.

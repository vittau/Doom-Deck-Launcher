# 114 — Launch at DSDA-Doom's menu

## Changes

- Ordinary catalog, base-game and custom-setup resolution leaves both skill and warp unset. Home Continue and `ddl launch` use those same resolution paths. Pistol start still produces exactly one `-pistolstart` when enabled. The remaining arguments retain their order, including compatibility, files/patches, isolated directories and transient overrides.
- `LaunchPlan` retains optional explicit skill and warp for low-level recording plans. Existing argument-order and recording argv tests retain explicit skill when requested.
- Removed the profile skill field, its validation/default, CLI `--skill`, UI controls, focus stops, copy and summaries. Pistol start fills the former two-option row in Play setup and base-game setup; controller tests cover the surviving row.
- Kept profile schema version 1 and atomic persistence. A deserialization-only document accepts and discards the retired `skill` field, while other unknown fields still fail. Existing release fixtures remain unchanged; the fixture generator no longer assigns a removed field.
- Catalog lint rejects every runtime warp, including empty and otherwise valid values, with a D-044 diagnostic. Resolution ignores legacy recipe/install-record warp; change classification ignores warp-only differences, with no change prompt. Updated recipe and contributor comments.
- No new dependencies, migration routines, configuration keys or explanatory UI copy.

## Verification

Merged local `main` (`55fd127`, stats research only) before final checks. All Rust builds used `--jobs 2`, with one Cargo build at a time.

| Check | Result / real output |
| --- | --- |
| `cargo fmt --all --check` | Exit 0, no output. |
| `cargo clippy --workspace --all-targets --jobs 2 -- -D warnings` | Exit 0: `Finished dev profile [unoptimized + debuginfo] target(s) in 33.45s`; no warnings. |
| `cargo test -p ddl-core --jobs 2` (focused iteration) | `652 passed; 0 failed` in the core unit group, with all integration groups and doctests passing. The final workspace run also includes the added strict-unknown-field guard. |
| `cargo test -p ddl --jobs 2 cli::tests::launch` | `2 passed; 0 failed`. |
| `cargo test -p ddl --jobs 2 controller_0` | `4 passed; 0 failed`; covers D-pad traversal through the remaining full-width option and Back/tab behavior. |
| `cargo test -p ddl --jobs 2 headless::journeys::profiles_isolation_and_continue` | `1 passed; 0 failed`; actual fake-runner argv for Home Continue with pistol start On/Off and profile directories preserved. |
| `cargo test -p ddl --jobs 2 launch_114_profiles_and_setup_have_no_skill` | `1 passed; 0 failed`; four PNGs below. |
| `cargo test -p ddl --jobs 2 headless::shell_navigation_and_focus_render_headlessly` | `1 passed; 0 failed`; verifies the equivalent derived fixture defaults after Clippy requested that simplification. |
| Required ordinary workspace test command | Exit 0: **1,019 passed**, zero failed/ignored across 23 unit/integration/doctest groups; DDL unit group `257 passed; 0 failed; 10 filtered out`, core unit group `653 passed; 0 failed`. |
| `target/debug/ddl launch --skill 4` | Exit 2: ddl: unknown option `--skill`, followed by `Try 'ddl --help' for a list of options.` |
| `target/debug/ddl catalog lint catalog` | Exit 0: `Catalog lint passed (offline).` |

The ordinary workspace command was:

```sh
cargo test --workspace --locked --jobs 2 -- --skip headless::journeys::title_fit_ --skip headless::journeys::categories_favorites_and_screenshots
```

The first denied-warning Clippy run found only `derivable_impls` in the test-only `ProfileView` default: after removing skill, every remaining field uses its type default. Replaced the equivalent manual implementation with `#[derive(Default)]` and verified the headless shell journey above; no production behavior changed after the ordinary suite. The final Clippy result is recorded in the table above.

Acceptance coverage:

- **A1 passed:** catalog resolver, base-game builder and custom builder each inspect actual argv with pistol start On/Off. No `-skill`/`-warp`; `-pistolstart` occurs exactly once only when enabled. CLI dry-run and actual Home Continue launch argv also cover both choices. Existing exact argument-order and recording tests pass.
- **A2 passed:** production node assertions and inspected renders contain no skill setting/summary; real CLI unknown-option error above.
- **A3 passed:** the existing release-state fixture trees load, and the fixture-backed v1 profile store test loads `skill: 3`, atomically saves, and checks that JSON has no `skill`. Invalid retired values are ignored as well; other unknown fields remain rejected. No schema bump is necessary because this is an explicitly ignored retired field.
- **A4 passed:** lint rejects empty, valid and out-of-range warp arrays with D-044; the real catalog lints clean. Legacy resolver warp is ignored, and warp-only differences classify as Current.
- **A5 passed:** fmt, denied-warning workspace Clippy and the ordinary workspace test command all pass.

## Headless evidence

The production renderer is exercised through Slint's `MinimalSoftwareWindow` and software renderer; no desktop window or screen capture is used. The focused `launch_114_profiles_and_setup_have_no_skill` case asserts no skill text in node labels/details and writes these 1280×800 PNGs under `target/headless/launch_114_profiles_and_setup_have_no_skill/`:

- `target/headless/launch_114_profiles_and_setup_have_no_skill/114-play-setup.png`
- `target/headless/launch_114_profiles_and_setup_have_no_skill/114-base-game-setup.png`
- `target/headless/launch_114_profiles_and_setup_have_no_skill/114-custom-setup-profile.png`
- `target/headless/launch_114_profiles_and_setup_have_no_skill/114-mapset-detail.png`

All four PNGs were opened and visually inspected. The surviving Pistol start control fills the available row in both setup views; focus borders and adjacent controls fit. Mapset detail shows `Pistol start Off` and the add-ons/extra-files summary without a skill row. The custom setup profile view has no skill text or control. The ordinary suite also regenerated the installed Play setup and detail/profile journey evidence.

## Open questions and follow-up

- **A6 — Needs Deck Validation:** confirm menu/title startup in Gaming Mode, New Game skill selection in DSDA-Doom, pistol-only inventory on the second map with pistol start enabled, and Home Continue opening the menu with Load Game working. The upstream evidence and pinned links remain in spec 114; this task makes no claim of physical Deck validation.
- Coordinator-owned documentation proposals: update `docs/deck-testing.md` with A6 and replace its old U7 skill-setting check; update `docs/status.md`/`docs/journal.yml` with the implementation and verification outcome. `OVERVIEW.md` still mentions launcher difficulty selection and should reflect D-044 if the coordinator updates it.
- Outside this task's ownership, `docs/steam-deck-guide.md:530` still advertises `ddl launch --skill`; remove that option in a documentation follow-up. Historical design mockups also show the old skill setting; production source and headless renders are authoritative for this change.

# 032 — Base games and IWAD sources

## Result

Implemented original-game targets derived from the identified IWAD registry, without catalog recipes, downloads, or installation records. Stable target ids are `base:doom`, `base:doom2`, `base:tnt`, and `base:plutonia`; registered, Ultimate and shareware Doom share the identified `doom` game family. Nerve is excluded because it is an expansion PWAD, as confirmed by the coordinator; players may add it as a Doom II Extra file.

Base launches reuse spec 026's local launch builder and spec 003's separate-argument `LaunchPlan`, including canonical IWAD links, identity checks, managed configuration, skill, pistol start, ordered WAD/ZIP and DEH/BEX extras, and per-profile save/data/shot/run directories. Base targets cannot collide with recipe or custom ids because those reject `:`. Profile filenames remain safe and use the existing global store, checking both documents and orphan runtime directories before allocating an id.

**Confirmed:** vanilla complevels are Doom 3, Doom II 2, and TNT/Plutonia 4, per [DSDA-Doom v0.30.1 settings.c L59–75](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L59-L75), recorded in `docs/research/iwad-identification.md` §3. Registry classic-copy preference is preserved. Play setup can choose another identified physical copy, including a second copy with the same variant; unavailable selections fall back to the preferred remaining copy with a visible notice.

Merged local main through spec 031 and reused its shared free-text name editor, case-insensitive `profiles::validate_name`, rename command and Steam keyboard callback for base-profile creation and renaming; no second keyboard path was added. Library has a Base games group; each game has an Original game page and Play setup. Home combines base games, mapsets and custom setups in recent history. Settings lists automatic Steam, GOG/Heroic/minigalaxy/offline and opt-in standard source folders, including inspected empty folders, with labels and games found, followed by player folders. Automatic rows can receive controller focus for scrolling and show their path through the existing toast; only player folders offer removal. `ddl doctor` includes base games and their profiles; `ddl launch base:<game>` supports the existing dry-run, profile, skill, pistol-start and accept-changes controls.

## Persistence and dependencies

No new dependencies. No app-state or profile schema bump: the existing versioned, atomic stores are reused. The coordinator approved additive `Profile::iwad_path: Option<PathBuf>` with serde default and omission when unset; existing profile documents load unchanged, and a test checks that ordinary recipe profiles omit the new field. `deny_unknown_fields` remains by coordinator instruction: downgrading to an older binary cannot load base-game profiles that have a selected physical copy; recipe and custom profiles do not set the field.

## Verification

- **A1 / R1, R3:** `iwads::steam::tests::base_games_from_deck_layout_prefer_classic_and_support_kex_only` reuses spec 028's generated Deck layout, resolves its synthetic ambiguous identities, asserts exactly four games with 3/2/4/4, classic `base/` picks, and a rerelease-only Doom II. No commercial IWAD data is included.
- **A2 / R2, R4:** `iwads::base_games::tests::base_plan_has_vanilla_argv_and_isolated_profiles_and_extras` checks Doom II `-iwad`, `-complevel 2`, no `-file`, separate save paths, same-variant physical-copy selection, skill/pistol start, patch extras, missing-extra refusal and missing-copy fallback. The headless `base_games_library_page_setup_sources_and_recent_history` journey checks Library, page, compact Play setup, copy selection/fallback, profile creation/selection, shared free-text renaming and duplicate-name rejection, extras navigation and Home history. The headless backend renders at 1280×800 and never opens an OS window.
- **A3 / R5:** the same headless journey renders Settings with Steam sources and one player folder, asserts only one Remove action, and verifies selecting an automatic source never opens Remove. Core tests cover empty automatic Steam/Heroic/standard folders. Existing IWAD discovery tests remain green (71 tests).
- **A4 / R6:** `cli_exec::tests::base_game_doctor_and_launch_share_the_original_game_plan` checks doctor output, Doom II dry-run argv, actual fake-runner launch, persisted per-game history, and no custom setup record.
- **A5:** after merging main through spec 029 (`e45c776`), `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` are green (ddl 139, ddl-core 387, plus the integration suites). The merge exposed one regression of this spec: with a base game present, Library's first focus fell on the Base games card, so the spec 010/015 favorite journeys (X on the first mapset) failed in seven headless journeys. Library now gives first focus to the first installed mapset, then the first custom setup, and only then to a base game; base games stay reachable by navigation.

Inspected PNG evidence is generated under `target/headless/base_games_library_page_setup_sources_and_recent_history/` (intentionally untracked): `032-library-base-games.png`, `032-base-game.png`, `032-base-play-setup.png`, `032-base-copy-fallback.png`, `032-iwad-sources.png`, `032-home-base-game.png`, and `032-base-profile-name.png`. The initial Play setup render exposed controls below the fold; it was compacted and re-rendered so all controls fit at 1280×800.

## Remaining validation

**Needs Deck Validation:** play Ultimate Doom and Doom II from Library on the physical Steam Deck, including controller focus after the child engine exits. Automated tests use generated WADs and a fake engine runner and cannot establish actual DSDA gameplay or Gaming Mode focus behavior.

Possible follow-up: a first-class No Rest for the Living expansion entry; it is deliberately outside this original-IWAD task. No changes are proposed to the protected product/status/journal files beyond recording completion and the Deck re-check when the coordinator is ready.

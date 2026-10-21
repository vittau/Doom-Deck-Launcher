# 064 — Direct Play actions, UI spacing and detected engine resolution

**Status:** implemented on `vittau/button-actions` (worker report) · **Spec:** `docs/specs/064-button-actions-and-display.md` · **Owner brief recovered 2026-10-21** · **Merged `main` at `83e35d4` (includes `0dd4749`)**

Scope note: this task did **not** touch spec 067 / audio, `docs/decisions.md`, `docs/status.md`,
`docs/journal.yml`, workflows or packaging. The 067 worker privately merged this branch's
`83e35d4`; later 064 commits must be union-merged into 067 before integration.

## Requirements

### R1 — Direct Play from Home and Detail

- Home `Continue` now dispatches `Command::Play` for the remembered catalog item and profile.
  The same action is used for the base-game and custom-setup latest entries (`LocalCommand::Play`),
  after their file-identity checks. Missing IWAD, missing/changed user file, missing engine and an
  unusable DSDA config fall back to Play setup, where the existing actionable blocker is shown;
  nothing is launched in those cases.
- Detail's main action (id 1) is labelled **Play** and dispatches `Command::Play`; **Edit** (id 6)
  still opens Play setup at the profile row. An uninstalled mapset keeps the install action.
- After a direct launch, DDL returns to the screen that started it (Home/Detail), not to Play
  setup; a launch started in Play setup still returns to Play setup. This also applies to the
  engine-failure toast path.
- Validation, the remembered-profile selection (`state.last_profiles` → `Services::profile`),
  `reconcile_install`, the run guard and profile `last_played` updates are unchanged.

### R2 — Red only for game launches

- Only kind-2 actions whose label is **Play** or **Continue** pass `primary = true`; all other
  primary flags (New custom setup, Settings sections and actions, Install/Install engine,
  Discover/Show all, Retry, Reset, Add folder, Add files, Default profile, Use game, …) are now
  false. Chips keep their independent green selected border; amber remains focus-only.
- A headless journey asserts across Home, Discover, Library, Settings, Detail and Setup that every
  red `kind == 2` action is `Continue` or `Play` (`064-direct-play-running.png`,
  `064-detail-play.png`).

### R3 — Base-game menu/title art

- Four new bundled assets, recorded with source, credit and retrieval date in each
  `SOURCES.toml` (see `crates/ddl/ui/assets/screenshots/base-*/`):
  - `base-doom/01.jpg` ← `https://doomwiki.org/w/images/0/04/UltimateDoom_title.gif`
  - `base-doom2/01.jpg` ← `https://doomwiki.org/w/images/5/51/Doom2_title.png`
  - `base-tnt/01.jpg` ← `https://doomwiki.org/w/images/e/ed/TNT_title.gif`
  - `base-plutonia/01.jpg` ← `https://doomwiki.org/w/images/0/0c/Plutonia_title.gif`
- These are the games' menu/title art, sourced from the Doom Wiki, under the owner's explicit
  narrow exception to D-020 (recorded in the spec); credit is "id Software" and each file records
  that it is removed on the rights holder's request. doomwiki.org returns Cloudflare 403 to
  automated clients, so the files were read from Wayback Machine snapshots (secondary evidence,
  like the existing SIGIL II screenshot research); URLs and the retrieval date are recorded.
- The art is used on the base-game Library cards, the Home hero, base detail/setup panels and the
  top band. `screenshots.rs` maps `iwads::Game` to the asset and the asset test proves all four
  files exist and that a credit is recorded. Base detail also shows
  "Base-game menu art: DoomWiki.org" as the visible attribution, preserving D-024-style credit
  after the About list was removed (R5).

### R4 — Footer, card padding, header inset

- The footer `Start` hint is **Quit** (`LocalStrings.quit`, asserted in the Settings journey);
  local pickers still show **Add selected** while selecting.
- Library/mapset cards gained an internal gutter: art at +20 px, title/author/status/tags at
  +24 px with 336 px content width (custom cards and base-game cards follow the same rule). A
  headless assertion checks every card text/tag node inside a card starts at least 20 px from its
  left edge (`010-library.png`).
- The current tab's content title (and its status line) moved from x = 40 to x = 52 px, with the
  width reduced by 12 px so the art band still ends at the same place. This is a Slint layout
  change; verified visually in `010-home-continue.png` / `013-about.png` (no DOM node exposes it).

### R5 — Home, About

- Home keeps only the latest game (catalog, base or custom) and the readiness panel; the lower
  "Recently played" list, its rows and its empty state are gone. History storage is unchanged.
- About no longer renders the long screenshot-credit list (`SCREENSHOT_CREDITS` is now test-only
  provenance). It shows **Made by Vitor Machado** and
  `https://github.com/vittau/Doom-Deck-Launcher`, above the engine/sandbox/status-bar licence
  lines so both are visible without scrolling. Mapset detail keeps its per-recipe screenshot
  credit, and base detail shows the DoomWiki credit line (R3).

### R6 — Automatic aspect and detected fullscreen resolution

- Template v5 sets `render_aspect = 0` (Auto; `v_video.c:1357-1376`) instead of the fixed 16:10
  value 2. The three-way migration updates an untouched value and keeps a player-chosen aspect
  with a conflict (tests below).
- The detected drawable size is passed per launch as transient, separate argv tokens
  `-width W -height H` after `-shotdir` and before `-assign`. Both flags are confirmed integer
  options (`args.c:536-545`), read in `I_InitScreenResolution` after the config's
  `screen_resolution`, so they change only the current session (`i_video.c:1050-1062`). The
  persisted `screen_resolution "1280x800"` remains the fallback; no player config is rewritten
  for display detection. Research §1 now records both flags with citations.
- Detection: `ui.window().is_fullscreen()` and `ui.window().size()` (physical/drawable pixels;
  `display.rs` already documents that Slint reports physical pixels and gamescope maps them to
  the output). Windowed mode, an unavailable size or a size outside DSDA's minimum (320×200) and
  DDL's ceiling (16 384) yields `None` and keeps the config fallback. Logical UI coordinates are
  never scaled into the engine value.

### R7 — Hardware renderer default

- Template v5 sets `videomode = "OpenGL"` (string key). Confirmed values: `"gl"`/`"OpenGL"`
  select the OpenGL path, every other string (including the upstream `"Software"` default)
  selects software; the Video Settings menu lists exactly `Software`/`OpenGL`
  (`i_video.c:1003-1013`, `m_menu.c:3463-3468`, `configuration.c:1165-1168`).
- The new `[upstream_strings]` table records `videomode = "Software"` so the migration can tell
  DSDA's own untouched value from a player choice. Untouched `Software` migrates to `OpenGL`;
  any other explicit spelling (e.g. `gl`) is kept and reported as a conflict. The engine setting
  only; Slint's renderer is untouched.
- Owner measurement (reported 2026-10-21, recorded in D-037): Deck on a 1080p/120 fps TV,
  software CPU power about 4–5 W, hardware about half or less. It is CPU power for that
  scenario, not total system power or a battery-life multiplier; the report does not promote it
  to measured facts. Deck validation remains open.

## Tests and verification

Commands run with the coordinator's limits (`pipefail`, `CARGO_BUILD_JOBS=2`, `-j 2`,
`RUST_TEST_THREADS=2`, `--test-threads=2`):

```sh
cargo fmt --all --check
CARGO_BUILD_JOBS=2 cargo clippy -j 2 --workspace --all-targets -- -D warnings
RUST_TEST_THREADS=2 CARGO_BUILD_JOBS=2 cargo test -j 2 --workspace -- --test-threads=2
```

- `cargo fmt --all --check` — clean.
- Clippy with `-D warnings` — clean (full workspace, all targets).
- Full workspace test — see the final run appended to this report; the focused runs below passed
  before it.

Focused coverage added or updated:

- `headless::journeys::direct_play_home_detail_and_edit` (new, offline fake services): creates and
  remembers a non-default SIGIL profile; Home `Continue` dispatches `Command::Play` and launches
  that profile (not the default); Detail's main action is `Play`, Edit opens Setup at focus 60
  and never launches; missing engine falls back to Setup without launching; R2 scan over six
  screens. Captures `064-home-continue.png`, `064-direct-play-running.png`,
  `064-detail-play.png`.
- `runtime::local_ui::tests::home_continue_validates_custom_files_before_direct_play` (new): Home
  Continue on a custom setup opens the missing-file dialog (and never launches), then the
  changed-file dialog. Captures `064-home-missing-file.png`, `064-home-changed-file.png`.
- `headless::journeys::blocked_setup_and_settings`: Home Continue on a blocked latest item
  (missing IWAD) falls back to Play setup with no fake launch.
- `headless::journeys::profiles_isolation_and_continue`: Home Continue launches the remembered
  "Casual" profile directly and keeps its isolated save/data/shot paths.
- `headless::journeys::library_favorite_and_home`: no "Recently played" section, Continue is the
  only red action, card gutter assertions.
- `headless::journeys::settings_rows_and_about`: no `Screenshots:` list; Made by/repository
  visible above the fold; status-bar licence still present and inside the panel; `Start` hint
  string is Quit; `013-about.png`.
- `headless::journeys::title_fit_home`: Home now audits the single latest title/author slot.
- `engines::dsda::plan::tests::a_detected_screen_emits_width_and_height_after_shotdir`,
  `a_config_fallback_plan_emits_no_width_or_height`, `screen_sizes_are_bounded`.
- `runtime::visual_copy_tests::the_drawable_screen_only_comes_from_a_fullscreen_window`.
- Template/migration tests: `a_fresh_install_provisions_auto_aspect_and_the_hardware_renderer`,
  `a_version_bump_to_v5_sets_auto_aspect_and_hardware_rendering`,
  `a_version_bump_to_v5_keeps_player_renderer_and_aspect_choices`, plus merge tests for a new
  untouched/changed string key and the updated v1/v2/v3/v4 migration fixtures. The existing v4
  VSync/sensitivity tests still pass.

### Headless PNGs for review

- `target/headless/direct_play_home_detail_and_edit/064-home-continue.png`
- `target/headless/direct_play_home_detail_and_edit/064-direct-play-running.png`
- `target/headless/direct_play_home_detail_and_edit/064-detail-play.png`
- `target/headless/home_continue_validates_custom_files_before_direct_play/064-home-missing-file.png`
- `target/headless/home_continue_validates_custom_files_before_direct_play/064-home-changed-file.png`
- `target/headless/library_favorite_and_home/010-library.png`
- `target/headless/library_favorite_and_home/010-home-continue.png`
- `target/headless/settings_rows_and_about/013-about.png`
- `target/headless/base_games_library_page_setup_sources_and_recent_history/032-library-base-games.png`
- `target/headless/base_games_library_page_setup_sources_and_recent_history/032-base-game.png`
- `target/headless/base_games_library_page_setup_sources_and_recent_history/032-base-play-setup.png`
- `target/headless/base_games_library_page_setup_sources_and_recent_history/032-home-base-game.png`
- `target/headless/local_custom_library_home_missing_changed_and_delete_dialog/026-home-custom.png`

Reviewed: Home has no lower recent list and the header title sits 12 px further in; Library text
and tags keep a visible gutter; Detail shows red Play plus Edit; About shows the author and
repository without the screenshot list; base-game Home/detail/band use the title art with the
DoomWiki credit on detail; the footer reads Start → Quit.

## Files changed

- UI/wiring: `crates/ddl/ui/app-window.slint`, `local-strings.slint`, `wiring-strings.slint`,
  `wiring_copy.rs`, `runtime.rs`, `local_ui.rs`, `screenshots.rs`, `build.rs`.
- Launch/services: `crates/ddl/src/services.rs`, `crates/ddl-core/src/engines/dsda/plan.rs`
  (`ScreenSize`, `-width`/`-height`, describe), `resolver/resolve.rs`, `local/launch.rs`,
  `engines/dsda/mod.rs`, `iwads/base_games.rs` (through the shared local plan).
- DSDA config: `crates/ddl-core/src/dsda_config/template.rs`, `merge.rs`, `manager.rs`,
  `templates/dsda/steam-deck-default.toml` (v5).
- Tests: `headless_journeys.rs`, `headless_production.rs`, `local_ui_tests.rs`, plan/merge/
  template/manager unit tests, `crates/ddl-core/tests/dsda_launch.rs`.
- Assets/provenance: `crates/ddl/ui/assets/screenshots/base-{doom,doom2,tnt,plutonia}/`.
- Research: `docs/research/dsda-engine.md` (§1 `-width`/`-height`; §9 `videomode` values and the
  v5 implications, including the owner measurement caveat).

No new dependencies. No coordinator-owned docs, workflows or packaging files were edited.

## Open items / limitations

- **Explicit "Software" is indistinguishable from the engine default** in the file. A v4 install
  that never touched the renderer and one where the player picked Software look identical; both
  migrate to OpenGL. Any other explicit choice is preserved with a conflict.
- **GPU compatibility**: the OpenGL path and its visual fidelity across the reference mapsets are
  **Needs Deck Validation**. The owner accepted the tradeoff; no claim is made beyond the
  owner-reported CPU-power measurement.
- **Screen detection** uses the fullscreen drawable; windowed DDL keeps the config fallback, and
  the CLI (`ddl launch`, no window) also keeps it. Internal-panel and external-TV behavior,
  Auto aspect and return focus after direct Play are **Needs Deck Validation**.
- The header inset is a Slint layout constant; there is no node/DOM seam for an exact automated
  assertion, so it is covered by the headless renders above (and the card gutter has a node
  assertion).
- Base-game art provenance is read from Wayback snapshots because doomwiki.org blocks automated
  clients; the original doomwiki URLs and retrieval date are recorded in each `SOURCES.toml`.

## Final full test run

Command above with `pipefail`, `CARGO_BUILD_JOBS=2`, `-j 2`, `RUST_TEST_THREADS=2` and
`--test-threads=2`: **exit 0**. Results: `ddl` 182 passed / 0 failed (includes the new direct-play
journey and the updated journeys); `ddl-core` 594 passed / 0 failed; every integration test
binary and doctest passed. `cargo fmt --all --check` and clippy with `-D warnings` are clean.

One additional migration test
(`a_fresh_install_provisions_auto_aspect_and_the_hardware_renderer`) was added after that run to
cover a fresh install's template defaults; it was run on its own and with the full `ddl-core`
suite afterwards (**all 595 core tests pass**). No production code changed after the full run.

# 027 — Split the headless production journey into focused tests — worker report

**Status:** R1–R4 implemented; A1–A4 verified. No production or UI code, no
visual, and no assertion changed: the one `fixture_screens_…` test and the
~2,400-line production journey are now 25 independently runnable headless tests,
each with its own scratch XDG root and its own artifact directory. The full
`cargo test --workspace` passes three times in a row from an empty
`target/headless`.

## What changed

- `crates/ddl/src/headless.rs` — the render helpers stay; `headless_dir()` now
  nests under `<DDL_HEADLESS_DIR>/<test>/`; a new `isolated(module, case)`
  harness runs one case alone in a subprocess (Slint allows one platform per
  process). The single `fixture_screens_navigation_and_focus_render_headlessly`
  becomes `shell_navigation_and_focus_render_headlessly`, which covers only the
  spec 008/010 shell navigation and focus walk.
- `crates/ddl/src/headless_journeys.rs` (new) — 24 isolated tests: 12 production
  journey segments, three formerly separate journeys (categories, recipe changes,
  uninstall), and nine title-fit passes.
- `crates/ddl/src/headless_production.rs` — the journey's fixture block becomes
  `Scene` + `scene()`; the linear body is split into `first_run`, `iwad_review`,
  `discover_and_detail`, `install_and_setup`, `play_setup_options`,
  `library_home_and_favorite`, `blocked_setup_and_settings`, `settings_rows`,
  `engine_updates_and_dialogs`, `install_error`, `auger_setup` and `profiles`.
  `title_fit_journey` gains a screen/component/sheet/recipe-group selection.
- `crates/ddl/src/local_ui_tests.rs` — reuses the shared harness.
- `scripts/readme-screenshots.py` — runs the three tests that own the four
  README captures and finds them under the per-test directories.
- `docs/steam-deck-guide.md` — the R4 "Running UI tests" note.

### Test-only seams (no production behaviour)

1. `Scene` + the `Captures` closure parameter: the journey's capture call sites
   take a sink, so a later test can replay its predecessors with a no-op sink
   and start from exactly the interactive state. Only `crates/ddl/src/headless*.rs`
   uses them.
2. `play_setup_options` now waits for the real run guard before asserting it is
   held. The old single-process test got that delay from the render inside the
   capture; a silent replay does not render, so the wait makes it deterministic
   under parallel load.
3. `title_fit_journey`'s screen group / component / sheet / recipe-group
   parameters; each was one loop body before.
4. `isolated()` and the per-test artifact directory. `DDL_TITLE_ONLY` is removed
   (title-fit tests are named now); `DDL_TITLE_RECIPE` and `DDL_TITLE_AUDIT`
   still work.

## R2 — old section → new test, with every capture

Every old PNG name survives; none was lost (229 unique capture names before and
after; the only new files are the per-test comparison artefacts that used to
collide on one shared path).

| New test | Old journey section | Captures (unchanged names) |
| --- | --- | --- |
| `headless::shell_navigation_and_focus_render_headlessly` | the shell walk in `fixture_screens…` | `home`, `discover`, `mapset-detail`, `play-setup-sigil`, `library`, `settings`, `quit-confirmation`, `play-setup-auger`, `play-blocked-iwad` |
| `headless::journeys::first_run_and_engine_setup` | `journey` lines 1–170: loading, first run, empty Home/Library, quit, engine install/cancel/retry | `010-loading`, `010-first-run`, `010-home-empty`, `010-library-empty`, `010-quit-dialog`, `010-first-run-engine-install`, `010-cancel-dialog` |
| `headless::journeys::iwad_review` | controller browser, scan and ambiguity choice | `010-directory-browser`, `010-iwad-ambiguous`, `010-iwad-found` |
| `headless::journeys::discover_and_detail` | Discover fixtures, fallback, Eviternity, mapset detail | `010-discover`, `013-discover-fallback`, `013-discover-eviternity`, `010-mapset-detail` |
| `headless::journeys::install_progress_and_play_setup` | SIGIL install progress/stages and ready Play setup | `010-install-progress`, `010-install-verify`, `010-install-extract`, `010-install-commit`, `010-play-setup-ready` |
| `headless::journeys::play_setup_options_and_engine_exit` | advanced components, compat dialog, import, scroll, playing, engine-exit toast | `010-compat-dialog`, `010-import-browser`, `010-play-setup-options-scroll`, `010-playing`, `010-engine-failed-toast` |
| `headless::journeys::library_favorite_and_home` | Library, favorite star, Home Continue and credits | `010-library`, `015-library-favorite`, `010-home-continue` |
| `headless::journeys::blocked_setup_and_settings` | missing-IWAD blocked Play setup, Settings summary | `010-play-setup-blocked`, `010-settings` |
| `headless::journeys::settings_rows_and_about` | catalog update rows, Settings sections, About | `010-settings-folders`, `010-settings-config`, `010-settings-diagnostics`, `013-about` |
| `headless::journeys::engine_updates_reset_and_remove_folder` | engine update/rollback, DSDA reset, folder removal | `010-reset-dialog`, `010-remove-folder-dialog` |
| `headless::journeys::install_error_keeps_previous` | checksum rejection with the previous install kept | `010-install-error` |
| `headless::journeys::auger_options_and_components` | AUGER;ZENITH install and exclusive add-on chooser | `010-play-setup-auger`, `010-play-setup-auger-alternate`, `010-play-setup-auger-deckard` |
| `headless::journeys::profiles_isolation_and_continue` | `profiles_journey` | `018-profile-picker`, `018-profile-names`, `018-delete-confirm`, `018-home-continue`, `018-library-profiles` |
| `headless::journeys::categories_favorites_and_screenshots` | `categories_journey` | `015-discover-all`, the 13 `023-discover-<id>`, `023-detail-{ancient-aliens,auger-zenith,back-to-saturn-x-e1}`, `023-discover-white-sky`, `023-detail-white-sky`, `023-home-long-credit`, `023-discover-fallback`, `015-discover-category`, `015-discover-favorites-empty`, `015-discover-all-favorite`, `015-discover-favorites`, `015-detail-tags` |
| `headless::journeys::recipe_changes_update_keep_and_removed` | `recipe_changes_journey` | `020-library-update`, `020-detail-update`, `020-update-notice`, `020-library-removed`, `020-detail-removed` |
| `headless::journeys::uninstall_confirm_and_keep_profiles` | `uninstall_journey` | `022-detail-uninstall`, `022-uninstall-confirm`, `022-uninstall-delete-profiles` |
| `title_fit_home` / `title_fit_discover` / `title_fit_library` / `title_fit_detail` / `title_fit_progress` | `title_fit_journey` screen loop | `025-{home,discover,library,detail,progress}-<id>` for all 14 fixtures |
| `title_fit_setup_and_components_{first,second}` | `title_fit_journey` setup screen + components | `025-setup-<id>` and `025-component-<id>-<n>` (recipe halves) |
| `title_fit_profiles` / `title_fit_uninstall` | `title_fit_journey` sheet loop | `025-profiles-<id>` / `025-uninstall-<id>` |

The journey's assertions (including the `RunGuard` checks, the focus/render
checks and the title-fit measurements) moved with their segments unchanged;
`025-fit.txt` (or `025-audit.txt` with `DDL_TITLE_AUDIT`) is written once per
title-fit test.

## R3 — speed

Every test is runnable alone by name (`cargo test -p ddl <name>`), owns its
scratch XDG root, and writes into its own `<DDL_HEADLESS_DIR>/<test>/`, so no two
tests touch the same path (the old flat layout had the shell and production
captures both writing `comparison-mapset-detail.png` and friends). The gated
loopback fixtures were already releasing `engine_gate`/`package_gate`/`play_gate`
before the slow renders, and `engine_response_timeout` stays at the 120 s
test-only knob; the silent replay renders nothing, so it cannot race a gate.

### Per-test wall time (individually, dev Mac)

Before: the single `fixture_screens_navigation_and_focus_render_headlessly`
test (shell walk + every production journey) finished in **354.98 s** — far over
the 60 s target, and one flake risk for the whole suite.

After, each test run alone by name (`cargo test -p ddl <name>`), seconds:

| Test | s | Test | s |
| --- | ---: | --- | ---: |
| `first_run_and_engine_setup` | 10.65 | `title_fit_home` | 29.05 |
| `iwad_review` | 4.98 | `title_fit_discover` | 23.88 |
| `discover_and_detail` | 5.06 | `title_fit_library` | 30.03 |
| `install_progress_and_play_setup` | 5.06 | `title_fit_detail` | 29.96 |
| `play_setup_options_and_engine_exit` | 4.07 | `title_fit_progress` | 30.48 |
| `library_favorite_and_home` | 4.12 | `title_fit_setup_and_components_first` | 37.47 |
| `blocked_setup_and_settings` | 4.95 | `title_fit_setup_and_components_second` | 22.12 |
| `settings_rows_and_about` | 5.16 | `title_fit_profiles` | 19.10 |
| `engine_updates_reset_and_remove_folder` | 6.52 | `title_fit_uninstall` | 17.58 |
| `install_error_keeps_previous` | 5.82 | `categories_favorites_and_screenshots` | 22.74 |
| `auger_options_and_components` | 7.61 | `recipe_changes_update_keep_and_removed` | 5.80 |
| `profiles_isolation_and_continue` | 7.85 | `uninstall_confirm_and_keep_profiles` | 5.29 |
| `shell_navigation_and_focus_render_headlessly` | 26.00 | | |

Longest single test: **37.47 s** (`title_fit_setup_and_components_first`), under
the ~60 s target. The 25 headless tests in the default parallel harness
(`cargo test -p ddl --bin ddl headless::`) finish in **82.5 s and 94.4 s** on two
runs (one earlier run on a loaded host read 196 s, so treat the range as
host-load dependent). The `ddl` unit binary is now **123 passed in 82.84 s**
(before: 99 tests, 355 s — the one journey test was the bulk of it).

### Total wall time

| Run (`env -u DDL_HEADLESS_DIR`, from empty `target/headless`) | real | result |
| --- | ---: | --- |
| `cargo test --workspace` run 1 (also rebuilt test targets) | 251.52 s | exit 0 |
| `cargo test --workspace` run 2 | 120.23 s | exit 0 |
| `cargo test --workspace` run 3 | 115.94 s | exit 0 |

Before, the single headless UI test alone was **354.98 s**; the workspace was
never faster than that. After, the whole workspace is ~116–120 s warm and the
worst single headless test is 37.5 s.

## R4 — docs

`docs/steam-deck-guide.md` gains a "Running UI tests" subsection: run one test by
name, run every production flow, regenerate/verify the README screenshots, and
where PNGs and the title-fit report land (`DDL_HEADLESS_DIR/<test>/`).

## Verification

Dev host: macOS arm64, Rust 1.99.1. Every run had `DDL_HEADLESS_DIR` unset and
started from `rm -rf target/headless`.

```text
$ env -u DDL_HEADLESS_DIR cargo fmt --all --check
FMT_CLEAN                                          (exit 0, no output)

$ env -u DDL_HEADLESS_DIR cargo clippy --workspace --all-targets -- -D warnings
    Checking ddl v0.0.5 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.47s   (exit 0)

$ env -u DDL_HEADLESS_DIR cargo test --workspace     # three consecutive runs
run 1: real 251.52s  exit=0
run 2: real 120.23s  exit=0
run 3: real 115.94s  exit=0
  ddl unit:        test result: ok. 123 passed; 0 failed; ... finished in 82.84s
  ddl-core unit:   test result: ok. 376 passed; 0 failed; ... finished in 1.19s
  ddl_launch_probe: 0 tests;  dsda_launch: 16 passed;  local_content: 2 passed
  Doc-tests:       3 passed
```

The `ddl` unit binary grew from 99 to 123 tests (the one journey became 25
headless tests); 25 of them spawn their own render subprocess. Default parallel
runs are green.

### Capture preservation

229 unique non-comparison PNG names were produced before and after the split;
none is missing. The only new artefacts are the previously-colliding
`comparison-*.png` files: the shell and production captures both wrote
`comparison-mapset-detail.png`, `comparison-settings.png`,
`comparison-play-setup-sigil.png` and `comparison-play-blocked-iwad.png` on the
shared flat path. Each test now writes them under its own directory.

### README screenshots (spec 016 R5, A3)

The four README source PNGs are byte-identical between the pre-split run
(`target/headless-before`) and the split tests (`target/headless-per`):

```text
cmp 010-home-continue.png        library_favorite_and_home/           → IDENTICAL
cmp 015-discover-all-favorite.png categories_favorites_and_screenshots/ → IDENTICAL
cmp 010-play-setup-auger.png      auger_options_and_components/        → IDENTICAL
cmp 015-detail-tags.png           categories_favorites_and_screenshots/ → IDENTICAL
```

`scripts/readme-screenshots.py` runs the three producer tests and collects the
PNGs from the per-test directories; the names and the lossless recompression are
unchanged. One pre-existing drift, unrelated to this task:
`docs/screenshots/play-setup.png` matches neither the pre-split nor the split
render (its raw pixels differ from the current `010-play-setup-auger.png`), so
`--check` reported it stale both before and after. Refreshing that committed
asset is a separate visual update and was left out of scope.

## Open questions / follow-ups

- The per-test comparison artefacts are no longer shared, so a contact sheet must
  gather them per test directory; the R4 note says where.
- `012-render-metrics.tsv` and `025-fit.txt` are now per test, so comparing a
  single run across screens needs the per-test files.
- No Deck validation was needed: only off-screen software rendering is involved.

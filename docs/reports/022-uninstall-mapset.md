# Spec 022 — Uninstall a mapset

Implemented R1–R6 (A1–A4) with synthetic content, loopback HTTP and off-screen Slint software
rendering. No visible window was opened and no screen was captured. No new dependencies, engine
flags, DSDA config keys, compatibility claims or Steam Input code were added. Physical
Gaming-Mode uninstall (A5 / U1–U3) remains **Needs Deck Validation**.

## Behavior and storage

- **Inactive first, then delete (R3).** `Installer::begin_uninstall` writes `library/<id>/active.json`
  with a new defaulted `uninstalled: true` field using the existing atomic temp-file-plus-rename;
  `load_manifest` then reports the mapset as not installed immediately, so a crash can never leave a
  launchable half-install. `Installer::finish_uninstall` then deletes content. Both phases are
  idempotent and retryable.
- **Containment.** Only paths under `<data>/library/<id>` are removed. Every directory goes through
  the existing `remove_retired` guard, which now also canonicalises the target and proves it stays
  inside DDL's library root; symlinks are never followed out (a symlinked direct child is unlinked
  as a link). The shared download cache, IWAD links and any neighbouring mapset are untouched.
- **Leftovers on next start.** `Services::finish_uninstalls` runs during `Command::Startup` before
  metadata adoption: it retries the marked deletes, then removes the record and drops the recipe
  from `state.recipes`. `cleanup_retired` deliberately skips uninstalled records so spec 020's
  generation queue and spec 022's whole-install delete never fight over the same directory.
- **Profiles (R4).** Kept by default — `ProfileStore` files and `save`/`data`/`shots`/`run` dirs
  stay, and a reinstall finds them (same recipe id, same remembered selection). With `--delete-profiles`
  / the checked box, `ProfileStore::delete_fully` removes each profile document **first**, then its
  runtime directory, proven inside DDL's profile root and refusing a symlinked directory. The
  remembered-profile entry is cleared only in that case; favorites stay, since they are a catalog
  preference.
- **User-imported components (D-014)** live inside the install directory, so they are deleted with
  it; the confirm dialog names them when `InstallManifest::has_imports()` is true.
- **Reinstall after an interrupted uninstall** adopts and un-marks the record in
  `Services::install_content`, so a half-deleted install can be rebuilt without a stale inactive
  record.

## UI (R1, R2, R6)

Mapset detail gains an installed-only **Uninstall** action (id 5) below the active-profile action;
it is present for installs no longer in the catalog and unreachable while the game runs. Per the
coordinator's 2026-10-17 clarification, there is no Library-level binding: Library → card (A) →
detail → Uninstall, and **Y stays reserved**. The confirm reuses the Ember sheet: it names the
mapset and the installed size, shows a controller-toggleable checkbox
"Also delete profiles and saves (N profiles)" that is **unchecked every time the sheet opens**, and
focuses **Cancel** first. A new `ViewNode` kind 15 renders the Ember checkbox (amber tick when
checked); Up/Down/Left/Right move focus inside the sheet and **A** toggles the box without any text
entry. The consequence line switches from "…profiles and saves are kept." to "Profiles, saves and
screenshots will be deleted too." when checked. Confirming returns to Library with a toast.

## Requirement → verification

| Item | Evidence |
| --- | --- |
| R1 / A2 | Detail id 5 is asserted present for an installed mapset and for a removed-catalog install (`recipe_changes_journey`); `action()` ignores all input while playing. |
| R1 / review fix | `assert_detail_actions_unique` asserts unique labels and exactly one primary for the Current (`uninstall_journey`) and Update (`recipe_changes_journey`) states; the pre-existing duplicate was reproduced on `main` (`020-detail-removed.png`). |
| R2 / A2 | `022-uninstall-confirm.png` (unchecked, Cancel focused, size and count) and `022-uninstall-delete-profiles.png` (checked, amber tick, delete consequence) inspected; the journey drives `UpArrow` + `Return` with real keys and asserts `art` 0 → 1 and that the sheet stays open. |
| R3 / A1 | `uninstall_marks_inactive_first_and_removes_only_this_recipe` (mark makes `load_manifest` `None` while files remain, then the recipe dir goes and the neighbour/cache stay); `retired_cleanup_refuses_profile_paths_and_symlink_escapes` (traversal + symlink); `uninstall_never_follows_a_symlinked_directory_out_of_the_library`; `an_interrupted_uninstall_stays_inactive_and_is_finished_on_the_next_start` (crash between mark and delete, then startup completes it). |
| R4 / A1 | `uninstall_keeps_profiles_by_default_and_a_reinstall_finds_them` (saves/shots survive, selection kept, reinstall reuses them); `uninstall_can_delete_profiles_and_saves_but_keeps_favorites`; `delete_fully_removes_the_document_and_runtime_dirs_without_a_last_profile_guard`; `delete_fully_refuses_a_symlinked_runtime_directory`. |
| R3 / R4 (imports) | `installed_size_and_imports_feed_the_uninstall_dialog`; the dialog's imported line renders when a `package = None` member exists. |
| R5 / A3 | `cli_uninstall_uses_the_shared_transaction_and_reports_profiles` (success line, second call "not installed", unknown id, `--delete-profiles` count); `cli_uninstall_refuses_while_the_state_lock_is_held`; `mapset_uninstall_parses_the_optional_profile_flag`; real-binary `cli_help` cases. |
| R6 | `docs/design/ui.md` §10 documents the entry point, the checkbox, containment and the keep-first order. |
| A4 | Command tails below. |
| A5 | Open — **Needs Deck Validation**; `docs/deck-testing.md` U1–U3 describe the Gaming-Mode journey. |

## Headless visual review

Inspected the off-screen 1280×800 PNGs under `target/headless/`:

- `022-detail-uninstall.png`: the fifth detail action scrolled into view with its amber ring and
  the "A Uninstall" hint; the action column is Play setup → Add to favorites → Active profile →
  Uninstall, each label once (see the review fix below).
- `022-uninstall-confirm.png`: title "Uninstall AUGER;ZENITH?", "Installed size: 24 B" (the
  synthetic fixture member), the unchecked checkbox with the profile count, the "profiles and saves
  are kept" line, and Cancel focused first with Uninstall separated to the right.
- `022-uninstall-delete-profiles.png`: the same sheet with an amber tick and focus ring on the
  checkbox and the consequence line switched to "Profiles, saves and screenshots will be deleted
  too."; the hint bar shows the checkbox action.

No continuous animation at idle (the capture helper still asserts a clean idle frame).

## Review fix — duplicate "Play setup" on detail

**Pre-existing, not from this spec.** Detail's second action was unconditional: for an installed
mapset with no catalog update (Current or Removed) it fell back to the label `T::Setup` while the
primary button already said `T::Setup`, so the column showed "Play setup" twice. The block is
unchanged from `main` (`git diff 6cd5df0 HEAD -- crates/ddl/src/runtime.rs` shows no hunk in the
detail actions) and I reproduced it by building `main` (`da93e97`) in a detached worktree and
running the existing `fixture_screens` headless test: its `020-detail-removed.png` shows two
stacked "Play setup" buttons.

Fix: the second action now renders only when the recipe has an update (`RecipeStatus::Update`), so
it reads "Update" and never repeats the primary. For every other detail state each label appears
once: installed → Play setup / Add to favorites / Active profile / Uninstall; update available →
Play setup / Update / Keep current / Active profile / Uninstall; not installed → Install / Add to
favorites / Active profile. The primary is still the action that makes sense — Install when there
is no install, Play setup otherwise (the established Continue → Play setup → Play flow; a bare
"Play" here would silently skip setup). No button ids or y positions moved, so existing navigation
and the spec 020 update/keep tests are unaffected.

`assert_detail_actions_unique` in the production headless test now asserts, for both the update
state (`recipe_changes_journey`) and the installed Current state (`uninstall_journey`), that the
detail action labels are unique and that exactly one is primary.

Re-rendered and inspected: `022-detail-uninstall.png` (single Play setup) and `020-detail-removed.png`
(single Play setup) both drop the duplicate.

## Final verification

All commands exited 0 on macOS arm64 after merging `main`; no external-network or physical Deck
validation was run. `cargo fmt --all --check` prints nothing; clippy is clean.

```text
cargo fmt --all --check
  exit 0 (no output)

cargo clippy --workspace --all-targets -- -D warnings
  Compiling ddl v0.0.4 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-022-uninstall/crates/ddl)
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.43s

cargo test --workspace
  ddl unit:       test result: ok. 81 passed; 0 failed
  catalog_cli:    test result: ok. 2 passed; 0 failed
  cli_help:       test result: ok. 2 passed; 0 failed
  logo_assets:    test result: ok. 2 passed; 0 failed
  ui_strings:     test result: ok. 1 passed; 0 failed
  ddl-core unit:  test result: ok. 317 passed; 0 failed
  dsda_launch:    test result: ok. 16 passed; 0 failed
  Doc-tests:      test result: ok. 3 passed; 0 failed
```

424 tests passed. Merged `main` at `4c84341` (spec 016 phase B logo assets, README, spec 023) with
no conflicts, leaving tested application sources unchanged; the coordinator's spec 022 R1 wording
("detail only; Y stays reserved") arrived in that merge and matches the implementation.

## Open items

- A5 / U1–U3: Gaming-Mode uninstall (checkbox interaction and profile removal) needs a physical
  Deck; `docs/deck-testing.md` records the steps.
- No implementation questions or new dependencies. `OVERVIEW.md`, `docs/decisions.md`,
  `docs/status.md` and `docs/journal.yml` were not edited by this worker.

# 059 — Flatpak sandbox-ready — worker report

**Status:** R1–R8 implemented; A1–A4 covered by tests and off-screen renders; A5 green. The last
acceptance item (**Needs Deck Validation**) remains for the owner's first local Flatpak build.
No manifest, metainfo, desktop entry, `flathub.json` or submission text was written.

## Result

DDL now detects `/.flatpak-info` and, only when it is present, extracts the verified DSDA-Doom
AppImage instead of trying FUSE, says when a recorded content path is unreachable from the
sandbox, offers a one-time copy of a tarball-era install, opens Steam's keyboard through
`xdg-open`, and points app updates at Discover. Outside the sandbox every path is unchanged.
The engine still comes from the official AppImage with the same reviewed hashes (D-001, D-007,
D-011, D-013), and the catalog/engine downloads stay in-app.

Implementation is in seven small commits (`1ffb951` … `c0c0fd9`); `3a86a27` merges `main` before
the final checks. The previous worker's uncommitted snapshot at
`refs/keep/flatpak-sandbox-ready` was reviewed; it was rustfmt-only, kept, and committed as
`c0c0fd9`. Files owned by the concurrent spec 050 worker (`idgames/`, `cli_exec.rs`, the
`CliFailure` exit-code area, `.github/workflows/idgames-index.yml`) were not touched.

## Requirements

| Req | Implementation | Evidence |
| --- | --- | --- |
| **R1** detect the sandbox | `crates/ddl-core/src/platform/sandbox.rs`: `Sandbox::detect` probes `/.flatpak-info` through a `Probe` trait and reads `FLATPAK_ID` through the existing `platform::paths::Env` trait, so tests pass a fake marker and fake variables. `detect_host` is the production probe. The startup log in `attach_with_sandbox` records `flatpak` and `app_id`. Settings → Diagnostics/About shows `Flatpak sandbox: yes (<id>)`, `yes` or `no`. | `platform::sandbox::tests` (5 tests, incl. marker-only detection and a rejected empty/multiline id); `headless::flatpak::diagnostics_*` renders. |
| **R2** always extract in the sandbox | `EngineManager::with_sandbox`; `perform_install` sets `force_extract` when `Sandbox::must_extract_engine()`, which skips the AppImage `-v` FUSE probe and takes the existing D-013 extraction path. Installed `AppRun` is launched with the unchanged D-011 argv. Verification, pinning, active/previous-good state and rollback are the same code. | `a_sandboxed_install_extracts_first_and_never_probes_the_appimage` records the AppImage's only call as `--appimage-extract`; `an_unsandboxed_install_still_probes_the_appimage_first` records `-v`; `rollback_works_between_two_sandboxed_installs`. |
| **R3** paths | No production change was needed: `paths.rs` already honours absolute `XDG_CONFIG_HOME`/`XDG_DATA_HOME`/`XDG_CACHE_HOME`/`XDG_STATE_HOME` and only falls back to `HOME`. Added a test that pins Flatpak's layout (`~/.var/app/<id>/{config,data,cache,.local/state}`) with a fake environment and asserts no host-layout fallback. | `flatpak_xdg_directories_resolve_inside_the_app_data_dir`. |
| **R4** content visibility | `crates/ddl-core/src/iwads/visibility.rs` follows readable launcher records (Steam `libraryfolders.vdf`, Heroic `installed.json`, minigalaxy) and configured folders, then reports a path only when opening it fails. `Snapshot::hidden_paths` is populated only when sandboxed. The IWAD review screen and Settings → IWAD folders show one line per unreachable path; the controller picker maps a missing/denied root to `Problem::NotVisible`, whose text is exactly `<path> is not visible to the Flatpak — see the guide`. Discovery itself is untouched, so the Steam/Flatpak-Heroic/SD-card paths already handled by `iwads::steam` and `iwads::stores` keep working through read-only host access. | `iwads::visibility::tests` (5); `services::flatpak_tests::{a_steam_library_the_flatpak_cannot_see_is_reported_not_silently_empty, a_configured_folder_that_vanished_from_the_sandbox_is_named, browsing_a_folder_the_flatpak_cannot_open_says_so}`; `headless::flatpak::settings_name_a_library_the_flatpak_cannot_see`. |
| **R5** one-time import | `crates/ddl-core/src/platform/legacy_import.rs`. `find` requires a Flatpak sandbox, no recorded answer, no state of the sandbox's own, and state at the host's `~/.config/ddl` / `~/.local/share/ddl`; the decision is stored as `state/legacy-import.json`. `import` stages under each destination root, then commits top-level entries (existing ones are renamed `<name>.pre-import`, never deleted). It copies, never moves. JSON strings and `.cfg` text that point into the old config/data directories are rewritten; symlink targets are rewritten only when absolute and inside those directories, so IWAD links keep naming the player's files. Engine versions in AppImage (FUSE) mode are skipped and reported so DDL reinstalls them; extracted versions and the `installed.json` state are carried across. The dialog is `Sheet::ImportOffer` (safe choice focused; B postpones without remembering; "No, don't ask again" is remembered), and accepting emits the import toast. Copied files are read by the existing spec 055 readers on the next open, so every supported older shape imports. | `platform::legacy_import::tests` (7) include the spec 055 `v0.0.7-deck` fixture import with the old tree snapshotted before/after; `services::flatpak_tests::every_release_fixture_imports_and_loads_in_the_sandbox` imports `v0.0.5-screens.1`, `v0.0.6-local` and `v0.0.7-deck` and loads engine, IWAD folders, installs and profiles; `headless::flatpak::import_offer_*`. |
| **R6** Steam keyboard | `osk::command(&Sandbox)` selects `xdg-open steam://open/keyboard` when sandboxed and `steam …` otherwise; `focused` takes the sandbox. The existing visible "Steam + X" hint is unchanged as the fallback. | `runtime::osk::tests::{the_flatpak_opens_the_keyboard_through_xdg_open, outside_the_sandbox_the_command_is_unchanged}`. |
| **R7** updates | DDL has no app self-update path to hide — a repository grep finds none (engine/catalog updates are separate features and unchanged). The policy is encoded in `Sandbox::allows_self_update` for any future updater. About shows the DDL version (`App version`) and the engine version on its own line, then, only when sandboxed, "DDL updates come from Discover (Flathub). DSDA-Doom updates stay in Settings, Engine." | `platform::sandbox::tests` assert the flag both ways; `headless::flatpak::diagnostics_show_the_flatpak_sandbox_line_and_the_update_source` and `diagnostics_outside_the_sandbox_say_no_and_hide_the_store_note` assert the text appears only under Flatpak. |
| **R8** guide | `docs/steam-deck-guide.md` gained "Flatpak (Discover)": marked as applying once DDL is on Flathub, install from Discover, manual "Add a Non-Steam Game" with either the exported entry or `/usr/bin/flatpak` + `run io.github.vittau.Doom-Deck-Launcher` (**Needs Deck Validation**), Compatibility tool none, artwork, no Steam overlay, data under `~/.var/app/…`, IWAD visibility and the one-line message, and importing a tarball setup. | Guide diff; links to `docs/research/flatpak.md`. |

## Acceptance criteria

- **A1 (R1–R3)** — fake sandbox marker, fake `XDG_*`, and the engine plan tests pass:
  sandboxed install extracts then points at `squashfs-root/AppRun` with no FUSE `-v` call;
  unsandboxed install still probes the AppImage. Rollback across extracted versions tested.
- **A2 (R4)** — "not visible" tests with fake roots; the picker test shows the visible Steam
  library is still offered and the hidden SD-card library is not.
- **A3 (R5)** — import from spec 055 fixture trees into empty fake sandbox directories passes:
  recorded paths rewritten, IWAD links unchanged, original files byte-identical (snapshot
  comparison), and the offer renders off-screen (`059-import-offer.png`,
  `059-import-offer-import-focused.png`, `059-after-import.png`).
- **A4 (R6, R7)** — keyboard command selection tested for both sandboxes; the Diagnostics render
  shows `Flatpak sandbox: yes (io.github.vittau.Doom-Deck-Launcher)` and the Discover update
  line, while the plain render shows `Flatpak sandbox: no` and no store note
  (`059-diagnostics-flatpak.png`, `059-diagnostics-plain.png`; the hidden-folder render is
  `059-folders-not-visible.png`).
- **A5** — `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
  and `cargo test --workspace` are green (details below), and the packaging grep at the end
  shows no manifest/metainfo/desktop/`flathub.json` was added.
- **Needs Deck Validation** — still open: extracted engine with GL/audio/pad, IWAD discovery,
  import and the keyboard in Gaming Mode, as listed in the spec.

## Verification

Run after merging `main` (`3a86a27`) and committing the formatting pass (`c0c0fd9`):

```sh
cargo fmt --all --check                                  # passed
cargo clippy --workspace --all-targets -- -D warnings    # passed, no warnings
cargo test --workspace                                   # 797 passed, 0 failed
```

Breakdown: `ddl` unit tests 169, `ddl-core` unit tests 586, integration tests 39, doctests 3.
The UI tests render through Slint's software renderer into `target/headless/<case>/`; no visible
window was opened and no screen was captured. No new dependency was added.

## Packaging-rule evidence (A5)

No manifest, AppStream metainfo, `.desktop` file, `flathub.json`, `.flatpakref`/`.flatpakrepo`
or submission text exists in this branch's changes:

```text
$ (git diff --name-only main...HEAD; git ls-files --others --exclude-standard) \
    | grep -Ei '(^|/)(flathub\.json|.*\.desktop|.*metainfo.*|.*\.flatpak|.*\.flatpakref|.*\.flatpakrepo)$|(^|/)flatpak.*\.ya?ml$|(^|/)io\.github\..*\.ya?ml$'
none
```

`git diff --name-status main...HEAD` lists only Rust sources, Slint strings and
`docs/steam-deck-guide.md`; no YAML or JSON file is among them (the import decision file
`legacy-import.json` is runtime user state, not a repository file). The owner still writes the
manifest, metainfo, desktop entry and Flathub text by hand (D-034).

## Open questions / notes for the coordinator

1. **R4 wording is intentionally conservative.** From inside the sandbox DDL cannot tell "the
   host has this path but denied it" from "the host never had it"; it reports any path a
   launcher record or the player's folder list names that cannot be opened. That matches the
   spec's "permission error or missing mount" and the research's no-extra-permission constraint.
2. **R7 has nothing to hide today.** There is no in-app app-update mechanism; the About note is
   the whole change. A future self-updater must consult `Sandbox::allows_self_update()`;
   `EngineManager` and the catalog updater deliberately keep working (D-034).
3. **Imported FUSE engines are not copied**; the UI toast says DSDA-Doom will be installed again
   for the Flatpak and reports the count. This avoids importing an engine that cannot start.
4. **Proposed coordinator edits (not made):** add 059 to `docs/status.md` and an entry to
   `docs/journal.yml`. D-034 already records the decision; no decision change is needed.
5. **Deck checklist remains** (spec's last acceptance item): `flatpak run` + extracted
   `AppRun -v` and `ldd` sweep, GL/audio/pad, SD-card and Steam-Flatpak IWAD discovery, the
   import offer on real data, OSK in Gaming Mode, and which Steam shortcut route the client
   offers.

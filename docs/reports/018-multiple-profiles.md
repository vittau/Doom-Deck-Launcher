# Spec 018 — Multiple profiles per mapset

Local implementation and verification are complete. Physical Steam Deck save isolation
(A5 / C4) remains **Needs Deck Validation**. All UI images are off-screen Slint software renders;
no visible windows were opened and no screen was captured.

## Changes

- Play setup has a named active-profile row, and Mapset detail has an active-profile action.
  Both open a framed Ember picker using existing fonts, tokens and controller focus rings.
  Previous/Next switches without wrapping; the picker shows the active name and profile count.
- New offers separate copy-settings and default-settings actions. Names use the first free
  `Profile N`, starting at 2. Stable ids are allocated independently, skipping existing profile
  files and orphaned runtime directories. Copies include components, skill, pistol start,
  IWAD variant and scalar overrides, but start with fresh history and empty save/data/shot dirs.
- Rename offers Default, Casual, UV, Pistol start, Co-op, Speedrun and the first free Profile N.
  No text entry is used. Preset display strings are translated resources; stored names are data.
- Delete is unavailable for the last profile, with a second guard in the core store. Confirmation
  names the profile and says its saves and screenshots will be deleted; Cancel is focused first.
  The profile document is removed before runtime directories. Directory cleanup failure can
  leave orphaned data, but never a profile document pointing at deleted saves; subsequent
  profile creation skips that orphan's id. Installed mapset content is retained.
- Home shows the remembered profile name; Continue opens Play setup with it selected, and Play launches it. Mapset history is the most
  recent launch across its profiles, so selecting a fresh run does not hide the Continue card.
  Library shows the profile count. Switching, option saves and profile creation persist selection.
- `ddl launch <id> --profile <name|id>` selects within that mapset, preferring exact ids and
  rejecting ambiguous names with an instruction to use an id. Dry runs do not change selection;
  real launches remember the profile and history without saving per-launch option overrides.
  `ddl doctor` lists names and ids grouped by mapset, including the implicit Default.

## State compatibility

`app.json` stays at schema 2 with a defaulted `last_profiles` map (recipe id → stable profile id).
The task explicitly permits an additive schema-2 field. This follows the existing additive
search-folder setting, avoids a gratuitous version bump, and preserves folders, favorites and
install snapshots when reading older schema-1/2 files. The new field is written atomically with
existing state. A missing/stale remembered id falls back to the most recently played available
profile, or a deterministic available profile; an empty collection exposes Default.

No new dependencies, DSDA flags/config keys, Steam Input code or engine/config ownership changes.
The existing D-011 per-profile directory contract is retained; actual engine save behavior still
requires Deck validation. No new upstream compatibility claims are made.

## Requirement → verification

| Item | Evidence |
| --- | --- |
| R1 / A1 | `multiple_profiles_copy_defaults_rename_delete_and_remember`, `schema_two_without_last_profiles_loads_and_persists_selection`; production headless profile journey checks remembered selection and actual fake-engine save argv through Play and Home Continue. |
| R2 / A1 | Lifecycle service test checks copied components, skill, pistol start, variant, overrides, fresh history, default settings, first free name and independent dirs; `new_profile_does_not_adopt_orphaned_runtime_data` checks id allocation. |
| R3 / A1 | Lifecycle service test renames and reloads; headless keys select Casual from the preset dialog. |
| R4 / A1 | `destructive_delete_guards_last_and_removes_only_its_data`, `document_failure_preserves_data_and_directory_failure_leaves_no_document`; headless journey checks last-profile disabled, safe default, B cancellation, explicit confirmation and surviving other profile dirs. |
| R1, R5 / A2 | Production headless journey drives picker with actual arrow/Return/Escape keys, checks focus restoration and disabled-edge recovery, Home profile name, Library count and per-profile launch paths. |
| R6 / A3 | `launch_profile_accepts_names_and_ids_and_rejects_missing_or_repeated_values`, lifecycle service checks unknown/ambiguous names, `launch_profile_selects_isolated_paths_and_remembers_without_saving_overrides` checks dry-run/launch and doctor output. |
| R7 | Bindings documented in `docs/design/ui.md`; only C4 wording changed in `docs/deck-testing.md`. Existing layouts and global confirmation geometry retained. |
| A4 | Final command tails below. |
| A5 | Open — **Needs Deck Validation**, updated C4 describes the two-run save/screenshot/Continue journey. |

## Headless visual review

Inspected all five 1280×800 PNGs under `target/headless/`:

- `018-profile-picker.png`: active Profile 2, count, creation actions and amber focus ring fit
  within the carved frame; the setup selector remains visible behind the overlay.
- `018-profile-names.png`: all seven presets fit without text entry or clipping.
- `018-delete-confirm.png`: Casual is named, saves/screenshots consequences are explicit,
  and the safe Cancel focus is visible with separated destructive action.
- `018-home-continue.png`: Casual appears on the Continue hero and recent entry.
- `018-library-profiles.png`: AUGER shows 2 profiles and SIGIL shows 1 profile; counts and chips
  fit within the existing cards.

The headless navigation test exposed an initially passive selector row, which was corrected
and retested. The existing fixture focus-ring check also caught a shared-dialog height change;
the original 300 px geometry is retained, with extra room only for new profile dialogs.

## Final verification

All commands exited 0 on macOS arm64; no external-network or physical Deck validation was run.

```text
cargo fmt --all --check
  exit 0 (no output)

cargo clippy --workspace --all-targets -- -D warnings
  Checking slint v1.18.1
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 35s

cargo test --workspace
  ddl unit:       test result: ok. 64 passed; 0 failed
  catalog_cli:    test result: ok. 2 passed; 0 failed
  cli_help:       test result: ok. 2 passed; 0 failed
  logo_assets:    test result: ok. 2 passed; 0 failed
  ui_strings:     test result: ok. 1 passed; 0 failed
  ddl-core unit:  test result: ok. 300 passed; 0 failed
  dsda_launch:    test result: ok. 16 passed; 0 failed
  Doc-tests:      test result: ok. 3 passed; 0 failed
  all doctests ran in 1.08s; merged doctests compilation took 0.90s
```

390 tests passed. The targeted CLI run also passed all 35 tests; core profile-store tests passed
all 4 tests; the targeted headless journey passed (111.20 s) before the final workspace run.

Merged current `main` at `efaf2e8` with no conflicts (`3210789`); only documentation and design
assets arrived from spec 016, leaving tested application sources unchanged. The C4 wording
merged cleanly. The coordinator explicitly confirmed finishing against this main now; spec 017 will merge after
018 and resolve its own About-credit conflict.

## Coordinator clarification

The coordinator confirmed through Orca ask/reply that Continue keeps spec 010’s existing
Continue → Play setup → Play flow; spec 018 changes which profile is preselected, rather than
skipping setup. Tests assert both the selected identity and the ensuing engine argv.

## Open items

Only A5 / physical Deck validation remains outside local acceptance. No implementation questions
or new dependencies. Protected overview/decision/status/journal files were not edited.

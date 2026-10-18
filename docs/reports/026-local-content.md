# Spec 026 phase A — local content core and CLI

Implemented phase A (R1–R7, acceptance A1, A2, A4) in `ddl-core` and the headless `ddl` CLI.
No Slint/UI work: the file picker, order editor and badges are phase B (A3). No new dependency, no
engine flag, no config key and no Steam Input code was added. No window was opened and no screen
was captured.

Two incidental edits outside `ddl-core` were needed to keep the workspace compiling: the Slint
runtime's blocker-to-string map (`crates/ddl/src/runtime.rs`, one match arm) and hiding a changed
file's `Blocker` copy behind the existing `IntegrityFailed`/`Setup` labels. `crates/ddl/ui/**` was
not touched (spec 025).

## Engine facts (R1)

**Confirmed**, pinned to upstream tag `v0.30.1` (`8ed581ad5c868b4dba1c39eb21d29e4ddbed9fad`).
`dsda_Loadfiles` routes each `-file` value by a **case-insensitive** suffix (`dsda_HasFileExt` uses
`strcasecmp`): `.deh`/`.bex` → the `-deh` patch queue, `.zip` → the ZIP loader, `.wad`/`.lmp` → the
WAD loader, and **anything else calls `I_Error("File type … is not supported")`**. A value with no
extension is searched as `.wad`, `.lmp`, `.zip`, `.deh`, `.bex`. So `.pk3` and `.pk7` abort the
engine; `.lmp` is accepted as WAD content (the loader appends `.wad`). A `-deh` value goes through
`I_RequireDeh`, which returns an existing absolute path unchanged and parses it as text.
[`prboom2/src/d_main.c:1770-1820`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1770-L1820);
[`prboom2/src/dsda/utility.c:103-106`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/utility.c#L103-L106);
[`prboom2/src/SDL/i_system.c:641-670`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_system.c#L641-L670).
This is appended to `docs/research/dsda-engine.md` §1.

**Proposed (DDL policy).** A file is accepted only when its **content** kind (WAD header + directory
sanity, ZIP local-file header, bounded text) and its suffix agree; `.pk3`-named ZIPs and
`.wad`-named ZIPs are both refused, because the engine routes by suffix. The size cap is 4 GiB, the
ceiling implied by the WAD format's 32-bit offsets.

## Behavior and storage (R1–R6)

- **Kinds (`ddl-core/src/local/kind.rs`).** `UserFile::read` rejects relative paths, missing files,
  directories/devices/sockets, symlinks resolving to non-regular files, files over 4 GiB, content
  that is not WAD/ZIP/bounded-text, suffixes the engine's `-file` would abort on, and
  content/suffix mismatches. A WAD must pass a header + directory bounds check. `verify()` re-checks
  size and SHA-256 only.
- **Custom setups (`setup.rs`, `store.rs`).** One versioned JSON document per setup at
  `<data>/custom/<id>.json`, written temp-file-plus-rename with mode `0600`. The model keeps two
  ordered groups — `files` (WAD/ZIP) and `patches` (DEH/BEX) — because DSDA applies every patch
  after every WAD regardless of argv position. Compatibility is `auto` (no `-complevel`) or one
  reviewed number (`0-17`, `21`); its deserializer refuses placeholders. Ids are `[a-z0-9-]` and are
  also the file name; `slug(name)` plus a numeric suffix picks a free default.
- **Profiles are reused, not duplicated.** A custom setup's runtime directories are the selected
  profile's (`<data>/profiles/<profile id>/`, D-011), and `custom create` writes a
  `<id>-default` profile. Deleting a setup removes DDL's record, and with `--delete-profiles` its
  profiles and saves; user files are never touched (R6).
- **Launch (`launch.rs`).** The plan goes through the existing spec 003 `LaunchPlan` builder:
  absolute paths, separate argv tokens, one `-file` list, one `-deh` list, `-noautoload`, the
  DDL-owned `-config`/`-save`/`-data`/`-shotdir` and working directory. `LaunchPlan::complevel`
  is now `Option<Complevel>` so `auto` emits no flag; catalog launches still pin one. A user-file
  IWAD is re-identified at launch and goes through the D-017 canonical link when it is a known
  variant, otherwise it is passed as-is. Missing/changed files are reported before the engine lookup
  so the blocker names the path even with no engine installed.
- **Extras (R5).** `Profile` gains an additive `extras: Vec<UserFile>`; an older document loads with
  none. `resolver::resolve` verifies every extra, appends WAD/ZIP entries after the recipe's files
  and DEH/BEX entries after the recipe's patches, and returns `Blocker::UserFileMissing` /
  `UserFileChanged` otherwise. The summary lists extras as "Extra file". The CLI warns when a
  changed profile already has `.dsg` saves.
- **CLI (R7).** `ddl custom list|show|create|add-file|move|remove-file|delete`,
  `ddl profile extras <recipe> [--profile …] list|add|move|remove|clear`, and
  `ddl launch <id> [--accept-changes]` resolves either a recipe or a custom setup. `--help` exists
  for every new subcommand.

## New dependency

None. `serde`/`serde_json`, `sha2` (via `iwads::identify::sha256_file`) and `tempfile` were already
in use.

## Requirement → verification

| Item | Evidence |
| --- | --- |
| R1 / A1 | `local/kind.rs`: WAD content accepted as `.wad`/`.lmp`; ZIP content as `.zip`; bounded text as `.deh`/`.bex`; `.pk3`-named ZIP, binary `.deh`, `.xyz` and extensionless files refused; truncated/bad WAD header and >4 GiB refused; directory, relative path, missing file and a symlink-to-directory refused while a symlink-to-file is kept. `tests/local_content.rs` repeats detection and the `.pk3` refusal against `tests/fixtures/local/` (a hand-built PWAD, a stored ZIP, a DEH and a `.pk3` copy, no real game data). |
| R2 / A1 | `local/setup.rs` (round-trip, group routing, duplicate and out-of-range rejection, `validate`, safe ids) and `local/store.rs` (atomic save/load/list/delete, free-id suffixing, bad JSON/schema, no-touch of a referenced user file). |
| R3 / A1 | `local/launch.rs::auto_omits_complevel_and_pinned_emits_it_with_two_ordered_groups` asserts exact argv: no `-complevel` for Auto, `-complevel 21` when pinned, one `-file` list then one `-deh` list in model order, `-noautoload`, skill 4, pistol start. `a_user_file_iwad_is_passed_through_without_the_registry` covers the user-file IWAD. |
| R4 / A1 | `kind.rs::identity_check_reports_missing_and_changed_files`, `launch.rs::missing_and_changed_files_block_and_accepting_rerecords_only_changes`, `resolver/tests.rs::profile_extras_load_after_the_recipe_files_and_patches` (missing → `UserFileMissing`, changed → `UserFileChanged` then accepted), and the real-binary round trip below. |
| R5 / A2 | `resolver/tests.rs::profile_extras_load_after_the_recipe_files_and_patches` (recipe files then extras, recipe patch then extra patch; summary "Extra file"); `profiles/store.rs::a_profile_written_before_spec_026_loads_without_extras`; the CLI test asserts the saves warning on stderr. |
| R6 / A1 | `store.rs::deleting_never_touches_a_referenced_user_file`; the CLI round trip removes a file then deletes the setup and asserts the player file still hashes the same. |
| R7 / A1 | `cli.rs` parser tests for every new subcommand and rejection; `tests/custom_cli.rs` drives the real binary in a scratch XDG root; `tests/cli_help.rs` covers the new `--help`. |
| A2 | The extras resolver test plus the CLI `profile extras` round trip (saves warning, then remove/clear keep the files). |
| A4 | Command tails below. |
| A3 / A5 | Open — phase B and **Needs Deck Validation** respectively. |

### Exact argv (core test, abridged)

Auto, a user WAD + ZIP and one DEH patch (no environment paths):

```text
-iwad <canonical doom2 link> -file <first.wad> <second.zip> -deh <fix.deh> -skill 4 -noautoload -config <cfg> -save <dir> -data <dir> -shotdir <dir> -pistolstart
```

Pinned `--complevel 21` inserts `-complevel 21` between `-deh` and `-skill`. A catalog mapset with
extras resolves to `-file <recipe maps> <extra wad> … -deh <recipe patch> <extra deh>`.

### CLI round trip (scratch XDG root, real binary)

```text
$ ddl custom create "My Setup" --id mine --iwad-file <iwad> --complevel 21
Created custom setup `mine` (My Setup)
$ ddl custom add-file mine <player.wad> <fix.deh>
mine: 1 file(s), 1 patch(es)
$ ddl custom add-file mine <stray.pk3>
ddl: `<stray.pk3>` ends in `.pk3`, which DSDA-Doom's `-file` rejects; accepted extensions are .wad, .lmp, .zip, .deh and .bex
$ ddl custom move mine files 0 0
mine: 1 file(s), 1 patch(es)
# after editing player.wad:
$ ddl launch mine --dry-run
ddl: cannot launch `mine`: `<player.wad>` changed since DDL recorded it; confirm the change to use it
$ ddl launch mine --dry-run --accept-changes
ddl: DSDA-Doom is not installed
$ ddl custom show mine
mine  My Setup
  IWAD: <iwad>
  compatibility: 21
  files (load order):
    0: <player.wad>  wad  28 bytes
  patches:
    0: <fix.deh>  deh  53 bytes
  files: ok
$ ddl custom delete mine --delete-profiles
Deleted custom setup `mine` and its profiles and saves
$ ddl custom list
No custom setups.
```

## UI-facing API for phase B

Types (all under `ddl_core::local`):

- `Kind { Wad, Zip, Deh }` — `is_load_file()`, `is_patch()`, `as_str()`.
- `UserFile { path: PathBuf, size: u64, sha256: String, kind: Kind }` — `UserFile::read(&Path) ->
  Result<Self, FileError>`, `verify() -> Result<(), IdentityError>`, `rehash()`.
- `CustomIwad { Identified { variant: String }, File { file: UserFile } }`.
- `Compatibility { Auto, Level(Complevel) }` — `parse(&str)`; JSON is `"auto"` or a number.
- `CustomSetup { schema_version, id, name, iwad, files: Vec<UserFile>, patches: Vec<UserFile>,
  compatibility, created_at, updated_at }` with `add`, `remove(Group, index)`,
  `move_entry(Group, from, to)`, `validate()`.
- `Group { Files, Patches }`, `group_indices(&[UserFile], Group) -> Vec<usize>` (for a profile's
  single `extras` list).
- `SetupStore::new(data_dir)` — `list()`, `load(id)`, `load_optional(id)`, `save(&setup)`,
  `delete(id)`, `free_id(name)`, `contains(id)`, `path(id)`, `root()`; records live at
  `<data>/custom/<id>.json`.
- `build_plan(&CustomSetup, &Profile, &Engine, &Registry, data_dir, &DsdaConfigManager) ->
  Result<LaunchPlan, LaunchError>`; `verify_setup`, `first_issue`, `accept_changed`,
  `recipe_game`, `assign_allowlist`.
- `has_saves(&Path)` for the R5 warning. `Profile::extras: Vec<UserFile>` (additive).
- Errors: `FileError`, `IdentityError`, `SetupError`, `LaunchError` (all `Display`); resolver
  blockers `Blocker::UserFileMissing { path }`, `Blocker::UserFileChanged { path }`.
- Runtime directories are the profile's `ProfileDirs` (`<data>/profiles/<profile id>/{save,data,shots,run}`),
  so a custom setup needs a `Profile` (recipe id = the setup id). `Services::custom_*`,
  `Services::extras_*`, `Services::custom_resolution`, `Services::run_custom` and
  `Services::accept_extra_changes` wrap them for the CLI and are available to phase B's wiring.
- Phase B still needs: file-browser filtering by `Kind`, the order editor over `files`/`patches`
  and `Group`/`group_indices`, an entry point for `custom create`, extras on the Play setup, and the
  "Custom — not verified" badge. Custom setups are **not** yet in `Snapshot`; phase B should add
  them there (or consume `SetupStore`/`Services::custom_list`).

## Open questions

- **User-file IWAD mission.** D-017 passes every IWAD through a canonical-name symlink because DSDA
  derives the *mission* from the file name. A user-file IWAD that identification does not resolve to
  a known variant is passed as-is, so its mission follows its file name. Flag for deck validation.
- **`.lmp`.** Accepted as WAD content (the engine's WAD loader). A raw `.lmp` lump that is not a WAD
  is refused by DDL though the engine's parser accepts it syntactically. Confirm this is the intended
  reading of R1.
- R2's "its own profiles" is implemented by reusing spec 018's `ProfileStore` with the setup id as
  the recipe id; a per-setup file browser and multi-profile UI are phase B.

## Final verification

All commands ran on macOS arm64 after `cargo fmt`; no external-network or physical Deck validation
was run.

```text
cargo fmt --all --check
  exit 0 (no output)

cargo clippy --workspace --all-targets -- -D warnings
  Checking ddl-core v0.0.5 (.../crates/ddl-core)
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.33s

cargo test --workspace   (exit 0)
  ddl unit:        test result: ok. 94 passed; 0 failed
  catalog_cli:     test result: ok. 3 passed; 0 failed
  cli_help:        test result: ok. 2 passed; 0 failed
  custom_cli:      test result: ok. 2 passed; 0 failed
  logo_assets:     test result: ok. 2 passed; 0 failed
  ui_strings:      test result: ok. 1 passed; 0 failed
  ddl-core unit:   test result: ok. 371 passed; 0 failed
  ddl-launch-probe: 0 tests
  dsda_launch:     test result: ok. 16 passed; 0 failed
  local_content:   test result: ok. 2 passed; 0 failed
  Doc-tests:       test result: ok. 3 passed; 0 failed
```

496 tests passed. The headless UI suite (`ddl` unit, ~196 s) is included and green with the added
`Blocker` match arm.

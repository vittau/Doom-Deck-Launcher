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

## Phase B — local content UI

Implemented R8–R11 in the production runtime, with focused headless UI coverage for A3 and the
UI share of A4. No new core query or dependency was required; the state-lock lifetime fix below was added after the coordinator requested investigation of a parallel-test regression. `main` was merged before
finishing (includes specs 024 and 025); custom titles reuse spec 025's measured `Nodes::title`
and local file rows use its fitted-label renderer. No visible window was opened and no screen
was captured.

### Changes

- `Snapshot` now carries custom setups, profiles, worker-checked file issues and the profile ids
  that have saves. `LocalCommand` routes reads, validation, changes and launch to the existing
  services worker thread. Custom launch calls `run_custom`, using its existing run guard,
  fakeable engine boundary, isolated config and profile save directories. Extra changes call
  `accept_extra_changes`; custom acceptance also covers a changed player-file IWAD.
- Library has **New custom setup**, also reachable on Y, with the first unused automatic name.
  Controller pickers offer identified IWAD variants, a player file with an `IWAD` header, and
  Auto or the reviewed compatibility numbers. The player can edit these choices from custom
  detail. Custom cards and Home Continue/history use the existing painted fallback art and
  **Custom — not verified**; profiles with extras receive the same badge.
- The file picker extends the existing directory service, begins at home and supplies home,
  existing Downloads/Documents, Steam-library and `/run/media/...` mount shortcuts. It uses
  `UserFile::read` for accepted files, shows detected kind/size, keeps selection across folders,
  toggles with A, confirms with Start or **Add N files**, and cancels with B. Invalid suffix or
  content combinations are hidden. Unreadable/broken entries are skipped. Full paths remain in
  the model and selected-directory breadcrumb; shortcut rows stay compact, and long directory
  paths are shortened for display.
- The order editor keeps **Files (load order)** and **Patches (applied after files, in order)**
  separate. A picks up/drops, Up/Down persists moves within a group, X removes only the
  reference, Y adds. Catalog extras apply to the active profile; enabled recipe members are
  derived from the installed manifest in recipe order, dimmed, locked and skipped by focus.
  Opening extras with existing saves warns that saves may not load after changes.
- Missing/changed-file sheets name the path and block Play. Cancel has initial focus; a changed
  file requires **Use the changed file**. Custom deletion names the setup, states that the
  player's files are never touched, and defaults the D-022 profiles/saves checkbox to unchecked.
  The checked and unchecked paths both use the phase A deletion operation.

### Focused headless verification

The pre-existing long headless journey was **not extended**. Slint permits one platform setup
per process, so five independently named tests re-execute only their own case in a headless
subprocess. Each subprocess uses `MinimalSoftwareWindow` and the Slint software renderer;
calling `show()` there creates only the off-screen surface. No native backend is initialized.

| Requirement | Test evidence |
| --- | --- |
| R8 / R10 / R11 | `local_new_setup_iwad_compatibility_and_content_filtered_multiselect`: Library Y entry, home start, Downloads/Documents shortcuts, false-WAD and PK3 rejection, IWAD-only filtering, Auto creation, persisted explicit level 21 and identified-IWAD selection via the existing fake registry seam, two A-selected files, Start addition and B cancellation. |
| R9 | `local_order_moves_with_dpad_stays_in_group_and_removes_without_deleting`: D-pad focus into the first file, pickup, move, boundary clamp before patches, drop, X removal, original file retained. |
| R10 / R4 / R6 | `local_custom_library_home_missing_changed_and_delete_dialog`: Library focus order, custom Continue focus/fallback art/history, missing named path, changed-file Cancel → Right → accept (including a changed player-file IWAD), delete Cancel → Up → checkbox, reset on reopening, unchecked deletion retaining profiles and all player files. |
| R5 / R9 / R10 | `local_catalog_extras_lock_recipe_members_warn_for_saves_and_keep_profile_scope`: a second active catalog profile, saves warning, recipe member locked/dimmed before extras, D-pad skips it, persisted reordering, default profile still has no extras. |
| R6 | `local_delete_checked_removes_only_owned_profiles_and_saves`: checkbox and confirm through the production click dispatcher, owned profile/save directories removed, all player WAD/ZIP/DEH and IWAD files retained. |

The fixtures reuse phase A's hand-built `crates/ddl-core/tests/fixtures/local/{tiny.wad,tiny.zip,
tiny.deh,tiny.pk3}` plus an original 12-byte, empty-directory IWAD at
`crates/ddl/tests/fixtures/local/base.wad`. They contain no game data. No external network is
used: the extras test reuses the existing loopback-only installed-mapset fixture.

Off-screen PNGs are written to `target/headless/026-*.png` (or `DDL_HEADLESS_DIR`): new custom,
IWAD picker, compatibility, custom detail, file-picker multiselect, order mid-move, Library,
Home, Play setup extras, extras order, saves warning, missing file, changed file and delete.
The worker opened the generated PNG files for visual inspection. Iteration corrected dialog
focus theft, long-list panel height, moving-row contrast, custom Home history/art and compact
shortcut labels; the final renders retain the Ember frame, readable body text and amber ring.

Focused rerun:

```sh
cargo test -p ddl runtime::local_ui -- --nocapture
# test result: ok. 5 passed; 0 failed
```

### Remaining validation

**Needs Deck Validation:** A5 remains open: physical SD-card shortcuts, controller timing, launch
and return from real DSDA-Doom with a custom PWAD, extras over a catalog mapset, and moving a
referenced file. The mount enumeration shares the existing spec 010 production path; the macOS
host has no `/run/media` mounted card to exercise. Auto compatibility, file kinds and DSDA flags
are the phase A policy and pinned upstream facts above; phase B adds no engine assumptions.
Optional renaming/on-screen keyboard remains omitted by the spec's explicit allowance for an
automatic name. No other phase B requirement is left open.


### Parallel-test regression and state-lock lifetime fix

The first full run hit the known spec 025 fit-report directory failure. The fixed spec 025 was
then merged from `main` (merge `b456099`), resolving the shared helper conflict by retaining
`headless_dir()` and exposing `write_png` to the focused tests. No spec 025 behavior was rewritten.
A subsequent run passed the long journey but failed once in the existing
`cli_exec::tests::launch_profile_selects_isolated_paths_and_remembers_without_saving_overrides`
at `NewProfile` immediately after `register_fake_iwads`, with `Problem::Busy`. That test passed
when rerun alone. The coordinator required a source investigation and three consecutive full
**default parallel** workspace runs; no thread limit or test skipping was used as a fix.

**Confirmed audit:** `StateLock` opens `<config>/state.lock` with Rust `OpenOptions` and calls
`std::fs::File::try_lock`; on this Unix host the operation is `flock(LOCK_EX | LOCK_NB)`, and
`unlock` is `flock(LOCK_UN)` ([Rust 1.99.1 File API](https://doc.rust-lang.org/1.99.1/std/fs/struct.File.html#method.try_lock)).
The guard previously had no `Drop` implementation: normal completion only closed its `File`.
No production `dup`, `try_clone`, raw libc open, `FromRawFd` or explicit descriptor inheritance
exists in the state-lock path. The docs specify that inherited/duplicated references can extend
a lock until their last close; explicit unlock ends it immediately
([Rust 1.99.1 unlock contract](https://doc.rust-lang.org/1.99.1/std/fs/struct.File.html#method.unlock)).

**Confirmed environment audit:** the parent focused tests launch exactly one named test with
`--exact`, and the child sets up its own fresh `TempDir`. Its `Environment` resolves paths through
a closure that supplies only that temporary HOME and returns None for all XDG variables; inherited
real HOME/XDG values do not select application state. Every focused case and every existing CLI
fixture owns a separate scratch root. Only PNG output is shared, with distinct filenames; it is
not a state-lock path.

**Confirmed reproduction (macOS arm64, pinned Rust 1.99.1):** a standalone probe, independent of
DDL, repeatedly opens/locks/drops one file while another thread runs 250 self-exec subprocesses.
The child has `env_clear()` and immediately exits, so there is no DDL setup, shared XDG state,
network or competing mutation. `fcntl(F_GETFD)` returns `1` on the lock fd in both variants;
the installed macOS SDK header confirms `F_GETFD = 1` and `FD_CLOEXEC = 1`.

```text
implicit close: 43,316 acquisitions attempted; 95 transient WouldBlock results; FD_CLOEXEC set
explicit unlock: 41,569 acquisitions attempted; 0 WouldBlock results; FD_CLOEXEC set
```

This demonstrates that close-on-exec did not by itself make the guard's logical release timely
under concurrent process creation. The precise kernel spawn stage was not traced; a transient
inherited open-file-description reference is the inference consistent with this reproduction
and the documented semantics. The source defect is relying on the descriptor's final close
instead of ending the mutation lock at guard drop.

**Fix:** `StateLock::drop` explicitly unlocks. `end()` records a successful release so Drop does
not unlock twice (important for the documented platform-dependent no-lock behavior). The normal
mutual-exclusion contract is unchanged. The new deterministic test
`dropping_the_guard_releases_even_with_another_descriptor_alive` keeps a duplicated descriptor
alive after dropping the guard: it fails against the previous Drop behavior with Busy, passes
with explicit unlock, and verifies that a newly acquired guard still excludes other acquisitions.
This small core change was authorized by the coordinator's follow-up investigation request; it
adds no dependency and no UI code to `ddl-core`.

```text
before fix: test result: FAILED. 0 passed; 1 failed (Busy)
after fix: platform::lock::tests — 3 passed; 0 failed
```

Reproduction source (diagnostic only; the F_GETFD call is macOS-specific):

```rust
use std::{fs::OpenOptions, sync::{Arc, atomic::{AtomicBool, Ordering}}, process::{Command, Stdio}, thread, os::fd::AsRawFd};
unsafe extern "C" { fn fcntl(fd: i32, cmd: i32, ...) -> i32; }
fn main() {
    if std::env::args().any(|s| s == "--child") { return; }
    let explicit = std::env::args().any(|s| s == "--unlock");
    let root = std::env::temp_dir().join(format!("ddl-lock-probe-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("state.lock");
    let running = Arc::new(AtomicBool::new(true)); let flag = running.clone();
    let worker = thread::spawn(move || {
        for _ in 0..250 {
            Command::new(std::env::current_exe().unwrap()).arg("--child")
                .env_clear().stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status().unwrap();
        }
        flag.store(false, Ordering::SeqCst);
    });
    let (mut attempts, mut busy) = (0u64,0u64); let mut flags = -1;
    while running.load(Ordering::SeqCst) {
        let file = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&path).unwrap();
        flags = unsafe { fcntl(file.as_raw_fd(), 1) };
        match file.try_lock() {
            Ok(()) => if explicit { file.unlock().unwrap(); },
            Err(std::fs::TryLockError::WouldBlock) => busy += 1,
            Err(e) => panic!("{e:?}"),
        }
        attempts += 1;
    }
    worker.join().unwrap();
    println!("explicit_unlock={explicit} attempts={attempts} busy={busy} fd_flags={flags} (FD_CLOEXEC=1)");
    std::fs::remove_dir_all(root).unwrap();
}
```

Compile with `rustc --edition 2024 probe.rs -o probe`, then run `./probe` and `./probe --unlock`.
The production regression test uses safe `File::try_clone` and contains no libc calls.


### Final phase B verification (A3, A4)

`main` through `4f0e569` (fixed spec 025 plus spec 024) is merged. The code stayed unchanged
through the three consecutive full runs below. `RUST_TEST_THREADS`, `DDL_HEADLESS_DIR`,
`DDL_TITLE_ONLY` and `DDL_TITLE_RECIPE` were all unset: this is the default parallel harness,
with the default artifact directory and the complete long headless journey. Every run exited 0
and passed **505 tests**; no test was ignored or filtered. The five focused local-content cases
are included in every run. All 14 `026-*.png` frames were inspected off-screen.

```text
cargo fmt --all --check
exit 0 (no output)

cargo clippy --workspace --all-targets -- -D warnings
Checking ddl-core v0.0.5 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-026b-local-content-ui/crates/ddl-core)
    Checking ddl v0.0.5 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-026b-local-content-ui/crates/ddl)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.43s
```

Default parallel run 1, `cargo test --workspace` (exit 0):

```text
test result: ok. 98 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 151.45s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 376 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Default parallel run 2, `cargo test --workspace` (exit 0):

```text
test result: ok. 98 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 191.21s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.68s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.04s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 376 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.94s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Default parallel run 3, `cargo test --workspace` (exit 0):

```text
test result: ok. 98 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 781.72s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.27s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.62s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.07s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 376 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.42s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

A3 is checked in the spec. A4 remains checked with the UI and lock-fix verification above.
A5 remains **Needs Deck Validation**. No open implementation question remains for phase B.

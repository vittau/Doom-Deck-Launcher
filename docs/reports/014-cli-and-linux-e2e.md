# 014 — Headless CLI and the real-network Linux e2e job

## Result and rationale

**Confirmed:** R1–R4 are implemented. `ddl` gained `engine status`,
`engine install [--version <V>] [--force-extract]`, `iwad scan [PATH...]`,
`mapset list`, `mapset install <id> [--with <component>...]`,
`launch <id> [--dry-run] [--with …] [--skill N] [--pistol-start]` and `doctor`.
Every subcommand drives the **same** `crates/ddl/src/services.rs` `Services` the UI's
worker thread uses — same XDG paths, same `app.json`, engine state, IWAD registry,
library manifests and profiles — so a UI started afterwards sees what the CLI
installed, and vice versa. Each subcommand has `--help` and exits 0; failures exit
non-zero with one actionable line. Nothing here ever creates a window.

Two small, additive `ddl-core` changes back the CLI:

- `install::InstallRequest` gained `force_extract`, and `stage()` takes the D-013
  extraction fallback before the AppImage's `-v` probe when it is set. The reviewed
  download, size and SHA-256 checks are unchanged.
- `platform::lock::StateLock` is a new advisory-OS-lock guard for DDL's mutable state
  (see **The state lock** below).

## R1 — Subcommands

`crates/ddl/src/cli.rs` is still a small hand-written parser, now with one parser per
subcommand. `--version` is only DDL's at the top level, so `ddl engine install
--version 0.30.1` means the engine's version. `crates/ddl/src/cli_exec.rs` executes a
parsed command against an explicit `Environment` and writer pair, which is what lets
the tests drive the real services against the loopback fakes without capturing a
process's stdout. `main.rs` routes every non-UI command through `cli_exec` after
initialising logging (so the e2e artifact gets the logs).

`ddl launch --dry-run` prints `program:`, then every argv token on its own line, then
`working directory:` — one argument per line, as R1 asks. It resolves through the
same private `resolve_launch` that the UI's `Command::Play` now uses, so the dry-run
argv and the launched argv cannot drift. Without `--dry-run` it launches through the
same run guard; the CLI does not persist the `--with`/`--skill`/`--pistol-start`
overrides (they are a one-off), which the UI's Play does for `last_played`.

### `--help` samples

```text
$ ddl --help
ddl: DoomDeckLauncher, a native DOOM mapset launcher for Steam Deck

Usage: ddl [OPTIONS]
       ddl catalog lint [--fetch] [PATHS...]
       ddl engine status
       ddl engine install [--version <V>] [--force-extract]
       ddl iwad scan [PATH...]
       ddl mapset list
       ddl mapset install <id> [--with <component>...]
       ddl launch <id> [--dry-run] [--with <component>...] [--skill <1-5>] [--pistol-start]
       ddl doctor
...
```

```text
$ ddl engine install --help
ddl engine install [--version <V>] [--force-extract]

Download a reviewed DSDA-Doom AppImage, verify its size and SHA-256,
smoke-test it with `-v` and install it. Progress goes to stderr.

Usage: ddl engine install [--version <V>] [--force-extract]

Options:
      --version <V>     Reviewed version (0.30.1) or tag (v0.30.1);
                        default: the newest reviewed release
      --force-extract   Extract the AppImage even when it would run, so the
                        D-013 fallback can be validated on purpose
```

```text
$ ddl launch --help
ddl launch <id> [--dry-run] [--with <component>...] [--skill <1-5>] [--pistol-start]

Resolve a mapset exactly as the UI's Play does. With --dry-run it prints the
argv (one argument per line) and the working directory, or the typed blocker
with its explanation. Without --dry-run it starts DSDA-Doom through the same
run guard as the UI.
...
```

`ddl iwad scan --help`, `ddl mapset list --help`, `ddl mapset install --help`,
`ddl engine status --help`, `ddl engine --help`, `ddl iwad --help`,
`ddl mapset --help` and `ddl doctor --help` are covered by the `cli_help`
integration test and by `cli::tests::per_topic_help_mentions_its_own_command`.

### Sample runs against the test fakes

From `cli_exec::tests::sample_output_show` (a temporary run; the assertions live in
the permanent tests) against a loopback HTTP server, a fake engine script and a
scratch XDG root — paths elided to `…` for readability:

```text
$ ddl engine status
engine: not installed

$ ddl engine install
Installed DSDA-Doom 0.30.1 at …/engines/dsda-doom/0.30.1/dsda-doom.AppImage (appimage)
download
  0% (0 / 248 bytes)
  100% (248 / 248 bytes)
verify
commit

$ ddl engine status
installed:
  0.30.1  appimage  reviewed  active
active: 0.30.1

$ ddl engine install --force-extract   # fresh data dir
Installed DSDA-Doom 0.30.1 at …/engines/dsda-doom/0.30.1/squashfs-root/AppRun (extracted)
download
  0% (0 / 248 bytes)
  100% (248 / 248 bytes)
verify
extract
commit

$ ddl mapset list
sigil              doom     not installed  SIGIL

$ ddl mapset install sigil
Installed sigil 1.23
  …/library/sigil/1.23/free/SIGIL_V1_23.wad
  …/library/sigil/1.23/free/SIGIL_V1_23.txt
  …/library/sigil/1.23/free/SIGIL_README_V1_23.txt
...

$ ddl iwad scan <dir>
…/iwads/maybe.wad (ambiguous: doom2-1.9, doom2-1.8, …, tnt-1.9, plutonia-1.9) sha256=3284b061…
unrecognized: …/iwads/junk.wad

$ ddl launch sigil --dry-run
(error) cannot launch: doom is missing; add a folder containing your own copy (accepted: doom-1.9ud)

$ ddl launch sigil --dry-run --skill 4 --pistol-start   # with the IWAD seam
program: …/engines/dsda-doom/0.30.1/dsda-doom.AppImage
-iwad
…/iwads/doom-1.9ud/doomu.wad
-file
…/library/sigil/1.23/free/SIGIL_COMPAT_V1_23.wad
-complevel
3
-skill
4
-pistolstart
-noautoload
-config
…/config/ddl/dsda/active.cfg
-save
…/profiles/sigil-default/save
-data
…/profiles/sigil-default/data
-shotdir
…/profiles/sigil-default/shots
working directory: …/profiles/sigil-default/run

$ ddl doctor
DoomDeckLauncher 0.0.3 doctor
paths:
  config: …/config/ddl
  data:   …/data/ddl
  cache:  …/cache/ddl
  state:  …/state/ddl
  logs:   …/state/ddl/logs
engine:
  state file: …/data/ddl/engines/dsda-doom/installed.json
  0.30.1  appimage  …/engines/dsda-doom/0.30.1/dsda-doom.AppImage
  active: 0.30.1
  previous good: none
iwads:
  …/iwads/maybe.wad (ambiguous)
  …/seam-iwads/doom-1.9ud.wad (doom-1.9ud)
mapsets:
  sigil              doom     installed
dsda config:
  active: …/config/ddl/dsda/active.cfg
  usable: no
  template: 1
  conflicts: 0
```

## R2 — Same state as the UI, and concurrent mutation

The subcommands call `Services::open`, so they read and write exactly the UI's files.
The dry-run is read-only; `mapset install` saves the default profile before installing
(mirroring the UI's `Command::Install`) and writes the install manifest and `app.json`.

### The state lock

Spec 005's `RunGuard` covers *an engine run*, and the engine/config managers already
refuse to change the engine set or the config while it is held. It deliberately does
**not** cover installs, profile edits or registry refreshes, so nothing stopped a UI
and a CLI process from racing on the same files. A small lock was therefore missing,
and `crates/ddl-core/src/platform/lock.rs` adds one:

- one advisory OS lock at `<config>/ddl/state.lock`, acquired non-blocking;
- the OS releases it when the process dies, so no stale-PID recovery is needed;
- `Services::execute` takes it for every mutating `Command` (a `mutates_state`
  predicate), and the CLI's `engine install`, `mapset install`, `iwad scan` and the
  test seam take it too;
- `Command::Play`, `Browse` and `Updates` deliberately do not: Play holds the run
  guard for the whole game, and a two-hour game must not block a CLI install;
- a refused mutation is a clear message (`another DDL process is changing DDL's
  state; close it and try again`), surfaced as the new `Problem::Busy` in the UI and
  as a one-line error in the CLI.

The UI needed two new translated strings (`error-busy` and `error-engine-runtime`)
and their `Problem` arms; no UI flow changed.

## R3 — The Linux e2e workflow

`.github/workflows/e2e.yml` is `workflow_dispatch` only. After the first run
([38980417407](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/38980417407))
failed in the engine step in both jobs, it was restructured to prove what matters:

- `build` builds `ddl` with `--features test-support` **inside the same pinned sniper
  SDK container digest** used by `ci.yml`/`release.yml` (mirroring their toolchain
  install and cargo cache) and uploads `target/release/ddl` as an artifact. It also
  runs one informative `continue-on-error` step that downloads the official AppImage,
  extracts it, runs `AppRun -v` inside the container and writes the exact output to
  `$GITHUB_STEP_SUMMARY` — the glibc evidence below.
- `e2e` (`needs: build`) runs the pre-built binary on a plain `ubuntu-latest` host — a
  normal host like SteamOS — after installing the runtime libraries the binary and the
  engine need, and runs every real-network step.

The runtime library list is deliberately minimal and commented: `libfontconfig1` is
the one non-glibc library `ddl` hard-links (`docs/research/steam-runtime-build.md` §6),
and `libgl1 libopengl0` supply the GL entry points the official AppImage does not
bundle (the first run's `libOpenGL.so.0` failure). Any further missing library the
smoke test names belongs there.

The `e2e` run, against real upstream hosts and a scratch XDG root, is:

1. `ddl engine install` (AppImage first, then the `-v` smoke test) and, in a fresh
   data dir, `ddl engine install --force-extract`; the second must report `extracted`.
2. `ddl mapset install` for every id from `ddl mapset list`, then every free optional
   component (`sigil --with compat`, `auger-zenith --with alternate-blaster`, then
   `--with deckard-blaster` one at a time).
3. `ddl launch <id> --dry-run` for every recipe must exit non-zero with the typed
   `MissingIwad` blocker; then, with `DDL_TEST_IWAD_SEAM` set, the dry-run argv is
   asserted against D-011: exactly one `-file`, `-iwad`, `-complevel`, `-skill`,
   `-noautoload`, `-config`, `-save`, `-data` and `-shotdir`; a numeric `-complevel`;
   an absolute `-iwad`.
4. `ddl doctor` and the logs are uploaded as a workflow artifact, with `if: always()`.
   The AppImage-direct (FUSE) result is recorded in the step summary, not asserted.

**The IWAD seam.** `ddl`'s `test-support` feature (off by default; `release.yml`
builds without features) exposes `Services::register_fake_iwads`, which writes a
**sparse** file of each needed variant's exact size and records it in the IWAD
registry. It never downloads anything, and the resolver's dry-run path only reads the
file's size. It is only reachable when `DDL_TEST_IWAD_SEAM` names a directory, and the
same function is exercised offline by `launch_dry_run_prints_the_d011_argv_with_a_registered_iwad`.

I cannot run this workflow from this worktree: it must exist on `main` before the
coordinator can dispatch it, and it downloads real upstream bytes. The YAML is
validated locally with PyYAML in a venv, and every `run:` block with `bash -n`.

## First e2e run: findings and the runtime diagnosis

The first real run
([38980417407](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/38980417407))
failed at `ddl engine install` in both jobs, and both failures are host facts, not DDL
bugs:

- **Plain Ubuntu runner:** the AppImage mounted directly via FUSE, then the `-v`
  smoke test exited 127 with `libOpenGL.so.0: cannot open shared object file`. The
  runner lacks `libopengl0`; SteamOS ships libglvnd. **FUSE itself works on plain
  Ubuntu.**
- **Steam Runtime SDK container (sniper, Debian 11, glibc 2.31):** no FUSE, so DDL
  correctly extracted the AppImage, then `AppRun -v` failed with `GLIBC_2.34 not
  found`, `GLIBC_2.33 not found` and `GLIBCXX_3.4.29 not found`. The official
  AppImage does not bundle glibc and needs **≥ 2.34**. On the Deck a non-Steam
  shortcut runs on the host by default; it only breaks if a Steam "Compatibility
  tool" is forced on DDL's shortcut.

**Product fix.** `crates/ddl-core/src/engines/dsda/smoke.rs` now classifies engine
startup stderr into a typed `RuntimeError`:

- `GLIBC_… not found` / `GLIBCXX_… not found` → `RuntimeError::Glibc`, whose message
  names the missing symbols and says to remove any Compatibility tool from DDL's
  Steam shortcut so DSDA-Doom runs on SteamOS directly;
- `error while loading shared libraries: <lib>` → `RuntimeError::MissingLibrary`,
  whose message names the library and says to install the package that provides it,
  or run DDL on a system that has it (kept neutral — see follow-up 2).

`smoke_test_with_timeout` returns `SmokeError::Runtime` for these, leaving the FUSE
`Exited` path intact so the extraction fallback still runs. `InstallError` prints the
message directly instead of wrapping it as a generic smoke-test failure; `Services`
maps it to the new `Problem::EngineRuntime`, which the UI renders through the new
`error-engine-runtime` string and the CLI prints as-is. Unit tests use the exact
captured stderr lines, plus a manager install test and two CLI tests.

## Follow-up 2: every missing library at once, and SteamOS coverage

The second run
([38982231350](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/38982231350))
passed the build job and the host job's install step, then the diagnosis worked:
`ddl: DSDA-Doom could not start because the system library libjack.so.0 is missing.`
Discovering one library per run is slow, so:

- **The host job now sweeps once.** Before `ddl engine install` it downloads the exact
  reviewed AppImage (tag/asset/SHA-256 read from `crates/ddl-core/data/engines.toml`,
  digest verified), extracts it, and runs `ldd` on `usr/bin/dsda-doom` and every
  bundled `usr/lib/*.so` with the bundled directory on `LD_LIBRARY_PATH`. The
  deduplicated `not found` set and the full `ldd` output go to
  `$GITHUB_STEP_SUMMARY`; the step is `continue-on-error` and never fails.
- **The install step now installs the whole derived set**, each package commented with
  the library it provides: `libjack-jackd2-0` (libjack.so.0), `libgpg-error0`,
  `libdrm2`, `libgbm1`, `libwayland-client0`, `libx11-6`, `libx11-xcb1`, `libxcb1`,
  plus the earlier `libfontconfig1`/`libgl1`/`libopengl0`; `libasound2t64` falls back
  to `libasound2` for the 64-bit `time_t` rename.
- **The message is neutral.** `RuntimeError::MissingLibrary` no longer claims
  "(SteamOS does)"; it says to install the package that provides the library, or run
  DDL on a system that has it.

I derived the set by extracting the reviewed v0.30.1 AppImage locally (SHA-256
`b06adf3f…`) and reading every bundled ELF's `DT_NEEDED`. The engine's
`usr/bin/dsda-doom` links the bundled `libfluidsynth.so.3`, which links
`libjack.so.0` — a **hard, transitive** dependency, not an optional `dlopen`. The
complete non-bundled set is: `libGL.so.1`, `libOpenGL.so.0`, `libX11.so.6`,
`libX11-xcb.so.1`, `libxcb.so.1`, `libasound.so.2`, `libdrm.so.2`, `libgbm.so.1`,
`libwayland-client.so.0`, `libgpg-error.so.0`, `libjack.so.0`, `libz.so.1`,
`libstdc++.so.6`, `libgcc_s.so.1`, and glibc.

**SteamOS coverage (Confirmed, cited in `docs/research/dsda-engine.md` §7).** SteamOS
3.8.0's package manifest lists `pipewire-jack` (and no `jack2`), and the SteamOS
`jupiter-rel` mirror's own `pipewire-jack` package installs `usr/lib/libjack.so.0`;
the same manifest lists glibc 2.41, libglvnd, mesa, alsa-lib, libdrm, libx11, libxcb,
wayland, libgpg-error, gcc-libs and zlib. So the Deck provides every library the
engine needs, **no DDL-side mitigation is required**, and DDL will not bundle system
libraries without a separate owner decision.

## R4 — Docs

- README gained a **Troubleshooting from Konsole** section listing every subcommand, a
  few examples and common fixes, and its command-line-options block now mirrors
  `ddl --help`. Its install steps now warn **not** to set a Steam Compatibility tool on
  the DDL shortcut, with the glibc reason.
- `docs/deck-testing.md` E3 now uses `ddl engine install --force-extract`, and its
  **How to install a build** step warns against the Compatibility tool.
- `docs/research/dsda-engine.md` §7 records the e2e findings as **Confirmed (CI run
  38980417407)**.

## Verification

macOS arm64, pinned repository toolchain. **344 tests passed, zero failures** across
the workspace (ddl 47, cli_help 2, catalog_cli 2, logo_assets 2, ui_strings 1,
ddl-core 271, dsda_launch 16, doctests 3, plus the 0-test probe bin).

`cargo fmt --all --check` — exit 0, no output.

`cargo clippy --workspace --all-targets -- -D warnings` — exit 0, tail:

```text
   Compiling ddl-core v0.0.3 (.../crates/ddl-core)
   Compiling ddl v0.0.3 (.../crates/ddl)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.03s
```

`cargo clippy -p ddl --features test-support --all-targets -- -D warnings` — exit 0,
so the e2e seam also compiles clean under the feature the workflow builds with.

`cargo test --workspace` — exit 0, result lines:

```text
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 48.94s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 271 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Tests added: `cli::tests` (per-subcommand parsing and per-topic help), `cli_exec::tests`
(eleven offline runs against the loopback server, fake engine, scratch XDG: engine
install and status, `--force-extract`, the two runtime-failure diagnoses, mapset
list/install, component validation, IWAD scan with ambiguous and unrecognized files,
the MissingIwad blocker, the dry-run argv against D-011, doctor, and the busy state
lock), `platform::lock::tests`, the manager's `force_extract` and
`a_runtime_smoke_failure_is_reported_actionably` tests, the smoke classifier unit
tests, the two `dsda_launch` runtime-classification tests, the `runtime` UI-detail
test, and the `cli_help` integration test. No test touches the network, opens a window
or captures the screen.

## Open questions and items left open

- **A2** stays unchecked: the first two e2e runs
  ([38980417407](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/38980417407),
  [38982231350](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/38982231350))
  each failed at the engine step on a missing *host* library. The workflow now sweeps
  the whole AppImage `DT_NEEDED` set once and installs it; the coordinator re-triggers
  it after merge and will forward any failure. The report should then link the passing
  run and the AppImage-direct result.
- **A4** stays unchecked: `ddl doctor` and `ddl launch` from Konsole on the Deck are a
  hardware check, as is confirming the engine runs on the host without a forced
  Compatibility tool.
- **Owner decision, if it ever comes up:** bundling the engine's system libraries is
  explicitly *not* done. SteamOS 3.x provides all of them (research §7), so no
  mitigation is needed today; a future redistribution would be a separate decision.
- New dependencies: **none**. The parser stays hand-written; the lock uses `std::fs`.

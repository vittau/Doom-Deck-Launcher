# 062 — Experimental Flatpak CI package

Contract: [spec 062](../specs/062-flatpak-ci-package.md). Decision: D-034 (authoring exception).
Research: [flatpak.md](../research/flatpak.md), [spec 059](../specs/059-flatpak-sandbox-ready.md).

**Outcome:** the packaging inputs, offline source build and artifact-only workflow are in place.
No application code changed. No GitHub Actions run happened, because the task forbids pushing and
the workflow only executes on GitHub; all claims below are from static checks and source generation
on the development host. No Flathub submission, release, tag or push was made.

## What changed

| Path | Purpose |
| --- | --- |
| `packaging/flatpak/io.github.vittau.Doom-Deck-Launcher.yml` | Flatpak manifest: runtime, permissions, pinned Rust, offline Cargo/Slint build, metadata and licence installs |
| `packaging/flatpak/io.github.vittau.Doom-Deck-Launcher.desktop` | Desktop entry; `Exec=ddl`, matching icon and app id |
| `packaging/flatpak/io.github.vittau.Doom-Deck-Launcher.metainfo.xml` | AppStream metadata, OARS, release and tagged screenshots |
| `packaging/flatpak/cargo-sources.json` | Generated, hash-pinned vendor list for the checked-out `Cargo.lock` (1,119 source entries) |
| `packaging/flatpak/catalog-serial` | Commit-time serial input; checked in as `0` sentinel, written by CI |
| `packaging/flatpak/flathub.json` | x86_64 arch declaration only; not a submission |
| `packaging/flatpak/.gitignore` | Keeps local build/bundle outputs out of git |
| `.github/workflows/flatpak.yml` | Artifact-only Flatpak job |
| `docs/flatpak-build.md` | Developer/installer instructions, provenance, refresh and limitations |

No Rust, `Cargo.toml`, `Cargo.lock`, `catalog/` or coordinator-owned document was edited.

## Packaging design

### App source and serial

The app module uses a local directory source:

```yaml
- type: dir
  path: ../..
  skip: [.git, .scratch, target, ...]
```

`flatpak-builder` resolves `../..` against the manifest directory, so the source is the repository
root of the checkout. There is no application commit pin; new packaging files cannot be omitted by
a stale self-reference.

`packaging/flatpak/catalog-serial` holds the source commit's UTC time. CI writes
`git log -1 --format=%ct` into it immediately before the build; the build command refuses serial
`0`. That keeps `DDL_CATALOG_SERIAL`/`EMBEDDED_SERIAL` equal to the checkout's commit time instead
of the build time (spec 019's rule), without making the manifest self-referential.

### Rust compiler and offline Cargo

| Pin | Value | Verification |
| --- | --- | --- |
| Runtime / SDK | `org.freedesktop.Platform` / `org.freedesktop.Sdk` `26.08` | Flathub API reports `freedesktop-sdk-26.08.2` (2026-09-27) for the branch; the SDK bundle points at the same `26.08` runtime |
| Rust SDK extension | `org.freedesktop.Sdk.Extension.rust-stable//26.08` | Confirmed available, but its latest release is **1.98.0** (2026-08-20). Current stable upstream is 1.99.1 (2026-10-15). The extension is below the repository's `rust-version = "1.99"` |
| Actual compiler | `rust-1.99.1-x86_64-unknown-linux-gnu.tar.xz` from `static.rust-lang.org/dist/2026-10-15/` | xz SHA-256 `231fcae47ee950621374725b5fc3b5441d2fbda2955ac45dd7dc816d8876cfe7` from the official channel manifest |
| Cargo generator | `flatpak-builder-tools` commit `74697c75b630d7330e77250fc13cb5ea688d9479` (2026-09-30) | Download SHA-256 `0a2db6be87d75910facef28ab46d4d6460802e8419ab850d0caa6a364d26b380` hard-coded in CI |
| Python generator deps | `aiohttp==3.14.5`, `tomlkit==0.15.2` | Pinned in CI; generator-only, not app dependencies |

The extension is **not used**. Using 1.98 would make Cargo reject the workspace's
`rust-version = "1.99"`; lowering the repository pin would violate R2 and the MSRV rule. The
manifest instead installs the exact official 1.99.1 toolchain as a hash-pinned archive module,
then `cleanup: /rust` removes it from the published app. This is the one deliberate deviation from
the "use the Rust extension" wording, forced by upstream state on 2026-10-20. The workflow records
the extension's available version in `rust-extension.txt`; switch the manifest back once the
extension ships a compiler that satisfies the MSRV.

The build uses `CARGO_HOME=/run/build/ddl/cargo`, `CARGO_NET_OFFLINE=true` and
`cargo build --release --locked --offline -p ddl`. Every crate comes from
`packaging/flatpak/cargo-sources.json`, which is generated from the checked-out `Cargo.lock` and
verified against `static.crates.io` SHA-256 hashes. `flatpak-builder` fetches those sources and the
compiler archive before the build sandbox starts; the build itself has no network.

The compiler install script and crate build scripts execute inside the build sandbox. That is
inherent to building Rust/Slint from source and is not the runtime "never execute downloaded
archives" rule: no IWAD, engine AppImage or mapset archive is extracted or run during packaging.

### Metadata and files

- Production icon reused from `crates/ddl/ui/assets/logo/icon.png` (256×256) plus
  `emblem.svg` as the scalable icon.
- Desktop entry and metainfo installed under the app id:
  `/app/share/applications/<id>.desktop`, `/app/share/metainfo/<id>.metainfo.xml`.
- `LICENSE` plus Inter, Barlow and Pirata One licences installed under
  `/app/share/licenses/<id>/`, matching the release tarball's licence set.
- Metainfo screenshots point at the `v0.0.9-focus` tag (not a branch) and the release entry is
  `0.0.9` / `2026-10-20`. It does not claim Flathub availability, verification or Deck hardware
  validation.
- `separate-locales: false` because the UI and metadata are English-only (D-008).
- OARS 1.1 marks cartoon/fantasy/realistic violence and bloodshed as moderate, matching the
  catalogue art the app displays.

## Sandbox permissions and rationale

```yaml
--require-version=1.16.0
--share=ipc
--socket=wayland
--socket=fallback-x11
--share=network
--socket=pulseaudio
--device=dri
--device=input
--filesystem=/run/udev:ro
--filesystem=home:ro
--filesystem=/run/media:ro
--filesystem=~/.var/app/com.valvesoftware.Steam:ro
--filesystem=~/.var/app/com.heroicgameslauncher.hgl:ro
```

| Grant | Why |
| --- | --- |
| `--require-version=1.16.0` | `--device=input` was added in Flatpak 1.15.6; 1.16 is the safe minimum. SteamOS 3.10 has 1.18.4. |
| `--share=ipc` | X11 fallback needs a shared IPC namespace. |
| `--socket=wayland`, `--socket=fallback-x11` | Slint/winit window on Wayland, X11 when Wayland is unavailable. |
| `--share=network` | In-app catalog bundle refresh and engine/mapset downloads (D-025, spec 006). |
| `--socket=pulseaudio` | DSDA-Doom audio through PipeWire's PulseAudio shim; the 26.08 runtime covers ALSA too. |
| `--device=dri` | OpenGL for Slint femtovg and the extracted engine; the runtime supplies libglvnd/GBM. |
| `--device=input` | evdev pads under `/dev/input`; narrower than `--device=all`, which would bind all of `/dev`. |
| `--filesystem=/run/udev:ro` | gilrs → libudev reads controller metadata (`ID_INPUT_*`, vendor/product). |
| `--filesystem=home:ro` | IWADs, local WADs and the controller-first picker need stable in-place paths; the app never writes outside its own XDG dirs (D-026). A portal path would break stored profiles. |
| `--filesystem=/run/media:ro` | Steam libraries on SD cards/removable media. |
| `--filesystem=~/.var/app/com.valvesoftware.Steam:ro` | IWAD discovery when Steam is the Flatpak; `home` access excludes `~/.var/app`. |
| `--filesystem=~/.var/app/com.heroicgameslauncher.hgl:ro` | Same for Heroic's Flatpak. |

Deliberately absent: `--device=all`, `home:rw`, `host-os`, `/mnt`/`/media`, and
`--talk-name=org.freedesktop.Flatpak`. There is no host shell execution and no Steam Input code.
The engine remains a runtime download, extracted before launch inside the sandbox (spec 059).

These are experimental CI permissions, not an approved Flathub set. A future submission would need
written exceptions for `home:ro` and the two `~/.var/app` grants; that is out of scope and not
eligible under Flathub's current AI policy anyway.

## Workflow

`.github/workflows/flatpak.yml`:

- Triggers: `workflow_dispatch`, `pull_request` and `push` to `main`, limited to packaging,
  Rust/catalog/app source and the workflow itself.
- `permissions: contents: read`; no secrets; no release or repository write.
- `concurrency: flatpak-${{ github.ref }}` with `cancel-in-progress: true`; `timeout-minutes: 90`.
- `runs-on: ubuntu-26.04` (Ubuntu 26.04's Flatpak is 1.16.6, satisfying `--require-version`).
- Installs `flatpak`, `flatpak-builder`, `desktop-file-utils` and `appstream` from Ubuntu, adds
  Flathub, installs `org.freedesktop.Platform//26.08` and `org.freedesktop.Sdk//26.08`.
- Records the available Rust extension version and SDK info without using them.
- Checks the manifest's Rust version equals `rust-toolchain.toml` and satisfies `rust-version`.
- Regenerates `cargo-sources.json` from the checked-out `Cargo.lock` with the pinned generator,
  verifies the generator's SHA-256 and fails on any diff from the committed file.
- Parses the manifest, checks every local source path and installed repository file, and asserts
  the desktop `Exec`/`Icon` and metainfo id/launchable.
- Runs `desktop-file-validate` and `appstreamcli validate --no-net --explain`.
- Writes the commit-time catalog serial, builds with `flatpak-builder --arch=x86_64 --disable-rofiles-fuse`, exports a
  local repo, installs it, and runs `ddl --version` and `ddl --help` through `flatpak run`
  (headless; both exit before a window).
- Creates `ddl-flatpak-x86_64.flatpak` with the Flathub runtime repo URL, writes its SHA-256,
  reinstalls the bundle from the file and runs `ddl --version` again.
- Uploads the bundle and checksum on success, and the full build log plus metadata/runtime info
  with `if: always()`.
- Never creates a GitHub Release, pushes a Flatpak repo or submits to Flathub.

Pinned actions: `actions/checkout@11d5960a326750d5838078e36cf38b85af677262` (v4) and
`actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02` (v4). The Flathub
runtime/SDK, generator and Rust archive are the other external inputs; see the pin table.

## Verification performed (development host, macOS arm64)

No Flatpak runtime exists on the macOS host, so a real build was not possible here. Static and
source-generation checks were run:

| Command | Result |
| --- | --- |
| `desktop-file-validate packaging/flatpak/<id>.desktop` | exit 0 |
| `appstreamcli validate --no-net --explain packaging/flatpak/<id>.metainfo.xml` | `✔ Validation was successful: pedantic: 1`; the only pedantic note is `cid-contains-uppercase-letter`, expected from the spec/D-034 app id |
| PyYAML `safe_load` of the workflow and manifest | parsed; 2 modules, 13 finish-args |
| Manifest input script used by CI (local copy of the same Python) | `manifest and metadata inputs OK` (all local sources and installed files exist; desktop/metainfo ids match) |
| `shasum -a 256` on the pinned generator | matches `0a2db6…b380` |
| Generator rerun against `Cargo.lock` and `cmp` | identical to `packaging/flatpak/cargo-sources.json`; 1,119 source entries |
| `actionlint .github/workflows/flatpak.yml` | passes with `-ignore 'label "ubuntu-26.04" is unknown'`; actionlint 1.7.12's hard-coded label list predates GitHub runner-images' `ubuntu-26.04` entry, so this is tool lag, not a workflow error |
| `cargo fmt --all --check` | exit 0 (no Rust change) |
| `git diff --check` | exit 0 |
| `git diff --name-only` outside owned paths | none |

**Not run:** `flatpak-builder`, `flatpak run`, `flatpak-bundle`, any Linux job or any Deck check.
The workflow has never executed on GitHub in this task because pushing is forbidden. The report
does not claim a CI pass; the first push/PR run is the remaining proof for A2's remote-execution
clause.

### Why the macOS workspace suite was not rebuilt

This change touches no Rust file, no `Cargo.toml`/`Cargo.lock` and no catalog recipe. A macOS
`cargo test --workspace` would compile and test exactly the same Linux-agnostic code as before,
and cannot exercise `flatpak-builder`, the 26.08 runtime, the sandbox permissions or the Flatpak
desktop/metainfo path. `cargo fmt --all --check` was run because it is cheap and confirms no Rust
file was accidentally touched. The meaningful validation is the Linux CI job described above.

## Dependencies

- **Cargo dependencies:** none added; `Cargo.lock` is unchanged.
- **App build dependencies:** the Freedesktop 26.08 SDK (compiler, pkg-config, fontconfig and
  systemd/libudev headers) and the official Rust 1.99.1 toolchain module.
- **CI-only tools:** Ubuntu `flatpak` 1.16.6, `flatpak-builder` 1.4.8, `desktop-file-utils`,
  `appstream`; Python `aiohttp==3.14.5`, `tomlkit==0.15.2`; the pinned
  `flatpak-cargo-generator.py` script. None are app dependencies.
- **Runtime:** `org.freedesktop.Platform//26.08`, installed on the target by the bundle's
  `--runtime-repo` pointer.

## Limitations and open questions

1. **No CI run yet.** Static checks pass, but the offline build, sandbox CLI checks and bundle
   creation must be confirmed by the first GitHub Actions run. This is the main open acceptance
   item (A2).
2. **Rust extension is stale.** The manifest pins 1.99.1 instead of the extension. Revisit when
   `org.freedesktop.Sdk.Extension.rust-stable//26.08` ships ≥1.99; then remove the toolchain module
   and restore `prepend-path: /usr/lib/sdk/rust-stable/bin` plus the standard extension env.
3. **Runtime branch is not immutable.** `26.08` is the maintained branch (currently 26.08.2);
   app/Cargo/Rust inputs are pinned, but the runtime can move within the branch. A Flathub build
   would use the branch too.
4. **Uppercase app id.** AppStream pedantically prefers a lowercase component id. The id is fixed
   by D-034/spec 062 and matches the repository; do not silently rename it.
5. **Experimental provenance.** The manifest and metadata were written by an AI agent at the
   owner's request (D-034). They are not eligible for Flathub under its current Generative AI
   policy and must not be submitted as-is.
6. **`flathub.json` is inert.** It only declares x86_64; no remote, store page or update channel
   exists.
7. **Build executes source scripts.** The Rust `install.sh` and crate build scripts run in the
   build sandbox from hash-pinned sources; this is required to build Rust/Slint from source, not a
   runtime archive execution.
8. **Permission set is a design proposal, not an approval.** `home:ro` and the two app-data grants
   would need exceptions and review.

### Needs Deck Validation (unchanged by this task)

- Extracted DSDA-Doom starts with working GL, audio and gamepad input under `flatpak run`.
- IWAD visibility through `home:ro`, `/run/media:ro` and the Steam/Heroic Flatpak grants.
- One-time import of a tarball-era `~/.config/ddl` / `~/.local/share/ddl`.
- On-screen keyboard through the OpenURI portal (`xdg-open steam://open/keyboard`) in Gaming Mode.
- Launching the Flatpak shortcut from Gaming Mode; absence of the Steam overlay; Steam Input
  layout selection; suspend/resume and controller focus.

## Acceptance mapping

- **A1 (R1/R3):** manifest parses, desktop and metainfo validate, all referenced repository files
  exist, destination paths use the app id; permission list inspected above. Static checks pass.
- **A2 (R2/R4):** source generation from `Cargo.lock` is meaningful and reproducible; compiler
  handling and all pins are documented. The Linux CI run is still pending because no push happened.
- **A3 (R5):** `docs/flatpak-build.md` describes the actual workflow, artifact, local refresh,
  source model, provenance and limitations; existing tarball instructions are untouched.
- **A4:** this report contains commands/results, dependencies, permissions, limitations and open
  Deck checks; no Rust behaviour changed.

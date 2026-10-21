# Experimental Flatpak build

Status: **experimental, AI-assisted, CI artifact only.** The normal Steam Deck download is still
the release tarball described in the [README](../README.md#install-on-steam-deck).

There are now two Flatpak paths:

| Path | Workflow | Output |
| --- | --- | --- |
| Build validation and `.flatpak` artifact | `.github/workflows/flatpak.yml` | workflow artifact, 14 days |
| Signed install repository | `.github/workflows/flatpak-remote.yml` | `gh-pages`, served at <https://www.vitormach.dev/Doom-Deck-Launcher/> |

For the one-command install, updating and the signing key, read
[`docs/flatpak-remote.md`](flatpak-remote.md). Everything below describes the artifact-only build,
which stays the place to validate manifest changes before publishing.

The files live in [`packaging/flatpak/`](../packaging/flatpak) and the workflows in
[`.github/workflows/`](../.github/workflows). Spec 062 defines the artifact scope, spec 063 the
repository; D-034 records the owner's authoring exception.

## What this builds

| Item | Value |
| --- | --- |
| App id | `io.github.vittau.Doom-Deck-Launcher` |
| Runtime / SDK | `org.freedesktop.Platform` / `org.freedesktop.Sdk` **26.08** |
| Architecture | `x86_64` only |
| App sources | the checked-out tree (`type: dir`), no self-referential commit pin |
| Rust | **1.99.1**, official `static.rust-lang.org` archive pinned by SHA-256 |
| Cargo sources | `packaging/flatpak/cargo-sources.json`, generated from the checked-out `Cargo.lock` |
| Generator | `flatpak-builder-tools` commit `74697c7`, SHA-256 checked in CI |
| Artifact | `ddl-flatpak-x86_64.flatpak` + `.sha256` |
| Result | manual workflow run artifact, kept 14 days |

The package is built from source: Rust, Cargo dependencies and Slint are compiled inside the
Flatpak build sandbox with `--locked --offline`. The app source is the checked-out tree, so a CI
run never depends on a commit pin that could omit new packaging files.

### Why the Rust extension is not used

`org.freedesktop.Sdk.Extension.rust-stable//26.08` was available on 2026-10-20, but its newest
release was **Rust 1.98.0** (`flathub.org` appstream, release timestamp 2026-08-20). This repository
pins **1.99.1** in `rust-toolchain.toml` and declares `rust-version = "1.99"`, so Cargo refuses to
build with 1.98. The manifest therefore installs the exact official 1.99.1 toolchain as a pinned
archive module. Do not lower the repository toolchain to match an older extension; switch the
manifest back to the extension once it ships a compiler that satisfies the MSRV.

The compiler archive and crate sources are downloaded by `flatpak-builder` **before** the build
sandbox starts, so the build itself has no network access. Installing the compiler and compiling
crates executes build scripts from pinned, hash-verified sources; the "never execute downloaded
archives" rule applies to user content and the engine at runtime, not to a source build.

## Run the workflow

1. Open **Actions → Flatpak → Run workflow** on the branch or `main`.
2. Wait for the job. It installs Flatpak on `ubuntu-26.04`, adds Flathub, installs the 26.08
   runtime/SDK, regenerates and verifies `cargo-sources.json`, validates the desktop file and
   metainfo, builds the Flatpak offline, runs `ddl --version` and `ddl --help` inside the sandbox,
   creates the bundle and reinstalls that bundle for one more smoke test.
3. Download the **ddl-flatpak-x86_64** artifact and extract the zip.

The workflow also runs on pull requests and pushes to `main` that touch the packaging, the Rust
lockfile/toolchain, the app sources or the catalog. It has read-only repository permissions, a
90-minute timeout, no secrets and no publication step.

### Install the artifact

On a Linux host with Flatpak 1.16 or newer (SteamOS 3.10 ships 1.18):

```sh
sha256sum -c ddl-flatpak-x86_64.flatpak.sha256
flatpak install --user ./ddl-flatpak-x86_64.flatpak
flatpak run io.github.vittau.Doom-Deck-Launcher
```

The bundle records `https://flathub.org/repo/flathub.flatpakrepo` as its runtime source, so the
26.08 runtime can be installed on a machine that does not have it yet. Remove it with:

```sh
flatpak uninstall --user io.github.vittau.Doom-Deck-Launcher
```

To add it to Steam, use the same manual flow as the tarball but point the shortcut at
`/usr/bin/flatpak` with launch options `run io.github.vittau.Doom-Deck-Launcher`, or pick the
exported desktop entry if Steam lists it. Set **Compatibility tool: none**.

## Build locally (Linux)

macOS cannot run `flatpak-builder`; use a Linux machine or the CI workflow.

```sh
# 1. Tooling and runtimes
sudo apt install flatpak flatpak-builder desktop-file-utils appstream
flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --user -y flathub org.freedesktop.Platform//26.08 org.freedesktop.Sdk//26.08

# 2. Catalog serial: the source commit's UTC time, not the build time.
printf '%s\n' "$(git log -1 --format=%ct)" > packaging/flatpak/catalog-serial

# 3. Cargo sources (needs Python 3.9+; see "Refresh Cargo sources" below)
python3 flatpak-cargo-generator.py Cargo.lock -o packaging/flatpak/cargo-sources.json

# 4. Validate metadata
desktop-file-validate packaging/flatpak/io.github.vittau.Doom-Deck-Launcher.desktop
appstreamcli validate --no-net --explain packaging/flatpak/io.github.vittau.Doom-Deck-Launcher.metainfo.xml

# 5. Build, export and install
flatpak-builder --user --arch=x86_64 --force-clean --disable-rofiles-fuse \
  --repo=/tmp/ddl-flatpak-repo /tmp/ddl-flatpak-build \
  packaging/flatpak/io.github.vittau.Doom-Deck-Launcher.yml
flatpak build-export --arch=x86_64 /tmp/ddl-flatpak-repo /tmp/ddl-flatpak-build master
flatpak --user remote-add --if-not-exists --no-gpg-verify ddl-local /tmp/ddl-flatpak-repo
flatpak --user install -y ddl-local io.github.vittau.Doom-Deck-Launcher//master
flatpak run --command=ddl io.github.vittau.Doom-Deck-Launcher//master --version
```

Restore the sentinel before committing:

```sh
git checkout -- packaging/flatpak/catalog-serial
```

The checked-in `0` is intentional: a build that forgets to write the serial fails instead of
baking serial 0 into the binary and losing every embedded-vs-cached comparison.

## Refresh Cargo sources

`cargo-sources.json` is a hash-pinned vendor list for every crate in `Cargo.lock`. Regenerate it
whenever `Cargo.lock` changes:

```sh
curl --proto '=https' --tlsv1.2 -fsSL \
  https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/74697c75b630d7330e77250fc13cb5ea688d9479/cargo/flatpak-cargo-generator.py \
  -o /tmp/flatpak-cargo-generator.py
python3 -m venv /tmp/flatpak-cargo-venv
/tmp/flatpak-cargo-venv/bin/pip install 'aiohttp==3.14.5' 'tomlkit==0.15.2'
/tmp/flatpak-cargo-venv/bin/python /tmp/flatpak-cargo-generator.py Cargo.lock \
  -o packaging/flatpak/cargo-sources.json
```

CI regenerates the file from the checked-out `Cargo.lock` and fails if the committed file differs.
That is the check that keeps the lockfile authoritative; do not hand-edit the JSON.

## Sources: local checkout vs. published release

- **Flatpak app source:** the manifest uses `type: dir` with `path: ../..`. `flatpak-builder`
  copies the current checkout (minus `.git`, `target` and local build dirs), so the package always
  contains the exact tree that was checked out, including new packaging files.
- **Cargo sources:** each crate is downloaded from `static.crates.io` and verified against the
  SHA-256 in `cargo-sources.json`, generated from the checked-out lockfile.
- **Rust:** one official compiler archive, pinned by URL and SHA-256.
- **Release tarball:** `.github/workflows/release.yml` is unchanged. It still builds
  `ddl-linux-x86_64.tar.gz` in the Steam Linux Runtime SDK and publishes it. `flatpak.yml` never
  publishes anything; only `flatpak-remote.yml` writes the `gh-pages` branch, and it never creates a
  Release.

`flathub.json` here only declares the x86_64 architecture. It is not a submission and the package
is not eligible for Flathub under its current Generative AI policy (D-034).

## Permissions

Each `finish-args` entry, and why it exists:

| Permission | Reason |
| --- | --- |
| `--require-version=1.16.0` | `--device=input` needs Flatpak 1.16+; SteamOS 3.10 has 1.18. |
| `--share=ipc` | X11 fallback needs a shared IPC namespace. |
| `--socket=wayland`, `--socket=fallback-x11` | Slint/winit window; Wayland first, X11 fallback. |
| `--share=network` | Catalog bundle refresh and engine/mapset downloads. |
| `--socket=pulseaudio` | DSDA-Doom audio through PipeWire's PulseAudio compatibility layer. |
| `--device=dri` | OpenGL for Slint's femtovg renderer and the engine. |
| `--device=input` | Gamepad evdev nodes (Steam Input's virtual pad appears there). Narrower than `--device=all`. |
| `--filesystem=/run/udev:ro` | gilrs/libudev reads controller IDs and capabilities. |
| `--filesystem=home:ro` | IWAD/mapset discovery and the controller-first file picker, in place and read-only. |
| `--filesystem=/run/media:ro` | Steam libraries on SD cards and removable drives. |
| `--filesystem=~/.var/app/com.valvesoftware.Steam:ro` | IWAD discovery when Steam is the Flatpak; `home` excludes `~/.var/app`. |
| `--filesystem=~/.var/app/com.heroicgameslauncher.hgl:ro` | Same for Heroic's Flatpak. |

Deliberately absent: `--device=all`, `home:rw`, `host-os`, `--talk-name=org.freedesktop.Flatpak`
(no host execution), and any Steam Input integration. The engine is downloaded at runtime and
extracted first inside the sandbox (spec 059); it is never mounted with FUSE.

These grants are what the code needs today, not an approved Flathub permission set. A future
submission would need written exceptions for `home:ro` and the two `~/.var/app` grants.

## Limitations and open validation

This package has **not** been run on a Steam Deck. The CI smoke test only proves that the binary
starts and prints help inside the sandbox; it does not open a window.

Still open (Needs Deck Validation):

- Extracted DSDA-Doom starts with working GL, audio and controller input inside `flatpak run`.
- IWADs are visible through `home:ro`, `/run/media:ro` and the two Flatpak app-data grants.
- The one-time import of a tarball-era `~/.config/ddl` / `~/.local/share/ddl` works.
- The on-screen keyboard opens through `xdg-open steam://open/keyboard` in Gaming Mode.
- Steam can launch the Flatpak shortcut from Gaming Mode; the Steam overlay will not attach.
- Suspend/resume and controller focus behave like the tarball build.

Packaging limitations:

- No CI run has been recorded yet: the workflow only runs after it is pushed to GitHub, and this
  task explicitly does not push. See the report for the static checks that did run.
- The Flatpak build downloads roughly 1 GB of SDK/runtime and compiles Slint from source, so the
  first run is slow (the 90-minute timeout covers it).
- The package uses the 26.08 runtime branch, which moves within the branch. The Rust compiler,
  crates and app tree are pinned; the runtime is a supported branch, not a commit.
- `flathub.json` is inert here; there is no store page and no Flathub update channel. Updates come
  from the self-hosted repository described in [`docs/flatpak-remote.md`](flatpak-remote.md).

## Troubleshooting

- **`catalog-serial` is 0:** the local build skipped step 2. Write the commit time and rebuild.
- **`cargo-sources.json is stale`:** rerun the refresh command above and commit the result.
- **`rustc` is not 1.99.1:** the build is not using `/app/rust/bin`; check the manifest's
  `append-path`.
- **The app refuses to start on an older distro:** it requires Flatpak 1.16+, as declared by
  `--require-version`.

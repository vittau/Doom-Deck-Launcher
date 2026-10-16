# Building DDL for Steam Deck: runtime, container and renderer

**Spec:** 001 (R8) · **Status:** evidence gathered 2026-10-16

This document records *why* the release build uses a particular Steam Linux Runtime SDK
image, how Rust gets into that image, and which Slint backend/renderer DDL compiles
with. Every claim is labelled:

- **Confirmed** — verified from upstream source, upstream documentation, or by
  inspecting the published image/manifest directly (links below).
- **Proposed** — the chosen default; reasonable, but not yet proven on hardware.
- **Needs Deck Validation** — must be verified on a physical Steam Deck
  (tracked in `docs/deck-testing.md` by the coordinator).

## 1. Decision summary

| Question | Choice | Label |
| --- | --- | --- |
| Runtime to target | Steam Linux Runtime 3.0 "sniper" | Proposed |
| Build image | `registry.gitlab.steamos.cloud/steamrt/sniper/sdk@sha256:220d1dc739254155a91c010f959a4f8e7c183e69d18300d6be53a20aee12b02d` (tag `3.0.20261007.263965`) | Confirmed (digest verified) |
| Rust installation | `rustup`, installed inside the container, channel read from `rust-toolchain.toml` (`1.99.1`) | Confirmed (image ships no Rust) |
| Slint backend | `winit` (`backend-winit`: X11 + Wayland) | Proposed |
| Slint renderer | FemtoVG (`renderer-femtovg`, OpenGL ES via EGL/GLX) | Proposed |
| Slint default features | Disabled; explicit minimal feature set | Proposed |

## 2. Why sniper (Steam Runtime 3) rather than Steam Runtime 4

Valve's own guidance, and the facts behind it:

- **Confirmed** — Valve documents two current container runtimes for native Linux
  software: `Steam Linux Runtime 3.0 (sniper)`, a "Steam Runtime 3 'sniper'"
  container, and `Steam Linux Runtime 4.0`, a "Steam Runtime 4 'steamrt4'"
  container which Valve describes as "the recommended environment for developers of
  new native Linux games"
  ([ValveSoftware/steam-runtime README](https://github.com/ValveSoftware/steam-runtime/blob/master/README.md)).
- **Confirmed** — the two SDK images differ in base distribution and hence in the
  oldest supported glibc:

  | Image | Base | glibc | Evidence |
  | --- | --- | --- | --- |
  | `steamrt/sniper/sdk:3.0.20261007.263965` | Debian 11 "bullseye" | 2.31 (`glibc 2.31-9`) | [`…Sdk-amd64,i386-sniper.os-release.txt`](https://repo.steampowered.com/steamrt3/images/3.0.20261007.263965/com.valvesoftware.SteamRuntime.Sdk-amd64,i386-sniper.os-release.txt) (`PRETTY_NAME="Steam Runtime 3 (sniper)"`) and its [`…manifest.dpkg.built-using`](https://repo.steampowered.com/steamrt3/images/3.0.20261007.263965/com.valvesoftware.SteamRuntime.Sdk-amd64,i386-sniper.manifest.dpkg.built-using) |
  | `steamrt/steamrt4/sdk:4.0.20261007.263973` | Debian 13.7 | ~2.41 | [`…steamrt4.os-release.txt`](https://repo.steampowered.com/steamrt4/images/4.0.20261007.263973/com.valvesoftware.SteamRuntime.Sdk-amd64,i386-steamrt4.os-release.txt) (`DEBIAN_VERSION_FULL=13.7`) |

DDL is **not** shipped through Steam: it is delivered as a tarball that the user adds
as a **non-Steam** shortcut, and Steam does not automatically put a container runtime
around a non-Steam game. The binary therefore runs directly against whatever glibc
and shared libraries SteamOS provides. Spec 001 (R8) says to default to choices that
**minimise runtime library requirements on SteamOS**, and a binary linked against
glibc 2.31 (sniper) has strictly lower requirements than one linked against glibc
~2.41 (steamrt4).

**Proposed consequence:** build with the sniper SDK. If the Deck provider later
reports that SteamOS can no longer satisfy a sniper build — or that DDL has to run
*inside* the SLR 4.0 container — switch to `steamrt/steamrt4/sdk`. The switch is a
one-line change in `.github/workflows/ci.yml` and `release.yml`.

**Needs Deck Validation:** the exact behaviour of a non-Steam shortcut in Gaming
Mode (does Steam wrap it in a runtime container, or run it against SteamOS
libraries?) and whether the released tarball needs any library that a stock Deck
lacks. This is acceptance criterion A7, owned by the coordinator.

## 3. The pinned image

```
registry.gitlab.steamos.cloud/steamrt/sniper/sdk@sha256:220d1dc739254155a91c010f959a4f8e7c183e69d18300d6be53a20aee12b02d
```

- **Confirmed** — the tag `3.0.20261007.263965` is the newest stable sniper SDK at
  the time of writing (2026-10-16). Tag list obtained from the GitLab container
  registry API:
  `GET https://registry.gitlab.steamos.cloud/v2/steamrt/sniper/sdk/tags/list`
  (anonymous pull token from `https://gitlab.steamos.cloud/jwt/auth`).
- **Confirmed** — the digest above is the digest of the manifest served for that tag:
  `HEAD .../v2/steamrt/sniper/sdk/manifests/3.0.20261007.263965` returns
  `docker-content-digest: sha256:220d1dc7…102d`, and hashing the manifest body
  locally reproduced it. The manifest's config reports
  `"architecture": "amd64"`, build id `3.0.20261007.263965`, OS release
  `Steam Runtime 3 (sniper)`.
- **Confirmed** — the image tag is pinned **by digest, not by tag**, so a future
  re-tag of `latest` (or even of `3.0.20261007.263965`) cannot silently change what
  DDL releases are built from.

`ci.yml` and `release.yml` intentionally contain the same digest; both files carry a
comment saying they must be kept in sync.

## 4. Rust inside the container

- **Confirmed** — the sniper SDK image ships **no** Rust toolchain. Its published
  package manifest
  ([`…Sdk-amd64,i386-sniper.manifest.deb822.gz`](https://repo.steampowered.com/steamrt3/images/3.0.20261007.263965/com.valvesoftware.SteamRuntime.Sdk-amd64,i386-sniper.manifest.deb822.gz))
  contains no `rustc`, `cargo` or `rust-*` package. The image's default command is
  `/bin/bash`.
- **Confirmed** — the image *does* provide everything the Rust build needs beyond the
  toolchain itself: `curl`, `wget`, `git`, `build-essential`/`gcc-10`/`gcc-14`,
  `pkg-config`, `tar`, and the `-dev` packages for the libraries Slint probes at build
  time (`libfontconfig-dev`, `libfontconfig1-dev`, `libfreetype-dev`,
  `libxkbcommon-dev`, `libegl1-mesa-dev`, `libgl1-mesa-dev`, `libwayland-dev`,
  `libx11-dev`, `libxcb*-dev`). No `apt-get install` is needed in the container jobs.

Method actually used by both workflows:

1. Download and run the official rustup installer with `--default-toolchain none`,
   so no toolchain is guessed;
2. read `channel` out of `rust-toolchain.toml`
   ([`sed -n 's/^channel *= *"\([^"]*\)"/\1/p' rust-toolchain.toml`]) — currently
   `1.99.1`;
3. `rustup toolchain install "$channel" --profile minimal --component rustfmt --component clippy`
   and `rustup default "$channel"`.

Pinning the channel in the repository and reading it back in CI means the container
build, the GitHub-hosted build and local development all use byte-identical
toolchains.

## 5. Slint backend and renderer

DDL depends on Slint as:

```toml
slint = { version = "=1.18.1", default-features = false, features = [
    "compat-1-18", "std", "backend-winit", "renderer-femtovg",
] }
```

- **Confirmed** — `winit` is the default Slint backend on desktop and `renderer-femtovg`
  is the default renderer; the mandatory feature is `compat-1-18`
  ([Slint 1.18.1 "Feature flags and backend selection" docs](https://docs.rs/slint/1.18.1/slint/docs/cargo_features/index.html)).
- **Proposed** — `default-features = false` was chosen deliberately. The defaults also
  enable `system-tray` (pulls in `ksni` on Linux), `accessibility` (pulls in AccessKit)
  and `renderer-software`; a placeholder menu needs none of them, and every dropped
  dependency is one less library that can be missing on SteamOS.
  `backend-winit` (rather than `backend-winit-x11` or `backend-winit-wayland`) keeps
  both X11 and Wayland support so the same binary works in Gaming Mode (gamescope/X11)
  and under a Wayland session.
- **Confirmed** — FemtoVG renders through OpenGL/OpenGL ES. `glutin` loads EGL and GLX
  with `libloading` and `glow` loads the GL function pointers at runtime, so nothing is
  linked at build time; the libraries must exist when DDL starts.
- **Needs Deck Validation** — that FemtoVG can create a GL/EGL context under gamescope
  in Gaming Mode. If it cannot, add the `renderer-software` feature (plus
  `SLINT_BACKEND=winit-software`) as the fallback; that is a two-line change in
  `crates/ddl/Cargo.toml` and the documented startup path. Not added now, because it
  would ship an unused renderer in the placeholder.

## 6. Shared library requirements of the released binary

Two categories. The **authoritative list is printed by CI** — `ci.yml` runs
`ldd target/release/ddl` in the same container as the release build — so treat the
table below as the expectation derived from the dependency graph and from reading
each crate's build script, not as a substitute for that output.

**Linked at build time** (appears in `ldd`):

| Library | Why | Label |
| --- | --- | --- |
| `libc.so.6`, `libm.so.6`, `libgcc_s.so.1` | Rust `std` and unwinding | Proposed |
| `libfontconfig.so.1` (+ its own dependencies: `libfreetype.so.6`, `libexpat.so.1`, `libpng16.so.16`, `libbrotli*`, `libz.so.1`, …) | Slint's text stack: `parley` → `fontique` → `yeslogic-fontconfig-sys`, whose build script runs `pkg-config find_library("fontconfig").unwrap()` unless `RUST_FONTCONFIG_DLOPEN` is set | Confirmed (from the crate's `build.rs`) |

**Loaded at runtime with `dlopen`** (does **not** appear in `ldd`, but DDL cannot run
without it):

| Library | Why | Label |
| --- | --- | --- |
| `libEGL.so.1`, `libGLESv2.so.2` (or `libGL.so.1` for GLX) | FemtoVG/glutin GL context | Proposed |
| `libX11.so.6` (and `libXext`, `libXcursor`, `libXi`, `libXrandr`, `libXfixes`, `libXrender`, `libXxf86vm`, `libxcb*`) | `winit` X11 backend (`x11-dl` dlopens them) | Proposed |
| `libxkbcommon.so.0` | keyboard handling (`xkbcommon-dl`) | Proposed |
| `libwayland-client.so.0` | `winit` Wayland backend (`wayland-sys` is built with its `dlopen` feature, so nothing is linked) | Confirmed (feature resolution) |

`libfontconfig.so.1` is the only non-glibc library the binary is *hard-linked* to.
SteamOS ships it (it is part of the desktop stack), and it is present in the sniper
runtime; the Deck test (A7) is what proves it on hardware.

## 7. Sources

- Valve, *Steam Runtime* README — runtime list, when to use sniper vs steamrt4:
  <https://github.com/ValveSoftware/steam-runtime/blob/master/README.md> (Confirmed)
- Steam Runtime 3 "sniper" SDK documentation (image usage, digest pinning):
  <https://gitlab.steamos.cloud/steamrt/sniper/sdk> (Confirmed; the GitLab web UI is
  behind a bot check, so the SDK's published artifacts on `repo.steampowered.com`
  were used for the concrete version/glibc facts)
- Published sniper SDK artifacts for build `3.0.20261007.263965` (os-release,
  dpkg manifest, `built-using`):
  <https://repo.steampowered.com/steamrt3/images/3.0.20261007.263965/>
- Published Steam Runtime 4 SDK artifacts for build `4.0.20261007.263973`:
  <https://repo.steampowered.com/steamrt4/images/4.0.20261007.263973/>
- Container registry API used for the tag list and the manifest digest:
  `https://registry.gitlab.steamos.cloud/v2/steamrt/sniper/sdk/…` (Confirmed)
- Slint 1.18.1 cargo features, backends and renderers:
  <https://docs.rs/slint/1.18.1/slint/docs/cargo_features/index.html> and
  <https://releases.slint.dev/1.18.1/docs/slint/guide/backends-and-renderers/backends_and_renderers/>
  (Confirmed)
- Steam Runtime issue tracker, "Run native Linux games inside the sniper runtime
  inside CI" (SDK vs Platform images, X11/Wayland inside containers):
  <https://github.com/ValveSoftware/steam-runtime/issues/525> (Confirmed)

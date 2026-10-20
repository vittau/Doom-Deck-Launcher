# Shipping DDL as a Flatpak installable from Discover

**Spec:** 058 · **Status:** research complete 2026-10-20 · **Scope:** can DDL be distributed as a
Flatpak on SteamOS/Discover, how the DSDA-Doom AppImage runs inside the sandbox, what permissions
the content paths need, and what Flathub would require. **No code was written and no manifest was
added to the repository.**

Every factual claim carries one of these labels:

- **Confirmed** — read in pinned upstream source, upstream documentation, a package manifest, or
  DDL's own code.
- **Proposed** — a DDL design position suggested here; not yet proven.
- **Needs Deck Validation** — depends on a physical Steam Deck, the current Steam client, or
  Gaming Mode behaviour that cannot be reproduced on the development host.

## 1. Bottom line

| Question | Finding | Label |
| --- | --- | --- |
| Can a Flatpak run the downloaded AppImage? | **Not by mounting it.** Flatpak's sandbox disables nested user namespaces and does not map root, so the type-2 runtime cannot get a FUSE mount. Extraction and running `AppRun` works. | Confirmed |
| Which runtime? | `org.freedesktop.Platform` 26.08 (latest, 26.08.2). It covers every non-bundled library the engine needs, including the `libjack.so.0` gap, through PipeWire's JACK shim. | Confirmed (composition) / Needs Deck Validation (run) |
| Recommended architecture | (a) download the official AppImage at runtime and **always extract first** inside the sandbox. Keeps D-001/D-007/D-011/D-013 untouched. | Proposed |
| Existing DSDA-Doom Flatpak? | None on Flathub (search API, 2026-10-20). | Confirmed |
| What permissions? | `home:ro`, `/run/media:ro`, `/run/udev:ro`, `--device=input`, `--device=dri`, network, Wayland/X11, PulseAudio, plus two `~/.var/app` grants. `home:ro` and the `~/.var/app` grants need a Flathub linter exception. | Proposed |
| Does Discover add Flatpak apps to Steam? | **No.** Discover 6.7.3 (the SteamOS 3.10.0 build) has no such action; the only Steam-related code is the SteamOS update backend. | Confirmed |
| Can the Steam overlay attach? | No: Flatpak unsets `LD_PRELOAD`, which is how Steam injects `gameoverlayrenderer.so`. Steam Input still works at the kernel level. | Confirmed (code) / Needs Deck Validation (effect) |
| Does the OSK call still work? | `steam` is not on `$PATH` in the sandbox. `xdg-open steam://open/keyboard` should reach the host through the OpenURI portal; unproven in Gaming Mode. | Proposed / Needs Deck Validation |
| Flathub's biggest blocker | Not technical: Flathub's **Generative AI policy** forbids AI-generated or AI-assisted **manifest** content and AI-agent submission PRs, and requires disclosure of AI-assisted application code. | Confirmed (policy text) |
| Friction win | Removes the browser/download/extract steps and makes updates automatic. It does **not** remove the Desktop Mode trip, the manual Steam shortcut, or manual artwork. | Proposed |

## 2. Sources and pins

| Source | Pin / date | Used for |
| --- | --- | --- |
| [Flatpak `common/flatpak-run.c`](https://github.com/flatpak/flatpak/blob/1.18.4/common/flatpak-run.c) | Tag `1.18.4` (the version in SteamOS 3.10.0) | Device access, environment scrubbing, namespace policy, seccomp |
| [Flatpak `common/flatpak-context.c`](https://github.com/flatpak/flatpak/blob/1.18.4/common/flatpak-context.c) | Tag `1.18.4` | `XDG_*` environment inside the sandbox, `~/.var/app` handling |
| [Flatpak sandbox permissions](https://docs.flatpak.org/en/latest/sandbox-permissions.html) | Read 2026-10-20 | `--filesystem` semantics, reserved paths, `--device` |
| [flatpak-metadata(5)](https://docs.flatpak.org/en/latest/flatpak-command-reference.html) | Read 2026-10-20 | `/.flatpak-info`, metadata keys |
| [Flatpak `NEWS`](https://github.com/flatpak/flatpak/blob/1.18.4/NEWS) | Tag `1.18.4` | When `--device=input` was added (1.15.6) |
| [AppImage type-2 runtime `runtime.c`](https://github.com/AppImage/type2-runtime/blob/fedfad51acc2860b2916a1eef32ba4a500ff31ef/src/runtime/runtime.c) | Commit `fedfad5` | `find_fusermount`, `APPIMAGE_EXTRACT_AND_RUN`, extraction target |
| [AppImage docs: FUSE troubleshooting](https://github.com/AppImage/docs.appimage.org/blob/b29179c8f91387098988525c1f901889facab944/source/user-guide/troubleshooting/fuse.rst) and [run/extract](https://github.com/AppImage/docs.appimage.org/blob/b29179c8f91387098988525c1f901889facab944/source/user-guide/run-appimages.rst) | Commit `b29179c` | AppImage's own guidance |
| [freedesktop-sdk `elements/platform.bst`](https://gitlab.com/freedesktop-sdk/freedesktop-sdk/-/blob/master/elements/platform.bst) | `master`, read 2026-10-20 | Runtime library composition |
| [freedesktop-sdk `components/pipewire.bst`](https://gitlab.com/freedesktop-sdk/freedesktop-sdk/-/blob/master/elements/components/pipewire.bst) | `master`, read 2026-10-20 | `libjack.so.0` in the runtime |
| [freedesktop-sdk tags](https://gitlab.com/freedesktop-sdk/freedesktop-sdk/-/tags) | `freedesktop-sdk-26.08.2` (2026-09-28) | Latest runtime release |
| [SteamOS 3.10.0 package manifest](https://steamdeck-images.steamos.cloud/steamdeck/20261015.1000/steamdeck-20261015.1000-3.10.0.manifest.pacman) | Image `20261015.1000` | Flatpak 1.18.4, Discover 6.7.3, glibc 2.43, PipeWire, portals |
| [KDE Discover v6.7.3 source](https://github.com/KDE/discover/tree/v6.7.3) | Tag `v6.7.3` | No "Add to Steam"; `.flatpakref`/`.flatpakrepo` handling |
| [Flathub requirements](https://docs.flathub.org/docs/for-app-authors/requirements) | Read 2026-10-20 | Inclusion, AI, ID, license, permissions, build rules |
| [Flathub MetaInfo guidelines](https://docs.flathub.org/docs/for-app-authors/metainfo-guidelines) | Read 2026-10-20 | Metainfo, screenshots, OARS, releases |
| [Flathub submission](https://docs.flathub.org/docs/for-app-authors/submission) | Read 2026-10-20 | PR flow, review, publication |
| [Flathub maintenance](https://docs.flathub.org/docs/for-app-authors/maintenance) | Read 2026-10-20 | `x-checker-data`, update cadence |
| [flatpak-builder-lint `finish_args.py`](https://github.com/flathub-infra/flatpak-builder-lint/blob/e38c396ec0f593fe11a6b16fe83a33608db33f1e/flatpak_builder_lint/checks/finish_args.py) | Commit `e38c396` | Which permissions are errors vs. review findings |
| [flatpak-cargo-generator](https://github.com/flatpak/flatpak-builder-tools/blob/74697c75b630d7330e77250fc13cb5ea688d9479/cargo/README.md) | Commit `74697c7` | Offline Rust vendoring |
| [org.freedesktop.Sdk.Extension.rust-stable](https://flathub.org/apps/org.freedesktop.Sdk.Extension.rust-stable) | Flathub API, 2026-10-20 | Rust in the build sandbox |
| [GamingOnLinux: extra software on SteamOS](https://www.gamingonlinux.com/guides/view/how-to-install-extra-software-apps-and-games-on-steamos-and-steam-deck) | Published 2025-01-13, updated 2025-01-14 | Flathub preconfigured in Discover (secondary) |
| Flathub precedent manifests | Pins in §4 | What reviewers already accept |
| DDL source | Commit `1edd4ec` (task branch base) | Paths, dependencies, launch contract |

Secondary sources are labelled as such: GamingOnLinux and the Steam community thread are not
upstream documentation.

## 3. Q1 — The engine inside the sandbox

### 3.1 FUSE cannot work, and that is decided by the sandbox, not by missing packages

**Confirmed, three independent reasons:**

1. **No root, so no setuid helper.** The type-2 runtime only mounts through a `fusermount`
   that is setuid root: `find_fusermount()` rejects anything where `sb.st_uid != 0 ||
   (sb.st_mode & S_ISUID) == 0`
   ([runtime.c L417–L505](https://github.com/AppImage/type2-runtime/blob/fedfad51acc2860b2916a1eef32ba4a500ff31ef/src/runtime/runtime.c#L417-L505)).
   Flatpak runs with `--unshare-user` and maps only the calling user (plus `nfsnobody` for
   unmapped ids), so a root-owned setuid file appears as an unmapped uid and cannot gain
   privileges ([run.c L2415](https://github.com/flatpak/flatpak/blob/1.18.4/common/flatpak-run.c#L2415),
   passwd generation immediately below it).
2. **No nested user namespace.** Flatpak passes `--unshare-user --disable-userns`, and its
   seccomp blocklist returns `EPERM` for `clone(CLONE_NEWUSER)`
   ([run.c L2415](https://github.com/flatpak/flatpak/blob/1.18.4/common/flatpak-run.c#L2415),
   [run.c L2068](https://github.com/flatpak/flatpak/blob/1.18.4/common/flatpak-run.c#L2068)).
   Unprivileged FUSE mounts need exactly that namespace.
3. **The runtime does not ship a fusermount.** `platform.bst` (the `org.freedesktop.Platform`
   composition) has no `fuse3` element, and the `fuse3` element that exists is built with
   `chmod 4755` for the *SDK*, not the platform
   ([platform.bst](https://gitlab.com/freedesktop-sdk/freedesktop-sdk/-/blob/master/elements/platform.bst),
   [fuse3.bst](https://gitlab.com/freedesktop-sdk/freedesktop-sdk/-/blob/master/elements/components/fuse3.bst)).

Even `--device=all`, which does a full `--dev-bind /dev /dev`
([run.c L411–L416](https://github.com/flatpak/flatpak/blob/1.18.4/common/flatpak-run.c#L411-L416)),
does not change reasons 1 and 2. **Conclusion:** inside a Flatpak, the AppImage must never be
mounted; the mount path is not a fallback that sometimes works.

**Confirmed precedent.** Lutris' Flathub manifest sets `--env=APPIMAGE_EXTRACT_AND_RUN=1` with
the comment "Needed for appimages"
([net.lutris.Lutris.yml @ `8de7159`](https://github.com/flathub/net.lutris.Lutris/blob/8de715981eabf6907d1e971efa3a2dc91a9dcf3c/net.lutris.Lutris.yml)).
Lutris is the closest analogue to DDL on Flathub.

### 3.2 Therefore: extraction first, always

**Confirmed.** The type-2 runtime has a non-FUSE path: `APPIMAGE_EXTRACT_AND_RUN=1` or the
`appimage-extract-and-run` argument
([runtime.c L1580](https://github.com/AppImage/type2-runtime/blob/fedfad51acc2860b2916a1eef32ba4a500ff31ef/src/runtime/runtime.c#L1580));
`--appimage-extract` writes `squashfs-root/` into the current working directory
([AppImage docs](https://github.com/AppImage/docs.appimage.org/blob/b29179c8f91387098988525c1f901889facab944/source/user-guide/run-appimages.rst)).

Two usable flavours:

| Flavour | Where it extracts | Fit for DDL |
| --- | --- | --- |
| `--appimage-extract` (DDL's own call, cwd set by DDL) | `./squashfs-root` | **Proposed.** D-013's existing fallback; extract once into the engine cache, keep the AppDir for rollback and `-v` smoke tests. |
| `APPIMAGE_EXTRACT_AND_RUN=1` / `appimage-extract-and-run` | `$TMPDIR` or `/tmp` (runtime.c L1492–L1498) | Not preferred: Flatpak's `/tmp` is the app's private dir, and the runtime deletes the tree after every launch — a ~40 MB re-extract per start. |

**Proposed:** when `/.flatpak-info` exists, DDL skips the FUSE attempt entirely and goes straight
to extraction. Trying FUSE first would add a failed mount and a confusing stderr line to every
first launch.

**Confirmed.** `/.flatpak-info` is the documented way for an app to know it is sandboxed
([flatpak-metadata(5)](https://docs.flatpak.org/en/latest/flatpak-command-reference.html)).

### 3.3 Runtime and the engine's shared libraries

**Confirmed.** `org.freedesktop.Platform` 26.08 is the current stable branch (26.08.2, 2026-09-28)
and `org.freedesktop.Sdk.Extension.rust-stable` exists for it
([freedesktop-sdk tags](https://gitlab.com/freedesktop-sdk/freedesktop-sdk/-/tags),
[Flathub API](https://flathub.org/apps/org.freedesktop.Sdk.Extension.rust-stable)).
`org.freedesktop.Platform` is the right choice: DDL is Slint + winit, with no GNOME/KDE toolkit
dependency.

The engine research measured the AppImage's complete non-bundled `DT_NEEDED` set
([dsda-engine.md §7](dsda-engine.md)): `libGL.so.1`, `libOpenGL.so.0`, `libX11.so.6`,
`libX11-xcb.so.1`, `libxcb.so.1`, `libasound.so.2`, `libdrm.so.2`, `libgbm.so.1`,
`libwayland-client.so.0`, `libgpg-error.so.0`, `libjack.so.0`, `libz.so.1`, `libstdc++.so.6`,
`libgcc_s.so.1`, glibc and the loader. Against the 26.08 composition:

| Library | Provided by the runtime | Evidence |
| --- | --- | --- |
| `libGL`, `libOpenGL`, `libEGL` | `components/libglvnd.bst` | `platform.bst` L98 |
| `libX11`, `libxcb`, `libX11-xcb` | `xorg-lib-x11`, `xorg-lib-xcb` | `platform.bst` L201–L203 |
| `libwayland-client` | `components/wayland.bst` | `platform.bst` L185 |
| `libasound` | `components/alsa-lib.bst` | `platform.bst` L9 |
| `libdrm` | `components/libdrm.bst` | `platform.bst` L89 |
| `libgbm` | `components/dummy-gbm.bst`, replaced by the GL extension's real `libgbm` when a driver is loaded | `platform.bst`; `dummy-gbm.bst` description |
| `libgpg-error` | `components/libgpg-error.bst` | `platform.bst` L99 |
| **`libjack.so.0`** | `components/pipewire.bst` ships the JACK shim; the platform includes it | `platform.bst` L158, `pipewire.bst` split rules |
| `libstdc++`, `libgcc_s`, glibc | `public-stacks/runtime-gnu.bst` | `platform.bst` |

**Confirmed via composition, Needs Deck Validation at runtime:** the engine needs glibc ≥ 2.34
(measured in CI); the 26.08 runtime's glibc is comfortably newer, and the engine's own payload
runs against the runtime's glibc, not SteamOS's. The remaining check is to run the extracted
`AppRun -v` inside `flatpak run` on a Deck and record `ldd`'s `not found` set (empty expected).

**Proposed:** do not add `libjack`, SDL or any other system library as a manifest module. The
engine already bundles what it wants; adding a second copy risks two SDL2s and a symbol clash.

### 3.4 Graphics, audio and input

| Need | Flatpak mechanism | Label |
| --- | --- | --- |
| OpenGL | `--device=dri` + the `org.freedesktop.Platform.GL` extension, matched to the host driver by `flatpak run` | Confirmed (docs/code) |
| Wayland / X11 | `--socket=wayland --socket=fallback-x11` | Confirmed |
| Audio | `--socket=pulseaudio` — the docs state it also covers ALSA devices in `/dev/snd` | Confirmed (docs) |
| Gamepads: evdev nodes | `--device=input` dev-binds `/dev/input` (added in Flatpak 1.15.6; [NEWS L1016–L1033](https://github.com/flatpak/flatpak/blob/1.18.4/NEWS#L1016-L1033), [run.c L552–L558](https://github.com/flatpak/flatpak/blob/1.18.4/common/flatpak-run.c#L552-L558)) | Confirmed |
| Gamepads: udev metadata | `--filesystem=/run/udev:ro` — `gilrs-core` 0.6.8 links `libudev-sys` unconditionally on Linux, and libudev reads `ID_INPUT_*` from `/run/udev/data` | Confirmed (code) |
| Raw/virtual input devices | Only `--device=all` ("raw and virtual input currently require `--device=all`") | Confirmed (log text) |
| `/dev/hidraw*` | Not exposed by `--device=input`; only the full `/dev` bind of `--device=all` includes it (run.c L411–L416 vs L552–L558) | Confirmed (code) |

**Proposed — why `--device=input` is enough.** Inside Gaming Mode Steam Input presents the Deck's
controls as a virtual pad, and DDL's own `gilrs` reads controllers through evdev. Raw HID matters
for per-controller extras outside Steam (rumble, light bars, vendor protocols), which DDL does not
use and the engine does not need to start a game. If a Deck test shows a controller that only
appears as `/dev/hidraw*`, `--device=all` is the documented remedy — but it should not be the
default.

**Confirmed.** DDL's own UI needs `/run/udev:ro` too, not just the engine: `gilrs` 0.11.2 →
`gilrs-core` 0.6.8 → `libudev-sys` (`Cargo.lock`). **Proposed.** Two accepted Flathub manifests
cover the two mechanisms separately: `net.odamex.Odamex` uses `--device=input` (with the newer
`--device-if=all:!has-input-device` fallback), and `com.heroicgameslauncher.hgl` uses
`--filesystem=/run/udev:ro`.

**Proposed — `--device=input` over `--device=all`.** `--device=input` exposes only `/dev/input`;
`--device=all` bind-mounts the whole host `/dev` (including `/dev/mem`, `/dev/kvm` and every
disk node that exists). The narrower permission is sufficient for a pad that Steam Input creates
in the kernel. Caveat, **Confirmed** by the linter source: `--device=input` requires
`--require-version=1.16.0` or a conditional-permission fallback, otherwise it is a hard linter
error ([finish_args.py L194–L206](https://github.com/flathub-infra/flatpak-builder-lint/blob/e38c396ec0f593fe11a6b16fe83a33608db33f1e/flatpak_builder_lint/checks/finish_args.py#L194-L206)).
SteamOS 3.10.0 has Flatpak 1.18.4, so this costs Deck users nothing; it excludes older distros.

### 3.5 Process behaviour inside the sandbox

- **Confirmed.** Flatpak unsets `LD_PRELOAD`, `LD_LIBRARY_PATH`, `LD_AUDIT`, `TMPDIR`, `TEMP`
  and `XDG_RUNTIME_DIR` before exec ([run.c L663–L690](https://github.com/flatpak/flatpak/blob/1.18.4/common/flatpak-run.c#L663-L690)).
  DDL already launches with `std::process::Command` and an explicit argv; nothing in the launch
  contract (D-011) depends on host environment variables.
- **Confirmed.** `AppRun` and the payload run from the app's writable data directory, which is a
  normal bind mount; execution is not blocked. Lutris proves the pattern in production.
- **Proposed.** DDL keeps its own working directory, `-config`, `-save`, `-data` and `-shotdir`
  under `$XDG_DATA_HOME/ddl`; inside the sandbox that is `~/.var/app/<id>/data/ddl`, writable.
- **Confirmed.** The engine's IWAD symlinks (D-017) resolve only if the target is visible, which
  the permissions in §5 provide.

## 4. Q2 — Alternatives for the engine

| Option | One engine: official DSDA-Doom | Version pinning + rollback (spec 004) | Sandbox story | Flathub path | Verdict |
| --- | --- | --- | --- | --- | --- |
| **(a)** download AppImage, extract, run in sandbox | yes — unmodified official binary | yes — any reviewed version, multiple installed | strong: engine never touches the host | precedent (Lutris, Heroic, DoomRunner) | **Recommended** |
| (b) `flatpak-spawn --host` | yes | yes | none: engine runs on the host | `--talk-name=org.freedesktop.Flatpak` is a review finding; Lutris and DoomRunner have it | Fallback |
| (c) build DSDA-Doom as a manifest module | yes, from official source | **no** — one version per app build | strong | best: fully from source | Fallback |
| (d) depend on an existing DSDA Flatpak | n/a | n/a | n/a | **none exists** | Unavailable |
| (e) AppImage as `extra-data` source | yes | one pinned build + in-app extras | strong | policy says `extra-data` is for non-redistributable sources | Not recommended |

### (a) Download and extract inside the sandbox

**Proposed.** This is D-013 with one change of control flow: inside a Flatpak, extraction is the
primary path, not the fallback. It preserves every property the owner has already decided on:

- D-001/D-011 — one engine, the official upstream binary, the existing launch contract.
- D-007 — DDL still checks the reviewed SHA-256 list and still warns on unlisted releases.
- D-013 — DDL never mirrors or bundles the engine; it downloads, verifies and runs it.
- Spec 004 — several engine versions can coexist for rollback, independent of the Flatpak
  version.

**Confirmed precedent for runtime engine downloads in Flathub apps:** Lutris downloads emulator
and runtime binaries, Heroic downloads Wine/Proton, and `io.github.Youda008.DoomRunner` is a
Doom launcher that launches engines it does not ship.

### (b) `flatpak-spawn --host`

**Confirmed.** `flatpak-spawn --host` requires the `org.freedesktop.Flatpak` D-Bus interface
(Flatpak documentation); the manifest grants it with `--talk-name=org.freedesktop.Flatpak`. The
linter records `finish-args-flatpak-spawn-access` as a review finding that must be justified in
an exception
([finish_args.py L550–L552](https://github.com/flathub-infra/flatpak-builder-lint/blob/e38c396ec0f593fe11a6b16fe83a33608db33f1e/flatpak_builder_lint/checks/finish_args.py#L550-L552)).

**Confirmed.** Flathub's requirements do not name that interface, but they do say static
permissions "must be kept to an absolute minimum" and that exceptions are allowed case by case
([requirements, read 2026-10-20](https://docs.flathub.org/docs/for-app-authors/requirements)).
Lutris and DoomRunner carry the grant on Flathub today, so it is allowed with justification — not
forbidden.

**Proposed — not chosen.** It keeps the engine on the host, which means the sandbox no longer
describes what actually runs; DDL's own promise ("never execute anything from downloaded
archives") becomes harder to audit; and every engine path must be re-mapped between sandbox and
host. It also removes the only real benefit of the Flatpak for the engine (running against the
runtime's libraries). Keep it as the emergency fallback if extraction is rejected on Deck.

### (c) Build DSDA-Doom from source as a module

**Confirmed.** DSDA-Doom is GPL-2.0-or-later and DDL is GPL-3.0-or-later; a combined GPLv3 work
is permitted by the "or later" clause, and Flathub's source build would satisfy the
corresponding-source obligation. Flathub requires "All source available submissions must be
built entirely from source code … as well as any runtime dependencies included in the manifest"
([requirements](https://docs.flathub.org/docs/for-app-authors/requirements)).

**Proposed — the strongest Flathub-policy option, but a decision change.** It would:

- move the engine to `/app/libexec/` (read-only) and add a "bundled engine" variant to spec 004;
- remove user-visible engine version choice and rollback — the engine version becomes the app
  version, updated by an `x-checker-data` PR on the DSDA-Doom release tarball;
- require DDL to stop downloading an engine at all on that channel (or keep downloads as an
  advanced path, which reintroduces (a) anyway).

That contradicts D-013 and weakens D-007's reviewed-hash model. It is the fallback to choose if
Flathub reviewers refuse a runtime-downloaded engine.

### (d) An existing DSDA-Doom Flatpak

**Confirmed (2026-10-20).** Flathub's search API returns no DSDA-Doom app for `dsda`,
`dsda-doom` or `doom` (the Doom results are DoomRunner, Chocolate Doom, GZDoom, Odamex, Doomsday,
Zandronum, DSDA-adjacent tools). There is nothing to depend on and no maintainer to talk to.

### (e) `extra-data`

**Confirmed.** An `extra-data` source downloads a fixed file at install time into the app's
`/app/extra`
([flatpak-builder reference](https://docs.flatpak.org/en/latest/flatpak-builder-command-reference.html)).
It would pre-seed one pinned AppImage, and DDL could still download others.

**Proposed — rejected.** Flathub reserves `extra-data` for "non-redistributable sources"
([requirements](https://docs.flathub.org/docs/for-app-authors/requirements)); the DSDA-Doom
AppImage is freely redistributable, so using it here invites a review objection. It also bloats
every install by ~14 MB and couples the engine version to the Flatpak release, which is exactly
what D-007/spec 004 avoid. No Flathub precedent uses `extra-data` for a redistributable engine.

## 5. Q3 — Files, paths and permissions

### 5.1 What the sandbox shows by default

**Confirmed.** Without filesystem permissions an app sees the runtime, `/app`, its own
`~/.var/app/<id>` and `$XDG_RUNTIME_DIR/app/<id>`; `$HOME` is an empty writable directory and
`~/.var/app` is excluded from `home` access
([sandbox permissions](https://docs.flatpak.org/en/latest/sandbox-permissions.html)).
**Confirmed.** `XDG_DATA_HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME` and `XDG_STATE_HOME` are set
to `~/.var/app/<id>/{data,config,cache,.local/state}`
([flatpak-context.c L4111–L4116](https://github.com/flatpak/flatpak/blob/1.18.4/common/flatpak-context.c#L4111-L4116)).

**Confirmed — no code change needed in `paths.rs`.** DDL resolves its directories purely from
`XDG_*` with `$HOME` fallbacks (`crates/ddl-core/src/platform/paths.rs`). All four variables are
set, including `XDG_STATE_HOME`, so config, data, cache and state land under
`~/.var/app/<id>/` automatically.

### 5.2 DDL's discovery paths mapped to permissions

| DDL need | Host path | Visible by default? | Permission |
| --- | --- | --- | --- |
| DDL state | `~/.var/app/<id>/{config,data,cache,.local/state}/ddl` | yes, rw | none |
| Steam (native) | `~/.local/share/Steam`, `~/.steam/root` | no | `home:ro` |
| Steam (Flatpak) | `~/.var/app/com.valvesoftware.Steam/...` | no (`home` excludes `~/.var/app`) | explicit grant |
| SD-card Steam libraries | `/run/media/<user>/<disk>/SteamLibrary` | no | `/run/media:ro` |
| Heroic (native) | `~/.config/heroic`, `~/Games/Heroic` | no | `home:ro` |
| Heroic (Flatpak) | `~/.var/app/com.heroicgameslauncher.hgl/...` | no | explicit grant |
| GOG offline installers | `~/GOG Games` | no | `home:ro` |
| minigalaxy (native) | `~/.config/minigalaxy` | no | `home:ro` |
| minigalaxy (Flatpak) | `~/.var/app/io.github.sharkwouter.Minigalaxy/...` | no | not granted in v1 |
| User files (spec 026 picker, D-026) | anywhere under `$HOME` | no | `home:ro` |
| Optional system data | `/usr/share/games/doom` | no | `host-os:ro` (not in v1) |

### 5.3 Proposed `finish-args`

```yaml
finish-args:
  - --require-version=1.16.0
  - --share=ipc
  - --share=network
  - --socket=wayland
  - --socket=fallback-x11
  - --socket=pulseaudio
  - --device=dri
  - --device=input
  - --filesystem=/run/udev:ro
  - --filesystem=home:ro
  - --filesystem=/run/media:ro
  - --filesystem=~/.var/app/com.valvesoftware.Steam:ro
  - --filesystem=~/.var/app/com.heroicgameslauncher.hgl:ro
```

**Confirmed linter consequences** ([finish_args.py @ `e38c396`](https://github.com/flathub-infra/flatpak-builder-lint/blob/e38c396ec0f593fe11a6b16fe83a33608db33f1e/flatpak_builder_lint/checks/finish_args.py)):

| Permission | Linter result | Needs |
| --- | --- | --- |
| `--device=input` | hard error without a minimum version | `--require-version=1.16.0` (present) |
| `home:ro` | `finish-args-home-ro-filesystem-access` review finding | an exception with a reason |
| `~/.var/app/com.valvesoftware.Steam:ro` | `finish-args-flatpak-appdata-folder-com.valvesoftware.Steam-ro-access` | exception |
| `~/.var/app/com.heroicgameslauncher.hgl:ro` | `…-com.heroicgameslauncher.hgl-ro-access` | exception |
| `/run/udev:ro`, `/run/media:ro` | not flagged | — |
| `/usr` (would be `host-os`) | reserved path if requested directly | use `host-os:ro` if ever needed |

**Proposed rationale for each exception** (this is what the exception requests would say):

- `home:ro` — DDL is a launcher for content the player already owns: Steam/GOG IWADs, local
  WADs and user-chosen files. It reads them in place and never modifies them (D-026). Portals
  cannot replace this because the controller-first picker must browse directories offline and
  because the engine needs stable paths, not per-session document-portal paths.
- `~/.var/app/com.valvesoftware.Steam:ro` and `…hgl:ro` — read-only IWAD discovery when the
  player runs the Steam or Heroic **Flatpak**; `home:ro` explicitly excludes `~/.var/app`.

**Proposed — deliberately absent:**

- `--device=all` — unnecessary and over-broad (see §3.4).
- `--talk-name=org.freedesktop.Flatpak` — no host execution in the recommended design.
- `home:rw` — DDL never writes outside its own XDG directories.
- `host-os:ro` — only needed for `/usr/share/games/doom`, a generic distro location that a Deck
  user is unlikely to populate; DDL scans it but works without it. `net.odamex.Odamex` requests
  `host-os:ro` for exactly that reason, so it is a known-acceptable follow-up if a non-Deck user
  needs it.
- `/mnt:ro` or `/media:ro` — same reason; the picker cannot browse what is not mounted.

### 5.4 Portals as an alternative

**Confirmed (requirements, read 2026-10-20).** Flathub's rule is that "static permissions must be
kept to an absolute minimum", that apps should use XDG portals "wherever possible", and that
"when a suitable XDG portal exists, is supported across the ecosystem, and covers the
application's use case … using the portal becomes mandatory instead of static permissions", with
exceptions allowed case by case
([requirements](https://docs.flathub.org/docs/for-app-authors/requirements)). That is why the
permissions in §5.3 need written reasons and why the OSK uses OpenURI.

**Proposed — not sufficient for files.** `org.freedesktop.portal.FileChooser` returns a
document-portal path valid for the current session and shows a GTK dialog. That breaks the
controller-first picker of spec 026, and D-026's "reference files in place" model needs a stable
path stored in the profile. Portals are the right tool for a one-off import, not for the library.

**Proposed — the right portal use is the OSK** (§6.3), not file access.

### 5.5 Existing installs

**Confirmed.** There is no automatic migration: a previous `~/.local/share/ddl` (engines,
installed mapsets, IWAD symlinks) and `~/.config/ddl` (profiles, settings) are invisible to the
Flatpak build. `home:ro` makes the old tree *readable*, so an in-app "import existing DDL data"
can copy it into `~/.var/app/<id>/` without any extra permission. **Proposed:** offer that import
once, on first run, and never write to the old location.

**Confirmed.** Flathub's own guidance is to avoid sharing configuration with non-Flatpak
installations; a one-way import is the compatible reading.

## 6. Q4 — Steam and Gaming Mode

### 6.1 Discover does not add apps to Steam

**Confirmed (source inspection, not documentation).** Discover `v6.7.3` — the version in the
SteamOS 3.10.0 manifest — has no "Add to Steam" action. A case-insensitive search of every
`.cpp`, `.h` and `.qml` file finds only the SteamOS update backend (`HoloBackend`,
`SteamOSBackend`) and a comment about `steam.desktop`. The only Steam-facing code is the
system-update backend.

So the Flatpak flow still ends with the manual step from the current guide: Desktop Mode →
Steam → **Games → Add a Non-Steam Game to My Library…**. Two ways to fill it:

- pick the DDL desktop entry if Steam lists it (Flatpak exports it under
  `~/.local/share/flatpak/exports/share/applications/`), or
- **Browse…** to `/usr/bin/flatpak` and set launch options to
  `run io.github.vittau.Doom-Deck-Launcher`.

**Needs Deck Validation:** which of the two the current Steam client offers, and whether it
shows the Flatpak-exported entry. There is no `steam://` URL that creates a shortcut with a
given target (spec 043 research).

### 6.2 Steam Input and the overlay

- **Confirmed.** Steam Input still works: Steam creates the virtual pad in the kernel, and the
  Flatpak reads it as a normal evdev device through `/dev/input` (`--device=input`) plus
  `/run/udev:ro`. The per-shortcut layout mechanism is unchanged (D-033's guide text still
  applies).
- **Confirmed (code).** The Steam overlay cannot attach: Steam injects
  `gameoverlayrenderer.so` through `LD_PRELOAD`, and Flatpak unsets `LD_PRELOAD` before exec
  ([run.c L669](https://github.com/flatpak/flatpak/blob/1.18.4/common/flatpak-run.c#L669)).
  `SteamAppId`/`SteamGameId` are *not* unset, so DDL can still detect a Steam launch.
  Consequences: no Steam FPS overlay, no Shift+Tab overlay in DDL, no Steam screenshot hotkey in
  DDL. DDL's own screenshots are unaffected.
- **Needs Deck Validation:** whether the missing overlay changes the Steam+X keyboard, focus, or
  the controller layout picker in Gaming Mode.

### 6.3 The on-screen keyboard

**Confirmed.** `Command::new("steam")` fails inside the sandbox: `/usr/bin` is the runtime, and
`/app/bin` contains only DDL's own files. `steam` is not on `$PATH`.

**Proposed.** Use the OpenURI portal through `flatpak-xdg-utils`' `xdg-open`, which the runtime
provides ([`platform.bst` L31](https://gitlab.com/freedesktop-sdk/freedesktop-sdk/-/blob/master/elements/platform.bst))
and which routes to `org.freedesktop.portal.OpenURI`:

```text
Command::new("xdg-open").arg("steam://open/keyboard")
```

The host's default handler for `x-scheme-handler/steam` is the Steam client. This is the same
URI SDL uses (`docs/research/steam-osk.md`), carried by the portal instead of a direct spawn.

**Needs Deck Validation:** whether the portal has a working OpenURI backend in **Gaming Mode**.
SteamOS 3.10.0 ships `xdg-desktop-portal` 1.22.1, `-gtk` 1.15.3, `-kde` 6.7.3, `-holo` and
`-gamescope`; the gamescope backend implements only Access, ScreenCast and Screenshot, so
OpenURI must come from the gtk or holo backend. If it does not work, the existing `Steam + X`
hint (already shown) remains the fallback, and `flatpak-spawn --host` is the last resort — at the
cost of the Flatpak D-Bus permission.

### 6.4 Launch advice for the guide

**Confirmed.** The current advice — leave the shortcut's Compatibility tool unset — still holds,
for a new reason: the Flatpak carries its own runtime, and forcing a Steam Linux Runtime around
`/usr/bin/flatpak` would run the Flatpak tool inside an older container. **Proposed:** the guide
should say "Compatibility tool: none", offer the `/usr/bin/flatpak run <id>` recipe, and keep the
note that artwork is still manual (Discover supplies the window icon and store page, not the
Steam library capsule).

**Needs Deck Validation:** launch from Gaming Mode, gamescope/Wayland negotiation, controller
focus, suspend/resume.

## 7. Q5 — The Flathub path

### 7.1 Submission checklist

| Requirement | DDL status / work | Label |
| --- | --- | --- |
| App id `io.github.vittau.Doom-Deck-Launcher` | valid shape: 4 components, and a dash is allowed in the last component ([requirements, read 2026-10-20](https://docs.flathub.org/docs/for-app-authors/requirements)) | Confirmed (policy) |
| Code-hosting id maps to a repository URL | computed `github.com/vittau/doom-deck-launcher` resolves to the actual repository (GitHub is case-insensitive; checked 2026-10-20) — no exception, no rename | Confirmed (policy/tooling) |
| Manifest at repo root, named after the id | to write | — |
| Runtime on Flathub, latest version | `org.freedesktop.Platform` 26.08 | Confirmed |
| Build entirely from source, no network at build | Rust vendored via `flatpak-cargo-generator` (`CARGO_NET_OFFLINE=true`, `cargo --offline fetch/build`) + `org.freedesktop.Sdk.Extension.rust-stable` | Confirmed (tooling) |
| No binaries in the submission PR | DDL's sources only; no engine, no screenshots beyond metadata assets | Confirmed (rule) |
| License installed to `$FLATPAK_DEST/share/licenses/$FLATPAK_ID` | add `LICENSE` install step | Confirmed (rule) |
| Metainfo `%{id}.metainfo.xml` with matching id | to write | — |
| Screenshots (direct image URLs, tag/commit, not a branch) | reuse D-024's bundled shots, served from a release tag on GitHub | Confirmed (rule) |
| OARS `content_rating type="oars-1.1"` | to fill; the app displays violent game screenshots and installs Mature content | Confirmed (rule) |
| `releases` tag with dates | generate from the changelog | Confirmed (rule) |
| Desktop file + SVG/256 px icon | add; `Categories=Game;` | Confirmed (rule) |
| `flathub.json` | `{"only-arches": ["x86_64"]}` — DDL is x86_64-only | Proposed |
| Linter findings | `home:ro` + two `~/.var/app` exceptions, each with a written reason (§5.3); no hard errors | Confirmed (linter source) / Proposed (reasons) |
| English UI and metadata | DDL is English (D-008) | Confirmed |

### 7.2 The policy risk that matters most

**Confirmed (requirements page, read 2026-10-20).** Flathub's Generative AI policy says, among
other things:

- submitters must disclose AI-generated code, documentation, packaging and other material, with
  the affected parts and approximate extent;
- **"Flathub manifests must not contain AI-generated or AI-assisted content"**, and disclosure
  does not exempt manifests;
- "AI tools or agents must not open or automate Flathub submission pull requests, or generate
  their commit messages, descriptions, review comments, or replies";
- AI used only for research, discussion or debugging needs no disclosure.

DDL is developed with AI agents (this repository's `AGENTS.md`/orchestration is the evidence).
That is not automatically disqualifying for the *application* — it must be disclosed, and
reviewers have discretion — but it **does** mean the Flatpak manifest and every submission
interaction must be written by the owner, not by an agent. This is a process constraint the
orchestration model has to respect, and it is the single most likely reason a submission would be
rejected if handled mechanically.

**Proposed:** the owner writes `io.github.vittau.Doom-Deck-Launcher.yml`, the metainfo, the
permission reasons and the PR; agents may research, review and test locally, and must not author
or post submission text.

### 7.3 The other inclusion risks

**Confirmed (requirements page).** Two clauses could be read against a launcher:

- "**Host-dependent applications** — Applications that rely on host components or complicated
  post installation setups for core functionality will not be accepted. Exceptions can be
  considered on a case by case basis."
- "**Minimal submissions** — … exists as a thin wrapper or launcher around other tools will not
  be accepted."

**Proposed counter-argument:** after the Flatpak change DDL is not host-dependent — the engine,
content and configuration all run/read inside the sandbox, and the only host dependency is the
content the player already owns, which is the same situation as Lutris, Heroic and DoomRunner.
DDL is also far more than a wrapper: bundled catalog with hashes, downloader and safe extractor,
IWAD identification, profile/config management, controller-first UI. Three comparable launchers
are accepted on Flathub today.

**Needs owner validation:** whether "launcher" in the Minimal submissions clause is applied to a
curated game launcher with its own catalog. The only way to know is a human-written submission
PR.

### 7.4 Review and publication

**Confirmed (submission FAQ, read 2026-10-20):**

- reviewers are volunteers, "there is no definite time limit", and merges happen in batches;
- after merge, publication is "usually … within 1-2 hour", and the store page appears within a
  few hours;
- the submitter receives write access to the new `flathub/<id>` repository and must have 2FA and
  accept within a week;
- updates are normal PRs to that repository; an automated external-data-checker runs every two
  hours on the default branch and opens `x-checker-data` PRs.

**Proposed `x-checker-data`:** point it at the GitHub *release* (tag + source archive) so a DDL
release produces a PR, and keep the engine out of it (the engine is downloaded at runtime).

**SteamOS specifics. Confirmed (secondary).** SteamOS ships Flathub as a preconfigured remote for
Discover, so a published app appears in search without the player adding anything
([GamingOnLinux, 2025-01-13](https://www.gamingonlinux.com/guides/view/how-to-install-extra-software-apps-and-games-on-steamos-and-steam-deck)).
**Needs Deck Validation:** run `flatpak remotes` on a Deck to confirm the remote is user-scoped and
enabled, and check that Discover lists a newly published app without a restart.

### 7.5 Self-hosted repository instead of Flathub

**Confirmed.** A `.flatpakrepo` (remote) plus `.flatpakref` (single app) can be served from any
HTTPS host; a static ostree repo with `flatpak build-update-repo --generate-static-deltas` and a
GPG key is the documented shape
([Flatpak repository docs](https://docs.flatpak.org/en/latest/repositories.html)).

**Confirmed (source).** Discover `v6.7.3` installs `.flatpakref` files and adds `.flatpakrepo`
remotes, both from a local file and from an HTTP(S) URL
([FlatpakBackend.cpp L459, L1642](https://github.com/KDE/discover/blob/v6.7.3/libdiscover/backends/FlatpakBackend/FlatpakBackend.cpp)).

**Needs Deck Validation:** whether SteamOS's Discover is registered as the handler when a
`.flatpakref` link is opened, and whether a user can reach "Install from file" in Desktop Mode.

**Proposed:** a self-hosted repo is the right **pre-Flathub** channel (no review, fast
iteration, GPG-signed), not the end state: it has no Discover search result, no store page and no
review badge.

## 8. Q6 — Self-update and catalog

| Concern | Behaviour | Label |
| --- | --- | --- |
| App binary update | Flathub replaces `/app`; the Flatpak ref is the update unit. `~/.var/app/<id>` is untouched. | Confirmed |
| In-app self-update | Must not exist. Detect `/.flatpak-info` and say updates come from the system/Discover. | Proposed |
| Catalog (D-019/D-025) | Embedded catalog updates with the app; the once-a-day `catalog.json` fetch on the `catalog` branch works unchanged (`--share=network`, rustls + bundled roots, no host CA needed). | Confirmed (code) |
| Installed mapsets | Live in `~/.var/app/<id>/data/ddl`; unaffected by app updates. | Confirmed |
| Engine downloads | Same data dir; survive updates. The reviewed-hash list is compiled into the binary, so an app update can change what is offered but never deletes installed engines. | Proposed |
| Engine rollback (spec 004) | Unaffected: still in-app, independent of the Flatpak version. | Confirmed (design) |
| App rollback | Flathub keeps previous builds; `flatpak update --commit=<sha>` downgrades. This is a CLI/advanced action, not something DDL can drive. | Confirmed (Flathub docs) |
| Update prompt UX | **Proposed:** hide the app-update UI when sandboxed; optionally open Discover via `appstream://<id>` through the portal. | Needs Deck Validation |

**Proposed:** do not try to update the Flatpak from inside the app. Writing to `/app` is
impossible (read-only), and `flatpak install`/`update` from the sandbox would need the Flatpak
socket, which is exactly the broad permission the design avoids.

## 9. Q7 — Recommendation, friction and draft specs

### 9.1 Recommended architecture

**Download the official AppImage at runtime and run it extracted inside the sandbox**, with the
Flatpak as the delivery and update channel for DDL itself. Specifically:

1. Ship DDL as `io.github.vittau.Doom-Deck-Launcher` on Flathub, runtime
   `org.freedesktop.Platform` 26.08, built from source with vendored Cargo dependencies.
2. When `/.flatpak-info` is present, skip FUSE and extract the AppImage with
   `--appimage-extract` into DDL's engine cache, then run the AppDir's `AppRun` with the D-011
   launch contract.
3. Request the permissions in §5.3 with written exception reasons for `home:ro` and the two
   `~/.var/app` grants.
4. Disable app self-update in the sandbox; keep catalog and engine downloads in-app.
5. Keep the Steam shortcut manual and document it.

This is the only option that keeps D-001, D-007, D-011 and D-013 intact **and** gives a working
sandbox, and it mirrors what Lutris, Heroic and DoomRunner already do on Flathub.

### 9.2 Friction, Discover versus today's tarball

| Step | Today (GitHub tarball) | Flatpak from Discover |
| --- | --- | --- |
| 1. Reach a desktop | Desktop Mode | Desktop Mode |
| 2. Get the app | Browser → GitHub Releases → download `ddl-linux-x86_64.tar.gz` | Discover → search "DoomDeckLauncher" → Install |
| 3. Install it | Extract with Dolphin; keep the folder together | nothing |
| 4. Add to Steam | Games → Add a Non-Steam Game → Browse… → select `ddl` | same, choosing the exported entry or `/usr/bin/flatpak` + `run <id>` |
| 5. Name and artwork | Rename, icon, capsule, hero, logo (4–6 manual picks) | same; Discover's icon is not a Steam capsule |
| 6. First launch | Desktop Mode once | Desktop Mode once |
| 7. Updates | Re-download, re-extract, replace; shortcut may point at the old folder | Discover → Updates (automatic), user data kept |
| 8. Uninstall | Delete the folder; profile/data left behind | Discover → Uninstall (asks about app data) |

**Proposed reading:** the Flatpak removes the two steps that require file management and makes
updates automatic; it does **not** remove the Desktop Mode trip or the Steam/artwork work. The
honest gain is "no terminal, no archive, updates handled", not "install from Gaming Mode".

### 9.3 Draft spec outlines

| Draft spec | Scope | Size |
| --- | --- | --- |
| **059 — Flatpak packaging and manifest** | App id `io.github.vittau.Doom-Deck-Launcher` (matches the repository URL) and `io.github.vittau.Doom-Deck-Launcher.yml`; runtime 26.08 + rust-stable extension; `cargo-sources.json` generated from `Cargo.lock`; desktop file, SVG icon, metainfo with screenshots at a tag and OARS; license install; `flathub.json` `only-arches`; finish-args from §5.3; local `flatpak-builder` + `flatpak-builder-lint` runs in CI; a CI job that builds the Flatpak and runs `ddl --version` inside it. | L |
| **060 — Sandbox-aware paths and content** | Detect `/.flatpak-info`; skip FUSE and extract first; verify `paths.rs` under the sandbox's `XDG_*`; IWAD discovery through `home:ro`, `/run/media:ro` and the two `~/.var/app` grants; a "path not visible" message instead of a silent empty list; one-time import of a pre-Flatpak `~/.local/share/ddl` + `~/.config/ddl`; tests with a fake sandbox marker and fake roots. | M |
| **061 — Steam, Gaming Mode and the guide** | Replace `Command::new("steam")` with the portal `xdg-open steam://open/keyboard` path when sandboxed, keeping `Steam + X` as the visible fallback; document the Flatpak install, the `/usr/bin/flatpak run <id>` shortcut, "Compatibility tool: none", artwork, and the missing overlay; Deck checklist for launch, suspend/resume, controller focus, OSK. | M |
| **062 — Update channel split** | Hide the app self-update path when sandboxed; explain that updates come from the system; keep catalog (D-025) and engine downloads; report the Flatpak version and the engine version separately in About; decide whether to offer an "Open Discover" action via `appstream://`. | S |
| **063 — Flathub submission and maintenance** | The human-owned submission PR, exception texts, verification, `x-checker-data` on the DDL release, release checklist (tag → update manifest → test build → merge), runtime upgrade cadence, and the rule that submission text is never agent-written (Flathub AI policy). | M |

### 9.4 Risks

| Risk | Impact | Mitigation | Label |
| --- | --- | --- | --- |
| Flathub AI policy forbids AI-assisted manifests and agent-run submission | Submission rejected; process must change | Owner writes manifest/metainfo/PR; agents only research and test; disclose AI-assisted app source | Confirmed (policy) |
| "Thin wrapper / host-dependent" inclusion clauses | Possible rejection | Point at Lutris/Heroic/DoomRunner; emphasise catalog, downloads, IWAD ID, profiles | Confirmed (text) / owner decision |
| Permission review (`home:ro`, two `~/.var/app`) | Exceptions refused or delayed | Written reasons in §5.3; offer to drop the Steam-Flatpak grant first | Proposed |
| App-id choice | A wrong id blocks submission or forces a repository rename | Use `io.github.vittau.Doom-Deck-Launcher`: dashes are allowed in the last component, and the computed (lowercased) URL resolves to the repository | Confirmed (policy/tooling) |
| Engine extraction inside the sandbox | Startup failure, missing libs | Deck test of `flatpak run` + extracted `AppRun -v`; `ldd` sweep in CI | Needs Deck Validation |
| No Steam overlay | Lost FPS overlay/screenshots | Document; DDL's own screenshots and UI are unaffected | Confirmed (code) |
| OSK portal unavailable in Gaming Mode | Profile names need `Steam + X` | Keep the hint; last resort `--talk-name=org.freedesktop.Flatpak` | Needs Deck Validation |
| Rust pin vs. `rust-stable` extension | Flatpak build uses a different compiler than the GitHub release | Validate the extension's version; if it is older than 1.99, vendor the exact toolchain as a source | Needs Build Validation |
| Bundled community screenshots in the app | Copyright/review questions | Keep the provenance files, credit in About, removal on request (D-024) | Proposed |
| Flatpak-only users lose their old DDL data | App appears empty after install | One-time import from `~/.local/share/ddl` | Proposed |
| Flatpak ≥ 1.16 requirement | Older distros cannot install | Accept for Deck; keep `--device=all` as a documented fallback for a wider build | Proposed |

### 9.5 What would change an owner decision (questions, not edits)

- **D-002 (build and release on GitHub Actions).** A second release channel (Flathub) means
  two artifacts from one tag. The GitHub tarball can stay the primary/offline channel; Flathub
  becomes the recommended one in the guide. The build still happens in CI; Flathub builds from
  source with its own toolchain, so the two binaries are not byte-identical.
- **D-013 (engine acquisition).** No change to *what* is acquired; only *how* it is started
  inside a sandbox (extraction always, never FUSE). Worth recording as an explicit clause so a
  future reader does not reintroduce a FUSE attempt.
- **D-001/D-011.** Unchanged.
- **A new decision** on distribution (Flathub as the recommended channel, GitHub tarball as the
  offline/advanced one) and on the human-only Flathub process imposed by the AI policy.

Open questions for the owner are listed in the report.

## 10. What was not verified here

- Running the extracted engine inside `flatpak run` on a Deck (libraries, GL, audio, pad).
- Gaming Mode: shortcut creation, launch, gamescope/Wayland, Steam Input, overlay absence, OSK
  portal.
- Steam's "Add a Non-Steam Game" dialog listing the Flatpak-exported desktop entry.
- A real `flatpak-builder` build of DDL (Rust extension version, Slint build time, vendored
  crate size).
- Whether Discover on SteamOS opens a `.flatpakref` link from a browser.

These are collected in the report as open questions and, where they need hardware, are Deck
checks.

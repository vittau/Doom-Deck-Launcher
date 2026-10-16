# DoomDeckLauncher

**A native DOOM mapset launcher for Steam Deck.**

DoomDeckLauncher (DDL) is a small, native desktop application — Rust and
[Slint](https://slint.dev), no Electron, no web view — whose goal is a
controller-only DOOM experience on SteamOS:

> Open DDL → discover a compatible mapset → install → play, entirely with the controller.

DDL manages an unmodified official [DSDA-Doom](https://github.com/dsda-org/dsda-doom)
AppImage and vetted community mapsets, so you never have to type a command line or
pick a complevel by hand. It is built for **Steam Deck / SteamOS (Gaming Mode
first)**, Linux x86-64.

## Status: pre-alpha

**DDL is not usable yet.** The current release is a placeholder: it opens a
fullscreen window with a three-entry menu ("Play", "Settings", "Quit") so the build,
packaging and release pipeline can be validated end to end on real hardware.
Nothing is downloaded, no game is launched, and the menu entries log a message and
do nothing.

What exists today:

- a Rust/Slint window that opens fullscreen (or in a 1280×800 window with
  `--windowed`), navigable with the arrow keys and Enter/Escape;
- `ddl --version` and `ddl --help`;
- the XDG directory resolution and the logging foundation the later work builds on;
- a CI and release pipeline that produces the Linux x86-64 tarball described below.

Gamepad input, DSDA-Doom installation, IWAD detection, the curated catalog and the
real screens are still ahead. Follow the repository's specs and journal for progress.

## Install (from a GitHub Release)

> Pre-alpha: installing today gets you the placeholder screen. It is still worth
> doing if you want to help confirm that the released binary starts on your Deck.

1. On the Steam Deck, download **`ddl-linux-x86_64.tar.gz`** from the
   [latest release](https://github.com/vittau/Doom-Deck-Launcher/releases).
   Each release also publishes `ddl-linux-x86_64.tar.gz.sha256`; verifying it is
   encouraged but the download is small and re-downloadable.
2. Extract the archive. It contains a single top-level `ddl/` directory:

   ```text
   ddl/
   ├── ddl        # the executable
   ├── LICENSE
   └── README.md
   ```

   In Desktop Mode you can extract it wherever you like, for example
   `~/Applications/`. Keep the directory together.
3. Add it to Steam as a **non-Steam game**: in Desktop Mode open Steam →
   **Games → Add a Non-Steam Game to My Library…** → **Browse…** and select the
   `ddl` executable you just extracted.
4. DDL now appears in your library. Launch it once from Desktop Mode to check it
   starts, then switch to Gaming Mode and launch it from your library like any
   other game.

There is no installer, no background service and no root access involved: everything
DDL needs lives in the extracted directory and in your user's XDG directories.

## Build from source

Requirements:

- Rust **1.99.1** — the exact toolchain is pinned in `rust-toolchain.toml`, so
  `rustup` picks it up automatically;
- on Linux: `pkg-config` and fontconfig development headers
  (`libfontconfig1-dev` on Debian/Ubuntu) plus the usual X11/Wayland development
  packages; macOS needs the Xcode command line tools.

```sh
git clone https://github.com/vittau/Doom-Deck-Launcher.git
cd Doom-Deck-Launcher
cargo build --release -p ddl
./target/release/ddl --help
```

Run the checks the project requires before opening a pull request:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Release binaries are not built on a development machine: they are produced by
GitHub Actions inside a pinned Steam Linux Runtime SDK container, so that they run
against the libraries SteamOS actually provides. See
[`docs/research/steam-runtime-build.md`](docs/research/steam-runtime-build.md).

### Command-line options

```text
Usage: ddl [OPTIONS]

Options:
      --windowed  Run in a 1280x800 window instead of fullscreen
  -h, --help      Print this help and exit
  -V, --version   Print version information and exit

Environment:
  DDL_LOG  Log level for stderr diagnostics: error, warn, info, debug or trace
           (default: info)
```

## License

DDL is licensed under the **GNU General Public License, version 3 or later**
(GPL-3.0-or-later). See [`LICENSE`](LICENSE).

DDL is not affiliated with, endorsed by, or sponsored by id Software, Bethesda
Softworks, ZeniMax, Microsoft, Valve Corporation, Doomworld, or the DSDA-Doom
maintainers. **DOOM** is a trademark of id Software LLC; **Steam** and **Steam Deck**
are trademarks of Valve Corporation. All trademarks belong to their respective
owners and are used here only to describe what DDL does.

## Credits

- **[UZDeck](https://github.com/karkusviktor/UZDeck)** and
  **[GZDeck](https://github.com/flegald/GZDeck)** — the projects that inspired DDL.
  DDL is an independent reimplementation; no original code or assets have been
  reused.
- **[DSDA-Doom](https://github.com/dsda-org/dsda-doom)** — the DOOM source port DDL
  targets, used unmodified as the official AppImage.
- **[Slint](https://slint.dev)** — the native UI toolkit DDL is built with.
- The DOOM mapping community, whose mapsets DDL exists to make playable.

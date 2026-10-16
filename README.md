<p align="center"><img src="docs/design/logo/full-name.svg" alt="DoomDeckLauncher — Ember Gate logo" width="700"></p>

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
   ├── art/       # Steam icons, capsules, hero and transparent logo
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

> **Leave the shortcut's Compatibility tool unset.** DDL runs on the host system,
> and so does the official DSDA-Doom AppImage it launches. Forcing a Steam Linux
> Runtime ("Compatibility tool" in the shortcut's Properties) makes the engine run
> on an older runtime whose glibc is too old for it: DSDA-Doom then fails to start
> with `GLIBC_2.34 not found` (confirmed in CI). Set the Compatibility tool to
> **None** / leave it unset. DDL cannot change this for you.

There is no installer, no background service and no root access involved: everything
DDL needs lives in the extracted directory and in your user's XDG directories.

Once DDL can launch a game, set up your controls as described in
[Recommended Steam Deck Controller Setup](#recommended-steam-deck-controller-setup):
Steam Input is configured by hand in Gaming Mode, and DDL never edits your controller
files.

## Make it look native in Steam

The release includes original Ember Gate artwork in `ddl/art/`. In Desktop Mode,
add DDL as a non-Steam game, then apply the images manually:

1. Open the shortcut's **Properties**, click its icon and select `art/icon-256.png`
   (or `icon-512.png`). Choose PNG / all files in the picker if needed.
2. In the Library grid, right-click DDL → **Manage → Set custom artwork** and
   choose `art/library-capsule.png`. For its wide Recent Games tile, repeat with
   `art/wide-capsule.png` on that tile (it appears after launching DDL).
3. Open DDL's Library details page, right-click the large background → **Set
   custom background** → `art/hero.png`. Right-click there again → **Set custom
   logo** → `art/logo.png`. Adjust the logo position if desired.
4. Return to Gaming Mode and check the artwork in your Library.

| File in `art/` | Size |
| --- | --- |
| `icon-256.png`, `icon-512.png` | 256×256, 512×512 |
| `library-capsule.png` | 600×900 |
| `wide-capsule.png` | 920×430 |
| `hero.png` | 3840×1240 |
| `logo.png` (transparent) | 1280×720 |

**Confirmed:** sizes follow Valve's [Library Assets](https://partner.steamgames.com/doc/store/assets/libraryassets)
and [Shortcut Icon](https://partner.steamgames.com/doc/store/assets/community) specifications
(reviewed 2026-10-16). These are the intended artwork dimensions; they are not a
claim that the custom picker rejects other sizes. Menu wording can vary with the
Steam client. **Needs Deck Validation:** the final crop and appearance in Gaming
Mode. DDL never writes Steam configuration; all steps above are manual.

Repository owners can upload [social-preview.png](docs/design/logo/social-preview.png)
(1280×640) in GitHub **Settings → General → Social preview**. See the
[logo usage and export guide](docs/design/logo/README.md) for SVG masters.

## Recommended Steam Deck Controller Setup

**Steam Input is manual by design.** DDL does not install, import or edit controller
layouts, and it never touches Steam's `.vdf` configuration files. You build the layouts
yourself, with the Deck's own controls, using the steps below.

> **Names change.** Steam relabels its menus from time to time. The Steam UI names here
> are **as of 2026-10 and may change** — match them to what your Deck shows. Names that
> come from DSDA-Doom itself are given with the config key DDL uses, which is stable.

Two layouts are suggested:

- **Classic** — plain gamepad, no gyro. The simplest setup and the closest to
  DSDA-Doom's own defaults. Start here.
- **Modern + Gyro** — the same gamepad controls plus **Gyro To Mouse** for fine aiming,
  activated while your thumb rests on the right stick.

> **Pre-alpha:** DDL's own screens are not controller-navigable yet (see
> [Status](#status-pre-alpha)) and cannot launch a game. This section is therefore
> preparation: it describes the layouts you will use once gamepad input and launching
> land, and the settings below are the ones DSDA-Doom already defaults to.

### 1. Open the DDL controller settings (controller only)

1. In **Gaming Mode**, open your **Library** and select the **DoomDeckLauncher**
   shortcut you added during [install](#install-from-a-github-release).
2. Press the **controller icon** on the right side of the game page, next to the gear
   icon. This opens **Controller Settings** for that shortcut.
3. Select **Edit Layout**.

You are now editing a layout that belongs to this shortcut alone. It does not affect
your other games, and you do not need to browse or import Community Layouts — build
both layouts from the default gamepad template so you know exactly what each control
sends.

Changes apply to the shortcut as soon as you make them. To keep a named copy you can
switch back to later, press the **cog** beside **Edit Layout** → **Export Layout** →
**Save New Layout**, and give it a name such as "DDL Classic". Choose **New Personal
Save** so it stays attached to this shortcut.

### 2. Suggested layout A — Classic (no gyro)

In **Edit Layout**, pick a control from the left-hand list and assign the command from
the table. The right-hand column is what DSDA-Doom itself does with that input, using
its own defaults — **you do not need to rebind any buttons inside DSDA-Doom.**

| Deck control | Steam Input assignment | What DSDA-Doom does with it (its default) |
| --- | --- | --- |
| Left stick | Left joystick (leave as-is) | Move: `input_forward`, `input_backward`, `input_strafeleft`, `input_straferight` |
| Right stick | Right joystick (leave as-is) | Turn / look |
| Right trigger | Right trigger | **Fire** — `input_fire` |
| Left trigger | Left trigger | **Automap** toggle — `input_map` |
| A | A | Use / open doors — `input_use` |
| B | B | Jump — `input_jump` |
| X / Y | X / Y | Previous / next weapon — `input_prevweapon` / `input_nextweapon` |
| Left bumper (L1) | Left bumper | Strafe modifier — `input_strafe` |
| Right bumper (R1) | Leave unassigned | `input_use_artifact` exists for Heretic/Hexen artifacts, but DOOM does not use it |
| L3 / R3 (stick clicks) | Stick clicks | Autorun toggle — `input_autorun`; 180° reverse — `input_reverse` (disabled while recording, see §6) |
| D-pad | D-pad | Menu navigation — `input_menu_up`, `input_menu_down`, `input_menu_left`, `input_menu_right` |
| A / B in menus | A / B | Confirm — `input_menu_enter`; go back — `input_menu_backspace` |
| Start | Start | Open/close the game menu — `input_menu_escape` |
| View (Back) | Back | Clear a binding in DSDA-Doom's controls menu — `input_menu_clear` |
| Right trackpad | **As Mouse** | Aiming and menu clicking |

Most of these already match the default gamepad template, so on a fresh shortcut you
may only need to set the right trackpad.

### 3. Suggested layout B — Modern + Gyro

Start from Classic, then add gyro and set the trackpad. Still controller-only:

1. In **Edit Layout**, choose **Gyro** from the list of controls.
2. Set the gyro **Behavior** to **Gyro To Mouse**. (The current name; older Steam
   clients call the same mode "As Mouse".)
3. Open the gyro activation settings — Steam labels these something like *Choose Gyro
   Button(s)* / *Gyro Enable Button* — and set **Right Stick Touch**. Gyro then only
   turns on while your thumb is on the right stick.
4. Go back, choose **Trackpads → Right Trackpad**, and set its **Behavior** to
   **As Mouse**.
5. Leave the right stick as a plain joystick.

The result:

| Deck control | Steam Input assignment | Why |
| --- | --- | --- |
| Left stick | Left joystick (unchanged) | Movement |
| Right stick | Right joystick (unchanged) | Fast, big turns |
| Gyro | **Gyro To Mouse**, activated by **Right Stick Touch** | Fine aim while your thumb is on the stick |
| Right trackpad | **As Mouse** | Precision aim and menus |
| Rear buttons (L4/R4, optional) | Gyro button, set to toggle on/off | Only if you prefer a manual on/off over touch activation |
| Triggers, face buttons, bumpers | Same as Classic | Fire, use, weapons, map |

### 4. Gamepad and mouse at the same time — avoid double input

- Steam Input can emit **two kinds of events at once**: a virtual controller (Xbox
  style) and keyboard/mouse events. A gyro-as-mouse layout uses both.
- DDL's DSDA-Doom baseline deliberately enables both sources:
  `use_game_controller 1` **and** `use_mouse 1`. The pad drives movement, turning and
  buttons; the mouse path drives gyro and the right trackpad. That coexistence is the
  whole point of the Modern + Gyro layout.
- **Bind one action once.** DSDA-Doom already binds the right trigger, A, the bumpers
  and so on by default, so do not *also* send a mouse click or a keyboard key from the
  same Deck control in Steam. If you want to change something, change it in Steam and
  leave DSDA-Doom at its default, or change it in DSDA-Doom's own menu — never both.
- DDL's own launcher UI **ignores pointer motion**: it hides the cursor and only reacts
  to a click, so a gyro or trackpad mouse layout will not fling a pointer around the
  menu. Navigating DDL is gamepad-first — D-pad or left stick to move, A to select, B to
  go back (decision D-009).

### 5. Where to tune sensitivity

Two places influence the same movement. Change one at a time and test.

- **Steam side.** Gyro and trackpad sensitivity live in the same **Edit Layout** screens
  — for example the sensitivity under **Gyro To Mouse**, and the **As Mouse** sensitivity
  on the right trackpad. These set how much *mouse movement* Steam sends.
- **DSDA-Doom side.** Stick look speed and mouse response are DSDA-Doom settings, tuned
  in its own **Options** menu. The values DDL provisions are:
  - right-stick look: `right_analog_sensitivity_x`, `right_analog_sensitivity_y`,
    `analog_look_acceleration`;
  - mouse (gyro/trackpad) aim: `mouse_sensitivity_horiz`, `mouse_sensitivity_mlook`;
  - stick deadzones: `left_analog_deadzone`, `right_analog_deadzone`.
- **DDL keeps your in-game changes.** A DSDA-Doom option you change in its menu is
  preserved on the next launch. DDL only re-applies its baseline when the template
  version changes, and it backs up the old file first (spec 005, decision D-012).

Suggested order: leave DDL's DSDA values at their defaults, get the gyro feeling right in
Steam, then fine-tune the right stick inside DSDA-Doom's menu.

### 6. Recording strict-mode demos? Use a keyboard + mouse layout

While DSDA-Doom records a demo with strict mode on, it refuses to mix real gamepad motion
with mouse motion: whichever it sees first wins and the other is ignored for that
recording. That makes the Modern + Gyro layout unusable for strict-compatible demos.

If you record demos, add a third layout that emulates **keyboard + mouse only** and switch
to it before recording. Map **every** control you use to a keyboard or mouse command, so
DSDA-Doom never sees a virtual gamepad button — that is what keeps you out of the
mixed-input path:

| Deck control | Assign to (keyboard/mouse) | DSDA-Doom binding |
| --- | --- | --- |
| Right trigger | Left mouse click | `input_fire` (bound to left mouse by default) |
| Left stick | `W` / `A` / `S` / `D` | `input_forward`, `input_strafeleft`, `input_backward`, `input_straferight` |
| Right trackpad and gyro | Mouse movement (**As Mouse** / **Gyro To Mouse**) | mouse aim |
| A | Space | `input_use` |
| Left trigger | Tab | `input_map` |
| L3 | Caps Lock | `input_autorun` |
| Left trackpad | Touch Menu with `1`–`9` and `0` | `input_weapon1` … `input_weapon9`; `input_toggleweapon` |
| D-pad | Arrow keys | `input_menu_up` … `input_menu_right` |
| Start / View | Escape / Backspace | `input_menu_escape` / `input_menu_backspace` |

The left-trackpad Touch Menu is optional; it just gives you weapon switching without
attaching a keyboard. Do not turn strict mode off to work around this, and remember that
DDL does not add flags to force recording to accept hybrid input.

### 7. Not yet verified on hardware

The layouts above follow DSDA-Doom v0.30.1's documented defaults and DDL's design plan,
but these points have **not** been confirmed on a physical Deck yet:

- whether a Steam Input layout assigned to the non-Steam DDL shortcut stays active while
  DDL starts DSDA-Doom and after focus returns to DDL (child-process handoff);
- whether gyro-as-mouse activated by right-stick touch behaves as expected in current
  Gaming Mode, and does not disturb DDL's UI;
- whether strict demo recording really does block the mixed input exactly as described;
- comfortable sensitivity and deadzone values — DDL ships DSDA-Doom's upstream defaults,
  which are a starting point, not tuned Deck values.

These are tracked as checks **I1–I4** in [`docs/deck-testing.md`](docs/deck-testing.md);
the results will be folded back into this section once they run.

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

`cargo test` never opens a window: the placeholder screen is verified headlessly with
Slint's software renderer, and the frames it draws at 1280×800 are written to
`target/headless/*.png` (see `crates/ddl/src/headless.rs`). To inspect them somewhere
else, set `DDL_HEADLESS_DIR`.

Release binaries are not built on a development machine: they are produced by
GitHub Actions inside a pinned Steam Linux Runtime SDK container, so that they run
against the libraries SteamOS actually provides. See
[`docs/research/steam-runtime-build.md`](docs/research/steam-runtime-build.md).

### Command-line options

```text
Usage: ddl [OPTIONS]
       ddl catalog lint [--fetch] [PATHS...]
       ddl engine status
       ddl engine install [--version <V>] [--force-extract]
       ddl iwad scan [PATH...]
       ddl mapset list
       ddl mapset install <id> [--with <component>...]
       ddl launch <id> [--dry-run] [--with <component>...] [--skill <1-5>] [--pistol-start]
       ddl doctor

Options:
      --windowed  Run in a 1280x800 window instead of fullscreen
  -h, --help      Print this help and exit
  -V, --version   Print version information and exit

Environment:
  DDL_LOG  Log level for stderr diagnostics: error, warn, info, debug or trace
           (default: info)
```

## Troubleshooting from Konsole

The same binary that opens the launcher also has headless subcommands, so you can
diagnose and repair DDL from a terminal on the Deck without starting the UI. Every
subcommand uses the same files the UI uses, so what you do here shows up in the app
(and vice versa). Only one DDL process may change DDL's state at a time: if the UI
is open and busy, a subcommand refuses with a message telling you to close it.

Start with the one report that answers most questions:

```sh
./ddl doctor
```

It prints DDL's config/data/cache/state paths, the log location, the installed
DSDA-Doom versions and the active one, the identified IWADs, every mapset and
whether it is installed, and the DSDA config state. Paste it when asking for help.

| Subcommand | What it does |
| --- | --- |
| `ddl doctor` | One paste-friendly report of paths, engine, IWADs, mapsets and DSDA config. |
| `ddl engine status` | Installed DSDA-Doom versions, the active one and how it runs (`appimage`, `extracted` or `manual`). |
| `ddl engine install [--version <V>] [--force-extract]` | Download, verify and smoke-test a reviewed DSDA-Doom release. `--force-extract` takes the extraction fallback on purpose. |
| `ddl iwad scan [PATH...]` | Find and identify IWADs in your Steam libraries and in each `PATH`; ambiguous files are listed, never auto-chosen. |
| `ddl mapset list` | List every catalog mapset and whether it is installed. |
| `ddl mapset install <id> [--with <component>...]` | Install a mapset, optionally enabling a component (e.g. `--with compat`). |
| `ddl launch <id> --dry-run` | Print exactly what DDL would run (argv and working directory), or why it cannot. |
| `ddl launch <id>` | Start DSDA-Doom, exactly as the UI's **Play** does. |

Common fixes:

- **"is missing; add a folder containing your own copy"** — DDL found no usable
  IWAD. Run `ddl iwad scan /path/to/your/wads` (or add the folder in the UI).
  DDL never downloads IWADs.
- **`ddl engine status` shows nothing** — install the engine with `ddl engine
  install`, or use **Install DSDA-Doom** on the first-run screen.
- **A mapset will not play** — `ddl launch <id> --dry-run` prints the blocker
  (missing IWAD, not installed, a component you must import, …).
- **An install failed** — the logs are in `~/.local/state/ddl/logs/ddl.log`
  (the exact path is in `ddl doctor`); attach that file.

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

# Steam Deck guide

DDL is pre-alpha: the repository implements installing DSDA-Doom, identifying your own
IWADs, browsing and installing mapsets, and launching through Play setup. Release features
may lag the source tree. **Needs Deck Validation:** Gaming Mode, controller handoff and
real gameplay; see the [hardware checklist](deck-testing.md).

- [Install](#install-from-a-github-release)
- [Steam artwork](#make-it-look-native-in-steam)
- [Flatpak (Discover)](#flatpak-discover)
- [Controller layouts](#recommended-steam-deck-controller-setup)
- [Build from source](#build-from-source)
- [Konsole troubleshooting](#troubleshooting-from-konsole)
- [FAQ](#faq)

**Confirmed (source inspection):** the controller bindings below are the
DSDA-Doom settings DDL itself provisions (baseline template v4, specs 034 and
056). They change nine bindings from DSDA-Doom [v0.30.1 defaults](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c);
everything the table does not name stays at DSDA's own default. Template v4 also
turns **VSync** on with uncapped/interpolated rendering kept and no FPS cap, and
presets DSDA's mouse **Horizontal Sensitivity** and **Vertical Sensitivity** to
**5**. The [engine research](research/dsda-engine.md) §4 and §10 record the pinned
upstream evidence — button codes, the three profile segments, untouched defaults
and how the frame-pacing keys interact. Steam Input itself is still manual; the
[runtime research](research/steam-runtime-build.md) records the Compatibility tool
/ glibc constraint. The layout is **Proposed** until the deck test confirms it;
physical Deck results remain **Needs Deck Validation**.

## Install (from a GitHub Release)

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
4. Rename the shortcut to **DoomDeckLauncher**: right-click it → **Properties** and replace
   the name `ddl`. Artwork tools such as the SteamGridDB Decky plugin search by this name, and
   the [SteamGridDB page](https://www.steamgriddb.com/game/5558703) is titled DoomDeckLauncher.
5. DDL now appears in your library. Launch it once from Desktop Mode to check it
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

Set up your controls as described in
[Recommended Steam Deck Controller Setup](#recommended-steam-deck-controller-setup):
Steam Input is configured by hand in Gaming Mode, and DDL never edits your controller
files.

## Make it look native in Steam

The release includes original Rose Compass (Stained Glass) artwork, set in a vector Ember Cathedral interior, in `ddl/art/`. In Desktop Mode,
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

The same artwork is on [SteamGridDB](https://www.steamgriddb.com/game/5558703) under
**DoomDeckLauncher**, so a SteamGridDB tool can apply it for you once the shortcut carries that
name. Steam's default logo position (bottom left) suits the hero.

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

Repository owners can upload [social-preview.png](design/logo/social-preview.png)
(1280×640) in GitHub **Settings → General → Social preview**. See the
[logo usage and export guide](design/logo/README.md) for SVG masters.

## Flatpak (Discover)

> **This section applies once DDL is on Flathub.** Until then, install from the GitHub
> tarball above. Everything here is **Proposed** until the first Flatpak build has run on a
> Deck (**Needs Deck Validation**); the facts it relies on are in the
> [Flatpak research](research/flatpak.md).

**Install.** In Desktop Mode open **Discover**, search for **DoomDeckLauncher** and install it.
Updates to DDL itself come through Discover (Flathub); DDL has no self-update of its own.
DSDA-Doom engine updates and the mapset catalog still update inside DDL (Settings → Engine).

**Add it to Steam.** **Confirmed (source inspection):** Discover has no "Add to Steam" action, so
this stays manual. In Desktop Mode open Steam → **Games → Add a Non-Steam Game to My Library…**
and then either

- pick **DoomDeckLauncher** if Steam lists it (Flatpak exports a launcher for it), or
- **Browse…** to `/usr/bin/flatpak` and set the launch options to
  `run io.github.vittau.Doom-Deck-Launcher`.

**Needs Deck Validation:** which of the two the current Steam client offers. Rename the shortcut
to **DoomDeckLauncher** so artwork tools find it.

**Compatibility tool: none.** Leave it unset, as for the tarball. The Flatpak brings its own
runtime, and a Steam Linux Runtime wrapped around `/usr/bin/flatpak` would run the Flatpak tool
inside an older container.

**Artwork.** Discover supplies the window icon and store page, not Steam's library capsule.
Apply the artwork by hand as in [Make it look native in Steam](#make-it-look-native-in-steam),
taking the images from the release tarball's `art/` folder or from
[SteamGridDB](https://www.steamgriddb.com/game/5558703).

**No Steam overlay.** **Confirmed (code):** Steam injects its overlay with `LD_PRELOAD`, which
Flatpak clears, so the Steam overlay (FPS counter, Shift+Tab, screenshot hotkey) does not attach
to DDL or to the games it starts. DDL's own screenshots are unaffected, and Steam Input layouts
work as described below. **Needs Deck Validation:** the on-screen keyboard and controller layout
picker in Gaming Mode without the overlay. DDL asks for Steam's keyboard through `xdg-open`
when sandboxed; if it does not appear, the **Steam + X** hint on screen is the fallback.

**Where your data lives.** Inside `~/.var/app/io.github.vittau.Doom-Deck-Launcher/`:

| What | Folder |
| --- | --- |
| Settings, profiles, DSDA config | `config/ddl` |
| Engine, installed mapsets, saves | `data/ddl` |
| Downloads and caches | `cache/ddl` |
| Logs | `.local/state/ddl/logs` |

The engine is unpacked into `data/ddl/engines` once, and DDL runs it from there; the AppImage
is never mounted inside the Flatpak.

**Your IWADs.** DDL looks in the same places as the tarball: Steam (including SD-card libraries
and the Steam Flatpak), GOG and Heroic (including its Flatpak) and the folders you add. A
Flatpak only sees what it has been given access to. When a folder that a Steam or Heroic record
(or your own folder list) names cannot be opened, Settings → IWAD folders and the IWAD review
say so in one line, for example `/run/media/deck/SD/SteamLibrary is not visible to the Flatpak
— see the guide`. Grant read access to that location (for example with Flatseal, or
`flatpak override --user --filesystem=<path>:ro io.github.vittau.Doom-Deck-Launcher`), then
use **Rescan**. **Needs Deck Validation:** which locations the first Flathub build can read.

**Importing a tarball setup.** If you used the tarball before, the first start of the Flatpak
(with no data of its own yet) offers **Import your existing setup?** It *copies* your settings,
profiles, installed mapsets and saves from `~/.config/ddl` and `~/.local/share/ddl`; it never
moves or changes the originals, and your IWADs stay where they are. A DSDA-Doom engine that the
tarball ran as an AppImage is not copied: DDL installs it again for the Flatpak. **No, don't ask
again** is remembered (B closes the question until the next start). To be asked again, delete
`~/.var/app/io.github.vittau.Doom-Deck-Launcher/.local/state/ddl/legacy-import.json` while DDL
still has no data of its own.

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

In **Edit Layout**, pick a control from the left-hand list and assign the command
from the table. The right-hand column is what DSDA-Doom does with that input
under DDL's baseline: **you do not need to rebind any buttons inside
DSDA-Doom.**

| Deck control | Steam Input assignment | What DSDA-Doom does with it (DDL's baseline) |
| --- | --- | --- |
| Left stick | Left joystick (leave as-is) | Move: `input_forward`, `input_backward`, `input_strafeleft`, `input_straferight` |
| Right stick | Right joystick (leave as-is) | Turn / look |
| Right trigger | Right trigger | **Fire** — `input_fire` (DSDA default) |
| A | A | Use / open doors — `input_use` (DSDA default) |
| B | B | **Toggle run (autorun)** — `input_autorun`; jump still keeps its DSDA default on B, so on maps that allow jumping B does both |
| X | X | **Quick 180° turn** — `input_reverse` (disabled while a strict-mode demo records, see §6) |
| Y | Y | Nothing: previous/next weapon moved to the D-pad |
| Left bumper (L1) | Left bumper | **Quick save** — `input_quicksave` |
| Right bumper (R1) | Right bumper | **Quick load** — `input_quickload` |
| L2 (left trigger) | Left trigger | Nothing: the automap moved to View |
| View (Select) | Back | **Toggle the automap** — `input_map` |
| D-pad Up / Down | D-pad | **Automap zoom in / out** while the map is open — `input_map_zoomin` / `input_map_zoomout`. Outside the map these buttons keep their menu (and DOOM fly/Hexen) defaults, because DSDA lets one button carry several actions |
| D-pad Left / Right | D-pad | **Previous / next weapon** — `input_prevweapon` / `input_nextweapon`. They still navigate menus, and they also switch weapons while the automap is open |
| L3 / R3 (stick clicks) | Stick clicks | Nothing: autorun moved to B and the 180° turn to X |
| Start | Start | Open/close the game menu — `input_menu_escape` (DSDA default) |
| A / B in menus | A / B | Confirm — `input_menu_enter`; go back — `input_menu_backspace` |
| Right trackpad | **As Mouse** | Aiming and menu clicking |

DSDA-Doom's own defaults put the automap on L2, weapons on X/Y, autorun on L3
and the 180° turn on R3, and have no quick save/load buttons. DDL changes only
those nine bindings and keeps every keyboard binding, so keyboard play is
unchanged. If you change a binding yourself in DSDA-Doom's own **Controls**
menu, DDL keeps your version: it only re-applies a baseline when its template
version changes, and it backs the old file up first (spec 005).

### 3. Suggested layout B — Modern + Gyro

Start from Classic, then add gyro and set the trackpad. Still controller-only:

1. In **Edit Layout**, choose **Gyro** from the list of controls.
2. Set the gyro **Behavior** to **Gyro To Mouse**. (The current name; older Steam
   clients call the same mode "As Mouse".)
3. In the same gyro screen, set these recommended values:
   - **Gyro Angles to Mouse Pixels: 25000** — how many mouse pixels Steam sends
     for a full 360° turn of the Deck; higher is faster. A good starting point
     for DOOM with DDL's 5/5 mouse sensitivity.
   - **Gyro Speed Deadzone: 0** — no minimum rotation speed, so the smallest
     movement produces aim.
   - **3DOF to 2D Conversion Style: Yaw** — turning the Deck left/right aims
     horizontally and tilting it like a steering wheel is ignored.

   The first and third names were checked against Valve's
   [10/25/23 Gyro To Mouse notes](https://steamcommunity.com/games/1675200/announcements/detail/3714966246894291122)
   and current community guides; Steam relabels this screen from time to time.
   **Needs Deck Validation:** the exact current labels and whether 25000 px feels
   right on the Deck's panel. Match by meaning if a name differs.
4. Open the gyro activation settings — Steam labels these something like *Choose Gyro
   Button(s)* / *Gyro Enable Button* — and set **Right Stick Touch**. Gyro then only
   turns on while your thumb is on the right stick.
5. Go back, choose **Trackpads → Right Trackpad**, and set its **Behavior** to
   **As Mouse**.
6. Leave the right stick as a plain joystick.

DDL already presets DSDA's mouse **Horizontal Sensitivity** and **Vertical
Sensitivity** to **5** (template v4), which is the in-game side of this gyro aim;
you do not need to change it in DSDA's menu.

The result:

| Deck control | Steam Input assignment | Why |
| --- | --- | --- |
| Left stick | Left joystick (unchanged) | Movement |
| Right stick | Right joystick (unchanged) | Fast, big turns |
| Gyro | **Gyro To Mouse**, activated by **Right Stick Touch** | Fine aim while your thumb is on the stick |
| Right trackpad | **As Mouse** | Precision aim and menus |
| Rear buttons (L4/R4, optional) | Gyro button, set to toggle on/off | Only if you prefer a manual on/off over touch activation |
| Triggers, face buttons, bumpers | Same as Classic | Fire, use, weapons, map, autorun and the 180° turn |

### 4. Gamepad and mouse at the same time — avoid double input

- Steam Input can emit **two kinds of events at once**: a virtual controller (Xbox
  style) and keyboard/mouse events. A gyro-as-mouse layout uses both.
- DDL's DSDA-Doom baseline deliberately enables both sources:
  `use_game_controller 1` **and** `use_mouse 1`. The pad drives movement, turning and
  buttons; the mouse path drives gyro and the right trackpad. That coexistence is the
  whole point of the Modern + Gyro layout.
- **Bind one action once.** DSDA-Doom's baseline already binds the right trigger,
  A, B, X, the bumpers and the D-pad, so do not *also* send a mouse click or a
  keyboard key from the same Deck control in Steam. If you want to change
  something, change it in Steam and leave DSDA-Doom at its default, or change it
  in DSDA-Doom's own menu — never both.
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
  - mouse (gyro/trackpad) aim: `mouse_sensitivity_horiz` (menu **Horizontal
    Sensitivity**, preset **5**), `mouse_sensitivity_vert` (**Vertical
    Sensitivity**, preset **5**) and `mouse_sensitivity_mlook` (**Free Look
    Sensitivity**, left at its default **10**);
  - stick deadzones: `left_analog_deadzone`, `right_analog_deadzone`.
- **DDL keeps your in-game changes.** A DSDA-Doom option you change in its menu is
  preserved on the next launch. DDL only re-applies its baseline when the template
  version changes, and it backs up the old file first (spec 005, decision D-012).

Suggested order: leave DDL's DSDA values at their defaults (the 5/5 mouse preset is
the recommended gyro starting point), get the gyro feeling right in Steam, then
fine-tune the right stick inside DSDA-Doom's menu.

> If you enable **Free Look** in DSDA-Doom's own menu, vertical mouse/gyro aim uses
> **Free Look Sensitivity** instead of **Vertical Sensitivity**. DDL leaves it at
> 10; lower it in DSDA's menu if vertical aim feels too fast.

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
- whether the gyro preset above (Gyro To Mouse, 25000 px, speed deadzone 0, Yaw)
  feels right, and whether Steam's current labels still match;
- whether strict demo recording really does block the mixed input exactly as described;
- whether VSync with uncapped FPS removes tearing and lowers power draw without
  hurting smoothness (spec 056);
- comfortable sensitivity and deadzone values — DDL now ships the owner's 5/5 mouse
  look preset, but the analog stick values remain DSDA-Doom's upstream defaults,
  which are a starting point, not tuned Deck values.

These are tracked as checks **I1–I4** in [`docs/deck-testing.md`](deck-testing.md);
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

`cargo test` never opens a window: the launcher screens are verified headlessly with
Slint's software renderer, and the frames it draws at 1280×800 are written to
`target/headless/<test>/` (see `crates/ddl/src/headless.rs`). To inspect them
somewhere else, set `DDL_HEADLESS_DIR`.

### Running UI tests

Slint allows one render platform per process, so each UI test re-runs only its own
case in a headless subprocess, with its own scratch XDG root and its own artifact
directory under `DDL_HEADLESS_DIR` (default `target/headless/<test>/`). Run one
screen or flow by name:

```sh
cargo test -p ddl first_run_and_engine_setup      # one journey test
cargo test -p ddl headless::journeys              # every production flow, in parallel
cargo test -p ddl shell_navigation_and_focus_render_headlessly
```

The PNGs land in `target/headless/<test>/`; the title-fit report is
`<test>/025-fit.txt`. Keep the shared directory clean between runs if you inspect
it in a contact sheet.

Regenerate the README screenshots (lossless recompression into `docs/screenshots/`);
`--check` verifies them without writing:

```sh
python3 scripts/readme-screenshots.py
python3 scripts/readme-screenshots.py --check
```

Release binaries are not built on a development machine: they are produced by
GitHub Actions inside a pinned Steam Linux Runtime SDK container, so that they run
against the libraries SteamOS actually provides. See
[`docs/research/steam-runtime-build.md`](research/steam-runtime-build.md).

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
- **The DDL window looks too big or too small** — you do not need a Steam launch
  option for this: DDL fits its 1280×800 design to the window's physical size, so
  Gaming Mode shows it at 1:1. If you want to override that on purpose, set
  `SLINT_SCALE_FACTOR` (for example `SLINT_SCALE_FACTOR=1 %command%` as a launch
  option); a user value always wins and is logged in `ddl.log`.

## FAQ

**Do I need to own DOOM?** Yes. Supply a compatible IWAD from your own copy; DDL never
bundles or downloads IWADs. Use **Add IWAD folder** if discovery does not find yours.

**Can I use Proton or force a Steam Linux Runtime?** Leave the Compatibility tool unset;
see [install](#install-from-a-github-release) for the confirmed glibc failure.

**Can I use another engine?** DDL targets the official, unmodified DSDA-Doom AppImage only.

**Will my settings survive?** DDL preserves DSDA menu changes, backs up the config before
baseline upgrades, and keeps its config separate from a standalone DSDA installation.

**Where are the logs?** Normally `~/.local/state/ddl/logs/ddl.log`; XDG overrides can
change that path. `./ddl doctor` prints the exact paths. `DDL_LOG=debug ./ddl doctor`
adds stderr detail. Review reports for personal paths before sharing them.

**Are the screenshots actual gameplay?** They are off-screen renders of the real launcher
UI with deterministic sample state. The images on the mapset cards are real in-game
screenshots, credited on each card; their sources are recorded next to them in
`crates/ddl/ui/assets/screenshots/<mapset>/SOURCES.toml`. The frame and decoration are original
art (see the [asset provenance](design/visual-report.md) and [Rose Compass notes](design/logo/README.md)).
Synthetic test IWADs and installs demonstrate the UI and do not prove game compatibility.

## Maintainer note: regenerate README screenshots

From the repository root, run:

```sh
python3 scripts/readme-screenshots.py
python3 scripts/readme-screenshots.py --check
python3 scripts/check-readme-links.py
```

The command runs the existing Slint software-renderer test, using scratch user state,
loopback fixtures and a fake engine, at 1280×800. It never opens a window or captures the
screen, never downloads content, and exports Home, Discover (category strip, tags and a
favorite), Play setup and Mapset detail. PNG compression uses Python's standard library;
no image editor or new dependency is needed. `--check` renders again and compares the
committed bytes. Use the pinned Rust toolchain and bundled fonts for reproducibility.

See [logo v2 concepts](design/logo-v2/README.md) for the separate vector concept exporter.
The owner selected Rose Compass (D-023) and then its Stained Glass evolution (D-029, `design/logo-v3/`); it is used in the app, Steam artwork and README. The D-023 production files are kept as history in `design/logo-v2/production-d023/`.

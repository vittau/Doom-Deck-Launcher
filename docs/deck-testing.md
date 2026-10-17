# Steam Deck test checklist

Manual checks run by the owner on a physical **Steam Deck LCD**, using builds downloaded from
GitHub Releases. Each check names the release it was run on and its result. Hardware facts
learned here flow back into `research/` and `decisions.md`.

**How to install a build**
1. Desktop Mode → download `ddl-linux-x86_64.tar.gz` (and `.sha256`) from the release page.
2. Optional: `sha256sum -c ddl-linux-x86_64.tar.gz.sha256` in Konsole.
3. Extract to e.g. `~/Applications/ddl/`.
4. Steam → *Add a Non-Steam Game* → browse to `ddl/ddl` → add.
5. Return to Gaming Mode and launch **ddl** from the library.

> **Do not set a Compatibility tool on the DDL shortcut.** DDL and the DSDA-Doom
> AppImage it launches run on the host system. Forcing a Steam Linux Runtime makes
> the engine run on an older glibc and it fails to start with `GLIBC_2.34 not found`
> (confirmed in CI). Leave **Compatibility → Force the use of a specific Steam
> Play compatibility tool** off.

Result legend: ✅ pass · ❌ fail (add notes) · ⚠️ partial · ⏳ not run yet

## Release 0.0.x — skeleton (spec 001)

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| S1 | `./ddl --version` in Konsole prints `ddl <version>` | Desktop | ⏳ | |
| S2 | `./ddl --windowed` opens a 1280×800 window with the menu | Desktop | ⏳ | |
| S3 | Launched from Steam, DDL opens fullscreen, text is readable at arm's length | Gaming | ⏳ | |
| S4 | D-pad / stick moves the focus highlight; A activates; B or Quit exits back to Steam | Gaming | ⏳ | keyboard-only build: note what the controller does |
| S5 | No crash on exit; Steam returns to the library | Gaming | ⏳ | |
| S6 | Output of `ldd ddl` — any "not found" libraries? | Desktop | ⏳ | paste missing ones |

## Release 0.0.2+ — first playable build (spec 010); 0.0.3 adds Ember UI, logo, Eviternity

Visuals are the interim look; the DOOM-flavoured identity (spec 012, D-020) comes later. Run in
this order (≈30 min); then the E, C, M and I tables below. S1–S6 apply too: `--windowed` and
the controller now work (gilrs).

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| F1 | First run: **Install DSDA-Doom** downloads, shows progress, finishes; cancel mid-way then retry works | Gaming | ⏳ | covers E1/E2 |
| F2 | IWADs: Steam DOOM + DOOM II found automatically, or **Add IWAD folder** browser reaches `/run/media/*` (SD card) with D-pad only | Gaming | ⏳ | covers C3; note paths |
| F3 | Discover → SIGIL → **Install**: download + extraction with progress; then SIGIL II and AUGER;ZENITH | Gaming | ⏳ | note any error text |
| F4 | Play setup shows base game found, numbered load order, complevel; **Play** starts DSDA fullscreen | Gaming | ⏳ | covers E4 |
| F5 | Quit DSDA (menu → Quit): DDL comes back with focus on Play, no stray input | Gaming | ⏳ | covers E4/I1 |
| F6 | AUGER;ZENITH add-on choice (None / Alternate / Deckard) persists after relaunch of DDL | Gaming | ⏳ | |
| F7 | Missing IWAD (e.g. no Ultimate DOOM): Play disabled with the reason and **Add IWAD folder** | Gaming | ⏳ | |
| F9 | **0.0.3:** Ember Cathedral look is readable at arm's length; screens render without lag (S3) | Gaming | ⏳ | spec 012 A4 |
| F10 | **0.0.3:** Eviternity installs (≈50 MB) and plays at complevel 11 (MAP01, music, DEHACKED changes active) | Gaming | ⏳ | complevel Proposed |
| F11 | **0.0.3:** Steam artwork from `ddl/art/` set by hand per the [Steam Deck guide](steam-deck-guide.md#make-it-look-native-in-steam) looks right in the library | Gaming | ⏳ | spec 013 A4 |
| F8 | Logs: `~/.local/state/ddl/logs/ddl.log` exists; attach it to any ❌ | Desktop | ⏳ | path shown in Settings → Diagnostics |

## Engine (specs 003, 004)

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| E1 | DDL installs DSDA-Doom v0.30.1; `-v` smoke test passes | Gaming | ⏳ | |
| E2 | Official AppImage runs directly (FUSE available?) | Gaming | ⏳ | |
| E3 | Extraction fallback runs on purpose: in Konsole `./ddl engine install --force-extract` finishes and `./ddl engine status` shows `extracted`; a launch from that install works | Desktop | ⏳ | covers the D-013 fallback (spec 014 R1); `--force-extract` takes the fallback even when the AppImage would run |
| E4 | A launch from DDL starts DSDA fullscreen; quitting DSDA returns focus to DDL | Gaming | ⏳ | |
| E5 | Controller works in DSDA with the DDL config (`use_game_controller 1`) | Gaming | ⏳ | |

## Config and profiles (specs 005, 007)

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| C1 | Change a DSDA menu option, quit, relaunch from DDL: it persists | Gaming | ⏳ | |
| C2 | A standalone DSDA install's config (`~/.dsda-doom/` or `~/.local/share/dsda-doom/`) is untouched | Desktop | ⏳ | |
| C3 | DDL finds the DOOM + DOOM II Steam install and identifies its IWADs (`base/` and `rerelease/`) | Gaming | ⏳ | |
| C4 | Saves for two profiles stay separate | Gaming | ⏳ | Spec 018: create two runs, change skill/components, save and screenshot in each; switch and Continue after restart; verify isolation and named delete confirmation (Needs Deck Validation) |

## GOG and other stores (spec 021)

Everything here is **Needs Deck Validation**: the paths come from the launchers' own upstream
source (`docs/research/iwad-identification.md` §4.1), but only a real Deck confirms the exact
layout.

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| G1 | A GOG *DOOM + DOOM II* installed through **Heroic** is found automatically; the IWAD list names it "GOG via Heroic" (spec 021 A5) | Gaming | ⏳ | Covers R2/R3; note the install path |
| G2 | A GOG game installed with the **offline installer** under `~/GOG Games/…` is found | Gaming | ⏳ | default folder per GOG support |
| G3 | A game installed with **minigalaxy** (`install_dir` in `~/.config/minigalaxy/config.json`) is found | Gaming | ⏳ | |
| G4 | **Search standard Linux folders** is off by default; turning it on in Settings → IWAD folders finds a WAD placed in `~/.local/share/games/doom` (or `$DOOMWADDIR`) after a Rescan | Gaming | ⏳ | covers R4 |
| G5 | `./ddl iwad scan` prints each file's source label ("Steam", "GOG via Heroic", …) | Desktop | ⏳ | covers R3 |
| G6 | **Flatpak Heroic** (`~/.var/app/com.heroicgameslauncher.hgl/config/heroic/gog_store/installed.json`) is read; also note whether Flatpak paths are reachable from DDL's sandbox-less install | Gaming | ⏳ | |
| G7 | A GOG game installed through **Lutris** is *not* auto-found (Lutris uses a SQLite DB); **Add IWAD folder** reaches it | Gaming | ⏳ | open question 10; whether to add SQLite |

## Reference mapsets

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| M1 | SIGIL: episode 5 reachable, complevel 3 plays correctly | Gaming | ⏳ | promotes recipe to verified |
| M2 | SIGIL compat variant replaces E3 | Gaming | ⏳ | |
| M3 | SIGIL II: episode 6 reachable, complevel 3 | Gaming | ⏳ | |
| M4 | AUGER;ZENITH at complevel 2: MAP01–MAP21 behave; no visual errors | Gaming | ⏳ | complevel still Proposed |
| M5 | AUGER;ZENITH with AlternateBlaster add-on: new pistol sound and muzzle flash | Gaming | ⏳ | |
| M6 | Pistol start toggle resets inventory each map | Gaming | ⏳ | |

## Input and Steam Input (manual setup)

Follow the [controller layouts](steam-deck-guide.md#recommended-steam-deck-controller-setup).

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| I1 | DDL UI with the default Steam layout: no double input | Gaming | ⏳ | |
| I2 | Gyro-as-mouse layout: cursor does not disturb DDL's UI (D-009) | Gaming | ⏳ | |
| I3 | Gamepad + gyro-as-mouse together in DSDA, normal play | Gaming | ⏳ | |
| I4 | Strict demo recording blocks mixed input as documented | Gaming | ⏳ | |

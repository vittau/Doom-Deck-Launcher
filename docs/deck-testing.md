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

## Release 0.0.9 — focus clarity, Library fix, Flatpak-ready

Install over your current setup as before. The 0.0.8 checks (U1–U9) still apply, except U6,
which V1 replaces.

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| V1 | Play setup: D-pad down from Advanced lands on **Skill** (left); Right reaches Pistol start; up from Recipe load order lands on Skill | Gaming | ⏳ | spec 060 R1 |
| V2 | The Play button is ember red when not focused and no longer looks selected; every focused item shows the same amber ring | Gaming | ⏳ | spec 060 R2 |
| V3 | A recipe's required component ("cannot be unchecked") looks greyed out with a lock, so it is clear why the D-pad skips it; unavailable buttons look disabled | Gaming | ⏳ | spec 060 R3 |
| V4 | Library opens with "Mapsets and custom setups" at the top and no frame strips under the header; moving focus never leaves a sliver of a card under the header | Gaming | ⏳ | 061 |
| V5 | Library: the D-pad reaches every Base games card and focus stays there (it used to jump back to the first mapset) | Gaming | ⏳ | 061 |
| V6 | Settings → Diagnostics / About shows "Flatpak sandbox: no" for the tarball build | Gaming | ⏳ | spec 059; the Flatpak itself comes later |

## Release 0.0.8 — upgrade fix and controller touch-ups

**Install it over your current setup — do not delete `~/.config/ddl` first**: U1 checks the
upgrade path. The 0.0.7 checks below (R1–R13) still apply.

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| U1 | Replace the DDL folder with 0.0.8 and open it: engine, IWADs, installed mapsets, profiles and favorites are all there; no "Install DSDA-Doom" first-run, no "Could not read saved setup" | Gaming | ⏳ | spec 055; ideally also from a 0.0.6 setup |
| U2 | If a saved file is ever unreadable: the recovery screen names it; Retry says when it still fails; Start fresh keeps the file as `*.unreadable-<time>`; Settings → Diagnostics shows paths | Gaming | ⏳ | spec 055 (only if it happens) |
| U3 | In DSDA: no tearing, motion still smooth, fan/power lower than 0.0.7 on the same scene (VSync on, uncapped interpolation) | Gaming | ⏳ | spec 056; Options → Video → FPS shows Vertical Sync on |
| U4 | DSDA Options → Mouse: Horizontal and Vertical Sensitivity are 5 (unless you changed them before); gyro with the guide's settings (Gyro to Mouse, 25000, deadzone 0, Yaw) feels right | Gaming | ⏳ | spec 056, [guide](steam-deck-guide.md) |
| U5 | Right stick scrolls long screens (error text, Settings pages, mapset detail, dialogs) without moving the selection | Gaming | ⏳ | spec 057 |
| U6 | Play setup: D-pad down from Advanced lands on Skill/Pistol start, then Recipe load order; up mirrors it | Gaming | ⏳ | spec 057 |
| U7 | A new profile starts on "Skill 4 (UV)"; skill 5 reads "(Nightmare)"; your existing profiles keep their skill | Gaming | ⏳ | spec 057, D-032 |
| U8 | Tabs read Home, Library, Discover, Settings | Gaming | ⏳ | spec 057 |
| U9 | B on a tab's top level opens Quit with Quit selected; B closes it; in Settings' right page B first returns to the list | Gaming | ⏳ | spec 057, D-032 |

## Release 0.0.7 — re-checks after the 0.0.6 run

Fixes since 0.0.6 that need the Deck. **Before you start, remove `SLINT_SCALE_FACTOR=1 %command%`
from the shortcut's launch options**: DDL now fits the 1280×800 panel itself (spec 029). Rename
the shortcut to **DoomDeckLauncher** ([guide](steam-deck-guide.md#install-from-a-github-release)).

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| R1 | Without the launch option, DDL opens fullscreen at 1:1: all four tabs fit and text is readable (S3) | Gaming | ⏳ | spec 029 |
| R2 | `./ddl --windowed` opens a 1280×800 window with the menu (S2) | Desktop | ⏳ | |
| R3 | IWADs: the Steam DOOM + DOOM II package finds DOOM, DOOM II, TNT and Plutonia (`Ultimate Doom/base/` and its `doom2/`, `tnt/`, `plutonia/` subfolders) (F2) | Gaming | ⏳ | spec 028 |
| R4 | **Play** starts DSDA fullscreen in widescreen, not 4:3 (F4) | Gaming | ⏳ | spec 029 video keys |
| R5 | Base games (DOOM, DOOM II) appear and launch with complevel `vanilla` | Gaming | ⏳ | spec 032, D-030 |
| R6 | Profiles: create a second profile for a mapset, switch, launch; saves stay separate | Gaming | ⏳ | specs 018, 031 |
| R7 | Controller in DSDA with the new layout (E5 follow-up) | Gaming | ⏳ | spec 034 |
| R8 | Logo v3 (Stained Glass) in header, loading screen, About and Steam icon look right | Gaming | ⏳ | D-029 |
| R9 | Panel frames: corners meet their rails, no gaps or steps; wall background has no seam; title fades into the art band | Gaming | ⏳ | specs 045, 048, 049 |
| R10 | Mapset detail / Play setup show complevel as "Vanilla (2)", "MBF (11)"…, with no "not yet verified" text | Gaming | ⏳ | spec 046 |
| R11 | Install a Dropbox-first DBP (e.g. Haunting Hollow): fast download; cancel mid-way, install again → it resumes ("Resuming…") instead of restarting | Gaming | ⏳ | specs 038, 040 |
| R12 | Steam artwork from `art/` or SteamGridDB (capsule, wide, hero, logo, icon) looks right in Gaming Mode | Gaming | ⏳ | spec 054 |
| R13 | Catalog update toast disappears after the manual check (S2 of the catalog table) | Gaming | ⏳ | spec 030 |

### 0.0.7 run, 2026-10-20 (owner)

- ❌ **Upgrade over 0.0.6:** DDL opened on the first-run "Install DSDA-Doom"; Install showed
  "Install stopped" / "Could not read saved setup"; every Settings panel was empty and Retry did
  nothing. Cause: `app.json` stores recipe copies, and spec 046 removed `complevel_status`.
  Workaround: delete `~/.config/ddl`. Fix: spec 055.
- ✅ After a clean config: DSDA-Doom 0.30.1, Ultimate DOOM and DOOM II found; AUGER;ZENITH
  installed and played.
- Requests: VSync on with uncapped FPS and look sensitivity 5/5 (spec 056); right-stick scroll,
  Play setup D-pad order (Advanced skipped the Skill row), skill default 4 "(UV)", tab order Home,
  Library, Discover, B at root opens Quit (spec 057); Flatpak/Discover research (spec 058).

## Release 0.0.x — skeleton (spec 001)

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| S1 | `./ddl --version` in Konsole prints `ddl <version>` | Desktop | ✅ | 0.0.6: prints `ddl 0.0.6` |
| S2 | `./ddl --windowed` opens a 1280×800 window with the menu | Desktop | ⏳ | |
| S3 | Launched from Steam, DDL opens fullscreen, text is readable at arm's length | Gaming | ❌ | 0.0.6: Gaming Mode renders at ~2× (winit reads the panel DPI); only Home/Discover tabs fit. `SLINT_SCALE_FACTOR=1 %command%` fixes it → spec 029 |
| S4 | D-pad / stick moves the focus highlight; A activates; B or Quit exits back to Steam | Gaming | ✅ | 0.0.6: D-pad/stick/A/B work. Tabs locked on first run look active (spec 030) |
| S5 | No crash on exit; Steam returns to the library | Gaming | ⏳ | |
| S6 | Output of `ldd ddl` — any "not found" libraries? | Desktop | ✅ | 0.0.6: no missing libraries |

## Release 0.0.2+ — first playable build (spec 010); 0.0.3 adds Ember UI, logo, Eviternity

Visuals are the interim look; the DOOM-flavoured identity (spec 012, D-020) comes later. Run in
this order (≈30 min); then the E, C, M and I tables below. S1–S6 apply too: `--windowed` and
the controller now work (gilrs).

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| F1 | First run: **Install DSDA-Doom** downloads, shows progress, finishes; cancel mid-way then retry works | Gaming | ⚠️ | 0.0.6: install finished quickly; cancel not exercised (too fast) |
| F2 | IWADs: Steam DOOM + DOOM II found automatically, or **Add IWAD folder** browser reaches `/run/media/*` (SD card) with D-pad only | Gaming | ❌ | 0.0.6: only `Ultimate Doom/base/DOOM.WAD` found — Steam app 2280 installs to `Ultimate Doom/` with `base/doom2/`, `base/tnt/`, `base/plutonia/`, `rerelease/` → spec 028. No SD card on this Deck |
| F3 | Discover → SIGIL → **Install**: download + extraction with progress; then SIGIL II and AUGER;ZENITH | Gaming | ✅ | 0.0.6: SIGIL installs |
| F4 | Play setup shows base game found, numbered load order, complevel; **Play** starts DSDA fullscreen | Gaming | ⚠️ | 0.0.6: SIGIL plays, but DSDA opens 4:3 — DDL sets no video keys → spec 029 |
| F5 | Quit DSDA (menu → Quit): DDL comes back with focus on Play, no stray input | Gaming | ✅ | 0.0.6: returns with focus on Play |
| F6 | AUGER;ZENITH add-on choice (None / Alternate / Deckard) persists after relaunch of DDL | Gaming | ⏳ | |
| F7 | Missing IWAD (e.g. no Ultimate DOOM): Play disabled with the reason and **Add IWAD folder** | Gaming | ✅ | 0.0.6: AUGER;ZENITH shows "Base game missing" (Doom II not found, see F2) |
| F9 | **0.0.3:** Ember Cathedral look is readable at arm's length; screens render without lag (S3) | Gaming | ⏳ | spec 012 A4 |
| F10 | **0.0.3:** Eviternity installs (≈50 MB) and plays at complevel `mbf` (11) (MAP01, music, DEHACKED changes active) | Gaming | ⏳ | |
| F11 | **0.0.3:** Steam artwork from `ddl/art/` set by hand per the [Steam Deck guide](steam-deck-guide.md#make-it-look-native-in-steam) looks right in the library | Gaming | ⏳ | spec 013 A4 |
| F8 | Logs: `~/.local/state/ddl/logs/ddl.log` exists; attach it to any ❌ | Desktop | ⏳ | path shown in Settings → Diagnostics |

## Engine (specs 003, 004)

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| E1 | DDL installs DSDA-Doom v0.30.1; `-v` smoke test passes | Gaming | ✅ | 0.0.6 |
| E2 | Official AppImage runs directly (FUSE available?) | Gaming | ⏳ | |
| E3 | Extraction fallback runs on purpose: in Konsole `./ddl engine install --force-extract` finishes and `./ddl engine status` shows `extracted`; a launch from that install works | Desktop | ⏳ | covers the D-013 fallback (spec 014 R1); `--force-extract` takes the fallback even when the AppImage would run |
| E4 | A launch from DDL starts DSDA fullscreen; quitting DSDA returns focus to DDL | Gaming | ✅ | 0.0.6 |
| E5 | Controller works in DSDA with the DDL config (`use_game_controller 1`) | Gaming | ✅ | 0.0.6: works; owner wants a different layout later |

## Config and profiles (specs 005, 007)

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| C1 | Change a DSDA menu option, quit, relaunch from DDL: it persists | Gaming | ⏳ | |
| C2 | A standalone DSDA install's config (`~/.dsda-doom/` or `~/.local/share/dsda-doom/`) is untouched | Desktop | ⏳ | |
| C3 | DDL finds the DOOM + DOOM II Steam install and identifies its IWADs (`base/` and `rerelease/`) | Gaming | ⏳ | |
| C4 | Saves for two profiles stay separate | Gaming | ✅ | 0.0.6: saves separate. Profile sheet UX confusing (spec 031) |

## Uninstall (spec 022) — needs Deck validation

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| U1 | Detail → **Uninstall** → confirm with the checkbox **unchecked**: the mapset leaves Library; its profiles and saves survive a restart and reappear on reinstall | Gaming | ✅ | 0.0.6 |
| U2 | Repeat with the checkbox **checked**: profiles and their saves/screenshots are gone after a restart | Gaming | ⏳ | A toggles the checkbox; no text entry |
| U3 | An installed mapset that is no longer in the catalog still shows **Uninstall** on detail and removes cleanly | Gaming | ⏳ | spec 020 R5 |

## Screenshots and catalog updates (specs 023, 019) — needs Deck validation

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| S1 | Discover, Library, Home and Mapset detail show real in-game screenshots with a small "Screenshot: <credit>" caption, sharp on the Deck LCD | Gaming | ✅ | 0.0.6: shown and sharp. Discover header needs a margin above the list; L2/R2 strip needs hold-repeat and wrap (spec 030) |
| S2 | Settings → Engine shows `Catalog: <date> (built-in)`; **Check for catalog updates** succeeds online and reports `updated` or "already current" | Gaming | ⚠️ | 0.0.6: "Catalog is up to date", but the toast never disappears (spec 030) |
| S3 | Airplane mode: start DDL — no error or delay from the automatic catalog check; the manual check shows a clear network error | Gaming | ⏳ | Automatic check is silent offline |

## Local content (spec 026) — needs Deck validation

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| L1 | Library → **New custom setup** → add a PWAD and a DEH from the SD card (file picker, A to select, Start to add), pick the IWAD, reorder with A/D-pad/A, launch | Gaming | ⚠️ | 0.0.6: custom setup with no files + Ultimate Doom plays (used to play the base game; spec 032). PWAD/DEH from SD not run |
| L2 | Installed mapset → Play setup → **Extra files** → add a PWAD on top; it loads after the recipe files; saves warning shows when the profile has saves | Gaming | ⏳ | Recipe files locked at the top |
| L3 | Move or edit an added file in Desktop Mode, then launch: the missing-file / changed-file dialog names the file; "Use the changed file" works | Gaming | ⏳ | A user-file IWAD's campaign follows its file name (D-017 note) |

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
| M1 | SIGIL: episode 5 reachable, complevel 3 plays correctly | Gaming | ⚠️ | 0.0.6: SIGIL plays; episode 5 and complevel not checked explicitly |
| M2 | SIGIL compat variant replaces E3 | Gaming | ⏳ | |
| M3 | SIGIL II: episode 6 reachable, complevel 3 | Gaming | ⏳ | |
| M4 | AUGER;ZENITH at complevel `vanilla` (2): MAP01–MAP21 behave; no visual errors | Gaming | ⏳ | |
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

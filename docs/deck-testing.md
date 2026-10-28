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
| V1 | Play setup: D-pad down from Advanced lands on **Skill** (left); Right reaches Pistol start; up from Recipe load order lands on Skill | Gaming | — | spec 060 R1; superseded by spec 114 (no Skill row) |
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
| U6 | Play setup: D-pad down from Advanced lands on Skill/Pistol start, then Recipe load order; up mirrors it | Gaming | — | spec 057; superseded by spec 114 (no Skill row) |
| U7 | A new profile starts on "Skill 4 (UV)"; skill 5 reads "(Nightmare)"; your existing profiles keep their skill | Gaming | — | spec 057, D-032; superseded by D-044 (skill is chosen in DSDA's menu) |
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
| L1 | Library → **Add local mapset** → add a PWAD and a DEH from the SD card (file picker, A to select, Start to add), pick the IWAD, reorder with A/D-pad/A, launch | Gaming | ⚠️ | 0.0.6: custom setup with no files + Ultimate Doom plays (used to play the base game; spec 032). PWAD/DEH from SD not run |
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

## MIDI soundfonts and purchased SIGIL editions (specs 067–068)

All checks below are **Needs Deck Validation**. Offline tests verify identities, safe install,
selection, configuration and load order; they do not establish audible playback on hardware.

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| A1 | Settings → Audio: download Arachno, select it, launch MIDI music, exit and switch back to Default | Gaming | ⏳ | Download alone must not change the selection; compare audible output after each launch |
| A2 | Restart the signed Flatpak with Arachno selected and launch again | Gaming | ⏳ | Confirm persistent choice and DSDA access to the managed SF2 inside the sandbox |
| A3 | Remove the managed Arachno font between runs, launch, then reinstall through Audio | Desktop / Gaming | ⏳ | Default fallback, visible warning and recoverable Reinstall action; do not modify a running engine |
| M7 | Import purchased SIGIL REG and SIGIL II MP3 WADs; launch native E5/E6 and hear their paid music | Gaming | ⏳ | Complete replacements: do not load the free map WAD alongside the paid WAD |
| M8 | Import SIGIL SHREDS_COMPAT, launch E3, then switch to native SIGIL or REG | Gaming | ⏳ | Required COMPAT maps activate automatically, music follows maps, switching leaves no orphaned music |
| M9 | On an imported component row, toggle On/Off, replace its file with Import, and dismiss with Back | Gaming | ⏳ | Repeat Import after removing its managed copy; original user file remains untouched |

## Current UI and catalog refinements (specs 071–077)

Pending the next prerelease; all physical checks remain **Needs Deck Validation**. Headless
geometry, fake transports and persistence tests complement these checks but do not replace them.

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| R1 | Inspect mapset cards and original base-game cards, both focused and unfocused | Gaming | ⏳ | Art must clear the ornate frame; long titles, readiness text and two tag rows must remain readable |
| R2 | Toggle Pistol start repeatedly for an installed large mapset and an original base game, then restart DDL | Gaming | ⏳ | Immediate visible feedback, final value persists, no lost input or noticeable pause; actual launch still validates files |
| R3 | Open SIGIL setup and import normal paid music, then COMPAT paid music | Gaming | ⏳ | Player-facing choices distinguish normal and COMPAT; exact filenames remain in import help |
| R4 | Open Discover twice within one day online, then once offline | Gaming | ⏳ | Each online start checks in the background; UI/play stay usable during slow network; offline retains catalog without an error |
| R5 | Verify the ordered specific categories, last mapset accessibility and catalog date after an update | Gaming | ⏳ | No All/Featured tab; Doomer Boards follows Community Projects; source date uses English UTC, not device fetch time |
| R6 | Install Memento Mori and Memento Mori II from Classics and launch each | Gaming | ⏳ | Doom II, vanilla, map WAD then original music WAD; confirm MIDI playback and map selection |

## Shared cards/list format (spec 095)

This source change follows `v0.0.12-catalog` and is included in `v0.0.13-update`.
All physical checks remain **Needs Deck Validation**; host/headless tests verify the state,
navigation and layout but do not establish Gaming Mode input or measured Deck performance.

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| CV1 | In Discover, press Select once to switch to List, then change to Library | Gaming | ⏳ | Library uses List too; holding Select must not repeatedly alternate formats |
| CV2 | In Library, switch back to Cards, close DDL normally and reopen; repeat with List | Gaming | ⏳ | Both tabs restore the final choice, including rapid toggles immediately before closing |
| CV3 | Toggle on a non-first item, move through categories and Library base-game/mapset/local sections, open detail and return | Gaming | ⏳ | Same item stays focused and visible; chronology, favorites, filters and available actions remain correct |
| CV4 | Inspect long titles, availability/installed/favorite markers, five complete Discover rows with the category strip visible, and the full footer | Gaming | ⏳ | Readable on the Deck LCD; no overlap or clipped focus; check perceived response while toggling and scrolling |
| CV5 | Try Select in Home, Settings, detail, setup, dialogs and during a running game; return to Discover/Library | Gaming | ⏳ | Format changes only on the two unobstructed browsing roots; the footer names the next format |

## Updated Flatpak and catalog messages (v0.0.13-update)

All checks below remain **Needs Deck Validation**. Publication smoke tests validate the
installed package version and source provenance on Linux, not Discover on a physical Deck.

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| UP1 | Update the signed Flatpak, then compare Discover, `flatpak info`, `flatpak list --columns=version` and `flatpak run --command=ddl com.doomdecklauncher.DoomDeckLauncher --version` | Desktop | ⏳ | All must report `0.0.13`; verify normal upgrade retains profiles, mapsets and settings |
| UP2 | If a future catalog requires a newer DDL, manually check it from Settings | Gaming | ⏳ | The message names the required version and asks for a DDL update; it does not offer a misleading connection retry |

## Continuous browsing and favorite ordering (v0.0.14-browse)

All checks below remain **Needs Deck Validation** until tested on the owner's Deck.

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| CB1 | Hold Down through several screens in Discover and Library List, then reverse direction | Gaming | ⏳ | Scroll moves continuously with neighboring rows retained; selected row settles fully visible; no queued actions or page jumps |
| CB2 | Scroll manually with the right stick/trackpad, release, then navigate again | Gaming | ⏳ | Manual position survives neutral ticks; new focus navigation reveals the selected row |
| CB3 | Favorite/unfavorite installed mapsets and toggle Cards/List | Gaming | ⏳ | Favorites precede other installed mapsets; chronology/title order within groups; selection keeps its mapset identity |
| CB4 | Browse newly added real mapset screenshots in Cards and detail | Gaming | ⏳ | Representative image and credit fit correctly; unresolved searches are listed in report103 |
| CB5 | Update the signed Flatpak and compare Discover, Flatpak list/info and binary version | Desktop | ⏳ | All should report `0.0.14` after publication; installed mapsets/profiles/settings remain intact |

## Input and Steam Input (manual setup)

Follow the [controller layouts](steam-deck-guide.md#recommended-steam-deck-controller-setup).

| # | Check | Mode | Result | Notes |
| --- | --- | --- | --- | --- |
| I1 | DDL UI with the default Steam layout: no double input | Gaming | ⏳ | |
| I2 | Gyro-as-mouse layout: cursor does not disturb DDL's UI (D-009) | Gaming | ⏳ | |
| I3 | Gamepad + gyro-as-mouse together in DSDA, normal play | Gaming | ⏳ | |
| I4 | Strict demo recording blocks mixed input as documented | Gaming | ⏳ | |


## Local-content polish — source after spec 106

**Needs Deck Validation:** these are new source checks, not results for the published app.
Host software-render and synthetic-file evidence is in [report106](reports/106-local-content-polish.md).
Use legally owned content already on the Deck; do not download IWADs for this checklist.

| Check | Journey | Mode | Result |
| --- | --- | --- | --- |
| LC1 | Library Y / Add local mapset explains base game → Create setup → Add files; first content filename names the new setup, Rename confirms/cancels without changing profiles or saves | Gaming | ✅ 0.0.16, 2026-10-23: owner ran it with a WAD already on the Deck |
| LC2 | Browse a large WAD folder on internal storage and SD; compare listing-ready feel, cancel while browsing and while confirming a large selection, verify no partial add | Gaming | ⏳ |
| LC3 | Browse an SD folder, close/reopen picker and restart DDL; folder is remembered; unmount card and reopen → Home without repeated retry | Gaming | ⚠️ 0.0.16, 2026-10-23: internal-storage folder remembered after reopening the picker and restarting DDL; SD and unmount steps N/A (no SD card on this Deck) |
| LC4 | Move a middle local file, use Locate file and select the same bytes; original slot, compatibility, profile and saves remain; restart and launch | Gaming | ⏳ |
| LC5 | Locate different bytes, first cancel then explicitly accept; Cancel starts focused and saves warning is readable; wrong patch/content kind or non-IWAD in IWAD slot fails safely | Gaming | ⏳ |
| LC6 | Locate a custom file-based IWAD, then catalog/base-game profile extras; only the intended profile changes; registered copies/recipe members retain their existing recovery | Gaming | ⏳ |
| LC7 | Check Rename text input/Steam keyboard, D-pad/footer/Back behavior and the Library/detail/missing/picker/confirmation layouts at LCD scale; verify Flatpak folder visibility | Gaming | ⚠️ 0.0.16, 2026-10-23: Rename/Steam keyboard, D-pad/Back and folder visibility work for the screens reached in LC1; Locate/missing/confirmation layouts not reached |


**0.0.16 quick pass (2026-10-23, owner, Gaming Mode):** update → owned games found →
install a small catalog mapset → play → return: ✅ (data kept, input and Flatpak audio OK).
LC2 and LC4–LC6 need a folder of the owner's own WADs and were deferred.

## Recipe variant clarity — source after spec 107

**Needs Deck Validation:** see [report107](reports/107-catalog-variant-clarity.md) for focused
host evidence. These rows do not record a new physical test or a release result.

| Check | Journey | Mode | Result |
| --- | --- | --- | --- |
| RV1 | On a mapset without alternatives (e.g. Going Down), Show recipe variants stays visible and greyed out with No recipe variants available; D-pad skips it and clicking does nothing | Gaming | ✅ 0.0.16, 2026-10-23: Going Down row greyed out |
| RV2 | SIGIL → Show recipe variants immediately focuses/reveals COMPAT; selecting it asks for confirmation with Keep current settings focused; cancel preserves choices, explicit accept changes the variant, Hide preserves the selected components | Gaming | ✅ 0.0.16, 2026-10-23 |
| RV3 | Ready setup shows resolved Complevel plus Set by recipe; blocked setup explains recipe control; custom setup Compatibility still offers Auto/numeric choices | Gaming | ⚠️ 0.0.16, 2026-10-23: catalog Complevel + Set by recipe correct; blocked and custom-setup parts not run |


## Pinned Discover categories — source after spec 111

**Needs Deck Validation:** see [report111](reports/111-pinned-discover-categories.md) for the
focused tests and off-screen renders. Published in `v0.0.17-discover`.

| Check | Journey | Mode | Result |
| --- | --- | --- | --- |
| PD1 | Discover Cards and List: scroll deep with the D-pad and the stick; the category strip never moves and no card/row draws over it; the focused item is always fully visible | Gaming | ✅ 0.0.17, 2026-10-23 |
| PD2 | Up from the first row reaches the strip without a jump; L2/R2 and choosing a chip return the list to the top; touch on a chip and on a half-hidden card hits the right target | Gaming | ✅ 0.0.17, 2026-10-23 |
| PD3 | 340px cards: art, title, author and the one-row tags (14px) are readable at arm's length on the LCD | Gaming | ✅ 0.0.17, 2026-10-23 |


## Discover order and Favorites focus — source after specs 112–113

Owner-validated on the Deck with 0.0.18 (2026-10-26); see [report112](reports/112-discover-installed-favorites-first.md) and
[report113](reports/113-favorites-focus-after-unfavorite.md). Published in `v0.0.18-favorites`.

| Check | Journey | Mode | Result |
| --- | --- | --- | --- |
| DF1 | In a Discover category with installed and favorite mapsets: installed favorites first, then installed, then favorite-only, then the rest (Cards and List); X on a card moves it to its new group with focus kept | Gaming | ✅ 0.0.18, 2026-10-26 |
| DF2 | Favorites category: X on a middle, the last and the only card leaves focus on a visible card (or the Favorites chip with the empty panel); same after unfavoriting on the mapset page and pressing B | Gaming | ✅ 0.0.18, 2026-10-26 |

## Games open at DSDA-Doom's menu — source after spec 114

**Needs Deck Validation:** see [report114](reports/114-launch-at-dsda-menu.md) (D-044). DDL no
longer has a skill setting and passes neither `-skill` nor `-warp`.

| Check | Journey | Mode | Result |
| --- | --- | --- | --- |
| MM1 | Play any catalog mapset, a base game and a custom setup: DSDA-Doom opens at its title screen and menu, not in the first map; New Game asks for the episode (where the mapset has several) and the skill | Gaming | ⏳ |
| MM2 | With Pistol start On, start from the menu and finish the first map: the second map starts with only the pistol and 50 bullets; with it Off, the inventory carries over | Gaming | ⏳ |
| MM3 | Home Continue also opens the menu; Load Game lists that profile's saves and loading one works | Gaming | ⏳ |
| MM4 | Play setup, base-game setup and Mapset detail show no skill row or skill text; D-pad and Back still move cleanly around Pistol start | Gaming | ⏳ |

# 026 — Local content: custom setups and extra files with a player-set load order

**Status:** Done  ·  **Milestone:** M3  ·  **Depends on:** 004, 005, 010, 018 · **Decision:** D-026
**Owner request (2026-10-17):** "é crucial que rode qualquer coisa arbitrária que a pessoa queira,
e que seja possível ajustar a ordenação. Tem que ser possível também adicionar PWADs em cima de
coisas baixadas pelo DDL."

## Goal

A player can launch any WADs and patches they have on the Deck — alone or on top of an installed
catalog mapset — choosing the load order from the controller, with the same isolated configs,
profiles and saves as catalog mapsets, and without DDL ever claiming the result is verified.

## Concepts

- **Custom setup:** a player-made launch target: a name, a base IWAD, an ordered list of user
  files, a compatibility choice, and its own profiles (spec 018) and saves. Shown in Library and
  Home like an installed mapset, badged **Custom — not verified**.
- **Extra files:** an ordered list of user files attached to **one profile of an installed catalog
  mapset**, loaded after the recipe's own files. The recipe's verified order is never changed;
  the profile is badged **Custom — not verified** while it has extras.
- **User file:** a file the player chose, referenced **in place** by absolute path, with its
  SHA-256, size and detected kind recorded. DDL never copies, moves, modifies or executes it.

## Requirements

### Phase A — core (`ddl-core`, CLI)

- **R1** — **Accepted files:** exactly what DSDA-Doom 0.30.1 loads through `-file` and `-deh`
  (see `docs/research/dsda-engine.md` §1 — WAD, ZIP, DEH/BEX; confirm in upstream source whether
  other extensions such as `.pk3` or `.lmp` are accepted and cite it; do not guess). Kind is
  detected from content (WAD `IWAD`/`PWAD` header and directory sanity, ZIP local-file header,
  DEH/BEX as bounded text), not the extension alone. Reject: directories, devices, sockets, files
  above a documented size cap, unreadable files, symlinks that resolve to non-regular files.
- **R2** — **Custom setup model:** versioned JSON under DDL's data dir (one file per setup, atomic
  writes, schema version), holding name, IWAD (an identified IWAD from spec 004, **or** a user file
  whose header is `IWAD`), ordered user files, compatibility (**Auto** = no `-complevel`, or one
  value from the engine's documented list), and created/updated times. Ids are stable and safe as
  directory names.
- **R3** — **Launch arguments:** reuse the existing DSDA launch builder: absolute paths, separate
  arguments, `-noautoload`, the DDL-owned config/save/data paths of the selected profile; one
  ordered `-file` list (WAD/ZIP in player order) and one ordered `-deh` list (patches in player
  order). DSDA applies patches after WADs regardless of argv position (research §1), so the model
  keeps two ordered groups rather than pretending they interleave. For a catalog mapset with extra
  files: recipe files first in recipe order, then the extras; recipe patches, then extra patches.
- **R4** — **File identity at launch:** before every launch, check each user file still exists and
  matches its recorded size and SHA-256. Missing → launch blocked with the path named. Changed →
  launch blocked until the player confirms "Use the changed file" (re-records the hash) or removes
  it. Never silently substitute.
- **R5** — **Extra files on catalog profiles:** a profile (spec 018) of an installed mapset can
  carry an ordered extras list (additive field, older state loads unchanged). Changing extras on a
  profile that has saves warns that existing saves may not load.
- **R6** — **Deleting:** deleting a custom setup removes only DDL's record and — with the D-022
  checkbox — its profiles and saves; **user files are never touched**. Removing a file from a list
  never deletes it from disk.
- **R7** — **CLI (Konsole, spec 014 style):** `ddl custom list|show|create|add-file|move|remove-file|delete`
  and `ddl profile extras …` (or equivalent) so phase A is testable end to end and the Deck can be
  rescued without the UI; launches go through the existing `ddl` launch path.

### Phase B — UI (`crates/ddl`)

- **R8** — **File browser:** extend the spec 010 controller directory browser into a file picker:
  starts in the home dir with shortcuts to Downloads, Documents and mounted SD cards
  (`/run/media/…`); shows only accepted kinds with size; multi-select with A, confirm with Start or
  a "Add N files" action; B backs out. No text entry needed.
- **R9** — **Order editor:** the setup / extras screen lists **Files (load order)** and **Patches
  (applied after files, in order)**. A picks up an item, D-pad up/down moves it, A drops it; X
  removes; Y adds files (Y is the reserved Library binding for this, spec 015). Recipe files of a
  catalog mapset appear above the extras, locked and dimmed.
- **R10** — **Entry points:** Library gains **New custom setup** (name via Steam's on-screen
  keyboard or an auto name the player can keep), custom setups appear in Library and Home Continue;
  Play setup of an installed catalog mapset gains **Extra files** for the active profile. Custom
  items show the **Custom — not verified** badge and the painted fallback art (spec 013).
- **R11** — **Compatibility and IWAD choice:** controller pickers for the IWAD (identified IWADs
  plus "a file I added that is an IWAD") and compatibility (Auto or the documented list, with a
  one-line hint that Auto lets DSDA-Doom decide).

## Out of scope

Downloading arbitrary files from URLs, `/idgames` browsing, editing files, demos/recording,
free-form extra command-line arguments (later, if asked), Lutris (deferred by D-026).

## Acceptance criteria

- [x] **A1** (R1–R4, R6–R7) — tests with fixture WAD/ZIP/DEH files: kind detection and
  rejections; atomic, versioned setup storage; exact argv for a custom setup (both groups, Auto and
  explicit complevel, user-file IWAD); missing and changed files block launch; deleting never
  touches user files; CLI round trip in a scratch XDG root.
- [x] **A2** (R3, R5) — exact argv for a catalog mapset profile with extras (recipe first);
  old profile state loads; saves warning shown.
- [x] **A3** (R8–R11) — headless renders: file browser with multi-select, order editor mid-move,
  New custom setup, Library and Home with a custom setup, Play setup with extras; D-pad focus order
  tested.
- [x] **A4** — fmt, clippy `-D warnings`, tests green.
- [ ] **A5** — *Needs Deck Validation:* add a local PWAD from the SD card, reorder, launch; layer a
  PWAD over an installed mapset; move the file and see the missing-file message.

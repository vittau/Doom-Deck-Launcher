# 076 — Plain normal/COMPAT SIGIL edition copy

## Scope and result

The SIGIL recipe now names its editions in plain player language instead of
`REG`/`E3` jargon, while every identifier, hash, dependency, default and
load-order rule stays exactly as it was.

| Component | Before | After |
| --- | --- | --- |
| `main` | "SIGIL" / "Main episode/mapset with bundled MIDI music." | "Normal episode (MIDI)" / the normal Episode 5 maps with the free MIDI soundtrack; DSDA-Doom supports it directly |
| `compat` | "Episode 3 compatibility WAD" / E3M1–E3M9 replacement | "COMPAT edition maps (older ports)" / same maps in DOOM's Episode 3 slots for older ports; DSDA-Doom does not need it |
| `soundtrack` | "Buckethead soundtrack (REG WAD)" | "Normal episode with Buckethead music (paid)" / the complete normal episode with Buckethead's MP3 score, replacing the free MIDI WAD |
| `compat-soundtrack` | "Buckethead E3 soundtrack (paid)" | "COMPAT edition soundtrack (Buckethead, paid)" / music only, used with the COMPAT maps, never with the normal episode |

The recipe `description` now states the three-way relationship in one sentence:
the free MIDI episode, the optional paid Buckethead replacement, and the COMPAT
edition for older ports that DSDA-Doom does not need.

Filenames remain in the obtain notes where they identify the purchased file:
`SIGIL_V1_23_REG.wad` (166,530,532 bytes) and `SIGIL_SHREDS_COMPAT.wad`
(162,146,269 bytes). `REG` now appears only inside that filename, never as an
edition name. The load order is unchanged: the paid Buckethead WAD replaces the
free map WAD in the maps position; the COMPAT soundtrack still requires the
free `compat` maps and loads after them; no ID, hash, byte count, dependency,
default or ordering field changed.

## Evidence

Headless 1280×800 renders of the real production widgets:

- SIGIL advanced options, soundtrack row focused:
  `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/app-header-brand/target/headless/sigil_editions_name_the_normal_and_compat_editions/076-sigil-options.png`
- Imported paid soundtrack row opening the shared import sheet:
  `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/app-header-brand/target/headless/sigil_editions_name_the_normal_and_compat_editions/076-sigil-import-sheet.png`

The options render shows all four labels on one line each with room to spare;
"Normal episode with Buckethead music (paid)" and "COMPAT edition soundtrack
(Buckethead, paid)" are the longest and both fit at the default 22 px row size.
The import sheet shows "Import" and "On" side by side, fully inside the sheet
frame. The full per-recipe fitter pixel audit also rendered every SIGIL
component row (`025-component-sigil-10*.png`) and found no glyph outside its
slot.

## Verification

- `headless::journeys::sigil_editions_name_the_normal_and_compat_editions`
  (new): renders SIGIL Setup with advanced options and an imported paid
  soundtrack, asserts the four labels are fitted and stay at or above the 20 px
  floor within the 30 px row, captures the options and the import sheet, and
  checks the sheet actions. This is the meaningful end-to-end check; the
  intermediate core copy test was removed because it only re-read the TOML
  fields already exercised here and in the recipe parser tests.
- `DDL_TITLE_RECIPE=sigil title_fit_setup_and_components_first`: production
  fitter pixel audit of every SIGIL screen and component row; passes.
- `cargo test -p ddl-core catalog::`: 107 passed.
- `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets -- -D warnings`: clean.

## Limits

Physical Deck legibility is owner validation. The five shared screenshots are
owned by the final 074/077 refresh; this copy-only change does not alter any of
their captured states. No visible window was opened and no screen was captured.
No push, tag, release or coordinator-document change was made.

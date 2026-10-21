# 068 — Verify purchased SIGIL editions

**Status:** Complete. R1–R5 are implemented: the paid SIGIL/SIGIL II identities are recorded, and
the SIGIL COMPAT + SHREDS composition is modeled, validated, switchable from the controller and
resolved in map-before-music order. Deck playback remains **Needs Deck Validation**.

## What was inspected (R1)

Read-only inspection only. I parsed the 12-byte WAD header and the lump directory with a local
script, never executed a WAD or a bundled tool, and copied no paid bytes into Git. The owner's
directories were not modified.

### WAD files

| File | Size (bytes) | SHA-256 | WAD directory |
| --- | ---: | --- | --- |
| `SIGIL_USB_IMAGE_V1_23/WAD/SIGIL_V1_23.wad` | 4,640,555 | `29c3b8400657be4546eab1878e2883ffc50771c6be575677b1be5a0d0430f55d` | PWAD, 146 lumps, E5M1–E5M9, 11 MIDI |
| `SIGIL_USB_IMAGE_V1_23/WAD/SIGIL_V1_23_REG.wad` | 166,530,532 | `46fcc22fba47fff3872d9d294ee07bdbab4b0ec9feaab277f83836f5aed80bd0` | PWAD, 146 lumps, E5M1–E5M9, 11 MP3 |
| `SIGIL_USB_IMAGE_V1_23/WAD/SIGIL_COMPAT_V1_23.wad` | 4,634,503 | `a79416b5e1a64b2946c09dcc5704f5d1448728b1afa2d761ccc067cd9ecce2b5` | PWAD, 158 lumps, E3M1–E3M9, 11 MIDI, no UMAPINFO |
| `SIGIL_USB_IMAGE_V1_23/WAD/SIGIL_SHREDS_COMPAT.wad` | 162,146,269 | `e5592bc76951a63d8647c5dc7004378ca4d7fcecfda3ab73acc6c220f1193ab7` | PWAD, 11 lumps only, no maps, 11 MP3 (`D_E3M1…D_E3M9`, `D_INTRO`, `D_INTER`) |
| `SIGIL_II_REGISTERED_V1_0/SIGIL_II_V1_0.WAD` | 5,632,323 | `9cadd239ea960ad40eeb655757941313cc8cb4f5f1c07a701ac986be4710a359` | PWAD, 171 lumps, E6M1–E6M9, 11 MIDI |
| `SIGIL_II_REGISTERED_V1_0/SIGIL_II_MP3_V1_0.WAD` | 146,563,690 | `8a803f5425d4bf31ddfe537b659d794acdff0631316f51b9ed45311b5ed059dc` | PWAD, 171 lumps, E6M1–E6M9, 11 MP3 |

### Supplied documentation

| File | Size (bytes) | SHA-256 |
| --- | ---: | --- |
| `SIGIL_USB_IMAGE_V1_23/WAD/SIGIL_README_V1_23.txt` | 10,580 | `9d3877d88eb49a0475300f465b1c2771c847ac0e83550d31c74de24018fbef6c` |
| `SIGIL_USB_IMAGE_V1_23/WAD/SIGIL_V1_23.txt` | 5,001 | `d1445fb883296f69dde7dbd300032bb0c8804f6a6a4b944f8fbb5231bcb79b9e` |
| `SIGIL_II_REGISTERED_V1_0/SIGIL_II_README_V1_0.txt` | 9,442 | `0db9b8f0ffdad6a36e48d9dbad1be2c72885f1164a75b52665327c5396700706` |
| `SIGIL_II_REGISTERED_V1_0/SIGIL_II_V1_0.txt` | 3,586 | `7c6dab2977c157311d945fb2afa7f8a27399d9a5040e768a82a506a333733f4a` |
| `SIGIL_II_REGISTERED_V1_0/SIGIL_II_MP3_README_V1_0.txt` | 9,475 | `bb52c8d45bbb86eb587f8cdfc694f3c6c4d9849f52c23eba24fd649451fcac0c` |
| `SIGIL_II_REGISTERED_V1_0/SIGIL_II_MP3_V1_0.txt` | 3,230 | `441b89ba9e1012f2ddf67868ea3b71a4fa62d3c39f8f3e661b13cae4372bbc2d` |

The USB image also contains 11 WAV soundtrack tracks, a `Behind the Scenes` MP4 and a `.DS_Store`;
none is a loadable mapset component. The SIGIL II folder contains no separate soundtrack WAD.

## Composition findings (R1/R2)

**Confirmed by lump-level comparison (read-only):**

- `SIGIL_V1_23_REG.wad` is a **complete replacement**, not a music add-on. It has the same 146
  lump names in the same order as the free WAD; exactly 11 music lumps (`D_E5M1…D_E5M9`,
  `D_INTER`, `D_INTRO`) differ. Every map, `DEHACKED`, `UMAPINFO`, graphic and demo lump is
  byte-identical to `SIGIL_V1_23.wad`.
- The REG music lumps are MP3 (`ID3`); ID3 tags name **Buckethead** and the track titles match
  `SIGIL_V1_23.txt` (for example `D_E5M1` “Romero One Mind Any Weapon”, `D_INTRO` “Eye on Spiral
  part 1”).
- `SIGIL_II_MP3_V1_0.WAD` is also a **complete replacement**: 171 lumps, E6M1–E6M9, with exactly
  11 music lumps differing from `SIGIL_II_V1_0.WAD`; every non-music lump is byte-identical. Its
  music lumps are raw MPEG frames (no ID3); the THORR identity comes from
  `SIGIL_II_MP3_README_V1_0.txt` and `SIGIL_II_MP3_V1_0.txt`, not from the filename alone.
- `SIGIL_SHREDS_COMPAT.wad` is **music-only**: 11 lumps, no maps, no `DEHACKED`, no `UMAPINFO`.
  It is only useful after `SIGIL_COMPAT_V1_23.wad` because its lumps are named `D_E3M*`.
- `SIGIL_COMPAT_V1_23.wad` is the E3 remap: 158 lumps, E3M1–E3M9, `DEHACKED` with E3 strings,
  MIDI `D_E3M*`, no `UMAPINFO`. It is the free-edition compat WAD, byte-identical to the catalog
  archive member.

**Native versus COMPAT on DSDA-Doom v0.30.1 (Confirmed from bundled docs + pinned source):**

- `SIGIL_V1_23.txt`: the compat WAD is for older ports that do not support E5 and makes the maps
  replace E3.
- `SIGIL_II_V1_0.txt`: DSDA-Doom is listed under “Tested With”; the warning is only that “some
  engines do not support episode 6”, and there is no SIGIL II compat WAD.
- DSDA-Doom `dsda_LoadMapInfo` adds original episodes and then loads every `UMAPINFO` lump
  ([mapinfo.c:452–457](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/mapinfo.c#L452-L457));
  the `episode` key appends the custom E5/E6 episode
  ([umapinfo.cpp:331–360](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/umapinfo.cpp#L331-L360)).
  `G_InitNew` skips its episode clamp when the requested map lump exists
  ([g_game.c:3003](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L3003)),
  so E5M1/E6M1 start natively. In-WAD MP3/OGG is routed through SDL_mixer
  ([i_sound.c:1375](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_sound.c#L1375),
  [i_sound.c:1496](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_sound.c#L1496)).

**Recommended DSDA-Doom load order for the owner files:**

| Goal | Load exactly | Why |
| --- | --- | --- |
| SIGIL, free MIDI | `SIGIL_V1_23.wad` | Native E5 UMAPINFO; no compat needed. |
| SIGIL, paid music | `SIGIL_V1_23_REG.wad` | Complete map+MP3 replacement; never add the free WAD. |
| SIGIL II, free MIDI | `SIGIL_II_V1_0.WAD` | Native E6 UMAPINFO. |
| SIGIL II, paid music | `SIGIL_II_MP3_V1_0.WAD` | Complete map+MP3 replacement; never add the free WAD. |
| Old/strict port only | `SIGIL_COMPAT_V1_23.wad`, then `SIGIL_SHREDS_COMPAT.wad` | E3 remap, then music-only MP3 override; not the DSDA path. |
| Any | Optional wide status bar last | The recipe already places it after the mapset. |

Do not combine REG/free, MP3/free, COMPAT/main or SHREDS_COMPAT/main. The COMPAT path is not “more
correct” for DSDA-Doom: it replaces E3 and discards native E5, and SIGIL II has no E6 equivalent.
Audio playback is **Needs Deck Validation**; only source support and file identity are confirmed.

## Changes made (R3, bounded)

- `catalog/sigil.toml`: the existing `soundtrack` component now names
  `SIGIL_V1_23_REG.wad`, records its exact size/SHA-256
  (`46fcc22fba47fff3872d9d294ee07bdbab4b0ec9feaab277f83836f5aed80bd0`), and states that it is a
  complete replacement for `SIGIL_V1_23.wad`. `replaces = "main"` and the exclusive edition group
  are unchanged, so only one map WAD can be selected.
- `catalog/sigil-ii.toml`: the `soundtrack` component now names `SIGIL_II_MP3_V1_0.WAD`, records
  its exact size/SHA-256
  (`8a803f5425d4bf31ddfe537b659d794acdff0631316f51b9ed45311b5ed059dc`), and states that it is a
  complete replacement for `SIGIL_II_V1_0.WAD`.
- Both recipes record the owner-supplied read-only inspection and the MP3 playback limitation.
  Free-edition package/member hashes, URLs, components and load order are unchanged.
- No paid content, binary fixture, download URL, rehosting or soundtrack bundling was added.
- `docs/research/reference-sigil.md`: added §2.1 (inspected files, hashes, composition) and §2.2
  (native UMAPINFO versus COMPAT on DSDA v0.30.1), corrected the paid variant/composition tables,
  and updated the open questions.

### R5 — COMPAT + SHREDS composition

- Added an optional, backward-compatible `Component.requires` field (serde default) and one-level
  dependency validation. Chains, cycles, unknown/self prerequisites, replacement-plus-dependency
  and a prerequisite at an equal or later load position are rejected by lint and by runtime
  `active_components`, including for hand-built recipes and inactive components.
- `catalog/sigil.toml` now has a paid `compat-soundtrack` component for
  `SIGIL_SHREDS_COMPAT.wad` (known SHA-256, one music member, `requires = "compat"`, position 2).
  The existing free `compat` remains the E3 map replacement at position 1; the wide status bar moved
  to position 3. Legal SIGIL orders are now:
  - native/free: `main`, then optional wide bar;
  - paid native: `SIGIL_V1_23_REG.wad` alone, then optional wide bar;
  - COMPAT: `compat`, then optional `compat-soundtrack`, then optional wide bar.
- Central selection helpers in `Recipe` expand a lone dependent enable (SHREDS adds `compat`), clear
  exclusive peers and their dependents (choosing REG/native removes the COMPAT pair), and remove
  dependents when the prerequisite is disabled. Persisted orphan selections still fail profile and
  resolver validation instead of being silently repaired.
- Explicit CLI `--with` selections use order-independent validation: contradictory ids such as
  `--with compat --with soundtrack` fail with an actionable error; `--with compat-soundtrack` adds
  the free maps and then reports the missing paid import if needed.
- Imported user-supplied rows now show On/Off once imported. Confirming toggles the component;
  the safe action opens the existing file picker to replace/re-import it. The import path enables an
  initial import without toggling an already-enabled replacement off, and stages the required free
  COMPAT maps before importing SHREDS. No audio, config or launch integration was changed; those
  sections remain owned by task 067.
- No paid bytes, download URL, rehosting or binary fixture was added. The only paid identity is the
  supplied SHA-256 metadata.

### Post-review correction — imported-row sheet dispatch

- `Runtime::handle` wrapped the `Sheet` match in an outer `if confirm`, while
  `Sheet::UserComponent` contained its own `if confirm { toggle } else { browse }`. The nested
  branch was unreachable: Left/Import on an imported row dismissed the sheet without opening the
  picker. The sheet now dispatches both actions explicitly: Return toggles On/Off, Left opens the
  existing picker to replace the file, and Back still only dismisses.
- `Command::Import` now removes the component's previous imported member from the staging base
  before `stage_update`. Replacing a missing or corrupt imported file therefore recovers instead of
  failing `NotInstalled` while trying to retain the old file; the original manifest is still kept
  for generation retirement.
- Added the headless controller regression `imported_component_sheet_actions`. It drives the real
  imported soundtrack row and sheet, asserts the Import/On labels and 901 focus at 1280×800,
  exercises Left → fake directory picker → known-file replacement, Return → Off while keeping the
  file, deletes the imported file and recovers it through Left → Import, and checks Back leaves
  both the profile and the screen unchanged. The fixture gives only the test scene's soundtrack a
  synthetic known hash; the existing `purchased.wad` remains an unknown-file case. No native
  window or picker is opened.

### Why the supplied hashes were not the whole fix

Before this change both user-supplied components had `known_sha256 = []`. The importer therefore
always returned `matched = false`, and the UI rejected even the correct purchased WAD as
`UnknownFile`, so these editions could not be enabled. The exact hashes above make the supplied
paid WADs recognizable while every other file remains an unmatched mismatch (D-014). That was a
native identity fix only. The post-review correction also changes import-adjacent selection
behavior (the imported-row sheet dispatch and re-import staging), but it does not add a generic
resolver policy or change audio/launch behavior.

## Tests (R4)

- Added `catalog::tests::purchased_sigil_editions_are_known_complete_replacements`: freezes the
  three paid hashes, asserts `user_supplied`, `replaces = main` or `requires = "compat"`, the
  exclusive edition group, the exact filenames in `obtain`, and the map/music load order. It also
  asserts the free WAD hash is not accepted as a paid identity and that SHREDS never loads alone.
- Added `catalog::tests::compatibility_music_activates_maps_and_switching_keeps_the_selection_legal`:
  dependency activation, order-independent contradictory explicit selections, orphan rejection and
  runtime rejection of a hand-built broken dependency.
- Added `resolver::tests::sigil_compat_soundtrack_requires_maps_and_loads_after_them`: persisted
  orphan blocker, missing-import blocker, exact `compat` → `compat-soundtrack` → wide-bar file
  order, summary positions, and missing free-map blocker.
- Added service and CLI tests for controller-style switching, exclusive clearing, idempotent
  re-import enablement, and `ddl launch --with` contradiction/auto-prerequisite behavior.
- The existing synthetic-fixture test
  `library::tests::imports_known_and_unknown_user_files_and_never_rejects_unknown_hashes` covers
  positive hash recognition and unmatched-file behavior without touching the owner's assets.
- Added `headless::journeys::imported_component_sheet_actions`, the post-review controller
  regression described above. Its artifact is
  `target/headless/imported_component_sheet_actions/068-imported-component-sheet.png`; the rendered
  frame shows the “Imported component” title, the help text, the focused `On` button and the
  `Import` button at 1280×800.
- Before the correction, `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` passed on
  the merged tree with `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2`. The unchanged full suite was not
  repeated for this correction; the focused reruns were `cargo fmt --all --check`,
  `cargo clippy -p ddl --all-targets -- -D warnings` and
  `cargo test -p ddl imported_component_sheet_actions -- --test-threads=2`, all passing. No visible
  window or native picker was opened; the only screenshot is the headless artifact above.

## Left open

- Both recipes still accept only `doom-1.9ud` (Ultimate Doom), while the bundled texts also allow
  registered Doom 1.9. This is pre-existing and was not changed here; it needs a separate decision
  and Deck validation (registered `doom.wad` has only three IWAD episodes, but the WAD supplies its
  own `M_EPI5`/`M_EPI6` graphics).
- Audio playback and the episode menu under cl3 remain **Needs Deck Validation**; no engine was
  launched and no owner media was played.
- SHREDS is intentionally offered only as the paid COMPAT companion. Whether the advanced COMPAT
  path should be hidden further in the basic UI is a product decision, not a blocker: native E5
  remains the default and the recipe description states the dependency.

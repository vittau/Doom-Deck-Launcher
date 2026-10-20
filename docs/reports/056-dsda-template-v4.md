# 056 — DSDA template v4 (VSync, look sensitivity) and the gyro guide

**Status:** implementation complete; DSDA behaviour and the Deck feel remain
**Needs Deck Validation**. Branch `vittau/dsda-template-v4`.

## What changed

| File | Change |
| --- | --- |
| `crates/ddl-core/templates/dsda/steam-deck-default.toml` | `template_version = "4"`; `render_vsync = 1`, `uncapped_framerate = 1` (no `dsda_fps_limit`); `mouse_sensitivity_horiz = 5`, new `mouse_sensitivity_vert = 5`; new `[upstream_values]` table recording the v0.30.1 defaults of the three value keys that are new in v4. |
| `crates/ddl-core/src/dsda_config/template.rs` | Parses and validates `[upstream_values]` (each key must be a value the template owns) and exposes `Template::upstream_value`; new `render_vsync`, `uncapped_framerate`, `mouse_sensitivity_vert` fields in `Values`; tests for v4 values, ranges, the absent FPS cap and the upstream record. |
| `crates/ddl-core/src/dsda_config/merge.rs` | Rule 2 of the three-way merge now also accepts a value key's recorded upstream default, mirroring the spec-034 binding rule; tests for an untouched-new-value and a player-edited-new-value. |
| `crates/ddl-core/src/dsda_config/manager.rs` | The 029/034 migration tests now build accurate older templates and expect v4; two new v3 → v4 migration tests (untouched keys take 5/5 + VSync; player-edited sensitivities and VSync survive, with conflicts reported). |
| `crates/ddl-core/src/dsda_config/overrides.rs` | `render_vsync` and `uncapped_framerate` added to the per-profile `-assign` allowlist, so the invariant "every template value is overridable" still holds. |
| `docs/research/dsda-engine.md` | New §10 with pinned v0.30.1 links for every key, the menu labels, the render loop and the VSync/FPS-limit interaction; two new validation gates. |
| `docs/steam-deck-guide.md` | Template v4 framing; the owner's gyro preset (Gyro To Mouse, 25000 px, speed deadzone 0, Yaw) with one-line explanations; 5/5 preset stated in the gyro and sensitivity sections; free-look caveat; validation list updated. |

No README change: the README links to the controller section and does not repeat
it.

## Research findings (pinned to DSDA-Doom v0.30.1)

Source: tag `v0.30.1` = `8ed581ad5c868b4dba1c39eb21d29e4ddbed9fad` (shallow clone
verified with `git rev-parse HEAD`). All links are in research §10.

- **Keys confirmed:** `render_vsync` bool, default `0`, `0–1`, on-change
  `M_ChangeVideoMode` [configuration.c:1185–1188]; `uncapped_framerate` bool,
  default `1` [1189–1192]; `dsda_fps_limit` int, default `0`, `0–1000`
  [1193–1196]; `dsda_background_fps_limit` int, default `35` [1197–1200].
- **How they combine:** `uncapped_framerate` drives `movement_smooth`, so the
  loop draws and interpolates every display frame [r_fps.c:81–87,
  d_main.c:614–645]. `D_Display` presents through `I_FinishUpdate`
  [d_main.c:550–552]. VSync is a renderer/swap setting: software adds
  `SDL_RENDERER_PRESENTVSYNC` [i_video.c:1298–1305], OpenGL calls
  `SDL_GL_SetSwapInterval(1)` [i_video.c:1358–1361], and the GL swap is
  `SDL_GL_SwapWindow` [gl_main.c:972–977]. `dsda_LimitFPS` only sleeps when a
  numeric limit is set and the game is uncapped or unfocused [time.c:104–121].
  With VSync on, uncapped on and `dsda_fps_limit 0`, the effective frame rate is
  the panel refresh with interpolation kept — the owner's request.
- **Stop condition not triggered:** DDL does not set `videomode`, so a fresh
  install is the software path; the GL path also honours VSync. Neither is a
  no-op by construction. `render_vsync` is forced off only for `-timedemo` /
  `-fastdemo`, which DDL never passes.
- **Sensitivity labels confirmed:** in Options → Mouse Settings,
  "Horizontal Sensitivity" is `mouse_sensitivity_horiz` (default `10`) and
  "Vertical Sensitivity" is `mouse_sensitivity_vert` (default `1`)
  [m_menu.c:3565–3566]. The controller page uses "Left/Right Horizontal/Vertical
  Sensitivity" for the analog keys, which have a different `0–16384` scale
  [m_menu.c:3587–3590]. The stop-and-ask condition for R2 is therefore not
  triggered; the requested 5/5 is the mouse pair, not the analog pair.
- **Axis detail:** horizontal motion uses `mouse_sensitivity_horiz`; vertical
  uses `mouse_sensitivity_vert / 8` when free aim is off and
  `mouse_sensitivity_mlook` when `dsda_FreeAim()` is on [g_game.c:1396–1416].
  `mouse_sensitivity_mlook` is left at 10 because the owner asked only for the
  two menu-labelled sensitivities.
- **Gyro names:** "Gyro To Mouse", "3DOF to 2D Conversion Style" (Yaw/Roll/Yaw +
  Roll) and the speed deadzones are named in Valve's 10/25/23 client notes;
  "Gyro Angles to Mouse Pixels" is corroborated by current community guides. The
  guide marks the exact current labels and the 25000 px feel **Needs Deck
  Validation**, as the spec allows.

## Migration design (R3)

A v3 install has no values for `render_vsync`, `uncapped_framerate` or
`mouse_sensitivity_vert`, but DSDA rewrites the whole file on save, so the active
file holds their upstream defaults. The existing three-way merge already handled
this for bindings through a recorded `upstream` triple; v4 records the scalar
equivalent in `[upstream_values]`. Rule 2 then treats a matching active value as
"untouched" and takes the v4 value; anything else becomes a conflict that keeps
the player's value, as in 029/034. The migration is recorded in `state.json` with
the v4 baseline and a `<timestamp>-4.cfg` backup, unchanged from the existing
manager path.

A boolean key whose player edit equals the upstream default (VSync left off)
cannot be distinguished from untouched; it takes the v4 value. That is the same
limitation the binding rule has and is noted under open questions.

## Acceptance

| Item | Result | Evidence |
| --- | --- | --- |
| **A1** (R1, R2) research section with pinned links; template v4 under test | **Done** | Research §10; `v4_turns_on_vsync_without_capping_the_frame_rate`, `v4_sets_both_mouse_look_sensitivities_to_five`, `deck_video_values_are_in_the_documented_ranges`. |
| **A2** (R3) untouched v3 → v4 values; player-edited sensitivity/vsync kept | **Done** | `a_version_bump_to_v4_applies_vsync_and_look_sensitivity`, `a_version_bump_to_v4_keeps_player_edited_sensitivity_and_vsync`, plus the updated 029/034 migration tests. |
| **A3** (R4) guide updated; no Steam Input code | **Done** | `docs/steam-deck-guide.md` §3 and §5; grep below. |
| **A4** fmt, clippy `-D warnings`, `cargo test --workspace` green | **Done** | Commands and results below. |
| **Needs Deck Validation** | **Open** | VSync/no tearing/power draw and gyro feel; guide and research mark them. |

### No Steam Input code (A3)

```text
$ grep -rni "steam[ _-]*input\|gyro" crates --include='*.rs' --include='*.slint' --include='*.toml'
crates/ddl-core/templates/dsda/steam-deck-default.toml:43:# Keep mouse input enabled so trackpad / gyro-as-mouse keeps working alongside
```

The only match is a comment in hand-authored template data. No Steam Input code,
layout file handling or `.vdf` controller access exists; the `vdf` matches under
`crates/ddl-core/src/iwads/steam.rs` are the pre-existing Steam library scanner.

## Verification run

```text
$ cargo fmt --all --check
(no output; clean)

$ cargo clippy --workspace --all-targets -- -D warnings
(no output; clean)

$ cargo test --workspace
test result: ok. … (12 suites)
658 passed; 0 failed; 0 ignored; exit=0
```

Run on 2026-10-20 after merging `main` (`d16e9b2`), on macOS arm64. The run
includes the two new v3 → v4 migration tests and the updated 029/034 migrations.
Targeted during development: `cargo test -p ddl-core dsda_config` — 68 passed;
`cargo clippy -p ddl-core --all-targets -- -D warnings` — clean.

## New dependencies

None. The implementation uses the existing `serde`/`toml` stack and
`std::collections::BTreeMap`.

## Open questions / coordinator-owned proposals

- **Needs Deck Validation:** VSync on the real Deck — no tearing, frame rate
  tracks the panel (60 Hz LCD / 90 Hz OLED), interpolation smooth, and lower power
  draw than 0.0.7 at the same scene.
- **Needs Deck Validation:** gyro feel with 25000 px, deadzone 0, Yaw, plus the
  5/5 mouse sensitivity; confirm current Steam labels.
- **Free look:** with `allow_freelook` enabled, vertical aim uses
  `mouse_sensitivity_mlook` (left at 10). The owner asked for Horizontal/Vertical
  only; if 5/5 should also cover free look, that is a one-line template change and
  a new v5 migration — needs an owner decision.
- **Boolean default ambiguity:** a player who explicitly sets VSync to its
  upstream default (off) looks untouched and receives v4's on. Same limitation as
  the binding rule; no extra metadata can recover intent.
- **Propose for coordinator-owned files:**
  - `docs/status.md`: record spec 056/template v4 as done (pending Deck
    validation) with `render_vsync 1`, uncapped kept, 5/5.
  - `docs/journal.yml`: add the v4 template and research §10 entry.
  - `docs/deck-testing.md`: add an I5/0.0.7 check for VSync + uncapped FPS (no
    tearing, lower power) and one for the gyro preset. The file already records
    the request under the 0.0.7 run but has no check row.
  - `docs/decisions.md`: D-033 already captures the decision; no change needed.

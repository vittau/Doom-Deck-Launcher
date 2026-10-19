# 029 — Deck display: fit the screen, start DSDA widescreen — worker report

**Status:** R1–R5 implemented; A1–A3 verified headlessly and A4's fmt/clippy/tests
are green. The launcher no longer needs a launch option for scaling, and
DSDA-Doom opens fullscreen 16:10 with the widescreen status bar scaled to the
panel. All work is macOS-buildable and headless; the only item left is the Deck
re-check (S3, F4), which **Needs Deck Validation**.

## What changed

- `crates/ddl/src/display.rs` (new) — the display fit. A small `DisplayWindow`
  trait (`reported_scale_factor`, `physical_size`, `apply_scale_factor`),
  implemented for `slint::Window` and by a fake in tests. `fit_factor` is the
  pure `min(w / 1280, h / 800)`; `configure` reads `SLINT_SCALE_FACTOR` (a user
  value wins), otherwise fits in fullscreen and keeps the reported factor in
  `--windowed`, applies it through `WindowEvent::ScaleFactorChanged` (the public
  Slint 1.18.1 path) and logs the reported factor, physical size and applied
  factor.
- `crates/ddl/src/main.rs` — calls `display::configure` after choosing fullscreen
  or the 1280×800 logical window, then re-applies and re-logs whenever the
  physical size changes (a 250 ms Slint timer).
- `crates/ddl-core/src/dsda_config/template.rs` — the template gains a `[strings]`
  table (optional) and four integer video keys; `Template` owns `strings` and
  `merge`/`reset`/`config_text` handle them.
- `crates/ddl-core/templates/dsda/steam-deck-default.toml` — `template_version`
  bumped `1`→`2` and the Deck video baseline added (R4).
- `crates/ddl-core/src/dsda_config/{merge,manager,overrides}.rs` — strings flow
  through the three-way merge and reset; the video integer keys join the
  per-profile override allowlist.
- `crates/ddl/src/headless.rs` — additive test helpers: `setup_at(scale, size)`,
  `render_at(w, h)` and `isolated_env(module, case, vars)`.
- `docs/research/dsda-engine.md` — new §9 (R3).
- `docs/steam-deck-guide.md` — R5 troubleshooting line.

## R3 — DSDA-Doom v0.30.1 video keys (research §9)

From the pinned tag `v0.30.1` = `8ed581ad5c868b4dba1c39eb21d29e4ddbed9fad`, all
**Confirmed** unless marked: `videomode` (default `"Software"`),
`screen_resolution` (**string**, default `"640x480"`, parsed `%dx%d`),
`custom_resolution`, `use_fullscreen` (bool, default **0**),
`exclusive_fullscreen` (bool, default 0), `screenblocks` (10–11),
`render_screen_multiply` (1–5), `integer_scaling`, `render_aspect` (0 Auto /
1 16:9 / **2 16:10** / 3 4:3 / 4 5:4), `render_stretch_hud` (0 `not_adjusted` /
1 `doom_format` / 2 `fit_to_width`), `render_patches_scalex`/`y`,
`aspect_ratio_correction`, `hud_displayed` and `dsda_exhud`. Every row cites the
tagged `configuration.c`; the aspect mapping cites `v_video.c:1357–1376` and the
status-bar scaling cites `st_stuff.c:399–426`. A wide `STBAR` needs **no** flag;
the widescreen bar of spec 017 is covered. §9 records the corrected default for
`render_stretch_hud`: v0.30.1 defaults it to `patch_stretch_doom_format` (1), not
`not_adjusted` (0) as a §3 note in `docs/research/widescreen-statusbars.md` says.

## R4 — Deck video defaults and migration

Template v2 adds `screen_resolution "1280x800"`, `use_fullscreen 1`,
`exclusive_fullscreen 0`, `render_aspect 2` (16:10) and `render_stretch_hud 2`
(status bar fills the panel). Each is inside the confirmed range, asserted by
`deck_video_keys_are_within_dsdas_documented_ranges`. The string table is a small,
deliberate schema extension: DSDA's resolution key is a string and the old
template could only hold integers.

## Acceptance criteria

- **A1 (R1)** — `crates/ddl/src/display.rs` tests, run under the headless
  software renderer:
  - `fitting_ignores_the_reported_dpi_on_the_deck_panel`: a 1280×800 physical
    window that reports scale 2.25 is fitted to 1.0, and its frame is
    **byte-identical** to the same window reporting 1.0 (tab 4's focus ring is
    present at the fitted layout).
  - `fitting_scales_up_uncropped_on_a_larger_screen`: 1920×1080 → applied 1.35,
    the design is scaled and uncropped (tab 4 on screen; the spare right strip
    is the window background, not a crop).
  - `a_user_scale_factor_overrides_both`: with `SLINT_SCALE_FACTOR=1.5` in the
    child environment, the applied factor is 1.5 on both screen sizes.
  - PNGs (inspected), each under `target/headless/<case>/`:
    - `fitting_ignores_the_reported_dpi_on_the_deck_panel/029-deck-fit-1280x800-reported-2.25.png`
      — all four tabs visible, Settings focused; the fitted frame.
    - `fitting_ignores_the_reported_dpi_on_the_deck_panel/029-deck-fit-1280x800-reported-1.0.png`
      — byte-identical to the frame above.
    - `fitting_scales_up_uncropped_on_a_larger_screen/029-deck-fit-1920x1080.png`
      — the design scaled 1.35×, uncropped.
    - `a_user_scale_factor_overrides_both/029-deck-fit-1920x1080-user-1.5.png`
      — the `SLINT_SCALE_FACTOR=1.5` override.
  - **Letterboxing, not adapting.** The root `.slint` element pins
    `width: 1280px; height: 800px`, so a scaled-up window draws the 1280×800
    design scaled to 1728×1080 and the spare right strip is the window
    background (`Tokens.canvas`, the same `#120C0D` as the header), not a crop
    and not unpainted black. That is R1's "letterbox the spare axis with the
    background"; the design is left-aligned, not centred. Adapting would have
    meant letting the root fill the wider logical width — the layout does not do
    that, and editing `crates/ddl/ui/**` belongs to spec 030.
- **A2 (R2)** — `reported_physical_and_applied_reach_the_log` captures a
  `tracing` subscriber and asserts `reported=2.25`, `physical_width=1280`,
  `physical_height=800` and `applied=1` all reach the log.
- **A3 (R3, R4)** — `deck_video_keys_are_within_dsdas_documented_ranges` (parse +
  ranges + values), `strings_are_optional`, and
  `a_version_bump_adds_deck_video_defaults_and_keeps_a_user_change` (v1→v2 keeps
  `use_game_controller 0` and `screen_resolution "1920x1080"`, adds the four new
  integer defaults, keeps an unknown engine key).
- **A4** — `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --
  -D warnings`, `cargo test --workspace` (see "Verification"). The Deck re-check
  (S3 without a launch option, F4 widescreen) is **Needs Deck Validation**.

## Verification

Run on macOS arm64 after merging `main` (`152f326`, specs 028 and 030); the
branch is `vittau/deck-display`, the merge is `f1a737a` (no conflicts).

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo test --workspace` — **950 passed, 0 failed** across 16 test binaries.
- `python3 scripts/check-readme-links.py` — PASS (32 local links/anchors).

The four A1 PNGs were opened and inspected: the fitted 1280×800 frame shows all
four tabs with the Settings ring, its reported-1.0 twin is byte-identical, and
the 1920×1080 frame is the design at 1.35× with a background letterbox strip.

## New dependencies

None. `display.rs` uses only `slint` and `tracing`, already dependencies.

## Open questions

- `render_stretch_hud = 2` (`fit_to_width`) is DDL's explicit "fill the panel"
  choice; the research shows `doom_format` (the default) produces the same 4×
  uniform scaling for a 1280×800 / 16:10 screen, so either is defensible. Needs
  the Deck re-check to confirm the bar fills edge to edge.
- A user-changed `screen_resolution` (and any user-changed template key) is kept
  and reported as a merge conflict by design (spec 005); the Deck check should
  confirm that is the wanted behaviour for a stale resolution.
- String template keys are not reachable through `Overrides` (which is
  integer-only `-assign`), so `screen_resolution` is not a per-profile override.

## Spec items left open

- A4 Deck re-check (S3, F4) — physical hardware.
- A1's "PNGs inspected" is done by looking at the four files above; no golden
  image is committed (consistent with the other headless tests).

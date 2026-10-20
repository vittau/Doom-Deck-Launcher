# 060 — Focus clarity

Contract: [spec 060](../specs/060-focus-clarity.md), R1–R3 / A1–A3.

## Changes

- Retained inherited R1 commit `7799b39`; completed its wider-row behavior:
  when more than one target overlaps the source, enter the entire row at its
  leftmost eligible item. A source overlapping exactly one target keeps that
  column. Disabled nodes remain ineligible. Production and fixture navigation
  share this rule, including sheets and local-content screens. Per coordinator
  clarification, the fixture Home test now expects Up from Continue to reach
  the active tab, rather than jump diagonally to Discover in an overlapping
  vertical band; Down returns to the remembered Continue target.
- Primary actions now use blood red with bone text. All targets use the same
  separated amber focus ring with optional subtle own-fill brightening. Removed
  amber selection/progress fills. Selected add-on chips have a green edge and
  check; category chips use a green edge;
  checkbox selection retains its check and green edge; On text marks toggles.
  Active tabs use a bone underline. The profile-name field also receives the
  shared ring. First-run Install retains its spec 030 pulse on the red palette.
- Required components now use the same locked renderer as recipe files in
  Extra files: sunken fill, dimmed readable text, lock glyph, no activation or
  focus affordance. The required-recipe explanation remains visible. Unavailable
  actions use the same disabled fill/text without a lock; the coordinator
  clarified that the glyph specifically means locked by the recipe.
- Documented default, focused, primary, selected and disabled states in
  `docs/design/ui.md`; refreshed the four README renders.

No dependencies added. No remote, engine, compatibility or platform behavior changed.

## Verification

**Confirmed (headless):** targeted tests passed:

- `controller_060_rows_are_entered_at_their_leftmost_item`: Advanced Down →
  Skill; load order Up → Skill; Right → Pistol start; Left → Skill; disabled
  required component skipped; Advanced Up → first add-on chip; Right → next
  chip; full-width dialog checkbox Down → left Cancel, Right → Uninstall.
- `focus::tests`: Up and Down from a wide source spanning only the right two
  cells of a three-item row still reach the left edge; matching single columns
  remain aligned; disabled left cell is skipped.
- `controller_057_focus_walks_every_multi_item_screen`: now asserts exact row
  landing, including single-column retention, rather than row membership alone.
- `render_060_primary_selection_and_focus_are_distinct`: pixel assertions on
  the shared production renderer verify no amber control fill, red primary in
  both focus states, dark add-on/category selected-chip fills, sunken
  locked/unavailable fills, and amber
  rings on exactly the eligible focused target (even stale focus cannot highlight
  a disabled control).
- `render_060_focus_language` and the real ready-to-play production journey
  produce the acceptance renders below. Locked Extra files are also covered by
  `local_catalog_extras_lock_recipe_members_warn_for_saves_and_keep_profile_scope`.

Audited rows and navigation routes:

| Surface | Exact landing / coverage |
| --- | --- |
| Play setup | All optional/component option rows, Advanced, Skill / Pistol start, Recipe load order; both vertical directions |
| Add-on chips | Advanced Up lands on the leftmost None chip; Right enters the next chip |
| Settings | Sidebar; full-width preference → Add folder / Find IWADs; each returns Up to preference |
| Mapset detail | Primary and favorite action column |
| Base-game detail/setup | Action-row boundaries; profile/copy/compatibility rows; Skill / Pistol start; next full-width row |
| Custom detail/new | Two-button first row; following full-width rows; reverse traversal |
| Dialog | Full-width checkbox → Cancel / Uninstall; horizontal movement unchanged |
| Discover fixture | Wide card Up reaches the leftmost category chip; active-tab return preserves it |
| Matching grids/rows | Shared ranker unit test covers every aligned column in both directions; fixture/controller journeys retain grid behavior |
| Other sheets, picker lists, profile rows | Same shared ranker; their existing workspace controller journeys remain the regression coverage |

The component-layout audit now measures the actual lock/check label insets,
checks dimmed locked text versus bone selected-chip text, and scrolls locked
rows explicitly because they cannot receive focus.

Final checks on implementation commit `0d93270`: **all green**.

- `cargo fmt --all --check`.
- `cargo clippy --workspace --all-targets -- -D warnings`.
- `cargo test --workspace`: 770 passed across test targets and doctests;
  163 launcher tests and 565 core unit tests, zero failures/ignored tests.
- `python3 scripts/readme-screenshots.py` and
  `python3 scripts/readme-screenshots.py --check`; all four refreshed images
  visually inspected.

A1–A3 are **Confirmed (headless)**. Physical Deck acceptance remains below.

## Contrast

WCAG sRGB relative luminance, opaque foreground/background tokens (rounded).
Visual QA confirms the fills and readable labels in the actual software renders.

| Foreground / background | Ratio | Requirement |
| --- | ---: | --- |
| Bone `#F8EAD5` / primary `#9A3226` | 6.20:1 | ≥4.5:1 |
| Bone / focused primary `#A93D2E` | 5.22:1 | ≥4.5:1 |
| Bone / brightest pulse `#B03F30` | 4.92:1 | ≥4.5:1 |
| Bone / raised `#352527` | 12.26:1 | ≥4.5:1 |
| Bone / focused raised `#3F2D2F` | 10.89:1 | ≥4.5:1 |
| Locked/unavailable label and explanation `#9A928C` / deep `#120C0D` | 6.33:1 | ≥3:1 |
| Disabled tab text `#817C7B` / raised | 3.53:1 | ≥3:1 |
| Selected green `#88D9B0` / raised | 8.72:1 | Non-text mark |

## Headless PNG evidence

All paths below are repository-relative, 1280×800 software renders. No visible
window or screen capture was used. The before renders replay the pre-060 UI
styles from HEAD at takeover; the inherited locked-node classification is
already present, so they isolate the style change rather than reconstructing
an earlier application release. The ready-to-play pair was sent to the
coordinator in status message `msg_7e3c0bdee1b0` before final verification.

Ready-to-play comparison (actual production install journey):

- `target/headless/060-before/install_progress_and_play_setup/060-play-ready-unfocused.png`
- `target/headless/060-before/install_progress_and_play_setup/060-play-ready-focused.png`
- `target/headless/060-before/install_progress_and_play_setup/060-play-ready-skill-focused.png`
- `target/headless/060-after/install_progress_and_play_setup/060-play-ready-unfocused.png`
- `target/headless/060-after/install_progress_and_play_setup/060-play-ready-focused.png`
- `target/headless/060-after/install_progress_and_play_setup/060-play-ready-skill-focused.png`

Additional before/after states (including locked required component beside optional rows).
These local fixtures are uninstalled: their nominal Play-focus snapshot falls
back to the Install remediation, so the real ready-to-play pair above is the
Play focus acceptance evidence:

- `target/headless/060-before/render_060_focus_language/060-setup-play-unfocused.png`
- `target/headless/060-before/render_060_focus_language/060-setup-play-focused.png`
- `target/headless/060-before/render_060_focus_language/060-setup-skill-focused.png`
- `target/headless/060-before/render_060_focus_language/060-first-run-install.png`
- `target/headless/060-before/render_060_focus_language/060-dialog.png`
- `target/headless/060-before/render_060_focus_language/060-setup-addon-chips.png`
- `target/headless/render_060_focus_language/060-setup-play-unfocused.png`
- `target/headless/render_060_focus_language/060-setup-play-focused.png`
- `target/headless/render_060_focus_language/060-setup-skill-focused.png`
- `target/headless/render_060_focus_language/060-first-run-install.png`
- `target/headless/render_060_focus_language/060-dialog.png`
- `target/headless/render_060_focus_language/060-setup-addon-chips.png`

Extra files locked-row evidence:

- `target/headless/local_catalog_extras_lock_recipe_members_warn_for_saves_and_keep_profile_scope/026-extras-order.png`

README renders:

- `docs/screenshots/home.png`
- `docs/screenshots/discover.png`
- `docs/screenshots/play-setup.png`
- `docs/screenshots/mapset-detail.png`

Before snapshots are retained locally under `target/headless/060-before`.
Final acceptance states regenerate with `cargo test -p ddl render_060_focus_language`
and `cargo test -p ddl headless::journeys::install_progress_and_play_setup` (the
latter writes ready variants under `target/headless/install_progress_and_play_setup`).
README images regenerate with `python3 scripts/readme-screenshots.py`.

## Integration and remaining validation

Merged current `main` (`2a38b7c`) in `a21b508` and merged again before final
checks (already up to date). Coordinator instructed not to wait for spec 059;
its later import dialog / Diagnostics integration remains coordinator-owned.
No coordinator-owned status/journal/decision/overview files were edited; the
status change on this branch came only from merging main.

**Needs Deck Validation:** Play no longer looks selected, focus is obvious on
all screens, physical handheld readability and pulse appearance. R1–R3 are
implemented; no software spec items intentionally left open.

# 034 — The owner's DSDA-Doom controller layout as the Deck default — worker report

**Status:** R1–R4 implemented; A1 and A2 verified headlessly, A3's fmt/clippy/tests
green on macOS. The only item left is the owner's on-device E5 re-check with the
new layout (**Needs Deck Validation**). No requested binding was dropped, so no
`ask` was needed (see R3 below).

## What changed

- `crates/ddl-core/templates/dsda/steam-deck-default.toml` — template
  `template_version` `"2"` → `"3"`; new `[[bindings]]` array with the owner's
  table. Each entry carries the wanted keyboard/mouse/gamepad codes and the
  DSDA-Doom v0.30.1 `upstream` default it replaces.
- `crates/ddl-core/src/dsda_config/template.rs` — `Template` now owns
  `bindings: Vec<TemplateBinding>`; the TOML record is validated (`action` in
  the new `KNOWN_BINDING_ACTIONS`, keyboard `0`/`1..511`, mouse `-1..7`, gamepad
  `-1..22`, no duplicates) and expanded to all three DSDA input profiles. New
  `TemplateError::Invalid` keeps the existing `invalid DSDA config template: …`
  message. `TemplateBinding` is re-exported.
- `crates/ddl-core/src/dsda_config/merge.rs` — rule 2 added: a template key the
  baseline does not have yet, whose active value is exactly the recorded
  `upstream` default, takes the template value. A rebound binding falls through
  to the existing conflict rule and is kept.
- `crates/ddl-core/src/dsda_config/manager.rs` — `reset()` re-applies bindings
  like values and strings; migration test for v2→v3.
- `docs/research/dsda-engine.md` — new §4 subsection "Controller layout
  semantics (spec 034)" plus two implication bullets.
- `docs/steam-deck-guide.md` — the Classic layout table now shows DDL's shipped
  binding for every control, with a paragraph on what changed from DSDA's
  defaults and how a user's own rebind survives.
- No new dependencies. No UI, catalog, Steam Input or launcher code touched.

## R1 — research (A1)

All from the pinned tag `v0.30.1` = `8ed581ad5c868b4dba1c39eb21d29e4ddbed9fad`
(raw files fetched from GitHub; the exact links are in the new research
subsection). The findings that shaped the implementation:

- **Binding storage (Confirmed).** `input_defs[]` (`m_misc.c:391-581`) gives each
  action a name and default `{key, mouse, joy}`; `dsda_input_t`
  (`input.h:212-218`) holds a key **list**, one mouse and one gamepad button.
  Ranges: keys `1..511` (`0` = none), mouse `-1..7`, gamepad `-1..22`
  (`input.h:24-27`, `input.c:289-318`). A file stores integers, never menu names.
- **Three profiles (Confirmed).** A line is
  `action key[,key…] mouse joy | … | …`; DSDA writes all three segments
  (`m_misc.c:642-670`) and installs each default in all three
  (`m_misc.c:696-701`). DDL writes the same triple to every profile.
- **One button, several actions (Confirmed).** Pad state is one bitmask per event
  (`game_controller.c:118-167`); every action independently compares its own
  `joyb` (`input.c:158-176`). So D-pad Up/Down can zoom the automap *and* keep
  their menu/fly defaults.
- **Automap zoom (Confirmed).** `input_map_zoomin/out` are handled only inside
  `AM_Responder` while the map is open (`am_map.c:1156-1187`, release at
  `1244-1256`); `G_Responder` tries the automap first and returns when it
  consumes the event (`g_game.c:1295-1305`). Automap panning has no pad binding
  (`m_misc.c:445-448`), so D-pad Left/Right still switch weapons while the map is
  open.
- **180° turn (Confirmed).** The action is `input_reverse` (`m_misc.c:401`); it
  adds `QUICKREVERSE` (32768 half-turn units) on the first activated tick
  (`g_game.c:206`, `554-563`), is skipped while strafing, and is disabled with a
  message while a strict-mode demo records.
- **Quick save/load (Confirmed).** F6/F9 with no pad default
  (`m_misc.c:436-437`), handled by `M_Responder` during play
  (`m_menu.c:5640-5659`), which runs before the game responder
  (`d_main.c:252-259`).
- **Proposed / Needs Deck Validation:** the Deck-control-to-SDL-button labels
  (View→`pad back`=4, L1→`pad l`=9, R1→`pad r`=10, X→`pad x`=2) are Steam Input
  behaviour; the source confirms the SDL order only. The research table labels
  them as such.

## R2 — the template (A2)

The owner's table maps to: `input_map`=4 (View), `input_map_zoomin`=11,
`input_map_zoomout`=12, `input_use`=0 (already DSDA's default, so **not owned**),
`input_autorun`=1 (B), `input_nextweapon`=14, `input_prevweapon`=13,
`input_quicksave`=9 (L1), `input_quickload`=10 (R1), `input_reverse`=2 (X).
Every entry keeps DSDA's keyboard binding (`Tab`, `=`, `-`, `Caps Lock`, `F6`,
`F9`, `/`; next/prev weapon had none), so keyboard play is unchanged. Fire,
strafe, movement, look, jump and all menu actions keep DSDA's defaults.

Tests proving A2 (all in `cargo test -p ddl-core dsda_config`, 60 tests):

- `every_binding_is_a_known_action_with_valid_codes` — every action is in
  `KNOWN_BINDING_ACTIONS` and every keyboard/mouse/gamepad code (wanted *and*
  upstream) is inside DSDA's accepted ranges; all three profiles are identical.
- `deck_controller_layout_matches_the_owner_table` — the exact nine buttons, the
  recorded upstream buttons, keyboard/mouse preserved, `input_use` deliberately
  unowned.
- `upstream_defaults_match_the_researched_source` — the template's `upstream`
  triples are the ones in the research table.
- `a_version_bump_applies_the_deck_layout_and_keeps_a_rebound_button` (manager) —
  a realistic v2→v3 migration: the previous baseline's scalars plus a DSDA-shaped
  active file where the player rebound only `input_map` to `pad start` (joy 6).
  The migration keeps joy 6 (one conflict, `input_map`), applies the other eight
  buttons, and keeps `input_quicksave`/`input_quickload` on 192/195.
- `merge::an_untouched_new_binding_takes_the_template_button` /
  `a_new_binding_the_user_rebound_is_kept_and_reported` /
  `a_keyboard_change_to_a_new_binding_is_also_kept` /
  `a_user_changed_binding_survives_a_later_template_change`.
- The spec-029 migration test now migrates v1→v3 and also asserts a spec-034
  binding is added.

**Why the merge changed.** DSDA rewrites the whole file (all bindings) when it
saves, and template v2 owned no binding, so a v2→v3 baseline has *nothing* for
these keys while the active file has DSDA's default. Without rule 2 every new
binding would have been reported as a conflict and the owner's layout would never
apply on an existing install. Comparing the active value with the researched
`upstream` default distinguishes "untouched" from "rebound"; a rebound binding
(including a keyboard-only change) is kept and reported exactly as before.

## R3 — conflicts

**D-pad behaviour, stated explicitly.** With the new bindings, D-pad
Up/Down/Left/Right are read like this:

- **With the automap open:** Up/Down zoom the map in/out
  (`input_map_zoomin`/`input_map_zoomout`, press to start, release to stop,
  `am_map.c:1156-1187`/`1244-1256`); Left/Right still run the previous/next
  weapon actions, because the automap has no pad pan binding
  (`m_misc.c:445-448`). Menu bindings do not fire while a level map is open
  unless the game menu is up.
- **Outside the automap (in play):** nothing zooms — those actions exist only in
  the automap responder. Up/Down still do whatever else holds them: DSDA's
  defaults `input_flyup`/`input_flydown` (D-pad, useful in DOOM's fly cheat and
  Hexen flight) and, in a menu, `input_menu_up`/`input_menu_down`. Left/Right
  switch weapons in play and navigate menus when a menu is open.
- **Why there is no conflict to resolve:** DSDA fires every action bound to a
  button, so DDL adds the zoom buttons without removing the menu or fly
  defaults. A menu is offered the event before the game/automap responder
  (`d_main.c:252-259`).

**180° turn: DSDA has one, and it exists in v0.30.1.** It is the `input_reverse`
action (`m_misc.c:401`, default `/` on the keyboard and Right Stick click),
which adds `QUICKREVERSE` = 32768 half-turn units on the first activated tick
(`g_game.c:206`, `g_game.c:554-563`). It is not applied while the strafe modifier
is held, and while a strict-mode demo records DSDA prints "180 key disabled by
strict mode" and does not turn. The owner's table maps it to X, so X is bound
rather than left at DSDA's default.

No requested binding was dropped, so no `ask` was needed. The collisions DSDA's
defaults create, and the choice made:

| Button | Requested | DSDA default it meets | What DSDA does | Choice |
| --- | --- | --- | --- | --- |
| D-pad Up/Down | automap zoom | `input_menu_up/down` (11/12), `input_flyup/flydown` (11/12) | one button drives every matching action; menus are handled before `G_Responder`; zoom only responds while the map is open | keep the defaults, add zoom. D-pad keeps navigating menus and still flies in DOOM's fly cheat/Hexen |
| D-pad Left/Right | prev/next weapon | `input_menu_left/right`, `input_invleft/right` (13/14) | same multi-action behaviour | keep; D-pad still navigates menus. While the automap is open the buttons also switch weapons (automap pan is keyboard-only) |
| B | toggle run | `input_jump` (1) | jump and autorun are separate actions, both fire | keep the default (R2: jump is not named), so B toggles run and still jumps on maps that allow jumping. **Open question for the owner** — see below |
| X | 180° turn | `input_prevweapon` (2) | weight moved | previous weapon moves to D-pad Left, so X has a single action |
| Y | — | `input_nextweapon` (3) | DSDA allows only one pad button per action | next weapon moves to D-pad Right, so Y becomes unbound |
| View | automap | `input_menu_clear` (4) | menu-only (DSDA's controls menu) | keep; in play View only toggles the map |
| L1/R1 | quick save/load | `input_strafe` (9) / `input_use_artifact` (10, Heretic) | — | quick save/load move to L1/R1; the strafe modifier and Heretic artifact keys lose their pad button (keyboard/mouse unchanged) |
| L3/R3 | — | `input_autorun` (7) / `input_reverse` (8) | — | both move; L3/R3 become unbound |
| L2 | — | `input_map` (21) | — | automap moves to View; L2 becomes unbound |

## R4 — guide (A3)

`docs/steam-deck-guide.md` §"Recommended Steam Deck Controller Setup" now shows
DDL's shipped binding for each Deck control (with the changed-from-default cases
called out), explains that one button may carry a menu binding and a play
binding, and states that a user's own rebind in DSDA's Controls menu is kept.

## Verification

- `cargo test -p ddl-core dsda_config` — 60 passed (targeted, allowed anytime).
- `cargo clippy -p ddl-core --all-targets -- -D warnings` — clean.
- A3 full-workspace `fmt --check`, `clippy`, `test` run once at the end (see the
  coordinator's build slot request).
- No network is used by any test; the research fetch was a one-off desk step and
  the sources are pinned by tag/commit in the document.

## Open questions / spec items left open

- **A3 Deck re-check.** E5 must be repeated with template v3: the nine buttons
  above, that Steam's View button really is SDL back, that L1/R1 are read as
  `pad l`/`pad r`, and that D-pad Up/Down zoom feels right. **Needs Deck
  Validation**; `docs/deck-testing.md` (coordinator-owned) should gain a line.
- **B does two things.** Because jump keeps DSDA's B default, B toggles run on
  every press, including when jumping on MBF21 maps with jumping enabled. R2
  says keep defaults for unnamed actions, so this is what the template ships; if
  the owner prefers B to be autorun-only, DDL would have to own `input_jump` and
  move it (to L3, its old authorun slot is now free) — that is a decision for the
  coordinator, not a silent change.
- **`input_profile`.** DSDA has three profiles; DDL writes the same binding to
  all three but does not own `input_profile`, so a player using a non-zero
  profile still gets the layout. No action needed.
- **Proposals for coordinator-owned files** (not edited here): bump the spec-034
  checkboxes and add a `docs/status.md` / `docs/journal.yml` entry; add the E5
  layout re-check to `docs/deck-testing.md`.

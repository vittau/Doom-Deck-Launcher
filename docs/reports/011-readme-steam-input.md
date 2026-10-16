# Spec 011 implementation report

Date: 2026-10-16. Branch: `vittau/spec-011`. Worktree: `spec-011`.

## Result

Added the section **Recommended Steam Deck Controller Setup** to `README.md` and linked it
from the install section. Documentation only: no code and no other files were changed.

The section contains:

1. an introduction that states Steam Input is manual, that DDL never edits `.vdf` files, and
   that Steam UI names are **as of 2026-10, may change**;
2. controller-only steps in Gaming Mode (Library → DDL shortcut → controller icon →
   Controller Settings → **Edit Layout** → assign → gyro/trackpad → save/apply), including how
   to save the result as a named personal layout;
3. **Classic** (§2) and **Modern + Gyro** (§3) tables mapping each Deck control to what
   DSDA-Doom does with it by default;
4. a **double input** explanation (§4): virtual controller versus mouse emulation, why DDL's
   baseline enables `use_game_controller 1` and `use_mouse 1`, one-action-once guidance, and
   DDL's own pointer behaviour (D-009);
5. **sensitivity** guidance (§5): Steam-side gyro/trackpad versus DSDA-side stick/mouse keys,
   and how DDL preserves in-menu changes (spec 005, D-012);
6. the **strict demo-recording** caveat (§6) with a keyboard + mouse emulation layout for demo
   runners;
7. **Not yet verified on hardware** (§7), linking `docs/deck-testing.md` checks **I1–I4**.

## Requirement to verification mapping

| Requirement | Where in `README.md` | Notes |
| --- | --- | --- |
| R1 steps + date | §1; introduction | Controller-only steps; "as of 2026-10, may change" note on Steam names |
| R2 two suggested layouts | §2, §3 | Classic (no gyro) and Modern + Gyro (Gyro To Mouse, right-stick touch activation); every DSDA binding is an upstream default, no invented binding |
| R3 double input | §4 | Virtual controller + mouse, `use_game_controller 1` / `use_mouse 1`, one-action-once, D-009 pointer behaviour |
| R4 sensitivity | §5 | Steam (gyro/trackpad) versus DSDA (stick/mouse keys); spec 005 / D-012 persistence |
| R5 strict demo caveat | §6 | Blocks mixed gamepad+mouse; keyboard + mouse emulation layout instead |
| R6 unverified note | §7 | Child-process handoff, gyro-via-stick-touch, strict behaviour, sensitivity values; links `docs/deck-testing.md` I1–I4 |
| R7 friendly + linked | install section, last paragraph | Links `#recommended-steam-deck-controller-setup`; short tables and numbered steps |

## Acceptance criteria

- **A1 — checked.** Every DSDA-Doom binding key named in the README exists in
  `docs/research/dsda-engine.md` §4. Full list below.
- **A2 — checked.** The section never asks the user to edit `.vdf` files, import a community
  layout, or use a wizard. It explicitly says DDL never edits `.vdf` files and that you do not
  need to browse or import Community Layouts.
- **A3 — left unchecked.** *Needs Deck Validation:* the owner following the steps in Gaming
  Mode.

### A1 — binding keys named, all from research §4

Input action keys (`input_defs` inventory):

`input_forward`, `input_backward`, `input_strafeleft`, `input_straferight`, `input_fire`,
`input_map`, `input_use`, `input_jump`, `input_prevweapon`, `input_nextweapon`, `input_strafe`,
`input_use_artifact` (mentioned as existing but unused in DOOM), `input_autorun`,
`input_reverse`, `input_menu_up`, `input_menu_down`, `input_menu_left`, `input_menu_right`,
`input_menu_enter`, `input_menu_backspace`, `input_menu_escape`, `input_menu_clear`,
`input_weapon1`–`input_weapon9`, `input_toggleweapon`.

Configuration keys used in §5:

`use_game_controller`, `use_mouse`, `right_analog_sensitivity_x`, `right_analog_sensitivity_y`,
`analog_look_acceleration`, `mouse_sensitivity_horiz`, `mouse_sensitivity_mlook`,
`left_analog_deadzone`, `right_analog_deadzone`.

Controller default pairing used in the tables (`game_controller.h` order, research §4):
`A` → `input_use` / `input_menu_enter`, `B` → `input_jump` / `input_menu_backspace`,
`X` → `input_prevweapon`, `Y` → `input_nextweapon`, `LEFTSHOULDER` → `input_strafe`,
`RIGHTSHOULDER` → `input_use_artifact`, `LEFTSTICK` → `input_autorun`,
`RIGHTSTICK` → `input_reverse`, `TRIGGERRIGHT` → `input_fire`, `TRIGGERLEFT` → `input_map`,
`DPAD_*` → `input_menu_*`, `START` → `input_menu_escape`, `BACK` → `input_menu_clear`.

Keyboard defaults used in the §6 keyboard+mouse layout (`'w'`, `'a'`, `'s'`, `'d'`,
`KEYD_RCTRL`/mouse 0, `' '`, `KEYD_TAB`, `KEYD_CAPSLOCK`, `'0'`–`'9'`, arrow keys,
`KEYD_ENTER`, `KEYD_BACKSPACE`, `KEYD_ESCAPE`) are likewise from research §4.

No DSDA-Doom binding, config key, default or complevel was invented. Steam-side names (gyro
modes, trackpad modes, UI labels) are Valve's, not DSDA's; they are listed below.

## Steam UI names and sources

Steam-side wording used in the README, all marked "as of 2026-10, may change" in the section:
controller icon / **Controller Settings**, **Edit Layout**, **Gyro**, **Gyro To Mouse**,
**As Mouse**, gyro activation ("*Choose Gyro Button(s)* / *Gyro Enable Button*" — described
with "something like"), **Trackpads → Right Trackpad**, **Touch Menu**, **Community Layouts**,
and **Export Layout → Save New Layout / New Personal Save**.

Consulted Valve-hosted pages:

- Steamworks — *Steam Input* handbook:
  <https://partner.steamgames.com/doc/features/steam_controller>
- Steamworks — *Getting Started for Players* (Controller Settings, "Controller Configuration",
  "Edit Layout" in practice, Quick Settings for gyro/trackpad):
  <https://partner.steamgames.com/doc/features/steam_controller/getting_started_for_players>
- Steamworks — *Input Source Modes* ("Mouse (aka Absolute Mouse)", "Joystick Camera", etc.):
  <https://partner.steamgames.com/doc/features/steam_controller/input_source_modes>
- Steamworks — *Touch Menus* (used for the optional weapon-selection Touch Menu in §6):
  <https://partner.steamgames.com/doc/features/steam_controller/touch_menus>
- Steam Support — *Steam Deck – Basic Use & Troubleshooting Guide*:
  <https://help.steampowered.com/en/faqs/view/69E3-14AF-9764-4C28>
- Steam Support — *Steam Deck and Steam Machine Desktop: FAQ*:
  <https://help.steampowered.com/en/faqs/view/671A-4453-E8D2-323C>

For the **"Gyro To Mouse"** name specifically: Valve introduced it in a September 2023 Steam
client update as the renovation of the gyro "As Mouse" mode; the release notes are quoted here:
<https://www.gamingonlinux.com/2023/09/valve-overhauled-steam-deck-as-mouse-gyro-option-in-new-beta/>.
The Steamworks pages above describe the mode family and the configurator, but the literal
label is client-side and can change, which is why the README repeats the date caveat.

Note on method: the `help.steampowered.com` FAQ pages are JavaScript-rendered and returned no
readable body to a plain HTTP fetch; they are cited as the pages that exist for the user, while
the Shop/Steamworks pages were readable and carried the concrete wording. Every Steam label in
the README is therefore flagged as changeable rather than asserted as permanent. No community
guide or wiki is cited as authority for a Steam label.

## Honesty and scope

- The section is marked **pre-alpha**: DDL cannot navigate by controller or launch a game yet,
  so the setup is documented as preparation for the release that adds gamepad input, not as a
  working feature today.
- §7 is explicit that the child-process layout handoff, gyro-via-right-stick-touch, the strict
  mixed-input block and the sensitivity values are unverified on hardware.
- The sensitivity and deadzone values DDL provisions are described as upstream DSDA-Doom
  defaults, not Deck tuning (consistent with the baseline template's own warning).
- Strict mode is not to be disabled as a workaround, and DDL adds no flags to force hybrid
  recording, matching research §5 and OVERVIEW §8.2.

## Open questions

- **DSDA-Doom menu labels** for stick/mouse sensitivity are not in the research (research §4
  lists keys, not menu text). The README points to DSDA's **Options** menu and names the keys;
  the exact labels are on the Deck-validation list.
- **Steam gyro activation wording** ("Choose Gyro Button(s)" vs "Gyro Enable Button") differs by
  client version; the README hedges with "something like" and a date caveat.
- **Comfortable values** remain a hardware task; a follow-up can replace the default pointers
  with tested numbers once `docs/deck-testing.md` I1–I4 have results.

## Files changed

- `README.md` — install-section link + the new section.
- `docs/specs/011-readme-steam-input.md` — A1 and A2 ticked (A3 left unchecked).
- `docs/reports/011-readme-steam-input.md` — this report.

No code, no `OVERVIEW.md`, `docs/decisions.md`, `docs/status.md` or `docs/journal.yml` change.

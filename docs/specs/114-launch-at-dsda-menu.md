# 114 — Launch at DSDA-Doom's menu; remove the skill setting

**Status:** Ready · **Depends on:** 003, 009, 018, 026, 031, 075 · **Decision:** D-044
**Origin:** owner, 2026-10-28: pistol start should still open the game's menu instead of the
first map — "tira a opção da dificuldade totalmente. Deixa ir pro menu sempre."

## Context

`LaunchPlan::to_command` (`crates/ddl-core/src/engines/dsda/plan.rs`) always emits
`-skill <profile skill>` and emits `-warp` when a recipe prescribes one. In DSDA-Doom v0.30.1
either flag sets `autostart`, so the engine skips the title/menu and starts the first map
(**Confirmed**: [d_main.c:1995–2000](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1995-L2000),
[d_main.c:1616–1623](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1616-L1623),
[d_main.c:2276–2285](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2276-L2285)).
`-pistolstart` does not autostart: startup copies it into the temporary game modifier
([d_main.c:2219](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2219),
[skill_info.c:170–186](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/skill_info.c#L170-L186)),
and a menu New Game re-reads it through `G_InitNew` → `dsda_UpdateGameSkill` →
`dsda_RefreshGameSkill` ([g_game.c:3083](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L3083),
[skill_info.c:215–241](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/skill_info.c#L215-L241)),
so each map load resets the player ([g_game.c:1229–1232](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L1229-L1232)).
No `catalog/*.toml` sets `compatibility.runtime.warp` today.

## Requirements

- **R1 — Menu launch.** Every ordinary launch — catalog mapset, base game, custom setup, Home
  Continue, and the headless `ddl launch` — emits neither `-skill` nor `-warp`. Pistol start On
  still emits `-pistolstart`; Off emits nothing for it. Every other argument (IWAD, files,
  patches, complevel, `-noautoload`, config/save/data/shot dirs, soundfont and the rest) and its
  order are unchanged. `LaunchPlan` may keep optional skill/warp support for the existing
  recording path, which needs an explicit skill (research §2, §6); ordinary resolution must not
  set them. Keep the one-occurrence-per-flag rule.
- **R2 — No skill setting.** Remove the skill choice from every profile surface (Play setup,
  base-game setup, custom-setup profile, profile summaries on Mapset detail and Home) and from
  `ddl launch` (`--skill` is no longer accepted: report it as an unknown option with the
  existing parser error style; update help text). Summaries keep pistol start, add-ons and extra
  files, e.g. `Pistol start On` / `Add-ons: … · Extra files: …`. Remove the now-unused skill
  strings, properties, focus stops and copy. Focus order, D-pad/Back behavior and layout stay
  coherent where the row disappears; no new explanatory copy (tone rule, spec 033).
- **R3 — Profiles.** The profile model stops carrying and writing a skill. Existing profiles on
  disk that contain `skill`, including the saved-state fixtures guarded by spec 055, still load,
  with the value ignored; writes stay atomic. No migration beyond that (D-042). If the schema
  version must change, explain why in the report.
- **R4 — Recipes.** `ddl catalog lint` rejects any `compatibility.runtime.warp` with a clear
  message pointing to D-044, and the resolver no longer turns a recipe or install-record warp
  into a launch argument. Update the recipe-format comments/docs that mention optional warp
  defaults (e.g. the `auger-zenith.toml` comment) and catalog contributor docs if they mention
  it. Recipe-change detection (`library/change.rs`) must not report a change for this.
- **R5 — Evidence.** Unit tests: `to_command`/resolver argv for a catalog mapset, base game and
  custom setup with pistol start On and Off (no `-skill`, no `-warp`, `-pistolstart` only when
  On); `--skill` rejection; a v1 profile with `skill` loading and re-saving without it; lint
  rejecting warp. Update tests and fixtures that asserted `-skill`/`-warp`. Headless renders of
  Play setup, base-game setup, a custom-setup profile and Mapset detail summary without the
  skill row. This moves production UI nodes, so run the **ordinary test command** from
  `docs/orchestration.md` § Verification before finishing.

## Out of scope

- Writing DSDA-Doom's `default_skill` or any other config key to preselect the menu's skill.
- Demo recording UI, map selection, progress tracking.
- Changes to the DSDA config template, Steam Input or the README's controller guide.

## Acceptance criteria

- [ ] **A1** (R1) — argv tests show no `-skill`/`-warp` for every ordinary launch kind;
  `-pistolstart` present only with pistol start On.
- [ ] **A2** (R2) — no skill row or skill text in the renders; `ddl launch --skill 4` fails as an
  unknown option.
- [ ] **A3** (R3) — fixture profiles with `skill` load; a re-saved profile has no `skill`.
- [ ] **A4** (R4) — lint rejects a recipe with `warp`; the real catalog lints clean.
- [ ] **A5** (R5) — fmt, denied-warning workspace Clippy and the ordinary test command pass.
- [ ] **A6** — DSDA-Doom opens at its title/menu from Gaming Mode; New Game asks for the skill;
  with pistol start On, the second map starts with only the pistol; Continue also opens the
  menu and Load Game works. *(Needs Deck Validation → `docs/deck-testing.md`)*

# 046 — Complevels as DSDA aliases, without a verification status

**Status:** Ready (after 037 and 045)  ·  **Milestone:** M3  ·  **Depends on:** 002, 003, 019, 045, D-030
**Origin:** owner, 2026-10-19, on the "Complevel 2 · recipe recommendation (not yet verified)" line:
"Prefira os aliases" and "Remova essa info de verificação de complevel, vamos pelo bom senso" (D-030).
Facts: `docs/research/dsda-engine.md` §1 — DSDA v0.30.1 accepts `-complevel` words `vanilla`,
`boom` (9), `mbf` (11), `mbf21` (21), case-insensitive; `vanilla` resolves to 4 for TNT/Plutonia,
2 for other commercial (Doom II) games, otherwise 3. The recipe model already parses
`Complevel::Alias` (`catalog/model.rs`).

## Requirements

- **R1** — **Recipes use aliases:** every `catalog/*.toml` whose complevel equals what an alias
  resolves to for its IWAD is rewritten to the alias (2 on Doom II, 3 on Doom/Ultimate Doom, 4 on
  TNT/Plutonia → `"vanilla"`; 9 → `"boom"`; 11 → `"mbf"`; 21 → `"mbf21"`). A number stays only
  for a level no alias names for that IWAD (e.g. 2 on Ultimate Doom). Lint rejects a number that an
  alias names, with a message that suggests the alias; CONTRIBUTING documents the rule and the
  common-sense source order (author's TXT / `COMPLVL` / UMAPINFO, else the port family convention).
- **R2** — **No complevel status:** remove `complevel_status` from the recipe schema, the resolver
  (`blocker.rs` `complevel_label`), tests, CONTRIBUTING, the PR template and every recipe; remove
  the "Proposed complevel…" header comments and the "Proposed complevel N" phrases in recipe
  notes (keep the rest of each note). A recipe that still has the key fails lint with a clear
  message.
- **R3** — **Launch:** the launch plan passes the alias word to `-complevel` when the recipe uses
  one (separate argument, as today) and the number otherwise; the plan's text output (spec 014)
  shows both, e.g. `complevel: vanilla (2)`. Base games (D-027) launch with `vanilla`. Profile and
  custom-setup complevel overrides keep working (numbers or aliases).
- **R4** — **UI:** wherever a complevel is shown, show the alias name with its resolved number,
  e.g. "Complevel: Vanilla (2)", "Boom (9)", "MBF21 (21)", or the bare number when there is no
  alias. No verification wording (spec 045 already removed it).
- **R5** — **Remote catalogs (spec 019):** DDL builds that still require `complevel_status`
  (`v0.0.6` and older) must not break on a bundle without it: they reject such recipes per recipe
  and keep their embedded catalog. Verify this with the bundle acceptance rules and set
  `min_ddl_version` in the bundle if that is the mechanism that keeps old clients safe. Say in the
  report which one applies.

## Acceptance criteria

- [ ] **A1** (R1, R2) — lint tests: alias accepted; number with an alias equivalent rejected with
  the suggestion; number without one accepted; `complevel_status` rejected; every recipe lints.
- [ ] **A2** (R3) — plan tests: alias argv, numeric argv, base game `vanilla`, text output.
- [ ] **A3** (R4) — headless renders of Mapset detail and Play setup show "Vanilla (2)"-style text.
- [ ] **A4** (R5) — a bundle test with an old-schema reader (or the documented
  `min_ddl_version` path), and the report states the outcome.
- [ ] **A5** — fmt, clippy `-D warnings`, tests green; the coordinator runs the Linux e2e.

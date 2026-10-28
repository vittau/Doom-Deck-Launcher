# 116 — Per-mapset progress from DSDA-Doom's stats

**Status:** Ready · **Milestone:** M4 · **Depends on:** 018, 026, 098, 114, 115 · **Decision:** D-045
**Origin:** owner, 2026-10-28, approving the scope proposed in report 115.

## Context

Evidence: [`docs/research/dsda-stats.md`](../research/dsda-stats.md) (Q1–Q5) and
[report 115](../reports/115-dsda-stats-research.md) (proposal, limits, draft requirements, Deck
checks). Read both before starting; every DSDA fact below is cited there at `v0.30.1`.

DSDA-Doom writes `<-data>/dsda_doom_data/<iwad stem>/<pwad stems…>/stats.txt`: a version
line (`1`), a total-kills line, then one row of 15 integers per detected map
(`lump episode map best_skill best_time best_max_time best_nm_time total_exits total_kills
best_kills best_items best_secrets max_kills max_items max_secrets`). The map list is built
from the loaded lumps and covers only PWAD maps once any PWAD map exists. DDL passes a
per-profile `-data` directory (`ProfileDirs::data`).

## Requirements

- **R1 — Path derivation.** In `ddl-core`, derive the exact stats path from a resolved launch
  (catalog mapset, base game, custom setup, each with profile extras) using DSDA's organizer
  rules: `dsda_doom_data`, lowercase canonical IWAD stem, then the stems of up to the first
  eight loaded files ending in `.wad` (case-insensitive) in load order; patches and non-`.wad`
  files do not contribute. Use the same file list the launch would use, not recipe titles.
  Unsupported or ambiguous layouts (anything the research flags as unverified, such as files
  the engine might load differently) yield "unavailable", never a guess.
- **R2 — Safe parser.** Parse untrusted bytes with the research's read contract (Q5): read at
  most 1 MiB (+1 byte to detect overflow), at most 4,096 rows and 512 bytes per line, exactly
  version 1, exactly two header lines and 15 tokens per row, checked integers, `-1` only where
  documented as unknown, skill 0–5, ASCII map IDs of at most 8 bytes, no duplicate IDs, LF or
  CRLF. Any violation rejects the whole file. Open without following a symlink that escapes the
  profile's data directory; a non-regular file is unavailable. Never write, repair or delete
  engine files. No new dependency unless justified in the report.
- **R3 — Engine gate.** Import only for engine versions audited for this format; today only
  DSDA-Doom `0.30.1`. Keep the allowlist next to the engine metadata so a future engine release
  needs an explicit audit to enable it.
- **R4 — Progress rule (D-045).** For the selected profile: denominator = parsed rows minus
  `progress.exclude_maps`; completed = remaining rows with `best_skill > 0`. Expose per map: ID,
  completed, `total_exits`, highest recorded skill (none when 0). States: *available*,
  *no stats yet* (file missing — never shown as 0 completed), *unavailable* (unreadable,
  invalid, unsupported engine, ambiguous layout). Excluded IDs that are not in the file are
  ignored.
- **R5 — Recipe field.** Add an optional `[progress]` table with `exclude_maps` (list of map
  lump IDs, `MAPxx`/`ExMy` style, unique, validated by lint). Because older DDL builds reject
  unknown fields, follow the D-025/D-038/spec 098 capability gate: raise the catalog bundle's
  `min_ddl_version` to the version that ships this field. Add `exclude_maps = ["MAP18"]` to
  `catalog/abandon.toml` (its verification notes record MAP18 as a short ending map) and update
  contributor docs. Do not add exclusions to other recipes in this task; list candidates you
  notice in the report, with evidence.
- **R6 — Refresh.** Read after the DSDA process has exited (the runner already reaps it) and
  when Library or Mapset detail is built; never while a game is running. Reads are cheap and
  bounded; do not hash files or rebuild the catalog snapshot for this. No persistent cache.
- **R7 — UI.** Library cards for installed catalog mapsets, base games and custom setups show
  `N/M maps` for the selected/remembered profile when *available*; nothing otherwise. Mapset
  detail shows a short progress summary and a simple, controller-scrollable per-map table
  (map ID, completed mark, exits, highest skill); *no stats yet* and *unavailable* show one
  plain line each. No time, kills, items or secrets columns; no map titles. Follow
  `docs/design/ui.md` (amber = focus only, tone rule of spec 033, no decorative subtitles).
  Focus and the right stick behave as on other long Mapset detail content.
- **R9 — Progress bar (owner, 2026-10-28).** Next to every `N/M` count — Library card and the
  Mapset detail summary — draw a horizontal bar filled to `N/M` (empty at 0, full at M; no bar
  when there is no count). It is decoration of the count, not a focusable control. Its fill must
  be neither amber (focus only) nor the launch-action red (D-020, D-036, spec 064); pick a
  tone from `docs/design/ui.md` that reads clearly on the card and detail backgrounds at Deck
  LCD distance, and say which in the report.
- **R10 — UV-Max trophy (owner, 2026-10-28; D-045 amendment).** In the per-map table, a map
  shows a small trophy mark with the text `UV-Max` beside its ID when `best_skill >= 4`,
  `max_kills >= 0`, `max_secrets >= 0`, `best_kills >= max_kills` and
  `best_secrets >= max_secrets` (the parser already reads these fields; `-1` totals mean
  unknown → no trophy). No trophy for excluded maps. Draw the mark in Slint (vector/paths or an
  existing bundled asset style), not a new raster download; it must stay legible at 1280×800
  and not be amber or launch red. Expose the rule in `ddl-core` beside the D-045 rule.
- **R11 — Kills % and Secrets % (owner, 2026-10-28; D-045 second amendment).** Add two table
  columns, `Kills` and `Secrets`, each `floor(100 × best ÷ max)` followed by `%`
  (`best_kills`/`max_kills`, `best_secrets`/`max_secrets`). Show `—` when the max is `-1` or
  `0`. Do not cap: a kills value above 100% is shown as computed. Compute in `ddl-core` beside
  the other rules and carry the values in `MapProgress`. Items stay out. Keep every column
  aligned and legible at 1280×800 with the UV-Max mark; shorten headers (e.g. `Skill`) or
  rebalance widths if needed rather than shrinking text below the detail body size.
- **R8 — Evidence.** Unit fixtures for the parser (valid, zero/unknown values, LF/CRLF, short
  and long rows, overflow, version 2, duplicates, oversize, non-ASCII, symlink escape,
  directory instead of file), path derivation (catalog, base game, custom, extras, nine WADs,
  DEH ignored, mixed case), the progress rule (skill 0 vs >0, exclusions) and lint of
  `exclude_maps`. Headless renders: a Library card with progress and its bar, Mapset detail with the
  table and summary bar (short and long lists, scrolled, at least one UV-Max row), *no stats yet*
  and *unavailable*; the tables show the R11 columns. Unit tests for R11: exact, rounding down
  (e.g. 199/200 → 99%), `-1` and `0` maxima → none, kills above 100%. Unit tests for R10: UV with all kills/secrets → trophy; HMP (3), one kill
  or secret short, unknown `-1` totals, a no-secrets map (`max_secrets = 0`) → trophy only when
  the rule holds. This adds production UI
  nodes, so run the ordinary test command from `docs/orchestration.md` § Verification.

## Out of scope

- Writing DSDA files or config keys; `-levelstat`, `-analysis`, splits, demos.
- Map titles (UMAPINFO/MAPINFO/DEHACKED parsing), playtime, last map, resume, cross-profile
  totals, achievements, telemetry.
- Discover cards (not installed) and any network access.

## Acceptance criteria

- [ ] **A1** (R1–R4) — unit tests listed in R8 pass; a fixture `stats.txt` with 18 rows, 12 at
  `best_skill > 0` and Abandon's exclusion yields `12/17` (or the equivalent under the rule).
- [ ] **A2** (R5) — lint accepts Abandon's `exclude_maps` and rejects malformed or duplicate
  IDs; the catalog bundle's `min_ddl_version` is raised; the real catalog lints clean.
- [ ] **A3** (R7) — headless PNGs reviewed by the coordinator and shown to the owner before
  merge (visible UI change).
- [ ] **A4** (R8) — fmt, denied-warning workspace Clippy and the ordinary test command pass.
- [ ] **A5** — on the Deck: after finishing a map, the card and detail update; profiles stay
  separate; adding an extra WAD switches to a separate (initially empty) progress; Flatpak
  writes `stats.txt` where DDL reads it. *(Needs Deck Validation → `docs/deck-testing.md`,
  using the checks listed in report 115.)*

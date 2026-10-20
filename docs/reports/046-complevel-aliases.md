# Spec 046 — Complevels as DSDA aliases, no verification status

## What changed

- **Engine type (`engines/dsda/plan.rs`):** `Complevel` now keeps the DSDA word that names it
  (`ComplevelWord`: `vanilla`, `boom`, `mbf`, `mbf21`) next to the resolved number. New
  `Complevel::from_word`, `word()`, `argument()` (what `-complevel` receives) and `label()` (the
  player text). `to_command` sends the word when there is one and the number otherwise, as its own
  argv token. `describe` and `Display` print `vanilla (2)`; a bare number keeps its old text
  (`7 (Boom compatibility mode)`).
- **Resolver:** `resolve_complevel` builds the level with `from_word` for aliases (`vanilla` still
  resolves by IWAD family, 3 / 2 / 4, D-017). `Summary.complevel` is now the typed level;
  `complevel_status` and `blocker::complevel_label` are gone. The "Will launch" text prints
  `complevel: vanilla (2)`.
- **Base games (D-027):** `BaseGame::complevel()` returns the `vanilla` level, so a base game
  launches with `-complevel vanilla`. `ddl doctor` prints `complevel vanilla (2)`.
- **Recipe schema:** `complevel_status` and `EvidenceStatus` are removed from `Compatibility`.
  `Game::vanilla_complevel()` and `ComplevelAlias::naming(number, game)` say which alias names a
  number on an IWAD.
- **Lint:** a number an alias names on the recipe's `base_game` is rejected with
  `complevel 2 is what the alias `vanilla` means on doom2` and the hint
  `write complevel = "vanilla" instead of 2`. A recipe that still has `complevel_status` fails with
  `` `complevel_status` is no longer part of the recipe schema (D-030) `` and the hint
  `delete the `complevel_status` line` — in `ddl catalog lint` (with the line number) and in the
  embedded/remote loader's skip reason.
- **Catalog (89 recipes):** rewritten by script. 81 → `"vanilla"` (79 were 2 on Doom II, 2 were
  3 on Doom), 4 → `"boom"`, 2 → `"mbf"`, 2 → `"mbf21"` (one was already the word). No number is
  left. Removed every `complevel_status` line and the `# Proposed complevel…` / `# Confirmed
  complevel…` header comment (35 recipes carried the `Confirmed` variant; it is the same status
  label, so it went too). In the notes, `Proposed complevel 2 (…)` and `Confirmed complevel 2 (…)`
  became `Complevel vanilla (…)`, `Proposed complevel 9 (Boom)` became `Complevel boom`, and the rest
  of each note is untouched. Two sentences were reworded by hand: the MBF21 one ("…the recipe stays
  Proposed until Deck-tested per CONTRIBUTING" → "Complevel mbf21 is the authors' own
  declaration.") and the header comment of `eviternity-ii.toml`.
- **UI:** Play setup shows `Complevel: Vanilla (3)` (`Complevel: {}` with `Complevel::label()`);
  the legacy shell's mapset detail shows `Complevel: Vanilla (3)` / `Vanilla (2)`; base game detail
  and card show `Complevel: Vanilla (2)`. The words "recipe recommendation" are gone.
- **Docs and CI:** `CONTRIBUTING.md` (schema row, new "Choosing the complevel" section with the
  source order, template and annotated example, evidence step), the PR template (asks which source
  gave the complevel), and `.github/workflows/e2e.yml` (the dry-run check accepted only a numeric
  `-complevel`; it now also accepts the four words — this file is outside the listed ownership but
  would fail the coordinator's Linux e2e otherwise).

## Acceptance criteria

| | How it was verified |
| --- | --- |
| **A1** | `catalog/tests.rs`: `a_number_an_alias_names_is_rejected_with_the_alias_as_the_hint` (2/3/4 per IWAD and 9/11/21), `an_alias_or_a_number_no_alias_names_is_accepted` (aliases; 2 on Doom, 4 on Doom II, 7), `the_removed_complevel_status_key_fails_lint_with_a_clear_message` (lint line + message + hint, and the loader skip), `every_catalog_recipe_uses_an_alias_whenever_one_names_its_level`, plus the existing every-recipe lint tests over `catalog/`. |
| **A2** | `plan.rs`: `an_alias_is_sent_as_a_word_and_a_number_as_a_number` (argv `vanilla`, `boom`, `7`; describe `complevel: vanilla (2)`), `a_word_names_its_level_and_keeps_the_resolved_number`; `resolver/tests.rs`: `vanilla` per IWAD reaches argv as the word and the summary snapshot reads `complevel: vanilla (3)`; `base_games.rs` and `cli_exec.rs`: base game argv `-complevel vanilla`. |
| **A3** | `headless::shell_navigation_and_focus_render_headlessly` asserts the mapset detail node `Complevel: Vanilla (3)`; `headless::journeys::install_progress_and_play_setup` asserts the Play setup label `Complevel: Vanilla (3)`. Both render with Slint's software renderer into `target/headless`; no window opened. |
| **A4** | See below. |
| **A5** | See "Final checks". |

## R5 — which mechanism keeps old clients safe

**The per-recipe drop applies; `min_ddl_version` is not needed and stays at `0.0.4`.**

- A v0.0.6 client requires `complevel_status`, so every recipe of the new catalog fails to parse
  there (`missing field `complevel_status``) and is dropped alone by `Catalog::from_sources`.
  The bundle itself passes the whole-document gate (schema 1, `min_ddl_version` 0.0.4 ≤ 0.0.6,
  serial newer), so `accept_bundle` returns a catalog with zero recipes. The app then logs
  "catalog bundle carried no usable recipe; keeping the current catalog" and returns
  `CatalogOutcome::Current` without touching the cache (`services.rs`), and `store_cache` would
  refuse an empty bundle anyway. The old client keeps its embedded catalog and nothing breaks.
- **Verified against the real old code:** I exported tag `v0.0.6-local` to a scratch directory,
  built a bundle from this branch with `ddl catalog bundle` (89 recipes) and ran v0.0.6's own
  `parse_bundle` + `accept_bundle`: it parsed, passed the gate, accepted **0** recipes and dropped
  **89**, each with `missing field `complevel_status``.
- **Permanent test:** `bundle.rs` `an_old_schema_reader_drops_every_recipe_of_a_bundle_without_complevel_status`
  models the v0.0.6 `[compatibility]` table (required `complevel_status`, unknown keys refused),
  checks all recipes of a bundle built from `catalog/` fail it, that a 0.0.6 client still passes
  `accept_bundle`'s gate, and that today's reader keeps every recipe.
- Trade-off, for the coordinator: raising `min_ddl_version` (for example to the release that
  ships this) would turn the silent "keep embedded" into an explicit `TooNew` for old clients, but
  it also blocks every client built from `main` before the version bump, and
  `DEFAULT_MIN_DDL_VERSION` is documented as raised only when a bundle needs a newer client. I kept
  it because the per-recipe path is proven safe. Say if you want it raised with the release.

## Final checks

Run on the merged tree (`main` at `aa315e4` merged), with `set -o pipefail`:

- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo test --workspace`: green (ddl 142, ddl-core 437, integration and doc tests all pass). The
  first run failed once in `crates/ddl/tests/catalog_cli.rs`, which expected the `invalid-hash`
  fixture's error on line 42; removing the fixture's two status lines moved it to 40, so I updated
  the test and reran the whole workspace.
- The coordinator still runs the Linux e2e (A5).

## New dependencies

None.

## Open questions and items left open

- **Custom-setup complevel** still takes `auto` or a number (`--complevel auto|<n>`); the stored
  record stays a number. The spec says overrides "keep working (numbers or aliases)", but there is
  no alias override today, and `vanilla` cannot resolve to a number without the IWAD, which the
  CLI parser does not know. Accepting `boom`/`mbf`/`mbf21` would be small; `vanilla` needs the
  identified game at plan time. Not done without your word.
- **Older research notes** (`docs/research/reference-*.md`, `idgames-browse.md`) still describe
  `complevel_status`; they are dated evidence records, so I left them.
- **Coordinator-owned files** that still describe a "Proposed" complevel and should be edited by
  the coordinator: `docs/decisions.md` D-016 ("complevel 2 (Proposed until Deck validation)"),
  `docs/status.md` (Eviternity row), `docs/journal.yml` (older entries), `docs/deck-testing.md`
  (M4 "complevel still Proposed"). Suggest marking spec 046 done in `docs/status.md`.
- `docs/design/visual-v2/render.py` still draws the old "recipe recommendation" line in a design
  mock; it is a design asset and not part of the build.

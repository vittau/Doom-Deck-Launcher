# Spec 009 implementation report

Date: 2026-10-16. Branch: `vittau/spec-009` (worktree `spec-009`).

## Result

Implemented the end-to-end core path from a catalog recipe and a player's
choices to installed files and a validated `LaunchPlan`:

- `ddl_core::library` — `Installer` installs a recipe's selected components
  through spec 006's fetcher/extractor, re-uses cached packages, records an
  atomic, versioned `InstallManifest`, and imports user-supplied files by
  SHA-256 (`matched`/`unknown`, never rejecting an unknown hash).
- `ddl_core::profiles` — `Profile` (JSON, versioned, atomic) with enabled
  components, skill, pistol start, optional IWAD-variant override, last played
  time and spec 005's per-profile `-assign` overrides, plus `ProfileStore` and
  the per-profile `save`/`data`/`shots`/`run` directories (D-011).
- `ddl_core::resolver` — `resolve(recipe, profile, iwads, manifest, engine,
  config) -> Result<Resolution, Blocker>` produces the ordered `-file`/`-deh`
  lists, the canonical IWAD symlink path, the numeric complevel resolved from
  the identified IWAD family, `-warp`, `-pistolstart`, the config path and
  `-assign` overrides, and a `Summary` for the Play setup panel.
- Catalog loading embeds `catalog/` at build time and validates every recipe
  with the 002 model, skipping bad ones with a logged reason.
- `ddl catalog lint --fetch` now runs on spec 006's fetcher (R8).

All three reference recipes drive the resolver tests; the two end-to-end
install tests use a loopback HTTP server and generated archives, so no test
touches the real network. No visible windows were opened and no screen was
captured.

## Requirement to verification mapping

| Requirement | Verification |
| --- | --- |
| R1 installer (fetcher, members, keep, cache reuse, atomic placement, no user-supplied download) | `library::tests::installs_main_only_then_adds_the_addon_without_redownloading` (local server, nested add-on ZIP, one download then a cache hit); `installing_a_user_supplied_component_never_downloads_it` |
| R2 install manifest (versioned, atomic, recipe identity, packages/members with hashes + paths, install time) | `library::manifest::tests::round_trips_and_looks_up_members`, `missing_file_is_none_and_bad_schema_is_rejected`; the A1 test reloads and compares the record; `library::tests::a_changed_recipe_is_refused_rather_than_silently_updating` |
| R3 user-supplied import (copy, SHA-256, matched/unknown, never reject unknown) | `library::tests::imports_known_and_unknown_user_files_and_never_rejects_unknown_hashes` (known → matched, unknown → accepted; not-user-supplied and unknown component are typed errors) |
| R4 profiles (fields, exclusive groups/replacements, per-profile dirs, JSON) | `profiles::profile::tests::{a_default_profile_validates..., invalid_skill_target_and_combination..., toggling_components...}`; `profiles::store::tests::{save_load_list_and_delete_round_trip, unsafe_ids_and_bad_schema...}` (runtime dirs exist after save) |
| R5 resolver + typed blockers | `resolver::tests::{auger_zenith_order_is_main_then_addon_and_exclusive_groups_are_enforced, sigil_compat_replaces_main_and_the_missing_soundtrack_blocks_only_when_enabled, missing_iwad_names_the_game_and_vanilla_resolves_per_family, blockers_cover_not_installed_engine_missing_integrity_and_bad_overrides, warp_pistol_start_and_assign_overrides_reach_the_plan, prepare_creates_directories_and_the_canonical_link_and_then_validates}` |
| R6 "Will launch" summary | `resolver::tests::summary_snapshot_for_the_three_reference_recipes`; `resolver::blocker::tests::summary_renders_the_numbered_load_order_and_status` |
| R7 catalog loading (embedded, validated, skip + reason) | `catalog::tests::{embedded_catalog_loads_the_reference_recipes_without_skips, invalid_recipes_are_skipped_with_a_logged_reason_never_crashing, duplicate_recipe_ids_keep_the_first_file}` |
| R8 lint `--fetch` on the shared fetcher | `catalog::tests::{https_fetch_downloads_through_the_shared_006_pipeline, lint_fetch_calls_the_injected_boundary_only_when_requested, injected_fetch_verifies_declared_bytes_and_reports_failures}` |
| A6 fmt/clippy/tests, offline | `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (tails below); the CLI offline lint tests still pass |

Reference-recipe coverage: the three shipped recipes are the inputs of the
summary snapshot and of the resolver selection tests; the AUGER;ZENITH and
SIGIL component shapes (add-on ordering, exclusive group, replacement,
user-supplied soundtrack) are exercised directly.

## Dependencies

One new direct dependency, pinned in the workspace `[workspace.dependencies]`:

| Added to | Crate | Version | Reason |
| --- | --- | --- | --- |
| `ddl-core` | `tracing` | 0.1.44 | R7 requires invalid recipes to be *skipped with a logged reason*; `tracing` is already a workspace dependency used by `crates/ddl` for structured diagnostics. |

Everything else reuses existing pins: `serde`/`serde_json`/`toml` (state and
recipes), `sha2` (member/import hashing), `ureq` + `zip` (via `downloads`),
`semver`/`url` (via `catalog`). `crates/ddl-core/build.rs` uses only `std`. No
new UI, process, or async dependency was added; `ddl-core` remains UI-free.

## Decisions and interpretations (for review)

1. **Embedding the catalog (R7).** The build script scans `catalog/` and emits
   `EMBEDDED_HOSTS` / `EMBEDDED_RECIPES` with `include_str!`, so the recipes
   ship inside the binary (a release tarball cannot lose them) and adding a
   mapset stays a one-file pull request (D-010) with no Rust edit. A test fails
   if the shipped catalog is missing a recipe or is not skip-free.
2. **Recipe identity (R2).** The manifest records the SHA-256 of the recipe's
   canonical TOML form (`toml::to_string`), not the on-disk bytes. This is the
   recipe's *content* identity: it ignores comments/whitespace but changes
   whenever the recipe's meaning changes, which is exactly the "a later catalog
   update never silently changes an installed manifest" behaviour (OVERVIEW §7).
   Re-installing over a changed recipe is refused with `RecipeChanged`; a
   non-atomic catalog update can never mutate an existing install, because
   resolution reads the manifest, not the catalog.
3. **`resolve` signature.** `R5` names an "iwad registry"; the canonical IWAD
   path (007) additionally needs DDL's data root. The second parameter is
   therefore `Iwads { data_dir, registry }` (a small bundle), keeping the
   remaining parameters as written.
4. **`resolve` stays pure (coordinator guidance).** It reads the registry,
   manifest and files, verifies integrity and computes the plan, but writes
   nothing. `Resolution::prepare()` creates the per-profile directories
   (`LaunchPlan::prepare_dirs`, spec 003) and the canonical IWAD symlink; its
   filesystem error is an `io::Error`, so `Blocker` keeps exactly the six
   user-explainable variants. `Resolution::validate()` passes after `prepare()`.
5. **Profile `overrides`.** Confirmed with the coordinator: `Profile` carries an
   optional `overrides` map (BTreeMap) that the resolver renders as spec 005
   `-assign` tokens, restricted to `ALLOWED_KEYS`; an unknown key is
   `IncompatibleChoice`.
6. **Engine handle.** Spec 004 is in progress in another worktree, so the
   resolver defines a minimal `Engine { executable, version }`; `None` or an
   unusable executable is `EngineMissing`. Spec 010 adapts spec 004's installed
   state to it.
7. **Complevel aliases.** `vanilla` is resolved from the identified family
   (D-017: doom 3, doom2 2, tnt/plutonia 4; nerve 2). The word aliases follow
   upstream (`docs/research/dsda-engine.md` §1): `boom` → 9, `mbf` → 11,
   `mbf21` → 21. A numeric complevel is passed through `Complevel::new`, so the
   placeholders 18–20 can never reach a plan.
8. **Install layout.** Downloaded members go to
   `<data>/library/<id>/<release>/<package id>/<member path tail>`; imports to
   `.../imports/<component id>/<file name>`. A fresh install is assembled in a
   staging directory under `<data>/library/` and moved into place with one
   `rename`; an update extracts only the missing members. Keep members travel
   with their package so permission text survives (spec 002 / D-010).
9. **Integrity.** The installer verifies member size and SHA-256 through spec
   006 and records them; the resolver re-verifies each launched member and the
   IWAD target before returning a plan (`IntegrityFailed`/`MissingIwad`).

## Safety notes

- All downloads go through spec 006 (HTTPS-only, allowlist, redirect limit,
  bounded size, SHA-256, cache keyed by hash); extraction is the hardened
  explicit-member extractor (no traversal/symlink/zip-bomb, no exec bits).
- Imports are bounded (256 MiB), copied to a temp file, permission-stripped,
  and renamed into place; the empty file and over-large cases are typed errors.
- Manifest and profile reads are bounded; writes are atomic (temp + fsync +
  rename) and versioned. Nothing downloaded is ever executed.

## Open questions / items left open

- **Reinstall after a catalog change.** A changed recipe is refused with
  `RecipeChanged`; there is deliberately no "force update" API in this spec.
  Deciding the reinstall/update UX (and whether to keep the old install for
  rollback) belongs with the UI spec (010).
- **Paid SIGIL/SIGIL II soundtracks** remain user-supplied and unvalidated
  (identity/replacement semantics), exactly as `reference-sigil.md` records;
  import accepts an unknown file and reports it as unmatched.
- **Complevels, episode menus, load order and controller behaviour** still need
  physical Deck validation; nothing here promotes the `Proposed` complevels.
- **`RecipeChanged` identity** could be bound to the exact file bytes instead of
  the canonical content form if reviewers prefer the literal wording of R2; the
  chosen form is documented above.
- No implementation acceptance item remains open; A1–A6 and all four tasks are
  checked.

## Final command output tails

`cargo fmt --all --check`:

```text
(no output; exit 0)
```

`cargo clippy --workspace --all-targets -- -D warnings` (tail):

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.47s
```

`cargo test --workspace` (result lines):

```text
     Running unittests src/main.rs (target/debug/deps/ddl-...)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s
     Running tests/catalog_cli.rs (target/debug/deps/catalog_cli-...)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.85s
     Running unittests src/lib.rs (target/debug/deps/ddl_core-...)
test result: ok. 221 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
     Running unittests src/bin/ddl-launch-probe.rs (target/debug/deps/ddl_launch_probe-...)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/dsda_launch.rs (target/debug/deps/dsda_launch-...)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
   Doc-tests ddl_core
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Delivery

- `12395a5` — Embed the catalog and run lint fetch on the shared downloader.
- `64b2b3f` — Add mapset install, profiles and launch resolution.
- Spec checkboxes ticked and this report follow.

No coordinator-owned document was edited; proposed status/decision updates (if
any) are in the interpretations above.

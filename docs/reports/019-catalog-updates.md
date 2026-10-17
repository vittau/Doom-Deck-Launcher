# Spec 019 — Catalog updates without a DDL release

Implemented R1–R6 (A1–A3, A5) with loopback HTTP, a fake clock, scratch XDG roots and off-screen
Slint software rendering. No visible window was opened and no screen was captured. No new
dependencies were added; no engine flags, DSDA config keys, complevels or recipes were invented.
The `catalog` branch is **not** pushed by this worker (A4 stays with the coordinator, who verifies
the first CI run after merge).

## Behavior and storage

- **Bundle (R1).** `ddl catalog bundle --out <file>` builds one JSON document with
  `bundle_schema = 1`, `catalog_serial`, `source_commit`, `min_ddl_version` and every recipe plus
  `tags.toml`/`hosts.toml` as their original TOML text, keyed by file name (`catalog/bundle.rs`,
  `Bundle`). It runs `lint_paths` first and **refuses to write on any lint error** (the diagnostics
  go to stderr). `--serial`, `--commit` and `--min-ddl-version` are optional; they default to
  `DDL_CATALOG_SERIAL` (then `SOURCE_DATE_EPOCH`), `DDL_SOURCE_COMMIT` (then `GITHUB_SHA`) and
  `DEFAULT_MIN_DDL_VERSION` — a deliberate, stable constant (`0.0.4`), **not** the building binary's
  version, so a `Cargo.toml` bump on `main` cannot make every installed DDL reject every bundle
  (spec 019 review fix).
- **The serial (R1/R3).** `build.rs` bakes `EMBEDDED_SERIAL` from `DDL_CATALOG_SERIAL` (then
  `SOURCE_DATE_EPOCH`), the same UTC Unix commit time CI passes to `catalog bundle`. It is offline
  and deterministic; when neither variable is set (a local/developer build) the serial is `0`,
  meaning "unknown", so every real published serial is newer — the safe direction for a development
  binary. `cargo:rerun-if-env-changed` keeps a changed serial from being cached.
- **Fetch and validate (R3).** `catalog/remote.rs` fetches
  `https://raw.githubusercontent.com/vittau/Doom-Deck-Launcher/catalog/catalog.json` through spec
  006's download stack. The stack had no way to fetch a file whose SHA-256 DDL cannot know ahead of
  time, so `downloads::Fetcher` gained one **additive** method, `download_document` (and
  `DownloadRequest::unpinned`): identical HTTPS-only, allowlist-on-every-hop, redirect-limit,
  size-scaled-deadline and hard-cap policy, but no pin and no cache lookup. The bundle is bounded to
  2 MiB and stored in the content-addressed cache like any transfer.
  Whole-document checks reject the bundle entire: unknown `bundle_schema`, `min_ddl_version` newer
  than the running DDL, `catalog_serial` not newer, malformed JSON, oversize, an unsafe file name.
  Per-recipe checks reuse exactly CI's `Catalog::from_sources`, with **hosts taken from the embedded
  `hosts.toml`** (the bundle's own `hosts.toml` is recorded for provenance and otherwise ignored), so
  a remote recipe naming a host the binary does not allow is dropped with a diagnostic, not
  accepted. The tag vocabulary travels with the bundle, like CI.
- **Storage and precedence (R4).** The accepted bundle is written atomically (temp file, `fsync`,
  rename, mode `0600`) to `<XDG data>/ddl/catalog/catalog.json` as `{ fetched_at, bundle }`, holding
  only the accepted recipes. On start `Services::open` rebuilds a `Catalog` from it with the embedded
  hosts and loads the newer of embedded vs cached by serial. A missing, oversized, corrupt or
  recipe-less cache is ignored and logged, never fatal. The fetch allowlist is captured when the app
  opens, so accepting a bundle can never widen it.
- **When (R5).** After `Command::Startup` finishes and the UI is up, the runtime dispatches a
  background `CatalogCheck { automatic: true }` when automatic updates are on and the last attempt is
  at least 24 h old (`catalog_check_due`, a pure function taking `now`). The attempt is recorded
  **before** the network, so a failure still throttles; `execute` swallows the error for automatic
  checks (silent on an offline Deck) and surfaces it (`Problem::Catalog`) for the manual action.
  Settings → Engine now shows `Catalog: <serial date> (built-in | updated)`, a
  **Check for catalog updates** button and an **Automatic catalog updates** row (default On),
  persisted in `app.json` as two additive fields (`catalog_auto_update`, `catalog_last_check`); the
  file stays schema 2 and older files load through serde defaults. `ddl catalog status` prints the
  catalog in use without touching the network; `ddl catalog update` runs the manual check.
- **New recipes without art (R6).** A bundle carries TOML only, never images, so a recipe that
  arrives by bundle has no bundled screenshot and takes the spec 013 fallback cover
  (`art()` returns the fallback index for an unknown id), which ships only with releases.

## Requirement → verification

| Item | Evidence |
| --- | --- |
| R1 / A1 | `build_refuses_a_catalog_that_does_not_lint`, `build_collects_recipes_hosts_and_tags`, `a_bundle_round_trips_through_json`, `the_default_min_version_is_a_stable_semver`, `catalog_bundle_writes_a_lint_clean_bundle_and_refuses_a_bad_one` (real binary writes a bundle, uses the default constant, and an empty catalog never produces a file). |
| R2 / A4 | `ci.yml` `catalog-bundle`: push-to-`main` or a manual run on `main`, `concurrency` serialises the job (no force push race), job-scoped `contents: write`, `catalog/**` diff gate plus the first-publish and manual-run exceptions, orphan `catalog` branch with ordinary commits, `git push origin HEAD:catalog` (never `--force`). `actionlint` clean. First upload is the coordinator's after merge (a manual `workflow_dispatch` run on `main` publishes it immediately). |
| R3 / A1 | Loopback fake (`the_real_fetch_runs_through_the_download_stack`) and injected-bytes fakes: `a_valid_bundle_is_accepted_and_a_stale_one_is_not`, `whole_document_rejections_are_distinct`, `parse_rejects_malformed_and_oversized_documents`, `malformed_unknown_schema_and_oversize_are_rejected`, `a_too_new_min_version_is_rejected`, `accept_keeps_good_recipes_and_drops_disallowed_hosts`, `accept_drops_a_lint_failure_and_keeps_the_rest`, `a_host_outside_the_allowlist_never_reaches_the_recipe_check`; `a_recipe_naming_a_disallowed_host_is_dropped_not_accepted`. |
| R4 / A2 | `cache_round_trips_and_is_ignored_when_corrupt`, `store_cache_is_atomic_and_leaves_only_the_final_file`, `an_empty_bundle_is_never_cached`, `a_newer_bundle_is_accepted_cached_and_loaded_next_start` (accepted → cached → loaded next start), `a_corrupt_or_older_cache_never_replaces_the_embedded_catalog`. |
| R5 / A3 | `throttle_is_due_when_never_checked_or_older_than_a_day` (pure, explicit clock), `automatic_checks_are_throttled_opted_out_and_manual_always_runs` (seeded clock: throttled, opted out, due after 24 h, manual always), `an_automatic_failure_is_silent_and_still_throttles`, `catalog_status_and_update_report_and_persist_the_catalog` and `catalog_bundle_and_status_run_headless` (real-binary CLI), `catalog_status_and_update_parse` / `catalog_bundle_parses_its_options_and_requires_out`; headless Settings rows asserted in the production journey (below). |
| R6 | `art()` fallback index for unknown ids; no image ever enters a bundle (`Bundle` holds TOML text only); `013-discover-fallback.png` renders in the existing journey. |
| A5 | Command tails below; `actionlint` clean. |

## Headless visual review

Inspected the off-screen 1280×800 PNGs under `target/headless/` (the journey asserts a clean idle
frame, so no continuous animation):

- `010-settings.png`: Settings → Engine now carries, below the engine block, the label
  `Catalog: built-in` (serial `0` in a developer build), the **Check for catalog updates** button and
  the **Automatic catalog updates** row reading `On`. The engine panel grew to 700 px so all three
  fit inside its border; the existing engine rows, the amber focus ring and the hint bar are
  unchanged.

The production journey asserts the rows exist, that the summary says `built-in`, and that toggling
id 74 flips the row `On` → `Off` → `On` and the persisted snapshot with it.

## Maintainer notes — the `catalog` branch (R2)

- The branch is an **orphan** (`catalog` shares no history with `main`) holding exactly one file,
  `catalog.json`. It is created on the first publish and updated afterwards by ordinary commits on
  top of the previous head; it is **never force-pushed**, so a rejected non-fast-forward push fails
  CI loudly instead of losing history.
- `catalog-bundle` runs for a push to `main` that changes `catalog/**`, for a manual
  `workflow_dispatch` run on `main`, and for the **first publish** — when the remote `catalog`
  branch does not exist yet, a push that does not touch `catalog/**` still publishes (checked with
  `git ls-remote` in the detect step), so merging spec 019 creates the branch immediately instead of
  waiting for the next recipe change. It has its own `concurrency` group
  (`cancel-in-progress: false`) so two publishes cannot race. `contents: write` is scoped to that
  job; the workflow default stays `contents: read`, and PRs, tags and other refs are skipped by the
  job's `if`.
- `min_ddl_version` is the deliberate `DEFAULT_MIN_DDL_VERSION` constant (`0.0.4`), not the building
  binary's version: raise it only when a bundle needs a newer client (a new bundle/recipe schema),
  because a recipe-level incompatibility is already dropped per recipe with a diagnostic. Bumping
  `Cargo.toml` alone must never invalidate every published bundle.
- The app reads
  `https://raw.githubusercontent.com/vittau/Doom-Deck-Launcher/catalog/catalog.json` — no GitHub
  release is involved. To inspect: `git fetch origin catalog && git show FETCH_HEAD:catalog.json`,
  or `gh api repos/vittau/Doom-Deck-Launcher/contents/catalog.json?ref=catalog`.
- The serial is the source commit's `git log -1 --format=%ct`. `release.yml` now exports
  `DDL_CATALOG_SERIAL` in its build step, so a released binary's `EMBEDDED_SERIAL` is the same value
  the `catalog` bundle built from that commit carries.

## Final verification

All commands exited 0 on macOS arm64 after merging `main` (`aca466b`, spec 023 phase A) with no
conflicts; no external-network or physical Deck validation was run. `cargo fmt --all --check` prints
nothing; clippy is clean.

```text
cargo fmt --all --check
  exit 0 (no output)

cargo clippy --workspace --all-targets -- -D warnings
    Checking ddl-core v0.0.4 (...)
    Checking ddl v0.0.4 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.16s

actionlint .github/workflows/ci.yml .github/workflows/release.yml
  exit 0 (no output)

cargo test --workspace
  ddl unit:       test result: ok. 90 passed; 0 failed
  catalog_cli:    test result: ok. 3 passed; 0 failed
  cli_help:       test result: ok. 2 passed; 0 failed
  logo_assets:    test result: ok. 2 passed; 0 failed
  ui_strings:     test result: ok. 1 passed; 0 failed
  ddl-core unit:  test result: ok. 343 passed; 0 failed
  dsda_launch:    test result: ok. 16 passed; 0 failed
  Doc-tests:      test result: ok. 3 passed; 0 failed
```

460 tests passed. Merged `main` at `aca466b`; spec 023 phase A's recipe `screenshots` field, lint
rules and build-time asset indexing coexist with the bundle code (separate modules
`catalog/bundle.rs` and `catalog/remote.rs`, and a distinct `EMBEDDED_SERIAL` block in `build.rs`).

## Review fixes (2026-10-17)

1. **First publish + `workflow_dispatch`.** `catalog-bundle`'s detect step treats a missing remote
   `catalog` branch (`git ls-remote --exit-code --heads origin catalog`) as "changed", so merging
   this branch creates the branch even though it does not touch `catalog/**`; `ci.yml` gained a
   `workflow_dispatch` trigger and the job's `if` accepts a manual run, still only on
   `refs/heads/main`, with the same job-scoped `contents: write`, `catalog-bundle` concurrency group
   and no-force push. PRs, tags and other refs cannot write.
2. **`min_ddl_version` is deliberate.** `DEFAULT_MIN_DDL_VERSION = "0.0.4"` now sits next to
   `BUNDLE_SCHEMA` and is `ddl catalog bundle`'s default (`--min-ddl-version` still overrides); a
   `Cargo.toml` bump can no longer make installed DDLs reject every bundle. Tests:
   `the_default_min_version_is_a_stable_semver`, and the CLI test asserts the default constant is
   written when the flag is omitted.
3. **Release serial.** `release.yml`'s build step now exports
   `DDL_CATALOG_SERIAL="$(git log -1 --format=%ct)"` before `cargo build --release`, so a released
   binary's `EMBEDDED_SERIAL` matches the bundle built from the same commit. `actionlint` clean on
   both workflows.

## Open items

- **A malformed `tags.toml` inside an otherwise valid bundle** degrades to the empty vocabulary:
  tagged recipes are then dropped per-recipe with a diagnostic (the same behaviour CI's
  `Catalog::from_sources` has), rather than rejecting the whole bundle.
- **Safety addition beyond R3's letter:** the check compares `catalog_serial` against
  `max(effective serial, embedded serial)`, so an accepted bundle can never walk the catalog
  backwards; the embedded-vs-cached floor only tightens the spec's "newer than embedded" rule.
- **Raise `DEFAULT_MIN_DDL_VERSION` deliberately, not with the version.** A bundle built today
  declares `min_ddl_version = 0.0.4`; raise the constant only when a bundle needs a newer client
  (a new bundle/recipe schema). `Cargo.toml` bumps must not touch it.
- No implementation questions and no new dependencies. `OVERVIEW.md`, `docs/decisions.md`,
  `docs/status.md`, `docs/journal.yml`, `docs/orchestration.md`, `crates/ddl/ui/**` and the recipes
  were not edited by this worker. A4 (first CI upload on the `catalog` branch) is the coordinator's
  after merge; a manual `workflow_dispatch` run on `main` publishes it immediately.

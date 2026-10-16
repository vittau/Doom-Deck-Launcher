# Spec 004 implementation report

Date: 2026-10-16. Branch: `vittau/spec-004`. Worktree: `spec-004`.

## Result

Implemented the DSDA-Doom engine manager in `ddl-core::engines::dsda`:

- `data/engines.toml` + `reviewed` — the reviewed release list (R1/R9), seeded
  with v0.30.1 (`dsda-doom-0.30.1-linux-x86_64.appimage`, 14,608,888 bytes,
  `b06adf3f…`), each entry carrying the tag, asset, size, SHA-256, a `validated`
  flag, licence, release page and tagged source tree.
- `discovery` — the `ReleaseApi` seam, the real `GithubApi` client (stable query
  string, DDL User-Agent, optional `GITHUB_TOKEN`, 15 s timeout, weak-ETag
  `If-None-Match`), a 24 h-throttled JSON/ETag/`fetched_at` cache, offline
  fallback for 403/429/network errors with an "offline (cached <time>)" note, and
  `Reviewed`/`Unreviewed`/`Unsupported` classification.
- `install` — the staging-based install pipeline: download through spec 006's
  `Fetcher` (size + SHA-256), copy into staging, `chmod 0755`, run spec 003's
  `smoke_test` (`-v`, matching version), and the D-013 FUSE fallback
  (`--appimage-extract` once → `squashfs-root/AppRun`, full tree kept), then an
  atomic directory rename. A failure at any step removes staging and records
  nothing.
- `state` — the versioned, atomically written `installed.json` (path, mode,
  SHA-256, source URL, tag, reviewed flag, origin, install time; active version;
  previous known-good version).
- `manager` — `EngineManager` tying it together: `discover`, `install_reviewed`,
  `install_unreviewed` (consent + digest), `select_engine`, `rollback`, `remove`
  and `register_manual`, all sharing spec 005's `RunGuard`, plus `attribution`
  for the Settings screen (R9).

## Requirement to verification mapping

| Requirement | Verification |
| --- | --- |
| R1 reviewed list | `reviewed::tests::embedded_list_parses_and_matches_the_research`, `lookups_work_by_tag_and_version`, plus the rejection matrix (duplicate tag, missing `v`, unsafe version, non-https URL, zero size, wrong schema, empty list) |
| R2 discovery + cache | `discovery::tests::classifies_all_three_historical_naming_schemes_and_skips_drafts`, `a_fresh_response_is_cached_and_reused_within_the_throttle`, `a_forced_refresh_sends_if_none_match_and_handles_304`, `a_rate_limited_response_falls_back_to_the_cache` (403 and 429), `a_network_error_falls_back_to_the_cache`, `an_offline_error_without_a_cache_is_typed`, `a_corrupt_cache_is_ignored_and_overwritten`, `the_matching_rule_accepts_both_naming_schemes`, `digest_parsing_ignores_other_algorithms`, `the_real_client_parses_status_etag_and_body_against_a_loopback_server` |
| R3 availability + opt-in | classification assertions above; `manager::tests::unreviewed_installs_require_consent_and_a_digest`, `a_null_digest_unreviewed_release_is_refused` |
| R4 install | `manager::tests::installing_a_reviewed_version_downloads_verifies_and_activates`, `a_reviewed_install_by_name_builds_the_upstream_url`, `a_size_or_hash_mismatch_leaves_no_partial_install`, `a_non_fuse_smoke_failure_is_reported_and_leaves_nothing` (all assert no `<version>/` and no `.staging-*` remain) |
| R5 extraction fallback | `manager::tests::a_fuse_smoke_failure_falls_back_to_extraction` (mode `Extracted`, `squashfs-root/AppRun`, tree kept) |
| R6 installed state | `state::tests::save_and_load_round_trip`, `a_missing_file_is_the_empty_state`, `unsupported_schema_and_invalid_json_are_rejected`, `insert_replaces_by_version_and_remove_clears_previous_good` |
| R7 select/rollback/remove + guard | `manager::tests::select_rollback_and_remove_follow_the_rules`, `rollback_without_a_previous_version_is_a_typed_error`, `select_rollback_and_remove_are_refused_while_a_run_is_active` |
| R8 manual path | `manager::tests::register_manual_records_a_manual_engine_that_cannot_be_removed` (absolute path, smoke test, `Mode::Manual`, relative path and duplicate version rejected, never removed) |
| R9 attribution | `manager::tests::attribution_uses_reviewed_then_derived_urls` |

All acceptance criteria A1–A4 are ticked in `docs/specs/004-engine-manager.md`.
A5 (install/launch on the Deck) stays unchecked (Needs Deck Validation).

## Design decisions taken where the spec was silent

- **Null API digest (confirmed with the coordinator).** An `Unreviewed` release
  whose `assets[].digest` is `null` is refused with
  `Error::NoIntegrityValue` ("this DSDA-Doom release has no published checksum;
  DDL cannot verify it"). Spec 006's fetcher always needs an expected SHA-256,
  and D-007 never trusts a binary without a hash.
- **`validated = false` for v0.30.1.** The flag records Deck validation (A5);
  it stays `false` until that hardware test is recorded, so DDL never claims
  validation it has not done. "Reviewed" (offered by default) is presence in the
  list, independent of `validated`.
- **Version directory name** is the tag without the leading `v` (`0.30.1`),
  matching the smoke-test output.
- **Install activates the first engine** when nothing is active yet; later
  installs do not change the active version.
- **Rollback swaps active and previous**, so it is its own inverse.
- **Manual version collision** (`register_manual` with a version already
  installed) is `Error::VersionConflict`; select/remove stay keyed by version.
- **Install is not guarded by the run lock** (R7 lists only select/rollback/
  remove); installing a different version never touches the running one, and
  reinstalling the active version is already refused as `AlreadyInstalled`.

## Cross-area edits (small, non-behavioural)

- `downloads`: `mod test_http` → `pub(crate) mod test_http` so the engine tests
  can reuse the spec 006 loopback HTTP server (test-only visibility).
- `dsda_config`: re-exported `DSDA_DIR` and `recorded_pid` so the engine manager
  points its run guard at the same `<config>/dsda/run.lock` as spec 005.

## Dependencies

No new direct dependencies were added: `serde`, `serde_json`, `toml`, `url`,
`ureq` (custom headers/status/timeout), `sha2` and `tempfile` were already
workspace-pinned and used by earlier specs. `Cargo.toml`/`Cargo.lock` are
unchanged.

## Open questions

- **GPL-2.0-only vs GPL-2.0-or-later** (research `upstream-sources.md` open Q9).
  Spec R9 fixes the value to `GPL-2.0-or-later`; `engines.toml` uses it.
- **Deck validation (A5).** FUSE mounting, the extraction fallback and the real
  AppImage smoke test are still hypotheses; `validated` stays `false`.
- **Manual engine identity** is the version it reports, so two different manual
  executables reporting the same version cannot coexist (documented above).
- The discovery cache stores the raw response body; a future schema change would
  be handled by the `schema_version` field (a mismatched cache is refetched).

## Final command output tails

`cargo fmt --all --check`:

```text
(no output; exit 0)
```

`cargo clippy --workspace --all-targets -- -D warnings` (tail):

```text
    Checking slint v1.18.1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 56.33s
```

`cargo test --workspace` (result lines):

```text
     Running unittests src/main.rs (target/debug/deps/ddl-be5b1682ae575ea9)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
     Running tests/catalog_cli.rs (target/debug/deps/catalog_cli-1634c3115e516dd0)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
     Running unittests src/lib.rs (target/debug/deps/ddl_core-09fdbe8f1766adcd)
test result: ok. 235 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
     Running unittests src/bin/ddl-launch-probe.rs (target/debug/deps/ddl_launch_probe-6de3b7aa200ae46a)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/dsda_launch.rs (target/debug/deps/dsda_launch-ce3a66c9fec089c1)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
   Doc-tests ddl_core
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

No network access, no visible windows and no screen capture were used; every
engine test runs against a loopback HTTP server and shell-script fixtures.

## Delivery

- `287ef5c` — reviewed list and cached release discovery (R1–R3).
- `c6a2ad4` — install pipeline, state and manager (R4–R9).
- Spec checkboxes and this report follow.
- A1–A4 are checked; A5 and the GPL reading remain evidence/hardware follow-ups.

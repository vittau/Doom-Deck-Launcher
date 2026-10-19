# 040 — Resumable downloads — worker report

**Status:** R1–R6 implemented with tests; targeted `fmt`/`clippy`/tests are green. The full
workspace check and the Linux e2e still need the build slot. No new dependencies.

## What changed

### R1 — a pinned partial survives a network error, timeout or cancel

- New `crates/ddl-core/src/downloads/partial.rs`:
  - `<sha256>.part` plus a `<sha256>.part.json` sidecar with `schema_version` (`1`), the
    request URL that served the bytes, the flushed byte count and the validator (`ETag`
    preferred, else `Last-Modified`, used verbatim as `If-Range`). Unknown schema versions and
    damaged sidecars are discarded instead of guessed at.
  - `write_sidecar` writes a temporary file, fsyncs it and renames it over the old sidecar.
    Callers sync the data file first, so the sidecar never describes unflushed bytes.
  - `load` validates the schema, URL, byte count against the file length and the recipe size.
    A partial for another mirror URL is left in place (R4) rather than deleted.
- `fetch.rs` streams a pinned transfer into a `.tmp-*` file first. On a network error, timeout
  or cancel it fsyncs, renames it to `<sha256>.part` and writes the sidecar; on any other
  failure the temporary file is removed. A resumed transfer appends to the existing `.part`
  after truncating any bytes past the recorded count.
- Unpinned documents (the catalog bundle) use a throwaway temporary file and never write or
  keep a partial.

### R2 — resume with `Range`/`If-Range`

- Before a request the fetcher validates the partial, hashes the bytes already on disk to seed
  the SHA-256, and sends `Range: bytes=<n>-` (plus `If-Range` when a validator is stored).
  Both headers are re-sent on every redirect hop, so a redirected download still resumes.
- `206` with a `Content-Range` whose start is `<n>` and whose total agrees with the recipe size
  appends the body and continues the digest from `<n>`. A `200`, `416`, a missing/mismatched
  `Content-Range` or any other 2xx discards the partial and restarts the URL exactly once from
  zero. Host/scheme/redirect rules are applied on every hop exactly as before; the size cap is
  compared with the whole-file byte count.
- The sidecar records the **mirror URL** the fetcher was asked for, not the final redirect
  target. Dropbox's per-request host rotates (spec 038), so resuming to the final host would
  never match a later attempt; the recipe URL does, and redirects again.

### R3 — verification is unchanged

The full file (resumed or fresh) is hashed and compared with the pin. A mismatch deletes
`<sha256>.part` and its sidecar and returns the existing `Error::HashMismatch`. A resumed file
is never trusted without the complete hash.

### R4 — automatic retries

- A transient failure (`Error::Network`, `Error::Stalled`, `Error::TooSlow` or an HTTP `5xx`)
  is retried up to three times on the same URL, waiting 2 s, 5 s and 10 s (`RETRY_DELAYS`).
  Each retry re-reads the partial and resumes when one exists. A cancel never retries.
- After the third retry the next mirror is tried, in the existing order. When the next mirror
  succeeds, the previous mirror's partial is removed because the pin is now complete.
- Retries apply to **pinned install downloads only**. `download_document` (the catalog bundle)
  keeps its own 24 h throttle in `services`: multiplying a failing background check by four
  was neither in the spec's install context nor desirable, and it broke
  `an_automatic_failure_is_silent_and_still_throttles`. This scoping is called out under
  "Open questions".
- Tests inject a sleeper (`Fetcher::with_retry_sleeper`) so they assert the exact waits without
  spending 2/5/10 s.

### R5 — bounded partial cache

`Fetcher::new`/`with_timeouts` runs `partial::cleanup` at startup: partials whose `.part` mtime
is older than 7 days are removed, then the oldest are removed until the total is at most 2 GiB.
Only names that parse as `<sha256>.part` are considered; sidecars go with their data file and
nothing outside `downloads/` is touched.

### R6 — UI

- `Progress` gains `resuming: bool` (and `Progress::new` for the common non-resumed case). It is
  `true` only while the range request is in flight and `false` once the server answered.
- `Fetcher::resumable_bytes(&request)` lets the install layer start its first progress event at
  the bytes already on disk. Both install call sites use it instead of a synthetic zero, so a
  resumed install never jumps back to 0 %.
- `runtime::download_row` is the small progress model (fraction + resuming flag) used by the
  progress screen. The stage row now selects a new `Text::Resuming` (`"Resuming…"`, `@tr` in
  `wiring-strings.slint`) while a resume is in flight. No new screens.

## Verification against the acceptance criteria

### A1 (R1–R4) — fetcher tests, local server only

`cargo test -p ddl-core downloads` → **85 passed, 0 failed** (includes the unchanged spec 038
redirect tests). New tests:

| Test | Case |
| --- | --- |
| `resumes_a_cut_body_on_the_automatic_retry` | cut mid-body, retry sends `bytes=102400-`, appends, completes; progress reports `resuming` at the kept count |
| `a_server_that_ignores_range_restarts_from_zero` | `200` path plus `If-Range` sent; partial gone; progress falls back to a fresh 0 |
| `a_changed_validator_is_not_resumed` | stored `If-Range` sent, changed ETag → `200`, full download |
| `a_bad_content_range_restarts_once_without_range` | wrong `Content-Range` start → discard, one un-ranged retry |
| `a_416_restarts_once_without_range` | `416` → discard, one un-ranged retry |
| `a_hash_mismatch_after_resume_deletes_the_partial` | wrong prefix + valid remainder → `HashMismatch`, no partial/sidecar/cache file |
| `progress_is_monotonic_and_cancel_keeps_a_pinned_partial` | cancel keeps `.part` + sidecar; sidecar bytes match the file |
| `unpinned_documents_never_keep_a_partial` | cut document → one request, no files left |
| `retries_a_transient_status_with_the_documented_waits` | 503 ×3 → 4 requests, waits exactly `[2 s, 5 s, 10 s]` |
| `gives_up_after_three_retries_and_moves_to_the_next_mirror` | exhausted retries then mirror; same waits |
| `a_user_cancel_is_never_retried` | pre-cancelled → 0 requests, no waits |
| `a_redirect_chain_still_resumes` | `Range` survives a redirect and the 206 applies |
| `a_redirected_transfer_records_the_mirror_url_in_its_sidecar` | kept sidecar names the mirror URL |
| `a_partial_from_another_url_is_left_alone_and_not_resumed` | other URL's partial not used; no `Range` |
| `starting_a_fetcher_bounds_the_partial_cache` | R5 cleanup runs in the constructor |

### A2 (R5) — cleanup by age and total size

`partial.rs` tests: `cleanup_removes_expired_partials_only` (8-day-old partial gone with its
sidecar, a fresh one and an unrelated file untouched) and
`cleanup_keeps_the_total_under_the_limit_oldest_first` (three sparse 1 GiB partials → oldest
removed until exactly the 2 GiB cap; the production constant is exercised without writing 2 GiB).
Parsing/damage coverage: `parses_valid_content_ranges`, `loads_a_matching_sidecar_and_refuses_
another_url`, `damage_discards_the_partial`, `hash_prefix_reads_exactly_the_recorded_bytes`.

### A3 (R6) — UI model and headless render

- Unit test `runtime::visual_copy_tests::a_resumed_download_row_starts_at_the_bytes_on_disk`:
  a transfer at 620/1000 starts the bar at 620 (not 0) and reports resuming.
- Headless test `headless::journeys::resuming_download_row`: sets a resuming 504/1000 transfer,
  asserts the rendered row says "Resuming…" and the bar value is 504, and writes
  `target/headless/resuming_download_row/040-resuming-download.png` (software renderer, no
  visible window).
- The existing `headless::journeys::install_progress_and_play_setup` still passes, so the real
  install row (now starting from `resumable_bytes`) renders as before.

### A4 — checks

Targeted and package-level, green:
- `cargo fmt --all --check`
- `cargo clippy -p ddl-core --all-targets -- -D warnings`
- `cargo clippy -p ddl --all-targets -- -D warnings`
- `cargo test -p ddl-core` → **430 passed, 0 failed** (85 of them `downloads::*`).
- `cargo test -p ddl --bin ddl -- --skip headless` after merging `main` → **114 passed, 0 failed**.
- The two spec-040 headless journeys and the runtime unit test pass after the merge
  (`headless::journeys::resuming_download_row`, `headless::journeys::install_progress_and_play_setup`,
  `runtime::visual_copy_tests::a_resumed_download_row_starts_at_the_bytes_on_disk`).
- An earlier full `cargo test -p ddl --bin ddl` (before merging `main`) ran 140 tests: 139 passed
  and one unrelated pre-existing failure,
  `headless::journeys::categories_favorites_and_screenshots`, which asserts every bundled
  screenshot is 640×360 while `a-doomer-boards-christmas-carol/01.jpg` (and several other assets
  from specs 037/043) are 1024×522/1024×640/1280×720. That asset and the test are untouched by
  this branch and the failure reproduces on `main`; it needs an owner outside spec 040.
- Full `cargo test --workspace`, full-workspace clippy and the Linux e2e wait for the build slot.

**Needs Deck Validation:** pull the network mid-download of a large mapset, reconnect, press
Retry and confirm progress continues from the kept bytes and the final SHA-256 verifies.

## Dependencies

None added. `serde`/`serde_json` were already dependencies (sidecar JSON, D-006).

## Decisions and open questions

1. **Retries are scoped to pinned installs.** R4 says "within one install"; an unpinned catalog
   document is not an install and already has its own throttle. If the coordinator wants the
   catalog bundle retried too, it is a one-line change (`retry_transient` in `download`), but the
   automatic check would then make four requests and wait 17 s before its existing throttle.
2. **Mirror URL in the sidecar.** Required for Dropbox, whose per-request host changes between
   attempts. The validator still comes from the final response.
3. **Restart-once is per URL attempt.** A retry after a transient failure may restart once again
   if the new range answer is also unusable; this is bounded by the three-retry budget.
4. **A pre-existing main failure** (`categories_favorites_and_screenshots`, non-640×360 bundled
   screenshots) will make the full workspace tests red until someone fixes the assets or the
   assertion. It is outside this task's ownership.

## Proposed coordinator-owned edits

- `docs/status.md` / `docs/journal.yml`: record spec 040 as implemented (core + UI) once the
  Linux e2e passes; note the Deck pull-the-plug validation is still pending.
- `docs/specs/040-resumable-downloads.md`: tick A1–A3 once this report is accepted; leave A4's
  e2e item for the coordinator.

# 024 — Download idle timeout and the flaky engine-download test — worker report

**Status:** R1–R4 implemented; A1 and A3 complete; A2 (20-run loop under a concurrent build) is
reported in [Verification](#verification) below. No production timeout was loosened; no new
dependency was added.

## What changed

### R1 — Idle timeout (60 s without body bytes)

- New `pub const BODY_IDLE_TIMEOUT: Duration = Duration::from_secs(60)` beside the existing body
  bounds in `crates/ddl-core/src/downloads/fetch.rs`.
- `DeadlinePolicy` (the test knobs) gained an `idle` field; `Fetcher::with_idle_timeout` shrinks it
  in tests.
- The cap is enforced in the **transport**, not in the read loop, because ureq 3.4.2 cannot express
  it: `timeout_recv_body` is a *total* budget that starts at the response headers and "is not
  restarted for each read" (confirmed in ureq 3.4.2 `src/timings.rs`, `CallTimings::next_timeout`,
  and `src/config.rs`, `ConfigBuilder::timeout_recv_body`). A total budget would abort a large
  archive that is still progressing, so it cannot serve as an idle bound.
- `Fetcher` now builds its agent with `Agent::with_parts` and ureq's **default connector chain**
  (`DefaultConnector::default()`), extended with a pass-through `IdleCapConnector` →
  `IdleCapTransport`. That wrapper lowers ureq's computed per-read `RecvBody` timeout to
  `min(remaining total budget, BODY_IDLE_TIMEOUT)` and delegates every other phase (connect, send,
  header receive, pooling, TLS detection). The size-scaled body deadline is unchanged and still
  arrives through ureq's own budget.
- Applies to every fetch through `downloads::Fetcher` — mapset packages, the engine AppImage and the
  catalog bundle all share this agent.

### R2 — Distinct error

- New `Error::Stalled { url, received, total }` in `crates/ddl-core/src/downloads/error.rs`, with a
  message that says the download stopped receiving data and can be retried
  (`download stalled: stopped receiving data after N of M bytes; check your connection and try
  again (url)`), and a unit test for the text.
- The read loop decides between `Stalled` and `TooSlow` from the gap since the last body byte: if
  `BODY_IDLE_TIMEOUT` was the bound that fired, the gap is at least the idle timeout; otherwise the
  total deadline fired. The partial file is discarded exactly as before (`TempFile` drop), so
  nothing is committed.
- The UI and CLI surface it like the other download errors: new `Problem::Stalled`
  (`crates/ddl/src/services.rs`, also used by the CLI via `Display`), mapped from `Error::Stalled`
  in `download_error`, with a `Text::ErrorStalled` key
  (`crates/ddl/src/wiring_copy.rs`, `crates/ddl/src/runtime.rs`) and one `@tr` string
  (`crates/ddl/ui/wiring-strings.slint`). This is the only file touched under `crates/ddl/ui/`
  (spec 025's area) — a single added line, no layout change.

### R3 — Cancellation still wins

- The loop still checks the token at the top of every iteration, and now also checks it **before**
  mapping a timed-out read to `Stalled`/`TooSlow`, so a cancel that races either timer reports
  `Cancelled`. `cancel_during_a_stall_reports_cancelled` covers this.

### R4 — Flaky engine-download fixture

- **Actual cause.** The gated headless fixture serves the engine AppImage from a loopback server
  whose handler blocks on `engine_gate` *before writing the response headers*. The download worker
  is therefore parked in the **response-header** receive, bounded by the production
  `RESPONSE_HEADERS_TIMEOUT` (15 s). Under a concurrent build the test thread can be descheduled for
  longer than 15 s between showing progress and calling `engine_gate.release()`, so ureq times the
  header wait out first and the download fails with `Error::Network` (a timeout) instead of reaching
  the post-header cancel check and reporting `Cancelled`. Production timeouts are correct; the
  fixture was racing a real 15 s timer.
- **Fix (test-only, production untouched).** `EngineManager` gained a test-support
  `with_response_timeout`, `TestEnvironment` carries `engine_response_timeout`, and
  `Services::open` applies it. The fixture sets `Some(120 s)`, which is longer than the fixture's
  own 60 s `Gate` safety timeout, so the gate — not a timer — always decides the outcome: either the
  test releases the gate, or the gate self-releases at 60 s; either way the headers arrive, the
  already-set cancel token is observed and the result is `Cancelled`. No production constant changed.

## Verification

### A1 — loopback tests (R1–R3)

`cargo test -p ddl-core downloads` — tail:

```text
test downloads::fetch::tests::an_idle_transfer_is_aborted_with_the_stalled_error ... ok
test downloads::fetch::tests::a_slow_but_steady_transfer_below_the_idle_gap_succeeds ... ok
test downloads::fetch::tests::cancel_during_a_stall_reports_cancelled ... ok
test downloads::fetch::tests::a_transfer_that_outlasts_the_total_deadline_reports_too_slow ... ok
test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 292 filtered out; finished in 0.83s
```

- `an_idle_transfer_is_aborted_with_the_stalled_error`: headers + one 16 KiB chunk, then a 5 s
  silence, with a 250 ms idle timeout and the production total deadline → `Error::Stalled` with
  `received == 16 KiB`.
- `a_slow_but_steady_transfer_below_the_idle_gap_succeeds`: four 16 KiB pieces 150 ms apart with a
  400 ms idle timeout → completes.
- `cancel_during_a_stall_reports_cancelled`: a cancel lands while the read is blocked on a stalled
  server → `Error::Cancelled`, not `Stalled`.
- `error::tests::stalled_names_progress_and_suggests_a_retry`: message text (with and without a
  total).
- `a_transfer_that_outlasts_the_total_deadline_reports_too_slow`: the size-scaled total deadline
  still fires (and is still reported as `TooSlow`) when it is the first bound.

### A2 — 20 runs under a concurrent build

Loop command (run from the worktree; the concurrent build uses a separate `CARGO_TARGET_DIR` so it
does not serialize on the cargo lock):

```sh
for i in $(seq 1 20); do
  cargo test -p ddl --bin ddl \
    headless::fixture_screens_navigation_and_focus_render_headlessly -- --exact
done
# concurrently, for the whole loop:
for n in $(seq 1 8); do
  touch crates/ddl-core/src/lib.rs crates/ddl/src/main.rs
  CARGO_TARGET_DIR=target-load cargo build --workspace
done
```

Result: <!--A2_RESULT-->

### A3 — fmt, clippy, tests

```text
cargo fmt --all --check          → clean
cargo clippy --workspace --all-targets -- -D warnings
                                 → Finished `dev` profile ... (no warnings)
cargo test --workspace
  ddl unit (bin):   test result: ok. 90 passed; 0 failed; ... finished in 149.24s
  catalog_cli:      test result: ok. 3 passed; 0 failed
  cli_help:         test result: ok. 2 passed; 0 failed
  logo_assets:      test result: ok. 2 passed; 0 failed
  ui_strings:       test result: ok. 1 passed; 0 failed
  ddl-core unit:    test result: ok. 347 passed; 0 failed
  dsda_launch:      test result: ok. 16 passed; 0 failed
  Doc-tests:        test result: ok. 3 passed; 0 failed
```

### Scope / dependencies

- New dependencies: **none**. The idle cap uses ureq's already-depended-on
  `ureq::unversioned::transport` (marked non-semver by upstream) plus `ureq::unversioned::resolver`;
  ureq is pinned at 3.4.2 in `Cargo.lock`.
- Files touched outside `crates/ddl-core/src/downloads/**`: the test-only `EngineManager` knob
  (`crates/ddl-core/src/engines/dsda/manager.rs`), the fixture and its test wiring
  (`crates/ddl/src/{headless_production,services,recipe_change_tests,cli_exec}.rs`), the download
  error wording (`crates/ddl/src/{services,runtime,wiring_copy}.rs`) and the one string in
  `crates/ddl/ui/wiring-strings.slint`. No other spec, `OVERVIEW.md`, `docs/decisions.md`,
  `docs/status.md` or `docs/journal.yml` was edited.

## Open questions

- `ureq::unversioned::transport` is explicitly outside ureq's semver promise. The pin in
  `Cargo.lock` keeps this stable; a future ureq upgrade should re-check `Transport::await_input` and
  `NextTimeout`. If ureq ever ships a real per-read/idle body timeout, `IdleCapTransport` can be
  dropped.
- Cancellation during a *silent* body read is observed when the read returns, i.e. within the idle
  timeout (60 s in production, the shrunk value in tests). This is the same blocking-read model as
  before (where the bound was the much larger total deadline); a truly prompt interrupt would need
  an interruptible transport. Flag if the spec expects sub-second cancel during a body stall.
- `crates/ddl/ui/wiring-strings.slint` is shared with spec 025; the added `error-stalled` line is
  independent of layout work, but a merge with 025 may touch the same file.

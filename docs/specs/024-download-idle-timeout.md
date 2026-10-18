# 024 — Download idle timeout and the flaky engine-download test

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 006, 010
**Owner request (2026-10-17):** "Pode ajustar o timeout."

## Goal

A download that stops receiving bytes fails within about a minute with a clear message, instead
of hanging until the size-scaled body deadline (up to 6 h). The headless engine-download test
stops failing intermittently under load.

## Background

`crates/ddl-core/src/downloads/fetch.rs` bounds a transfer with a connect timeout (15 s), a
response-header timeout (15 s) and a size-scaled body deadline (`BODY_DEADLINE_BASE` 60 s +
size at a minimum rate, capped by `MAX_BODY_DEADLINE` 6 h) that fails with `Error::TooSlow`.
There is no limit on the gap between two received chunks, so a stalled connection on a big
archive (Eviternity II, 121 MiB) waits for the full deadline. Separately, the gated headless
engine-download fixture once failed under concurrent builds with `Network` instead of
`Cancelled` because the loopback server missed the 15 s response-header timeout
(`docs/reports/020-recipe-change-after-install.md`).

## Requirements

- **R1** — **Idle timeout:** abort a transfer when no body bytes arrive for `BODY_IDLE_TIMEOUT`
  (60 s; a `const` beside the others, shrinkable through the existing test knobs). Applies to
  every fetch through `downloads::Fetcher` (packages, the engine AppImage, catalog bundles). The
  size-scaled body deadline stays as the overall bound.
- **R2** — **Distinct error:** a new `Error::Stalled { received, total }` (or equivalent) with a
  user message that says the download stopped receiving data and can be retried; the UI and
  `ddl` CLI show it like the other download errors. A partial file is discarded as today; nothing
  is committed.
- **R3** — **Cancellation stays prompt:** user cancel still wins over both timers.
- **R4** — **Flaky test:** make the engine-download headless fixture deterministic under load —
  e.g. a test-only header timeout or an injected clock/knob — without loosening production
  timeouts. Explain the cause in the report.

## Acceptance criteria

- [x] **A1** (R1–R3) — loopback tests: a server that sends headers and some bytes then stalls
  fails with the idle error within the shrunk idle timeout; a slow-but-steady server below the
  idle gap still succeeds; cancel during a stall reports Cancelled; error text tested.
- [ ] **A2** (R4) — the fixture passes 20 consecutive runs while another `cargo build` runs
  (report the loop command and result).
- [x] **A3** — fmt, clippy `-D warnings`, tests green.

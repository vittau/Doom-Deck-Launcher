# 027 — Split the headless production journey into focused tests

**Status:** Done  ·  **Milestone:** M3  ·  **Depends on:** 010, 025, 026
**Owner request (2026-10-17):** "Esse teste tá muito longo, tem que quebrar depois em testes
menores."

## Goal

The UI test suite is a set of small, independent headless tests that each finish quickly, run
in parallel, and fail with a name that says which screen or flow broke — instead of one journey
that takes 2–5 minutes and has twice raced real network timeouts under load.

## Background

`crates/ddl/src/headless.rs::fixture_screens_navigation_and_focus_render_headlessly` calls
`headless_production::journey` (≈2,400 lines, 62 captures plus focus, navigation, wiring,
install/cancel, recipe-change and title-fit assertions). Slint allows one platform per process,
so spec 026 phase B runs its focused cases as **subprocesses** of the test binary
(`crates/ddl/src/local_ui_tests.rs`); that harness is the model. Two flakes came from gated
loopback servers holding response headers while slow renders ran (specs 024 R4 and 026 B).

## Requirements

- **R1** — **Split by screen/flow:** break the journey into independent tests (e.g. first run and
  engine setup, IWAD review, Home, Discover and categories, Library, Mapset detail, Play setup and
  profiles, install/cancel/retry, recipe updates, uninstall, catalog update rows, screenshots and
  credits, title fit, settings), each with its own scratch XDG root and fixtures, runnable alone
  by name. Use the subprocess harness (or an equivalent that respects one-platform-per-process).
- **R2** — **No coverage lost:** every existing capture, PNG name and assertion survives in some
  test; list the mapping (old journey section → new test) in the report. README screenshot
  regeneration (spec 016 R5) keeps working with the same file names.
- **R3** — **Fast and parallel-safe:** no single test above ~60 s on the dev Mac; the full
  `cargo test --workspace` wall time is not worse than today; tests share no mutable paths;
  gated network fixtures release their gates before slow renders or use the test-only timeout knob.
- **R4** — **Docs:** a short "Running UI tests" note (where: `docs/design/ui.md` or the guide's
  maintainer section): run one test, regenerate README screenshots, where PNGs land
  (`DDL_HEADLESS_DIR`).

## Out of scope

Changing UI behaviour or visuals; new coverage beyond what exists (welcome, but optional).

## Acceptance criteria

- [x] **A1** (R1, R2) — the journey test is gone or reduced to a thin smoke; the report maps
  every old section, capture and assertion to its new test.
- [x] **A2** (R3) — per-test durations and total wall time before/after in the report; the full
  default-parallel `cargo test --workspace` passes 3 times in a row.
- [x] **A3** (R4) — docs note; README screenshots regenerate byte-identically.
- [x] **A4** — fmt, clippy `-D warnings`, tests green.

Verified in [reports/027-split-headless-journey.md](../reports/027-split-headless-journey.md).

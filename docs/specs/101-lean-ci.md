# 101 — Keep routine CI proportional to the change

**Status:** Ready · **Depends on:** 001, 014, 062, 100
**Origin:** owner, 2026-10-22: CI takes too long; avoid redundant tests and tests that
do not apply to the change. The real-network e2e timed out twice on the unchanged
archive.org Terminal Voltage download during the 0.0.13 verification.

## Goal

Routine work gets useful offline feedback without rebuilding every delivery format
or rendering the exhaustive title/layout matrix on every push. Preserve release
build, signing, AppStream, integrity and publication gates.

## Requirements

- **R1 — One normal Rust check path.** Keep formatting, denied-warning workspace
  Clippy and the ordinary offline tests on relevant source pushes/PRs. Remove the
  duplicate Steam Runtime release build from routine CI: `release.yml` already builds
  and smoke-tests the exact tag in that SDK. Fold the separate PR catalog-lint command
  into the existing checks job, reusing its compilation instead of a second job/cache.
- **R2 — Exhaustive visual tests on demand.** Normal CI skips only
  `headless::journeys::title_fit_` and
  `headless::journeys::categories_favorites_and_screenshots` (the ten expensive
  exhaustive rendering cases). Keep all other model, navigation, persistence and
  focused headless tests. Add an explicit manual full-test option for the unfiltered
  suite; keep full coverage available and make skipped coverage clear in step names
  and documentation. A manual full-test request must not publish a catalog bundle.
- **R3 — Build Flatpak when needed.** Make the artifact-only Flatpak workflow manual.
  Keep the signed tag publication workflow and all its gates unchanged. Add lightweight
  offline checks for Flatpak metadata/helper changes: existing dependency-free helper
  tests, XML well-formedness and checked-in first release == workspace version. Avoid
  Rust/SDK builds for packaging/scripts/docs/workflow-only changes; explicit source
  path filters may be used. Keep independent website publication behavior intact.
- **R4 — Network diagnostics, not release gates.** Keep `e2e.yml` as an optional manual
  diagnostic for downloader/archive/engine integration changes or investigation. Remove
  the orchestration rule that runs it for every catalog-related change; it must not gate
  a metadata, copy or UI release. Do not change download hashes, allowlists, timeouts or
  skip failing network cases to manufacture a green result.
- **R5 — Proportionate local verification.** Update orchestration guidance to use focused
  checks for affected behavior; do not repeat a full workspace suite after documentation,
  metadata, workflow or test-only follow-ups that cannot invalidate its result. Choose
  one authoritative full-suite run when a change needs it, rather than repeating it on
  macOS and Linux by habit. Preserve required Linux release/build smoke verification.
- **R6 — Evidence.** Validate changed workflow syntax and the new command/trigger choices
  statically, with `actionlint` and YAML parsing where available; no full Cargo suite,
  network e2e or SDK build is needed for this workflow/documentation change. Report which
  automatic work was removed, which checks remain, and that actual duration improvements
  need a future relevant run. No new application/dependency or test-framework dependency.

## Scope and boundaries

Workflows (`ci.yml`, `flatpak.yml`, a small metadata-only workflow if needed, comments in
`e2e.yml`), developer/Flatpak build instructions and `docs/orchestration.md`, this spec and
`docs/reports/101-lean-ci.md` only. Do not edit `release.yml`, `flatpak-remote.yml`, production
Rust, source tests, Cargo files, `docs/status.md`, decisions/journal or product behavior.
Do not push, tag or release from the worker branch. Do not run full Cargo checks or e2e.
No visible window or screenshot. Coordinate with the publication in progress from immutable
`v0.0.13-update`; do not change its source or publication gates.

## Acceptance

1. Routine source CI retains fmt/Clippy/ordinary offline tests and one reused catalog lint.
2. All ten exhaustive render cases remain runnable through the documented manual option.
3. Packaging-only edits receive lightweight metadata checks without a Rust/SDK rebuild.
4. Flatpak artifact builds and network diagnostics are explicit manual tools; signed tag
   publication and Steam Runtime release verification are preserved.
5. Static checks pass and the report explains the reduced work and remaining coverage.

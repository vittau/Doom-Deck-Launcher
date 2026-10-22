# 099 — Keep the Flatpak AppStream release version current

**Status:** Ready · **Depends on:** 062, 063, 079
**Origin:** coordinator, 2026-10-22, after a physical Deck showed metadata version
`0.0.11` for the published `v0.0.12-catalog` package (source commit `409d6c5`),
while its binary correctly reported `0.0.12`.

## Goal

The newest AppStream release shipped in either Flatpak build matches the checked-out
workspace version, so the package metadata does not misrepresent the application version.

## Observed evidence

The coordinator verified that the checked-in metainfo and the published tag still list
`0.0.11` first, while `Cargo.toml` and `ddl --version` report `0.0.12`. The installed
Flatpak already matches the remote commit, so `flatpak update` correctly reports nothing
to do. This task repairs build metadata; it does not change update or catalog behavior.

## Requirements

- **R1 — Build-time synchronization.** Add a dependency-free helper under `scripts/flatpak/`
  that reads `[workspace.package].version` from `Cargo.toml` and updates the metainfo in
  place with that version as the first release. Preserve other releases and any existing
  description for the matching version, avoid duplicates on repeated runs, and accept an
  explicit ISO date from the workflow's UTC build clock (`date -u +%Y-%m-%d`). Invoke it
  in both `.github/workflows/flatpak.yml` and `.github/workflows/flatpak-remote.yml` before
  metainfo validation and `flatpak-builder`. Also add `0.0.12` to the checked-in metainfo.
- **R2 — Regression guard.** Add an offline Rust integration test under `crates/ddl/tests/`
  that reads both repository files and requires the first AppStream release to equal the
  workspace version. No new dependency. Test the helper's history preservation, promotion
  of an existing version, idempotence and invalid-date handling with standard-library tests.
- **R3 — Application unchanged.** Do not alter the application binary, application behavior,
  Flatpak bundle logic or catalog logic.
- **R4 — Validation preserved.** Keep both existing `appstreamcli validate --no-net --explain`
  gates unchanged and validate the rewritten XML with the same command locally if available.
- **R5 — Verification and handoff.** Run `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, helper
  tests and a local helper invocation. Record results and any unavailable tooling in
  `docs/reports/099-flatpak-appstream-version.md`. A publication run and physical Deck check
  must still confirm the real Flatpak/Discover display; do not claim those happened.

## Scope and boundaries

Only the helper and its tests, the two Flatpak workflows, the metainfo, the Rust regression
test, this spec and the worker report change. Use Python's standard library for TOML/XML
handling; no Cargo or Python dependency additions. Never push, tag, release or dispatch a
remote workflow. Do not edit `OVERVIEW.md`, `docs/decisions.md`, `docs/status.md` or
`docs/journal.yml`. Commit small changes on the task branch. No visible windows or screen
capture; all verification is headless.

## Acceptance

1. Both build paths synchronize the checked-out workspace version before validation and
   building, with the newest release first and historical entries retained.
2. The checked-in first release is `0.0.12`; the Rust guard would fail for `0.0.11`.
3. Helper tests demonstrate valid XML, repeatable updates and preservation of release history.
4. The requested Cargo checks, local helper invocation, AppStream validation when available,
   workflow static checks and `git diff --check` are recorded in the report.
5. The report states exactly what remains for publication and Deck validation, with no
   application/bundle/catalog behavior changes or new dependencies.

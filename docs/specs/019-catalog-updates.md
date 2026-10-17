# 019 — Catalog updates without a DDL release

**Status:** Done · **Decision:** D-025  ·  **Milestone:** M3  ·  **Depends on:** 002, 006, 015
**Backlog item:** "Catalog updates without a DDL release" — D-019 left fetching updates to a
separate decision: D-025 (owner answers 2026-10-17).

## Goal

A recipe merged into `catalog/` on `main` reaches users without waiting for the next DDL
release, with the same validation CI applies, and the embedded catalog stays the offline fallback.

## Requirements

- **R1** — **Bundle:** `ddl catalog bundle --out <file>` writes one JSON document:
  `bundle_schema = 1`, `catalog_serial` (UTC Unix seconds of the source commit),
  `source_commit`, `min_ddl_version`, and every recipe, `tags.toml` and `hosts.toml` as their
  TOML text keyed by file name. It runs lint first and refuses to write on any lint error.
- **R2** — **Publishing (CI):** a job in `ci.yml`, on push to `main` that touches `catalog/**`,
  builds the bundle and commits it as the single file `catalog.json` to a dedicated branch
  `catalog` (orphan history, ordinary commits, never force-pushed; `contents: write` for that job
  only). The app reads
  `https://raw.githubusercontent.com/vittau/Doom-Deck-Launcher/catalog/catalog.json`. No GitHub
  release is involved. Document the branch in `docs/orchestration.md`-style maintainer notes in
  the report.
- **R3** — **Fetch and validate (core):** `ddl-core` fetches
  `https://raw.githubusercontent.com/vittau/Doom-Deck-Launcher/catalog/catalog.json` with the
  existing download stack (timeout, ≤ 2 MiB bound, HTTPS only; `raw.githubusercontent.com` is
  already an allowlisted host). It accepts the bundle only if: `bundle_schema` is known; `min_ddl_version`
  ≤ the running version; `catalog_serial` > the embedded catalog's serial (from `build.rs`); every
  recipe passes the same lint as CI; **hosts are taken from the embedded `hosts.toml`**, so a
  remote recipe naming a host the binary does not allow is dropped (with a diagnostic), not
  accepted. Partial acceptance is per recipe; a malformed bundle is rejected whole.
- **R4** — **Storage and precedence:** the accepted bundle is written atomically to
  `<XDG data>/ddl/catalog/catalog.json` with its fetch time; on start DDL loads the newer of
  embedded vs cached (by serial). A corrupt cache is ignored and logged, never fatal. Installed
  mapsets keep resolving from their install manifest (D-019).
- **R5** — **When:** an automatic check at most once per 24 h, in the background after the UI is
  up, silent on failure (offline Deck); Settings shows "Catalog: <serial date> (built-in | updated)",
  "Check for updates" (manual, shows progress/result) and an "Automatic catalog updates" toggle
  persisted in app state (default on). CLI: `ddl catalog update` and `ddl catalog status`.
- **R6** — **New recipes without art:** recipes that arrive by bundle use the fallback cover
  (spec 013); covers ship only with releases.

## Out of scope

Signing the bundle (revisit: minisign, D-022). Remote changes to the hosts allowlist. Covers in
the bundle. Recipe-change handling for installed mapsets (spec 020).

## Acceptance criteria

- [x] **A1** (R1, R3) — tests with a loopback HTTP fake: valid bundle accepted; old serial,
  unknown schema, too-new `min_ddl_version`, oversize, malformed JSON rejected; recipe with a
  non-allowlisted host dropped, others kept; lint failure drops the recipe.
- [x] **A2** (R4) — precedence and corrupt-cache tests; atomic write.
- [x] **A3** (R5) — 24 h throttle test with a fake clock; headless render of the Settings rows;
  CLI tests.
- [ ] **A4** (R2) — workflow change reviewed; first upload verified by the coordinator after merge.
- [x] **A5** — fmt, clippy `-D warnings`, tests green.

## Decisions (owner answers 2026-10-17 → D-025)

- **Q1 → automatic:** check automatically (at most every 24 h, background, silent on failure),
  plus the manual "Check for updates" and an opt-out toggle.
- **Q2 → HTTPS from GitHub is enough for now** (the same trust as the release binary); signing
  (minisign) stays a later hardening step.
- **Q3 → no fixed pre-release:** the owner found it confusing and it would clutter Releases; the
  bundle lives on a CI-maintained `catalog` branch instead (R2).

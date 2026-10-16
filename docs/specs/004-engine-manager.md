# 004 — DSDA-Doom engine manager

**Status:** Ready (after 001 and 006 merge)  ·  **Milestone:** M1  ·  **Depends on:** 001,
006 (download pipeline), 003 (`smoke_test`)

## Goal

DDL finds, downloads, verifies, installs, selects and rolls back official DSDA-Doom AppImages
inside its own data directory, offline-first and without ever trusting an unreviewed binary by
default (D-007, D-013). Evidence: `docs/research/upstream-sources.md` §1, §2, §6.1 and
`docs/research/dsda-engine.md` §1 and §7.

## Requirements

- **R1** — **Reviewed engine list** shipped with DDL as TOML
  (`crates/ddl-core/data/engines.toml`): per version, the tag, expected asset name, size,
  SHA-256, and a `validated` flag. Seed it with v0.30.1:
  `dsda-doom-0.30.1-linux-x86_64.appimage`, 14,608,888 bytes,
  `b06adf3f0c7d4e82dc1870baf97e67e4dc4e663e37f69070ddc02fa3e97608b0`.
- **R2** — **Release discovery:** `GET https://api.github.com/repos/dsda-org/dsda-doom/releases?per_page=10`
  with a stable query string, `If-None-Match` using the stored weak ETag, a DDL User-Agent, an
  optional `GITHUB_TOKEN` from the environment (never required), and a 15 s timeout. Skip
  drafts and prereleases; pick the asset by case-insensitive pattern `-linux.*\.appimage$`.
  Cache the JSON, ETag and `fetched_at` in DDL's cache directory; refresh only on explicit
  request and at most once per 24 h unless forced; on 403/429 or a network error, keep the cache
  and report "offline (cached <time>)". Never retry in a loop.
- **R3** — **Availability model:** each discovered release is `Reviewed` (in R1's list),
  `Unreviewed` (newer than the list), or `Unsupported` (no Linux AppImage). The default install
  offers only `Reviewed`. Installing `Unreviewed` requires an explicit `allow_unreviewed` flag and
  then checks the API `digest` when it is non-null; it is recorded as unreviewed.
- **R4** — **Install:** download through spec 006's pipeline into a temp file (follow the GitHub
  redirect, enforce the expected size), verify the size and SHA-256 against the reviewed entry,
  move it atomically to `<data>/engines/dsda-doom/<version>/dsda-doom.AppImage`, set mode 0755,
  run 003's `smoke_test` (`-v`, expect the matching version), and only then record the
  installation. A failed step leaves no partial install behind.
- **R5** — **Extraction fallback (D-013):** if launching the AppImage fails in a way that
  indicates FUSE is unavailable (the smoke test exits non-zero and stderr mentions FUSE or
  `fusermount`), run `<appimage> --appimage-extract` once in
  `<data>/engines/dsda-doom/<version>/` (it creates `squashfs-root/`) and use
  `squashfs-root/AppRun` as the executable from then on. Keep the whole extracted tree.
  Record which mode is active.
- **R6** — **Installed state** (JSON, atomic writes, versioned schema): installed versions with
  path, mode (appimage/extracted), SHA-256, source URL, reviewed flag, install time; the
  **active** version; the previous known-good version.
- **R7** — **Select / rollback / remove:** set any installed version active; rollback re-activates
  the previous known-good one; remove deletes a non-active version. All three are refused
  while an engine run is in progress (shared run guard with spec 005).
- **R8** — **Manual engine path:** the user can register an existing executable (absolute path).
  It must pass `smoke_test`, and it is recorded as `manual` (never deleted by DDL).
- **R9** — **Attribution data:** each version exposes its license (GPL-2.0-or-later), the upstream
  release URL and the tagged source URL, for the Settings screen.

## Out of scope

UI. Automatic background updates (never: OVERVIEW §5 step 8). Mirroring or bundling the engine.

## Acceptance criteria

- [ ] **A1** (R1) — the list parses; the seed entry matches the research values.
- [ ] **A2** (R2, R3) — tests with recorded API fixtures (taken from the research, trimmed):
  asset pattern across all three historical naming schemes, prerelease skipping, ETag/304
  handling, 403/429 fallback to cache, 24 h throttling, classification.
- [ ] **A3** (R4, R5) — tests with a fake HTTP server and a fake "AppImage" script fixture:
  happy path, size mismatch, hash mismatch, smoke failure, FUSE-failure → extraction path
  (the fake responds to `--appimage-extract` by creating `squashfs-root/AppRun`); no partial
  state after failures.
- [ ] **A4** (R6, R7, R8, R9) — state round-trip, select/rollback/remove rules, run-guard
  refusal, manual registration.
- [ ] **A5** — *Needs Deck Validation:* install v0.30.1 on the Deck from DDL; launch directly
  and via the extraction fallback (tracked in `docs/deck-testing.md`).

## Tasks

- [ ] Reviewed list + discovery client + cache
- [ ] Install pipeline + fallback
- [ ] State, select/rollback/remove, manual path

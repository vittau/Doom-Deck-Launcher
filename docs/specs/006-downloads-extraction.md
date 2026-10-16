# 006 — Downloads and safe archive extraction

**Status:** Ready (after 001 merges)  ·  **Milestone:** M2 (pulled forward; 004 needs it)  ·
**Depends on:** 001

## Goal

One hardened pipeline fetches every remote file DDL uses (engine AppImages, mapset packages),
verifies it, and extracts **only the files a recipe names** from ZIP archives, including archives
nested inside archives (AUGER;ZENITH ships its add-ons as `DBP37_Addons.zip` inside
`dbp37_augzen.zip`). The user never unzips anything (D-010). All remote input is untrusted
(OVERVIEW §9, §12).

## Requirements

- **R1** — **Fetcher** (`ddl_core::downloads`): HTTPS only (HTTP refused); host allowlist
  supplied by the caller; at most 5 redirects, each target re-checked against the allowlist
  (with an explicit exception list for GitHub's signed `release-assets.githubusercontent.com`
  redirect targets); connect and read timeouts; a hard maximum size and an `expected_size`
  that aborts when exceeded; streamed to a temp file in the destination's filesystem while
  computing SHA-256; mirror fallback (try the next URL on network error or hash mismatch, never
  on a successful match). User-Agent `DoomDeckLauncher/<version>`.
- **R2** — **Progress and cancel:** a progress callback (bytes done/total) and a cancellation
  token checked between chunks; cancel removes the temp file. Downloads run off the UI thread
  (async or a worker thread; choose one and justify it).
- **R3** — **Cache:** completed downloads are stored by SHA-256 in DDL's cache dir
  (`downloads/<sha256>`), so reinstalling or switching components never re-downloads. A cached
  file is re-hashed before use.
- **R4** — **ZIP extraction by explicit member list:** the caller passes the members it wants as
  paths; a path may cross nested archives with `!/` (e.g. `DBP37_Addons.zip!/DBP37_AlternateBlaster.wad`),
  up to 2 levels. Member names are matched exactly (case-sensitive); a missing member is an
  error naming the closest case-insensitive match. Only stored and deflate members are supported
  (report others clearly). Each extracted member can be checked against an expected size and
  SHA-256.
- **R5** — **Safety limits:** reject absolute paths, `..` components, drive prefixes and
  backslash tricks in member names; never follow or create symlinks (symlink entries are
  rejected); limit total uncompressed size, per-member size, member count, compression ratio
  (zip-bomb guard) and nesting depth; extract only into a fresh staging directory under DDL's
  data dir, then move into place atomically. Executable bits are never set on extracted
  content, and no extracted file is ever executed.
- **R6** — **Errors** are typed and user-explainable: network, HTTP status, host not allowed,
  size exceeded, hash mismatch (with expected vs actual), cancelled, archive corrupt, member
  missing, unsafe member, limit exceeded.

## Out of scope

Recipe parsing (002), install records and library (later spec), UI. Formats other than ZIP
(7z/RAR are not supported in v1; recipes needing them are flagged by 002's lint).

## Acceptance criteria

- [x] **A1** (R1, R3) — local HTTP test server (no internet): happy path, redirect limit,
  redirect to a non-allowlisted host, HTTP refused, size exceeded, hash mismatch then mirror
  success, cache hit with re-hash.
- [x] **A2** (R2) — progress monotonic; cancel mid-stream leaves no temp file.
- [x] **A3** (R4) — fixtures generated in tests: flat zip, nested zip (`a.zip!/b.wad`), missing
  member with case hint, member hash check.
- [x] **A4** (R5) — adversarial fixtures generated in tests: `../` traversal, absolute path,
  symlink entry, zip bomb ratio, too many members, depth > 2; each rejected with its typed error
  and nothing written outside staging.
- [x] **A5** — `cargo test` needs no network.

## Tasks

- [x] Fetcher with allowlist, redirects, limits, hashing, mirrors, cache
- [x] Progress/cancel
- [x] ZIP extraction with nested paths and safety limits

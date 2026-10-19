# 040 — Resumable downloads

**Status:** Ready (after 038 phase 1)  ·  **Milestone:** M3  ·  **Depends on:** 006, 038
**Origin:** OVERVIEW §13 M3 ("interrupted-download recovery"); the 2026-10-19 Doomer Boards work
showed archive.org serving at about 2.5 MB/min, so a 156 MB mapset that drops at 90 % on the Deck
restarts from zero. Today `downloads/fetch.rs` streams into a fresh `.tmp-*` file that is removed
on any failure; there is no `Range` request.

## Requirements

- **R1** — **Keep the partial file** of a *pinned* download (expected SHA-256 known) when the
  transfer fails on a network error, a timeout or a user cancel. It lives in the downloads cache
  as `<sha256>.part` with a small JSON sidecar (`schema_version`, URL that served it, bytes on
  disk, the validator — `ETag` or `Last-Modified` — when the server sent one). Unpinned documents
  never keep partials. Writes stay atomic: the sidecar is updated only after the bytes it
  describes are flushed.
- **R2** — **Resume:** the next download of the same pin sends `Range: bytes=<n>-` (plus `If-Range`
  with the validator when one is stored) to the same URL first. `206` with a matching
  `Content-Range` → append and keep hashing from the bytes already on disk. `200` (server ignored
  the range or the file changed) → discard the partial and download in full. `416` or a
  `Content-Range` that does not start at `<n>` or whose total disagrees with the recipe size →
  discard and restart once. Every hop still passes the host allowlist and redirect rules
  (specs 006 and 038), and the size cap counts the whole file.
- **R3** — **Verify as today:** the SHA-256 of the complete file must equal the pin; a mismatch
  deletes both partial and sidecar and reports the existing hash error. A resumed file is never
  trusted without the full hash.
- **R4** — **Automatic retry:** within one install, a transient failure (connection reset,
  timeout, 5xx) retries with resume up to 3 times, waiting 2 s, 5 s and 10 s; a user cancel never
  retries. Mirror order is unchanged: a source that has exhausted its retries moves to the next URL
  (the partial is kept only for the URL that served it).
- **R5** — **Bounded cache:** partials older than 7 days, or beyond 2 GiB in total (oldest
  first), are removed when the fetcher starts. Nothing outside the downloads cache is touched.
- **R6** — **UI:** progress resumes from the bytes already on disk (no jump back to 0 %); the
  download row shows "Resuming…" while the range request is made. No new screens.

## Acceptance criteria

- [ ] **A1** (R1–R4) — fetcher tests against the local test server, no network: resume after a
  cut mid-body (206 path), server that ignores `Range` (200 path), changed validator, bad
  `Content-Range`, hash mismatch after resume, cancel keeps the partial, unpinned keeps nothing,
  retry counts and waits (with an injectable clock/sleeper), redirect + resume.
- [ ] **A2** (R5) — cleanup tests by age and total size.
- [ ] **A3** (R6) — the UI progress model starts at the resumed byte count (unit test); a headless
  render of a download row in "Resuming…".
- [ ] **A4** — fmt, clippy `-D warnings`, tests green; the coordinator runs the Linux e2e.
  **Needs Deck Validation:** pull the network mid-download of a large mapset, reconnect, retry.

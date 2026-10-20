# 044 — Weekly catalog link health

**Status:** Ready (after 040)  ·  **Milestone:** M4  ·  **Depends on:** 006, 014, 038, 040
**Origin:** with about 100 recipes after spec 037, dead or changed upstream files must be found
before players do. The real-network e2e (`.github/workflows/e2e.yml`) has no schedule on purpose:
every run downloads whole archives. This job checks links without downloading them.

## Requirements

- **R1** — **`ddl catalog check-links [<dir>]`:** for every source URL of every recipe (all
  mirrors, not only the first), follow redirects under the same allowlist and redirect rules as
  the downloader (specs 006, 038) and fetch only the headers: `HEAD`, or `GET` with
  `Range: bytes=0-0` when a host refuses `HEAD`. Compare the size (`Content-Length`, or the total
  in `Content-Range`) with the recipe's pinned size. Outcomes per URL: `ok`, `size-changed`,
  `gone` (404/410), `host-refused` (redirect left the allowlist), `error` (other status, timeout).
  Bounded: per-request timeouts, at most 4 requests in flight, at most one in flight per host.
- **R2** — **Report:** a Markdown table (recipe, URL, outcome, detail) on stdout and
  `--json` for machines; exit code non-zero when any recipe has **no** `ok` source left (a
  recipe with one dead mirror and one good one is a warning, not a failure).
- **R3** — **Workflow** `.github/workflows/catalog-health.yml`: weekly schedule plus
  `workflow_dispatch`, builds `ddl` on the Ubuntu runner like the `checks` job, runs R1, writes
  the table to the job summary, and opens or updates a single issue labelled `catalog-health`
  when R2 fails (closes it when a later run is clean). Least-privilege `permissions`
  (`contents: read`, `issues: write`).
- **R4** — **Lean e2e install set:** the real-network e2e stops installing every recipe. Its run
  of 2026-10-19 (run 39304815134) installed all 89 and failed after 41 min on one 156 MB
  archive.org file (`avpvtvjvvhvss`, "download too slow"). It now installs a deterministic sample
  chosen by a small, tested `ddl` helper or script (not a hand-kept list):
  - the smallest recipe per first-source download host;
  - every recipe with optional components, with each free component installed once, as today;
  - every widescreen status-bar case, as today.
  Each archive stays under a size cap, e.g. 64 MiB, unless it is the only recipe for its host.
  Full-catalog coverage moves to R1–R3 (headers, all mirrors). The sample and the reasoning go
  in the workflow comment, and the run summary lists the chosen ids.

## Acceptance criteria

- [ ] **A1** (R1, R2) — tests against the local test server, no network: each outcome, `HEAD`
  refused → range fallback, redirect out of the allowlist, a recipe with one dead and one good
  mirror (warning), a recipe with no good source (failure), concurrency bound.
- [ ] **A4** (R4) — a test of the sample selection over the real `catalog/`: every host is
  covered, component and status-bar recipes are included, the size cap holds. The coordinator's
  e2e dispatch after merge passes and finishes in under 30 minutes.
- [ ] **A2** (R3) — `actionlint` (or the repo's YAML check) passes; the coordinator triggers it
  once with `workflow_dispatch` after merge and links the run in the report.
- [ ] **A3** — fmt, clippy `-D warnings`, tests green.

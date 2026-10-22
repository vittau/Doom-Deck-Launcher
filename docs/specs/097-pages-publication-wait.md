# 097 — Pages publication wait leaves room for a real build

**Status:** Ready · **Milestone:** M3 · **Depends on:** 062, 063, 079 (signed Flatpak publication)
**Origin:** coordinator, 2026-10-22, after the `v0.0.12-catalog` publication verification
(`.scratch/catalog-publication-verification.md`). Routine repair within existing authority: no owner
input required.

## Goal

The signed Flatpak publication completes its own cache purge and canonical-file verification on
every release, instead of skipping them when the Pages build outlasts the workflow's wait (observed
on `v0.0.12-catalog`, `.scratch/catalog-publication-verification.md`).

## Observed evidence (why this exists)

`gh-pages` carries the whole signed Flatpak repository. Its Pages builds have been slow and
variable; these are the two durations observed during the `v0.0.12-catalog` publication:

| Pages build | Created | Finished | Duration |
| --- | --- | --- | --- |
| `fe7cfb02` (`v0.0.12-catalog`, run `39793977138` attempt 1) | 14:22:33Z | 14:33:28Z | **10 min 55 s** |
| `bf7655cd` (retry run `39798321701`) | 14:51:12Z | 15:00:16Z | **9 min 04 s** |

Both runs published the signed repository and the site correctly, then failed at `Wait for the
Pages deployment` with `Pages did not build … within 8 minutes`; the following `Purge the Cloudflare
cache prefix` and `Verify the published files` steps were therefore **skipped** in both runs. The
retry reproduced the identical exported tree (served app ref `a96c9dd0…`), so the repair is purely
about the wait window, not about content.

## Requirements

- **R1 — Bounded wait.** Raise the wait in `Wait for the Pages deployment`
  (`.github/workflows/flatpak-remote.yml`) from 48 to **120 polls** at the existing 10-second
  cadence: a nominal ~20 minutes of waiting, above both observed builds (10 min 55 s and 9 min 04 s),
  plus API time. Keep it finite and bounded: a fixed attempt count, not an open-ended loop and not a
  measured wall-clock deadline.
- **R2 — Behaviour preserved.** Do not change the polling cadence, the exact
  `commit == PAGES_COMMIT` and `status == "built"` success match, the immediate failure on
  `status == "errored"` for this commit, the trailing `::error::` + non-zero exit when the window
  expires, the Cloudflare prefix purge, or the canonical-file verification that follows it. No purge
  scope may be broadened and no verification weakened.
- **R3 — Job budget.** Confirm the job's `timeout-minutes` still leaves margin for the longer wait
  plus the build, signing, smoke test and verification steps, and say what bounds the wait besides
  the attempt count (each request's `curl --max-time`, and the job cap); change it only if the margin
  is insufficient.
- **R4 — Evidence.** Record the change and the observed 9–11-minute builds in the worker report,
  name the validation performed, and state explicitly that the 20-minute path itself can only be
  proven by a real publication run from the reviewed branch (coordinator, with `ref`
  `v0.0.12-catalog`). No success may be claimed before that run.

## Scope and boundaries

Only `.github/workflows/flatpak-remote.yml`, this spec and
`docs/reports/097-pages-publication-wait.md` change. No dependency, no new workflow, no secret,
credential or configuration change, no push, tag, release or remote dispatch, and no edit to
`docs/status.md`, `docs/journal.yml`, `docs/decisions.md` or `OVERVIEW.md`. The timing change is a
reversible timer adjustment; it needs no implementation-mirroring unit test and no cargo build.

## Acceptance

1. The workflow polls at most 120 times at the same 10-second cadence for the exact expected commit,
   with the same success condition, the same immediate failure on an errored build and the same error
   message form.
2. `Purge the Cloudflare cache prefix` and `Verify the published files` remain in place, unchanged in
   scope, and still run only after a successful wait.
3. The job `timeout-minutes` provably leaves margin for the longer wait.
4. Workflow static validation and `git diff --check` pass; the report records old/new limits, the
   unchanged gates and the outstanding remote validation.

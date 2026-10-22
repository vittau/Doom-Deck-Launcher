# 097 — Pages publication wait leaves room for a real build

**Status:** Ready · **Milestone:** M3 · **Depends on:** 062, 063, 079 (signed Flatpak publication)
**Origin:** coordinator, 2026-10-22, after the `v0.0.12-catalog` publication verification
(`.scratch/catalog-publication-verification.md`). Routine repair within existing authority: no owner
input required.

## Goal

The signed Flatpak publication completes its own cache purge and canonical-file verification on
every release, instead of skipping them because a Pages build takes longer than the workflow waits.

## Observed evidence (why this exists)

The `gh-pages` branch carries the whole signed Flatpak repository, so GitHub Pages builds of it are
slow and variable:

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

- **R1 — Bounded wait.** Raise the wait window in `Wait for the Pages deployment`
  (`.github/workflows/flatpak-remote.yml`) from 8 to **20 minutes**, leaving more than double the
  slowest observed build. Keep it finite and bounded: a fixed attempt count, not an open-ended loop.
- **R2 — Behaviour preserved.** Do not change the polling cadence, the exact
  `commit == PAGES_COMMIT` and `status == "built"` success match, the immediate failure on
  `status == "errored"` for this commit, the trailing `::error::` + non-zero exit when the window
  expires, the Cloudflare prefix purge, or the canonical-file verification that follows it. No purge
  scope may be broadened and no verification weakened.
- **R3 — Job budget.** Confirm the job's `timeout-minutes` still leaves margin for the longer wait
  plus the build, signing, smoke test and verification steps; change it only if the margin is
  insufficient.
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

1. The workflow waits up to 20 minutes for the exact expected commit, with the same cadence, the same
   success condition, the same immediate failure on an errored build and the same error message form.
2. `Purge the Cloudflare cache prefix` and `Verify the published files` remain in place, unchanged in
   scope, and still run only after a successful wait.
3. The job `timeout-minutes` provably leaves margin for the longer wait.
4. Workflow static validation and `git diff --check` pass; the report records old/new limits, the
   unchanged gates and the outstanding remote validation.

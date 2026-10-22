# Spec 097 report — Pages publication wait

**Worker:** OpenCode (verification/repair), 2026-10-22 · **Branch:** `vittau/pages-publication-wait`
· **Status:** fix implemented and statically validated; the 20-minute path awaits one real
publication run from the reviewed branch.

## Change

One step in `.github/workflows/flatpak-remote.yml`, `Wait for the Pages deployment`:

| | Before | After |
| --- | --- | --- |
| Attempts | `seq 1 48` | `seq 1 120` |
| Poll cadence | `sleep 10` | `sleep 10` (unchanged) |
| Wait window | 8 minutes (48 × 10 s) | **20 minutes** (120 × 10 s) |
| Expiry message | `… within 8 minutes` | `… within 20 minutes` |
| Job `timeout-minutes` | 120 | 120 (**unchanged**, margin verified below) |

A comment records why the window is 20 minutes and that the following gates run only after a real
success.

## Observed evidence for 20 minutes

`gh-pages` carries the whole signed Flatpak repository, so its Pages builds are slow:

| Pages build | Run | Created | Built | Duration |
| --- | --- | --- | --- | --- |
| `fe7cfb02` | `39793977138` attempt 1 | 14:22:33Z | 14:33:28Z | **10 min 55 s** |
| `bf7655cd` | `39798321701` (retry) | 14:51:12Z | 15:00:16Z | **9 min 04 s** |

Both runs published correctly and then failed with `Pages did not build … within 8 minutes`, so
`Purge the Cloudflare cache prefix` and `Verify the published files` were skipped in both. Twenty
minutes is more than double the slowest observed build and still finite.

## Job budget

Attempt 1's job ran **16 min 24 s** in total: `Install Flatpak tooling` 1 min 50 s, `Build the tagged
tree` 4 min 33 s, and the 8 min 17 s wait that failed. Replacing that wait with a 20-minute maximum
gives a worst case of roughly **28 minutes**, comfortably inside the existing
`timeout-minutes: 120`. The job limit therefore needs no change, and none was made.

## Gates that are unchanged

- Success still requires **the exact expected commit** and `status == "built"`
  (`[ "$commit" = "$PAGES_COMMIT" ] && [ "$status" = "built" ]`).
- An `errored` build for that commit still fails **immediately** with the build body in the message.
- The window stays **finite**: a fixed attempt count, then `::error::` and a non-zero exit.
- `Purge the Cloudflare cache prefix` is untouched: same one-prefix scope, same script, same
  missing-setting skip and partial-setting failure.
- `Verify the published files` is untouched: same URL set, same byte comparison against the rendered
  site, same 12 × 10 s convergence loop and same purge-aware cache-busting fallback.
- Nothing else in the workflow, no other file, and no secret, credential or configuration changed.

## Validation performed

| Check | Result |
| --- | --- |
| `actionlint .github/workflows/flatpak-remote.yml` (ignoring the pre-existing `ubuntu-26.04` runner-label notice) | exit 0, no findings on this change |
| `yamllint -f parsable` restricted to the changed lines | clean; the file's pre-existing long lines (5, 6, 9, 384, 392) are untouched |
| Ruby YAML parse of the whole workflow | parses |
| `bash -n` on the extracted `run:` script of the step | syntax OK |
| Script content check | `seq 1 120` once, `sleep 10` kept, both `built`/`errored` conditions and the `within 20 minutes` error message present |
| `git diff --check` | clean |

No cargo build or unit test: this is a reversible timer change with no logic to mirror in code, and
`AGENTS.md`'s test-per-requirement rule is met by the static checks above plus the real-run gate
below.

## Outstanding

- **Remote validation is the coordinator's, and it is the only proof of the 20-minute path:** run the
  `Flatpak repository` workflow from this reviewed branch with `ref` input `v0.0.12-catalog` and
  confirm that `Wait for the Pages deployment` succeeds and that `Purge the Cloudflare cache prefix`
  and `Verify the published files` actually execute. No success may be claimed before that run.
- Nothing was pushed, tagged, released or dispatched; no owner question was needed.

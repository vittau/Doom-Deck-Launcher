# /idgames browse responsiveness contract

## Changes

The owner requires archive browsing to remain responsive without freezing the launcher.
Specs 051 and 053 now make this an explicit implementation and acceptance requirement:

- Prepare and search the index outside the UI event loop, reuse prepared search data, and
  bound pending work while rejecting stale results.
- Keep the UI model to one 50-row page, debounce text input, and keep navigation/cancel
  responsive during loading, refresh, search and downloads.
- Exercise full-size and near-limit indexes, controlled slow workers, rapid input, repeated
  navigation and refresh, and measure release-build latency and memory.
- Define proposed Steam Deck LCD response targets with explicit hardware validation still
  required. These are acceptance targets, not measured performance claims.

## Verification

Documentation-only change. Reviewed both specs together for consistent worker bounds,
page limits and stale-result handling; `git diff --check` passes. No application code or
dependencies changed, so application tests were not run.

## Open work

Specs 051 and 053 remain unimplemented. All new acceptance items remain unchecked, including
deterministic event-loop tests, release-build measurements and physical Deck validation.
No hardware performance result is claimed. No owner decision is needed for this spec update.

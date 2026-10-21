# 075 — Make Pistol start a lightweight profile change

**Status:** Ready. **Origin:** owner reports perceptible lag toggling Pistol start.

## Requirements

- R1: Trace and remove unrelated expensive work from the Pistol start toggle. In particular,
  both catalog-mapset Setup and base-game BaseSetup toggles must use the lightweight path;
  changing this boolean must not rebuild the full catalog snapshot or re-hash installed WADs/
  extras across the library. The audit identified repeated catalog parsing; memoize immutable
  embedded source parsing with std facilities while preserving independent mutable catalog
  values and validation. No new dependency or global mutable user state.
- R2: Persist the selected profile atomically with normal validation and preserve all other
  profile fields. Reflect the value promptly in the row and launch-summary text. Keep focus/
  scrolling, and preserve the final requested value across repeated toggles and reopening.
  Handle write failure visibly without falsely presenting an unsaved value as saved. Avoid
  broad service/thread frameworks; use a bounded profile-specific result path if needed.
- R3: Preserve all integrity checks at actual launch and existing handling for modified/missing
  files. A cached presentation summary must never become permission to skip launch validation.
  Other Save operations/component changes need not take the new fast path unless equivalent
  behavior is tested and justified. Do not weaken downloaded-data verification.
- R4: Meaningful offline checks cover both toggle directions, persisted final state, preserved
  profile fields, repeated input, storage failure and no library-wide snapshot/hash work on the
  toggle path. Use deterministic counters/fakes or a fixture that would fail if hashed; avoid
  flaky wall-clock pass/fail thresholds. Record cheap before/after timings if available and
  label physical Deck responsiveness as Needs Deck Validation.

## Ownership and verification

Backend worker owns services/core catalog memoization, local_action BaseSetup toggle dispatch,
and the narrow runtime start/drain/
Pistol-start dispatch path, with focused tests. UI worker concurrently owns runtime card layout,
category test fixtures and header subtitle binding plus Slint; coordinate by function before
editing and commit narrow runtime changes separately for straightforward integration.
Read the 075 audit, correcting its missed full-file hashing in resolver::verify_installed and
extras.verify; the current 92-card IDs fix is already in 072 (audit uses an older UI baseline).
Never open visible windows or capture the screen. Jobs2/threads2, focused iteration and final
fmt/clippy; coordinator runs combined workspace checks. Report docs/reports/075-pistol-start-latency.md.
No main merge, push, release, new dependencies, subagents or coordinator-doc edits.

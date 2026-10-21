# 077 — Check the independent catalog at every startup and show its date

**Status:** Ready. **Origin:** owner; supersedes spec 019 R5's daily throttle.

## Requirements

- R1: Keep publishing the validated catalog JSON independently of app releases to the existing
  GitHub catalog branch. Every application startup with automatic updates enabled checks that
  source once in the background after the first usable UI snapshot, even if checked recently.
  Preserve the opt-out, manual check, offline startup and usable cached/built-in fallback.
  Do not make root navigation or play wait for a network response; do not start a second check
  after every service refresh. Failed automatic checks stay silent and retain current data.
- R2: Retain existing bounded fetch, validation, minimum app version, fixed embedded host
  allowlist, atomic cache and anti-rollback rules. Installed content keeps its frozen manifest.
  Verify that a remote recipe-only update can arrive without an app release.
- R3: Discover's header shows a small subtitle with the effective catalog source date/time,
  formatted exactly like `October 21, 2026 at 13:39 (UTC)`. Use the catalog source serial, never
  last fetch/check time; update it when a newer catalog becomes active. Unknown serial must be
  represented honestly (for example `Catalog date unavailable`), never today's date. Ensure
  release/build serial provenance represents catalog changes consistently across publication
  and tarball/Flatpak builds; if existing source serial is the app commit, document that limit
  and fix the source provenance consistently rather than invent a catalog date.
- R4: Keep title/subtitle legible within the current header, preserve card/category space and
  controls, and inspect a headless Discover PNG with the strip visible. After all visual fixes
  the UI worker refreshes the five shared README/site assets (074), with deterministic sample
  catalog timestamp if the fixture needs one, disclosed as sample state.
- R5: Offline tests cover two separate startups within 24 hours both checking, no repeated check
  within one startup, opt-out, failure fallback, newer remote adoption and displayed timestamp
  update. Format tests cover UTC, month/year boundaries and unknown serial. Use the existing
  fake transport, no network tests and no new dependency unless justified.

## Ownership and verification

Backend worker owns services and core catalog startup/date formatting, build/publish serial
plumbing if required and focused tests. Existing UI worker owns runtime header binding, Slint,
headless layout and screenshots; coordinate the shared boundary before editing runtime.rs.
Read 019, D-025 and the 075 audit first. Never open visible windows or capture the screen.
Merge main before final checks. Jobs2/threads2, targeted tests during iteration; coordinator
runs the combined workspace checks. Report docs/reports/077-catalog-startup-and-date.md with
source semantics, checks and limitations. No subagents/push/tag/release or coordinator-doc edits.

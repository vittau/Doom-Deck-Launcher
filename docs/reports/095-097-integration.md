# Shared compact browsing and publication follow-ups

## Delivered behavior

Select (physical-keyboard V) switches Discover and Library between Cards and List.
The two tabs share one persisted choice; Cards remains the default when no choice
exists. Switching retains item identity, focus, category and chronology. Compact
rows keep title, availability, authors and installed/favorite information readable,
with five complete rows below the normal Discover category strip at 1280×800.

The preference uses isolated versioned `view.json` state and the existing atomic
writer. One background writer coalesces rapid changes, and normal teardown finishes
its last save after the interactive event loop stops. Existing app/profile/install
schemas stay unchanged. No dependencies or Steam Input code were added.

The release-state follow-up adds the immutable `v0.0.12-catalog` fixture from the
tag's actual serializers. Its 15 files join the guard's eight releases; the 99 older
fixture files remain byte-identical. Reports 095 and 096 contain per-requirement
tests and serializer provenance.

## Review and integration

- Reviewed input suppression, identity preservation, writer bounds and teardown
  ownership; sent queue/normal-exit and copy fixes to the same UI worker before delivery.
- Inspected production headless Cards/List renders, long titles, scrolled focus,
  Library section transitions/local content and the crowded footer. Showed compact
  previews to the owner before merge. The final category-strip render proves five
  complete rows including the renderer inset.
- Integrated on `verify/095-097`, keeping `main` stable until checks passed:
  `aad0d0a` (095), `de3a682` (096), `fdb29d2` (097). Services merged without
  conflict; the fixture task touched only the existing guard's tag/audio sets.
- Final host checks ran on the combined application code at `fdb29d2`; later
  coordinator edits are documentation only.

## Combined host verification

All checks passed on macOS arm64 without opening a native window or capturing the
desktop. UI journeys use the Slint software renderer and isolated fixture data.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `CARGO_BUILD_JOBS=2 cargo clippy --workspace --all-targets -- -D warnings` | Passed |
| `CARGO_BUILD_JOBS=2 cargo test --workspace -- --test-threads=4` | **984 passed, 0 failed, 0 ignored** across 22 result groups |
| Eight-release saved-state guard | Passed in the combined suite |
| 1,000-entry layout/model/focus regression | Passed |
| Gated 2,000-change persistence/normal teardown regression | Passed |
| `git diff --check` | Passed |

The UI worker independently passed the full 984-test suite before integration.
The combined run took 603.4 s; this is test-suite elapsed time, not an application
or Steam Deck performance measurement. Local logs and preserved preview PNGs are
under `.scratch/integration-095-097/` and `.scratch/095-preview/` respectively.

## Published catalog delivery and Pages repair

The inherited publication exported its signed repository and site, but two runs
exceeded the former Pages wait and skipped purge and canonical-file comparison.
The reviewed repair uses 120 polls at the same 10-second cadence (nominal ~20 minutes
plus API time), preserving exact-commit matching, immediate failure on an errored
matching build, finite failure, prefix scope and byte verification. The job retains
its existing 120-minute cap.

[Run 39802331079](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39802331079)
used the reviewed workflow at `9ae7e5b` while building the immutable
`v0.0.12-catalog` application at `409d6c504da02cc69f1c4b39030293b44acfb40a`.
It passed signed smoke (`ddl 0.0.12`), matched Pages commit
`423e09eaf2007a9a7fb93630517a18ba1546da10`, executed the Cloudflare prefix purge
and passed canonical page/descriptor/repository-summary byte comparisons. Pages
built in 28 s and the wait took 31 s: the repaired workflow ran successfully,
but this run did not exercise the expanded ceiling. The site serves v0.0.12 dated
2026-10-22. The earlier
[Linux e2e 39797198619](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39797198619)
also passed for that published delivery.

The published tag remains immutable and does not contain the later compact view.
No new application release or tag was created for this integration.

## Integrated-source Linux checks

Pending the main push after the successful combined host checks. The read-only
OpenCode worker will report workflows for that exact source commit; publication
of the older immutable tag is separate evidence and is not a Linux test of the
new compact UI.

## Remaining validation and resource ownership

Physical Select mapping, LCD legibility, Gaming Mode navigation and perceived
responsiveness remain **Needs Deck Validation** (CV1–CV5 in `deck-testing.md`).
No Windows/macOS port work or new `/idgames` browse implementation was started.
The latter's responsiveness requirements are recorded separately in specs 051/053.

The fixture worker's terminal was released. Orca marked the settled UI terminal
`user_owned` / `user_takeover`, so its terminal and worktree are preserved rather
than forcibly closed. The existing OpenCode conversation is reused for read-only
integrated-source CI verification; no settled resource lacks an ownership decision.

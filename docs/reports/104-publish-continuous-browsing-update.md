# 104 — Continuous browsing update publication

## Current state

Publication is being prepared; **0.0.14 is not yet claimed published**.
Reviewed continuous scrolling and favorite-first Library behavior was merged as
`724a99f` after the focused evidence in report102. Missing screenshot research is
still running under spec103 and will be reviewed before the final integration.

Workspace/lockfile package versions and checked-in AppStream now name `0.0.14`.
The intended new immutable prerelease is `v0.0.14-browse`; previous tags stay intact.

## Verification and publication

Version/helper checks passed locally: 27 dependency-free Python helper tests,
the served-URL shell tests, XML parsing, workspace/lockfile/AppStream equality
at 0.0.14, `git diff --check`, and unmodified `appstreamcli validate --no-net`
(successful with one pedantic hint). The metainfo helper was run locally with
the UTC date 2026-10-22; the existing 0.0.13 and older releases were preserved.

Pending: final asset batch review/lint, integrated ordinary Linux CI,
paired release/Flatpak runs, canonical provenance and checksum checks,
and the tag-specific saved-state fixture. No exhaustive test replay or network e2e
is planned for this UI/asset/metadata update. Exact results will replace this
preparation record as they become available.

## Remaining physical checks

**Needs Deck Validation:** controller repeat/scroll feel, real trackpad/wheel/drag
inertia, actual Discover displayed version and upgrade preservation of the owner's
library/settings. See the CB checks in `docs/deck-testing.md`.

No dependencies or protected project status/decision/journal/overview files changed.

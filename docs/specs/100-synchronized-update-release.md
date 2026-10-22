# 100 — Publish the current launcher and Flatpak from one source commit

**Status:** In progress · **Depends on:** 095, 096, 097, 098, 099
**Origin:** owner, 2026-10-22: take over orchestration, finish and commit the outstanding
work, push and release as necessary, and provide an updated Flatpak. The prior
coordinator handed over the unmerged `spec-098-catalog-update-needs-newer-ddl` branch.

## Goal

Publish a new experimental update with shared Cards/List browsing, actionable
catalog-version messages and accurate Flatpak AppStream version metadata. Both the
GitHub prerelease and signed Flatpak publication must use the same immutable source SHA.

## Requirements

- **R1 — Integrate reviewed work.** Merge the completed catalog-message branch (098) and
  AppStream repair (099) with current main in a verification branch. Preserve their tests
  and release history; do not change unrelated product behavior.
- **R2 — Distinct update.** Set the workspace and lockfile package versions to `0.0.13`,
  synchronize the checked-in metainfo, and use the new prerelease tag `v0.0.13-update`.
  Leave all existing release tags immutable. Add release notes and Deck checks for the
  newly published browsing preference, catalog-version messages and displayed package version.
- **R3 — Verify before publication.** Run formatting, denied-warning workspace Clippy,
  the full workspace tests, helper tests and AppStream validation on the combined source.
  Push only after local checks pass, then verify Linux CI before tagging the checked source.
- **R4 — Matching publication.** Push the tag once to start the existing GitHub Release
  and signed Flatpak workflows. Verify both run on the tag's resolved Git SHA, both
  complete successfully, and the release assets/checksum and canonical site/repository
  serve the new update. Inspect the exported AppStream version and the Flatpak commit's
  recorded Git SHA (the export already stores it in its commit body). Extend the existing
  signed smoke test to compare the installed AppStream version with Cargo and check that
  the installed commit is the exported ref with the expected Git SHA in its commit body.
  This is evidence
  for this publication, not permanent enforcement of future paired releases.
- **R5 — Durable outcome.** Record commits, run URLs, verification and remaining Deck
  checks in `docs/reports/100-synchronized-update-release.md`. After tagging, extend the
  saved-state fixture guard using that tag's own serializers, including the shared view
  preference introduced after v0.0.12. Keep previous fixture bytes unchanged.

## Boundaries

Do not edit `docs/status.md` (explicit handoff instruction). No force-push, existing-tag
rewrite, remote-ref deletion, stable release or visible window. No application or Python
dependency additions; install the OSTree CLI in CI for read-only provenance verification.
Keep the existing signing, cache purge and canonical verification gates. A broader
permanent release/Flatpak orchestration change remains a separate future task.

## Acceptance

1. The combined code, version `0.0.13` and metainfo pass all local and Linux checks.
2. Both release workflows succeed for exactly the new tag SHA, with signed Flatpak
   version `0.0.13` and a checksum-verified GitHub tarball.
3. Canonical served files match publication output, and Flatpak export provenance
   identifies the same Git source SHA as the tag.
4. The durable report distinguishes verified CI/publication results from physical Deck
   checks that require the owner; the post-release serializer fixture guard passes.

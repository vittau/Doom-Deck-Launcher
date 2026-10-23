# 108 — Publish the reviewed local-content and variant-clarity update

**Status:** In progress · **Depends on:** 104, 106, 107
**Origin:** owner, 2026-10-23: publish the reviewed changes now; the owner is the only current
user and explicitly accepts publication before physical Deck checks.

## Goal

Deliver the reviewed local-content polish (106) and recipe-variant clarity (107) in one
experimental update, together with the ten recipe screenshots and the Headless Chicken MBF21
correction that already sit on `main`, with matching version metadata and source provenance
on both public channels.

## Requirements

- **R1 — Distinct version.** Workspace and lockfile packages plus the checked-in AppStream
  release become `0.0.15`; the new immutable prerelease tag is `v0.0.15-local`. Previous
  releases, tags and fixtures stay untouched.
- **R2 — Proportionate checks.** Reuse the reviewed 106/107 evidence (995 ordinary offline
  tests with the documented exclusions, eight affected follow-up cases, fmt and
  denied-warning Clippy). Run only offline helper/XML/version/whitespace/link checks for the
  metadata edit and one ordinary Linux CI run on the integrated source. No full-suite replay,
  title-fit matrix, network e2e, local SDK build or manual Flatpak build.
- **R3 — Paired publication.** Tag the green integrated commit once so both pipelines build it;
  keep every existing release and signed-Flatpak gate. Verify both runs used the identical
  source SHA, the displayed Flatpak and binary versions are `0.0.15`, the served repository
  records the same source commit, and the release tarball checksum matches. Never execute
  Linux artifacts on macOS.
- **R4 — Durable result.** Write `docs/reports/108-publish-local-content-update.md` with
  source/version, reused evidence, run URLs, publication provenance and remaining physical
  Deck checks in English. After tagging, add that tag's own serializer fixture and guard
  with focused generation/read checks; preserve every previous fixture byte.
- **R5 — Honest state.** Update `docs/status.md` and `docs/journal.yml` so the published
  checkpoint, the source-only follow-ups and the pending Deck checks are accurate.

## Boundaries and acceptance

Coordinator owns version, tag, publication operations, the release report and the protected
status/journal update. No behavioural code change in this task, no dependency, no stable
release, no force push, no tag mutation, no remote-ref deletion, no visible window or desktop
capture. Do not edit `OVERVIEW.md` or `docs/decisions.md`.

Accepted when ordinary Linux CI is green on the integrated source, both publications match the
new tag/source/version, canonical output and tarball checksum are verified, and the new fixture
guard passes. Physical controller feel, local-folder visibility, variant-row contrast and
upgrade preservation on the owner's Deck remain **Needs Deck Validation**.

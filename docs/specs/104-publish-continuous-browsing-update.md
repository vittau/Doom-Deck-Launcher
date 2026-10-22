# 104 — Publish continuous browsing and reviewed screenshot additions

**Status:** Published and verified; physical Deck checks pending · **Depends on:** 099, 100, 101, 102, 103
**Origin:** owner, 2026-10-22: finish, commit, push and publish updates as necessary;
followed by requests for continuous scrolling, Library favorites first and an OpenCode
worker to try missing screenshots.

## Goal

Deliver the reviewed browsing improvements and any verified screenshot additions in one
experimental update, with accurate Flatpak version metadata and matching source provenance.

## Requirements

- **R1 — Reviewed integration.** Merge accepted 102/103 changes while preserving honest
  unresolved image searches. Review focused off-screen browsing frames and the image
  contact sheet. Do not claim every missing image was found.
- **R2 — Distinct version.** Set workspace/lockfile packages and checked-in AppStream to
  `0.0.14`, use the new immutable prerelease tag `v0.0.14-browse`, and describe changes and
  physical Deck checks in English. Do not rewrite previous releases/tags.
- **R3 — Proportionate checks.** Reuse reviewed 102 focused test/format/Clippy results when
  the merged source is identical; perform 103 final offline catalog/asset checks and
  version/helper checks for the version edit. Run ordinary Linux CI once on the integrated
  source. No local/CI full-suite replay, title-fit matrix, network e2e or manual artifact
  build for this release; this follows the owner's later direction and spec101 instead
  of repeating spec100's earlier full-test procedure.
- **R4 — Paired publication.** Tag the green integrated commit once and keep all existing
  release and signed Flatpak build/smoke/signing/purge/canonical gates. Check both workflows
  use the identical tag SHA, displayed Flatpak and binary versions are `0.0.14`, and served
  signed repository provenance matches it. Verify the release tarball checksum without
  executing Linux artifacts on macOS. These required publication checks are not redundant
  workspace-test replays.
- **R5 — Durable result.** Write `docs/reports/104-publish-continuous-browsing-update.md`
  with source/version, focused evidence, run URLs, screenshot coverage, publication and
  remaining physical Deck validation. After tagging, add that tag's own serializer fixture
  and its guard with focused generation/read checks; preserve previous fixtures.

## Boundaries and acceptance

Coordinator owns integration/version/release operations and the release report/Deck
checklist. Implementation remains in102/103. No dependencies, stable release, force-push,
existing-tag mutation, remote-ref deletion, visible window or desktop capture. Do not
edit `docs/status.md`, `OVERVIEW.md`, `docs/decisions.md` or `docs/journal.yml`.

Accepted when focused checks and ordinary Linux CI pass, both publications match the
new tag/source/version, canonical output and checksum are verified, and the fixture guard
passes. Physical controller feel, actual Discover display and upgrade preservation on
the owner's Deck remain **Needs Deck Validation** until the owner checks them.

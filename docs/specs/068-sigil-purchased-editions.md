# 068 — Verify purchased SIGIL editions

**Status:** In progress. **Origin:** owner, 2026-10-21: verify hashes and correct support for
the supplied SIGIL USB 1.23 and SIGIL II registered 1.0 directories.

## Requirements

- **R1:** Read the owner's files under `/Users/vitor/Downloads/SIGIL_USB_IMAGE_V1_23` and
  `/Users/vitor/Downloads/SIGIL_II_REGISTERED_V1_0` without modifying them. Inventory relevant
  WADs and supplied documentation, compute exact lengths/SHA-256 and inspect WAD directory
  metadata to distinguish maps, music-only replacements and combined registered releases.
  Do not execute WADs/tools from these directories or copy proprietary content into Git.
- **R2:** Compare the local editions with SIGIL/SIGIL II recipes, user-supplied import validation
  and launch resolution. Establish the correct map/music load order, episode variant and
  registered/compatibility interactions from bundled documentation and pinned DSDA research.
  Do not infer roles from filenames alone. Keep audio playback as Needs Deck Validation.
- **R3:** Correct verified metadata, known hashes and supported component/import behavior where
  needed so these purchased editions are handled accurately. No automated paid-content download,
  rehosting, binary fixtures or soundtrack bundling. Preserve free-edition behavior and safety.
  If supporting a combined registered WAD requires a new recipe composition/schema or a change
  in the product interaction, report the concrete finding to the coordinator before expanding
  implementation; bounded known-hash/metadata corrections and regression tests are in scope.
- **R4:** Add meaningful offline tests for recognized identities/composition and rejected mismatches
  using synthetic fixtures, not the owner's paid assets. Unit tests must not depend on Downloads.
  Record local read-only inspection separately from reproducible tests. Update the SIGIL research
  and report at `docs/reports/068-sigil-purchased-editions.md`, citing supplied document names
  and inspected hashes; distinguish confirmed metadata from unplayed compatibility claims.

## Ownership and checks

A fresh OpenCode worker owns this task/worktree, with catalog/sigil*.toml, corresponding research,
import/resolver core paths only when R3 needs them, and relevant offline tests. No coordinator
docs, Settings/audio UI, other recipes, workflows or site edits. Merge main before final checks.
Current workers: 067 owns soundfont core/Settings/config/launch integration, 064 is in coordinator
integration verification, 063/page follow-ups own packaging/site. Report conflicts before editing
shared launch code. Never open visible windows or capture the screen — headless only.
No subagents; no push/tag/release or main merge. Use CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2,
pipefail, targeted tests while iterating and final required workspace checks once. Metadata-only
changes can state why no additional UI test is necessary. Send findings early, before large work.

# 078 — Preserve the v0.0.11 release state

**Status:** Ready after the coordinator publishes `v0.0.11-polish`.
**Origin:** spec 055 R4 and the release lifecycle in `docs/orchestration.md`.

## Requirements

- R1: Extend `tests/fixtures/state/generate.py` only after reviewing the new tag's actual
  serializers. Generate `tests/fixtures/state/v0.0.11-polish/` offline from that immutable tag,
  using the established scratch-checkout scenario and path normalization. Do not hand-author
  machine-state JSON or include owned game data, credentials or host-specific user paths.
- R2: Include the Default audio preferences introduced before this release. Replace the
  generator's exact `v0.0.10-audio` audio condition with an explicit reviewed set containing
  both audio-capable tags; retain behavior for all older tags.
- R3: Add the new tag to the release-state readability guard and its audio expectations.
  Preserve frozen launch recipes and the existing distinction between installed metadata and
  current display tags. Do not weaken validation or rewrite older fixtures.
- R4: The coordinator must confirm the tag and exact commit before fixture generation.
  Preparation can proceed beforehand; use one blocking orchestration question for that gate.
  Run the generator, the complete release-state guard, formatting and relevant Clippy checks,
  and report the actual source commit and results in `docs/reports/078-release-state-v011.md`.

## Ownership and constraints

The worker owns the generator, new fixture tree, the narrow release-state test and its report.
No product behavior, dependencies, existing fixture rewrites, coordinator documents, push, tag,
release, subagents, visible windows or screen capture. Use jobs2/threads2 and retain the tagged
scratch checkout until its output and provenance are reviewed. The coordinator integrates and
cleans up after verification.

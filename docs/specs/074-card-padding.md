# 074 — Give game cards and base-game art breathing room

**Status:** Complete — published in v0.0.11-polish. **Origin:** owner visual review; highest priority of the new fixes.

## Requirements

- R1: Inspect actual off-screen renders of mapset cards and base-game cards at 1280x800.
  Increase cramped internal spacing so text, status and art have a consistent visible inset.
  Base-game screenshots must not cover the card border or focus treatment. Keep the established
  visual language, grid capacity, readable labels and controller targets; no unrelated redesign.
- R2: Inspect focused and unfocused variants, installed and uninstalled mapsets, long titles and
  base-game art. Cover clipping/border regressions with focused headless geometry or pixel
  assertions where reliable; document visual evidence for the remainder.
- R3: After the final visual changes and integrated catalog, regenerate all five shared README
  and website screenshots with the existing lossless script. Inspect each PNG. Both consumers
  must reference the current shared assets; preserve sample-state provenance.

## Ownership and verification

The existing 071/072 UI worker owns Slint card layout, focused headless checks and screenshot
production. It may also implement the copy-only spec 076 in the same follow-up. The other
worker is researching Pistol start/catalog services and must not edit UI files. Never open
visible windows or capture the screen. Merge main before final checks; jobs2/threads2, focused
checks while iterating and appropriate fmt/clippy at completion. Report docs/reports/074-card-padding.md
with before/after headless paths, observed bounds and verification. Commit on the task branch;
no main merge, push, release, new dependencies or subagents. Coordinator handles integration.

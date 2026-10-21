# 064 — Direct Play actions, UI spacing and detected engine resolution

**Status:** In progress · **Origin:** owner's direct brief to the button-actions worker,
2026-10-21; recovered by the coordinator from that worker's status message.

## Requirements

- **R1:** Home Continue starts the remembered game/profile directly. Detail's main action starts
  the selected game/profile directly, with an accurate Play label; Edit still opens Play setup.
  Preserve missing-IWAD, missing/changed-file, engine and profile validation; errors stay actionable.
- **R2:** Ember red is reserved for actions that actually start a game. Other actions, including
  New custom setup and Settings actions, use the regular style. Amber remains focus-only. This
  owner request supersedes spec 060's red primary-action rule.
- **R3:** Base-game cards use the corresponding game's menu/title art sourced from DoomWiki.
  Record source, credit and provenance beside each image. The owner's explicit request permits
  these original-game images as a narrow exception to D-020's no-id-assets rule; do not extend
  that exception to fonts, branding or other artwork. Existing image limits/lint still apply.
- **R4:** The Start button footer hint says Quit. Library cards have internal padding around
  title, text and tags. The current tab's content header title has a small left inset.
- **R5:** Home keeps the last game and removes the lower Recently played section; history storage
  is unchanged. Diagnostics/About removes the screenshot-credit list, shows attribution to
  Vitor Machado and the repository URL (English repository text). Preserve required image
  attribution in a suitable detail/source location rather than silently discarding it.
- **R6:** DSDA's managed template sets automatic aspect ratio, with the existing three-way
  migration preserving player edits. Use the actual detected fullscreen drawable dimensions
  when launching DSDA, including external displays; do not infer resolution from logical UI
  coordinates. Confirm DSDA keys/CLI flags against pinned v0.30.1 upstream source and cite them
  in research/report. Values must be positive and bounded; unavailable/windowed measurements
  keep the existing fallback. Display launch overrides are transient and passed as separate argv
  tokens, never a shell string.
- **R7:** Use DSDA's hardware renderer by default. Confirm the persisted key and string against
  pinned v0.30.1 source before implementation. Include it in the new managed-template version,
  test new/defaulted profiles and migration, and preserve explicit player renderer choices.
  This concerns the engine, not Slint's renderer. Owner measured on a Steam Deck attached to
  a 1080p TV at 120 fps: software CPU power about 4–5 W; hardware CPU power about half or less
  (2026-10-21, owner-reported hardware measurement). Do not present CPU power as total system
  power or an established battery-life multiplier.

## Ownership and verification

Worker: existing `button-actions` agent/worktree. It may edit the UI/runtime/services and core
launch/config paths required above, relevant tests, screenshot assets/provenance and research.
Report at `docs/reports/064-button-actions-and-display.md`. No coordinator-owned docs, packaging
or workflow edits. The Flatpak worker owns packaging/workflows concurrently; merge main before
final checks. No push/tag/release or direct main merge.

Tests cover direct Play and validation failures, selected-profile use, red/focus styles, footer
and Home content, padding via headless renders, template migration preserving user edits,
resolution argv/fallback, fullscreen detection and hardware-renderer migration. Targeted tests while iterating, then fmt,
clippy with warnings denied and full workspace tests once at the end. Report any requirement
without a practical automated test and why. Headless UI PNGs must be reviewed before integration.

Never open visible windows or capture the screen — headless renders only. No subagents.

**Needs Deck Validation:** detected engine resolution in Gaming Mode on the internal display and
an external TV, rendering aspect and return focus after direct Play.

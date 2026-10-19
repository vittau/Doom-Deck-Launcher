# 031 — Profiles you can see, name and switch easily

## Changes

- Mapset detail shows the active profile, skill, pistol start, enabled add-on
  labels and extra-file count. Edit opens Play setup with the profile row focused.
  Base-game availability shares one line to make room; source/verification and
  content-update save warnings remain in the scrollable detail panel.
- Replaced the preset-name chooser with a real Slint TextInput. New profiles
  retain the existing numbered-name creation API, then immediately open the field
  with the name selected; Back keeps that default (coordinator clarification).
  Rename changes only the display name. Schema, IDs and save paths are unchanged.
- Added core `profiles::validate_name`: trims, counts Unicode characters (1–32),
  checks lowercase names within the mapset and excludes the profile being edited.
  Both UI submission and service persistence use it. Existing documents load
  unchanged, including every former preset name. Numbered-name generation also
  avoids case-insensitive collisions.
- Profiles appear in one list with a bullet marking the active profile, followed
  by Previous/Next and grouped creation/rename/delete actions. Three rows fit at
  once; D-pad navigation advances the visible slice for longer lists. The sheet
  opens focused on the active profile. Previous/Next and L1/R1 wrap in both
  directions; shoulder buttons also switch on Play setup's focused profile row.
  The footer shows L1/R1 Profile and uses spec 030's existing tab gating.
- Added the small keyboard launcher in `osk.rs`, nested under runtime to avoid
  touching spec 029's `main.rs`. Focus requests `steam` with the single URI argv
  `steam://open/keyboard`, using Command with separate arguments on Linux and a
  trait fake in tests. Tests and non-Linux hosts do not launch Steam. The helper
  is reaped off the UI thread; the fallback hint always remains visible.
- Removed unused preset copy/helper code; introduced translatable profile UI
  strings in the existing resources. No dependencies added.

## Acceptance evidence

| Criterion | Verification |
| --- | --- |
| A1 / R1, R5 | `headless::journeys::profiles_isolation_and_continue` navigates Right from the detail profile action to Edit, confirms, and asserts Setup/profile focus 60 and disabled tabs. Inspected 1280×800 software-renderer PNGs: `031-detail-profile.png`, `031-profile-list.png`, `031-profile-list-scrolled.png`. |
| A2 / R2 | `cargo test -p ddl-core profiles:: --quiet`: 12 passed. New core cases cover whitespace, blank names, 32/33 Unicode characters, case-insensitive collisions, per-mapset scope, excluding self, and old preset-name disk round trips. Service profile lifecycle test checks trim, duplicate rejection, and equality of all non-name fields and directories after rename. |
| A3 / R3 | The production headless journey replaces the launcher callback with a fake, drives the actual field focus, and asserts exactly `steam` plus `["steam://open/keyboard"]`. An injected NotFound error leaves typing/submission usable. Synthetic WindowEvent text replaces selected text and includes x/y/brackets without triggering root shortcuts. Inspected `031-name-input.png` and `031-name-invalid.png`. Research and pinned SDL source: [steam-osk.md](../research/steam-osk.md). |
| A4 / R4 | Same headless journey checks both wrap boundaries, Previous/Next buttons, L1/R1 in the sheet and focused picker, persistence/Continue and isolated runtime dirs. It also checks creation Back keeps numbered defaults and D-pad reaches all five entries in a longer list. |
| A5 | Final workspace checks recorded below. C4 keyboard/naming on a physical Deck remains Needs Deck Validation. |

Render artifacts are local under
`target/headless/profiles_isolation_and_continue/`; generated PNGs are not committed.
Only off-screen software rendering was used; no visible windows or screen capture.

## Final checks

After merging local `main` (merge commit `fee408e`):

- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed, 538 tests including doctests (126 ddl unit/headless tests and 381 core unit tests; zero failures).
- Final detail PNG re-inspected after preserving verification/update notices;
  profile summary, Edit, source and verification all fit in the viewport.

## Remaining validation / proposals

**Needs Deck Validation:** automatic overlay appearance from a non-Steam shortcut
in Gaming Mode, Steam keyboard delivery into the focused X11/XWayland TextInput,
dismissal/focus retention, and Steam + X fallback. Upstream demonstrates the URI;
headless tests demonstrate DDL's focus/keyboard-event path, not gamescope/Steam
behavior. See the research note for evidence and limitations.

No implementation requirements are intentionally left open. No changes are
proposed to app-state schema or save layout, and no coordinator-owned status,
journal, decisions or Deck checklist files were edited.

# 095 — Shared compact list view for Discover and Library

**Status:** Implemented; physical Deck validation pending · **Milestone:** M3 · **Depends on:** 008, 010, 030, 055, 057, 090
**Origin:** owner, 2026-10-22: Select toggles Discover and Library between cards and a
more compact list; both tabs share the preference and restore it after restarting DDL.

## Goal

Players can switch between the current illustrated cards and a readable compact list
with one controller button. Discover and Library always use the same saved format.

## Requirements

- **R1 — One shared toggle.** Select toggles Cards / List on the Discover and Library
  root screens, including empty lists and while focus is on their tabs or category
  strip. Changing either tab changes the format used by the other. Cards remains the
  default for new users and older saved state. The preference does not change Home,
  Settings, details, setup or sheets; Select has no toggle effect there, while playing,
  or while a modal is open. Map the ordinary gamepad Select button through the existing
  input boundary; never implement Steam Input integration. One press toggles once,
  with no held-button repeats. Provide a conflict-free physical-keyboard fallback
  and document it in the report and end-user guide.
- **R2 — Compact presentation.** Use single-column, consistent-height rows, clearly
  denser than the existing cards (at least five complete mapset rows in the normal
  1280×800 Discover viewport). Show a readable title plus a concise secondary line
  with the existing author/base-game information where applicable; retain meaningful
  installed/favorite/availability indicators. Artwork may be a small thumbnail;
  existing screenshot credits remain accessible on detail. Keep the DOOM visual
  language, amber focus-only and launch-only red rules. Long names, missing artwork,
  base games and local content must remain legible without overlap or stretched images.
- **R3 — Same content and actions.** Both formats use the same source lists, chronology,
  filters/categories, section order and item identities. Library includes base games,
  installed catalog mapsets and local content, with existing setup/import controls
  still reachable. A opens the same detail/action; existing X/Y/Start actions keep
  their meanings. Up/Down move between rows in list mode; tabs, category changes,
  section transitions and Back remain reachable. Switching format preserves the
  selected item by identity, the current filter and a visible focus position; no
  arbitrary jump to the first item. Preserve a focused non-item control if applicable.
- **R4 — Durable preference.** Persist the shared format promptly through versioned
  JSON and the existing XDG/atomic-write state mechanisms; do not depend on a clean
  application shutdown. Preserve existing installations, profiles, favorites, IWAD
  settings and catalog preferences. Old supported states missing the field default to
  Cards; current release-state compatibility fixtures must keep passing. Failed writes
  retain a usable in-memory view and show the existing appropriate save error. Rapid
  toggles converge to the last choice, and background catalog work must not overwrite
  a newer saved preference. Do not write the whole app state on the UI event loop.
- **R5 — Discoverability and responsiveness.** Show an English Select hint for the
  next format (List view / Card view) only where the action works. Keep the hint row
  within the existing 1280×800 footer. Switching format performs no network request,
  archive scan or unnecessary full library reload. Reuse the existing snapshot and
  item data; changing layout must not create a new job per row or reload full-size
  screenshots just to populate compact rows. Large-list and repeated-toggle tests
  must exercise focus/scroll correctness and bounded view-model/job lifetime.
- **R6 — Visual and behavioral evidence.** Add meaningful offline input, state,
  runtime/navigation and production headless tests for the behavior above. Produce
  headless renders of Discover and Library in both formats, including long titles,
  scrolled focus, mixed Library sections and a crowded footer. Never open visible
  windows or capture the screen. The coordinator reviews the renders and shows the
  compact views to the owner before merging this visual change. Actual Select,
  legibility and responsiveness on the physical Steam Deck remain Needs Deck Validation.

## Acceptance criteria

- [x] **A1 (R1):** input tests prove one Select action per press and no repeats;
  runtime tests prove both tabs share the toggle and all excluded contexts ignore it.
- [x] **A2 (R2, R6):** inspected 1280×800 production renders show at least five complete
  Discover rows, readable long names and status markers, correctly framed artwork,
  mixed Library sections, visible scrolled focus and an unclipped hint row. Cards
  retain their existing presentation. Compact renders are shown to the owner before merge.
- [x] **A3 (R3):** controller/headless journey switches both ways on a non-first item,
  changes tabs and category, opens detail and returns, toggles a favorite and navigates
  Library sections; identity, filters, order and available actions remain correct.
- [x] **A4 (R4):** service reopen restores List and Cards; old saved-state fixtures
  default to Cards; rapid toggles persist the final choice; injected save failure
  remains usable; an older background catalog instance cannot clobber the preference.
- [x] **A5 (R5):** a large synthetic catalog (at least 1,000 entries) and repeated
  toggles exercise a non-first/last item, focus visibility and no accumulating jobs or
  row models; a fake service observes no layout-triggered network or catalog reload.
- [x] **A6:** `cargo fmt --all --check`, workspace Clippy with `-D warnings` and
  `cargo test --workspace` pass after merging main; report tests per requirement and
  record any remaining physical Deck checks without claiming unmeasured performance.

## Ownership and delivery

One supervised Codex UI worker owns app Rust/Slint code, focused tests/fixtures,
the relevant user-guide shortcut documentation and `docs/reports/095-shared-compact-list-view.md`.
No other worker is editing those files. The coordinator owns specs, status, journal,
decisions and overview. Do not edit them; propose any necessary change in the report.
Prefer existing dependencies and state mechanisms; justify any new dependency.
Read AGENTS.md, the relevant sources of truth and this spec before coding. Merge main
before final checks, iterate with focused tests, and run full checks once at the end.
Commit small imperative-subject changes on the task branch. No nested workers, push,
tag, release or main merge. Send milestone heartbeats and report with worker_done.

## Out of scope

The future /idgames browse screen (051–053), new catalog content, Home redesign,
new sort/filter behavior, OS ports and Steam Input configuration.

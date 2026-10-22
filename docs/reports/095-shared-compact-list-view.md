# Spec 095 report — shared compact browsing format

Implementation and headless evidence complete. Final workspace verification is recorded below.
Physical Steam Deck checks remain **Needs Deck Validation**.

## Changes

- Discover and Library share Cards/List, toggled by ordinary gamepad Select through
  the existing gilrs boundary. Keyboard fallback: **V**, released between presses.
  Both boundaries suppress held-button repeats. The footer names the next format.
- List uses one column of 74 px rows on an 82 px pitch. The title uses the existing
  display face; a single secondary line shows base-game availability and authors.
  Installed/profile and favorite markers remain visible. Long text fits or elides
  within its own slot. Rows request no screenshot; artwork credits remain on detail.
- Both formats use the existing chronology, categories and item data. Library retains
  base games, installed mapsets, local setups and the New custom setup/Y action.
  Rows retain their action IDs across formats; overflow catalog IDs avoid category
  and local-control ranges, allowing the 1,000-entry regression to exercise real input.
- A layout switch invalidates only the scroll anchor and view model. It preserves
  focused items, category/tab/setup controls and the category selection, then follows
  focus into the visible viewport. Snapshot adoption separately preserves the focused
  recipe and selected detail recipe, including when those identities differ.
- The Library cards path now preserves negative tab focus during validation. Its
  headless owner-scene fixture explicitly selects the same chronological first mapset
  that its old implicit focus fallback selected, preserving the existing scene checks.

## Persistence and lifetime

`$XDG_CONFIG_HOME/ddl/view.json` (through the resolved application config
path) is versioned JSON: `{"schema_version":1,"format":"list"}` or `"cards"`.
A missing file or missing format defaults to Cards. The existing `app.json`, IWAD,
profile, installation and favorite schemas are unchanged. Catalog writers never write
`view.json`, so an older catalog service cannot overwrite a newer choice. Startup emits
the preference once; later snapshots carry no layout preference and cannot revert it.

The existing atomic JSON writer handles replacement, flushing and permissions. The UI
only replaces a mutex-protected intent and wakes one background writer. There is one
in-flight save, one pending choice and one result mailbox per runtime, with no per-row
jobs. Rapid changes coalesce to the last value. A write failure retains the current
in-memory format and shows the existing storage-error toast. Normal application teardown
calls `finish_preferences` after the Slint event loop returns, joining the writer after
its last pending save; interactive callbacks perform no disk IO or joins. Force-kill
before a write finishes is not guaranteed.

## Requirement evidence

All named tests below are offline; existing production tests may use local loopback
fixtures. No native desktop window or desktop capture was used.

| Requirement / acceptance | Evidence |
| --- | --- |
| R1 / A1 | `input::tests::select_is_edge_triggered_and_never_repeats` checks repeated pressed signals, timer ticks, release/repress and suspended input. `compact_view_scenes_and_identity_journey` sends actual Slint V press/release events and rejects held repeats. `compact_toggle_contexts_empty_lists_and_save_failure_remain_usable` checks empty Discover/Library, tab/category focus, every excluded screen, playing/neutral, sheet, local dialog and local setup/picker views. |
| R2 / A2 | Production 1280×800 renders listed below were inspected. `095-discover-list-category.png` has five complete rows while its category strip is visible; its test counts the renderer's 8 px inset. Long titles, installed/favorite/base-game status, mixed sections and focused scrolled rows fit. Existing card artwork uses its unchanged renderer. |
| R3 / A3 | `compact_categories_favorites_and_shared_root_journey` switches formats on a non-first item, compares order and category selection, saves a favorite through production services, opens detail and returns by identity, changes category, toggles on its chip, changes tabs and navigates base/local section transitions. `compact_view_scenes_and_identity_journey` additionally preserves base, local, tab and New setup control focus across formats. Existing Y/Start/detail/setup handlers remain the action path. |
| R4 / A4 | `compact_preference_survives_reopen_and_stale_catalog_state` restores List/Cards/List through fresh services and Startup, tests missing file/field defaults, then runs an older catalog instance's attempt record and full save without clobbering List. `compact_writer_coalesces_and_finishes_at_teardown_without_ui_tick` tests 2,001 requests and synchronous post-loop finish. `compact_writer_finishes_queued_last_choice_with_one_in_flight_save` gates the first save and proves exactly two writes for 2,000 queued changes. `compact_writer_reports_failed_atomic_save` injects a failed rename, and the runtime context test checks the storage toast and usable List layout. `compact_layout_survives_stale_snapshot_and_preserves_separate_focus_identity` exercises layout and identity across reordered snapshots. Existing release-import/state fixtures run in the workspace suite. |
| R5 / A5 | `compact_thousand_items_repeat_without_reload_or_retained_models` uses 1,000 synthetic entries, focuses entry 777, toggles 24 times, and checks the same snapshot allocation, no pending services command/busy job, visible focus, bounded node counts, no row screenshots in List and release of every replaced model through weak references. Up/Down and detail/Back use overflow identity; Library shares the final format. The gated writer test proves bounded work rather than relying on timing. |
| R6 / A6 | Production scenes use the software renderer only. The coordinator reported inspecting and showing the initial compact previews to the owner before merge, then inspecting scrolled/section/local scenes. Main was merged before final checks; results below. |

## Headless renders

All files are in:

`/Users/vitor/orca/workspaces/Doom-Deck-Launcher/compact-list-095/target/headless/compact_view_scenes_and_identity_journey/`

- `095-discover-cards.png` — existing cards with long-title content.
- `095-discover-list-category.png` — visible category strip, five complete compact rows, crowded footer.
- `095-discover-list-long-title.png` — long focused title, missing base, favorite and installed markers.
- `095-discover-list-scrolled.png` — non-first scrolled focus and footer.
- `095-library-cards-mixed.png` — base-game artwork plus catalog section.
- `095-library-list-mixed.png` — base-game rows and New custom setup control.
- `095-library-list-section-transition.png` — focused base row, section heading and installed mapsets.
- `095-library-list-local-scrolled.png` — focused local setup with its existing unverified-content label.

Initial Discover/Library compact paths were sent to the coordinator as soon as coherent
renders existed. Final files are regenerated by the production headless test.

## Verification

- Main merged in `eaf292d` (includes coordinator documentation through `a8ebba8`).
- Focused input suite: 7 passed.
- Focused `compact_` suite: 10 passed (9 new compact tests plus existing compact-brand test).
- `cargo fmt --all --check`: passed after main merge and final layout adjustment.
- `CARGO_BUILD_JOBS=2 cargo clippy --workspace --all-targets -- -D warnings`: passed after main merge and final layout adjustment.
- `CARGO_BUILD_JOBS=2 cargo test --workspace -- --test-threads=2`: passed; 984 tests across all test binaries/documentation tests, 0 ignored. Full log: `/tmp/ddl-095-workspace-tests.log`.

No new dependencies. No network, catalog scan or library reload is initiated by a
layout toggle. No Steam Input code, release, push, tag or merge into main was performed.

## Remaining validation

**Needs Deck Validation:** physical Select mapping, legibility on the Deck LCD, perceived
responsiveness and actual controller behavior in Gaming Mode, including return from a game.
The synthetic lifetime checks establish bounded models/jobs; they make no measured frame-time,
CPU-power or hardware-performance claim. No implementation requirement is knowingly left open.

The coordinator identified the missing published v0.0.12 release-state fixture and is
handling it separately under spec 096, with combined verification after integration.
This report claims only the legacy fixtures present on this task branch; no fixture
was invented or changed by the compact-view work.

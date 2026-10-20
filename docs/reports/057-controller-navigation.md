# 057 — Controller navigation touch-ups

Contract: [spec 057](../specs/057-controller-navigation-touch-ups.md), R1–R5,
with D-032 governing defaults, tab order and quit behavior.

## Changes

- Read both right-stick axes through the existing gilrs boundary. Scroll deltas
  are continuous, proportional to deflection outside the deadzone, bounded by
  the actual content, and independent of navigation actions. Ignore non-finite
  axes; clear on disconnect and require neutral after suspension/reconnection.
- Route scrolling to the active sheet, long error text, the focused Play setup
  column, or the main content plane. Keep manual offsets across service
  refreshes; reposition when the logical navigation target changes.
- Rank vertical targets by column overlap and nearest row so Advanced reaches
  Skill/Pistol start before Recipe load order. Preserve horizontal navigation.
- Start new profiles and new custom setup profiles at skill 4. Keep persisted
  skills unchanged, including legacy skill 3 documents. Show `Skill 4 (UV)` and
  `Skill 5 (Nightmare)`; retain the existing value style on base-game setup.
- Order tabs Home, Library, Discover, Settings in the renderer, direct tab
  indices, shoulder navigation, fixture shell and keyboard journeys. Keep the
  first-run Discover destination and contextual footer hints.
- B at a tab root opens Quit; Settings right-page B first returns to the
  sidebar and its footer says Back (coordinator clarification). Settings opened
  from another screen also retains that Back level. B cancels Quit; the left, initially focused Quit
  button confirms exit. Child-screen Back behavior and other confirmation
  dialogs retain their existing behavior.
- Refresh the four README screenshots through the headless production journeys.
  Update design documentation/examples that printed the old starting skill.

No new dependencies. The Ember Cathedral assets, typography and geometry are
retained; the setup summary and sheets gain bounded scrolling containers.

## Scroll inventory (R1/A1)

| View | Scrolling implementation / routing |
| --- | --- |
| Loading, First run | Shared main plane; bounded when content exceeds the viewport |
| Home, including Continue, recent/base/custom games and discovery | Shared main plane |
| Discover category area and mapset grid | Shared main plane; category chips retain their existing paged window |
| Library mapsets, base games, custom setups, import action | Shared main plane |
| Mapset detail and profile summary | Shared main plane |
| Play setup options/components/Advanced/skill/load order | Main plane, restricted to left-column nodes |
| Play setup launch summary, files and Play action | Separate right plane, selected by focused node |
| Progress and problem/retry/diagnostics panels | Shared main plane |
| Recovery (spec 055): unreadable-file list, Retry / Start fresh / Diagnostics | Shared main plane; the panel and buttons grow with the list, so a long list scrolls |
| IWAD review and directory browser | Shared main plane |
| Settings Engine, Folders, Configuration, Diagnostics/About | Shared main plane, including folder lists and credits |
| Base-game page, Play setup, copies and profiles | Shared main plane |
| Custom setup new/detail, IWAD choice, compatibility, file/patch order | Shared main plane |
| Local file picker | Shared main plane; complete file/shortcut rows retain existing pagination |
| Quit, cancel install, reset config, folder removal, component change, profile deletion, uninstall | Sheet plane, bounded to the available viewport |
| Profile list/name dialogs | Sheet plane; existing profile-list pagination retained |
| Custom deletion, changed/missing file, saves notice | Sheet plane |
| Long local operation error text | Nested text plane uses the wrapped text's measured height |
| Horizontal overflow in content/setup/sheets | Same planes accept right-stick X and clamp to content width |

The existing category/profile/file-picker page windows do not contain hidden
native scrolling regions; directional paging and focus selection remain intact.

**Proposed:** deadzone 0.35; linear speed up to 720 logical px/s at full
right-stick deflection, elapsed-time integration capped at 50 ms per poll to
avoid a jump after a stall. **Needs Deck Validation:** scroll feel, axis
orientation on the physical device, Play setup order, and B/quit flow.

## Verification and acceptance

| Acceptance | Evidence / result |
| --- | --- |
| A1 | `controller_057_right_stick_scrolls_planes_without_focus_changes` injects gilrs-boundary axis signals and asserts offsets and unchanged focus for the shared plane, setup summary, local compatibility, sheet, long error text and horizontal overflow. Root fixtures use explicit overflowing extents where their short content fits; the real Settings Engine page also verifies service-refresh persistence. `input::tests` verifies deadzone, proportional speed, neutral rearming, disconnect, non-finite values and no right-axis navigation events. **Confirmed (headless)**. |
| A2 | `controller_057_focus_walks_every_multi_item_screen` walks both columns down/up through Play setup options (including Advanced → Skill/Pistol start → load order), mapset detail actions, Settings sidebar/folder rows, base-game actions/setup, custom detail/new setup. **Confirmed (headless)**. |
| A3 | New profile is 4; a persisted legacy JSON file with skill 3 loads unchanged. The new custom setup's actual service-created profile is 4. The production skill test renders both labels and measures their bundled-font bounds in the existing button slot. **Confirmed (headless)**. |
| A4 | `controller_057_tabs_skills_and_root_quit` exercises forward/reverse/direct tab mappings and real B/A-equivalent keyboard events on every root; B opens, B cancels, A on default requests quit. PNGs show the order and left Quit focus. **Confirmed (headless)**. |
| A5 | README images regenerated (`scripts/readme-screenshots.py`; `--check` passes). `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` green after the spec 055 merge. **Confirmed (headless)**. |

Targeted verification already passed:

- `cargo test -p ddl-core profiles --quiet` (14 tests).
- `cargo test -p ddl-core resolver::tests --quiet` (11 tests).
- All three new spec 057 cases, input mapper tests, fixture shell navigation,
  production Play setup/options, base-game and profile isolation journeys.
- `cargo fmt --all --check`.

Merged `main` at `d16e9b2` in merge commit `f147136`, and again at `5873b75`
(spec 055, `Screen::Recovery`), both without conflicts. The coordinator-owned
journal/status updates were brought in by those merges only.

After the spec 055 merge: the right-stick test also scrolls a 12-file Recovery
list (content height grows naturally, focus unchanged), and the footer hint says
Quit on Recovery because B opens Quit there (it has no level left), matching the
coordinator rule that hints follow B on every focus target. The 055 recovery
journey (`saved_state_recovery_with_controller_keys`) still passes, including B
from Settings → Diagnostics returning to Recovery.

## Headless PNGs

All images were rendered off-screen with Slint's software renderer. No native
window or desktop capture was used. The targeted test names regenerate these
artifacts under `target/headless/`:

- `controller_057_tabs_skills_and_root_quit/057-skill-picker-uv.png`
- `controller_057_tabs_skills_and_root_quit/057-skill-picker-nightmare.png`
- `controller_057_tabs_skills_and_root_quit/057-quit-default.png`
- `controller_057_tabs_skills_and_root_quit/057-tab-0.png` (Home)
- `controller_057_tabs_skills_and_root_quit/057-tab-1.png` (Library)
- `controller_057_tabs_skills_and_root_quit/057-tab-2.png` (Discover)
- `controller_057_tabs_skills_and_root_quit/057-tab-3.png` (Settings)
- `controller_057_right_stick_scrolls_planes_without_focus_changes/057-long-error-scroll.png`

Visually inspected the two skill states, focused Quit and scrolled long error.
README assets are `docs/screenshots/home.png`, `discover.png`, `play-setup.png`
and `mapset-detail.png`, produced by `scripts/readme-screenshots.py`.

## Open questions / remaining work

Physical Steam Deck validation remains required by the spec. No new runtime,
network, DSDA configuration, or compatibility claims are introduced.

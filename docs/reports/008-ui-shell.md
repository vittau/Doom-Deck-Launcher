# Spec 008 — Fixture UI shell

## Result and scope

Implemented the native fixture shell in `crates/ddl/`: shared design tokens, embedded Inter 4.1,
an original SVG split-gate emblem, translated UI resources, root tabs, spatial focus, gilrs input,
and Home, Discover, Library, detail, ready/blocked setup and Settings. Start opens the temporary
quit-confirmation sheet; Keep playing is initially focused. B cancels sheets or restores the
invoking screen/control, and does nothing at a root screen.

No `ddl-core` code, user files, protected project documents, or other specs changed. No visible
window was opened and no desktop screen was captured. Fixture options and favorites live only
in memory; Play/Install/Import/Profiles/settings actions do not invoke production services.
Required components and recipe order remain locked and explain why when inspected.

`fixtures.rs` is the replacement boundary for spec 010: `MapsetView`, `ProfileView`, `mapset()`
and `files()` supply the view data; `shell.rs` owns only navigation and presentation state.
SIGIL compatibility is collapsed initially, requires explicit Advanced selection and confirmation,
and replaces rather than combines the two free main WADs. Soundtrack imports remain unwired.
AUGER choices are None / Alternate Blaster / Deckard Blaster, with at most one add-on after main.
Each mapset has independent fixture options. All confidence labels remain unverified.

## Requirement → evidence

| Requirement | Verification |
| --- | --- |
| R1 — tokens, assets, embedded font and OFL | `tokens.slint` reproduces ui.md §2 values; visual comparisons below verify their use. `build.rs` explicitly embeds resources, including the imported `InterVariable.ttf` (actual family name **Inter Variable**). Copied font/art/license retain source bytes; release workflow installs `Inter-LICENSE.txt`. Tarball execution is CI-only, so packaging was reviewed statically. Optional shadows are omitted. |
| R2 — shell, contextual hints, tab/back/menu conventions | `fixture_screens_navigation_and_focus_render_headlessly`: root tab cycle, child-screen shoulder suppression, root B, trapped sheets, safe cancellation and explicit quit callback. A hints follow the focused action; X/Y appear only where handled. |
| R3 — spatial focus, restoration, pointer policy | Same headless test drives grid/list/column movement and return with real synthetic key events, checks header edge bounds, verifies all four ring edges for **every actionable control** on every rendered screen, rejects pointer motion, and activates a control using synthetic pointer press/release. Header movement never wraps; shoulder cycling does. |
| R4 — gilrs input | `stick_deadzone_direction_repeat_and_neutral`, `buttons_map_once_and_hotplug_clears_hold`, `mapper_axes_disconnect_and_focus_loss_require_neutral`, `missing_gamepad_backend_is_idle`. Synthetic signals use gilrs `Button`/`Axis` types at the device-independent mapper boundary; OS event codes/devices are not needed. Direction changes, delay/rate, release, ignored library repeats/right stick, all face/shoulder/Start mappings, reconnect and focus-loss neutral gating are exercised. The UI headless test verifies focus retention on hotplug messages and Slint window activation. |
| R5 — fixture screens and options | Same headless test renders all screens, opens detail/setup from cards, exercises ready Play and uninstalled Install toasts, required-row lock, soundtrack import toast, compatibility cancellation/commit, exclusive AUGER choices and ordered filenames, skill cycling 1–5, pistol toggle, profile isolation and blocked Play explanation. Settings actions are static, as authorized. |
| R6 — translated visible copy | `visible_slint_literals_are_translatable` scans every `.slint` file, allowing only `@tr` and explicit non-copy literals (resource paths, font family, empty strings, keyboard keys). It also audits every view-builder Rust literal against a small proper-name/filename-format/glyph allowlist. English labels/descriptions/status/hints/errors live in `strings.slint`; catalog identities, filenames, URLs and numbers remain data. Existing CLI/logging text is outside UI localization. |
| R7 — fullscreen / windowed | Existing CLI unit tests `no_arguments_runs_fullscreen` and `windowed_flag_is_accepted`; production wiring retains fullscreen default and explicit 1280×800 windowed size. Every headless screen is actually rendered at 1280×800. A visible-window test is forbidden on this host. |
| R8 — headless PNGs and navigation | One thread-local `MinimalSoftwareWindow`, software renderer, synthetic `WindowEvent`s, no display backend/device/network. PNGs and reproducible side-by-side reference comparisons go to `target/headless/` (or `DDL_HEADLESS_DIR`). |

## Visual comparison

Ran the headless test and inspected all eight side-by-side comparisons (implementation left,
reference right). Actual screens are 1280×800; comparisons are 2560×800. The reference is taken
from `docs/design/visual/` where present, otherwise `docs/design/mockups/`. The reference
`download-install.png` is outside spec 008's R5 screen list and is not implemented.

| Actual PNG / comparison | Differences from reference |
| --- | --- |
| `home.png` / `comparison-home.png` | Same two-column hero/readiness and recent-row arrangement. Title starts higher inside the hero without the redundant CONTINUE eyebrow. Recent copy additionally names installed/confidence state. |
| `discover.png` / `comparison-discover.png` | Same category strip and three cards. Selected category uses an amber fill. SIGIL II deliberately shows missing IWAD/not installed, providing the reachable blocked fixture. Card metadata includes install state. |
| `mapset-detail.png` / `comparison-mapset-detail.png` | Same art/action and information columns. Adds an explicit Favorite control and release label; Components and Source are inspectable rows. Profiles is a static fixture action. Uninstalled SIGIL II additionally exposes Play setup next to Install so the blocked screen can be reviewed. |
| `play-setup-sigil.png` / `comparison-play-setup-sigil.png` | Same options/Will launch columns. Advanced starts collapsed as required by normal flow, rather than the expanded reference. Main explicitly says locked instead of presenting a misleading switch. Numbered order and compatibility are inspectable controls. The right preview stays fixed while left options scroll. No decorative header thumbnail. The root tab of origin remains active rather than forcing Library. |
| `play-blocked-iwad.png` / `comparison-play-blocked-iwad.png` | Uses **SIGIL II** as coordinator-authorized blocked fixture. Full legal-ownership remediation is in the subtitle. Add IWAD folder initially focused; a separate unavailable Play control explains the blocker. No false matching-IWAD or launch claim. |
| `play-setup-auger.png` / `comparison-play-setup-auger.png` | Accepted D-016 supersedes the stale incomplete-recipe mockup: actual main/add-on filenames, complevel 2 marked not yet verified, and the three-way exclusive choice. Capture shows Alternate Blaster after main. Skill/pistol share a row; load order is a separate locked inspection action. Play only toasts. |
| `library.png` / `comparison-library.png` | Keeps the two-column installed-card layout and filters. Uses attributed concept art, one Default fixture profile instead of the illustrative two-profile claim, and wrapped installed/confidence/recency copy. Whole cards activate Open; no duplicate nested focus target. Empty filters have a Discover action. |
| `settings.png` / `comparison-settings.png` | Initial focus is the first left section per the written contract, rather than the reference's right Check for updates control. Right/Confirm enters the actions; B restores the left section. Appearance & language wraps to two lines. Sections/actions are static fixture previews. |

Shared differences: flat opaque canvas/panels and steel borders rather than decorative gradient,
rails, arrow notches and shadows; the contract makes shadows optional. The footer uses readable
physical-label text rather than circular button badges. Art crops use Slint's native cover fit,
with an opaque **Original concept art** plaque (wrapped on narrow Library artwork). Four-pixel
amber rings have four pixels of canvas separation and do not resize controls. `quit-confirmation.png`
additionally demonstrates the safe default and trapped sheet; it has no supplied raster reference.

## Dependencies and assets

- **New direct runtime dependency: `gilrs = "=0.11.2"** — required by D-018/R4, maintained native
  controller event library; [versioned upstream documentation](https://docs.rs/gilrs/0.11.2/gilrs/).
  Polling uses `GilrsBuilder`, disables unused force feedback, and tolerates initialization failure.
  No SDL, webview, Steam Input API or controller-layout code added.
- New lockfile transitives: gilrs-core 0.6.8, inotify 0.11.5, inotify-sys 0.1.8, libudev-sys 0.1.4,
  objc2-io-kit 0.3.2, vec_map 0.8.2. These are the native event/platform support pulled by gilrs;
  Linux links libudev. Linux SDK compilation is left to CI, not claimed from the macOS run.
- Existing pinned Slint/slint-build 1.18.1 and test-only png 0.18.1 are reused; no new build/test
  dependencies. The test PNG encoder also composes exact comparison images without Python tooling.
- Unmodified **Inter 4.1** and OFL notice copied from `docs/design/assets/`:
  [upstream v4.1 license](https://github.com/rsms/inter/blob/v4.1/LICENSE.txt).
  Original concept art is copied from that same reviewed design asset directory; no game art/IWADs.

Content facts are **Confirmed** only to the extent recorded in
[reference-sigil.md](../research/reference-sigil.md),
[upstream-sources.md §5](../research/upstream-sources.md#5-reference-mapset--augerzenith-doomer-boards-project-37)
and accepted [D-016](../decisions.md). Complevel recommendations remain **Proposed** and gameplay
compatibility **Needs Deck Validation**; no fixture is called Verified. Engine v0.30.1 is explicitly
a research reference, not a latest-release claim.

## Coordinator decisions and remaining work

The coordinator explicitly confirmed fixture-only secondary action toasts (no picker/persistence),
Library as a fixture screen, D-016 overriding the older AUGER mockup, and the blocked fixture via
SIGIL II. It approved the following starting input configuration, centralized in `input::CONFIG`.

| Open item | Status / owner |
| --- | --- |
| Stick deadzone 0.35; initial repeat 350 ms; repeat 100 ms; poll 16 ms | **Needs Deck Validation**: tune on LCD hardware, test hotplug and default Steam layout for double input (A6). |
| Gaming Mode / gyro-as-mouse / handheld text readability | **Needs Deck Validation**, owner via `docs/deck-testing.md`; no hardware access here. |
| Real catalog/install/IWAD/engine/actions/persistence | Out of spec 008; spec 010 replaces the fixture provider and connects services. |
| Paid soundtrack identities and actual SIGIL/AUGER gameplay | Existing research/hardware gates, unchanged by fixture UI. No import or unknown combination can be enabled here. |
| Linux release binary size | **Deferred to CI by explicit coordinator instruction**: do not run a costly local release build on the shared machine; coordinator records the Linux size from the release job after merge. No macOS measurement is presented as Linux size. |

No non-Deck implementation item remains open. A6 remains unticked. A5's build-size measurement
has the explicit CI follow-up above; its local fmt/clippy/test requirements passed.

## Full-workspace checks

Command outputs below are from the final macOS arm64 run; tests have no network access.

`cargo fmt --all --check` — exit 0, no output.

`cargo clippy --workspace --all-targets -- -D warnings` — exit 0, tail:

```text
    Checking ddl v0.1.0 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-008-ui/crates/ddl)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.52s
```

`cargo test --workspace` — exit 0, 233 tests passed across the workspace and doctests.
Result lines and final tail:

```text
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.48s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.86s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 196 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s

   Doc-tests ddl_core

running 1 test
test crates/ddl-core/src/downloads/mod.rs - downloads (line 22) - compile ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

all doctests ran in 0.49s; merged doctests compilation took 0.27s
```

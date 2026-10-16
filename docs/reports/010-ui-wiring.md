# Spec 010 — UI wiring

Status: implementation complete; final full-workspace checks in progress.
Physical Deck and Linux CI validation remain pending. No visible window was opened and no
screen was captured: every PNG comes from the off-screen Slint software renderer.

## Result

Production now uses core-backed services for startup, first run, engine acquisition, IWAD
identification, catalog installs, saved play options, launch, return, Continue, Library and
Settings. `main.rs` attaches `runtime.rs`; `services.rs` owns the core adapters and filesystem
state. The spec 008 shell and fixture provider compile only under `cfg(test)`.

Every operation runs on a worker thread. Typed events cross a channel; Slint's
`Weak::upgrade_in_event_loop` schedules the completion callback through
`slint::invoke_from_event_loop`. The callback applies snapshots on the UI thread. Transfers
report real received/expected bytes; verification/extraction report stages without invented
percentages. Cancellation is cooperative between bounded reads/chunks, and deferred during
atomic commit. Scan/browser/update/import jobs have the same task/cancel path, accessible
through Start while busy. In-flight network requests and smoke probes may finish or time out
before cancellation is observed.

User-owned data stays in code-resolved XDG paths. Versioned atomic `app.json` stores selected
folders and the installed recipe snapshots; core stores IWAD identity, install manifests,
engine state, config baselines/backups and Default profiles. A launch re-resolves and validates
content, obtains the config run guard, prepares directories/canonical IWAD links, and uses
spec 003's `RealEngineRunner` with separate argv. Input is ignored while launching/running;
return restores Play focus and updates last played. Exit 255 and signal failures show the
engine log location; application details also append to `<state>/logs/ddl.log`.

## Requirement → verification

| Requirement | Evidence |
| --- | --- |
| R1 — startup/services/threading | `services::tests::startup_provisions_owned_paths_and_restores_the_default_profile`, `state_rejects_unknown_schema_and_cancelled_work_does_not_change_user_state`, `unreadable_config_is_a_reset_notice_without_losing_the_catalog`; production headless journey uses the real event-loop proxy to deliver worker callbacks. |
| R2 — engine, folders, IWAD review | `headless::fixture_screens_navigation_and_focus_render_headlessly` calls `headless_production::journey`: fresh XDG setup, reviewed engine transfer, safe cancel confirmation, cancelled engine not registered, Retry, directory browser/B return, bounded scan, ambiguous candidates, explicit Ultimate DOOM choice and identified review. `steam_wiring_tests::startup_discovers_steam_iwads_without_scanning_home_and_browser_offers_the_library` checks the automatic route and picker shortcuts; existing core Steam tests cover native/Flatpak libraries and VDF parsing. |
| R3 — catalog/install/errors | Same journey opens actual Catalog entries, installs generated content over the existing loopback HTTP seam, verifies installed manifest and readiness, and exercises genuine checksum rejection without damaging the previous install. `library::tests::cancellable_install_reports_stages_and_never_commits_a_cancelled_selection` covers Download/Extract/Verify cancellation. |
| R4 — resolver/profile/import | Same journey checks real Summary, skill/pistol persistence, compatibility confirmation cancellation and commit, replacement-only launch order, returning to main, and unknown paid-file import never enabling the component. `services::tests::additive_choices_are_exclusive_and_replacements_remove_the_main` checks AUGER exclusivity. Missing IWAD disables Play and does not call the runner. |
| R5 — guarded play/return | Same journey holds a fake runner while rendering Playing, asserts run lock, ignores keys/clicks, restores Play, records last played and log path, releases guard, and checks exit 255, signal 6 and clean exit. Existing spec 003 integration tests cover actual Command construction/process status/environment/log behavior. |
| R6 — Home/Library | Same journey verifies installed-only Library, Continue reopening the most recently played Default profile, and restoration after service restart. |
| R7 — Settings | Same journey tests safe reset cancellation/confirmation and backup creation, safe folder removal confirmation and preservation of user files. The same journey asserts no automatic update request, explicit offline/update checks preserving the installed engine, and rollback/return using installed state; core manager tests additionally cover guard refusal. |
| R8 — real-service headless journey | One headless platform/WindowAdapter, queued EventLoopProxy, scratch XDG tree, loopback HTTP, generated ZIP, smoke-only test executable and gated fake gameplay runner. No real engine/game/device/display or external network. PNG list below. |
| R9 — polish | Shared footer renders circular A/B/X/Y badges and shoulder/Start pills; production strings use `WiringStrings` translation resources; focused locked load-order row has full viewport/ring assertions. The focused multi-line row fits fully, and the Playing footer offers no active input hints. |
| R10 — fixture boundary | `fixtures`, `shell` and their copy adapter are `cfg(test)` modules; production renders service snapshots. Original spec 008 synthetic navigation assertions remain in the same passing headless test. |

## Authorized core changes

Small additive APIs provide engine/mapset progress and cancellation, cancellable ZIP extraction,
and cancellation during user-file hashing/copy. `test-support` exposes the existing loopback
HTTP/fetcher seams to the binary's integration tests; release builds do not enable it. Production
still refuses HTTP by policy, checks allowlists/redirects, bounds input and verifies digests.

The extended compat journey exposed a **spec 009 bug**: installing a replacement forgot prior
member records although files remained on disk. The coordinator explicitly authorized fixing
`install()` itself. It now retains prior records only when file size and SHA-256 match, logs and
drops invalid records, and repairs a selected invalid member from the verified cache. The
resolver still launches only selected components. The regression
`main_to_replacement_to_main_keeps_valid_records_without_redownloading` covers the round trip,
same-size corruption, record removal and cache repair without another request.

## Scope decisions

The coordinator confirmed these interpretations through Orca ask/reply:

- R1–R10 only; no design-only cache clearing, report export, appearance controls or unlisted
  engine installation. No inactive fixture buttons remain in production.
- One persisted Default profile per mapset; profile row is informational, no CRUD. Recipe order
  stays locked. Advanced compatibility replacement asks for confirmation.
- Call `ensure_ready()` after provisioning. Merge conflicts keep edits and show a config notice
  with reset as the next step. Play remains available when config is usable; unusable config
  blocks Play and exposes reset while retaining the catalog.
- No production favorites, category filters or standalone Library PWAD import: current recipes
  lack metadata and those need a later spec. Discover shows the complete curated catalog;
  Library shows installed mapsets. Component import is implemented under R4.
- Retain shared chrome, tokens, art/Original concept art plaque and the two-column setup.
- Fix the spec 009 manifest retention bug as described above.

## Dependencies

No new external package version was introduced. Added direct dependencies reuse the workspace's
reviewed pinned versions:

| Dependency | Scope / reason |
| --- | --- |
| `serde = =1.0.229` | Runtime, derive strict versioned application JSON state. |
| `serde_json = =1.0.154` | Runtime, serialize/deserialize atomic machine state and recipe snapshots. |
| `tempfile = =3.27.0` | Tests only, isolated XDG/user trees. |
| `zip = =9.0.4` | Tests only, generate an archive served by fake HTTP; exercises the actual extractor. |

Existing Slint 1.18.1, gilrs 0.11.2, png 0.18.1 and core networking/hashing crates are reused.
No UI dependency was added to `ddl-core`.

## Headless artifacts and visual review

Artifacts are reproducible under `target/headless/` (or `DDL_HEADLESS_DIR`). The `010-` prefix
separates production snapshots from retained spec 008 fixtures:

`first-run`, `first-run-engine-install`, `directory-browser`, `iwad-ambiguous`, `iwad-found`,
`mapset-detail`, `install-progress`, `play-setup-ready`, `play-setup-options-scroll`, `playing`,
`engine-failed-toast`, `play-setup-blocked`, `settings`, `library`, `home-continue`, `install-error`.

Progress captures hold the HTTP response; Playing holds the runner; checksum error comes from
actual rejected bytes. The synthetic ambiguous IWAD is explicitly chosen by the user, never
accepted by filename. Host temporary paths and tiny byte counts are test data.

Visual inspection found and fixed footer badge alignment and a toast overlapping lower actions.
The toast now occupies the subtitle band and wraps long log paths; Settings also retains the full
log location. Focused options scroll independently while Summary remains fixed. Final production states were inspected against `docs/design/visual/` and layout references.
The shared palette, typography, art plaque and separated amber rings are retained. Production
omits the coordinator-deferred controls and decorative shadows/arrows. Progress uses a simple
full-width task panel with real bytes and a bar, rather than the reference’s illustrated cover
and invented speed/estimate; no estimate is shown without evidence. Home and Library show
real available entries rather than illustrative recent/profile counts. Settings has the four
implemented sections. First run and review/browser states use the same panel/row geometry.

## Deck / CI follow-up

**Confirmed implementation contract:** paths, launch flags/config ownership, reviewed engine
identity and content identities come from core APIs and the repository research:
[DSDA-Doom v0.30.1 source](https://github.com/dsda-org/dsda-doom/tree/v0.30.1),
[DSDA fact sheet](../research/dsda-engine.md),
[SIGIL reference evidence](../research/reference-sigil.md),
[upstream/content evidence](../research/upstream-sources.md).
No new upstream flags, hashes, compatibility facts or "Verified" gameplay claim were invented.
Complevel recommendations remain **Proposed**; gameplay remains **Needs Deck Validation**.

Owner checks map to [deck-testing.md](../deck-testing.md):

- E1–E3: official HTTPS AppImage install, size/hash, smoke probe and real FUSE/extraction fallback.
- E4–E5: fullscreen handoff, controller ownership, DSDA quit returning focus and gamepad behavior.
- C1–C2: DSDA menu edits survive relaunch; standalone DSDA config stays untouched.
- C3: actual Steam DOOM + DOOM II library discovery, including SD card/Flatpak arrangements.
- C4: core profile isolation remains covered in tests; this UI exposes one Default profile per
  mapset by coordinator decision, so a second named-profile UI check requires a later spec.
- M1–M3: SIGIL E5, compat E3, SIGIL II E6 and proposed complevel 3 behavior.
- M4–M5: AUGER;ZENITH levels/complevel 2 and Alternate/Deckard add-ons after main.
- M6: pistol-start transitions/save behavior; source confirms the flag, hardware proves gameplay.
- I1–I3/S3: no double input, neutral-after-return, hotplug, mouse-motion policy and handheld LCD
  readability. Steam Input remains README-only; no Steam Input code was added.

Linux release build and CI green cannot be established from this macOS arm64 host. No release
build, push, tag or release was performed. Spec A3 remains open for the owner's Deck validation.

## Local verification (checkpoint)

At `ea5088e`, `cargo check -p ddl --tests` passed. The full headless fixture + production journey
passed in 28.25 seconds; six targeted library tests and four service tests passed. Final full
workspace fmt/clippy/test results are pending below; these checkpoint results do not claim CI.

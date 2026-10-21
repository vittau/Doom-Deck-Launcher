# 067 — MIDI soundfonts: implementation report

## Outcome

Spec 067 is implemented: Settings → Audio offers **Default** and a reviewed, optional
**Arachno SoundFont 1.0** install, persists one global choice in versioned JSON, and applies it to
every DDL launch path between engine runs. Missing/damaged installed fonts fall back to Default
with a visible reinstall action; real config/path failures abort the launch with an actionable
error instead of silently using another backend. Final 064 commits were merged before the final
checks, and `main` at `43fc298` was merged into the task branch before the full run.

## What changed

### Reviewed install and preference (`ddl-core::soundfonts`)

- Pinned the author-linked HTTPS Dropbox ZIP, exact archive/member sizes and SHA-256 values from
  `docs/research/midi-soundfonts.md`; source, terms, author and download size are public constants.
- Reused the existing bounded, timed, resumable/cancellable `Fetcher` and `Extractor`. The narrow
  Dropbox redirect policy is unchanged; GitHub's catalog exception is not granted to this source.
- Whole-archive inspection rejects unsafe names, symlinks, duplicate names (EOCD count vs index),
  excessive entry count, expansion and compression ratio before extraction. Only the SF2, Read Me
  and English HTML notice are extracted; HTML is retained as inert bytes. RIFF/`sfbk` length is
  validated and nothing from the archive is executed.
- Installs publish an immutable generation plus an atomic `installed.json` pointer. Failed,
  cancelled or interrupted installs leave the previous generation and preference intact.
- `audio.json` stores `schema_version` and `Option<Choice>`; `None` means no explicit choice yet.
  Download never selects. Arachno selection requires a full verification. `effective()` keeps the
  persisted Arachno selection while reporting a Default fallback when the install is missing or
  damaged; unreadable preferences leave the engine configuration untouched with a warning.
- `cached_font()` supplies the Settings snapshot without hashing on every frame; every launch hashes
  again. Paths are accepted only when absolute UTF-8 without quotes/control characters, matching
  DSDA v0.30.1's unescaped string grammar (spaces are supported). No new dependencies.

### Guarded engine application (`dsda_config`)

- `soundfonts::engine` classifies an existing `active.cfg` honestly: native `fluidsynth` + empty
  font is **Default**, anything else is **Custom**. `keys_for` renders the two owned keys and
  `apply` collapses duplicate audio lines so DSDA's last-occurrence rule cannot override the choice.
- `DsdaConfigManager::apply_audio` writes only `snd_midiplayer` and `snd_soundfont`, preserves
  comments/unrelated/engine-generated keys, backs up before a change, writes atomically, makes no
  write while the run guard is active, and is idempotent when the values already match.
- **Review correction:** it never copies the active file into `dsda/base/<version>.cfg`. The
  baseline remains the template comparison point, so an Audio change cannot turn a player's
  unrelated edits into "DDL-applied" values that a later template migration overwrites.
- Regression covers: player edits kept and reported as a conflict across a template bump, baseline
  unchanged, explicit choice surviving migration, no writes during a run, and idempotency.

### Services, launch routes and Settings UI

- `Services` exposes `AudioState` (explicit choice, cheap installed state, current engine
  classification, warning) and `Command::InstallSoundfont` / `Command::SelectSoundfont`.
- All four launch entry points (UI `Command::Play`, CLI mapset, base game and custom setup) share
  `begin_engine_run`, which applies the choice **before** acquiring the run guard. `prepare_audio`
  returns `Problem::Audio` for path/provision/write failures, so the launch stops with “Check
  Settings → Audio”; a missing/damaged selected font remains the intentional Default fallback.
- Settings gains an Audio section with Default always available, Arachno selectable only after a
  verified install, install/download/reinstall/retry actions, author/size/state, a secondary terms
  action, and concise “used for MIDI music on the next launch” help. No custom import, file picker,
  synth or quality/performance claim. Native mapset `SNDFONT` precedence is untouched.
- The secondary terms action and primary action are fully visible at 1280×800 without focus-follow
  scrolling; a headless assertion now enforces the 506 px visible content bound.

## Verification

All commands used `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2`; pipelines used `set -euo pipefail`.

- `cargo fmt --all --check` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo test --workspace` — passed, including doctests. This is the single final full run.
- `cargo test -p ddl-core --lib soundfonts` — **24 passed**: source constants, synthetic verified
  install/publication, archive/member hash and size failures, RIFF validation, unsafe/duplicate/
  symlink/bounds/ratio rejection, cancellation at two boundaries, retry preservation, pointer
  recovery, explicit/restarted/missing/corrupt selection states, unreadable preferences, path
  grammar, snapshot metadata invalidation and install-presence checks.
- `cargo test -p ddl-core --lib dsda_config::manager` — **20 passed**, including the two review
  regressions above and “template does not own the audio keys”.
- `cargo test -p ddl --bin ddl audio_tests` — **8 passed**: install-never-selects, restart
  persistence, verified absolute Arachno path, explicit Default replacing a custom setting with a
  backup, no-choice custom preservation, missing-font fallback with kept selection, no write during
  a run, and an unwritable config aborting with `Problem::Audio`.
- `cargo test -p ddl --bin ddl headless::audio` — **4 passed**: controller navigation/focus, Default
  selection, installed Arachno selection/restart, visible action bounds, download progress, error
  `Retry`/`Diagnostics`, and the missing-font recoverable state.
- `recipe_change_tests::a_cli_mapset_launch_applies_the_explicit_audio_choice` covers the CLI
  mapset route end to end through the fake engine; all four routes share the same guarded helper.
- Existing 064/066/Flatpak/SIGIL journeys were updated only for the new fifth Settings section and
  re-run; the final workspace run is green.

### Rendered previews (headless software renderer, 1280×800)

- `target/headless/audio_section_renders_and_selects_default/067-audio-settings.png` — default state,
  Terms fully visible.
- `target/headless/installed_arachno_is_selectable_and_persists/067-audio-installed.png` — installed
  and selected.
- `target/headless/a_missing_selected_font_stays_visible_and_recoverable/067-audio-missing.png` —
  warning + Reinstall.
- `target/headless/audio_download_and_error_states_render/067-audio-downloading.png` — progress,
  Cancel and byte totals.
- `target/headless/audio_download_and_error_states_render/067-audio-error.png` — failed download
  with Retry/Diagnostics.

PNGs were inspected: labels, selected state, actions and the Terms row are fully visible at the
baseline scroll position. No visible window was opened and no screen was captured.

## Requirement evidence

- **R1** — Audio section, Default always present, Arachno only reviewed; headless controller and
  focus tests (`headless::audio`).
- **R2** — Optional bounded download, progress, cancellation/retry through existing install
  patterns, no auto-select, duplicate installs serialized by the state lock; service/core tests.
- **R3** — Pinned official source/bytes, narrow hosts, archive/member/count/size/ratio/symlink/
  traversal/duplicate rejection, inert notices, no binaries in Git.
- **R4** — Versioned global choice, restart, missing/damaged fallback with warning and Reinstall,
  download never disturbs the installed font or choice.
- **R5** — All launch paths share pre-guard application; explicit choices own the two keys; custom
  settings preserved until an explicit choice; baseline and unrelated edits preserved; mapset
  `SNDFONT` precedence untouched. Real write failures abort with `Problem::Audio`.
- **R6** — Concise next-launch help, no unavailable Arachno in the selector, valid focus after
  install state changes, standard action styling (red remains launch-only), Terms secondary.

## Unautomatable / Needs Deck Validation

- Audible MIDI/MUS switching between Default and Arachno with the official DSDA-Doom AppImage, and
  switching back to Default, cannot be established by macOS offline tests.
- Gaming Mode controller navigation on hardware and Flatpak sandbox access to the installed SF2
  path need a physical Deck run.
- The 143 MiB author archive was not downloaded or installed during tests; all tests use synthetic
  verified fixtures. The exact reviewed hashes are pinned from research.
- No performance, battery or audio-quality claim is made.

## Dependencies

No dependencies added. Existing `serde`/`serde_json`, `sha2`, `zip`, `tempfile` (tests) and the
downloads/extraction machinery are reused.

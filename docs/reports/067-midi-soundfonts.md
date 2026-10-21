# 067 — MIDI soundfonts: interrupted worker handoff

## Outcome

Implementation is **incomplete**. The coordinator intentionally interrupted this Codex
Dispatch for an owner-requested OpenCode provider handover, rather than an implementation
failure. Standalone core work is preserved; existing Settings/config/runtime/launch files
were never edited because the 064 integration checkpoint had not arrived.

## Completed

- Merged local main at `da63a6d` before work; read updated spec, research, AGENTS,
  relevant OVERVIEW configuration/ownership sections and current decisions.
- Commit `d6c3010` adds `ddl_core::soundfonts` and minimal module registration.
- Pinned author-linked HTTPS Dropbox ZIP, exact archive and three member sizes/SHA-256
  values from `docs/research/midi-soundfonts.md`; reuse existing bounded, timed,
  cancellable Fetcher and Extractor. No new dependencies.
- Whole-archive safety checks reject unsafe names, symlinks, duplicate names, excessive
  count, member/total expansion and compression ratio. Important library detail: ZIP 9
  collapses duplicate names in its index; compare the classic EOCD declared count with
  index length to reject duplicates. The reviewed package is classic, single-disk ZIP;
  ZIP64/multi-disk is intentionally unsupported here.
- Extract only SF2, Read Me and English HTML documentation. HTML remains inert bytes.
  Validate RIFF/sfbk header and declared length. No converter or executable is run.
- Immutable installation generation under resolved XDG data; atomic `installed.json`
  pointer publishes all members together. Failed/cancelled install leaves the previous
  pointer and choice intact; unpublished generations are harmless orphans.
- Separate atomic versioned `audio.json` stores `Option<Choice>`: `None` means no explicit
  Audio choice, allowing later integration to preserve custom engine configuration.
  Download never selects. Arachno selection requires verified production bytes.
- `effective()` hashes selected Arachno fully and returns Default plus recoverable
  warning when missing/corrupt, retaining the persisted selection for reinstall.
- `cached_font()` is intended for background services snapshots. It hashes on cache miss,
  invalidates on member/pointer metadata changes (Unix inode/ctime included), and rejects
  symlinks; `verified_font()` remains a full verification for launch. Keep all these calls,
  install/network/ZIP work off Slint's UI thread. No caller integration exists yet.
- `config_path()` accepts absolute UTF-8 paths with spaces and rejects quotes/control
  characters because DSDA v0.30.1 has no string escape grammar.

## Verification

Last completed command (started before the handover instruction arrived):

```sh
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test -p ddl-core soundfonts --lib
```

**13 passed**, no network, 586 unrelated tests filtered out. Coverage: initial/explicit/restarted
preferences; synthetic verified install; no auto-select; only reviewed member publication;
archive/member hash failure; invalid RIFF/form/length; cancellation before work and at extraction;
retry preserving previous install and choice; interrupted unpublished generation recovery;
selected missing-font fallback; unsafe/unrequested ZIP names; bounds; duplicate and symlink
entries; unsupported state/escaping pointer; installed symlink rejection; DSDA path validation;
snapshot metadata invalidation. Two additional passing tests after `d6c3010` cover metadata
invalidation and cancellation at the extraction boundary and are committed with this handoff.

`rustfmt --edition 2024` was run on new module/tests. No full workspace fmt/clippy/test run
or Slint build/render was attempted. No visible windows were opened or screen captures made.
The cached research ZIP was inspected only for metadata (454 entries, maximum compression
ratio about 31.9, no dot/parent path components); tests do not depend on it. No font or notice
binaries were committed.

## Remaining implementation / handoff guidance

1. Obtain coordinator's explicit 064 integration checkpoint, then merge integrated main
   into this task branch. Main had advanced to `92bea57` when the handover arrived; it
   was not merged, because work was stopped immediately.
2. Implement guarded DSDA config-manager Audio application and custom-setting detection.
   Explicit choices own `snd_midiplayer` and `snd_soundfont`; `None` preserves custom
   values and UI must label those honestly. Preserve comments/unrelated keys/backups,
   no writes while run guard active, and protect explicit choices across migrations.
3. Apply before the existing run guard is acquired on every route. Read-only inspection
   found UI catalog `Command::Play`, CLI `run_mapset`, `run_custom`, plus base-game launch
   paths. `run_resolution` and `run_plan` run after guard acquisition in current main,
   so blindly placing config writes there will fail the guard or violate ownership.
   Re-inspect after merging 064, which owns these files.
4. Add services commands/snapshot audio state and use existing state/download guard,
   progress, cancellation, retry/error flows. Snapshot should use cached verification;
   prelaunch should fully verify selected font on services worker. Treat malformed state
   appropriately and avoid a corrupted audio preference unexpectedly blocking play.
5. Add Audio Settings section, installed selector (Default always available, Arachno only
   verified), honest custom label, download/installed/error states, credit Maxime Abbey,
   size, secondary original source/terms links, concise next-launch MIDI help. No automatic
   selection, custom import or quality/performance claims. Red remains launch-only.
6. Add config/run-guard/custom-preservation/migration and every-launch-route tests, plus
   headless controller/focus tests and 1280×800 Audio/progress/error PNGs; inspect PNGs.
   Existing runtime uses dynamic Settings node construction; no Slint file edits yet.
7. Final checks once after integration, with `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2` and
   `set -euo pipefail` for shell pipelines: fmt, clippy warnings denied, full workspace tests.
   Check coordinator before starting broader builds because host was overloaded.

## Requirement evidence and open items

- R1/R6: UI remains open; no headless renders produced.
- R2: core optional installer/progress/cancel/retry semantics implemented and tested;
  duplicate download guard and visible states await services/UI integration.
- R3: standalone source/safety/atomic publication implemented; transport mechanics rely
  on existing Fetcher offline tests. Final lint and review remain.
- R4: standalone versioned choice, restart, verified-selection guard and recoverable
  fallback implemented; player-facing message/reinstall awaits integration.
- R5: only safe path validation implemented. All config application, custom detection,
  migration protection, run guard integration and route evidence remain open.
- No dependencies added; existing serde/serde_json, sha2, zip and downloads machinery reused.
- **Confirmed:** engine string/config/SNDFONT behavior and source identity are grounded in
  pinned upstream links in `docs/research/midi-soundfonts.md`; this worker did not change it.
- **Needs Deck Validation:** audible MIDI playback/switch-back, official AppImage path access,
  Gaming Mode controls and Flatpak sandbox access. These cannot be established by macOS
  offline tests and were not claimed.

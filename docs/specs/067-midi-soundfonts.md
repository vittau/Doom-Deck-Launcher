# 067 — FluidSynth soundfont selection

**Status:** In progress; standalone core work may proceed in parallel, with Settings/config
integration following 064.
**Origin:** owner, 2026-10-21: Settings → Audio, downloadable Arachno and a selector for
installed soundfonts; the engine's built-in soundfont is labeled Default.

## Requirements

- **R1 — Audio settings:** add an Audio section to Settings using the established controller
  navigation and visual language. Show a MIDI soundfont selector with **Default** always
  available. Initially offer only **Arachno** as an optional download. No file picker, font
  editor, preview player, additional synth or arbitrary soundfont importing in this task.
- **R2 — Optional install:** show Arachno's author, download size, installed/downloading/error
  state, progress, cancellation and retry through existing download patterns. Downloading does
  not automatically select it. Arachno becomes selectable only after the reviewed install is
  complete and verified. Prevent duplicate concurrent downloads. Existing downloads and game
  launches retain their normal state/guard behavior.
- **R3 — Source and safety:** use the official author's HTTPS Dropbox mirror from
  `research/midi-soundfonts.md`, pinned ZIP/member SHA-256 and exact sizes. Reuse bounded,
  timed, cancellable downloads, the narrow Dropbox redirect policy and safe ZIP handling.
  Permit only the hosts needed for this source; do not broaden catalog download authority.
  Stage under DDL's resolved XDG directories and atomically publish only the SF2 and reviewed
  notices. Enforce archive/member/count/expanded-size bounds; reject traversal, symlink escapes,
  duplicates and invalid RIFF/sfbk content. Never run anything from the archive. No fonts or
  sample tracks are bundled, rehosted or committed. Credit Maxime Abbey and link original terms;
  retain supplied Read Me/English documentation as inert files. Tests are offline with fakes.
- **R4 — Persistent choice:** save the global choice atomically in versioned JSON; default
  for a new installation is Default. Restore the choice after restart. A failed/cancelled
  download must not disturb the installed font or current choice. If a selected managed font
  disappears or fails verification, use Default with a visible recoverable message and an
  actionable reinstall option; do not break game launch or pass an unchecked path to DSDA.
- **R5 — Engine integration:** all DDL game-launch paths, including Home Continue, direct
  Detail Play, catalog/custom setups and base games, apply the selected soundfont for the next
  game using FluidSynth. Default means `snd_midiplayer "fluidsynth"` and empty `snd_soundfont`;
  Arachno means that backend plus the verified absolute installed SF2 path. Confirm implementation
  against pinned upstream research; `-assign` cannot set these strings. Apply via the DSDA
  config manager only between runs, preserving unrelated settings, comments, backups and the
  existing run guard. Reject paths containing characters DSDA's unescaped string grammar
  cannot represent. Do not change a running engine. An explicit Audio selection owns these
  two keys; template migrations must not reset that choice. If an existing manually configured
  font/backend is found before a user makes a selection, preserve it and identify it honestly
  as a custom engine setting rather than falsely display Default. Choosing Default or Arachno
  replaces that custom setting explicitly. Preserve DSDA's native mapset-SNDFONT precedence.
- **R6 — Clear interaction:** concise help says the choice applies to MIDI music on the next
  launch. Keep unavailable Arachno out of the installed selector, keep focus valid as install
  state changes, and use regular Settings action styling (red is game launch only, amber focus
  only). Do not add research labels or implementation details to the player flow. Terms/source
  links are secondary. Show no unverified quality, battery or performance claim.

## Ownership and verification

One fresh Codex UI worker owns this feature; core implementation is allowed
in the same task to keep state/config/UI semantics consistent. While 064 is active, start only
the standalone soundfonts module/research/tests; wait for the coordinator's integration checkpoint
before editing existing Settings/config/launch files, then merge main with 064. It may edit a small soundfonts
module under `ddl-core`, config-manager integration, application services/runtime/Settings UI,
relevant fixtures/tests, research and `docs/reports/067-midi-soundfonts.md`. No coordinator-owned
docs, unrelated catalog, Flatpak/workflows or site changes. No new dependencies unless necessary
and justified. Merge main before final checks. Never open visible windows or capture the screen
— headless renders only. No subagents; no push/tag/release or direct main merge.

Tests cover verified install, invalid hashes/content/ZIP paths, bounds, cancellation, retry,
atomic recovery, default/selected/restarted/missing font states, custom engine preservation,
explicit Default/Arachno config changes, paths with spaces and rejected quote/newline paths,
no writes during an engine run, unrelated settings preservation and every launch route.
Headless tests and PNGs cover controller selection, install/progress/error state, focus and
the Audio section at 1280×800. Iterate with targeted tests; final fmt, clippy with warnings
denied and full workspace tests once. State why any requirement cannot be automated.

**Needs Deck Validation:** actual MIDI playback and switch-back on the official AppImage,
Gaming Mode controls and Flatpak sandbox access to the installed SF2. Use research evidence
for engine behavior and do not claim an audible test from macOS-only unit tests.

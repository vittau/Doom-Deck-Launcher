# Task 115 — Per-mapset progress research and proposal

## Outcome

**Proposed — recommendation to the owner.** A small, read-only progress view is
worth considering, subject to Deck validation. Start with DSDA's recorded exits
for the **selected profile and resolved WAD combination**, a detected-map count,
and a detail table using map IDs. Do not promise full campaign completion,
real playtime, last-map resume or trustworthy speedrun records. This report is
a decision input; it does not authorize or implement a feature.

**Confirmed — research completed.** [The evidence document](../research/dsda-stats.md)
answers Q1–Q5 against DSDA-Doom `v0.30.1`, commit
`8ed581ad5c868b4dba1c39eb21d29e4ddbed9fad`, and immutable DDL baseline
`88bfcb77245c61a89874eadb06bb275c3cbb1308`. Its pinned source links cover every
field, lifecycle, directory algorithm, settings interaction, map inventory and
reader risk. Research scope follows [spec 115](../specs/115-dsda-stats-research.md);
D-044's menu-only launch is assumed throughout, regardless of the older
skill/warp code visible at that DDL baseline.

## Q6 — What DDL could show

**Proposed — first slice.**

| Surface | Honest display | Meaning / limit |
| --- | --- | --- |
| Library card | “14/32 maps exited” plus selected profile context | Count distinct stats rows with `total_exits > 0`; denominator is valid detected rows, not guaranteed reachable maps. Includes no-monster exits. Explain “DSDA detected maps; includes secret/unused maps” in detail. Example values only. |
| Detail summary | Profile name, current load combination, “Recorded by DSDA-Doom”; recorded exit count | Never silently union multiple profiles or different organized paths. Show unavailable/stale states. |
| Per-map table | Map ID, exited status, exit count, highest recorded skill | Best skill 0 means no qualifying non-no-monster exit; an exit can exist without a best skill. Use the engine result, not a DDL skill setting. |
| Optional detail columns | Best kills/items/secrets with known denominators | Independent bests; do not imply one run achieved all three. A zero denominator is “none”, an unknown denominator is “—”. Counts above the current denominator must not become fabricated completion percentages. |
| Optional time columns | “DSDA best map time”, “DSDA max time”, “DSDA NM time”, unknown as “—” | Only first-completed-map times qualify; later continuous-play maps may have no time despite exits. Include a concise explanation. Consider deferring these columns to keep MVP legible. |
| Empty state | “No DSDA stats available yet” | A missing/invalid file is unknown, not proof of no play or zero completed maps. |

**Proposed — prefer explicit semantics over a hidden policy.** Use exits for the
first progress count. If the owner wants no-monster exits excluded, an alternative
is rows with `best_skill > 0`, labeled “maps with a recorded skill”; retain exits
separately. Neither policy can certify a legitimate, unmodified completion. The
format loses provenance, so a parser cannot recover the missing guarantees.

**Confirmed — reasons for those limits.** The stats writer has no timestamp,
run history, compatibility/modifier identity or friendly names; exit count and
best skill have different gates; best times require `levels_completed == 1`.
Nonempty loaded rows are not reconciled with changed WAD content.
[Fields/writer](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L234-L246),
[exit rules](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L266-L323),
[initialization](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L333-L338).

**Proposed — do not offer from this artifact.**

- Overall playtime, a continuous campaign time, sum-of-times “run record”,
  last played date/map, deaths, attempts, next-map recommendation or resume.
- Verified UV/Nightmare achievements, proof of pistol starts, proof of no cheats,
  per-complevel records, or attribution of best K/I/S to a single exit.
- Exact campaign percentage for arbitrary WADs; stats inventory may include
  inaccessible/test/secret maps, exclude unchanged IWAD maps, or be stale.
- Correct map titles from filename or base-game slot alone. Use `MAP01`/`E1M1`
  first; metadata parsing or curated names require a separate approved extension.
- Historical separation after same-stem content/patch changes. Suppress uncertain
  attribution rather than relabeling old stats as new content's progress.
- Cross-profile merging by default. A count across profiles would require a clear
  policy for matching map identities and mixed content, with no run-level history.

## Rough size and a possible implementation boundary

**Proposed — estimate, not a commitment.** Small-to-medium, about 4–7 engineering
days plus physical Deck availability: 1–2 days for bounded parsing and safe path
resolution, 1–2 for profile/content attribution and post-process refresh, 1–2 for
card/detail UI and headless tests, about one day for integration fixes and Deck
checks. No new dependency should be necessary for parsing. Safe filesystem opening
must use an established repository primitive or be explicitly designed/reviewed;
this estimate must not excuse a symlink race.

**Proposed — split into reviewable stages.** First approve the vocabulary,
completion policy and identity limitation, then validate DSDA files on Deck.
Only then write an implementation spec for parser/path resolver and selected-profile
UI. Defer WAD/UMAPINFO/DEHACKED parsing, recipe schema changes, aggregate time and
split/levelstat/analysis import. Those expand this into a different project.

## Draft requirements for a later implementation spec

Every item here is **Proposed**, not an accepted product requirement.

1. Read only audited DSDA `v0.30.1` stats version 1; never enable import for an
   unreviewed engine release solely because it still writes version 1.
2. Derive the exact organized path from DDL's actual resolved launch files,
   source classifications and profile data root. Cover lowercase stems, ordered
   PWADs, eight-PWAD cutoff and ignored patches. Reject unsupported/ambiguous
   load layouts instead of guessing ZIP-expanded paths.
3. Use the existing XDG resolver, including sandbox paths. Never hardcode HOME,
   app ID or host DSDA directories. Keep `-data`, save, config and working-directory
   semantics separate; do not add skill/warp/analysis/levelstat flags.
4. Treat the local file as untrusted: bounded anchored no-escape open, regular-file
   checks, actual byte cap, bounded rows/lines, exact header/field counts, checked
   integer conversion, strict version, known sentinels, duplicate rejection.
   Suggested limits: 1 MiB, 4,096 rows, 512 bytes/line, eight-byte ASCII map ID.
5. Refresh after the engine has terminated, preserving the last successful snapshot
   during active play. Distinguish missing, unreadable, corrupt, oversized,
   unsupported and stale data from a valid all-zero inventory.
6. Keep map progress scoped to selected profile and current resolved content identity.
   Detect changed files/components/patches and avoid presenting old same-path
   engine stats as belonging to replacement content. If provenance cannot be
   established, display unavailable with explanation; never alter DSDA files to fix it.
7. Define and test the owner-approved progress rule; proposed default is
   `total_exits > 0`, denominator valid detected rows. Mark denominator as detected,
   include secret/unused maps, and do not call it verified campaign completion.
8. Render map IDs and optional highest recorded skill/exit counts in a controller-
   accessible detail view. If K/I/S or times are included, preserve independent-best,
   changing-denominator, unknown-time and first-completed-level semantics.
9. Persist any optional DDL cache as versioned atomic JSON, separate from engine
   files. Read-only import means no creation, repair, deletion or rewrite of stats;
   no telemetry, network, Steam integration or other engines.
10. Unit fixtures must cover valid zero/unknown values, all 15 fields, LF/CRLF,
    short/malformed rows, signed overflow, future version, duplicate IDs/numeric
    keys, oversize, traversal/symlink attempts and counts above denominators.
    Integration fakes must cover profile isolation, stem collisions, patch/content
    changes, post-exit refresh and error/partial reads. No network in tests.
11. Verify controller focus/scroll and missing/stale states with headless Slint
    rendering/tests; never open windows or capture the dev machine screen.
12. Gate delivery on the Deck checks below and record runtime evidence explicitly.

## Deck checks required before delivery

All items below are **Needs Deck Validation**; none were run on this macOS host.
Use owner-provided IWADs and a known small mapset; preserve original engine files.

- Start from DDL at DSDA's menu with no skill/warp. Quit without starting a game;
  inspect whether a zeroed inventory is written to the expected profile path.
  New Game at two skills, exit normally, relaunch and compare all file fields.
- Complete two consecutive maps with continuous play, then with pistol start.
  Confirm exit counts for both and the predicted difference in time availability.
- Die/retry, load a mid-map save from the menu, re-exit, then start another New Game.
  Verify cumulative kill/exit behavior and restored completed-level time gating.
- Compare UV and Nightmare, no monsters, fast and respawn; verify skill upgrades,
  independent best counts, denominators and NM time. Confirm no claim of cheat-free
  completion is made by DDL's presentation.
- Compare two profile IDs, catalog/base/custom launches, WAD add-ons versus DEH/BEX,
  reordered WADs and nine PWADs. Replace content with the same basename and verify
  DDL reports attribution uncertainty rather than inheriting unearned progress.
- Check ordinary quit, controlled engine error and force termination. Observe
  which snapshots survive; do not assume level exit flushed `stats.txt`.
- Test packaged Flatpak/extracted AppRun and native launch: actual resolved path,
  write permission and readback. Test unwritable data directory and engine fallback
  behavior; DDL must not scan unrelated host directories for a substitute file.
- With disposable fixture copies, test corruption/truncation, future marker,
  symlink escape, oversized file and overlapping engine instances. Observe
  stale/unavailable UI rather than crashes or loss of previous known data.
- Check a single-map replacement, resource-only PWAD, UMAPINFO mapset and a mapset
  with secret/unused maps. Ensure “detected maps” is understandable on Deck and
  controller navigation remains usable at high row counts.

## Open decisions for the owner

These are **Proposed** decision questions for the coordinator to present; work
on this research is complete and no immediate answer is needed to settle it.

- Approve a small read-only stats feature at all? Recommendation: yes, after the
  Deck checks establish the core behavior; no implementation before approval.
- Count every recorded exit or only exits with recorded skill? Recommendation:
  all exits, explicitly named “maps exited,” including no-monster exits.
- Accept DSDA's detected-map denominator (possibly secret/unused/stale), or fund
  a separate curated/validated inventory? Recommendation: detected-map qualifier,
  unknown when identity is uncertain, no percentage/100% achievement.
- Keep scope to the selected profile, with IDs and exits first? Recommendation:
  yes; defer merging, titles and timing columns unless specifically wanted.
- Is conservative suppression after content changes acceptable? Recommendation:
  yes. The read-only engine format cannot retrospectively identify old results;
  automated deletion/reset is outside scope.

## Changes and verification

**Confirmed — task artifacts.** Added only `docs/research/dsda-stats.md` and this
report. No product code, catalog, protected coordinator docs or dependencies changed.
Cloned the pinned tag outside `crates/`, traced call sites with `rg`, inspected
numbered source, checked source paths/revisions/line bounds for all citations and manually compared each serialized field
against load/write/update code. `git merge main` was performed before completion
and was already up to date at the first merge check; final repository state is
recorded in the worker completion message. `git diff --check` is the applicable
repository check. Rust build/test suites are not relevant to documentation-only
research and were not run; no runtime or physical Deck result is claimed.

**Proposed — remaining work.** The research contract is answered; no implementation
requirement is claimed complete. Owner policy choices, implementation spec,
runtime validation and any feature build remain future work. No change to
OVERVIEW, decisions, status or journal is made or required by this research.

# 106 — Responsive local browsing and recoverable custom setups

**Status:** Implemented · Host checks passed; physical Deck validation pending · **Depends on:** 026, 055, 059, 095, 105 · **Decision:** D-041
**Origin:** owner, 2026-10-23: implement report105's manual-content improvements; avoid
DeepSeek V4.1 Flash Free. Physical Deck validation will be performed by the owner.

## Goal

Make occasional local mapsets easy to add, recognize and repair while preserving content
identity, explicit load order, isolated profiles/saves and the curated Discover experience.
This authorizes the routine UX choices below. No archive search, new Discover filters,
engine change, release or network dependency is part of the task.

## Requirements

### R1 — Lightweight folder listing

Separate accepted-file inspection from full identity capture. Local folder listing must use
bounded metadata/header/directory inspection and never calculate full-file SHA-256 or create
fake `UserFile` identities. Keep existing type/suffix/size/regular-file, WAD-directory and
IWAD-header validation, file filtering, shortcuts and enumeration limits. Inspect on the
background worker with cancellation between entries; no unbounded per-file threads/tasks.

Browse completion must retain the current snapshot: do not rebuild/rehash the library after
publishing a folder listing.

Hash and record only selected files when confirmed for addition, including selected local
IWADs. Revalidate at confirmation: a file changed, vanished or replaced after listing must
not acquire a trusted identity from stale metadata. Preserve full identity checks at launch.
Hashing a large selected file must observe cancellation in bounded chunks (reuse an existing
suitable helper or add a small one) so selection/addition and replacement cannot ignore Cancel
for the whole file. A cancelled or failed add does not persist a partial content list.

### R2 — Last successfully browsed folder

Remember the last usable local-picker folder across picker openings and normal app restart.
Use a small versioned JSON preference in the resolved XDG config dir, written atomically on
successful browsing only. Isolate it from app/catalog/view state so other writers cannot
clobber it. Bound reads, handle absent/corrupt/newer-schema preferences with a safe Home
fallback and diagnostics; a broken optional preference must not trigger saved-setup recovery.
Missing/inaccessible/unmounted remembered folder falls back to Home without a retry loop.
Keep Home/Downloads/Documents/Steam/mounted-card shortcuts and parent-folder behavior.
Opening a relocate picker may prefer the old path's parent when usable; it must have the
same fallback. Do not save a failed path as the new preferred directory.

### R3 — Locate a missing file without destroying its slot

A missing local-file issue sheet offers **Locate file**, opening the existing controller
picker in single-file mode. Support missing custom content/patches, a custom file-based IWAD,
and profile-scoped extras on catalog mapsets and base games. Do not modify downloaded recipe
members or registered/identified IWAD paths through this flow; retain existing remediation
for those. Preserve target setup/profile, group, position and unrelated files.

Only explicit selection can replace a reference. Compute/validate the chosen file's identity:
equal SHA-256/size and compatible kind/header updates the path; different bytes require a
confirmation naming the replacement and warning about saves, initially focused on Cancel.
Reject the wrong kind or a non-IWAD selected for an IWAD slot. Do not silently accept changed
bytes, search disks, modify player files, change identifiers/profiles/saves or switch profiles.
Cancel/back/failure leaves the original reference and other records untouched; identity/order
must still hold after reload. Stale UI completion or a changed target must not overwrite a
newer setup/profile. Keep the separate explicit changed-file confirmation behavior.

### R4 — Recognizable setups and plain guidance

The Library entry reads **Add local mapset** (Y retains the same binding and creation flow),
with help that makes the next steps clear: choose a base game, create the setup, then add local
files. Preserve advanced IWAD/compatibility controls and two ordered file/patch groups.
Do not build a second wizard or change catalog browse/sort/filter rules.

New setups created in this UI session may use a generated placeholder until the first content
file is added. On that first successful addition, derive a readable default from the first
non-patch filename stem (Unicode-safe, trim/collapse separators, bounded valid nonempty name;
fall back to the generated name). No guesses from ZIP contents or external metadata. Automatic
naming applies only to a still-unrenamed placeholder created in the current flow/session;
never rename an existing setup, a user-chosen name, a patch-only addition or a profile.
Handle name collisions safely (names need not become filesystem IDs).

Custom detail offers **Rename**, using the existing text/OSK infrastructure and a visible
confirm/cancel path. Validate existing name bounds/nonempty rules; cancellation preserves the
old name. Rename changes the display name only, preserving setup ID, profiles, file order,
saves and history. Update Library/Home/detail identity displays on success. Every new user
string belongs in the existing translatable strings. Footer hints/focus/back behavior must
match the current visible action. Add concise help that files stay in place; moving them
requires Locate file. Do not imply arbitrary GZDoom mods work in DSDA.

## Acceptance

- **A1 (R1):** deterministic tests prove listing does not call the full-file hashing path
  or read large payloads; valid/corrupt/mismatched files retain filtering; confirmation hashes
  selected files and rejects invalid mutations after listing; launch rejects changed bytes.
  Controlled cancellation proves no partial add/replacement. Avoid timing sleeps as assertions.
- **A2 (R1):** before/after host release-build diagnostic on the same synthetic folder with
  valid large WADs/mixed files. Record fixture sizes/counts, listing-ready time, bytes read (or
  instrumented bounded-read evidence), cache/run conditions and cancellation behavior. Host
  results are diagnostic; Deck/SD timings remain Needs Deck Validation. Do not download game data.
- **A3 (R2):** temporary-XDG tests for reopen/restart, untouched view/app preferences, missing,
  corrupt, future-schema and inaccessible/stale remembered paths; failed browse never poisons
  the preference. Keep older tagged state fixtures byte-identical and readable.
- **A4 (R3):** same-byte move, explicit different-byte accept/cancel, wrong group/header,
  missing custom IWAD, catalog/base extras, middle-slot order, profile isolation, stale target,
  reload and missing replacement tests; assert player files/saves/unrelated records untouched.
- **A5 (R4):** filename naming/Unicode/empty/collision/patch-only/custom-name cases and controller
  rename confirm/cancel/invalid-name tests; ID/profiles/saves/history unchanged; Library/Home
  refresh and Y entry guidance tested.
- **A6:** focused off-screen software-rendered frames of the Library entry, custom detail/rename,
  missing-file Locate action, picker and changed-content confirmation. Review them headlessly;
  no visible windows or desktop screenshots. Preserve readable LCD-scale layout and focus.
- **A7:** fmt, denied-warning workspace Clippy and the ordinary offline workspace suite (101)
  pass once on the integrated source. Iterate with focused tests. No exhaustive title matrix,
  unrelated full-suite reruns, network e2e, SDK/Flatpak build or release for this task.
- **A8:** update the user guide/checklist and report106 with requirement evidence, checks,
  benchmark, any dependency justification and remaining physical Deck checks. No new dependency
  unless demonstrated necessary. Update this spec's status honestly; coordinator owns status/journal.

## Ownership and boundaries

One Codex worker owns implementation in `crates/ddl-core/src/local/`, related scoped core
helpers/tests, `crates/ddl/src/` and `crates/ddl/ui/` for these flows, scoped fixtures, the guide,
Deck checklist, this spec and `docs/reports/106-local-content-polish.md`. No other worker edits
these files. Coordinator owns overview/decisions/status/journal/orchestration and final review.
Keep historical fixtures unchanged; additive optional preferences must not invalidate them.
Commit small changes on the task branch; merge `main` before final checks. Never push, tag,
release, add archive search, use Free models, open visible windows or capture the screen.

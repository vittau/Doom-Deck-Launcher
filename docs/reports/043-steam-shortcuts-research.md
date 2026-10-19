# 043 — Steam shortcuts research — worker report

**Status:** research complete 2026-10-19. **No code.** R1–R2 are in
`docs/research/steam-shortcuts.md`; this report answers R3.

**Recommendation — README-only.** Manual, advanced per-mapset Steam shortcuts belong in
documentation, not in DDL code. **Automatic in-app shortcut writing is a no-go for now.**

## What was delivered

| Item | Deliverable |
| --- | --- |
| R1 — mechanism | `docs/research/steam-shortcuts.md` §3: file location, binary format, app ids, artwork names, read/write lifecycle, `steam://` alternatives, fragility. |
| R2 — fit for DDL | `docs/research/steam-shortcuts.md` §4: what a shortcut would launch, a safe write design, multi-user/SD card/Flatpak, risks, existing DDL pieces. |
| R3 — recommendation | This report. |
| A1 | Every claim in the research doc carries **Confirmed**, **Proposed** or **Needs Deck Validation**; sources are linked, and code sources are pinned to commits/tags. |
| A2 | This report contains R3. |

No new dependency, no code change, no `cargo` run (research only). The two files above are the
only changes.

## R3 — Recommendation

### Chosen option: README-only

DDL should keep **one** Steam library entry — the DDL launcher itself — and let the player pick
the mapset inside DDL. A user who wants a mapset to appear as its own Steam library entry can do
it by hand; DDL should document that path and nothing more.

**Automatic in-app shortcut writing is a no-go now**, for these reasons (all evidenced in the
research doc):

1. **The primary flow forbids it.** In Gaming Mode DDL is launched from Steam, so Steam is
   running. Every maintained tool requires Steam to be closed before touching `shortcuts.vdf`,
   because Steam can overwrite the file and will not show the new entry until it restarts. DDL
   cannot satisfy that precondition from inside its own session.
2. **Closing Steam is not an acceptable side effect.** It would terminate DDL's parent and may
   end the Gaming Mode session. Even if that were safe, prompting a player to quit Steam from a
   mapset launcher is a bad experience.
3. **The format is unsupported and moving.** `shortcuts.vdf` is community-reverse-engineered, has
   inconsistent field casing, and the client has already changed ID behaviour (randomised
   NSG ids, 2023) and collection storage (late 2025). A DDL writer would own a compatibility
   burden with no upstream contract.
4. **Artwork is missing.** DDL has 1280×720 screenshots, not 600×900 capsules, 3840×1240 heroes,
   or transparent 1280×720 logos. Per-mapset artwork is a licensing and content project on top of
   any file writer.
5. **The error path is invisible.** A per-mapset shortcut would run `ddl launch <id>` headless.
   If the mapset is not installed or the profile was deleted, the typed error goes to stderr,
   which Gaming Mode does not show.
6. **It crosses a stated boundary.** `OVERVIEW.md` lists "modifying Steam settings/shortcuts
   files" as an MVP non-goal. Automatic writing needs an owner decision, not just a worker task.

A blanket no-go is not necessary: the manual path costs no code, respects every boundary, and
gives advanced users a documented escape hatch.

### What "README-only" means concretely

Add a short, clearly optional appendix to `docs/steam-deck-guide.md` (and a pointer from
`README.md`) — proposed text, not written here because those files are outside this task:

> #### Optional: give one mapset its own Steam entry (advanced)
>
> You do not need this. DDL launches every mapset from its own screen. If you want a mapset to
> appear as its own library entry:
>
> 1. In Desktop Mode, add DDL to Steam again: **Games → Add a Non-Steam Game to My Library… →
>    Browse… →** select the `ddl` executable.
> 2. Rename the new shortcut to the mapset (right-click → **Properties**).
> 3. In **Properties → Launch Options**, enter `launch <mapset-id>` (for example
>    `launch sigil`). To pin one profile, append `--profile UV`; leave it out to use the profile
>    you played last.
> 4. Install the mapset and create its profile in DDL first. The shortcut runs DDL headless, so
>    if the mapset is missing you will see the shortcut stop with no explanation.
> 5. Set artwork by hand, as in [Make it look native in Steam](#make-it-look-native-in-steam).
>
> DDL does not create or edit Steam shortcuts. These steps are unverified on a Deck; Steam UI
> wording can change.

**Do not publish these steps until they are Deck-validated** (see open question 1 below). Until
then this is a report recommendation, not user-facing documentation.

## Why not "go" with a code feature

If the owner nevertheless wants DDL to write shortcuts, the smallest honest version is much
larger than a file append:

- a lossless binary-VDF parser/writer, with golden fixtures and round-trip preservation;
- a Steam-running check that refuses to write and explains itself;
- a timestamped backup plus restore;
- explicit multi-user selection from `loginusers.vdf` and `userdata/<id>`;
- per-mapset artwork assets (capsule, wide capsule, hero, transparent logo, icon);
- a UI flow that is opt-in, reversible, and never touches non-DDL entries;
- Deck validation of the whole cycle: quit Steam, write, restart, see the shortcut and artwork.

That is a milestone-sized feature with a documented compatibility risk. The research doc §4.2
records the properties it would need.

## Conditional spec (only if the owner overrides the recommendation)

Not proposed now; recorded so the cost is visible.

**Requirements:** R1 — an opt-in "Prepare Steam shortcuts" action, never a side effect of
install/launch. R2 — refuse while Steam runs, with a clear explanation and a Desktop Mode path.
R3 — back up `shortcuts.vdf` before any change and offer restore. R4 — preserve every non-DDL
entry and unknown field byte-for-byte; refuse unknown shapes. R5 — one shortcut per selected
mapset, with a DDL-owned reserved-range `appid` and `LaunchOptions=launch <id>`; removing DDL
entries must never touch others. R6 — multi-user selection. R7 — copy DDL-owned artwork into
`config/grid/` for each mapset and never delete foreign files. R8 — atomic temp+rename writes.

**Acceptance:** A1 — parser/writer fixtures, round-trip, corrupt input, unknown fields. A2 —
refusal while Steam runs, backup/restore, no-op when Steam is absent. A3 — multi-user and
owned-entry removal tests. A4 — Deck validation: shortcut and artwork appear after a Steam
restart; DDL can revert cleanly.

## Spec discrepancy found

Spec 043 R2 says `ddl play <id> --profile …`. The implemented CLI is
`ddl launch <id> [--dry-run] [--profile <name|id>] …` (spec 014, spec 018, `crates/ddl/src/cli.rs`).
There is no `play` subcommand. The research doc uses `launch` and records the discrepancy. The
spec text should be corrected by its owner; this task does not edit spec files.

## Proposed changes to owner-owned documents (not made)

- `docs/steam-deck-guide.md` / `README.md`: optionally add the advanced appendix above, **after**
  Deck validation. No change needed now.
- `docs/decisions.md`: no change for the README-only outcome — it stays inside "documentation
  only" and does not modify Steam files. If the owner later chooses a code feature, a new
  decision is required and `OVERVIEW.md` §"MVP non-goals" ("modifying Steam settings/shortcuts
  files") must be amended.
- `docs/status.md` / `docs/journal.yml`: coordinator-owned; nothing to add beyond "043 research
  complete, README-only".

## Open questions

The research doc §5 lists eight Deck-validation questions. The ones that gate the README
recommendation are:

1. Does a Steam shortcut with `Exe=<ddl>` and `LaunchOptions=launch <id>` start correctly in
   Gaming Mode, including an `Exe` path with spaces?
2. Does the current client accept a DDL-written `appid` and display `grid/<id>p.jpg` etc.?
3. What exactly happens if Steam is asked to close while DDL runs in Gaming Mode?
4. Does `loginusers.vdf` `MostRecent` match the account actually running DDL on a shared Deck?

## Verification

- Read the pinned sources in §2 of the research doc: Valve Support, Steamworks asset pages,
  Valve Developer Community pages, SteamTinkerLaunch (wiki revision + source commit), BoilR
  tag `v.1.11.0`, Steam ROM Manager v2.5.44, `steam_shortcuts_util` 1.1.9, and
  ValveSoftware/steam-for-linux issue #9463.
- Read DDL's actual CLI, Steam discovery, DSDA runner and atomic-write code to ground R2.
- Confirmed every code claim against the cited file and line; no claim relies on memory.
- Direct fetches of Valve Developer Community pages hit an Anubis bot challenge, so their content
  was read through the search index on 2026-10-19 and the canonical URLs are cited. The
  Steamworks pages and GitHub APIs were fetched directly.
- No build or test run is needed for a research-only change; `git diff --check` is used before
  committing.
- No visible window was opened and no screen was captured.

## Files changed

- `docs/research/steam-shortcuts.md` (new)
- `docs/reports/043-steam-shortcuts-research.md` (new)

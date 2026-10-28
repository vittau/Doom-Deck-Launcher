# DSDA-Doom map progress evidence (Q1–Q5)

Research for spec 115, 2026-10-28. **Confirmed** means the cited source was
inspected, not that the AppImage was executed. **Proposed** marks DDL design or
interpretation; **Needs Deck Validation** marks runtime checks. All DSDA links
below pin `v0.30.1`, cloned at commit
`8ed581ad5c868b4dba1c39eb21d29e4ddbed9fad` into `/tmp/ddl-dsda-v0301`.
No DSDA execution, visible window, screenshot, dependency or product-code change
was involved. Existing context: [engine fact sheet](dsda-engine.md), especially
its data-organizer section. Q6 is in the [proposal report](../reports/115-dsda-stats-research.md).

## Q1 — Recorded data and lifecycle

**Confirmed — primary artifact.** The filename is `stats.txt`, version is integer
`1`, and the path is `dsda_DataDir()/stats.txt`. It is plain text, not JSON or a
savegame. The writer emits a version line, a WAD-wide cumulative kill-count line,
then one space-separated row per map (15 tokens: lump plus 14 signed decimal
integers); there is no serialized map count, timestamp, content hash, session ID
or display title. [Path/constants](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L32-L60),
[writer and field order](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L214-L250),
[types](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.h#L21-L44).

**Confirmed — every map-row field, in exact write order.** Initial values come
from zero initialization plus explicit `-1` assignments. The update rules below
are shared evidence for every row in this table:
[initialization](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L115-L145),
[exit updates](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L262-L323),
[kill updates](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L325-L331).

| Field | Meaning and initial value |
| --- | --- |
| `lump` | Map marker name, at most eight characters in the type; no friendly title. |
| `episode` | Parsed episode; `MAPn` uses 1; unrecognized names use -1. |
| `map` | Parsed map number, or -1. |
| `best_skill` | Highest one-based skill on a non-`nomonsters`, non-playback exit; initially 0. |
| `best_time` | Minimum eligible `leveltime`, initially -1; only when `levels_completed == 1`. |
| `best_max_time` | Minimum eligible first-completed-map time with no missed monsters and all secrets, without respawning; initially -1. Items are not required. |
| `best_nm_time` | Minimum eligible first-completed-map time on the highest skill (Doom Nightmare), initially -1. |
| `total_exits` | Number of completion-hook exits, initially 0; increments even with `nomonsters`. |
| `total_kills` | Cumulative kill-hook calls for this map, initially 0; not just kills in successful exits. |
| `best_kills` | Largest eligible `totalkills - missed_monsters`, initially 0; not updated with respawning. |
| `best_items` | Largest eligible console-player item count, initially 0. |
| `best_secrets` | Largest eligible console-player secret count, initially 0. |
| `max_kills` | `totalkills` from the most recent eligible exit, initially -1; a denominator, not a historical maximum. |
| `max_items` | `totalitems` from that exit, initially -1. |
| `max_secrets` | `totalsecret` from that exit, initially -1. |

**Confirmed — eligibility nuance.** All exit updates first reject missing map
records and demo playback. With monsters enabled, raising the skill resets
`best_time`, `best_max_time` and best K/I/S only if the old best skill was below
`num_skills - 1`; it does not reset `best_nm_time`. Subsequent result updates
require skill at least the stored best, **or** skill equal to `num_skills - 1`
(UV for Doom). Thus UV results still update after a Nightmare exit. K/I/S bests
may come from different exits; current denominators can differ from those of the
best counts. Time values are game tics (35 per second), not wall time.
[Eligibility](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L266-L323),
[tic rate](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomdef.h#L148-L150),
[Doom skills](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/skill_info.c#L28-L68),
[skill count](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/skill_info.c#L127-L130).

**Confirmed — lifecycle.** The following table describes code paths, not a
promise that a crash can safely persist data. Init loads once after patch loading;
a new inventory is built only if no map rows were loaded.
[Startup](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2170-L2195),
[init/load](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L151-L200),
[init fallback](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L333-L338).

| Event | Memory and disk behavior, with source |
| --- | --- |
| Level setup | Selects current stats by episode/map; no disk write. [Setup hook](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda.c#L578-L585), [lookup](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L252-L264). |
| Level exit | Increments `levels_completed` before the completion hook; updates stats in memory, without saving `stats.txt`. [Completion path](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L2155-L2177), [hook](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda.c#L591-L643). |
| Kill / death | Kill hook increments cumulative map/WAD kills; player rebirth resets `levels_completed`, not WAD stats. No death count or death-triggered stats write. [Kill hook](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda.c#L319-L334), [rebirth](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L1895-L1912). |
| Menu New Game | New-game initialization resets run time and completed-level count; it does not reload/clear/save WAD stats. [New game](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L3071-L3097). |
| Load game | Unarchives context via level initialization, then restores `leveltime`, `totalleveltimes` and `levels_completed`; WAD stats are not part of that archived context and are not rolled back or reloaded. [Saved context](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/save.c#L70-L145), [archive contents](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/save.c#L214-L258). |
| Normal quit | Essential quit writes analysis, splits and WAD stats. Writer truncates the destination (`wb`) then closes it; no temp-file rename or lock. Empty map inventory or forget flag suppresses saving. [Quit](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L201-L217), [writer](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L202-L250). |
| Error/crash | Essential quit is registered with `run_on_error=true`, so `I_SafeExit` attempts it even on error; abrupt termination bypassing that path cannot be assumed to save. [Registration](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L313-L322), [safe exit](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L177-L199). |

**Proposed — interpretation.** `total_exits > 0` honestly means “exited at least
once,” including no-monster exits. `best_skill > 0` is a narrower non-no-monster
exit indicator, still not a cheat-free or continuous-campaign certificate.
Kills can include failed/replayed attempts and save reloads. There is no reliable
last-played map, next map, deaths, attempts, real playtime or run history here.

**Confirmed — split tracker is a separate demo artifact.** Tracking is enabled
only for demo recording, or playback with `-track_playback`; ordinary menu play
is not tracked. The organized directory is the same as stats. Filename pattern
is `<demo-base>_<skill>_<episode>_<start-map>_<modifiers>_splits.txt`; basename
comes from playback-tracking argument or demo base. Modifiers are three positions:
respawn `r`/`x`, fast `f`/`x`, no monsters `o`/`x`. Complevel is absent.
[Gate](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L238-L240),
[name/path](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/split_tracker.c#L34-L93).

**Confirmed — split format/lifecycle.** Header is attempts then version (`1`).
Each row has episode, map, best level tics, best cumulative tics, exits, reference
level tics, reference cumulative tics (seven integers). Current/session-best and
delta fields are not serialized. It lazily loads once on reset or split record;
legacy version 0 converts zero times to unset (-1). Reset increments attempts;
completion records a split, updates minima and possibly reference splits. Essential
quit writes it when attempts is nonzero. It is not a full map inventory or a
persistent normal-game session log. [Loader](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/split_tracker.c#L95-L182),
[writer](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/split_tracker.c#L184-L214),
[reference/record/reset](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/split_tracker.c#L216-L293),
[quit](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L201-L210).

**Confirmed — split persistence events.** Attempt reset is called when the demo
recording header is written and after explicitly tracked playback starts. In
ordinary non-demo play, New Game, death and menu load do not activate tracking.
Map completion changes split memory only; split persistence is the essential-quit
call, including the controlled-error exit path described above, with no guarantee
on abrupt crash/kill. There is no split-file reload per save load or level entry:
the loader uses its process-static loaded guard.
[Recording reset](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L3536-L3542),
[playback reset](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L3990-L3997),
[gate](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L238-L240),
[load guard](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/split_tracker.c#L105-L118),
[quit](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L201-L210).

**Confirmed — optional `-levelstat`.** The flag enables a separate per-process
completion list; every completion rewrites `levelstat.txt` in the working directory.
Rows contain map lump (suffix `s` for secret exit), elapsed map minutes/seconds
with two decimal places, cumulative minutes/whole seconds, and K/I/S achieved/total
with optional per-player breakdowns. It uses kill count minus maxkilldiscount,
which differs from WAD best-kills semantics. No version header, persistent readback,
full inventory, skill or content identity is provided by this writer.
[Enable](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/e6y.c#L199-L209),
[collect and write on exit](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/e6y.c#L450-L505),
[path and formatting](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/e6y.c#L514-L610).

**Confirmed — optional `-analysis`.** At essential quit, the enabled writer
replaces working-directory `analysis.txt` with key/value lines. Its full key set
is skill, nomonsters, respawn, fast, pacifist, stroller, reality, almost_reality,
reborn, 100k, 100s, missed_monsters, missed_secrets, weapon_collector, tyson_weapons,
turbo, solo_net, coop_spawns, category and signature. It has no version marker,
per-map rows, times or persistent readback. Analysis reset clears run predicates;
it is not a WAD-progress database. [Flag](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L336-L345),
[reset/writer](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/analysis.c#L57-L118),
[quit call](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L201-L210).

**Proposed.** Do not add recording, `-levelstat` or `-analysis` for the first
feature. `stats.txt` needs no new launch flag; the other artifacts solve different
problems and would introduce new persistence/attribution requirements.

## Q2 — DDL paths and identity

**Confirmed — organizer algorithm.** `-data` must name an existing directory or
DSDA logs an error and falls back to `DOOMDATADIR`/config directory. DSDA appends
`dsda_doom_data`, then lowercase IWAD stem and up to the first eight PWAD stems,
in load order. Only filenames ending case-insensitively in `.wad` contribute;
directory and extension are removed. Source classification matters, so engine
resource WADs do not contribute. This is neither a hash nor a catalog ID.
[Root/fallback](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/data_organizer.c#L36-L81),
[ordered stems](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/data_organizer.c#L83-L139).

**Confirmed — DDL layout at research baseline.** `D` below is the resolved DDL
XDG data directory, not the DSDA default directory. DDL builds `ProfileDirs` from
the actual profile ID, supplies its `data` directory as `-data`, separate `save`
as `-save`, a config **file** as `-config`, and runs in `run`. Catalog files load
in recipe order followed by profile extras, with patches in a separate list.
Custom setups use their ordered file list; base-game launches route through that
same custom builder and add profile extras. These claims cite the immutable DDL
baseline; task 114 removes skill/warp separately under D-044, not the path layout.
[Profile dirs](https://github.com/vittau/Doom-Deck-Launcher/blob/88bfcb77245c61a89874eadb06bb275c3cbb1308/crates/ddl-core/src/profiles/store.rs#L24-L57),
[catalog resolver](https://github.com/vittau/Doom-Deck-Launcher/blob/88bfcb77245c61a89874eadb06bb275c3cbb1308/crates/ddl-core/src/resolver/resolve.rs#L177-L260),
[custom builder](https://github.com/vittau/Doom-Deck-Launcher/blob/88bfcb77245c61a89874eadb06bb275c3cbb1308/crates/ddl-core/src/local/launch.rs#L137-L172),
[base-game builder](https://github.com/vittau/Doom-Deck-Launcher/blob/88bfcb77245c61a89874eadb06bb275c3cbb1308/crates/ddl-core/src/iwads/base_games.rs#L91-L135),
[argv/cwd](https://github.com/vittau/Doom-Deck-Launcher/blob/88bfcb77245c61a89874eadb06bb275c3cbb1308/crates/ddl-core/src/engines/dsda/plan.rs#L798-L847).

**Proposed — exact derived examples.** These follow the two cited algorithms;
`P`, filenames and stems are illustrative, not hardcoded directories to add:

| Launch | Derived stats path |
| --- | --- |
| Catalog profile `P`, `doom2.wad` + `Example.wad` | `D/profiles/P/data/dsda_doom_data/doom2/example/stats.txt` |
| Base game, no extras, `doom2.wad` | `D/profiles/P/data/dsda_doom_data/doom2/stats.txt` |
| Custom setup, `doom.wad` + `A.wad` + `B.wad` | `D/profiles/P/data/dsda_doom_data/doom/a/b/stats.txt` |
| Catalog plus extra `Weapons.wad` | `D/profiles/P/data/dsda_doom_data/doom2/example/weapons/stats.txt` |
| Same load with extra `.deh`/`.bex` | Same organized path as before adding that patch. |

**Proposed — isolation/collision implications.** Different profile IDs isolate
stats even when stems match. Within one profile/root, identical ordered lowercase
stems collide regardless of source directories, bytes, complevel, patches or
changes after PWAD eight. Renaming, changing order or adding a qualifying WAD
can select a different file and appear to reset progress. Removing it can reveal
older stats. Replacing bytes without renaming can preserve misleading progress
and stale map inventory. Canonical IWAD basename, not physical user-file basename,
controls identified-IWAD paths. Derive paths from the resolved launch, never from
recipe title. ZIP expansion and unusual `.lmp` loads need engine-load-order
verification before claiming support.

**Confirmed — unrelated paths.** Explicit `-save` bypasses organized-save
redirection; it does not redirect stats. `-config` selects a config file, whereas
stats uses `dsda_DataDir`. Relative `levelstat.txt`/`analysis.txt` therefore land
in `D/profiles/P/run` with DDL's cwd, not beside `stats.txt`.
[Save selection](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/save.c#L258-L275),
[config selection](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L705-L715),
[stats path](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L53-L60),
[DDL cwd](https://github.com/vittau/Doom-Deck-Launcher/blob/88bfcb77245c61a89874eadb06bb275c3cbb1308/crates/ddl-core/src/engines/dsda/plan.rs#L837-L847).

## Q3 — Settings and what they invalidate

**Confirmed — source-based interaction matrix.** All rows also inherit the
[stats gates/updates](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L266-L331)
and [path identity](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/data_organizer.c#L83-L130).

| Setting/action | Effect |
| --- | --- |
| Menu-only launch (D-044) | Stats init does not depend on `-skill`/`-warp`; it occurs at startup. Starting New Game in the engine sets real skill/map and resets `levels_completed`. No DDL-selected skill should be inferred. [Startup](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2194-L2195), [new-game state](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L3071-L3097). |
| Continuous play | First exit has `levels_completed == 1`; later exits increment it, so they can update exits/skill/K/I/S without updating times. [Completion](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L2155-L2177). |
| Pistol start | When enabled and incompatibility is allowed, per-level load calls rebirth, which resets completed count. Subsequent exits can qualify for times. No pistol-start field distinguishes those results. [Load](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L1221-L1243), [rebirth reset](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L1904-L1912). |
| Complevel | No compatibility field/path segment or compatibility gate in WAD stats. Different complevels share the same profile/stem stats. |
| Add-ons/extras | WADs can change stem path and map inventory; DEH/BEX and ninth/later WADs can change behavior without changing path. Counts/times do not identify which combination produced them. |
| Nightmare | Highest skill becomes best skill; separate NM time is maintained only for first-completed-map exits. Respawn prevents best-kills/best-max-time updates; items/secrets and denominators still update. UV can subsequently update normal results without lowering best skill. [NM definition](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/skill_info.c#L59-L68). |
| Menu load | Restores saved skill, time and completed count; stats keep accumulated memory, so reloading/re-exiting can count again. First-completed-map time eligibility follows restored count. Save modifiers can also restore. [Context](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/save.c#L105-L145), [modifiers](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/save.c#L147-L198). |
| `nomonsters` | Suppresses best skill/time/K/I/S/denominator updates, but still increments total exits. No stored flag identifies those exits afterward. |
| Fast/respawn/cheats/co-op | No general validity gate for these in WAD stats; respawn specifically suppresses best kills/max time. No stored modifier, cheat or multiplayer provenance. `best_items`/`best_secrets` use console player; total kills use the general kill hook. [Hook](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda.c#L327-L334). |
| Demo playback | WAD exit/kill updates skipped; split tracking may separately run when explicitly requested. [Split gate](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L238-L240). |

**Proposed.** Display “DSDA recorded exits/results” rather than “verified UV
completion” or an achievement. Do not sum best times into campaign time, assume
missing time means unfinished, or present independent best K/I/S as one perfect run.
**Needs Deck Validation.** Verify menu-only start/quit, natural progression,
pistol start and loaded-save behavior against actual files; source paths alone
are insufficient acceptance evidence for the shipped AppImage.

## Q4 — Map denominator and names

**Confirmed — stats already contain unplayed maps.** Fresh creation reverse-scans
loaded lump information for `THINGS` or `TEXTMAP`, takes the preceding lump name,
deduplicates case-insensitively, and stops at IWAD lumps once a PWAD map was found.
Thus a map-bearing PWAD inventory excludes leftover IWAD maps; a resource-only
PWAD can leave an IWAD inventory. It sorts parsed episode/map names and assigns
unknown identifiers -1/-1. This is a structural heuristic, not a reachable-map
walk or campaign definition. Existing nonempty stats are loaded without rebuilding
or reconciling with current WAD content.
[Scan/sort](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L62-L149),
[load/init](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L151-L200),
[fallback only](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L333-L338).

**Proposed — candidate sources and limits.**

| Source | Useful for | Limit / recommendation |
| --- | --- | --- |
| Stats rows | DSDA's own initial detected-map denominator and IDs, even before a map was played | Exists only after a successful save; stale across same-stem content changes; may include secret, unused, test or inaccessible maps. Label “detected maps”; never claim all reachable campaign maps. |
| WAD directory scan | Map markers before first launch; structural cross-check | A regex of `MAPxx`/`ExMy` alone can mistake arbitrary lumps for maps and miss nonstandard identifiers. Validate marker plus map-lump structure, load order and overrides, with bounds. Still cannot infer reachability or authors' intended completion route. Defer in MVP. |
| UMAPINFO | Optional map labels/titles and progression metadata | Metadata can be partial, so entry count is not a map denominator. Need full syntax, defaults, merging and loaded-lump order; do not implement a regex title scraper. |
| MAPINFO | Potential titles in a supported dialect | A lump named MAPINFO is not a universal format contract; DSDA's inspected Hexen parser is gated to Hexen. Do not treat arbitrary ZDoom MAPINFO as authoritative for DDL's Doom mapsets. |
| DEHACKED/BEX strings | Replacement legacy HUD titles | Requires original-string/key substitution and patch/lump precedence, not merely scanning marker names. A leftover IWAD title is not evidence of a PWAD's intended name. |
| Recipe metadata | Curated total/map order/names, including intended secret-map policy | Current recipe schema lacks a map inventory/title table. Adding it would require a separately approved schema/catalog task and maintenance per version/component combination. |

**Confirmed — evidence underlying metadata limitations.** UMAPINFO entries
validate map names and process properties; `levelname`/`label` feed the HUD title;
the engine loads all UMAPINFO lumps in sequence. The Hexen MAPINFO parser returns
immediately outside Hexen. Legacy HUD titles come from game-specific map-name
string tables, and DEH/BEX substitutes lookup strings. DDL's current `Recipe`
has no map inventory field.
[UMAPINFO properties](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/umapinfo.cpp#L175-L217),
[entry parsing](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/umapinfo.cpp#L494-L532),
[HUD title](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/mapinfo/u.c#L433-L446),
[lump loading](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/mapinfo/u.c#L537-L549),
[Hexen gate](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/mapinfo/hexen.c#L391-L427),
[legacy title](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/mapinfo/legacy.c#L451-L484),
[string substitution](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_deh.c#L3020-L3050),
[recipe schema](https://github.com/vittau/Doom-Deck-Launcher/blob/88bfcb77245c61a89874eadb06bb275c3cbb1308/crates/ddl-core/src/catalog/model.rs#L3-L35).

**Proposed.** Start with lump IDs (`MAP01`, `E1M1`) and DSDA row count, with a
visible detected-map qualifier and unknown state before the file exists. Reject
ambiguous duplicate identifiers/episode-map pairs rather than inventing mappings.
Unknown -1/-1 rows cannot reliably be matched by the engine's numeric stats lookup.
Do not hardcode 32 maps for every Doom II mapset or 36 for every Doom IWAD.

## Q5 — Safe read and release compatibility

**Confirmed — upstream is permissive, not a parser model for DDL.** It splits
lines, accepts header version less than or equal to 1 (including older/negative
values), deletes the file on invalid/future headers, accepts rows where 15 scans
succeed and skips malformed rows. No content identity check, file-size bound,
strict token-count check or reconciliation exists in this loader. The writer
truncates in place, with no atomic rename/locking protocol.
[Load](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L151-L200),
[write](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L214-L250).

**Proposed — DDL read contract (not upstream guarantees).**

- Read only the path derived from a resolved launch/profile under DDL's known
  data root. Open a regular file without following untrusted symlink escapes;
  anchor directory traversal and file open to avoid check/open races. Never use
  a map-name token as a filesystem path or recursively discover “matching” stats.
- Bound the actual read to 1 MiB plus one overflow-detection byte; at most 4,096
  map rows, 512 bytes per line, eight ASCII bytes per map ID. These are proposed
  product limits, not DSDA limits; oversized output becomes unavailable.
- Support exactly version 1, exactly two headers and exactly 15 tokens per row.
  Parse checked signed integers; allow -1 only for documented unknown fields,
  otherwise reject negative counters/times; skill 0–5 for this Doom-only baseline.
  Reject duplicate IDs, ambiguous numeric keys, malformed/non-ASCII IDs, trailing
  tokens, overflow and missing final row termination. Accept LF/CRLF.
- Preserve legitimate zero times/counts; render -1 as unknown, never zero.
  Do not enforce best-count <= current denominator: independently accumulated
  bests and later denominator changes can violate that inequality legitimately.
- Refresh after DSDA process termination/reaping, or while no session is active.
  During play show the last successful snapshot, visibly stale if needed. A
  size/mtime check before and after read can reject observed concurrent changes;
  it does not prove consistency. Avoid reading during any known engine writer.
- Reject a malformed/truncated snapshot as a whole rather than inflating progress
  from a subset of rows. Missing/corrupt/unsupported/permission-denied means
  unavailable, not “0 completed”; keep the previous known snapshot with its
  identity and stale status. A syntactically valid prefix cut on a row boundary
  cannot be detected from this format alone; a denominator cross-check would
  require independently knowing the map inventory.
- Never repair/delete/rewrite engine files. Keep optional DDL cache as separately
  versioned, atomically written JSON with engine version, profile and resolved
  file/patch identity; fail closed on changed content. A new cache key alone
  cannot prove old engine stats belong to newly replaced same-stem content.
- Support the audited engine release explicitly. An unchanged file marker in a
  newer release is not evidence of unchanged semantics; re-audit before enabling
  import. Multiple concurrent DSDA processes sharing a root can overwrite each
  other's in-memory snapshots; no trustworthy cross-process merge is possible.

**Confirmed — XDG/Flatpak path evidence.** DDL resolves data from absolute
`XDG_DATA_HOME` plus `ddl`, falling back to `$HOME/.local/share/ddl`; profile paths
are built underneath that resolved root. Its existing Flatpak test models the
sandbox data home as `$HOME/.var/app/<app-id>/data` (the test still uses the old
`com.doomdecklauncher.DoomDeckLauncher` identity).
Do not scan the host's DSDA config directory to guess progress.
[XDG resolver](https://github.com/vittau/Doom-Deck-Launcher/blob/88bfcb77245c61a89874eadb06bb275c3cbb1308/crates/ddl-core/src/platform/paths.rs#L95-L125),
[Flatpak test](https://github.com/vittau/Doom-Deck-Launcher/blob/88bfcb77245c61a89874eadb06bb275c3cbb1308/crates/ddl-core/src/platform/paths.rs#L233-L263).

**Needs Deck Validation.** Verify the actual Flatpak/extracted AppRun receives
and can write DDL's resolved `-data` path; verify normal exit, controlled error,
force-stop, read-only directories, corrupt/truncated files, simultaneous processes,
profile changes and renamed/replaced WADs. The macOS source audit cannot certify
sandbox permissions, flush durability or Steam Deck shutdown behavior.

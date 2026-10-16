# DSDA-Doom v0.30.1 engine fact sheet

Desk research, 2026-10-16. Scope: upstream source and packaging; no engine build,
IWAD execution, Linux runtime test or physical Deck test was performed.

**Confirmed** means read directly in the tagged source. **Proposed** means a DDL
design implication or an inference. **Needs Deck Validation** marks runtime or
hardware behavior that source inspection cannot establish. Each source link is
pinned to `v0.30.1`; release metadata and separately identified artifact observations
are dated evidence, not source guarantees.

The requested shallow clone is in git-ignored `.scratch/dsda-doom`. The tag resolves
to `8ed581ad5c868b4dba1c39eb21d29e4ddbed9fad` (`git rev-parse HEAD` and
`git ls-remote --tags --refs` agreed). No newer `v0.*` tag was present on the research
date; [the release list](https://github.com/dsda-org/dsda-doom/releases) also identified
[v0.30.1](https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.1) as latest.
These are dated repository observations, not promises about future releases.

## 1. CLI and launch semantics

**Confirmed — general parser rules.** Options and aliases are case-insensitive.
The parser scans **right to left**: the last occurrence wins; an earlier duplicate
is warned about and ignored. This applies to arrays too: `-file a.wad -file b.wad`
loads only `b.wad`, and `-deh a.deh -bex b.bex` keeps only the latter list.
Use one flag with several separate argv values. Values stop at the next token
starting with `-` unless its prefix parses as an integer. There is no `--`
end-of-options escape in the option table/parser. String options take exactly one
value, and array arity is checked. Integer CLI parsing uses `%d`, so a numeric
prefix can be accepted with trailing junk (unlike the complevel parser).
Unknown options and invalid counts/ranges call `I_Error`.
[prboom2/src/dsda/args.c:55–79](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L55-L79); [prboom2/src/dsda/args.c:700–860](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L700-L860).

All following rows are **Confirmed**; limits describe parser acceptance, not a
promise that every accepted input is meaningful for a mapset.

| Option | Accepted form and behavior | Tagged source |
| --- | --- | --- |
| `-iwad path` | One string; explicit IWAD selection via normal WAD search (adds `.wad` if the named file is absent). Supply an absolute path with extension to avoid search ambiguity. | [prboom2/src/dsda/args.c:61–65](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L61-L65); [prboom2/src/d_main.c:1187–1225](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1187-L1225) |
| `-file a.wad b.wad …` | At least one string, processed in list order. Supports WAD/LMP, ZIP and DEH/BEX routing. Extensionless names are searched as `.wad`, `.lmp`, `.zip`, `.deh`, `.bex` in that order. DEH/BEX entries are deferred to patch processing, so interleaving them with WADs does not express arbitrary patch/lump order. | [prboom2/src/dsda/args.c:66–75](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L66-L75); [prboom2/src/d_main.c:1770–1823](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1770-L1823) |
| `-deh a.deh b.bex …`, alias `-bex` | Same array option; at least one string. Patches processed in list order **after** WAD-embedded DEHACKED and autoload patches, regardless of where flags appear in argv. Extensionless DEH search prefers BEX. | [prboom2/src/d_main.c:2108–2192](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2108-L2192); [prboom2/src/SDL/i_system.c:655–680](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_system.c#L655-L680) |
| `-complevel value`, alias `-cl` | One string; full decimal conversion with surrounding whitespace accepted, or case-insensitive aliases below. | [prboom2/src/dsda/settings.c:60–170](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L60-L170) |
| `-skill n` | Integer 1–255; engine start skill is `n-1`. Default config is skill 4; current built-in skill inventory has five skills; actual skill setup clamps to the highest built-in skill. `-uv` forces 4; `-nm` forces 5 and wins if both appear, independent of argv order. | [prboom2/src/dsda/args.c:121–135](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L121-L135); [prboom2/src/dsda/args.c:856–860](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L856-L860); [prboom2/src/d_main.c:1988–2005](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1988-L2005); [prboom2/src/dsda/configuration.c:241–244](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L241-L244); [prboom2/src/dsda/skill_info.c:129–130](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/skill_info.c#L129-L130); [prboom2/src/dsda/skill_info.c:231–241](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/skill_info.c#L231-L241) |
| `-warp [n [m]]` | Zero to two integers, each 0–99; autostarts. In Doom II, first is map (second ignored); in Doom, first is episode, second map (map defaults to 1). Empty `-warp` resolves first map. Hexen translates the first number via map metadata and errors for invalid translation. | [prboom2/src/dsda/args.c:116–120](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L116-L120); [prboom2/src/d_main.c:1616–1633](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1616-L1633); [prboom2/src/dsda/mapinfo/legacy.c:108–126](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/mapinfo/legacy.c#L108-L126); [prboom2/src/dsda/mapinfo/hexen.c:159–174](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/mapinfo/hexen.c#L159-L174) |
| `-config path` | One string: config **file**, not directory or isolation root. Read and later write that exact filename. | [prboom2/src/dsda/args.c:391–395](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L391-L395); [prboom2/src/m_misc.c:705–715](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L705-L715) |
| `-data directory` | One string; overrides `DOOMDATADIR` only if existing directory, otherwise logs error and falls back. Adds its own `dsda_doom_data` subtree (see §6). | [prboom2/src/dsda/data_organizer.c:42–81](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/data_organizer.c#L42-L81) |
| `-save directory` | One string; existing-directory requirement and fallback as for `-data`, with `DOOMSAVEDIR`. Explicit flag bypasses organized-save redirection. | [prboom2/src/dsda/save.c:258–275](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/save.c#L258-L275); [prboom2/src/dsda/data_organizer.c:42–67](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/data_organizer.c#L42-L67) |
| `-shotdir directory` | One string; writable-directory check and fallback (see §6). | [prboom2/src/dsda/args.c:401–405](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L401-L405); [prboom2/src/m_misc.c:902–914](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L902-L914) |
| `-noautoload`, alias `-noload` | Zero values; skips directory autoload WAD/ZIP/patch phases, **not** embedded DEHACKED in explicitly loaded WADs. `-nodeh` separately skips embedded DEHACKED. | [prboom2/src/dsda/args.c:491–505](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L491-L505); [prboom2/src/d_main.c:2059–2070](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2059-L2070); [prboom2/src/d_main.c:2108–2170](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2108-L2170) |
| `-record name` | One string; records demo, with naming described in §6; explicit skill and compatibility required by demo initialization. | [prboom2/src/dsda/args.c:101–105](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L101-L105); [prboom2/src/dsda/demo.c:271–300](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/demo.c#L271-L300) |
| `-playdemo path`, `-playlump name` | One string each; external demo playback versus internal demo lump. | [prboom2/src/dsda/args.c:81–90](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L81-L90) |
| `-timedemo path`, `-fastdemo path` | One string; timed maximum-speed playback versus fast playback with frame skipping. | [prboom2/src/dsda/args.c:91–100](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L91-L100) |
| `-recordfromto input output` | Exactly two strings; playback of first while writing second. | [prboom2/src/dsda/args.c:106–110](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L106-L110) |
| `-tas`, `-build`, `-dsdademo` | Demo-init paths lift strict restrictions; there is no `-strict` option in this version's complete option table. | [prboom2/src/dsda/args.c:55–694](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L55-L694); [prboom2/src/dsda/demo.c:285–298](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/demo.c#L285-L298) |
| `-v` | Prints version and returns success before config load, subsystem/window initialization or IWAD lookup. | [prboom2/src/SDL/i_main.c:266–296](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L266-L296) |
| `-help`, `--help` | Prints generated option descriptions and exits success, but only after main has loaded config, required port WAD, and preinitialized graphics. It is not the minimal smoke probe. | [prboom2/src/dsda/args.c:56–60](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L56-L60); [prboom2/src/SDL/i_main.c:294–342](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L294-L342); [prboom2/src/d_main.c:1954–1960](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1954-L1960) |

### Compatibility values and omission

**Confirmed.** Numerically accepted values are every integer **-1 through 21**.
`-1` selects `best_compatibility`, currently 21. Numeric acceptance includes
placeholder values 18–20; do not mistake parser acceptance for a supported,
named compatibility family. The enum assigns these meanings:
[prboom2/src/dsda/settings.c:115–155](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L115-L155); [prboom2/src/doomtype.h:113–140](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomtype.h#L113-L140);
[prboom2/src/g_game.c:2753–2772](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L2753-L2772).

| Numbers | Meaning in tagged enum (**Confirmed**) |
| --- | --- |
| 0, 1, 2, 3, 4 | Doom 1.2; Doom 1.666; Doom/Doom II 1.9; Ultimate Doom/Doom95; Final Doom |
| 5, 6 | DosDoom 0.47; TASDoom |
| 7, 8, 9 | Boom compatibility mode; Boom 2.01; Boom 2.02 |
| 10, 11 | LxDoom 1.3.2+; MBF |
| 12, 13, 14, 15, 16, 17 | PrBoom 2.03beta; 2.1.0–2.1.1; 2.2.x; 2.3.x; 2.4.0; latest PrBoom |
| 18, 19, 20 | Placeholder enum entries |
| 21 | MBF21 |

Source for every enum-table row: [prboom2/src/doomtype.h:113–140](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomtype.h#L113-L140).

**Confirmed.** Accepted words are `vanilla`, `boom` (9), `mbf` (11), `mbf21` (21).
`vanilla` resolves to 4 for TNT/Plutonia, 2 for other commercial Doom, otherwise 3.
Omitting the flag checks the winning (last-loaded) `COMPLVL` lump when not playing a demo;
its entire byte length must match one of those words, case-insensitively (no
numeric form or whitespace stripping in the lump decoder). [prboom2/src/w_wad.c:360–418](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/w_wad.c#L360-L418); [prboom2/src/w_wad.h:147–153](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/w_wad.h#L147-L153). An unrecognized lump
falls through. Then the configured `default_compatibility_level` applies, whose
upstream default is **21**, not automatic vanilla. Raven forces compatibility 0;
ZDoom map format forces 21 before CLI handling. Demo headers have their own
compatibility-reading path. [prboom2/src/dsda/settings.c:60–170](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L60-L170);
[prboom2/src/dsda/configuration.c:237–240](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L237-L240); [prboom2/src/g_game.c:2753–2772](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L2753-L2772);
[prboom2/src/g_game.c:3556–3652](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L3556-L3652).

### Search paths, autoload, and probe/error behavior

**Confirmed.** No `-home` option is registered. Linux home is `HOME`, then passwd
lookup, then `/`; `XDG_DATA_HOME` and `XDG_DATA_DIRS` affect search/config roots.
`DOOMWADDIR` is a single search directory and `DOOMWADPATH` a path-separated list.
The search sequence includes executable directory, engine config root, cwd,
`DOOMWADDIR`, compiled paths, SDL base and AppImage resource path, `HOME/doom`,
`HOME`, XDG data `games/doom`, system XDG `games/doom` and `doom`, then
`DOOMWADPATH`. Absolute explicit paths avoid this discovery policy.
[prboom2/src/dsda/args.c:55–694](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L55-L694); [prboom2/src/SDL/i_system.c:313–398](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_system.c#L313-L398);
[prboom2/src/SDL/i_system.c:470–615](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_system.c#L470-L615).

**Confirmed.** Autoload root is `I_ConfigDir()/autoload`, independent of `-config`.
Subdirectories are `all-all`, a game family (`doom-all`, `heretic-all`, `hexen-all`,
`chex-all`, `freedoom-all` where applicable), IWAD filename (including extension),
and each PWAD filename (including extension), e.g. `doom2.wad` and `example.wad`. Global/game/IWAD content precedes
explicit PWADs; PWAD-specific autoload follows them. Each directory loads sorted,
case-insensitive WAD/LMP and ZIP lists, and sorted DEH/BEX lists in separate phases.
The basename helper strips directories but retains extensions. [prboom2/src/dsda/utility.c:223–232](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/utility.c#L223-L232).
[prboom2/src/d_main.c:1315–1339](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1315-L1339); [prboom2/src/d_main.c:1412–1458](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1412-L1458);
[prboom2/src/d_main.c:1488–1609](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1488-L1609); [prboom2/src/d_main.c:2059–2070](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2059-L2070).

**Confirmed.** The source-safe, non-interactive engine smoke invocation is
`dsda-doom -v` (or official AppImage path with sole argument `-v`). It tests early
program entry and version reporting; it does not validate WAD loading, rendering,
audio, input, config persistence or saves. The version formatter prints `PROJECT_NAME vPROJECT_VERSION` plus the upstream URL. `I_Error` calls `I_SafeExit(-1)`; normal explicit exits
use 0. On Linux, the observable low-byte exit status for `-1` is expected to be 255
(**Proposed**, verify when running the asset). Not every logged error is fatal:
invalid data/save directory arguments can fall back and continue.
[prboom2/src/SDL/i_main.c:139–143](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L139-L143); [prboom2/src/SDL/i_main.c:177–199](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L177-L199);
[prboom2/src/SDL/i_main.c:266–296](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L266-L296); [prboom2/src/SDL/i_system.c:166–175](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_system.c#L166-L175);
[prboom2/src/lprintf.c:122–135](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/lprintf.c#L122-L135); [prboom2/src/dsda/data_organizer.c:42–67](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/data_organizer.c#L42-L67).

Implications for DDL:

- **Proposed:** Emit one ordered `-file` list and one ordered `-deh` list, with absolute paths and explicit IWAD/complevel/skill.
- **Proposed:** Use `-noautoload` for reproducible recipes; do not depend on global directories or COMPLVL discovery for reviewed recipes.
- **Proposed:** Use `-v` for installation smoke tests; capture stdout/stderr and status separately. A successful probe does not mark a release Deck-validated.
- **Proposed:** Allow only reviewed compatibility values in recipes; do not offer placeholders merely because the parser accepts them.

## 2. Pistol start

**Confirmed.** `-pistolstart` / `-wandstart` enables automatic reset on each map.
`dsda_pistol_start` is a temporary game modifier (default 0, boolean);
`dsda_always_pistol_start` is the persistent preference (default 0, boolean).
Both appear as “Pistol Start” and “Always Pistol Start” in Compatibility → Casual Play settings.
Only the latter is in the persisted config output list. Startup copies the CLI
flag or persistent preference into the temporary modifier. Disabling Pistol Start
also disables Always Pistol Start; enabling Always enables the temporary modifier.
[prboom2/src/dsda/args.c:206–210](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L206-L210); [prboom2/src/dsda/configuration.c:1004–1011](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1004-L1011);
[prboom2/src/m_misc.c:288–332](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L288-L332); [prboom2/src/m_menu.c:3822–3841](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_menu.c#L3822-L3841);
[prboom2/src/dsda/skill_info.c:153–214](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/skill_info.c#L153-L214).

**Confirmed.** At map load, `pistolstart` calls `G_PlayerReborn(0)` when
`allow_incompatibility` is true; this macro means neither recording nor playing
back a demo. There is no complevel-specific guard at this reset point. Ordinary
new-game player initialization also gives starting inventory, but is distinct
from repeated map resets. The save loader restores the saved temporary modifier
unless Always Pistol Start is enabled. [prboom2/src/g_game.c:1218–1232](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L1218-L1232);
[prboom2/src/g_game.c:1862–1912](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L1862-L1912); [prboom2/src/doomstat.h:246](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomstat.h#L246);
[prboom2/src/dsda/save.c:159–172](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/save.c#L159-L172).

**Confirmed.** Recording with the CLI flag fails explicitly. Config-based pistol
start is disabled by the game-modifier logic during demo recording/playback;
turning off strict mode does not bypass `allow_incompatibility`.
[prboom2/src/dsda/demo.c:271–292](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/demo.c#L271-L292); [prboom2/src/dsda/skill_info.c:160–167](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/skill_info.c#L160-L167);
[prboom2/src/doomstat.h:246](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomstat.h#L246).

Implications for DDL:

- **Proposed:** Offer per-launch automatic pistol start via `-pistolstart` for ordinary play, without changing shared persistent Always Pistol Start.
- **Proposed:** Reject pistol-start + recording in launch validation, and explain that config toggles do not enable demo-compatible per-map resets.
- **Needs Deck Validation:** Confirm menu UX, map transitions and save reload behavior with representative Doom/Boom/MBF21 content.

## 3. Config file ownership and persistence

**Confirmed.** Linux default file is `I_ConfigDir()/dsda-doom.cfg`. The engine first
uses `HOME/.dsda-doom` if it exists (`access(F_OK)`); otherwise it uses
`XDG_DATA_HOME/dsda-doom`, falling back to `HOME/.local/share/dsda-doom`.
This is a **data** directory, not `XDG_CONFIG_HOME`. The root directory is created.
`-config` changes the filename only. [prboom2/src/SDL/i_system.c:313–398](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_system.c#L313-L398);
[prboom2/src/m_misc.c:705–715](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L705-L715).

**Confirmed.** Format is line-oriented `name value`. Names are case-sensitive.
Reader buffers are 32,000 bytes and names at most 79 bytes. A line needs name
and value; non-alphanumeric first characters are skipped (hence generated `#`
comments). Integers use `%i` (decimal/octal and `0x` hex); strings begin with `"`
and the reader removes first and final character without a general escaping
parser. Do not assume JSON escapes, robust malformed-quote recovery, or general
inline comments. Binding lines have their own grammar in §4.
[prboom2/src/m_misc.c:678–809](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L678-L809); [prboom2/src/dsda/configuration.c:1686–1693](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1686-L1693).

**Confirmed.** Defaults initialize before loading. Missing keys retain defaults;
unknown file keys are ignored; mismatched string/integer types are ignored.
Integers above/below range clamp, except below-range `-1` resets to default;
odd values of `CONF_EVEN` settings reset to default. Unknown lines/comments are
not preserved on save. Missing file is tolerated; a later save opens that exact
path with `"w"`, creating it if its parent exists and permissions allow. Missing
parent directories for an arbitrary `-config` path are not created by this code;
write-open failure is silently ignored. [prboom2/src/m_misc.c:600–639](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L600-L639);
[prboom2/src/m_misc.c:693–723](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L693-L723); [prboom2/src/dsda/configuration.c:1381–1419](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1381-L1419);
[prboom2/src/dsda/configuration.c:1443–1484](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1443-L1484).

**Confirmed.** Save truncates and rewrites the whole known persisted configuration
and all binding profiles, using persistent values, not a changed-key patch or
atomic replacement. Normal quit calls save; error/crash safe exit skips `I_Quit`
because its `run_on_error` is false. Timing-demo completion also calls save even
when configured to keep running. Search of the complete source found these two
call sites, not periodic settings-menu writes. Hard termination cannot guarantee
persistence. `M_ForgetCurrentConfig` can suppress writes. [prboom2/src/m_misc.c:588–672](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L588-L672);
[prboom2/src/dsda/configuration.c:1474–1484](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1474-L1484); [prboom2/src/SDL/i_main.c:177–223](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L177-L223);
[prboom2/src/SDL/i_main.c:320–331](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_main.c#L320-L331); [prboom2/src/g_game.c:4025–4055](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L4025-L4055).

**Confirmed.** `-assign key=value …` applies transient overrides; `-update
key=value …` applies persistent ones, with update processed before assign.
Unlike unknown file keys, unknown CLI config assignments fail. These target
registered config values, not the separate `input_*` binding table.
[prboom2/src/dsda/args.c:326–335](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/args.c#L326-L335); [prboom2/src/dsda/configuration.c:1486–1542](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1486-L1542).

Implications for DDL:

- **Proposed:** Create the private config parent first, back up before migrations, and do not write the active file while DSDA runs (including timedemo completion).
- **Proposed:** Merge values against a versioned prior baseline after engine exit; preserve the engine-generated binding format, not comments or original layout.
- **Proposed:** Use transient `-assign` for supported per-profile scalar overrides where appropriate; avoid persisting recipe preferences into the shared baseline.
- **Proposed:** Treat a successful engine launch as insufficient proof that config was saved; explicitly verify the selected path in future integration tests.

## 4. Input config and complete binding inventory

**Confirmed.** Tables below transcribe tagged default values and inclusive ranges.
`INT_MAX` is the C integer maximum, not a proposed sensitivity ceiling. Booleans
accept 0–1. Defaults are upstream defaults, **not Deck tuning**.
[prboom2/src/dsda/configuration.c:77–82](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L77-L82); [prboom2/src/dsda/configuration.c:644–695](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L644-L695);
[prboom2/src/dsda/configuration.c:1295–1338](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1295-L1338).

| Scalar key | Default | Inclusive range | Source (**Confirmed**) |
| --- | --- | --- | --- |
| `vanilla_keymap` | `0` | `0–1` | [prboom2/src/dsda/configuration.c:245–248](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L245-L248) |
| `input_profile` | `0` | `0–2` | [prboom2/src/dsda/configuration.c:261–264](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L261-L264) |
| `movement_vertmouse` | `0` | `0–1` | [prboom2/src/dsda/configuration.c:339–342](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L339-L342) |
| `allow_freelook` | `0` | `0–1` | [prboom2/src/dsda/configuration.c:343–346](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L343-L346) |
| `autorun` | `1` | `0–1` | [prboom2/src/dsda/configuration.c:347–350](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L347-L350) |
| `use_mouse` | `1` | `0–1` | [prboom2/src/dsda/configuration.c:644–647](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L644-L647) |
| `mouse_menu_navigation` | `1` | `0–1` | [prboom2/src/dsda/configuration.c:648–651](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L648-L651) |
| `mouse_sensitivity_horiz` | `10` | `0–INT_MAX` | [prboom2/src/dsda/configuration.c:652–655](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L652-L655) |
| `mouse_sensitivity_vert` | `1` | `0–INT_MAX` | [prboom2/src/dsda/configuration.c:656–659](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L656-L659) |
| `dsda_mouse_acceleration` | `0` | `0–INT_MAX` | [prboom2/src/dsda/configuration.c:660–663](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L660-L663) |
| `mouse_sensitivity_mlook` | `10` | `0–INT_MAX` | [prboom2/src/dsda/configuration.c:664–667](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L664-L667) |
| `mouse_stutter_correction` | `1` | `0–1` | [prboom2/src/dsda/configuration.c:668–671](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L668-L671) |
| `mouse_doubleclick_as_use` | `0` | `0–1` | [prboom2/src/dsda/configuration.c:672–675](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L672-L675) |
| `mouse_carrytics` | `1` | `0–1` | [prboom2/src/dsda/configuration.c:676–679](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L676-L679) |
| `movement_mouseinvert` | `0` | `0–1` | [prboom2/src/dsda/configuration.c:680–683](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L680-L683) |
| `movement_mousestrafedivisor` | `4` | `1–INT_MAX` | [prboom2/src/dsda/configuration.c:684–687](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L684-L687) |
| `dsda_fine_sensitivity` | `0` | `0–99` | [prboom2/src/dsda/configuration.c:688–691](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L688-L691) |
| `use_game_controller` | `0` | `0–2` | [prboom2/src/dsda/configuration.c:692–695](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L692-L695) |
| `movement_strafe50` | `0` | `0–1` | [prboom2/src/dsda/configuration.c:700–703](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L700-L703) |
| `movement_strafe50onturns` | `0` | `0–1` | [prboom2/src/dsda/configuration.c:704–707](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L704-L707) |
| `movement_shorttics` | `0` | `0–1` | [prboom2/src/dsda/configuration.c:708–711](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L708-L711) |
| `left_analog_deadzone` | `6556` | `0–16384` | [prboom2/src/dsda/configuration.c:1295–1298](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1295-L1298) |
| `right_analog_deadzone` | `6556` | `0–16384` | [prboom2/src/dsda/configuration.c:1299–1302](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1299-L1302) |
| `left_trigger_deadzone` | `6556` | `0–16384` | [prboom2/src/dsda/configuration.c:1303–1306](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1303-L1306) |
| `right_trigger_deadzone` | `6556` | `0–16384` | [prboom2/src/dsda/configuration.c:1307–1310](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1307-L1310) |
| `left_analog_sensitivity_x` | `100` | `0–16384` | [prboom2/src/dsda/configuration.c:1311–1314](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1311-L1314) |
| `left_analog_sensitivity_y` | `100` | `0–16384` | [prboom2/src/dsda/configuration.c:1315–1318](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1315-L1318) |
| `right_analog_sensitivity_x` | `1536` | `0–16384` | [prboom2/src/dsda/configuration.c:1319–1322](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1319-L1322) |
| `right_analog_sensitivity_y` | `768` | `0–16384` | [prboom2/src/dsda/configuration.c:1323–1326](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1323-L1326) |
| `analog_look_acceleration` | `0` | `0–INT_MAX` | [prboom2/src/dsda/configuration.c:1327–1330](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1327-L1330) |
| `swap_analogs` | `0` | `0–1` | [prboom2/src/dsda/configuration.c:1331–1334](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1331-L1334) |
| `invert_analog_look` | `0` | `0–1` | [prboom2/src/dsda/configuration.c:1335–1338](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1335-L1338) |
| `playback_mouse_controls` | `1` | `0–1` | [prboom2/src/dsda/configuration.c:1363–1366](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1363-L1366) |

**Confirmed — controller selection caveat.** Although `use_game_controller` has
range 0–2, initialization assigns `configured_value && !nojoy`, a C logical
expression that collapses nonzero values to 1 before subtracting 1 to open the
SDL controller. Do not promise that value 2 selects the second controller from
this implementation. [prboom2/src/dsda/game_controller.c:197–229](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/game_controller.c#L197-L229).

**Confirmed — binding grammar.** Each line is
`input_action key[,key…] mouse_index joy_index | key[,key…] mouse_index joy_index | key[,key…] mouse_index joy_index`.
There are three input profiles; `input_profile` selects 0–2. `0` means no keyboard
key, `-1` no mouse/gamepad button. Keyboard IDs accepted are 1–511; mouse indices
-1–7; gamepad indices -1–22. Out-of-range additions are ignored. Omitted profile
segments retain initialized defaults. Defaults in the inventory apply to all
three profiles; a generated file always writes all three segments.
[prboom2/src/m_misc.c:643–672](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L643-L672); [prboom2/src/m_misc.c:696–701](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L696-L701);
[prboom2/src/m_misc.c:767–805](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L767-L805); [prboom2/src/dsda/input.h:24–27](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/input.h#L24-L27);
[prboom2/src/dsda/input.c:289–318](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/input.c#L289-L318).

**Confirmed — button names/IDs.** File syntax uses integers, **not** menu names.
The enum is contiguous from 0, in this order:
[prboom2/src/dsda/game_controller.h:22–49](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/game_controller.h#L22-L49); [prboom2/src/dsda/game_controller.c:47–72](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/game_controller.c#L47-L72).

| IDs | Menu names, same order (**Confirmed**) |
| --- | --- |
| 0–6 | pad a, pad b, pad x, pad y, pad back, pad guide, pad start |
| 7–10 | lstick, rstick, pad l, pad r |
| 11–14 | dpad u, dpad d, dpad l, dpad r |
| 15–20 | misc 1, paddle 1, paddle 2, paddle 3, paddle 4, touchpad |
| 21–22 | pad lt, pad rt |

**Confirmed — axes and mouse.** Sticks are not arbitrary action bindings:
left stick produces move events, right stick look events; `swap_analogs` swaps
these roles. Triggers become digital buttons 21/22 after their deadzone; there
is no action-to-axis config grammar in this implementation. Mouse indices
0/1/2 map to SDL left/right/middle; 3/4/5/6/7 map to SDL buttons 6/7/4/5/8.
Wheel directions are keyboard-style IDs (`KEYD_MWHEELUP` etc.), not mouse slots.
[prboom2/src/dsda/game_controller.c:82–174](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/game_controller.c#L82-L174); [prboom2/src/SDL/i_video.c:295–308](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_video.c#L295-L308);
[prboom2/src/SDL/i_video.c:375–395](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_video.c#L375-L395); [prboom2/src/doomdef.h:407–410](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomdef.h#L407-L410).

**Proposed — syntax example.** Bind attack to keyboard Ctrl (157), left mouse
(0), and right trigger (22) in each profile using:

```text
input_fire 157 0 22 | 157 0 22 | 157 0 22
```

This example matches the source default: [prboom2/src/m_misc.c:425](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L425);
[prboom2/src/doomdef.h:362](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomdef.h#L362); [prboom2/src/dsda/game_controller.h:22–49](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/game_controller.h#L22-L49).

### Every binding key and default

**Confirmed.** Complete `input_defs` inventory below (including menus, cheats,
Raven, demo/build and scripts). The keyboard column uses C source notation:
quoted characters are their character codes; `KEYD_*` symbols resolve in the
linked key-definition block, and `0` is unbound. This notation is for auditing,
not literal file syntax; saved files contain decimal integers. `DSDA_` controller
names below are abbreviated to their suffix, with the numeric mapping above.
[prboom2/src/m_misc.c:391–581](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L391-L581); [prboom2/src/doomdef.h:335–424](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomdef.h#L335-L424).

| Binding key | Keyboard default | Mouse default | Controller default | Source (**Confirmed**) |
| --- | --- | --- | --- | --- |
| `input_forward` | `'w'` | 2 | `-1` | [prboom2/src/m_misc.c:392](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L392) |
| `input_backward` | `'s'` | -1 | `-1` | [prboom2/src/m_misc.c:393](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L393) |
| `input_turnleft` | `'e'` | -1 | `-1` | [prboom2/src/m_misc.c:394](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L394) |
| `input_turnright` | `'q'` | -1 | `-1` | [prboom2/src/m_misc.c:395](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L395) |
| `input_speed` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:396](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L396) |
| `input_strafeleft` | `'a'` | -1 | `-1` | [prboom2/src/m_misc.c:397](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L397) |
| `input_straferight` | `'d'` | -1 | `-1` | [prboom2/src/m_misc.c:398](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L398) |
| `input_strafe` | `0` | 1 | `LEFTSHOULDER` | [prboom2/src/m_misc.c:399](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L399) |
| `input_autorun` | `KEYD_CAPSLOCK` | -1 | `LEFTSTICK` | [prboom2/src/m_misc.c:400](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L400) |
| `input_reverse` | `'/'` | -1 | `RIGHTSTICK` | [prboom2/src/m_misc.c:401](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L401) |
| `input_use` | `' '` | -1 | `A` | [prboom2/src/m_misc.c:402](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L402) |
| `input_flyup` | `'.'` | -1 | `DPAD_UP` | [prboom2/src/m_misc.c:403](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L403) |
| `input_flydown` | `','` | -1 | `DPAD_DOWN` | [prboom2/src/m_misc.c:404](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L404) |
| `input_flycenter` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:405](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L405) |
| `input_mlook` | `'\\'` | -1 | `-1` | [prboom2/src/m_misc.c:406](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L406) |
| `input_novert` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:407](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L407) |
| `input_weapon1` | `'1'` | -1 | `-1` | [prboom2/src/m_misc.c:409](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L409) |
| `input_weapon2` | `'2'` | -1 | `-1` | [prboom2/src/m_misc.c:410](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L410) |
| `input_weapon3` | `'3'` | -1 | `-1` | [prboom2/src/m_misc.c:411](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L411) |
| `input_weapon4` | `'4'` | -1 | `-1` | [prboom2/src/m_misc.c:412](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L412) |
| `input_weapon5` | `'5'` | -1 | `-1` | [prboom2/src/m_misc.c:413](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L413) |
| `input_weapon6` | `'6'` | -1 | `-1` | [prboom2/src/m_misc.c:414](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L414) |
| `input_weapon7` | `'7'` | -1 | `-1` | [prboom2/src/m_misc.c:415](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L415) |
| `input_weapon8` | `'8'` | -1 | `-1` | [prboom2/src/m_misc.c:416](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L416) |
| `input_weapon9` | `'9'` | -1 | `-1` | [prboom2/src/m_misc.c:417](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L417) |
| `input_nextweapon` | `0` | -1 | `Y` | [prboom2/src/m_misc.c:418](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L418) |
| `input_prevweapon` | `0` | -1 | `X` | [prboom2/src/m_misc.c:419](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L419) |
| `input_toggleweapon` | `'0'` | -1 | `-1` | [prboom2/src/m_misc.c:420](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L420) |
| `input_fire` | `KEYD_RCTRL` | 0 | `TRIGGERRIGHT` | [prboom2/src/m_misc.c:421](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L421) |
| `input_help` | `KEYD_F1` | -1 | `-1` | [prboom2/src/m_misc.c:423](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L423) |
| `input_pause` | `KEYD_PAUSE` | -1 | `-1` | [prboom2/src/m_misc.c:424](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L424) |
| `input_map` | `KEYD_TAB` | -1 | `TRIGGERLEFT` | [prboom2/src/m_misc.c:425](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L425) |
| `input_soundvolume` | `KEYD_F4` | -1 | `-1` | [prboom2/src/m_misc.c:426](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L426) |
| `input_hud` | `KEYD_F5` | -1 | `-1` | [prboom2/src/m_misc.c:427](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L427) |
| `input_messages` | `KEYD_F8` | -1 | `-1` | [prboom2/src/m_misc.c:428](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L428) |
| `input_gamma` | `KEYD_F11` | -1 | `-1` | [prboom2/src/m_misc.c:429](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L429) |
| `input_spy` | `KEYD_F12` | -1 | `-1` | [prboom2/src/m_misc.c:430](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L430) |
| `input_zoomin` | `'='` | -1 | `-1` | [prboom2/src/m_misc.c:431](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L431) |
| `input_zoomout` | `'-'` | -1 | `-1` | [prboom2/src/m_misc.c:432](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L432) |
| `input_screenshot` | `'*'` | -1 | `-1` | [prboom2/src/m_misc.c:433](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L433) |
| `input_savegame` | `KEYD_F2` | -1 | `-1` | [prboom2/src/m_misc.c:434](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L434) |
| `input_loadgame` | `KEYD_F3` | -1 | `-1` | [prboom2/src/m_misc.c:435](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L435) |
| `input_quicksave` | `KEYD_F6` | -1 | `-1` | [prboom2/src/m_misc.c:436](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L436) |
| `input_quickload` | `KEYD_F9` | -1 | `-1` | [prboom2/src/m_misc.c:437](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L437) |
| `input_level_table` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:438](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L438) |
| `input_endgame` | `KEYD_F7` | -1 | `-1` | [prboom2/src/m_misc.c:439](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L439) |
| `input_quit` | `KEYD_F10` | -1 | `-1` | [prboom2/src/m_misc.c:440](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L440) |
| `input_map_follow` | `'f'` | -1 | `-1` | [prboom2/src/m_misc.c:442](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L442) |
| `input_map_zoomin` | `'='` | -1 | `-1` | [prboom2/src/m_misc.c:443](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L443) |
| `input_map_zoomout` | `'-'` | -1 | `-1` | [prboom2/src/m_misc.c:444](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L444) |
| `input_map_up` | `KEYD_UPARROW` | -1 | `-1` | [prboom2/src/m_misc.c:445](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L445) |
| `input_map_down` | `KEYD_DOWNARROW` | -1 | `-1` | [prboom2/src/m_misc.c:446](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L446) |
| `input_map_left` | `KEYD_LEFTARROW` | -1 | `-1` | [prboom2/src/m_misc.c:447](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L447) |
| `input_map_right` | `KEYD_RIGHTARROW` | -1 | `-1` | [prboom2/src/m_misc.c:448](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L448) |
| `input_map_mark` | `'m'` | -1 | `-1` | [prboom2/src/m_misc.c:449](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L449) |
| `input_map_clear` | `'c'` | -1 | `-1` | [prboom2/src/m_misc.c:450](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L450) |
| `input_map_gobig` | `'0'` | -1 | `-1` | [prboom2/src/m_misc.c:451](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L451) |
| `input_map_grid` | `'g'` | -1 | `-1` | [prboom2/src/m_misc.c:452](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L452) |
| `input_map_rotate` | `'r'` | -1 | `-1` | [prboom2/src/m_misc.c:453](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L453) |
| `input_map_overlay` | `'o'` | -1 | `-1` | [prboom2/src/m_misc.c:454](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L454) |
| `input_map_textured` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:455](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L455) |
| `input_map_highlight_by_tag` | `'h'` | -1 | `-1` | [prboom2/src/m_misc.c:456](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L456) |
| `input_repeat_message` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:458](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L458) |
| `input_speed_up` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:460](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L460) |
| `input_speed_down` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:461](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L461) |
| `input_speed_default` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:462](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L462) |
| `input_demo_skip` | `KEYD_INSERT` | -1 | `-1` | [prboom2/src/m_misc.c:463](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L463) |
| `input_demo_endlevel` | `KEYD_END` | -1 | `-1` | [prboom2/src/m_misc.c:464](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L464) |
| `input_walkcamera` | `KEYD_KEYPAD0` | -1 | `-1` | [prboom2/src/m_misc.c:465](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L465) |
| `input_join_demo` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:466](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L466) |
| `input_restart` | `KEYD_HOME` | -1 | `-1` | [prboom2/src/m_misc.c:467](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L467) |
| `input_nextlevel` | `KEYD_PAGEDOWN` | -1 | `-1` | [prboom2/src/m_misc.c:468](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L468) |
| `input_prevlevel` | `KEYD_PAGEUP` | -1 | `-1` | [prboom2/src/m_misc.c:469](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L469) |
| `input_showalive` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:470](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L470) |
| `input_menu_down` | `KEYD_DOWNARROW` | -1 | `DPAD_DOWN` | [prboom2/src/m_misc.c:472](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L472) |
| `input_menu_up` | `KEYD_UPARROW` | -1 | `DPAD_UP` | [prboom2/src/m_misc.c:473](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L473) |
| `input_menu_left` | `KEYD_LEFTARROW` | -1 | `DPAD_LEFT` | [prboom2/src/m_misc.c:474](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L474) |
| `input_menu_right` | `KEYD_RIGHTARROW` | -1 | `DPAD_RIGHT` | [prboom2/src/m_misc.c:475](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L475) |
| `input_menu_backspace` | `KEYD_BACKSPACE` | -1 | `B` | [prboom2/src/m_misc.c:476](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L476) |
| `input_menu_enter` | `KEYD_ENTER` | -1 | `A` | [prboom2/src/m_misc.c:477](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L477) |
| `input_menu_escape` | `KEYD_ESCAPE` | -1 | `START` | [prboom2/src/m_misc.c:478](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L478) |
| `input_menu_clear` | `KEYD_DEL` | -1 | `BACK` | [prboom2/src/m_misc.c:479](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L479) |
| `input_iddqd` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:481](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L481) |
| `input_idkfa` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:482](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L482) |
| `input_idfa` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:483](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L483) |
| `input_idclip` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:484](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L484) |
| `input_idbeholdh` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:485](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L485) |
| `input_idbeholdm` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:486](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L486) |
| `input_idbeholdv` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:487](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L487) |
| `input_idbeholds` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:488](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L488) |
| `input_idbeholdi` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:489](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L489) |
| `input_idbeholdr` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:490](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L490) |
| `input_idbeholda` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:491](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L491) |
| `input_idbeholdl` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:492](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L492) |
| `input_idmypos` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:493](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L493) |
| `input_idrate` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:494](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L494) |
| `input_iddt` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:495](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L495) |
| `input_ponce` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:496](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L496) |
| `input_shazam` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:497](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L497) |
| `input_chicken` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:498](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L498) |
| `input_lookup` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:500](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L500) |
| `input_lookdown` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:501](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L501) |
| `input_lookcenter` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:502](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L502) |
| `input_use_artifact` | `KEYD_RSHIFT` | -1 | `RIGHTSHOULDER` | [prboom2/src/m_misc.c:503](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L503) |
| `input_arti_tome` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:504](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L504) |
| `input_arti_quartz` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:505](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L505) |
| `input_arti_urn` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:506](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L506) |
| `input_arti_bomb` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:507](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L507) |
| `input_arti_ring` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:508](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L508) |
| `input_arti_chaosdevice` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:509](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L509) |
| `input_arti_shadowsphere` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:510](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L510) |
| `input_arti_wings` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:511](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L511) |
| `input_arti_torch` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:512](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L512) |
| `input_arti_morph` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:513](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L513) |
| `input_invleft` | `KEYD_MWHEELDOWN` | -1 | `DPAD_LEFT` | [prboom2/src/m_misc.c:514](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L514) |
| `input_invright` | `KEYD_MWHEELUP` | -1 | `DPAD_RIGHT` | [prboom2/src/m_misc.c:515](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L515) |
| `input_store_quick_key_frame` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:516](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L516) |
| `input_restore_quick_key_frame` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:517](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L517) |
| `input_rewind` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:522](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L522) |
| `input_cycle_profile` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:519](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L519) |
| `input_cycle_palette` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:520](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L520) |
| `input_command_display` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:521](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L521) |
| `input_strict_mode` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:522](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L522) |
| `input_console` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:523](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L523) |
| `input_coordinate_display` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:524](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L524) |
| `input_fps` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:525](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L525) |
| `input_avj` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:526](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L526) |
| `input_exhud` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:527](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L527) |
| `input_mute_sfx` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:528](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L528) |
| `input_mute_music` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:529](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L529) |
| `input_cheat_codes` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:530](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L530) |
| `input_notarget` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:531](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L531) |
| `input_freeze` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:532](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L532) |
| `input_build` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:534](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L534) |
| `input_build_advance_frame` | `KEYD_RIGHTARROW` | -1 | `-1` | [prboom2/src/m_misc.c:535](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L535) |
| `input_build_reverse_frame` | `KEYD_LEFTARROW` | -1 | `-1` | [prboom2/src/m_misc.c:536](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L536) |
| `input_build_reset_command` | `KEYD_DEL` | -1 | `-1` | [prboom2/src/m_misc.c:537](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L537) |
| `input_build_source` | `KEYD_RSHIFT` | -1 | `-1` | [prboom2/src/m_misc.c:538](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L538) |
| `input_build_forward` | `'w'` | -1 | `-1` | [prboom2/src/m_misc.c:539](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L539) |
| `input_build_backward` | `'s'` | -1 | `-1` | [prboom2/src/m_misc.c:540](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L540) |
| `input_build_fine_forward` | `'t'` | -1 | `-1` | [prboom2/src/m_misc.c:541](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L541) |
| `input_build_fine_backward` | `'g'` | -1 | `-1` | [prboom2/src/m_misc.c:542](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L542) |
| `input_build_turn_left` | `'q'` | -1 | `-1` | [prboom2/src/m_misc.c:543](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L543) |
| `input_build_turn_right` | `'e'` | -1 | `-1` | [prboom2/src/m_misc.c:544](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L544) |
| `input_build_strafe_left` | `'a'` | -1 | `-1` | [prboom2/src/m_misc.c:545](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L545) |
| `input_build_strafe_right` | `'d'` | -1 | `-1` | [prboom2/src/m_misc.c:546](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L546) |
| `input_build_fine_strafe_left` | `'f'` | -1 | `-1` | [prboom2/src/m_misc.c:547](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L547) |
| `input_build_fine_strafe_right` | `'h'` | -1 | `-1` | [prboom2/src/m_misc.c:548](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L548) |
| `input_build_use` | `KEYD_SPACEBAR` | -1 | `-1` | [prboom2/src/m_misc.c:549](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L549) |
| `input_build_fire` | `KEYD_RCTRL` | -1 | `-1` | [prboom2/src/m_misc.c:550](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L550) |
| `input_build_weapon1` | `'1'` | -1 | `-1` | [prboom2/src/m_misc.c:551](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L551) |
| `input_build_weapon2` | `'2'` | -1 | `-1` | [prboom2/src/m_misc.c:552](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L552) |
| `input_build_weapon3` | `'3'` | -1 | `-1` | [prboom2/src/m_misc.c:553](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L553) |
| `input_build_weapon4` | `'4'` | -1 | `-1` | [prboom2/src/m_misc.c:554](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L554) |
| `input_build_weapon5` | `'5'` | -1 | `-1` | [prboom2/src/m_misc.c:555](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L555) |
| `input_build_weapon6` | `'6'` | -1 | `-1` | [prboom2/src/m_misc.c:556](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L556) |
| `input_build_weapon7` | `'7'` | -1 | `-1` | [prboom2/src/m_misc.c:557](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L557) |
| `input_build_weapon8` | `'8'` | -1 | `-1` | [prboom2/src/m_misc.c:558](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L558) |
| `input_build_weapon9` | `'9'` | -1 | `-1` | [prboom2/src/m_misc.c:559](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L559) |
| `input_jump` | `KEYD_RALT` | -1 | `B` | [prboom2/src/m_misc.c:561](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L561) |
| `input_hexen_arti_incant` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:562](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L562) |
| `input_hexen_arti_summon` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:563](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L563) |
| `input_hexen_arti_disk` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:564](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L564) |
| `input_hexen_arti_flechette` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:565](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L565) |
| `input_hexen_arti_banishment` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:566](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L566) |
| `input_hexen_arti_boots` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:567](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L567) |
| `input_hexen_arti_krater` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:568](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L568) |
| `input_hexen_arti_bracers` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:569](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L569) |
| `input_script_0` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:571](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L571) |
| `input_script_1` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:572](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L572) |
| `input_script_2` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:573](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L573) |
| `input_script_3` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:574](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L574) |
| `input_script_4` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:575](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L575) |
| `input_script_5` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:576](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L576) |
| `input_script_6` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:577](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L577) |
| `input_script_7` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:578](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L578) |
| `input_script_8` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:579](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L579) |
| `input_script_9` | `0` | -1 | `-1` | [prboom2/src/m_misc.c:580](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L580) |

Implications for DDL:

- **Proposed:** Generate numeric binding triples for all three profiles; preserve extra keyboard bindings and each independent profile on migration.
- **Proposed:** Start sensitivity templates from these audited values, then calibrate on Deck; leave controller selection/hotplug promises behind a validation gate.
- **Proposed:** Do not invent axis-binding keys or use display names in config files.
- **Needs Deck Validation:** Native pad identity, rear-button visibility, gyro-as-mouse coexistence, SDL mapping and comfortable sensitivity/deadzone values.

## 5. Strict mode and demos

**Confirmed.** `dsda_strict_mode` defaults to 1, but effective strict mode is
`config && demorecording`. It is intentionally not persisted in the config output
list. Recording startup resets strict according to `-tas`, `-build` or
`-dsdademo`, so editing the config to 0 is not a reliable recording-mode selector.
There is an `input_strict_mode` action and a “Strict Mode” setting on the Demo settings page.
[prboom2/src/dsda/settings.c:172–174](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L172-L174); [prboom2/src/dsda/settings.c:196–198](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L196-L198);
[prboom2/src/dsda/configuration.c:335–338](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L335-L338); [prboom2/src/m_misc.c:288](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L288);
[prboom2/src/m_misc.c:522](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L522); [prboom2/src/m_menu.c:3192–3210](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_menu.c#L3192-L3210);
[prboom2/src/dsda/demo.c:285–300](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/demo.c#L285-L300).

**Confirmed.** During strict recording, mouse and gamepad motion cannot be mixed:
seeing mouse motion blocks controller polling, seeing analog movement/look
blocks mouse polling. These are tracked game events, not merely device presence;
the flags reset for a new recording attempt. Non-strict use can mix them and
records a feature marker. Keyboard remains usable with either. Strict mode also
disables the 180-degree reverse action and rounds analog strafe movement to even
values. [prboom2/src/dsda/settings.c:361–389](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L361-L389); [prboom2/src/SDL/i_video.c:403–413](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_video.c#L403-L413);
[prboom2/src/SDL/i_video.c:459–469](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/SDL/i_video.c#L459-L469); [prboom2/src/g_game.c:553–567](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L553-L567);
[prboom2/src/g_game.c:1391–1439](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L1391-L1439); [prboom2/src/dsda/demo.c:295–300](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/demo.c#L295-L300).

**Confirmed.** Mouselook/free aim is governed separately from the strict toggle:
`dsda_FreeAim` returns false during demo recording/playback unless casual extended
command features are allowed, then respects map policy and `allow_freelook`.
The same gate governs right-stick vertical look. Thus `-tas` alone does not
promise ordinary demos with free aim. Strict configuration additionally forces
normal game speed and disables forced strafe50; menus prevent changes to strict
settings while strict recording is active.
[prboom2/src/dsda/excmd.c:54–73](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/excmd.c#L54-L73); [prboom2/src/doomstat.h:246](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomstat.h#L246);
[prboom2/src/g_game.c:1402–1438](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/g_game.c#L1402-L1438); [prboom2/src/dsda/configuration.c:233–236](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L233-L236);
[prboom2/src/dsda/configuration.c:700–707](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L700-L707); [prboom2/src/m_menu.c:1908–1915](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_menu.c#L1908-L1915).

Implications for DDL:

- **Proposed:** Surface strict recording as a distinct launch mode; require explicit skill/complevel and reject pistol start.
- **Proposed:** README can recommend manually configured keyboard+mouse emulation for strict recording; do not silently add `-tas` to preserve hybrid input.
- **Needs Deck Validation:** Which actual Steam Input events win the first-device gate, and whether buttons-only pad use behaves comfortably with gyro motion.

## 6. Saves, screenshots, demos, and profile isolation

**Confirmed.** Save base precedence is valid explicit `-save`, then `DOOMSAVEDIR`,
then `I_ConfigDir`. If `-save` is absent and `dsda_organized_saves` is enabled
(default 1), saves instead use the organized data directory. Supplying an invalid
`-save` still counts as an explicit flag, logs fallback, and bypasses organized
redirection. Saves use slot-based `.dsg` names.
[prboom2/src/dsda/save.c:258–296](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/save.c#L258-L296); [prboom2/src/dsda/data_organizer.c:42–67](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/data_organizer.c#L42-L67);
[prboom2/src/dsda/configuration.c:968–971](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L968-L971).

**Confirmed.** Data parent precedence is valid explicit `-data`, then
`DOOMDATADIR`, then `I_ConfigDir`. Organized data lives beneath
`parent/dsda_doom_data/<lowercase-iwad-stem>/<lowercase-pwad-stem>/…`, retaining up
to eight PWAD `.wad` stems after the IWAD. This identity is filename-based,
not hashes or DDL profile IDs; autoload content is excluded by source type.
Stats and splits also use this data directory.
[prboom2/src/dsda/data_organizer.c:35–143](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/data_organizer.c#L35-L143); [prboom2/src/dsda/wad_stats.c:48–58](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/wad_stats.c#L48-L58);
[prboom2/src/dsda/split_tracker.c:42–52](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/split_tracker.c#L42-L52).

**Confirmed.** Screenshots choose writable `-shotdir`, then writable
`screenshot_dir` (default empty), then cwd on Linux; names are `dsda0000` etc.
with PNG when SDL_image is built in, BMP otherwise. `-save`/`-data` do not redirect
screenshots. [prboom2/src/dsda/configuration.c:712–715](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L712-L715);
[prboom2/src/m_misc.c:847–856](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L847-L856); [prboom2/src/m_misc.c:894–938](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/m_misc.c#L894-L938).

**Confirmed.** Demo output uses the supplied recording name/path (relative paths
resolve from cwd); its extension is cut, `.lmp` appended, and collisions use
`-00002` etc. A trailing `$` requests time-based naming. Failed demos can be
redirected to `DataDir()/failed_demos` when `dsda_organize_failed_demos` is enabled
(default 0). [prboom2/src/dsda/demo.c:104–179](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/demo.c#L104-L179); [prboom2/src/dsda/demo.c:513–579](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/demo.c#L513-L579);
[prboom2/src/dsda/configuration.c:399–402](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L399-L402).

Implications for DDL:

- **Proposed:** Create existing per-profile `-save` and `-data` parents, pass explicit `-shotdir`, and use absolute recording output paths or a DDL-owned cwd.
- **Proposed:** Use DDL profile IDs for directories; filename-based organized saves alone cannot isolate versions, different recipes with matching names, or multiple profiles.
- **Proposed:** Keep one shared private config where appropriate, but make persistent Always Pistol Start and other global preferences visible when evaluating per-profile behavior.
- **Needs Deck Validation:** Save/resume under the official asset, screenshot writes and cwd behavior under Steam child-process launch.

## 7. Official AppImage packaging and host requirements

**Confirmed.** Tagged CI uses `ubuntu-22.04` for “Linux GCC”, builds via CMake,
installs SDL2/SDL2_image/SDL2_mixer and GL/GLU development packages plus audio/archive
dependencies, invokes CPack, uploads `build/*.appimage`, and attaches artifacts to
tag releases. LinuxGenerator finds linuxdeploy or downloads its **mutable
`continuous`** architecture AppImage, runs it with `--appimage-extract-and-run`,
points it at the staged executable/desktop/icon, and requests `--output=appimage`.
DSDA's script does not directly pin or invoke a named appimagetool version/runtime.
[.github/workflows/continuous_integration.yml:62–69](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/.github/workflows/continuous_integration.yml#L62-L69);
[.github/workflows/continuous_integration.yml:108–134](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/.github/workflows/continuous_integration.yml#L108-L134);
[.github/workflows/continuous_integration.yml:195–216](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/.github/workflows/continuous_integration.yml#L195-L216);
[prboom2/packaging/LinuxGenerator.cmake:1–39](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/packaging/LinuxGenerator.cmake#L1-L39).

**Proposed — artifact observations, not tagged runtime-source proof.** The official
[versioned release asset](https://github.com/dsda-org/dsda-doom/releases/download/v0.30.1/dsda-doom-0.30.1-linux-x86_64.appimage)
was downloaded into `.scratch/` without execution. Its size was 14,608,888 bytes,
and measured SHA-256 was
`b06adf3f0c7d4e82dc1870baf97e67e4dc4e663e37f69070ddc02fa3e97608b0`, matching the
[release API](https://api.github.com/repos/dsda-org/dsda-doom/releases/tags/v0.30.1)
on the research date. `file` reported x86-64 ELF static PIE; bytes 8–10 were
`41 49 02` (AppImage type-2 marker). The runtime strings advertise
`--appimage-extract`, `--appimage-extract-and-run`, FUSE and a fusermount helper.
This supports an extraction fallback design, but the payload was not executed or
extracted on this macOS host; it is not an end-to-end compatibility test.

**Confirmed (CI run 38980417407).** The official AppImage was downloaded and executed
on two Linux hosts by the spec 014 e2e workflow. On a plain Ubuntu runner the type-2
runtime mounted the AppImage directly (FUSE available), and the bundled DSDA
executable failed only because the runner lacked `libOpenGL.so.0`
(`error while loading shared libraries: libOpenGL.so.0: cannot open shared object
file`) — a missing GL driver, not a FUSE problem. Inside the pinned
`steamrt/sniper/sdk` container (Debian 11, glibc 2.31) there is no FUSE, DDL
extracted the AppImage as designed, and the extracted `squashfs-root/AppRun -v`
failed with `GLIBC_2.34 not found`, `GLIBC_2.33 not found` and `GLIBCXX_3.4.29 not
found`. The payload therefore needs a host with **glibc ≥ 2.34** and a GL driver;
Ubuntu 22.04 is a build environment, not the runtime baseline. On SteamOS a
non-Steam shortcut runs on the host by default, so this only bites when a Steam
"Compatibility tool" (a Steam Linux Runtime) is forced on DDL's shortcut. DDL now
classifies both stderr shapes into actionable errors
(`crates/ddl-core/src/engines/dsda/smoke.rs`).
[Run 38980417407](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/38980417407).

**Confirmed (reviewed v0.30.1 AppImage, extracted locally).** The second e2e run
([38982231350](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/38982231350))
failed next on `libjack.so.0`. Extracting the reviewed asset
([`dsda-doom-0.30.1-linux-x86_64.appimage`](https://github.com/dsda-org/dsda-doom/releases/download/v0.30.1/dsda-doom-0.30.1-linux-x86_64.appimage),
SHA-256 `b06adf3f0c7d4e82dc1870baf97e67e4dc4e663e37f69070ddc02fa3e97608b0`, matching
`crates/ddl-core/data/engines.toml`) and reading each bundled ELF's `DT_NEEDED`
shows `usr/bin/dsda-doom` links the **bundled** `libfluidsynth.so.3`, which links
`libjack.so.0`; `libjack.so.0` is not bundled, so it is a **hard, transitive
`DT_NEEDED`** of the engine (not an optional `dlopen`). The complete set of
non-bundled `DT_NEEDED` libraries across `usr/bin/dsda-doom` and every
`usr/lib/*.so` is: `libGL.so.1`, `libOpenGL.so.0`, `libX11.so.6`,
`libX11-xcb.so.1`, `libxcb.so.1`, `libasound.so.2`, `libdrm.so.2`, `libgbm.so.1`,
`libwayland-client.so.0`, `libgpg-error.so.0`, `libjack.so.0`, `libz.so.1`,
`libstdc++.so.6`, `libgcc_s.so.1`, plus glibc (`libc.so.6`, `libm.so.6`,
`libpthread.so.0`, `libmvec.so.1`) and the loader. The e2e host job now extracts the
AppImage and runs this `ldd` sweep once, recording the whole `not found` set.

**Confirmed (SteamOS 3.x package set).** SteamOS 3.8.0's published package manifest
lists `pipewire-jack 1:1.4.7-1.1` (and **no** `jack2`), and the SteamOS
`jupiter-rel` mirror's own `pipewire-jack` package installs `usr/lib/libjack.so.0` —
read from the SteamOS package itself, not only Arch's. The same manifest lists
`glibc 2.41`, `libglvnd 1.7.0`, `mesa` (which provides `libgbm.so.1`), `alsa-lib`,
`libdrm`, `libx11`, `libxcb`, `wayland`, `libgpg-error`, `gcc-libs` and `zlib`,
covering every library above. So SteamOS 3.x provides the engine's system libraries
in the default library path, and **no DDL-side mitigation is needed**: DDL does not
bundle system libraries, and doing so would be a separate owner decision.
[SteamOS 3.8.0 package manifest](https://steamdeck-images.steamos.cloud/steamdeck/20250903.1000/steamdeck-20250903.1000-3.8.0.manifest.pacman);
[SteamOS `pipewire-jack` package](https://steamdeck-packages.steamos.cloud/archlinux-mirror/jupiter-rel/os/x86_64/);
[SteamOS 3.7.20 package list](https://gist.github.com/hcartiaux/f5a901d048814dee7adec4830c9e8f5e);
[Arch `pipewire-jack` (`Provides: libjack.so=0-64`)](https://archlinux.org/packages/extra/x86_64/pipewire-jack/).

**Needs Deck Validation.** Actual `--appimage-extract`/`AppRun` operation, the
runtime's exact FUSE library/helper requirement (FUSE2 versus FUSE3), mount policy,
and SteamOS Gaming Mode launch remain unverified. A statically linked outer
runtime is not evidence that the bundled DSDA executable has no glibc dependency.
Ubuntu 22.04 is the build environment, **not a proven minimum glibc version** — the
CI evidence above now measures the requirement at ≥ 2.34 and a GL driver, and
SteamOS 3.8.0 ships glibc 2.41 and every system library the payload needs. The
remaining Deck check is simply to run the engine `-v` on the host (and confirm it
fails when a Steam Compatibility tool is forced). The exact payload ELF version
requirements and GL driver behavior must still be measured on Deck; source CI
dependencies establish GL involvement, not Deck driver support.

Implications for DDL:

- **Proposed:** Identify official assets by reviewed release metadata and digest; keep the measured digest above as research evidence, not an automatic trust-list change.
- **Proposed:** Design extraction/AppRun as a separately validated fallback; do not assume a host `libfuse.so.2` requirement from generic AppImage advice.
- **Proposed:** Preserve the complete AppDir in a fallback installation so bundled port WAD and libraries remain locatable.
- **Proposed:** Require a Linux asset inspection plus physical Deck tests before declaring a runtime/glibc/FUSE baseline in an engine-manager spec.
- **Confirmed:** The engine's non-bundled `DT_NEEDED` set is small and SteamOS 3.x provides all of it; DDL must not bundle system libraries (a redistribution decision, not a launcher one). A host missing one is a host problem, and DDL names the library so the user can install it.

## 8. License and unmodified upstream download

**Confirmed.** DSDA source headers grant GPL **version 2 or later**; the shipped
COPYING contains GPLv2, and the startup banner reports GPL v2.0. GPLv2 permits
running the program and provides conditions for copying/distributing binaries,
including corresponding source provisions. This is not evidence that every
bundled dependency has the identical license.
[prboom2/src/dsda/settings.c:1–15](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L1-L15); [prboom2/src/d_main.c:1761–1767](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1761-L1767);
[prboom2/COPYING:49–170](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/COPYING#L49-L170).

**Proposed.** DDL can fetch the unmodified official AppImage directly from upstream
into user storage and launch it as a separate program, retaining upstream license
and release/source links. This architecture requires no DSDA source modification.
If DDL later mirrors, bundles or redistributes the engine, explicitly review and
meet GPL binary/source and bundled-library notice obligations; “unmodified” alone
does not waive them. DDL's GPL-3.0-or-later policy does not relabel upstream files.

Implications for DDL:

- **Proposed:** Show upstream attribution, license, exact version and corresponding tagged source link in engine details.
- **Proposed:** Keep engine acquisition upstream-direct for the initial design; record redistribution obligations before introducing mirrors or bundled installers.

## Open questions and validation gates

- **Needs Deck Validation:** Execute the official AppImage `-v`, direct FUSE launch, `--appimage-extract`, and extracted `AppRun` on Linux/Deck; record statuses and stderr.
- **Needs Deck Validation:** Identify the official asset's actual runtime/toolchain version, FUSE helper/library needs and payload glibc symbol baseline. The tagged build downloads mutable linuxdeploy and cannot prove these.
- **Needs Deck Validation:** Verify GL/software rendering, audio, focus return and Steam child-process behavior on Deck.
- **Needs Deck Validation:** Confirm hybrid pad + gyro-as-mouse in ordinary play, first-device strict recording behavior, native controller identity/selection and hotplug.
- **Needs Deck Validation:** Calibrate the documented input defaults; verify rear controls and SDL button mappings with the intended manual Steam Input layout.
- **Needs Deck Validation:** Exercise missing config generation, menu persistence, timedemo writes, save isolation, screenshot output and pistol-start/save transitions with legal user-supplied IWAD fixtures.
- **Proposed:** Investigate whether second-controller selection's boolean collapse is intentional upstream before relying on value 2.
- **Proposed:** Before any redistribution/mirroring, inventory dependency licenses and corresponding-source delivery. This fact sheet does not audit every bundled binary.

## Three findings most likely to change OVERVIEW assumptions

1. **Proposed:** Replace pistol-start uncertainty with a concrete ordinary-play `-pistolstart` capability and an explicit recording incompatibility (§2); keep Deck acceptance pending.
2. **Proposed:** Expand isolation beyond `-config`: private saves/data, controlled screenshots/cwd and `-noautoload` are separate concerns; organized identity is filename-based (§1/§6).
3. **Proposed:** Strengthen `DsdaConfigManager` ownership rules: DSDA rewrites the whole file non-atomically, can write while still running after timedemo completion, and omitting complevel defaults to MBF21 (§1/§3).

## Verification and scope

**Proposed — research completion record.** Source clone/tag identity, remote tags,
release metadata, option consumers, config read/write call sites and all input
definitions were inspected. Documentation links and inventory completeness were
checked mechanically against the local tag. Only this document is a tracked
change; no new project dependency or code was introduced. Cargo checks are not
applicable to a source-research-only change. The open gates above remain required
for later executable specs and hardware acceptance; no OVERVIEW/decision/status
file was edited.

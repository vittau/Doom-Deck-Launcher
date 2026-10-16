# Widescreen status bars (NightFright's `doom_widehud`) — research

Desk research, 2026-10-16. Scope: what the upstream collection contains, whether DDL may
download it, whether DSDA-Doom v0.30.1 can use it, how it covers DDL's current catalog, and a
design proposal. No engine build, WAD execution, Linux runtime test or physical Deck test was
performed.

**Confirmed** = read directly in the pinned source/repo, quoted. **Proposed** = a DDL design
implication. **Needs Deck Validation** = runtime/hardware behaviour source inspection cannot
establish. Every repository and source link is pinned to a commit or tag; artifact observations
are dated evidence, not upstream guarantees.

**Bottom line:** the licence is **not granted anywhere** — that is a blocker for automatic
download (§2). DSDA-Doom v0.30.1 **does** support wide `STBAR` graphics out of the box with no
flag (§3). The collection covers only **1 of DDL's 4 catalog mapsets** plus the IWADs (§4).

## Method and evidence

- Research date **2026-10-16** (UTC); host macOS arm64. Tools: `git`, `curl`, `shasum`,
  `python3` (stdlib WAD-lump reader written for this task), `gh`/GitHub REST.
- A full clone of `NightFright2k19/doom_widehud` and a shallow clone of `dsda-org/dsda-doom`
  at tag `v0.30.1` were made in a private scratch dir; scratch bytes are never committed.
- Every file was hashed with SHA-256 after download (`shasum -a 256`); file sizes come from the
  GitHub tree API and were cross-checked against the downloaded bytes.
- WADs were only **parsed** (lump directory + patch headers); nothing was executed. No window
  was opened and no screen was captured.

---

## 1. What the repository contains

**Confirmed.** `NightFright2k19/doom_widehud` is a mirror of a long-running Doomworld/ZDoom
project ("Widescreen statusbars", first posted 2013-11-17). GitHub repo created
**2022-03-10**; the collection predates it. Default branch `main`; **no licence file**.

Pinned commit used below: **`78d962173334aecdb8bd974a1ba9bac95ab6cde4`** (2026-09-14, "Statusbar
update / Added: Crematomania"), the tip of `main` on the research date.
<https://github.com/NightFright2k19/doom_widehud/tree/78d962173334aecdb8bd974a1ba9bac95ab6cde4>

### 1.1 Structure and naming

| Item | Content at the pinned commit |
| --- | --- |
| `iwad/` | **11 files**, one status bar per IWAD/game (`chex_uwide.wad`, `doom_uwide.wad`, `doom_uwide_v2.wad`, `freedoom_uwide.wad`, `freedoom_uwide_v2.wad`, `hacx_uwide.wad`, `harm1_uwide.wad`, `id1_uwide.wad`, `id1_uwide_v2.wad`, `rekkr_uwide.wad`, `square1_uwide.wad`) |
| `pwad/` | **182 files**, one status bar per supported PWAD/episode |
| `readme.txt` | **28,630 B** — the only index: a free-text `Title ....... filename` list plus changelog. No machine-readable manifest. |
| Total | **193 WADs, 4,713,250 bytes** (working tree); packed repo ≈ 1.7 MB (GitHub `size` = 1686 KB) |

Naming has **no version markers** and is a hand-chosen abbreviation, not the PWAD filename:
`<tag>_wide.wad`, `<tag>_uwide.wad`, `<tag>_uwide_fix.wad` ("fix" = contains extra
ZDoom-compat lumps), and `jpcp_kana_uwide.wad` for the Japanese-font variant. Examples:
`av_uwide_fix.wad` = Alien Vendetta, `dbp38_uwide.wad` = DBP38, `cchest_wide.wad` = one file
for Community Chest 1–**4**, `btsx1/btsx2/btsx3_uwide.wad` = one per BTSX episode. The IWAD bars
follow the same pattern under `iwad/`.

**Confirmed — lump inventory (all 193 WADs parsed).** Every file is a `PWAD` whose payload is a
replacement status-bar patch; in most it is the only lump (`STBAR`, or `SQSTBAR` in
`iwad/square1_uwide.wad`).
- **169** files are status-bar-only.
- **24** files carry additional lumps: `MAPINFO` (13), `DEHACKED` (8), music (`M_*`), `STARMS`
  (5), `BOSSBACK`, `CWILVnn`, `INTRPICn`, and `STTNUM*/WINUM*` (Japanese font). These are the
  readme's `*` "Contains additional MAPINFO/DEHACKED for ZDoom" files; a status bar is *not*
  all they change.
- **2** files (`iwad/square1_uwide.wad`, `pwad/woolball_uwide.wad`) are the readme's `**`
  "Statusbar in PNG format (ZDoom/GZDoom only)"; parsed as patches their dimensions are garbage
  (20617×18254), i.e. raw PNG bytes — **not usable by DSDA-Doom**.

The wide bars are ordinary patches: e.g. `iwad/doom_uwide.wad` → `STBAR` 576×32;
`pwad/eviternity_wide.wad` → `STBAR` 486×32 (the standard is 320×32).

### 1.2 Release and tag practice

**Confirmed.** Tags are lightweight (commit objects), named `vYYMM`; GitHub Releases attach one
zip each.

| Tag | Commit | Date | Release asset |
| --- | --- | --- | --- |
| `v2509` | `be17aa95d899657aa43dd0e862747547c41de6bb` | 2025-09-06 | `doom_wide2509.zip` (1,134,324 B, SHA-256 `ee0aeaddfb8191fcab75ebf80a74a5a70118ba61fc61e99062c60d42292f23c7`) |
| `v2409` | `33c344dd2421b55c5befd729f6a2c4e367ebc2fd` | 2024-09-10 | `doom_wide2409.zip` (999,928 B) |
| `v2407` | `49056ae0b6c959094c7ef06d5503dda582e89017` | 2024-07-31 | `doom_wide2407.zip` (940,705 B) |
| `v2401` | `6a0a55b3110613a517c3fc95e3e4dc4d5d0fcd26` | 2024-01-23 | `doom_wide2401.zip` (844,553 B) |
| `v2312` | `ebd8d28dbc0f1f8297470d17d06e6ad66cd24f58` | 2023-12-12 | `doom_wide2312.zip` (782,697 B) |
| `v2206`/`v2205`/`v2204` | … | 2022 | `doom_wide2206/2205/2204.zip` |
| `v1808`, `r2203` | `884a9c2`, `fa44df4` | 2022-03-10 | historical tags on the first commits (older projects) |

**Confirmed — churn.** 155 commits since 2022-03-10: 44 (2022), 10 (2023), 73 (2024), 26 (2025),
2 (2026 through 2026-09-14). Most commit subjects are "Statusbar update". The WAD count grows per
release: 82 (`r2203`) → 113 (`v2206`) → 124 (`v2312`) → 135 (`v2401`) → 160 (`v2409`) → 181
(`v2509`) → 193 (unreleased HEAD, 6 commits after `v2509`). **The latest tag (`v2509`) is about a
year old and the working tree already contains 12 more bars** ("Version XXXX" changelog entry is
not tagged). So a pinned-tag design pins a slightly stale but stable snapshot; a pinned-HEAD
design pins current content but must be re-pinned often (D-019: catalog changes ship with a DDL
release anyway).

---

## 2. Licence and permission — **BLOCKER**

**Confirmed — the repository grants no licence and contains no permission text.**
- GitHub reports **no licence** for the repository (`license: null`; no license badge).
- The complete tree at the pinned commit is `iwad/`, `pwad/`, `readme.txt`. There is **no
  `LICENSE`, `COPYING`, `README.md` or credits file**. Verified against the tree API and a full
  clone (`git log --all --name-only | grep -i licen` → nothing).
- `readme.txt` was read in full and grepped for `permission`, `license`, `licence`, `copyright`,
  `redistribut` — **zero matches**. The only credit line is "Additional Credits to: ragu
  (ultrawide versions)" and the contributor lists. The `v2509` release zip contains the same
  readme document (its `v2509` revision) and no other terms file.

**Confirmed — the collection is derivative art by construction.** The readme lists
"As for the status bars: Graphics: Yes" and `Base: Original graphics from IWADs/PWADs`, i.e. the
bars are edits/expansions of id Software IWAD art and of each mapset author's own status bar.
The Doomworld thread documents the method explicitly, e.g. Eviternity's bar was "modified from
original wad gfx" and Antaresian Reliquary's was "made by me based on original wad graphic".

**Confirmed — the only author statement on reuse is permissive but not a licence.** In the
project's ZDoom forum thread NightFright wrote (2013-11-26):

> For now, the point of this project is only a cosmetic one, i.e. make the statusbars fill the
> full width of the screen. However, feel free to use these graphics to enhance them if you
> like.

Source: <https://forum.zdoom.org/viewtopic.php?t=37960> (post `p=725457`; archived
<https://web.archive.org/web/20241009102706/https://forum.zdoom.org/viewtopic.php?t=37960>).
This greets people borrowing the graphics, not redistribution of the collection as a package,
and it is not a copyright licence.

**Confirmed — corroborating signals.**
- An older independent NightFright release, "Widescreen HUDs 1.7a"
  (`zdoom_wide17a.zip`, 2014) is categorised **Licence: Proprietary** on ModDB
  (<https://www.moddb.com/games/doom/addons/widescreen-huds-17a-by-nightfright>).
- The author *does* grant reuse when they intend to: their other repo `doom_sbardef` says
  "**If you want to use any (or all) of these HUDs in a mod of yours, feel free to do so.
  However, please be fair and credit me for my work.**"
  (<https://github.com/NightFright2k19/doom_sbardef>). No such sentence exists in `doom_widehud`,
  which is evidence the omission is meaningful, not merely forgotten.
- id Software's own first-party widescreen assets are released under a **Limited License** that
  forbids distribution "in any other fashion" than layered into a WAD and forbids commercial
  use (<https://bethesda.net/en/game/doom-widescreen-mods>). Relevant context for the IWAD bars,
  which are derivatives of the same id art (the `doom_uwide.wad` lineage traces to MG_Man's
  community Doom HUD, per the forum thread).

### 2.1 Verdict

**Not confirmed as permitted — treat as a blocker.** NightFright's collection has no licence,
and most files are derivatives of IWAD art (id Software) and of PWAD authors' art. DDL's rules
require free content to be fetched from an official source "with its bundled permission text"
(D-014); there is no such text here, and the collection itself is the sole publisher of these
derivatives. DDL must **not** download-and-apply them automatically until it has, in writing:

1. permission from NightFright to redistribute/automate the collection (or at least the files
   DDL ships), and
2. a defensible position on the mapset authors' and id Software's rights in the derived bars.

This is exactly the case D-010 anticipated: "optional images with stated permission". **Proposed
fallbacks that avoid DDL distributing the files:** (a) make the wide bar a `user_supplied`
component (D-014) — the user downloads the collection themselves and DDL identifies the file by
SHA-256 and loads it last; or (b) link to the collection and give instructions only. Both are
compatible with the design in §5 and need no licence.

---

## 3. DSDA-Doom v0.30.1 support for wide status bars

Pinned tag **`v0.30.1` = `8ed581ad5c868b4dba1c39eb21d29e4ddbed9fad`** (same as
`research/dsda-engine.md`). All links below are to that tag.

**Confirmed — a wide `STBAR` is drawn automatically; there is no flag, cvar or file-name
convention.** DSDA loads the patch named `STBAR` and sizes the status bar from the patch itself:

- `R_SetPatchNum(&stbarbg, "STBAR")` — [st_stuff.c:978](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/st_stuff.c#L978)
- `ST_SetScaledWidth()` reads `stbarbg.width`, scales it by `render_stretch_hud`, then **caps it
  to the screen width and centres it**: `ST_SCALED_OFFSETX = (SCREENWIDTH - ST_SCALED_WIDTH)/2`
  — [st_stuff.c:398–426](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/st_stuff.c#L398-L426) (cap at L422–423, offset at L425)
- The bar is blitted and the side areas are copied/filled:
  [st_stuff.c:428–453](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/st_stuff.c#L428-L453)
- The leftover horizontal areas are painted with a colour derived from the bar's own palette
  (`sts_solid_bg_color`) or with the `grnrock`/`brdr_b` texture:
  [r_draw.c:502–540](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/r_draw.c#L502-L540),
  [r_draw.c:558–590](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/r_draw.c#L558-L590)
- `ST_SetScaledWidth()` is also re-run on every resolution change:
  [v_video.c:1440–1441](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/v_video.c#L1440-L1441)

**Confirmed — config keys involved (none required, all defaults fine).**
`render_stretch_hud` (file key, default `patch_stretch_not_adjusted` = 0; values
`patch_stretch_not_adjusted`/`patch_stretch_doom_format`/`patch_stretch_fit_to_width`,
[configuration.c:1254–1258](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c#L1254-L1258),
enum [stretch.h:41–60](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/stretch.h#L41-L60))
and `sts_solid_bg_color` (bool, default 0). `aspect_ratio_correction` and `render_aspect`
("auto/16:9/16:10/4:3/5:4") govern how logical pixels map to the panel and therefore how much of
a wide bar is visible. DDL need not change any of them to load a wide bar.

**Confirmed — load order: the widehud must be loaded *after* the mapset.** DSDA builds the lump
name index so that "the last lump of a given name appears first" in the lookup chain
([w_wad.c:398–411](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/w_wad.c#L398-L411)),
i.e. the later `-file` entry wins for a duplicate `STBAR`. This matches the readme's
"Load these files AFTER the PWAD they are meant to be used with". (The readme's Description also
says "with the lowest priority", which reads as the opposite; source evidence and the readme's
own point 1 agree that **last = overriding**, and DDL should follow that.) So in DDL's single
ordered `-file` list (D-011) the wide bar is the final entry, after the mapset and any add-on.

**Confirmed — interplay with the launch contract.**
- `-noautoload` (which DDL always passes, D-011) skips *directory* autoload, not explicitly listed
  files ([d_main.c:2059–2070](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L2059-L2070)).
  DDL must therefore name the wide bar explicitly; an autoload-folder layout (as dmslr's edition
  uses) will not work under DDL's `-noautoload` contract.
- **No complevel gate.** The wide-bar code is renderer-level and reads no compatibility variable
  (`grep complevel` in `st_stuff.c`/`r_draw.c` → nothing). Complevels 2/3/11 in DDL's catalog are
  unaffected. The status bar is not part of a demo, so it does not affect demo compatibility.
- **Caveat — a widehud that changes more than graphics.** 24 of the collection's files also carry
  `MAPINFO`/`DEHACKED`/music/intermission lumps (§1.1). Those can change map names, episodes and
  behaviour, so DDL must restrict itself to status-bar-only files (as the three files in §4 are)
  or verify each extra lump.
- **Caveat — HUD mode.** The classic `STBAR` is only drawn by the status-bar code path
  (`st_statusbaron`, `ST_refreshBackground`). A full-screen DSDA/fullscreen HUD replaces it, so a
  wide bar has no visible effect in those HUD modes. **Needs Deck Validation.**

**Confirmed — the `**` PNG files are unusable.** `square1_uwide.wad`/`woolball_uwide.wad` are
PNG-format bars for ZDoom/GZDoom only (absurd patch dimensions when parsed), and the readme says
so; DSDA should not be offered them.

**Needs Deck Validation — appearance on the Deck panel.** The Steam Deck screen is **1280×800
(16:10)** while many of the collection's bars are drawn for 16:9 or up to 21:9 (576 px wide;
ragu's "ultrawide" variants). DSDA caps `ST_SCALED_WIDTH` at `SCREENWIDTH` and centres it
(`st_stuff.c:422–425`), so how much of the extra art is actually visible depends on
`render_aspect`/`aspect_ratio_correction`/`render_stretch_hud` and the panel; the default
square-pixel 16:10 mapping the Deck uses must be checked on hardware before promising a visible
improvement. Source inspection cannot settle this.

---

## 4. Coverage of DDL's current catalog

**Confirmed.** DDL's catalog contains four recipes: `auger-zenith.toml`, `sigil.toml`,
`sigil-ii.toml`, `eviternity.toml` (`catalog/`). Searched the pinned tree **and the whole git
history** for `sigil`, `dbp37`, `augzen`, `auger` → **no such file ever existed**.

| DDL catalog item | Matching widehud file | Path | Size (B) | SHA-256 (pinned commit) |
| --- | --- | --- | --- | --- |
| **Eviternity** | **Yes** | `pwad/eviternity_wide.wad` | 19,962 | `f0f4222968c02ca4d666ef4db4d59cb102657ba67ce092a123ea9779330529f8` |
| **IWAD: Doom / Doom II** (`doom`, `doom2`) | **Yes** (one bar serves both) | `iwad/doom_uwide.wad` | 23,652 | `e1d29d0ed7db6e7adbac97ffd34018cb35c26d8fb607ba7d57f7a1155f46d0c5` |
| ″ (variant "with cracks") | Yes | `iwad/doom_uwide_v2.wad` | 23,652 | `eb364dbbfaeb0542ab052d095a857134a89d29438b351578bc36da80e3cec656` |
| **SIGIL** (Ultimate Doom, E5) | **No** | — | — | — |
| **SIGIL II** (E6) | **No** | — | — | — |
| **AUGER;ZENITH** (= DBP37) | **No** | — | — | — |

Pinned base URL (raw, immutable per commit):
`https://raw.githubusercontent.com/NightFright2k19/doom_widehud/78d962173334aecdb8bd974a1ba9bac95ab6cde4/<path>`

**Confirmed — the three catalog-relevant files are byte-identical at HEAD and at `v2509`**, so
pinning to the released tag is safe for them:

| Path | SHA-256 at `v2509` (`be17aa9`) |
| --- | --- |
| `pwad/eviternity_wide.wad` | `f0f4222968c02ca4d666ef4db4d59cb102657ba67ce092a123ea9779330529f8` |
| `iwad/doom_uwide.wad` | `e1d29d0ed7db6e7adbac97ffd34018cb35c26d8fb607ba7d57f7a1155f46d0c5` |
| `iwad/doom_uwide_v2.wad` | `eb364dbbfaeb0542ab052d095a857134a89d29438b351578bc36da80e3cec656` |

`readme.txt` differs between the two (`0ca8f55b…` at HEAD vs `c51fa12d…` at `v2509`).

**Findings for the owner's "detect the PWAD and offer the matching bar" idea:**

1. **Coverage is thin.** Only Eviternity of the four mapsets has a bar; SIGIL, SIGIL II and
   AUGER;ZENITH do not (DBP37 is conspicuously skipped — the DBP series in the collection runs
   DBP09…DBP73 but not 37). The feature would visibly do nothing for three of four cards today.
   (The "Cramunhão: A Sigil Tribute" file is a tribute PWAD, **not** SIGIL.)
2. **No hash mapping exists upstream.** The collection indexes by free text in `readme.txt`;
   filenames are abbreviations and carry no version. To match a *PWAD hash* to a *file*, DDL
   would have to build and maintain that table itself, keyed to each mapset version — a
   maintenance burden D-010/D-015 deliberately avoid.
3. **The Eviternity file needs care.** DDL's recipe records Eviternity as **CC BY-NC-ND 4.0**
   (`research/reference-eviternity.md`), which forbids derivatives and commercial use, while the
   collection's Eviternity bar is credited to "others" and described in the Doomworld thread as a
   modification of the original art. Its exact provenance is **not verifiable from the
   repository** — an open question that compounds the §2 blocker.

---

## 5. Design proposal

Two shapes were considered against the owner's request ("detect the PWAD and already download
and apply the matching widescreen status bar").

### 5.1 Option B — automatic matching by PWAD hash: reject

Match an installed mapset's WAD hash against a DDL-maintained hash→widehud table.

- Upstream publishes no hash table and no version in filenames (§4, finding 2); DDL would own a
  hand-maintained matrix of *mapset version × widehud file* that the upstream owner never
  signals when it breaks.
- A wide bar is authored against one specific mapset version; DWAD hash matching cannot tell
  "the right bar for this release" from "a bar that happens to exist".
- Adds no safety (SHA-256 of the downloaded bar is needed anyway) and much false-positive risk.
- **Proposed: reject.** It also does nothing about the §2 blocker.

### 5.2 Option A — declared per recipe as an optional component: recommend

Make the wide bar an ordinary **optional component** in the recipe that needs it, exactly like
AUGER;ZENITH's blaster add-on (D-010, D-016). This satisfies D-010/D-015 out of the box:

- **Package:** a single direct file URL on an allowlisted host, exact `size` + `sha256`,
  `kind = "wad"`, one member (`role = "graphics"`).
  `https://raw.githubusercontent.com/NightFright2k19/doom_widehud/<pinned-commit>/pwad/eviternity_wide.wad`
  (commit-pinned raw URL; D-015 requires direct file URLs and a pinned commit, which this is).
- **Component:** after `main` (`position` 2), `optional = true`, and shown like the existing
  add-on toggle. **No schema change is required** — packages, members, component roles, positions
  and `default` all already exist (spec 002 R4–R6).
- **Allowlist:** add `raw.githubusercontent.com` to `catalog/hosts.toml` `downloads` (D-015
  allowlisted host). The GitHub release zip is an alternative single artifact
  (`doom_wide2509.zip`, 1,134,324 B, SHA-256 `ee0aeadd…`) but it bundles all 181 files and is not
  commit-pinned — prefer per-file raw URLs.
- **IWAD bars:** `iwad/doom_uwide.wad` is not tied to one recipe; it would be duplicated across
  every Doom/Doom II recipe. Either (a) accept that duplication in each recipe, or (b) add a small
  global/per-base-game "cosmetic add-on" mechanism (a schema change — a separate spec). Recommend
  (a) for the first iteration to avoid schema churn, and revisit.

### 5.3 UI

**Proposed.** A "**Widescreen status bar**" toggle on the mapset detail page, rendered as a
component toggle (not as a locked required component), with the resulting load order visible
before launch (D-010 requires this). Default **on** is the owner's evident intent (it is purely
cosmetic and low-risk), but two caveats argue for making the default an explicit decision:
the licence blocker (§2) and the fact that the Deck is 16:10 while the art targets 16:9–21:9
(§3). Recommend: **ship it off by default until (a) permission is granted and (b) the Deck look
is validated**, then default on per-mapset once validated. The toggle must be a no-op when the
recipe has no bar (SIGIL, SIGIL II, AUGER;ZENITH today).

### 5.4 What the recipe schema needs

**Proposed: nothing new** for Option A — `[[packages]]` with `kind = "wad"` and one member, plus
one optional `[[components]]` entry with `role = "graphics"` and a `position` after `main`.
Two lint-level additions, not schema changes:

- extend `catalog/hosts.toml` with `raw.githubusercontent.com`;
- a lint rule (or recipe review rule) that a component whose role is `graphics` and whose label
  is a status bar must contain **only** status-bar lumps, so a `*`-type file that smuggles
  `DEHACKED`/`MAPINFO`/music into a "cosmetics" toggle is rejected.

### 5.5 Open questions for the owner

1. **Licence (blocking):** obtain written permission from NightFright to automate redistribution
   of the collection, and a position on the underlying IWAD/mapset-author rights in the derived
   bars. Until then, only the `user_supplied` (D-014) or link-only fallbacks are safe.
2. **Eviternity:** the file is a derivative of Eviternity's CC BY-NC-ND 4.0 art; confirm
   provenance and whether DDL may use it at all (it may not, as ND).
3. **Coverage:** SIGIL, SIGIL II and AUGER;ZENITH have no bar. Do we (a) ship without them,
   (b) commission/allow DDL-drawn original bars, or (c) drop the feature for the current catalog?
4. **Deck look:** does a 576 px (21:9) bar visibly help on the 1280×800 (16:10) panel under
   DSDA's default aspect mapping, or is it cropped to the standard view? Needs Deck Validation.
5. **IWAD bars:** per-recipe duplication vs a global cosmetic add-on (schema impact).
6. **Host allowlist:** OK to add `raw.githubusercontent.com`, or do we prefer GitHub release
   assets only?

---

## 6. Verification and scope

**Confirmed.** The repository tree, all 193 WADs' lump directories, tag/commit history, release
assets and the three catalog-relevant SHA-256 values were checked mechanically against a full
clone at the pinned commit; the DSDA-Doom claims were read in a shallow clone at
`v0.30.1` (`8ed581ad5c868b4dba1c39eb21d29e4ddbed9fad`, `git rev-parse HEAD`). Licence evidence is
quoted from the repository, the ZDoom forum thread, ModDB and Bethesda. Only this document is a
tracked change; no code, catalog or dependency was touched, so Cargo checks do not apply. The
open gates in §3/§5 remain required for any later spec, and no decision/status file was edited.

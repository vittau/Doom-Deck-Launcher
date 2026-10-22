# Reference evidence: UAC Ultra, Japanese Community Project, Stardate 20X6 (spec 087 D) and Ozonia (spec 088 D)

**Research date:** 2026-10-22 · **Host:** macOS arm64 · **Scope:** the five recipes this worker owns:
`catalog/uac-ultra.toml`, `catalog/japanese-community-project.toml`,
`catalog/stardate-20x6.toml` (spec 087), `catalog/ozonia.toml` (spec 088) and
`catalog/abandon.toml` (spec 089).
Nothing downloaded was extracted to disk, executed or loaded; WADs were parsed in memory.
No window was opened and no screen was captured.

Labels: **Confirmed** = read in the downloaded bytes or in a page quoted below; **Proposed** = a DDL
inference; **Needs Deck Validation** = cannot be settled on this host.

## Method

- `curl` with `--connect-timeout 10 --max-time 60` (text) or `300` (archives), a descriptive
  `DoomDeckLauncher/0.1 (catalog research)` User-Agent and no unbounded retries. Scratch bytes live
  in `.scratch/087d/` (git-ignored). `youfailit.net` timed out from this host and was not used;
  the coordinator directed this worker to `ftp.fu-berlin.de` and `mirrors.lug.mtu.edu`.
- Each archive was downloaded from **three mirrors** (`www.gamers.org`, `mirrors.lug.mtu.edu`,
  `ftp.fu-berlin.de`) and the three copies compared by byte count and SHA-256 (§5.1).
- ZIP central directories, member sizes/SHA-256 and WAD lump directories were read with Python's
  `zipfile`/`struct` in memory. No extraction, no execution, no IWAD download.
- Cacowards pages were read through the Wayback Machine with `id_` captures, because
  `doomworld.com` returns HTTP 403 to automated clients. The exact page read is quoted for each
  title; the catalog's canonical year URL (`https://www.doomworld.com/cacowards/<year>/`) is given
  alongside it, but those modern year pages were **not** fetched (HTTP 403, and the Wayback CDX
  index was temporarily unavailable during this session).
- **Date normalization.** Where a readme writes a numeric date ambiguous between day/month and
  month/day, the ZIP member timestamps decide it and the literal string is quoted anyway. This
  affects Stardate 20X6 (`06/02/2013` = 2013-06-02; WAD timestamp 2013-05-31 20:29:28, text
  2013-06-02 17:50:00) and
  Abandon (`05/07/2023` = 2023-05-07; `Abandon.txt` timestamp 2023-05-07). Where the archive was
  packaged after the readme's stated release (Ozonia), both dates are recorded.
- Verification commands and results are in `docs/reports/087-d.md`.

---

## 1. UAC Ultra (`uac-ultra`)

**Identity — Confirmed.** Archive `levels/doom2/Ports/s-u/uacultra.zip`, 2,309,100 B, SHA-256
`58ebd0fa2fc0ef26fbcd4908a5cb08dbccacc637613db7cb73cec682a863a5a5`. The ZIP has exactly two
members, both at the root, stored with the case shown:

| Member | Bytes | SHA-256 | keep |
| --- | --- | --- | --- |
| `uacultra.wad` | 7,473,347 | `db52da533223c74c01d8dd0eb2d81b39fc6a344249be3a7fe92a34a8a0926798` | no |
| `uacultra.txt` | 4,135 | `ee2372b5bc80393a9bb28a238d7b03f55ca248bd402c9a068bba2019a395ac8d` | yes |

**Release text — Confirmed, quoted.** The bundled `uacultra.txt` (UTF-8) says
`Title: UAC Ultra (v1.2)`, `Release date: 29th March 2010`, `Author: 40oz & Super Jamie`,
`Advanced engine needed: Boom-compatible. PrBoom-Plus recommended.`,
`An 11 (and a half) map levelset for Doom 2`, `New levels: 12`, `Map #: MAP01 - MAP12`,
`Dehacked/BEX Patch: Yes`, `Music: Yes`, `Other files required: No`.
So the shipped release is **v1.2**, not the v1.0/v1.1 archives the text also links
(`uacultra10.zip`/`uacultra11.zip` on superjamie.net). The "11½" phrasing and the 12 map slots are
both the author's; the recipe quotes the count honestly instead of picking one.

**WAD inspection — Confirmed.** `uacultra.wad` is a `PWAD` with **505 lumps**: twelve map markers
`MAP01`–`MAP12`, **14 `D_*` music lumps**, and one embedded `DEHACKED` lump. There is **no
`COMPLVL`, `UMAPINFO` or `MAPINFO`** lump. The patch and the music therefore load inside the WAD:
no separate `-deh` file and no soundtrack package exist. The archive contains no other WAD or patch.

**Complevel — Confirmed source.** There is no `COMPLVL` lump, so D-030's order applies at step 1
(author statement): the release text says "Boom-compatible. PrBoom-Plus recommended." → family
`boom`, DSDA-Doom alias `boom` (complevel 9).

**Permission — Confirmed, quoted.** "Authors MAY use the contents of this file as a base for
modification or reuse… You MAY distribute this file, provided you include this text file, with no
modifications… as long as you include this file intact." The text is kept unmodified as `credits`.

**Cacowards evidence — Confirmed (Wayback capture).**
- Page read: <https://web.archive.org/web/2026id_/https://www.doomworld.com/17years/best2.php>;
  `<title>` is "Doomworld -- The 17th Annual Cacowards".
- The edition index <https://web.archive.org/web/2026id_/https://www.doomworld.com/17years/> lists
  "2010 Cacowards … Page 2 UAC Ultra Unloved Valhalla Page 3 …", with "RUNNERS UP" under a separate
  heading → **Top Ten winner**, page 2, not a runner-up.
- Entry text starts "UAC Ultra - 40oz & Super Jamie … UAC Ultra is hands down the epic Doom murder
  fest of 2010"; it calls it "an 11.5 map set". The reviewer also writes that it "doesn't forgive
  player mistakes, especially on UV" — a difficulty comment, not a statement that the set targets
  experienced players, so no `challenging` tag.
- The catalog cites `https://www.doomworld.com/cacowards/2010/` as the canonical year URL.

**Other tags — Confirmed.** `classics`: released 2010-03-29, over five years before this review,
with the Cacowards entry as lasting recognition. Not `megawad`/`episode`: twelve maps is over nine
and under thirty.

**Screenshot — Confirmed.** `images/uacultra.png` (640×480 PNG) from the same Wayback capture:
<https://web.archive.org/web/2026id_/https://www.doomworld.com/17years/images/uacultra.png>,
converted with `scripts/lean-image.sh` to a 640×480, 52,534 B JPEG. Provenance and permission text
are in `crates/ddl/ui/assets/screenshots/uac-ultra/SOURCES.toml`.

**Not included.** The text mentions a separate `ultratex.wad` texture set with no URL in the
archive; it is neither downloaded nor declared.

---

## 2. Japanese Community Project (`japanese-community-project`)

**Identity — Confirmed.** Archive `levels/doom2/Ports/megawads/jpcp.zip`, 13,575,267 B, SHA-256
`2d9d54a4092252031cc038197d6bddc25cd692f4071eec76726cdca4bb70ce63`. Three root members:

| Member | Bytes | SHA-256 | keep |
| --- | --- | --- | --- |
| `JPCP.wad` | 45,520,073 | `b994fbbab8a78f7133fc792679b95571b30ba2491503e35179eb49d77c29c4dd` | no |
| `JPCP_HUD.wad` | 33,600 | `beff29573bb665b597ad9b0671e6f95b08bba45cdae57304fc4091444d7a2009` | no |
| `JPCP.txt` | 13,271 | `362a529fc38dc13ce6744472f3cbb1f41e2ee0cd8ed7bbb2975d15e9e1c5a349` | yes |

The case is mixed (`JPCP.txt`, `JPCP.wad`, `JPCP_HUD.wad`) and is declared exactly.

**Release text — Confirmed, quoted.** `Advanced engine needed: Limit Removing`;
`Tested with PrBoom+ with doom 2 compatibility (-complevel 2), ZDoom, GZDoom`;
`Title: Japanese Community Project`; `Release date: 16/07/24`; release log ends
`16/07/24 (Ver.2.3, Final version)`; `New levels: 32`; `Map #: MAP01-MAP32`;
`May Not Run With...: Vanilla Doom`; `Other files required: None`; `Dehacked/BEX Patch: Yes`;
`Music: Yes`. The map list names every slot Map01–Map32 with its author, and the credits list
thirteen level designers (Tatsurd-Cacocaco as project leader, burabojunior, Toooooasty, Masayan,
Namsan, Yullie, Guna, Masa, toMass, doom68k, Tyousen121, SkullDemon, Nanka Kurashiki), a musician
(Masayan), graphics artists and a DeHackEd editor.

**Encoding — Confirmed.** `JPCP.txt` is **not valid UTF-8**: byte `0x81` at offset 9,547 (a CP932
ellipsis in Nanka Kurashiki's comment). The recipe pins the exact bytes and keeps the file
unmodified; nothing in DDL needs to decode it as UTF-8 beyond display, which is outside this task.

**WAD inspection — Confirmed.** `JPCP.wad` is a `PWAD` with **2,135 lumps**: thirty-two map markers
`MAP01`–`MAP32`, **35 `D_*` music lumps**, one embedded `DEHACKED` lump (9,221 B, WhackEd4 header,
`Patch format = 6`, level-name/intermission strings) and one **ZDoom-format `MAPINFO`** (4,022 B,
`defaultmap`/`map MAP01 "…"`/`sky1`/`cluster`/`par`/`music`). No `COMPLVL` and no `UMAPINFO` lump.
The optional `JPCP_HUD.wad` has **23 lumps, all status-bar graphics**:
`STBAR000`, `STBAR`, `STARMS`, `STTNUM0`–`STTNUM9`, `WINUM0`–`WINUM9` — no maps, `DEHACKED`,
`MAPINFO` or music.

**Complevel — Confirmed source.** No `COMPLVL` lump; the release text declares a limit-removing
target and says it was tested in PrBoom+ "with doom 2 compatibility (-complevel 2)". D-030's
convention maps limit-removing to the `vanilla` alias, which resolves to 2 on Doom II — the same
number the authors tested.

**ZDoom `MAPINFO` — Confirmed inert.** DSDA-Doom removed ZDoom MAPINFO support in v0.30.0
(<https://github.com/dsda-org/dsda-doom/releases/tag/v0.30.0>, cited in
`docs/research/reference-valiant.md` §…); the mapset therefore plays as one continuous
MAP01–MAP32 run in DSDA-Doom v0.30.1. Its episode/par/music metadata is not applied.

**JPCP_HUD component — Proposed (needs review).** The readme says: "Japanese status HUD is
included. If you load JPCP_HUD.wad with this wad, status bar in 'Katakana' and 'Kanji' can be
used." The recipe declares it as an optional, **off-by-default** `graphics` component loaded after
`main` (position 2). Two details are recorded for the reviewer:
- The 23 lumps were enumerated above; none changes gameplay.
- The lint's cosmetic status-bar allowlist (`lint.rs` `STATUS_BAR_LUMP_PREFIXES`) covers `STBAR`,
  `STARMS`, `STT*` and friends but **not `WINUM*`**, so a component whose *label* contains
  "status bar" would be rejected by `--fetch` even though this file is cosmetic. The label is
  therefore "Japanese HUD graphics (Katakana/Kanji)" and the description states exactly what it
  replaces. If the allowlist gains `WINUM*` later, the label can say "status bar" again.

**Permission — Confirmed, quoted.** "Authors may NOT use the contents of this file as a base for
modification or reuse… You MAY distribute this file, provided you include this text file, with no
modifications… as long as you include this file intact."

**Cacowards evidence — Confirmed (Wayback capture).**
- Page read: <https://web.archive.org/web/2026id_/https://www.doomworld.com/23years/best3.php>;
  `<title>` is "Doomworld -- The 23rd Annual Cacowards"; the page header reads
  "Top Ten - Page 3"; the entry is "Japanese Community Project - Various".
- The 2016 edition is the 23rd Annual Cacowards → **Top Ten winner**, page 3, not a runner-up.
  (`docs/research/cacowards.md` line 296 records the same year and page; the entry text itself is
  the primary evidence used here.)
- The catalog cites `https://www.doomworld.com/cacowards/2016/` as the canonical year URL.

**Other tags — Confirmed.** `megawad`: 32 maps. `community-projects`: multi-author project with a
credits list of thirteen mappers plus musician/graphics/DeHackEd roles. `classics`: released
2016-07-24, over five years before this review, with the Cacowards entry as recognition.

**Screenshot — Confirmed.** `images/jcp.png` (1280×960) from the same capture:
<https://web.archive.org/web/2026id_/https://www.doomworld.com/23years/images/jcp.png>; resized to
1066×800, 102,763 B JPEG. Sources in `.../japanese-community-project/SOURCES.toml`.

---

## 3. Stardate 20X6 (`stardate-20x6`)

**Identity — Confirmed.** Archive `levels/doom2/Ports/s-u/stardate.zip`, 4,995,637 B, SHA-256
`4e2a73c60b1eab37bbb35a78706cb557020346f6532c293f45e1ec9c9ac3a0d4`. The ZIP has six entries: two
directory entries, `__MACOSX/`, `__MACOSX/stardate20x6/`, an AppleDouble
`__MACOSX/stardate20x6/._stardate20x6.txt` (226 B), and the two real members inside the folder:

| Member | Bytes | SHA-256 | keep |
| --- | --- | --- | --- |
| `stardate20x6/stardate20x6.wad` | 14,650,225 | `633d06668414de9fd20683b26924808e7b11f65a7e7b4546df68af691019b9ed` | no |
| `stardate20x6/stardate20x6.txt` | 7,262 | `d7ba39b2a625f4b15fe08676cd56cfe529eaea128497051a522ad09e641fd8a7` | yes |

The folder prefix is case-exact. The `__MACOSX` metadata is not declared and never loaded.

**Release text — Confirmed, quoted.** `Title: Stardate 20X6`; `Release date: 06/02/2013` — the
readme writes month/day, so the normalized date is **2013-06-02**: the ZIP's text member is dated
2013-06-02 17:50:00, the WAD member 2013-05-31 20:29:28 and the folder 2013-06-02 16:42:40. The
literal string stays quoted above. The coordinator's independent inspection of the same archive
reported the same two timestamps (2013-06-02 17:50:00 and 2013-05-31), which corroborates the
month/day reading;
`Author: Ribbiks`; `Advanced engine needed: Boom-Compatible (tested in prb+ cl9, gzdoom)`;
`Stardate 20X6 is a set of 8 boom-compatible levels`; `Map #: MAP01 - MAP07, bonus: MAP31`;
`Other: DEHACKED and MAPINFO lumps included.`; `Music: Yes`; `Dehacked/BEX Patch: Yes`.
The readme also says "The maps are very difficult, with UV hopefully presenting a worthy challenge
even for veteran doomers", and warns it needs "boom linedefs / voodoo doll shennanigans" and MIDI
playback.

**WAD inspection — Confirmed.** `stardate20x6.wad` is a `PWAD` with **638 lumps**: eight map
markers (`MAP01`–`MAP07`, `MAP31`), **11 `D_*` music lumps**, one embedded `DEHACKED` lump
(444 B, map-name and intermission strings) and one **ZDoom-format `MAPINFO`** (1,024 B). No
`COMPLVL` and no `UMAPINFO` lump. The patch and music are inside the WAD; no separate files exist.

**Complevel — Confirmed source.** The author states "tested in prb+ cl9" (Boom, complevel 9) →
DSDA-Doom alias `boom`.

**Permission — Confirmed, quoted.** "Authors MAY use the contents of this file as a base for
modification or reuse… You MAY distribute this file, provided you include the text file intact…
You may not charge for the contents of this file." The text credits GothicDM/Gothic2 textures and
Hexen/Plutonia/TNT resources; that disclaimer is kept in `credits`.

**Cacowards evidence — Confirmed (Wayback capture).**
- Page read: <https://web.archive.org/web/2026id_/https://www.doomworld.com/20years/best3.php>;
  `<title>` is "Doomworld -- The 20th Annual Cacowards"; the page navigation reads
  "Top Ten - Page 3 Forsaken Overlook Stardate 20X6 Pirate Doom Swim With the Whales"; the entry is
  "Stardate 20X6 - Ribbiks" → **Top Ten winner**, page 3.
- The catalog cites `https://www.doomworld.com/cacowards/2013/` as the canonical year URL.
- Stardate **20X7** is a separate release that won in 2017 (`sd20x7.zip`) and is **not** part of
  this recipe.

**Other tags — Confirmed.** `episode`: eight map slots, within the vocabulary's nine-or-fewer rule.
`classics`: released 2013-06-02, over five years old, with the Cacowards entry as recognition.
`challenging`: the author's own statement that the maps are "very difficult" and aimed at
"veteran doomers" on UV (quoted above).

**Screenshot — Confirmed.** `images/stardate.png` (1280×960) from the same capture:
<https://web.archive.org/web/2026id_/https://www.doomworld.com/20years/images/stardate.png>;
resized to 1066×800, 72,662 B JPEG. Sources in `.../stardate-20x6/SOURCES.toml`.

---

## 4. Ozonia (`ozonia`, spec 088)

**Identity — Confirmed.** Archive `levels/doom2/Ports/megawads/ozonia.zip`, 17,180,670 B, SHA-256
`f5bbc36ecf8d79652275a98a91672ee6ed6062132ac727069471a28c8f007228`. Two root members:

| Member | Bytes | SHA-256 | keep |
| --- | --- | --- | --- |
| `ozonia.wad` | 55,559,897 | `18c8ae66abafba79ed03d4eaa382d41af8c1d862a6906cba27d071fbe4508fb8` | no |
| `ozonia.txt` | 6,595 | `fb38dcf0da17da4b25fde66a03413a6454a583b650cb3221a547237946590275` | yes |

**Release text — Confirmed, quoted.** `Title: Ozonia`; `Release date: 23/12/2021` (unambiguous:
23 cannot be a month, so 2021-12-23). The archive itself was packaged later — the ZIP's WAD member is
dated 2022-10-30 and its text 2022-11-04 — so the recipe keeps the author's stated release date and
pins the archived bytes by size and SHA-256;
`Author: Deadwing`; `Advanced engine needed: Prboom`;
"Ozonia is the third mapset of the Moonblood series, containing 32 carefully designed short maps
with compatibility level 9."; `Game: DOOM2`; `Map #: Map01 to Map 32`; `Dehacked/BEX Patch: Yes`;
`Music: Yes`; two new enemies (Hierophant replaces Romero's head, Hellion replaces Commander Keen).
It lists three episode groups (E1 maps 1–11, E2 maps 12–20 plus 31–32, E3 maps 21–30). The level
list and the credits block disagree about one slot: the list shows "31 - Rooftops" and
"32 - Penitência (map by @antares031)", while the credits say "@antares031 (map 31 - Penitência)".
The recipe credits the guest mappers without asserting slot authorship and records the discrepancy
as a limitation.

**WAD inspection — Confirmed.** `ozonia.wad` is a `PWAD` with **1,532 lumps**: thirty-two map
markers `MAP01`–`MAP32`, **35 `D_*` music lumps**, one embedded `DEHACKED` lump (11,975 B,
WhackEd4 header, new-enemy definitions), one **`UMAPINFO`** (2,707 B, episode definitions
`episode = clear` / `EP1TTL` …) and one **ZDoom-format `MAPINFO`** (5,612 B, copied from Ancient
Aliens). No `COMPLVL` lump. DSDA-Doom v0.30.1 reads the UMAPINFO (episode menu) and ignores the
ZDoom MAPINFO (support removed in v0.30.0).

**Complevel — Confirmed source.** "compatibility level 9" plus "Advanced engine needed: Prboom" →
DSDA-Doom alias `boom` (9).

**Permission — Confirmed, quoted.** "Authors MAY use the contents of this file as a base for
modification or reuse… You MAY distribute this file, provided you include this text file, with no
modifications…" The text credits textures, sprites, music and skybox authors.

**Cacowards evidence — Confirmed (Wayback capture).**
- Page read: <https://web.archive.org/web/2026id_/https://www.doomworld.com/cacowards/2022/best1/>
  (`<title>` is "Top Ten - Page 1 - Doomworld"); entry header
  "Ozonia - @Deadwing Doom 2, Boom-compatible, 32 maps" → **Top Ten winner**, page 1.
- This URL is the canonical modern Cacowards path and is cited verbatim in the recipe.

**Other tags — Confirmed.** `megawad`: 32 maps. Not `classics`: released 2021-12-23, four years and
ten months before this review, short of the five-year rule. Not `challenging`: the author compares
the difficulty to Eviternity and describes an intentional learning curve in the early maps.

**Screenshot — Confirmed.** The live image host returns 403, so the image was fetched from its
Wayback capture:
<https://web.archive.org/web/2026id_/https://static.doomworld.com/pages_media/13_ozonia.png>
(1280×960). Resized to 1066×800, 135,029 B JPEG. Sources in `.../ozonia/SOURCES.toml`.

---

## 5. Cross-checks and shared evidence

### 5.1 Byte-identical mirrors — Confirmed

Every archive was downloaded from `www.gamers.org`, `mirrors.lug.mtu.edu` and
`ftp.fu-berlin.de`; all three copies had the same byte count and SHA-256:

| Recipe | Bytes | SHA-256 | gamers.org | mtu | fu-berlin |
| --- | --- | --- | --- | --- | --- |
| `uac-ultra` | 2,309,100 | `58ebd0fa…a5a5` | identical | identical | identical |
| `japanese-community-project` | 13,575,267 | `2d9d54a4…ce63` | identical | identical | identical |
| `stardate-20x6` | 4,995,637 | `4e2a73c6…a0d4` | identical | identical | identical |
| `ozonia` | 17,180,670 | `f5bbc36e…7228` | identical | identical | identical |
| `abandon` | 46,212,925 | `9ef49a28…bd27` | identical | identical | identical |

The recipes list exactly these three URLs, primary first. `youfailit.net` is allowlisted but was not
reachable from this host, so it is not listed.

### 5.2 Lint and tests — Confirmed (see report for the exact output)

- `ddl catalog lint` (offline) passes for the five recipes, including screenshot bytes,
`SOURCES.toml` provenance and tag citations.
- `ddl catalog lint --fetch` passes: all packages download from the primary mirror and every named
member matches its pinned size and SHA-256.
- `crates/ddl-core/tests/catalog_087_d.rs` pins package/member identities, base game and complevel
aliases, the one-entry `-file` load order (plus the optional JPCP HUD second), the tag sets and the
Cacowards citations.

### 5.3 Screenshots — Confirmed conversions

| Recipe | Original | Output | Bytes |
| --- | --- | --- | --- |
| `uac-ultra` | 640×480 PNG | 640×480 JPEG | 52,534 |
| `japanese-community-project` | 1280×960 PNG | 1066×800 JPEG | 102,763 |
| `stardate-20x6` | 1280×960 PNG | 1066×800 JPEG | 72,662 |
| `ozonia` | 1280×960 PNG | 1066×800 JPEG | 135,029 |
| `abandon` | 1600×900 JPEG | 1280×720 JPEG | 178,318 |

All were produced by `scripts/lean-image.sh` (JPEG quality 60), inside the 640×360–1280×800 and
192 KiB bounds. Abandon's source is 16:9; the other four are 4:3 and were not manually cropped, so the committed
images keep the full frame the Cacowards page published. **Needs Deck Validation:** how the card and
detail crops look on the 1280×800 panel.

### 5.4 Widescreen status bars — not added (Proposed)

None of the five recipes declares a `wide-status-bar` component. `UAC Ultra`, `JPCP` and `Abandon` replace
`STBAR`/`STARMS` themselves, so spec 017's no-replaced-lump guard would rule out an IWAD bar; this
worker did not evaluate NightFright's collection for a matching mapset bar, because spec 087/088
does not ask for one and no ownership was assigned for that change. Stardate 20X6 and Ozonia could
take the IWAD bar under the existing policy, but adding it here would extend scope; recorded as an
open follow-up rather than implemented.

### 5.5 Open checks for the Deck

- IWAD recognition and completion for all five; `boom`/`vanilla` aliases as launched by DDL.
- That the embedded DEHACKED patches apply (new monster in UAC Ultra; two new enemies in Ozonia;
map-name strings in all five) and that ZDoom `MAPINFO` inertness does not break map progression.
- Ozonia's UMAPINFO episode menu, and JPCP's continuous MAP01–MAP32 run with its secret exits.
- Audio: 14/35/11/35/21 embedded `D_*` lumps, including the JPCP music list quoted in its text.
- Abandon performance: no frame rate, memory, battery or thermal figure was measured; only the
  inspected map sizes and thing counts are recorded.
- Optional JPCP_HUD Katakana/Kanji status bar under DSDA-Doom v0.30.1.

---

## 6. Abandon (`abandon`, spec 089)

**Identity — Confirmed.** Archive `levels/doom2/Ports/megawads/abandon.zip`, 46,212,925 B, SHA-256
`9ef49a28f7852b074470abf21b02de895f2cf9a2a6eb8300c65b12c833dfbd27`. Two root members, case-exact:

| Member | Bytes | SHA-256 | keep |
| --- | --- | --- | --- |
| `Abandon.wad` | 177,444,166 | `ce48df1db341d8e3ef02250a0d3852032a8b15813c0734495d70ee489e534bea` | no |
| `Abandon.txt` | 4,997 | `b2e9c6423db89c132ccd4f9d95074ccac69fa035a76b1e2e6ed211f377e410a4` | yes |

The lead page <https://doomwiki.org/wiki/Abandon> was **not** read: DoomWiki returns HTTP 403 to
this project's clients. Identity and scope come from the author's own bundled text and the archive
bytes, as spec 089 requires.

**Release text — Confirmed, quoted.** `Title: Abandon`; `Release date: 05/07/2023` (month/day: **2023-05-07**; the ZIP's
`Abandon.txt` is dated 2023-05-07 and its WAD 2023-02-10);
`Authors: Bemused, Benjogami, Scotty`; `Description: Grandiose, oppressive slaughter maps.`;
`Advanced engine needed: Boom`; `New levels: 17`; `Map #: MAP01-MAP17`;
`May Not Run With: zDaemon as several maps use extended nodes`;
`Tested With: PRBoom+ 2.4.1.5, DSDA-Doom 0.x.x`; `Dehacked/BEX Patch: Yes`; `Music: Yes`.
So the pinned archive is a **2023-05-07 update**; the Cacoward discussed below was awarded for the
project's **2020** release, whose bytes are different and are not pinned here.

**WAD inspection — Confirmed.** `Abandon.wad` is a `PWAD` with **767 lumps**: eighteen map markers
`MAP01`–`MAP18`, **21 `D_*` music lumps**, one embedded `DEHACKED` lump (1,440 B, WhackEd4 header,
level names and intermission text) and its own status-bar graphics (`STBAR`, `STARMS`, `STTNUM*`,
`STGNUM*`, …). No `COMPLVL`, `UMAPINFO` or `MAPINFO` lump. The authors list seventeen levels; the
eighteenth marker is an ending map: `MAP18` has **4 things** in `THINGS` and the patch string
`HUSTR_18 = The End`. So the released level count is **17**, not the eighteen markers the WAD
happens to carry.

**Map scale — Confirmed by directory inspection (not a performance test).** The map lumps
(`THINGS` … `BLOCKMAP`) total **167,702,999 bytes** of the 177,444,166-byte file. The largest slots:

| Map | Map-lump bytes | Things |
| --- | --- | --- |
| MAP12 | 29,091,075 | 3,979 |
| MAP05 | 22,354,150 | 1,895 |
| MAP13 | 21,668,289 | 6,659 |
| MAP06 | 19,723,328 | 5,368 |
| MAP11 | 14,656,542 | 7,715 |
| MAP10 | 4,167,352 | 10,030 |
| MAP09 | 7,660,331 | 8,672 |

Across all eighteen slots the WAD places **74,510 things**. These are byte and record counts from
the WAD directory, nothing more: no frame rate, frame time, memory, battery or thermal figure was
measured, and the recipe says so.

**Complevel — Confirmed source.** No `COMPLVL` lump; the author declares "Advanced engine needed:
Boom" and lists DSDA-Doom among tested ports → family `boom`, alias `boom` (9). The extended-node
statement is recorded as the author's own warning, not as a DDL compatibility test.

**Permission — Confirmed, quoted.** "Authors may NOT use the contents of this file as a base for
modification or reuse… You MAY distribute this file, provided you include this text file, with no
modifications… as long as you include this file intact."

**Cacowards evidence — Confirmed (Wayback capture).**
- Page read: <https://web.archive.org/web/20201210111058id_/https://www.doomworld.com/cacowards/2020/best1/>;
  `<title>` is "2020 Cacowards - Top Ten Page 1 - Doomworld"; the entry header begins
  "Abandon - …" with the three authors credited → **Top Ten winner**, page 1.
- The entry describes "sombre vistas", "functionally infinite vertical variation" and environs
  that are "legitimately perilous, rife with baffling geometry and shattered terrain" — a reviewed
  source for the large/complex-map note and for the `challenging` tag, alongside the authors'
  "Grandiose, oppressive slaughter maps".
- The catalog cites `https://www.doomworld.com/cacowards/2020/best1/` verbatim (this modern URL was
  the page captured and is the URL the edition itself uses).

**Other tags — Confirmed.** `challenging` as quoted above. Not `megawad` (17 released maps) and not
`episode` (over nine): the project's own `New levels: 17` and `MAP01-MAP17` define the count, even
though "Abandon" is sometimes described as a megawad. Not `classics`: the pinned archive text is
dated 2023-05-07, under five years before this review, even though the project won in 2020.

**Performance — Needs Deck Validation.** Candidate risk factors are inspected above (177 MB WAD,
167.7 MB of map data, a 29.1 MB single map, up to 10,030 things) plus the author's extended-node
warning. Whether those maps hold a stable frame rate on the Steam Deck, and what they cost in
memory or battery, is unmeasured here and must be tested on hardware.

**Screenshot — Confirmed.** The 2020 Cacowards page embeds a 1600×900 JPEG at
`https://static.doomworld.com/pages_media/9_abandon.jpg?_cb=1607467020`; the live host returns 403,
so it was downloaded from its Wayback capture:
<https://web.archive.org/web/2026id_/https://static.doomworld.com/pages_media/9_abandon.jpg?_cb=1607467020>.
`scripts/lean-image.sh` produced a 1280×720, 178,318 B JPEG. Sources in `.../abandon/SOURCES.toml`.

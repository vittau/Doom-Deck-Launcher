# Reference evidence — worker B (specs 087, 088 and 089)

Titles: **Doom the Way id Did**, **No End in Sight**, **Doom II the Way id Did**,
**Deathless**, **Sunder** and **REKKR** (free PWAD edition). Everything below was gathered on 2026-10-22 by downloading
the releases, reading their bytes as data and comparing them with upstream source and pages.
Nothing from an archive was executed, and no IWAD was downloaded. Labels: **Confirmed** =
measured from the inspected bytes or read in the quoted source; **Proposed** = a
convention-based choice; **Needs Deck Validation** = not played on the target hardware.

## 0. Method and retrieval

- Downloads were made with `curl -sSL --connect-timeout 10 --max-time 300` into
  `.scratch/087-b/` (git-ignored research scratch), one file at a time.
- ZIP inventories came from `unzip -l`; member sizes and SHA-256 from `shasum -a 256`
  over the extracted files. WAD directories were parsed read-only with a small local
  Python script (PWAD header, 16-byte lump entries); DEH/BEX lumps were decoded as text.
- Mirror bytes: each `/idgames` package was downloaded from `www.gamers.org` **and**
  `ftp.fu-berlin.de` and compared byte-for-byte. `youfailit.net` refused the TLS
  connection on 2026-10-22 and was not retried; it is not listed.
- Cacowards pages were read through the Wayback Machine because `www.doomworld.com`
  answers automated requests with HTTP 403; the archived legacy pages are cited with
  their year and role, and the canonical year URL is the one recorded in the recipes.
- Commands and results are in §11.

## 1. Doom the Way id Did — release identity

| Field | Value | Evidence |
| --- | --- | --- |
| Title / version | Doom the Way id Did, v1.1a (revised 2012-02-05; first release 2011-12-10) | `DTWID.txt` lines 4–7 |
| Author credits | Founder Hellbent; managers Hellbent, Xaser, ellmo; artist Kracov; 19 credited mappers | `DTWID.txt` "Additional Credits to" |
| Base game | Doom; `Game : Doom`, `Map # : E1M1-E3M9` | `DTWID.txt` lines 100–101 |
| Dependency statement | `Other files required : None`; `May Not Run With... Versions of Doom prior to 1.9` | `DTWID.txt` lines 95, 115 |
| Permission | "Authors may NOT use the contents of this file as a base for modification or reuse… You MAY distribute this file, provided you include this text file, with no modifications." | `DTWID.txt` lines 198–208 |

**Confirmed.** Package and members as downloaded (case-exact, no extraction changes):

| Path | Bytes | SHA-256 | Declared |
| --- | --- | --- | --- |
| `dtwid.zip` (package) | 2,005,606 | `122b50713026a05423776e5c1330770f74a371d4236c63497f0f7f64bc9a62cb` | package |
| `DTWID.wad` | 5,073,423 | `5fcec0759a85aa3860ade5cb745cd2930c825eecaea3d823dc9f63840c5ceab5` | maps member |
| `DTWID.txt` | 8,448 | `d58e57291240e31a6df14d2323e15a70c18bc7d2376669fcbec22440e1849dee` | readme (keep) |
| `DTWID.deh` | 2,047 | `b247dfba24f754535d7ef5a5e860fb9979ebac68d0ad7d6d827182d7993a8463` | **not declared** (see §2) |

ZIP member dates: `DTWID.txt` 2012-02-05, `DTWID.wad` 2012-02-07, `DTWID.deh` 2011-12-11.

## 2. Doom the Way id Did — WAD and DeHackEd inspection

**Confirmed.** `DTWID.wad` is a PWAD, 5,073,423 bytes, 398 lumps. Map markers are exactly
`E1M1`–`E1M9`, `E2M1`–`E2M9`, `E3M1`–`E3M9` (27 maps). There is **no** `COMPLVL`,
`UMAPINFO` or `MAPINFO` lump, and no `STBAR`/`INTERPIC`; it does ship `TITLEPIC` (66,888
bytes) and `DEMO1`–`DEMO3`.

The WAD contains an **embedded `DEHACKED` lump of 1,286 bytes** (first lump at offset
935,932) with a `[STRINGS]` block:

```text
HUSTR_E1M1 = E1M1: Communications Bridge
...
HUSTR_E2M3 = E2M3: Rec Facility
...
HUSTR_E3M7 = E3M7: Inner Sanctum
HUSTR_E3M9 = E3M9: Lake of Fire
```

The external `DTWID.deh` (2,047 bytes, dated 2011-12-11) uses old-style DeHackEd `Text`
blocks whose replacement names differ from the embedded revision, for example
`E2M3: Rec Center` (external) versus `E2M3: Rec Facility` (embedded). The external file
also carries `E4M1`–`E4M9` text for the base game. The WAD and its embedded patch are the
later revision (2012-02-07 versus 2011-12-11). **Confirmed.**

**Why no separate `-deh`:** DSDA-Doom applies a WAD-embedded `DEHACKED` lump from every
explicitly loaded PWAD unless `-nodeh` is given, and `-noautoload` does not disable it
(`docs/research/dsda-engine.md`, `prboom2/src/d_main.c:2147–2156` at v0.30.1). Loading the
older external file afterwards would be a second, conflicting application (its `Text`
prefixes no longer match the already-replaced strings), so the recipe declares only the
map WAD. **Confirmed by source; Needs Deck Validation** for the visible intermission
names in-game.

**Complevel (Proposed):** `vanilla` (alias; resolves to 3 on Doom/Ultimate Doom). The
readme states a vanilla Doom 1.9 target, ships no compatibility lump and asks for no
source port. Family `vanilla`.

**Episode scope:** E1–E3 are replaced; with Ultimate Doom, E4 remains the original
id content and is still selectable. This is why the recipe accepts registered Doom 1.9
and the 1.9ud variants, and why it is not an `episode` tag (27 maps).

## 3. Doom the Way id Did — tags and artwork

- **cacowards (Confirmed):** Doomworld's 2012 Cacowards (19th annual), Top Ten **Page 1
  winner**. Canonical page https://www.doomworld.com/cacowards/2012/ returned 403 to
  automated requests; read through the Wayback copy of
  https://www.doomworld.com/19years/ and `/19years/best1.php` on 2026-10-22. The entry
  calls it "a phenomenal Doom OG megaWAD" and describes the 27-map three-episode set.
  Prose use of "megaWAD" is not a map-count statement and is not used as tag evidence.
- **classics (Confirmed):** first release 2011-12-10, more than five years old; the
  Cacoward and the 2013 sequel are the cited lasting recognition.
- **community-projects (Confirmed):** 19 mappers and managers credited in `DTWID.txt`.
- **Not `megawad`:** 27 maps, below the 30-map definition. **Not `episode`:** 27 maps,
  above the nine-map definition.

**Artwork:** two real in-game views from the project's official site, which is only
reachable through the Wayback Machine. The site homepage (snapshot 2012-01-09) served a
slideshow of screenshots; the gallery page is artwork only. Chosen images were
`https://i.imgur.com/AtBK7.png` and `https://i.imgur.com/PEsge.png`, both 1280×960 PNG.
Each was centre-cropped to 1280×720 and encoded at JPEG quality 60 with
`scripts/lean-image.sh` (120,117 and 104,254 bytes, both under the 192 KiB cap).
Provenance and the "no separate licence found" note are in
`crates/ddl/ui/assets/screenshots/doom-the-way-id-did/SOURCES.toml`.

## 4. No End in Sight — release identity

| Field | Value | Evidence |
| --- | --- | --- |
| Title / version | No End in Sight, v1.4 (updated 2018-01-29; first release 2016-11-28) | `NEIS.txt` lines 8–12 |
| Authors | Emil "NaturalTvventy" Brundage, Xaser Acheron, Chris Lutz | `NEIS.txt` lines 13–15 |
| Base game | Doom (Ultimate Doom): `Game : Doom`, `Map # : E1M1-E4M9 (with extras on E1M0 and E4M0)` | `NEIS.txt` lines 54–55 |
| Port requirement | `Advanced engine needed : Limit-removing for E4; E1-E3 are vanilla-compat. Some very minor effects are Boom-compat…` | `NEIS.txt` lines 3–6 |
| Patch note | `Dehacked/BEX Patch : Yes (embedded in-wad; the external NEIS.deh is only necessary for Vanilla/Chocolate Doom)` | `NEIS.txt` lines 45–46 |
| Permission | Same "Authors may NOT… You MAY distribute this file, provided you include this text file" terms as DTWID | `NEIS.txt` lines 135–145 |

**Confirmed.** Package and members:

| Path | Bytes | SHA-256 | Declared |
| --- | --- | --- | --- |
| `neis.zip` (package) | 6,431,036 | `2cbc625a7bb5f0796046fc42b478f5b375060899366d652432375f26f50da66a` | package |
| `NEIS.wad` | 19,830,307 | `f4a7f69f07d72ac5fa284e71c8fb66fb9d38256bb4000f183c46bb92fe35e069` | maps |
| `NEIS.txt` | 6,847 | `683ebdcedd268af10524e29b55ad4169fd85aaef86cd5e70838591b57f2a3e82` | readme (keep) |
| `NEIS.deh` | 6,052 | `1aae57326b9263c529a1e8c5fa19fc2a15d23ff84edd038479981aa664bf0122` | **not declared** (see §5) |

ZIP member dates: `NEIS.txt` 2018-01-29, `NEIS.wad` 2018-02-01, `NEIS.deh` 2018-01-28.

## 5. No End in Sight — WAD intake, scope and compatibility

**Confirmed.** `NEIS.wad` is a PWAD, 19,830,307 bytes, 1,110 lumps. Map markers (38):
`E1M1`–`E1M9` + `E1M0`; `E2M1`–`E2M9`; `E3M1`–`E3M9`; `E4M1`–`E4M9` + `E4M0`. Other
relevant lumps: embedded `DEHACKED` 3,233 bytes (BEX `[STRINGS]` names and episode story
text, including `E4M8 = E4M8: No End In Sight`), `MAPINFO` 467 bytes, `EMAPINFO` 467
bytes, `STBAR` 13,128, `TITLEPIC`/`INTERPIC` 66,888, `DEMO1`–`DEMO4`. No `COMPLVL`,
`UMAPINFO` or `ZMAPINFO`.

The external `NEIS.deh` (6,052 bytes) is the vanilla/Chocolate form of the same text
patches; the readme explicitly limits it to those engines. The recipe therefore relies on
the embedded patch only, exactly as for DTWID. **Confirmed.**

**Complevel (Proposed):** `vanilla` alias (3 on Ultimate Doom) with family
`limit-removing`. The author states E1–E3 are vanilla-compatible and only E4 needs a
limit-removing engine; a limit-removing mapset conventionally runs at the vanilla
complevel. The readme's "some very minor effects are Boom-compat" is recorded as a
limitation: complevel 3 does not enable Boom line specials, and whether any cosmetic
effect is lost is unverified (**Needs Deck Validation**).

**Progression notes (Confirmed from text, not played):**
- The bundled `MAPINFO` (ZDaemon/ZDoom) and `EMAPINFO` (Eternity) are not read by
  DSDA-Doom; the menu selects E1–E4 and the maps.
- `E4M0` is a credited super-secret map that the author says is reachable "if playing in
  ZDoom or Eternity"; its reachability under DSDA-Doom is **Needs Deck Validation**.
- The readme warns that normal and secret exits are intentionally not swapped on some E3
  maps; this is design, not a defect.

**Tags:** `cacowards` = 2017 Cacowards Top Ten **Page 1 winner** (canonical
https://www.doomworld.com/cacowards/2017/, read through the Wayback copy of
`/24years/best1.php`); `classics` = 2016-11-28/2018-01-29 release plus that award;
`megawad` = 38 map slots counted from the WAD and the author's list. Not
`community-projects`: three credited authors with named graphics/test contributors, not a
broad community team.

**Artwork:** none bundled. The DoomWiki article carries only a title screen, which the
contribution rules exclude; no official-site in-game image was available, so the painted
fallback stays. Recorded in the recipe limitations.

## 6. Doom II the Way id Did — release identity and WAD intake

| Field | Value | Evidence |
| --- | --- | --- |
| Title / version | Doom II the Way id Did, v1.0 (release 2013-11-07, finalized 2013-12-22) | `D2TWID.txt` lines 6–10 |
| Credits | Managers Alfonzo and Xaser; designers Alfonzo, Esselfortium, Jimmy, Marcaek, Megalyth, Pavera, Purist, Rottking, Tarnsman, Xaser | `D2TWID.txt` lines 32–38 |
| Base game | Doom 2: `Game : Doom 2`, `Map # : MAP01-MAP32`; `May Not Run With... Versions of Doom 2 prior to 1.9` | `D2TWID.txt` lines 60–61, 74 |
| Permission | Same "Authors may NOT… distribute with this text file intact" terms | `D2TWID.txt` lines 155–165 |
| Archive note | `Archive Maintainer : Reuploaded because of zip issues` | `D2TWID.txt` lines 2–4 |

**Confirmed.** Package and members:

| Path | Bytes | SHA-256 | Declared |
| --- | --- | --- | --- |
| `d2twid.zip` (package) | 1,964,344 | `e5ef240143640e537ab5b47239967ec9e70ba84a314ee40d3a2acb66757aed32` | package |
| `D2TWID.wad` | 6,753,456 | `128a613717fee36b08b72ff5e54cf0e689136b25ab6fe7460f6ab634226f0e7f` | maps |
| `D2TWID.txt` | 5,938 | `019df270d2b644cacec330729bcebfe322396d40b0e81fd005ef0010f3481c77` | readme (keep) |
| `D2TWID.deh` | 7,272 | `945e3db025638d839e88d4ae510a13d38c6b43774dd0dc3a9fbcd6dafd8f8977` | **not declared** |

**Confirmed.** `D2TWID.wad` is a PWAD, 6,753,456 bytes, 732 lumps. It has **33 map
markers**: `MAP01`–`MAP32` plus **`MAP33`**, each with the full THINGS…BLOCKMAP set. Other
relevant lumps: embedded `DEHACKED` 6,384 bytes (WhackEd4 1.0.2; `[STRINGS]` map titles
including `PHUSTR_1 = level 33: Hexsoil`, `[PARS]`, the Dopefish/Yorp/Garg Wolfenstein
replacements and frames), `MAPINFO` 4,384, `ZMAPINFO` 4,960, `EMAPINFO` 3,399, `DEMO1`–
`DEMO3`. No `COMPLVL` or `UMAPINFO`.

**MAP33 (Confirmed from the bundled scripts, not played):** `MAP02` carries
`secretnext = MAP33`, and `MAP33` is titled "level 33: Hexsoil" with creator "Blackgaze as
Michael Bukowski" in `EMAPINFO`, and "Hexsoil" in `MAPINFO`. The authors' own map list
credits 32 maps and does not mention MAP33. DSDA-Doom reads `UMAPINFO`, not these
ZDaemon/ZDoom/Eternity scripts, so vanilla `MAP02` secret-exit behaviour (to `MAP31`) is
what DDL would produce; MAP33 reachability is **Needs Deck Validation**. The recipe
describes the 32-map megawad plus the bonus slot honestly and does not count MAP33 as a
required map.

**DeHackEd:** the embedded patch is the later form (WhackEd4, `[STRINGS]`/`[PARS]`); the
external 7,272-byte `D2TWID.deh` uses legacy `Text` blocks and is superseded. As with
DTWID, only the map WAD is declared. **Confirmed.**

**Complevel (Proposed):** `vanilla` alias (2 on Doom II), family `vanilla`; the readme
targets Doom II 1.9 with no port requirement or compatibility lump.

**Tags:** `cacowards` = 2013 Cacowards Top Ten **Page 1 winner** (canonical
https://www.doomworld.com/cacowards/2013/, read through the Wayback copy of
`/20years/`); `classics` = 2013 release plus that award; `community-projects` = ten
credited designers and their managers; `megawad` = 32 credited maps (33 WAD markers).

**Artwork:** none bundled. No official-site or thread image was reachable under the
contribution rules, so the painted fallback stays (recorded in the recipe).

## 7. Deathless — release identity, WAD intake and tags

| Field | Value | Evidence |
| --- | --- | --- |
| Title / version | Deathless v1.1, 2020-05-25 | `deathless1.1.txt` lines 6–12 |
| Author | Jimmy | `deathless1.1.txt` line 9 |
| Base game | The Ultimate Doom; `New levels : E1M1-M9, E2M1-M9, E3M1-M9, E4M1-M9` | `deathless1.1.txt` lines 59, 73 |
| Port requirement | `Advanced engine needed : Limit-removing`; "A LIMIT-REMOVING PORT IS RECOMMENDED"; `May Not Run With... Vanilla Doom, Chocolate Doom` | `deathless1.1.txt` lines 4, 27–29, 88 |
| Patch | `Dehacked/BEX Patch : Yes - lost soul and spiderdemon health have been changed…`; `Other : ZMAPINFO, EMAPINFO, DECORATE to replicate DEHACKED behavior changes` | `deathless1.1.txt` lines 63–68 |
| Permission | "Authors MAY use the contents of this file as a base… You MAY distribute this file, provided you include this text file" | `deathless1.1.txt` lines 91–101 |

**Confirmed.** Package and members:

| Path | Bytes | SHA-256 | Declared |
| --- | --- | --- | --- |
| `deathless.zip` (package) | 1,743,823 | `47481ee568d7fada97ffb15ffd308dd3d9085f974ca3687e154c6b9f06e7c9ee` | package |
| `deathless.wad` | 5,212,998 | `295efe40a40e5f93a913b762e1c6052220dab3ed8d409a0e758c1abac49600c3` | maps |
| `deathless1.1.txt` | 6,950 | `e0540ba517e70a9b29abaab7b4fe39c315c4fea7b8bbe6ffc1ca0584db897495` | readme (keep) |

**Confirmed.** `deathless.wad` is a PWAD, 5,212,998 bytes, 700 lumps, with all 36 map
markers `E1M1`–`E4M9`. One embedded `DEHACKED` lump (2,131 bytes, WhackEd4 1.0.4) sets:

```text
Thing 19 (Lost Soul)   Hit points = 70
Thing 20 (Spiderdemon) Speed = 16, Width = 5767168, Hit points = 3500, Pain chance = 20
Frame 615              Duration = 15
```

plus `Text` name replacements for E1M1–E4M9. The archive ships **no external `.deh`**.
Other lumps: `ZMAPINFO` 7,393, `EMAPINFO` 201, `DECORATE` 216, `TITLEPIC`/`INTERPIC`,
`DEMO1`–`DEMO4`. No `COMPLVL`, `UMAPINFO` or `MAPINFO`. The embedded patch is therefore
the only patch and DSDA-Doom applies it automatically; the recipe adds none. **Confirmed.**

**Complevel (Proposed):** `vanilla` alias (3 on Ultimate Doom) with family
`limit-removing`, per the author's engine statement and the limit-removing convention.

**Tags:** `classics` = released 2020-05-25, over five years old, and the readme credits
the mapset's inclusion as an official DOOM + DOOM II add-on (also stated at
https://bethesda.net/en-US/news/add-ons-for-doom-doom-ii-2022); `megawad` = 36 maps.
Not `cacowards`: no award could be confirmed, so no claim is made. Not
`community-projects`: a single-author mapset with credited graphics/test contributors.

**Artwork:** none bundled. DoomWiki carries only a title screen and no official-site
in-game image was available; the painted fallback stays (recorded in the recipe).

## 8. Sunder — selected release, scope and complexity

**Release selection (Confirmed, with a deliberate exclusion):** the author's project
thread is https://www.doomworld.com/forum/topic/46002-sunder-map21-actually-exists-right-now/.
The thread's first post as archived links **"Sunder V.2407"** at
`https://www.dropbox.com/s/vi47z1a4e4c4980/Sunder%202407.zip?dl=1` (Dropbox is an
already-allowed download host). The newer **V.2512** announcement of 2025-10-02 (MAP21)
links a OneDrive address, which the download allowlist does not cover; the release was
therefore **not** substituted and no host-policy change was made. This is the pinned
release, by SHA-256.

| Path | Bytes | SHA-256 | Declared |
| --- | --- | --- | --- |
| `Sunder 2407.zip` (package) | 49,972,580 | `c11bb7bb6f601b0badbed58f23d04e8b8f352795ebee66074645329d3f17fd2e` | package |
| `Sunder 2407.wad` | 167,039,763 | `b58787ea6390da29a5f2c2fd3488065a4979a30dc684d67e5a6d7ae26834005c` | maps |
| `SunderReadMeV2.TXT` | 34,421 | `4c67a6f5c798840953dc2d20344e3760627016f926c26d2f1cfbc5e9948b0e40` | readme (keep) |

ZIP internal timestamps: WAD 2021-12-31, readme 2021-12-15. The readme has no version or
completion date ("Date Finished : It's not. Still mapping!"); the version string comes
from the author's link and archive file name.

**Confirmed.** `Sunder 2407.wad` is a PWAD, 167,039,763 bytes, **2,530 lumps**, with
**22 map markers**: `MAP01`–`MAP20` plus `MAP31`/`MAP32`. Relevant lumps: embedded
`DEHACKED` 6,734 bytes (WhackEd4 1.2.0 BETA) with `[STRINGS]` names for every map and the
Hive Mother actor changes, a 3,126-byte `MAPINFO` in ZDaemon/ZDoom syntax (`map MAP01
"Python" … next MAP02`), `TITLEPIC`/`INTERPIC` 66,888, `STBAR` 13,128. No `COMPLVL` or
`UMAPINFO`; the bundled `MAPINFO` is not read by DSDA-Doom.

**Scope honesty (Confirmed):** the readme header still says `Map # : Map 01 - Map 19
(+ 31 & 32)`, but the WAD bytes contain `MAP20` and the readme has a `*Map 20*` section
("Beneath the Waves of Styx"). The recipe therefore describes **22 slots**, not the
readme's stale count and not the 14-map DSDA dataset. MAP21 (in V.2512) is absent.

**Compatibility (Confirmed statements, Proposed complevel):** the readme says
`*For DOOM II*`, "should work in any Boom compatible port", tested in ZDoom, GZDoom and
PrBoom+ (GLPrBoom+ had issues with MAP03), and that the embedded DeHackEd patch adds the
Hive Mother. `family = "boom"`, `complevel = "boom"` (9) follow the author's own port
statement.

**Complexity and performance (quoted, not measured on Deck):** the author calls the set
"extremely difficult maps" and a slaughter mapset; the readme warns "Map 15 is extremely
system intensive. Map 32 might also cause slow down on older machines", and the MAP15
notes record an author-observed GZDoom software-mode drop to "6 - 8 FPS in one especially
bad section". No Steam Deck FPS, frame-time or battery figure is claimed. **Needs Deck
Validation.**

**Tags:** `classics` = first released 2009-04-20 and named in Doomworld's 2018 Top 25
Missed Cacowards as influential (DoomWiki page read through the Wayback Machine); it is
not a Cacowards winner/runner-up, so `cacowards` is **not** claimed. `challenging` = the
author's "extremely difficult maps" statement. Not `megawad`: 22 maps in the pinned build.

**Artwork:** none bundled. DoomWiki's gallery has only 162-px thumbnails of two MAP11
views whose original size and licence could not be verified from bytes; rather than
upscale or guess, the painted fallback stays (recorded in the recipe).

## 9. REKKR — free PWAD edition

**Edition scoping (Confirmed):** this is the original free PWAD edition from /idgames
(`levels/doom/megawads/rekkr.zip`, id `rekkr`). The readme calls it "a total conversion for
Ultimate Doom v1.9", says `Other files required : None` and `Advanced engine needed :
None`, and the archive is a PWAD over the player's own Doom IWAD. The standalone IWAD
edition and the paid Sunken Land release are **not** acquired, declared or inferred from.

| Field | Value | Evidence |
| --- | --- | --- |
| Title / version | REKKR, version 1.17 (first release 2018-07-09) | `REKKR.txt` "Update to: 1.17", "Release date: 7/9/2018" |
| Uploader note | 1.17 "pulls in the first three episode map fixes and updates from Rekkr Sunken Land" | `REKKR.txt` lines 1–5 |
| Lead | Revae (lead, textures, most maps, additional sounds) | `REK_CREDS.txt` |
| Other credits | Music Tom Jensen (Hexenmapper); sounds TerminusEst13; maps AngrySaint, Bzzrak, AD_79, lupinx-Kassman, Velcrosasquatch, SuperCupcakeTactics, Jaws In Space, Jimmy, TerminusEst13; original E4M1 Bzzrak | `REKKR.txt` and `REK_CREDS.txt` |
| Base game | Doom, Ultimate Doom v1.9; `New levels : 36`, `Map # : All` | `REKKR.txt` |
| Port requirement | `Advanced engine needed : None`; `Tested With : Chocolate Doom, GZDoom, Zandronum, PRBoom(+), Eternity` | `REKKR.txt` |
| Licence | CC BY-NC 4.0; redistribution with credit and licence link, no commercial use | `REKKR.txt` copyright section |

**Confirmed.** Package and members:

| Path | Bytes | SHA-256 | Declared |
| --- | --- | --- | --- |
| `rekkr.zip` (package) | 14,152,590 | `c28a32699a58c976c135f8d5bcc270f2801a88019fc8a1d7b4805a54cb0b8429` | package |
| `REKKR.wad` | 41,960,202 | `fadf0b0ddb492c56e19ba5f56c8ac2e77413f287e8436cbcab31f2d41d7e9b0f` | maps |
| `REKKR.txt` | 3,188 | `ad736135c000c80d348cb1dc5b12eed69980dd0e45dfa10207ebf77b7c377cae` | readme (keep) |
| `REK_CREDS.txt` | 4,848 | `e8e8b89970f957996cf44013c0df9939c6f1c130dfe1bf8aeb2579d6d0b640e8` | credits (keep) |
| `REKKR.deh` | 61,321 | `8f99e53fdca663b89f5fcb4cafa2c30f747f75c668e6e8b5ce10866168f1cea2` | **not declared** (identical to the embedded lump) |

ZIP member dates: `REKKR.txt` 2024-06-22, `REKKR.deh` 2024-06-24, `REK_CREDS.txt`
2024-07-23, `REKKR.wad` 2025-01-19.

**Confirmed.** `REKKR.wad` is a PWAD, 41,960,202 bytes, 3,330 lumps, with 36 map markers
`E1M1`–`E4M9`. Other lumps: embedded `DEHACKED` **61,321 bytes**, `MAPINFO` 5,036
(GZDoom-family), `DECORATE` 1,204, `PLAYPAL`, `STBAR`, `TITLEPIC`/`INTERPIC` and
`DEMO1`–`DEMO4`. No `COMPLVL`, `UMAPINFO` or `ZMAPINFO`.

**Patch identity (Confirmed):** the embedded `DEHACKED` lump is **byte-identical** to the
archive's external `REKKR.deh` — both 61,321 bytes with SHA-256 `8f99e53f…1cea2`. Declaring
the external file as well would apply the same total-conversion patch twice, so the recipe
declares only the map WAD; DSDA-Doom applies the embedded patch automatically (and
`-noautoload` does not skip it).

**Complevel (Proposed):** `vanilla` alias (3 on Ultimate Doom), family `vanilla`, from the
author's "Advanced engine needed: None" and the Chocolate Doom test credit.

**Secondary confirmation (DoomWiki):** https://doomwiki.org/wiki/REKKR returned HTTP 403 to
the live request and was read through the Wayback Machine on 2026-10-22. It describes REKKR
as a four-episode total conversion by Matthew Little (Revae) et al., with `Ports: Vanilla
Ultimate Doom, GZDoom`, `IWAD: Ultimate Doom`, `Year: 2018`, and states "This mod received
one of the 2018 Cacowards on Doomworld". This agrees with the recipe's base game, port class
and award; it was not used for any byte identity.

**Tags:** `cacowards` = 2018 Cacowards **Top Ten Page 2 winner**; canonical
https://www.doomworld.com/cacowards/2018/ returned 403 and was read through its Wayback
copy on 2026-10-22 (that feature also names Revae Creator of the Year). `classics` = first
released 2018-07-09 plus that award; `community-projects` = nine credited mappers plus
music, sound and testing contributors; `megawad` = 36 maps. A total conversion has no
separate tag in the reviewed vocabulary, so none is invented.

**Artwork:** none bundled. `manbitesshark.com` (the site named in the readme) answers the
automated request with a Cloudflare challenge (HTTP 403) and the Cacowards page carries no
REKKR screenshot; rather than guess at a third-party image, the painted fallback stays.

## 10. Mirror verification (Confirmed)

| Package | www.gamers.org | ftp.fu-berlin.de | Byte-identical |
| --- | --- | --- | --- |
| `dtwid.zip` | 2,005,606 | 2,005,606 | yes (`122b5071…`) |
| `neis.zip` | 6,431,036 | 6,431,036 | yes (`2cbc625a…`) |
| `d2twid.zip` | 1,964,344 | 1,964,344 | yes (`e5ef2401…`) |
| `deathless.zip` | 1,743,823 | 1,743,823 | yes (`47481ee5…`) |
| `Sunder 2407.zip` | — | — | author Dropbox only |
| `rekkr.zip` | 14,152,590 | 14,152,590 | yes (`c28a3269…`) |

`youfailit.net` did not accept a TLS connection on 2026-10-22 (75 s connect timeout) and
is not listed. Sunder has one HTTPS source because the author publishes it himself and no
allowlisted mirror carries this build.

## 11. Commands and results (2026-10-22)

```sh
# Offline lint of the five assigned recipes, from the worktree
ddl catalog lint catalog/doom-the-way-id-did.toml catalog/no-end-in-sight.toml \
  catalog/doom-ii-the-way-id-did.toml catalog/deathless.toml catalog/sunder.toml
# -> Catalog lint passed (offline).

# Real-fetch lint (downloads every package and verifies every named member)
ddl catalog lint --fetch <same five paths>
# -> Catalog lint passed (fetched packages verified).

# Focused offline integration test
CARGO_TARGET_DIR=<shared> CARGO_BUILD_JOBS=2 \
  cargo test -p ddl-core --test catalog_087_b -- --test-threads=2
# -> 5 passed; 0 failed.

cargo fmt --all --check        # -> clean
CARGO_TARGET_DIR=<shared> CARGO_BUILD_JOBS=2 \
  cargo clippy -p ddl-core --all-targets -- -D warnings   # -> clean
```

## 12. Open checks and limitations

- **Needs Deck Validation (all five):** actual IWAD detection, complevel, load order,
  gameplay, audio and episode/map selection on the Deck.
- **Doom the Way id Did:** embedded intermission names, E4 remaining vanilla, option of
  the separately distributed Mr. Freeze soundtrack (not inspected, not declared).
- **No End in Sight:** E4 limit-removing behaviour, the minor Boom-only effects, and
  `E4M0` reachability in DSDA-Doom.
- **Doom II the Way id Did:** `MAP33` reachability; the vanilla `MAP02` secret exit
  targets `MAP31` in DSDA-Doom.
- **Deathless:** the embedded monster rebalance and the author's Crispy-Doom
  recommendation versus DDL's DSDA-Doom launch.
- **Sunder:** performance on Deck is unmeasured; the pinned V.2407 build is not the
  author's newest V.2512, which is only offered on an allowlist-excluded host. MAP20
  exists despite the readme's stale count line.
- **REKKR:** the total-conversion DeHackEd behaviour (new monsters, weapons and strings)
  was not played on Deck; only the free PWAD edition over Ultimate Doom is declared.
- **Artwork:** only Doom the Way id Did has bundled screenshots; the other five record
  unavailable compliant sources and keep the painted fallback.

## 13. Original first-public dates for spec 090 chronology (research only)

Recorded for worker D's ascending release-order metadata; no schema field, recipe date or
version label was changed here. Precision is stated per title so sorting never overstates
what the source says.

| Title (recipe id) | Original first public date | Precision | Source | Version label kept in the recipe |
| --- | --- | --- | --- | --- |
| Doom the Way id Did (`doom-the-way-id-did`) | 2011-12-10 | day | `DTWID.txt` "Release date : 12/10/11" (the inspected v1.1a is a 2012-02-05 revision) | 1.1a, revised 2012-02-05 |
| No End in Sight (`no-end-in-sight`) | 2016-11-28 | day | `NEIS.txt` "Release date : 11/28/16" (the inspected v1.4 is the 2018-01-29 update) | 1.4, updated 2018-01-29 |
| Doom II the Way id Did (`doom-ii-the-way-id-did`) | 2013-11-07 | day | `D2TWID.txt` "Release date : 11/7/13 (finalized 12/22/13)" | 1.0, finalized 2013-12-22 |
| Deathless (`deathless`) | 2018-11-09 | day | DoomWiki infobox `Year 2018` and text "initial version of the full megawad was shared on November 9th", plus the author's release post dated "November 9, 2018" (`doomworld.com/forum/topic/103075-deathless-an-ultimate-doom-megawad-rc3/`). The selected v1.1 update is 2020-05-25; the July 2018 E4-only predecessor Griefless is a different release. | 1.1, 2020-05-25 |
| Sunder (`sunder`) | 2009-04-20 | day | DoomWiki (Wayback copy read 2026-10-22): "first release on April 20, 2009"; the project thread was also started 2009-04-20. The inspected build is V.2407, not the first upload. | V.2407 |
| REKKR (`rekkr`) | 2018-07-09 | day | `REKKR.txt` "Release date : 7/9/2018" (the inspected 1.17 update is later; first public release is the original free edition) | 1.17 |

Where only a day-precision date exists, it is used as-is. No title in this group relies on
an inferred month or year: each has an author-stated date or a dated public release/thread.

**Correction applied 2026-10-22 (second coordinator review):** the earlier row used the
selected v1.1 update date (2020-05-25) as the original release, and a first correction wrongly
said 2019-11-09. The primary sources fix the year as 2018: the author's release post is dated
"November 9, 2018" and DoomWiki's infobox says `Year 2018`, with "the initial version of the
full megawad … shared on November 9th" during the first NaNoWADMo. The honest value is
**2018-11-09 (day precision)**; 2020-05-25 remains the selected v1.1 upload. The predecessor
Griefless (July 2018) is an E4-only release and is not the same title. The recipe's version
label and hashes are unchanged.

# Doomer Boards Projects — inventory of DBP 01–77 (spec 037, R1)

Scope: every project published on the Doomer Boards **Project Catalog** page, so DDL can decide,
per project, whether a DSDA-Doom recipe exists or why it cannot. Research date: **2026-10-19**
(UTC). Host: macOS arm64 dev machine; tools `curl`, `python3` (stdlib `zipfile`, `hashlib`),
`sips`, `ffmpeg`. Nothing downloaded from any archive was executed: archives were only listed,
read and hashed.

Labels used below: **Confirmed** (observed/quoted from a primary source in this session),
**Proposed** (a DDL decision, not an upstream fact), **Needs Deck Validation** (cannot be
verified on the macOS arm64 dev host).

## Method and evidence

- **Project list.** The live page sits behind a proof-of-work bot challenge, so the Wayback
  capture of **2026-09-29** was used:
  `https://web.archive.org/web/20260929225359id_/https://doomer.boards.net/page/dbp`.
  **Confirmed:** the page body contains exactly **77 cards**, one per project, numbered by their
  CSS background variable `--imgdbpNN` for **NN = 01…77 with no gaps** (the stylesheet defines
  `--imgdbp01` … `--imgdbp99`, which is where the "DBP 01–99" impression comes from; the body
  only ever has 77 cards). Each card links to that project's release thread.
- **Release threads.** Each of the 77 thread URLs was fetched as a raw Wayback snapshot
  (`https://web.archive.org/web/2026id_/<thread>`), giving the first post's date, the stated
  target port, the map list and the published download links. **Confirmed.**
- **/idgames.** The recursive index `https://www.gamers.org/pub/idgames/ls-laR.gz` (fetched
  2026-10-19) was parsed for `dbp*` entries: **40 of the 77 projects are on /idgames**
  (`levels/doom2/Ports/d-f/`, `levels/doom2/Ports/megawads/`, `themes/xmas/`, `themes/chex/`).
  For each, the sidecar `.txt` and the `.zip` were downloaded; `Advanced engine needed`,
  `Tested With`, `Map #`, the map list and the copyright section are quoted from those files.
  **Confirmed.**
- **DSDA archive mirror.** `https://dsdarchive.com/search?search=dbp` lists 49 DBP wads. Nine
  projects that are not on /idgames exist only there (`dbp_34`, `dbp_35`, `dbp_39`, `dbp_45`,
  `dbp16_v0_99a`, `dbp21_rc2`, `dbp_54_rc1_3`, `dbp_66_rc3`, `dbp67_rc3`); their wad pages also
  record demos with the engine and complevel used (e.g. `DSDA-Doom v0.28.3cl2` for DBP34), which
  is direct evidence that the maps run under DSDA-Doom. **Confirmed.**
- **Dropbox.** D-028 allows Dropbox (`?dl=1`) when a project is not on /idgames. **Confirmed:**
  `www.dropbox.com` does **not** redirect to `dl.dropboxusercontent.com`; it redirects to a
  per-request hashed CDN host such as `uc6b74966005c2b22696e15c5150.dl.dropboxusercontent.com`,
  and three identical requests produced three different hashes. An exact-host allowlist
  therefore cannot authorise a Dropbox install, and `ddl catalog lint --fetch` rejects it. The
  21 Dropbox-only projects are therefore pinned from the archive.org item instead (D-028
  extension approved by the coordinator for 41/60/69 and used for the same reason here); the
  downloader has a deliberate no-suffix rule (`hosts.rs` test
  `does_not_match_subdomains_or_suffixes`), so a real Dropbox install needs a code change, not a
  catalog change.
- **archive.org.** The community item `dbp-wad-archive`
  (`https://archive.org/details/dbp-wad-archive`) holds one ZIP per project for DBP **01–78**,
  including the ones missing from /idgames and from the Dropbox links that have rotted. Its
  metadata names `ia600903.us.archive.org` as the item server; `archive.org/download/...`
  redirects through a per-request `dnNNNNNN.<region>.archive.org` host to that server, so the
  recipes use the item server URL directly.
- **Doomshack.** `doomshack.org` (used by DBP41 and several mirrors) no longer serves files:
  `http://doomshack.org/...` answers **302** with
  `Location: https://web.archive.org/web/20260103060906/https:/doomshack.org/uploads/…`, and the
  HTTPS endpoint does not connect from this host. The file was verified from its Wayback copy
  (3,529,660 bytes, identical to the `dbp-wad-archive` copy). **Confirmed.**

## Findings

1. **There is no DBP 78–99.** The spec's "DBP 01–99" comes from the page stylesheet
   (`--imgdbp01`…`--imgdbp99`), not from published projects: the card list holds 77 entries.
   DBP78 *"Ghost Riders of Christmas Past"* exists as an RC1 upload in the `dbp-wad-archive`
   item (uploaded 2026-01-01) but has no card on the catalog page, so it is out of scope here
   and left for a follow-up.
2. **Every DBP 01–77 is DSDA-Doom-playable.** All 77 are Doom II releases; the release posts
   and readmes describe them as **limit-removing** (`cl2`), except **DBP33 "Rainy Days in
   Casablanca"**, whose readme requires **Boom** (complevel 9), and **DBP42 "Slime on Earth"**
   (Chex-themed), whose readme states **MBF21** (complevel 21) and `Tested With: DSDA-Doom and
   GZDoom`. No project is GZDoom-only, UDMF or ZScript: none of the 77 readmes names UDMF,
   ZScript or a GZDoom-only exe as a requirement, and DBP48's readme offers a GZDoom original
   *and* a limit-removing version (the package in the catalog is the limit-removing one).
   **Confirmed** for the readme-backed projects; **Proposed** where a project has no readme
   (DBP32 and DBP70 are bare WADs, DBP16 ships only a credits file) and the release post's
   "designed for limit removing source ports" is the only statement.
3. **Source priority actually used.** Of the 76 new recipes: **43 come from /idgames**
   (mirrors: `www.gamers.org`, `youfailit.net`, `ftp.fu-berlin.de`), **9 from the DSDA archive
   mirror** (`dsdarchive.com`, already allowlisted) and **24 from the `dbp-wad-archive` item**
   (`ia600903.us.archive.org`): DBP41, DBP60 and DBP69, which have no /idgames entry, plus the
   21 projects that only publish on Dropbox, which cannot be used (see above). DBP37
   (AUGER;ZENITH) already has recipe `auger-zenith`.
4. **Versions pinned.** Where several copies exist, the recipe pins one exact archive: DBP05
   `dbp_05.zip` (idgames "testing version" title notwithstanding), DBP12 v1.2, DBP23 v2,
   DBP40 RC1, DBP44 RC2-4, DBP46 RC2, DBP64 RC3, DBP65 v3, DBP68 RC1.4, DBP73 v3, DBP76 v1.1.
   The archive.org copies are *not* always byte-identical to the /idgames or Dropbox ones
   (size and SHA-256 differ), so each recipe pins exactly the copy it was verified against.
5. **Permissions.** 35 of the /idgames readmes state **Creative Commons Attribution 4.0
   International**; the rest say "Authors may/may not use the contents … as a base", with the
   text file to travel unmodified. The full per-project copyright section is quoted in each
   recipe's `permission` field.

## Inventory

"Released" is the release thread's first-post date (the readme date where they differ is noted in
the recipe). "Port / format" quotes the readme's `Advanced engine needed`/`Tested With` lines or,
where there is no readme, the release post. All verdicts are **Needs Deck Validation** for
IWAD/complevel/gameplay: no DSDA-Doom run was performed on a Deck.

| # | Project | Released | Maps | IWAD | Port / format | /idgames | Other downloads | DSDA-Doom v0.30.1 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | [Monuments of Mars](https://doomer.boards.net/thread/1140/ti-limit-removing-monuments-mars/) | 2018-03-31 | 7 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp01.zip` | TSPG (Zandronum) | Playable — recipe `monuments-of-mars`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp01.txt) |
| 2 | [Circle of Caina](https://doomer.boards.net/thread/1208/ti-limit-removing-circle-caina) | 2018-04-30 | 8 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_02.zip` | — | Playable — recipe `circle-of-caina`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_02.txt) |
| 3 | [Forest Swords](https://doomer.boards.net/thread/1249/ti-limit-removing-forest-swords) | 2018-05-31 | 8 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_03.zip` | — | Playable — recipe `forest-swords`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_03.txt) |
| 4 | [Xenomorph Base](https://doomer.boards.net/thread/1330/ti-limit-removing-xenomorph-base) | 2018-07-01 | 6 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_04.zip` | MediaFire (not allowlisted) | Playable — recipe `xenomorph-base`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_04.txt) |
| 5 | [Coffin Curse](https://doomer.boards.net/thread/1391/ti-limit-removing-coffin-curse) | 2018-08-01 | 7 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_05.zip` | TSPG (Zandronum) | Playable — recipe `coffin-curse`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_05.txt) |
| 6 | [Vicarious Reality](https://doomer.boards.net/thread/1417/ti-limit-removing-vicarious-reality) | 2018-09-01 | 6 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_06.zip` | TSPG (Zandronum) | Playable — recipe `vicarious-reality`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_06.txt) |
| 7 | [The Merry Christmassy Doom Project](https://doomer.boards.net/thread/1453/limit-removing-christmassy-project-released) | 2018-12-24 | 5 | doom2 | limit-removing, cl2 (stated) | `themes/xmas/dbp7xmas.zip` | Doomshack (dead) | Playable — recipe `the-merry-christmassy-doom-project`; evidence: [readme](https://www.gamers.org/pub/idgames/themes/xmas/dbp7xmas.txt) |
| 8 | [Mindblood Genesis](https://doomer.boards.net/thread/1474/doom-mindblood-genesis-dbp08-released) | 2019-02-01 | 9 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp8_mbg.zip` | Dropbox | Playable — recipe `mindblood-genesis`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp8_mbg.txt) |
| 9 | [Legend of the Hidden Tech](https://doomer.boards.net/thread/1491/limit-removing-legend-hidden-released) | 2019-03-03 | 9 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_09.zip` | Dropbox | Playable — recipe `legend-of-the-hidden-tech`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_09.txt) |
| 10 | [Earth Ends](https://doomer.boards.net/thread/1505/news-earth-ends-idgames-version) | 2019-04-05 | 6 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp10.zip` | TSPG (Zandronum) | Playable — recipe `earth-ends`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp10.txt) |
| 11 | [Lilywhite Lilith](https://doomer.boards.net/thread/1508/limit-removing-dbp11-lilywhite-lilith) | 2019-05-01 | 10 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp11.zip` | Google Drive (not allowlisted) | Playable — recipe `lilywhite-lilith`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp11.txt) |
| 12 | [Into The Storm](https://doomer.boards.net/thread/1524/ti-doom-storm-dbp12-released) | 2019-06-04 | 10 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp12.zip` | Dropbox, TSPG (Zandronum) | Playable — recipe `into-the-storm`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp12.txt) |
| 13 | [Alien Bastards!](https://doomer.boards.net/thread/1541/dbp13-alien) | 2019-07-04 | 9 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp13.zip` | TSPG (Zandronum) | Playable — recipe `alien-bastards`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp13.txt) |
| 14 | [After the Fall](https://doomer.boards.net/thread/1545/limit-removing-dbp14-after-released) | 2019-07-30 | 10 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp14atf.zip` | ModDB page, TSPG (Zandronum) | Playable — recipe `after-the-fall`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp14atf.txt) |
| 15 | [Fuck Global Warming... This is Hell](https://doomer.boards.net/thread/1560/ti-community-dbp-global-warming) | 2019-09-03 | 8 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp15.zip` | Doomshack (dead), TSPG (Zandronum) | Playable — recipe `fuck-global-warming-this-is-hell`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp15.txt) |
| 16 | [Cyb's Freaky Colonoscopy](https://doomer.boards.net/thread/1570/limit-removing-cybs-freaky-colonoscopy) | 2019-10-05 | 12 | doom2 | limit-removing, cl2 | — (not published there) | [dsdarchive](https://dsdarchive.com/files/wads/doom2/4895/dbp16_v0.99a.zip); Dropbox, ModDB page, TSPG (Zandronum) | Playable — recipe `cybs-freaky-colonoscopy`; evidence: [DSDA archive page](https://dsdarchive.com/wads/dbp16_v0_99a) (demos recorded on DSDA-Doom) |
| 17 | [Alone](https://doomer.boards.net/thread/1578/ti-limit-removing-dbp17) | 2019-10-31 | 9 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp17_alone.zip` | — | Playable — recipe `alone`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp17_alone.txt) |
| 18 | [Umbral Platinum](https://doomer.boards.net/thread/1587/dbp18-umbral-platinum) | 2019-12-04 | 7 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp18_up.zip` | TSPG (Zandronum) | Playable — recipe `umbral-platinum`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp18_up.txt) |
| 19 | [A Doomer Boards Christmas Carol](https://doomer.boards.net/thread/1591/community-doomer-boards-christmas-released) | 2019-12-24 | 25 | doom2 | limit-removing, cl2 (stated) | `themes/xmas/dbp19xmas2.zip` | TSPG (Zandronum) | Playable — recipe `a-doomer-boards-christmas-carol`; evidence: [readme](https://www.gamers.org/pub/idgames/themes/xmas/dbp19xmas2.txt) |
| 20 | [Dungeons and Demons](https://doomer.boards.net/thread/1606/dbp20-dungeons-demons-rc-1) | 2020-02-02 | 9 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp20_dnd.zip` | Google Drive (not allowlisted) | Playable — recipe `dungeons-and-demons`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp20_dnd.txt) |
| 21 | [Occult Secrets of the Third Reich on Mars](https://doomer.boards.net/thread/1640/dbp21-occult-secrets-third-reich) | 2020-03-04 | 10 | doom2 | limit-removing, cl2 | — (not published there) | [dsdarchive](https://dsdarchive.com/files/wads/doom2/3585/dbp21_rc2.zip); Dropbox, ModDB page, TSPG (Zandronum) | Playable — recipe `occult-secrets-of-the-third-reich-on-mars`; evidence: [DSDA archive page](https://dsdarchive.com/wads/dbp21_rc2) (demos recorded on DSDA-Doom) |
| 22 | [Biotech is Godzilla](https://doomer.boards.net/thread/1682/limit-removing-dbp22-biotech-godzilla) | 2020-03-31 | 9 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp22.zip` | Dropbox | Playable — recipe `biotech-is-godzilla`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp22.txt) |
| 23 | [Evil Egypt](https://doomer.boards.net/thread/1714/evil-egypt) | 2020-05-01 | 12 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_23_v2.zip` | Doomshack (dead), TSPG (Zandronum) | Playable — recipe `evil-egypt`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_23_v2.txt) |
| 24 | [Spaceballs: The Quest For More DBP Maps](https://doomer.boards.net/thread/1749/dbp24-spaceballs-sequel-quest-more) | 2020-05-31 | 10 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp24_sb.zip` | Google Drive (not allowlisted) | Playable — recipe `spaceballs-the-quest-for-more-dbp-maps`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp24_sb.txt) |
| 25 | [Dead But Dreaming](https://doomer.boards.net/thread/1773/doomer-boards-project-dead-dreaming) | 2020-07-02 | 11 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp25_dbd.zip` | ModDB page | Playable — recipe `dead-but-dreaming`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp25_dbd.txt) |
| 26 | [The City of Damned Children](https://doomer.boards.net/thread/1821/dbp26-city-damned-children) | 2020-07-31 | 11 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp26_tcodc.zip` | — | Playable — recipe `the-city-of-damned-children`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp26_tcodc.txt) |
| 27 | [Ten Day Vacation](https://doomer.boards.net/thread/1846/dbp27-10-day-vacation) | 2020-09-01 | 10 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp27.zip` | Doomshack (dead) | Playable — recipe `ten-day-vacation`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp27.txt) |
| 28 | [Fear and Loathing](https://doomer.boards.net/thread/1868/dbp28-fear-loathing) | 2020-10-01 | 10 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp28.zip` | ModDB page | Playable — recipe `fear-and-loathing`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp28.txt) |
| 29 | [Morbid Autumn](https://doomer.boards.net/thread/1897/morbid-autumn) | 2020-11-01 | 10 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_29.zip` | Doomshack (dead), TSPG (Zandronum) | Playable — recipe `morbid-autumn`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_29.txt) |
| 30 | [The Magnificent Five](https://doomer.boards.net/thread/1948/dbp30-magnificent) | 2020-12-01 | 6 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp30.zip` | — | Playable — recipe `the-magnificent-five`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp30.txt) |
| 31 | [Santa's Outback Bender](https://doomer.boards.net/thread/1982/dbp31-santas-outback-bender-released) | 2020-12-24 | 16 | doom2 | limit-removing, cl2 | `themes/xmas/dbp31.zip` | ModDB page | Playable — recipe `santas-outback-bender`; evidence: [readme](https://www.gamers.org/pub/idgames/themes/xmas/dbp31.txt) |
| 32 | [Tomatomania](https://doomer.boards.net/thread/2035/dbp32-tomatomania) | 2021-02-24 | 13 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp32_tomatomania.zip); Dropbox, ModDB page, TSPG (Zandronum) | Playable — recipe `tomatomania`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 33 | [Rainy Days in Casablanca](https://doomer.boards.net/thread/2042/dbp33-rainy-days-casablanca) | 2021-03-02 | 6 | doom2 | Boom (cl9) | `levels/doom2/Ports/d-f/dbp33.zip` | Doomshack (dead), TSPG (Zandronum) | Playable — recipe `rainy-days-in-casablanca`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp33.txt) |
| 34 | [Luminous Gloom](https://doomer.boards.net/thread/2065/luminous-gloom) | 2021-04-01 | 10 | doom2 | limit-removing, cl2 (stated) | — (not published there) | [dsdarchive](https://dsdarchive.com/files/wads/doom2/3537/DBP_34.zip); Doomshack (dead) | Playable — recipe `luminous-gloom`; evidence: [DSDA archive page](https://dsdarchive.com/wads/dbp_34) (demos recorded on DSDA-Doom) |
| 35 | [Stroggman's Tundra](https://doomer.boards.net/thread/2104/stroggmans-tundra) | 2021-05-01 | 10 | doom2 | limit-removing, cl2 (stated) | — (not published there) | [dsdarchive](https://dsdarchive.com/files/wads/doom2/3982/DBP_35.zip); Doomshack (dead), TSPG (Zandronum) | Playable — recipe `stroggmans-tundra`; evidence: [DSDA archive page](https://dsdarchive.com/wads/dbp_35) (demos recorded on DSDA-Doom) |
| 36 | [Aquatic Wonder](https://doomer.boards.net/thread/2139/dbp36-aquatic-wonder) | 2021-06-01 | 8 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp36.zip` | TSPG (Zandronum) | Playable — recipe `aquatic-wonder`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp36.txt) |
| 37 | [AUGER;ZENITH](https://doomer.boards.net/thread/2176/dbp37-auger-zenith) | 2021-07-26 | 21 | doom2 | limit-removing (cl2) | `levels/doom2/Ports/megawads/dbp37_augzen.zip` | — | Playable — recipe `auger-zenith` (already in the catalog) |
| 38 | [Chronicles of Ghost Town](https://doomer.boards.net/thread/2218/chronicles-ghost-town) | 2021-08-01 | 17 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp38_chronicles-of-ghost-town-v1-5.zip); Doomshack (dead), TSPG (Zandronum) | Playable — recipe `chronicles-of-ghost-town`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 39 | [Carnage Oasis](https://doomer.boards.net/thread/2271/carnage-oasis) | 2021-09-02 | 12 | doom2 | limit-removing, cl2 (stated) | — (not published there) | [dsdarchive](https://dsdarchive.com/files/wads/doom2/3551/dbp_39.zip); Doomshack (dead) | Playable — recipe `carnage-oasis`; evidence: [DSDA archive page](https://dsdarchive.com/wads/dbp_39) (demos recorded on DSDA-Doom) |
| 40 | [Funnelcake Apparitions](https://doomer.boards.net/thread/2298/funnelcake-apparitions) | 2021-10-01 | 8 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp40.zip` | TSPG (Zandronum) | Playable — recipe `funnelcake-apparitions`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp40.txt) |
| 41 | [Hell Revealed III (Pt. 1)](https://doomer.boards.net/thread/2328/dbp41-revealed-iii-pt-1) | 2021-11-03 | 9 | doom2 | limit-removing, cl2 (stated) | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp41_hell-revealed-3-pt-1-beta-v1c.zip); Doomshack (dead) | Playable — recipe `hell-revealed-3-pt-1`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 42 | [Slime on Earth](https://doomer.boards.net/thread/2343/slime-on-earth) | 2021-12-01 | 7 | doom2 | MBF21 (cl21) | `themes/chex/dbp42.zip` | TSPG (Zandronum) | Playable — recipe `slime-on-earth`; evidence: [readme](https://www.gamers.org/pub/idgames/themes/chex/dbp42.txt) |
| 43 | [Christmas in Inferno](https://doomer.boards.net/thread/2365/christmas-inferno) | 2021-12-24 | 5 | doom2 | limit-removing, cl2 | `themes/xmas/dbp43.zip` | — | Playable — recipe `christmas-in-inferno`; evidence: [readme](https://www.gamers.org/pub/idgames/themes/xmas/dbp43.txt) |
| 44 | [Japanese Valentines](https://doomer.boards.net/thread/2432/japanese-valentines) | 2022-02-07 | 8 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp44.zip` | TSPG (Zandronum) | Playable — recipe `japanese-valentines`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp44.txt) |
| 45 | [Vrack Botanicals](https://doomer.boards.net/thread/2462/vrack-botanicals) | 2022-03-01 | 10 | doom2 | limit-removing, cl2 (stated) | — (not published there) | [dsdarchive](https://dsdarchive.com/files/wads/doom2/3695/DBP_45.zip); Doomshack (dead), TSPG (Zandronum) | Playable — recipe `vrack-botanicals`; evidence: [DSDA archive page](https://dsdarchive.com/wads/dbp_45) (demos recorded on DSDA-Doom) |
| 46 | [Monuments of Mars 2](https://doomer.boards.net/thread/2527/monuments-mars-2) | 2022-04-02 | 10 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_46_rc2.zip` | MediaFire (not allowlisted), TSPG (Zandronum) | Playable — recipe `monuments-of-mars-2`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_46_rc2.txt) |
| 47 | [Dreamcatcher Apparatus](https://doomer.boards.net/thread/2603/dreamcatcher-apparatus) | 2022-05-01 | 20 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/megawads/dbp47.zip` | TSPG (Zandronum) | Playable — recipe `dreamcatcher-apparatus`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/dbp47.txt) |
| 48 | [AvPvTvJVvHvSS](https://doomer.boards.net/thread/2649/dbp-avp-terminator-jason-seagal) | 2022-06-04 | 11 | doom2 | limit-removing, cl2 (stated) | — (not published there) | [Dropbox](https://www.dropbox.com/s/dy2mlhiay8ut9xx/DBP_48.zip?dl=1); Dropbox, ModDB page | Playable — recipe `avpvtvjvvhvss`; evidence: [release thread](https://doomer.boards.net/thread/2649/dbp-avp-terminator-jason-seagal) |
| 49 | [Mausoleum Nefarium](https://doomer.boards.net/thread/2666/mausoleum-nefarium) | 2022-07-01 | 11 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp49_mausoleum-nefarium-v1-01.zip); Dropbox, TSPG (Zandronum) | Playable — recipe `mausoleum-nefarium`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 50 | [Emerald City](https://doomer.boards.net/thread/2929/emerald-city-rc1-2) | 2022-08-01 | 16 | doom2 | limit-removing, cl2 | `levels/doom2/Ports/d-f/dbp50.zip` | — | Playable — recipe `emerald-city`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp50.txt) |
| 51 | [Deadly Ritual](https://doomer.boards.net/thread/2960/deadly-ritual) | 2022-09-01 | 12 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_51.zip` | MediaFire (not allowlisted) | Playable — recipe `deadly-ritual`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_51.txt) |
| 52 | [Havoc in Creation](https://doomer.boards.net/thread/2978/havoc-creation) | 2022-09-30 | 12 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_52.zip` | TSPG (Zandronum) | Playable — recipe `havoc-in-creation`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_52.txt) |
| 53 | [Plutonian Sunrise](https://doomer.boards.net/thread/2998/plutonian-sunrise) | 2022-11-02 | 12 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp53_plutonian-sunrise-v1-1.zip); Dropbox, TSPG (Zandronum) | Playable — recipe `plutonian-sunrise`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 54 | [Steamy Bathhouse](https://doomer.boards.net/thread/3022/steamy-bathhouse) | 2022-12-03 | 9 | doom2 | limit-removing, cl2 (stated) | — (not published there) | [dsdarchive](https://dsdarchive.com/files/wads/doom2/5148/DBP_54_RC1.3.zip); Dropbox, TSPG (Zandronum) | Playable — recipe `steamy-bathhouse`; evidence: [DSDA archive page](https://dsdarchive.com/wads/dbp_54_rc1_3) (demos recorded on DSDA-Doom) |
| 55 | [Jolly Junkyard](https://doomer.boards.net/thread/3050/jolly-junkyard) | 2022-12-24 | 8 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_55.zip` | Dropbox, TSPG (Zandronum) | Playable — recipe `jolly-junkyard`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_55.txt) |
| 56 | [Augustland Barrens](https://doomer.boards.net/thread/3101/augustland-barrens) | 2023-02-01 | 9 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_56.zip` | Dropbox, TSPG (Zandronum) | Playable — recipe `augustland-barrens`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_56.txt) |
| 57 | [Shatter Realm](https://doomer.boards.net/thread/3116/shatter-realm) | 2023-03-01 | 9 | doom2 | limit-removing, cl2 (stated) | `levels/doom2/Ports/d-f/dbp_57.zip` | Dropbox, TSPG (Zandronum) | Playable — recipe `shatter-realm`; evidence: [readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dbp_57.txt) |
| 58 | [Immortal Warfare](https://doomer.boards.net/thread/3169/immortal-warfare) | 2023-05-01 | 10 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp58_immortal-warfare.zip); Dropbox, TSPG (Zandronum) | Playable — recipe `immortal-warfare`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 59 | [Zeppelin Armada](https://doomer.boards.net/thread/3272/zeppelin-armada) | 2023-07-03 | 18 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp59_zeppelin-armada-full-v1-1.zip); Google Drive (not allowlisted), ModDB page, TSPG (Zandronum) | Playable — recipe `zeppelin-armada`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 60 | [Eldritch Voyage](https://doomer.boards.net/thread/3301/eldritch-voyage) | 2023-08-01 | 14 | doom2 | limit-removing, cl2 (stated) | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp60_eldritch-voyage-v1-1.zip); TSPG (Zandronum) | Playable — recipe `eldritch-voyage`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 61 | [Tempest Enterprise](https://doomer.boards.net/thread/3330/tempest-enterprise) | 2023-09-02 | 14 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp61_tempest-enterprise-v1-2.zip); Dropbox | Playable — recipe `tempest-enterprise`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 62 | [Haunting Hollow](https://doomer.boards.net/thread/3364/haunting-hollow) | 2023-10-01 | 11 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp62_haunting-hollow-v1-4.zip); Dropbox, TSPG (Zandronum) | Playable — recipe `haunting-hollow`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 63 | [Terminal Voltage](https://doomer.boards.net/thread/3421/terminal-voltage) | 2023-12-07 | 9 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp63_terminal-voltage.zip); Dropbox | Playable — recipe `terminal-voltage`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 64 | [The Vast Silence](https://doomer.boards.net/thread/3489/silence) | 2024-02-04 | 14 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp64_the-vast-silence-rc3.zip); Dropbox, TSPG (Zandronum) | Playable — recipe `the-vast-silence`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 65 | [Sanguine Holy Land](https://doomer.boards.net/thread/3531/dbp65-sanguine-holy-land) | 2024-04-13 | 14 | doom2 | limit-removing, cl2 (stated in the release post) | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp65_sanguine-holy-land-v3.zip); Dropbox, Google Drive (not allowlisted), archive.org copy | Playable — recipe `sanguine-holy-land`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 66 | [Lunar Strain](https://doomer.boards.net/thread/3575/lunar-strain) | 2024-06-20 | 23 | doom2 | limit-removing, cl2 (stated) | — (not published there) | [dsdarchive](https://dsdarchive.com/files/wads/doom2/5029/DBP_66_RC3.zip); Dropbox, TSPG (Zandronum) | Playable — recipe `lunar-strain`; evidence: [DSDA archive page](https://dsdarchive.com/wads/dbp_66_rc3) (demos recorded on DSDA-Doom) |
| 67 | [Marble Hill](https://doomer.boards.net/thread/3642/marble-hill) | 2024-08-07 | 20 | doom2 | limit-removing, cl2 (stated in the release post) | — (not published there) | [dsdarchive](https://dsdarchive.com/files/wads/doom2/5084/DBP67_RC3.zip); Dropbox, TSPG (Zandronum) | Playable — recipe `marble-hill`; evidence: [DSDA archive page](https://dsdarchive.com/wads/dbp67_rc3) (demos recorded on DSDA-Doom) |
| 68 | [Outposts of the Wasteland](https://doomer.boards.net/thread/3754/dbp68-outposts-wasteland-rc1-cl2) | 2024-10-04 | 19 | doom2 | limit-removing, cl2 (stated in the release post) | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp68_outposts-of-the-wasteland-rc1-4.zip); Dropbox, TSPG (Zandronum) | Playable — recipe `outposts-of-the-wasteland`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 69 | [Caustic Industry](https://doomer.boards.net/thread/3856/dbp69-caustic-industry-cl2) | 2024-12-16 | 20 | doom2 | limit-removing, cl2 (stated) | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp69_caustic-industry-v2.zip); Google Drive (not allowlisted), TSPG (Zandronum) | Playable — recipe `caustic-industry`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 70 | [Forgotten Psychadelics](https://doomer.boards.net/thread/3898/dbp70-forgotten-psychedelics) | 2025-02-12 | 20 | doom2 | limit-removing, cl2 (stated in the release post) | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp70_forgotten-psychedelics.zip); Dropbox | Playable — recipe `forgotten-psychedelics`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 71 | [Grief Arbiter](https://doomer.boards.net/thread/3908/grief-arbiter) | 2025-03-01 | 8 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp71_grief-arbiter.zip); Dropbox, Google Drive (not allowlisted), TSPG (Zandronum) | Playable — recipe `grief-arbiter`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 72 | [Wretched Morrow](https://doomer.boards.net/thread/3931/wretched-morrow) | 2025-04-01 | 7 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp72_wretched-morrow.zip); Dropbox, ModDB page, TSPG (Zandronum) | Playable — recipe `wretched-morrow`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 73 | [Narcoleptic Strata](https://doomer.boards.net/thread/4010/narcoleptic-strata) | 2025-06-01 | 14 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp73_narcoleptic-strata-v3.zip); Dropbox, Google Drive (not allowlisted), ModDB page, TSPG (Zandronum), archive.org copy | Playable — recipe `narcoleptic-strata`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 74 | [Phantom Processing](https://doomer.boards.net/thread/4043/phantom-processing) | 2025-07-02 | 8 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp74_phantom-processing_v2.zip); Dropbox, ModDB page, TSPG (Zandronum) | Playable — recipe `phantom-processing`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 75 | [Patagonian Pilgrimage](https://doomer.boards.net/thread/4082/patagonian-pilgrimage-dbp75) | 2025-08-01 | 16 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp75_patagonian-pilgrimage.zip); Dropbox | Playable — recipe `patagonian-pilgrimage`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 76 | [Stagnant Weather](https://doomer.boards.net/thread/4139/stagnant-weather-dbp76) | 2025-09-16 | 14 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp76_stagnant-weather-v1-1.zip); Dropbox | Playable — recipe `stagnant-weather`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |
| 77 | [The Economy of the Dead](https://doomer.boards.net/thread/4191/economy-dead-dbp77) | 2025-11-16 | 9 | doom2 | limit-removing, cl2 | — (not published there) | [archive.org copy](https://ia600903.us.archive.org/33/items/dbp-wad-archive/dbp77_economy-of-the-dead-v1.zip); Dropbox | Playable — recipe `the-economy-of-the-dead`; evidence: [archive.org item](https://archive.org/details/dbp-wad-archive) |

## Per-project notes

- **DBP33 — Rainy Days in Casablanca**: the only Boom-format project (`Advanced engine needed:
  Boom`, `Tested With: DSDA-Doom and GZDoom`). Recipe uses `family = "boom"`, `complevel = 9`.
- **DBP42 — Slime on Earth**: the only MBF21 project (`Advanced engine needed:
  Limit-removing/MBF21`). Chex-themed content on the Doom II IWAD. Recipe uses
  `family = "mbf21"`, `complevel = "mbf21"`.
- **DBP41 — Hell Revealed III (Pt. 1)**: only published on Doomshack (dead) and the
  `dbp-wad-archive` item; pinned from the latter, size 3,529,660 bytes.
- **DBP60 — Eldritch Voyage** and **DBP69 — Caustic Industry**: published via Google Drive
  (not allowlisted) and the `dbp-wad-archive` item; pinned from the item (6,032,738 and
  24,456,096 bytes).
- **DBP32 — Tomatomania** and **DBP70 — Forgotten Psychedelics**: bare `.wad` downloads (no
  archive, no bundled text file), so `kind = "wad"` with a single member.
- **DBP25 — Dead, But Dreaming**: `Advanced engine needed: UMAPINFO compatible/Limit Removing`;
  maps are `1–11, 33`, so the recipe notes the MAP33 slot.
- **DBP48 — AvPvTvJVvHvSS**: the readme documents a GZDoom original version and a
  limit-removing version; the catalog pins the limit-removing package, which also ships
  `DBP_48_MAPINFO.wad` (not loaded: it only carries map names for MAPINFO-capable ports).
- **DBP47 — Dreamcatcher Apparatus**: the readme names `DBP47_gfx.wad` as part of the release,
  so the recipe loads `DBP47.wad` + `DBP47_gfx.wad`. `DBP_47_full.wad` (merged copy),
  `dbp_47_wide.wad` (NightFright widescreen status bar) and `dbp47_gzdoom_strict_bugfix.wad`
  are deliberately not loaded.
- **DBP49 — Mausoleum Nefarium**: `DBP_49_gfx.wad` is an optional Pain Elemental/Lost Soul
  graphics replacement, offered as an optional component (off by default).
- **DBP03, DBP04, DBP09, DBP15, DBP16, DBP17, DBP18, DBP21, DBP24, DBP28, DBP31, DBP32, DBP70**:
  no explicit `cl2` line in the readme (or no readme). Complevel 2 is **Proposed** from the
  limit-removing target, not **Confirmed**.

## Screenshots (spec 023/035)

67 of the 76 new recipes carry one bundled in-game screenshot from the project's release thread
(JPEG, quality 60, 640×360…1280×800, ≤ 192 KiB), with source URL and credit in the recipe's
`SOURCES.toml`. Nine projects have **no screenshot** because the release thread only posts a
title screen, a logo or a movie still: **DBP08, DBP24, DBP31, DBP32, DBP41, DBP44, DBP55,
DBP63, DBP69**. Five are multi-view collages posted by the authors (DBP19, DBP40, DBP43, DBP65)
or were only reachable through a Wayback copy of a dead image host (DBP59, DBP60, DBP61, DBP64);
both facts are recorded in the matching `SOURCES.toml`.

## Left open

- **Dropbox redirect policy** (D-028): see the blocker above. The 21 Dropbox-only projects are
  pinned from archive.org for now; a `*.dl.dropboxusercontent.com` rule in the downloader would
  restore the intended source and is much faster than archive.org (~50 KB/s from this host).
- **DBP78 "Ghost Riders of Christmas Past"** (RC1, no catalog card) and any later project: a
  follow-up inventory.
- **Deck validation** of every recipe: IWAD detection, complevel, load order, gameplay and
  audio are all still **Needs Deck Validation**.
- **Screenshot quality on the Deck panel**: only compression-level review was possible here.

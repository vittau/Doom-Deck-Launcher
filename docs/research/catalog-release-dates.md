# Catalog original-release dates (spec 090)

**Research date:** 2026-10-22 · **Scope:** every recipe shipped in this integration:
118 curated PWAD entries, including the owner-expanded additions from specs 087–094.
This is the evidence behind the optional `original_release` field used for chronological
Discovery, Favorites and Library lists.

Labels: **Confirmed** = quoted from the release/readme, a dated release post, or cited project
research; **Proposed** = a DDL convention, not an upstream fact; **Needs Deck Validation** =
gameplay/runtime behaviour, never claimed here.

## Policy

- **The field is the first public release**, not the free-form `release` version label and not a
  later `/idgames`, repackaging or bug-fix upload. Example: Alien Vendetta ships the 2002
  Compet-N edition but its first public release is 2001-12-24; Eviternity ships the final v1.0
  (2019-02-10) while the announcement/opening post dates the public release 2018-12-10.
- **Precision is exactly what the source supports.** A readme that names a full date gives a
  day; a source that only establishes a year gives a year. No date is padded with January or a
  first-of-month day. If no first-release date is evidenced, the field is omitted (unknown).
- **Coarser dates do not claim an early slot.** Sorting uses `(year, month or 13, day or 32)`:
  a year-only entry follows that year's dated entries, and a month-only entry follows that
  month's dated entries. Unknown entries follow every dated entry. Equal dates tie by
  case-insensitive title, then recipe id.
- **Version labels are never parsed.** `release` stays the human version identity; two recipes
  whose labels would sort backwards (e.g. `9.0` before `10.0`) still sort by date.
- **Ambiguous numeric dates** are resolved against ZIP member timestamps and sibling documents,
  and the literal readme string is preserved in the recipe's inspected evidence:
  Stardate 20X6 `06/02/2013` = 2013-06-02 (text member 2013-06-02 17:50:00, WAD 2013-05-31);
  Abandon `05/07/2023` = 2023-05-07 (its text member is dated 2023-05-07); Going Down's
  `02/11/14` is day-first (idgames 2014-11-02); Overboard and Headless Chicken use the same
  day-first convention, proved by the inline `19/01/2026` in the latter.
- **Later updates kept separate:** Doom the Way id Did (2011-12-10 first; inspected v1.1a
  2012-02-05), No End in Sight (2016-11-28 first; inspected v1.4 2018-01-29), Deathless
  (2018-11-09 first; inspected v1.1 2020-05-25), Sunder (2009-04-20 first; inspected V.2407),
  REKKR (2018-07-09 first; inspected 1.17), Scythe II (2005-06-06 first; inspected Version 2),
  TNT: Revilution (2017-05-29 first; inspected 1.20), Going Down: Turbo (2024-11-05 first;
  inspected 2025-06-23 bug-fix), Haste (2021-09-22 first; 2022 archive), Headless Chicken
  (2023-12-09 RC1; 2026 RC14), Micro Slaughter (2021-01-27 first; 2026 v1b replacement),
  No Rest for the Living (2010-05-26 XBLA; later BFG/Unity/KEX variants), Swim With the Whales
  (2013-11-20 thread; readme 2013-12-02; archive 2013-12-21), and Ozonia (readme 2021-12-23;
  archive packaged 2022-10-30/11-04).
- **Community-project sources.** The 74 Doomer Boards entries use the release thread's first
  post date from `docs/research/doomer-boards-projects.md` (its inventory column), which is the
  project's public release rather than the `/idgames` upload.
- **Gate.** Every dated recipe carries `original_release`; pre-0.0.12 readers refuse unknown
  recipe fields, so the bundle minimum rises to 0.0.12 together with the app version (D-038).

## Per-recipe dates

The comment above each recipe field carries the same source in short form.

| Recipe | Original release | Precision | Evidence |
| --- | --- | --- | --- |
| `a-doomer-boards-christmas-carol` | 2019-12-24 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `abandon` | 2020 | year | the 2020 Cacowards Top Ten page 1 covers the original 2020 release; the pinned 2023 archive text is a later update, and no day is evidenced. See docs/research/catalog-release-dates.md. |
| `after-the-fall` | 2019-07-30 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `alien-bastards` | 2019-07-04 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `alien-vendetta` | 2001-12-24 | day | reference-alien-vendetta.md §1 (2001-12-24; the pinned archive is the 2002 Compet-N edition). |
| `alone` | 2019-10-31 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `ancient-aliens` | 2016-05-08 | day | reference-ancient-aliens.md §1 (v1, 2016-05-08; the pinned archive is v1.2). |
| `antaresian-reliquary` | 2020-08-24 | day | reference-antaresian-reliquary.md §1 (2020-08-24). |
| `aquatic-wonder` | 2021-06-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `arrival` | 2021-04-16 | day | reference-087-c.md §8: arrival.txt Release date 4/16/2021; the 2022-10-07 archive is later. |
| `auger-zenith` | 2021-07-26 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `augustland-barrens` | 2023-02-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `avpvtvjvvhvss` | 2022-06-04 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `back-to-saturn-x-e1` | 2013 | year | 2013 Cacowards Top Ten for the episode's original public release; the pinned 1.1.6 archive is dated 2020-08-13 and the exact first day is not evidenced (docs/research/catalog-release-dates.md). |
| `back-to-saturn-x-e2` | 2014 | year | 2014 Cacowards Top Ten for the original public release; the pinned 1.0.1 archive is dated 2020-08-13 and the exact first day is not evidenced (docs/research/catalog-release-dates.md). |
| `biotech-is-godzilla` | 2020-03-31 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `carnage-oasis` | 2021-09-02 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `caustic-industry` | 2024-12-16 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `christmas-in-inferno` | 2021-12-24 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `chronicles-of-ghost-town` | 2021-08-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `circle-of-caina` | 2018-04-30 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `coffin-curse` | 2018-08-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `cybs-freaky-colonoscopy` | 2019-10-05 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `dead-but-dreaming` | 2020-07-02 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `deadly-ritual` | 2022-09-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `deathless` | 2018-11-09 | day | reference-087-b.md original-date table: author release post November 9, 2018; the pinned 1.1 is the 2020-05-25 update. |
| `doom-ii-the-way-id-did` | 2013-11-07 | day | reference-087-b.md original-date table: D2TWID.txt Release date 11/7/13 (finalized 12/22/13). |
| `doom-the-way-id-did` | 2011-12-10 | day | reference-087-b.md original-date table: DTWID.txt Release date 12/10/11; the pinned v1.1a is the 2012-02-05 revision. |
| `dreamcatcher-apparatus` | 2022-05-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `dungeons-and-demons` | 2020-02-02 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `earth-ends` | 2019-04-05 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `eldritch-voyage` | 2023-08-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `emerald-city` | 2022-08-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `evil-egypt` | 2020-05-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `eviternity-ii` | 2023-12-10 | day | reference-eviternity-ii.md §1, initial release 2023-12-10 (pinned final 2024-09-30). |
| `eviternity` | 2018-12-10 | day | reference-eviternity.md §1, release-thread opening post 2018-12-10 (pinned final v1.0 2019-02-10). |
| `fear-and-loathing` | 2020-10-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `forest-swords` | 2018-05-31 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `forgotten-psychedelics` | 2025-02-12 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `fuck-global-warming-this-is-hell` | 2019-09-03 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `funnelcake-apparitions` | 2021-10-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `going-down-turbo` | 2024-11-05 | day | reference-087-a.md original-date table: DoomWiki first released November 5, 2024; the pinned archive is the 2025-06-23 bug-fix. |
| `going-down` | 2014-09-14 | day | reference-going-down.md §1, first complete release 2014-09-14 (pinned third bug-fix 2015-08-21). |
| `grief-arbiter` | 2025-03-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `haste` | 2021-09-22 | day | docs/research/haste.md §original public date: Haste.txt 22/09/2021, day-first; the 2022-01-08 archive is later. |
| `haunting-hollow` | 2023-10-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `havoc-in-creation` | 2022-09-30 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `headless-chicken` | 2023-12-09 | day | reference-087-c.md §8: RC1 9/12/2023 (day-first proven by 19/01/2026); the 2026 archive is RC14. |
| `hell-revealed-3-pt-1` | 2021-11-03 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `immortal-warfare` | 2023-05-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `into-the-storm` | 2019-06-04 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `japanese-community-project` | 2016-07-24 | day | JPCP.txt final v2.3 release log; docs/research/reference-087-d.md §2. |
| `japanese-valentines` | 2022-02-07 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `jolly-junkyard` | 2022-12-24 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `legend-of-the-hidden-tech` | 2019-03-03 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `lilywhite-lilith` | 2019-05-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `luminous-gloom` | 2021-04-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `lunar-strain` | 2024-06-20 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `lunatic` | 2011-08-11 | day | reference-087-c.md §8: Lunatic.txt Release date August 11, 2011. |
| `marble-hill` | 2024-08-07 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `mausoleum-nefarium` | 2022-07-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `memento-mori-ii` | 1996-07-27 | day | reference-memento-mori.md §1 (1996-07-27). |
| `memento-mori` | 1995-12-10 | day | reference-memento-mori.md §1 (1995-12-10; the pinned archive is the February 1996 update). |
| `micro-slaughter-community-project` | 2021-01-27 | day | reference-087-c.md §8: MSCP_v1b.txt Release date 27th Jan 2021; the 2026 archive is the v1b replacement. |
| `mindblood-genesis` | 2019-02-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `monuments-of-mars-2` | 2022-04-02 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `monuments-of-mars` | 2018-03-31 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `morbid-autumn` | 2020-11-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `narcoleptic-strata` | 2025-06-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `no-end-in-sight` | 2016-11-28 | day | reference-087-b.md original-date table: NEIS.txt Release date 11/28/16; the pinned v1.4 is the 2018-01-29 update. |
| `no-rest-for-the-living` | 2010-05-26 | day | spec 094 and docs/research/no-rest-for-the-living.md §4: Doom II Xbox Live Arcade release 2010-05-26; later BFG, Unity and KEX releases are variants. |
| `occult-secrets-of-the-third-reich-on-mars` | 2020-03-04 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `outposts-of-the-wasteland` | 2024-10-04 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `overboard` | 2022-08-04 | day | reference-087-c.md §8: overboard.txt 04/08/22 (day-first, author convention); the 2022-09-27 archive is a multiplayer update. |
| `ozonia` | 2021-12-23 | day | ozonia.txt release date; docs/research/reference-087-d.md §4. |
| `patagonian-pilgrimage` | 2025-08-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `phantom-processing` | 2025-07-02 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `plutonia-2` | 2008-12-31 | day | reference-087-a.md original-date table: PL2.TXT Version 1.0, December 31, 2008. |
| `plutonian-sunrise` | 2022-11-02 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `rainy-days-in-casablanca` | 2021-03-02 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `rekkr` | 2018-07-09 | day | reference-087-b.md original-date table: REKKR.txt Release date 7/9/2018; the pinned 1.17 update is later. |
| `sanguine-holy-land` | 2024-04-13 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `santas-outback-bender` | 2020-12-24 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `scythe-ii` | 2005-06-06 | day | reference-087-a.md original-date table: Erik Alm's release thread dated 6 June 2005; the pinned archive is the 2009 Version 2 re-release. |
| `scythe` | 2003-04-08 | day | reference-scythe.md §1 (2003-04-08; idgames entry 2003-04-10). |
| `shatter-realm` | 2023-03-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `sigil-ii` | 2023-12-10 | day | reference-sigil.md SIGIL II §1 (v1.0, 2023-12-10). |
| `sigil` | 2019-05-01 | day | reference-sigil.md §1 version history (1.0, first free release 2019-05-01). |
| `slime-on-earth` | 2021-12-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `spaceballs-the-quest-for-more-dbp-maps` | 2020-05-31 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `speed-of-doom` | 2010-03-28 | day | reference-087-a.md original-date table: SODFINAL.txt Release date 03/28/10. |
| `stagnant-weather` | 2025-09-16 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `stardate-20x6` | 2013-06-02 | day | stardate20x6.txt dated 06/02/2013, normalized to 2013-06-02; docs/research/reference-087-d.md §3. |
| `steamy-bathhouse` | 2022-12-03 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `stroggmans-tundra` | 2021-05-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `sunder` | 2009-04-20 | day | reference-087-b.md original-date table: DoomWiki first release April 20, 2009; the pinned V.2407 is a later build. |
| `sunlust` | 2015-08-07 | day | reference-sunlust.md §1 (2015-08-07). |
| `swim-with-the-whales` | 2013-11-20 | day | docs/research/swim-with-the-whales.md §5: release-thread first post 2013-11-20; the readme (2013-12-02) and archive (2013-12-21) are later. |
| `tempest-enterprise` | 2023-09-02 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `ten-day-vacation` | 2020-09-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `terminal-voltage` | 2023-12-07 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `the-10x10-project` | 2022-06-15 | day | reference-087-a.md original-date table: 10x10.txt Release date 06/15/2022. |
| `the-city-of-damned-children` | 2020-07-31 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `the-economy-of-the-dead` | 2025-11-16 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `the-magnificent-five` | 2020-12-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `the-merry-christmassy-doom-project` | 2018-12-24 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `the-vast-silence` | 2024-02-04 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `tnt-revilution` | 2017-05-29 | day | reference-087-a.md original-date table: TNTR.txt Release date 05/29/2017; the pinned 1.20 update is 2017-07-08. |
| `tomatomania` | 2021-02-24 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `uac-ultra` | 2010-03-29 | day | uacultra.txt; docs/research/reference-087-d.md §1. |
| `umbral-platinum` | 2019-12-04 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `valiant` | 2015-02-07 | day | reference-valiant.md §1 (2015-02-07; the pinned archive is the 2015-05 build). |
| `vanguard` | 2011-05-11 | day | reference-087-c.md §8: Vanguard.txt Release date May 11 2011. |
| `vicarious-reality` | 2018-09-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `vrack-botanicals` | 2022-03-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `wretched-morrow` | 2025-04-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `xenomorph-base` | 2018-07-01 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |
| `zeppelin-armada` | 2023-07-03 | day | release-thread first post, docs/research/doomer-boards-projects.md inventory. |

## Counts

- **118 recipes**, every one with a date: 115 day-precision, 0 month-precision, 3 year-precision.
- No shipped recipe relies on an unstated date. The only coarser-than-day entries are the ones
  whose source itself stops at a year/month (see the table).

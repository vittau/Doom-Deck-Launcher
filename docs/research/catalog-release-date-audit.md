# Original-release date audit — 16 non-DBP catalog titles (spec 090)

**Date:** 2026-10-22. **Scope:** the 16 original (non-DBP) recipes named for this audit.
**Purpose:** independent check of the `original_release` values currently drafted on
`verify/087-pwads` (commit `9d48e1f`), before D finalises the chronology fields.

**Rules applied.** A date must be the **first public availability** of the mapset, not the
Cacowards year, the internal readme build date, the later `/idgames` upload, or a
re-release. Day precision is used only when a dated post/page states the day; otherwise
year (or month). Public release candidates/betas are called out as such. "Retained" means the
current draft value is supported; "corrected" means it is not, even when the draft date was
a real later event.

## 1. Results

| Recipe | Current draft | Recommended | Precision | Source class and evidence | Disposition |
| --- | --- | --- | --- | --- | --- |
| `alien-vendetta` | 2001-12-24 | **2001-12-24** | day | **Primary (author site).** The authors' legacy AV news dated `24.12.2002` says: "Well, exactly one year after its original release, we put up a refined version of the wad." DoomWiki's infobox also gives `Year 2001`. | retained |
| `ancient-aliens` | 2016-05-08 | **2016-05-08** | day | **Secondary (DoomWiki, explicit).** "The first release candidate of the WAD was released on 8 May, 2016"; final to `/idgames` 19 July 2016. The recipe ships v1.2 (2023). | retained (public RC) |
| `antaresian-reliquary` | 2020-08-24 | **2020-05-17** | day | **Primary (author thread).** antares031's release thread: "By antares031, May 17, 2020 … Posted May 17, 2020". DoomWiki: first release candidate 17 May 2020, `/idgames` 6 June 2020. 2020-08-24 is the later DECORATE-only update. | corrected |
| `back-to-saturn-x-e1` | 2013 | **2012-12-10** | day | **Primary (author thread).** esselfortium: "By esselfortium, December 10, 2012 … Posted December 10, 2012" — "In celebration of Doom's birthday…". 2013 is only the Cacoward year. | corrected |
| `back-to-saturn-x-e2` | 2014 | **2014-09-01** | day | **Primary (author thread).** esselfortium: "By esselfortium, September 1, 2014 … now officially in public beta released!" DoomWiki agrees; final v1.0.1 came 2020-08-13. | corrected (public beta) |
| `eviternity` | 2018-12-10 | **2018-12-10** | day | **Secondary (DoomWiki, explicit).** "It was released on December 10, 2018, the 25th anniversary of Doom's release"; existing research records the author's opening post dated 2018-12-10. | retained |
| `eviternity-ii` | 2023-12-10 | **2023-12-10** | day | **Secondary (DoomWiki, explicit).** "It was initially released on December 10, 2023, the 30th anniversary of Doom's release"; final version 2024-09-30. | retained |
| `going-down` | 2014-09-14 | **2013-10-22** | day | **Secondary (DoomWiki, explicit).** "The first incomplete version, containing only 11 levels, was released on October 22, 2013"; 2014-04-03 added maps 12–20/31/32; first full version 2014-09-14; `/idgames` 2014-11-02; third bug-fix 2015-08-21. | corrected (earliest public, incomplete) |
| `hell-revealed-3-pt-1` | 2021-11-03 | **2021-11-03** | day | **Primary (project thread).** DBP41 thread first post by matador, "Nov 3, 2021 21:46:32 GMT -5", with the download and map list. It is a DBP release with no `/idgames` entry. | retained |
| `memento-mori` | 1995-12-10 | **1995-12-10** | day | **Secondary (DoomWiki, explicit).** "It was initially released on December 10, 1995, and saw an updated release in February 1996." | retained |
| `memento-mori-ii` | 1996-07-27 | **1996-07-27** | day | **Secondary (DoomWiki, explicit).** "Released on July 27, 1996"; the readme/`MM2.TXT` carries the same date. | retained |
| `scythe` | 2003-04-08 | **2003-04-08** | day | **Secondary (DoomWiki, explicit).** "released by Erik Alm on April 8, 2003"; the `/idgames` listing is 2003-04-10 (later upload). The readme's date field is blank. | retained |
| `sigil` | 2019-05-01 | **2019-05-31** | day | **Primary (author blog).** John Romero, "SIGIL Update", 2019-12-02: "I uploaded the free version on May 31, 2019." `romero.com/sigil-1` shows the planned "mid-May 2019" release and a May 11 delay update, matching the late-May upload. 2019-05-01 is an internal readme/version date, not public availability. | corrected |
| `sigil-ii` | 2023-12-10 | **2023-12-10** | day | **Secondary (DoomWiki, explicit).** "released December 10, 2023" (Doom's 30th anniversary); existing romero.com research records the same free-release date. | retained |
| `sunlust` | 2015-08-07 | **2015-06-13** | day | **Secondary (DoomWiki, explicit).** "the first public beta was released on June 13, 2015"; final version to `/idgames` 2015-08-09. 2015-08-07 is the readme/final date, not first availability. | corrected (public beta) |
| `valiant` | 2015-02-07 | **2015-02-07** | day | **Primary (author readme).** `Valiant.txt`: `Release date : February 7th, 2015`. Later builds: 2015-04, 2015-05-16 (the recipe ships the May build). | retained |

## 2. Corrections D should apply

1. `back-to-saturn-x-e1`: `{ year = 2013 }` → `{ year = 2012, month = 12, day = 10 }`
2. `back-to-saturn-x-e2`: `{ year = 2014 }` → `{ year = 2014, month = 9, day = 1 }`
3. `sigil`: `{ year = 2019, month = 5, day = 1 }` → `{ year = 2019, month = 5, day = 31 }`
4. `going-down`: `{ year = 2014, month = 9, day = 14 }` → `{ year = 2013, month = 10, day = 22 }`
5. `sunlust`: `{ year = 2015, month = 8, day = 7 }` → `{ year = 2015, month = 6, day = 13 }`
6. `antaresian-reliquary`: `{ year = 2020, month = 8, day = 24 }` → `{ year = 2020, month = 5, day = 17 }`

No recipe file is edited by this audit; D owns the fields.

## 3. Precision and date-class cautions

- **BTSX E1** is the clearest error: 2013 is the Cacoward year; the episode was public on
  2012-12-10 (author thread), with later updates (2013-08-01, 2017-02-27) and the shipped
  1.1.6 build (2020-08-13).
- **SIGIL** must use the free edition's public upload (2019-05-31), not the readme's
  internal 2019-05-01 date. The 2019-12-02 blog post is the author's retrospective, not the
  release date. If a future entry ever scopes the paid Buckethead/USB edition instead,
  that edition needs its own date; the catalog entry is the free one.
- **Going Down** and **Sunlust** have public pre-final releases. The recommended dates are
  the earliest public availability; the first *complete* Going Down release (2014-09-14)
  and the Sunlust final `/idgames` release (2015-08-09) are later events and should not be
  used for the chronology field. The same distinction applies to **BTSX E2** (public beta
  2014-09-01; final 2020-08-14) and **Ancient Aliens** (public RC 2016-05-08; final 2016-07-19).
- **Alien Vendetta** keeps day precision only because the author site anchors the original
  release one year before its dated 24.12.2002 update; the 2001 year alone is what DoomWiki
  states. If stricter evidence is required, `{ year = 2001 }` is the fallback, but the
  24.12.2001 reading is directly supported by the authors' "exactly one year after its
  original release" note.
- **Antaresian Reliquary** 2020-08-24 is a final update date; the author's public thread
  (2020-05-17) is the first availability.
- **Hell Revealed III (Pt. 1)** is DBP41; its date comes from the project thread, not
  `/idgames` (none). It is included here only because the audit list names it.

## 4. Sources and snapshots

All were read on 2026-10-22. DoomWiki blocks direct automated requests (HTTP 403), so its
pages were read through the Wayback Machine with the shown capture prefix; no anti-bot
bypass was used.

| Source | URL / snapshot | Used for |
| --- | --- | --- |
| Alien Vendetta legacy news | <https://doom2.net/av/old/index.html> (live) | original release one year before 24.12.2002 → 2001-12-24 |
| DoomWiki Alien Vendetta | <https://web.archive.org/web/2024id_/https://doomwiki.org/wiki/Alien_Vendetta> | `Year 2001`; 2002 Compet-N update |
| DoomWiki Ancient Aliens | <https://web.archive.org/web/2024id_/https://doomwiki.org/wiki/Ancient_Aliens> | public RC 2016-05-08; final 2016-07-19 |
| Author thread, Antaresian Reliquary | <https://web.archive.org/web/2024id_/https://www.doomworld.com/vb/thread/114478> | first post 2020-05-17 |
| DoomWiki Antaresian Reliquary | <https://web.archive.org/web/2026id_/https://doomwiki.org/wiki/Antaresian_Reliquary> | RC 2020-05-17; `/idgames` 2020-06-06 |
| Author thread, BTSX E1 | <https://web.archive.org/web/2024id_/https://www.doomworld.com/vb/thread/62529> | first post 2012-12-10 |
| DoomWiki Back to Saturn X | <https://web.archive.org/web/2026id_/https://doomwiki.org/wiki/Back_to_Saturn_X> | E1 2012-12-10; E2 public beta 2014-09-01; finals |
| Author thread, BTSX E2 | <https://web.archive.org/web/2024id_/https://www.doomworld.com/vb/thread/69960> | first post 2014-09-01, "public beta released" |
| DoomWiki Eviternity | <https://web.archive.org/web/2026id_/https://doomwiki.org/wiki/Eviternity> | released 2018-12-10 |
| DoomWiki Eviternity II | <https://web.archive.org/web/2026id_/https://doomwiki.org/wiki/Eviternity_II> | initial release 2023-12-10; final 2024-09-30 |
| DoomWiki Going Down | <https://web.archive.org/web/2026id_/https://doomwiki.org/wiki/Going_Down> | 2013-10-22, 2014-04-03, 2014-09-14, 2014-11-02 |
| DBP41 thread | <https://doomer.boards.net/thread/2328/dbp41-revealed-iii-pt-1> (live) | first post 2021-11-03 |
| DoomWiki Memento Mori | <https://web.archive.org/web/2024id_/https://doomwiki.org/wiki/Memento_Mori> | 1995-12-10; Feb 1996 update |
| DoomWiki Memento Mori II | <https://web.archive.org/web/2024id_/https://doomwiki.org/wiki/Memento_Mori_II> | 1996-07-27 |
| DoomWiki Scythe | <https://web.archive.org/web/2026id_/https://doomwiki.org/wiki/Scythe> | 2003-04-08; `/idgames` 2003-04-10 |
| John Romero blog (SIGIL Update) | <https://web.archive.org/web/2024id_/https://www.johnromero.com/> (archived post text) | "I uploaded the free version on May 31, 2019" |
| romero.com SIGIL updates | <https://romero.com/sigil-1> (live) | planned mid-May 2019; 2019-05-11 delay update |
| DoomWiki SIGIL II | <https://web.archive.org/web/2024id_/https://doomwiki.org/wiki/SIGIL_II> | released 2023-12-10 |
| DoomWiki Sunlust | <https://web.archive.org/web/2024id_/https://doomwiki.org/wiki/Sunlust> | first public beta 2015-06-13; final 2015-08-09 |
| DoomWiki Valiant | <https://web.archive.org/web/2024id_/https://doomwiki.org/wiki/Valiant> | 2015 build history; readme gives 2015-02-07 |
| Author readme `Valiant.txt` | existing pinned research (`docs/research/reference-valiant.md` §1) | `Release date : February 7th, 2015` |

## 5. Notes for the report

- Current draft values were read from `catalog/*.toml` on `verify/087-pwads` (commit
  `9d48e1f`); no recipe was modified by this audit.
- Six values need correction (section 2); the other ten are retained. The largest error is
  BTSX E1 (award year vs release year); SIGIL is the second (internal date vs public upload).
- Where a later dated event exists (final version, `/idgames` upload, bug-fix build), it is
  recorded above only to show what the draft date actually referred to.

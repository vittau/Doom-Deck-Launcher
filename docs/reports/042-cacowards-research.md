# 042 — Research report: Cacowards inventory

**Spec:** `docs/specs/042-cacowards-research.md` · **Branch:** `vittau/cacowards-research` · **Date:** 2026-10-19

## What changed

- Added `docs/research/cacowards.md`: the full 2004–2025 inventory (380 Top Ten/Twelve and runner-up entries), the 69 named special-award releases that are mapsets, source links with capture dates, `/idgames` paths and sizes, IWAD/port evidence, a DSDA-Doom verdict for each entry, and the current DDL recipe/tag mapping.
- Added this report: proposed recipe waves, tag-citation gaps, and open questions.
- No Rust code, catalog recipe, decision, status or journal file was changed. No new dependency was added.

## How the acceptance criteria were verified

- **A1 (R1–R3).** Every `best*`, `runnersup`, `others` and `multi` page of the 22 editions was fetched as a raw Wayback `id_` snapshot and parsed; the per-year tables link each page and carry its capture timestamp. `/idgames` paths/sizes come from `https://www.gamers.org/pub/idgames/ls-laR.gz` (fetched 2026-10-19); `.txt` sidecars were read for `Title`, `Game`/`Base` and `Advanced engine needed`. All claims carry Confirmed/Proposed/Needs Deck Validation. No archive was executed or extracted.
- **A2 (R4 and gaps).** The waves below and the tag-gap section are derived from that inventory. The scope finding is that the Cacowards start in 2004, not 1994; see the research file.

## R4 — Proposed recipe waves

Candidate pool: **136** playable, not-yet-cataloged entries with a verified `/idgames` path and a single-player/co-op purpose. Ordered by evidence strength first (text file engine line, then Cacowards page line), then year and archive size, in groups of ~10. All downloads are on `www.gamers.org`, already in `catalog/hosts.toml`; **no new host allowlist is needed for these waves**.

### Wave 1 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2004 | Top Ten p2 | Tremor | [`levels/doom2/Ports/s-u/tremor.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/tremor.zip) | 872 kB | Doom II | Boom-compatible (tested with prboom, ZDoom and Edge) |
| 2004 | Runner-up | Deus Vult | [`levels/doom2/Ports/d-f/dv.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dv.zip) | 6.3 MB | Doom II | A limit removing source port for Map 01-04, |
| 2005 | Top Ten p1 | Scythe 2 | [`levels/doom2/Ports/megawads/scythe2.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/scythe2.zip) | 10.6 MB | Doom II | Any limit removing port. Tested with prboom and zdoom. |
| 2006 | Top Ten p1 | IC2005 | [`levels/doom2/Ports/g-i/ic2005.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/g-i/ic2005.zip) | 1.6 MB | Doom II | Boom compatible (with increased SEG limit) |
| 2007 | Top Ten p2 | Sine Die | [`levels/doom2/Ports/s-u/sinedie.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/sinedie.zip) | 2.5 MB | Doom II | Boom, see "Tested With" |
| 2007 | Top Ten p3 | Community Chest 3 | [`levels/doom2/Ports/megawads/cchest3.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/cchest3.zip) | 12.4 MB | Doom II | Boom (and any Boom-compatible engine) |
| 2009 | Top Ten p2 | Mapgame | [`levels/doom/Ports/m-o/mapgame.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/m-o/mapgame.zip) | 1.8 MB | Ultimate Doom | Boom compatible (runs well on zdoom, prboom, and |
| 2010 | Top Ten p3 | Drip Feed | [`levels/doom2/Ports/d-f/dripfeed.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dripfeed.zip) | 676 kB | Doom II | BOOM COMPATIBLE |
| 2010 | Top Ten p2 | UAC Ultra | [`levels/doom2/Ports/s-u/uacultra.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/uacultra.zip) | 2.3 MB | Doom II | Boom-compatible. PrBoom-Plus recommended. |
| 2010 | Runner-up | Beyond Revival | [`levels/doom2/Ports/p-r/revival.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/p-r/revival.zip) | 4.2 MB | Doom II | Boom compatible |

### Wave 2 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2011 | Top Ten p1 | Lunatic | [`levels/doom2/Ports/j-l/lunatic.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/j-l/lunatic.zip) | 4.1 MB | Doom II | Boom compatible, limit removing ports |
| 2011 | Top Ten p2 | Vanguard | [`levels/doom2/Ports/v-z/vanguard.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/v-z/vanguard.zip) | 9.9 MB | Doom II | Boom compatible, limit removing ports |
| 2012 | Top Ten p3 | Combat Shock 2 | [`levels/doom2/Ports/a-c/c-shock2.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/c-shock2.zip) | 8.7 MB | Doom II | Boom |
| 2013 | Top Ten p2 | Kuchitsu | [`levels/doom2/Ports/j-l/kuchitsu.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/j-l/kuchitsu.zip) | 620 kB | Doom II | vanilla-compatible limit removing port |
| 2013 | Top Ten p3 | Forsaken Overlook | [`levels/doom2/Ports/d-f/fsknovlk.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/fsknovlk.zip) | 762 kB | Doom II | Boom-compatible, should work with non-MBF ports. |
| 2013 | Top Ten p2 | Fuel Devourer | [`levels/doom2/Ports/d-f/fueldevr.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/fueldevr.zip) | 2.3 MB | Doom II | Any Doom 2 Limit Removing port |
| 2013 | Top Ten p3 | Stardate 20X6 | [`levels/doom2/Ports/s-u/stardate.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/stardate.zip) | 5.0 MB | Doom II | Boom-Compatible (tested in prb+ cl9, gzdoom) |
| 2013 | Runner-up | Hadephobia | [`levels/doom2/Ports/megawads/h_phobia.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/h_phobia.zip) | 7.3 MB | Doom II | PRBoom+ (comp level 9) recommended |
| 2014 | Runner-up | The Wailing Horde | [`levels/doom/Ports/v-z/wailhord.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/v-z/wailhord.zip) | 362 kB | Ultimate Doom | Limit removing |
| 2014 | Top Ten p2 | Monster Hunter Ltd. Part I / Part II | [`levels/doom2/Ports/m-o/mohu1.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/m-o/mohu1.zip) | 2.6 MB | Doom II | Boom |

### Wave 3 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2014 | Runner-up | Bauhaus | [`levels/doom2/Ports/a-c/bauhaus.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/bauhaus.zip) | 3.0 MB | Doom II | Boom port |
| 2014 | Runner-up | Mayhem 2048 | [`levels/doom2/Ports/megawads/mayhem14.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/mayhem14.zip) | 7.3 MB | Doom II | Boom-compatible source port with suport for sky |
| 2014 | Top Ten p3 | Resurgence | [`levels/doom2/Ports/megawads/resurge.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/resurge.zip) | 13.9 MB | Doom II | Boom Compatible Ports |
| 2014 | Runner-up | Bloody Steel | [`levels/doom2/Ports/a-c/bldstlp.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/bldstlp.zip) | 42.1 MB | Doom II | prboom-plus |
| 2014 | Runner-up | Whitemare 2 | [`levels/doom2/Ports/megawads/white2.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/white2.zip) | 43.5 MB | Doom II | prboom-plus |
| 2015 | Top Ten p3 | Return to Hadron | [`levels/doom/Ports/g-i/hadron.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/g-i/hadron.zip) | 1.0 MB | Ultimate Doom | Limit removing |
| 2015 | Top Ten p2 | Swift Death | [`levels/doom2/Ports/megawads/swideath.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/swideath.zip) | 1.6 MB | Doom II | Boom/PrBoom |
| 2015 | Runner-up | Ol' No Name | [`levels/doom2/Ports/m-o/olnoname.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/m-o/olnoname.zip) | 5.3 MB | Doom II | Boom |
| 2015 | Top Ten p3 | 50 Shades of Graytall | [`levels/doom2/Ports/megawads/50shades.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/50shades.zip) | 5.5 MB | Doom II | "Boom" (PrBoom+ cl9) |
| 2015 | Runner-up | 32in24-14 | [`levels/doom2/Ports/megawads/32in2414.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/32in2414.zip) | 14.2 MB | Doom II | Boom compatible |

### Wave 4 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2016 | Top Ten p1 | Tech Gone Bad | [`levels/doom/Ports/d-f/e1m8b.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/d-f/e1m8b.zip) | 220 kB | Ultimate Doom | Limit-removing source ports |
| 2016 | Top Ten p3 | Alpha Accident: Terra Nova | [`levels/doom/Ports/a-c/aa_e1.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/a-c/aa_e1.zip) | 3.1 MB | Ultimate Doom | Limit-removing port may be needed |
| 2016 | Top Ten p1 | Mutiny | [`levels/doom2/Ports/megawads/mutiny.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/mutiny.zip) | 5.6 MB | Doom II | Limit Removing (Vanilla except with VPOs) |
| 2016 | Top Ten p2 | Comatose | [`levels/doom2/Ports/a-c/comatose.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/comatose.zip) | 7.2 MB | Doom II | prBoom+ (OGG-Music), complevel 2 |
| 2016 | Runner-up | Bloodstain | [`levels/doom2/Ports/megawads/bstain.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/bstain.zip) | 11.7 MB | Doom II | Boom Compatible Ports (-complevel 2 compatible) |
| 2016 | Top Ten p3 | Japanese Community Project | [`levels/doom2/Ports/megawads/jpcp.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/jpcp.zip) | 13.6 MB | Doom II | Limit Removing |
| 2016 | Runner-up | THT: Threnody | [`levels/doom2/Ports/megawads/thtthren.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/thtthren.zip) | 13.6 MB | Doom II | Boom compatible |
| 2017 | Runner-up | Return to Hadron Episode 2 | [`levels/doom/Ports/g-i/hadrone2.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/g-i/hadrone2.zip) | 1.1 MB | Ultimate Doom | Limit removing |
| 2017 | Top Ten p3 | Saturnine Chapel | [`levels/doom2/Ports/s-u/satchap.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/satchap.zip) | 1.9 MB | Doom II | Boom |
| 2017 | Top Ten p1 | No End in Sight | [`levels/doom/Ports/megawads/neis.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/megawads/neis.zip) | 6.4 MB | Ultimate Doom | Limit-removing for E4; E1-E3 are vanilla-compat. |

### Wave 5 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2017 | Top Ten p3 | Legacy of Heroes | [`levels/doom2/Ports/j-l/legher.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/j-l/legher.zip) | 7.2 MB | Doom II | Boom |
| 2017 | Top Ten p3 | Stardate 20x7 | [`levels/doom2/Ports/s-u/sd20x7.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/sd20x7.zip) | 7.9 MB | Doom II | pr-boom+ / zdoom / equivalent |
| 2017 | Top Ten p2 | Brigandine | [`levels/doom2/Ports/a-c/brigandine.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/brigandine.zip) | 8.2 MB | Doom II | Strong limit-removing (>32k seg limit) |
| 2017 | Top Ten p2 | Counterattack | [`levels/doom2/Ports/m-o/mek-catk.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/m-o/mek-catk.zip) | 8.4 MB | Doom II | Boom |
| 2017 | Runner-up | Disjunction | [`levels/doom2/Ports/d-f/disjunct.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/disjunct.zip) | 11.2 MB | Doom II | Boom |
| 2018 | Runner-up | Doom 404 | [`levels/doom2/megawads/doom404.zip`](https://www.gamers.org/pub/idgames/levels/doom2/megawads/doom404.zip) | 1.1 MB | Doom II | None (complevel 2) |
| 2018 | Runner-up | Port Glacia | [`levels/doom2/Ports/p-r/portglacia.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/p-r/portglacia.zip) | 5.1 MB | Doom II | PRBoom+ |
| 2018 | Runner-up | Man on the Moon | [`levels/doom2/Ports/m-o/motm.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/m-o/motm.zip) | 6.4 MB | Doom II | Prboom+ 2.1.5.4 at the very least (it was tested |
| 2018 | Runner-up | Exomoon | [`levels/doom2/Ports/megawads/exomoon.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/exomoon.zip) | 9.8 MB | Doom II | PrBoom -complevel 11 |
| 2018 | Top Ten p3 | Maskim Xul | [`levels/doom2/Ports/m-o/maskimxul.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/m-o/maskimxul.zip) | 11.0 MB | Doom II | Boom |

### Wave 6 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2018 | Top Ten p2 | REKKR | [`levels/doom/megawads/rekkr.zip`](https://www.gamers.org/pub/idgames/levels/doom/megawads/rekkr.zip) | 14.2 MB | Ultimate Doom | None |
| 2018 | Top Ten p2 | Preacher | [`levels/doom2/Ports/p-r/preacher.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/p-r/preacher.zip) | 15.7 MB | Doom II | Boom compatible |
| 2019 | Top Ten p2 | Shotgun Symphony | [`levels/doom/Ports/s-u/sgnsym.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/s-u/sgnsym.zip) | 2.2 MB | Ultimate Doom | Boom Compatible |
| 2019 | Top Ten p3 | Remnant | [`levels/doom2/Ports/p-r/remnant.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/p-r/remnant.zip) | 6.1 MB | Doom II | Limit-removing |
| 2019 | Runner-up | Remnant | [`levels/doom2/Ports/p-r/remnant.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/p-r/remnant.zip) | 6.1 MB | Doom II | Limit-removing |
| 2020 | Top Ten p3 | Syringe | [`levels/doom2/Ports/s-u/syringe.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/syringe.zip) | 2.0 MB | Doom II | Limit-removing |
| 2020 | Runner-up | Criticality | [`levels/doom2/Ports/a-c/criticality.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/criticality.zip) | 2.8 MB | Doom II | Boom |
| 2020 | Runner-up | 25 Years on Earth | [`levels/doom2/Ports/megawads/ph_25yoe.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/ph_25yoe.zip) | 3.4 MB | Doom II | limit-removing |
| 2020 | Top Ten p1 | Rowdy Rudy 2: Powertrip | [`levels/doom2/megawads/rudy2.zip`](https://www.gamers.org/pub/idgames/levels/doom2/megawads/rudy2.zip) | 6.6 MB | Doom II | Vanilla Compatible |
| 2020 | Runner-up | 180 Minutes Pour Vivre | [`levels/doom2/Ports/megawads/180mpv.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/180mpv.zip) | 10.0 MB | Doom II | Limit-removing port |

### Wave 7 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2020 | Top Ten p2 | Running Late 2 | [`levels/doom2/Ports/megawads/rl2.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/rl2.zip) | 12.7 MB | Doom II | BOOM compatible. |
| 2020 | Top Ten p1 | Antaresian Reliquary | [`levels/doom2/Ports/a-c/anta_req.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/anta_req.zip) | 12.8 MB | Doom II | Boom |
| 2020 | Runner-up | Intergalactic Xenology | [`levels/doom2/Ports/d-f/dzintxen.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dzintxen.zip) | 19.5 MB | Doom II | Boom compatible ports will work |
| 2020 | Top Ten p1 | Abandon | [`levels/doom2/Ports/megawads/abandon.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/abandon.zip) | 46.2 MB | Doom II | Boom |
| 2021 | Runner-up | Cydonia | [`levels/doom2/Ports/megawads/cydonia.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/cydonia.zip) | 1.4 MB | The Plutonia Experiment | Limit Removing |
| 2021 | Top Twelve p3 | Arrival | [`levels/doom2/Ports/a-c/arrival.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/arrival.zip) | 2.6 MB | Doom II | Limit-removing |
| 2021 | Runner-up | Deadly Standards 3 | [`levels/doom/Ports/s-u/standrd3.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/s-u/standrd3.zip) | 3.2 MB | Ultimate Doom | Limit-Removing Ports |
| 2021 | Runner-up | URE2020 | [`levels/doom/s-u/ure2020.zip`](https://www.gamers.org/pub/idgames/levels/doom/s-u/ure2020.zip) | 3.3 MB | Ultimate Doom | None / Made for Vanilla |
| 2021 | Top Twelve p2 | Unhallowed | [`levels/doom2/Ports/s-u/unhallowed.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/unhallowed.zip) | 3.8 MB | Doom II | PRBoom/DSDA |
| 2021 | Top Twelve p4 | Haste | [`levels/doom2/Ports/megawads/haste.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/haste.zip) | 5.5 MB | Doom II | Boom |

### Wave 8 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2021 | Runner-up | Atmospheric Extinction | [`levels/doom2/Ports/v-z/vel_aex.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/v-z/vel_aex.zip) | 22.6 MB | Doom II | Boom compatible |
| 2022 | Runner-up | Bellatrix: Tales of Orionis | [`levels/doom2/a-c/bx_e1.zip`](https://www.gamers.org/pub/idgames/levels/doom2/a-c/bx_e1.zip) | 1.8 MB | Doom II | Vanilla format, compatibility level 2 |
| 2022 | Top Ten p3 | The 10x10 Project | [`levels/doom2/Ports/0-9/10x10.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/0-9/10x10.zip) | 6.4 MB | Doom II | Boom-compatible |
| 2022 | Top Ten p3 | Overboard | [`levels/doom2/Ports/megawads/overboard.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/overboard.zip) | 7.7 MB | Doom II | Boom |
| 2022 | Runner-up | Shallow World Episode 1 | [`levels/doom2/Ports/s-u/shallowwe1.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/shallowwe1.zip) | 8.8 MB | Doom II | (Boom -cl9) |
| 2022 | Top Ten p1 | The Magenta Spire | [`levels/doom2/Ports/megawads/magspire.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/magspire.zip) | 9.5 MB | Doom II | MBF |
| 2022 | Runner-up | Capybara | [`levels/doom2/Ports/megawads/capybara.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/capybara.zip) | 12.6 MB | Doom II | Boom-compatible (+UMAPINFO support) |
| 2022 | Top Ten p1 | Ozonia | [`levels/doom2/Ports/megawads/ozonia.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/ozonia.zip) | 17.2 MB | Doom II | Prboom |
| 2023 | Runner-up | Solve Et Coagula | [`levels/doom/Ports/s-u/solvetco.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/s-u/solvetco.zip) | 1.4 MB | Ultimate Doom | limit removing port |
| 2023 | Runner-up | El Viaje de Diciembre | [`levels/doom2/v-z/viaje.zip`](https://www.gamers.org/pub/idgames/levels/doom2/v-z/viaje.zip) | 1.4 MB | Doom II | None! |

### Wave 9 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2023 | Runner-up | Glaive 3 | [`levels/doom2/Ports/g-i/glaive3.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/g-i/glaive3.zip) | 1.6 MB | Doom II | Limit Removing |
| 2023 | Runner-up | The Settlements | [`levels/doom2/Ports/s-u/settlements.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/settlements.zip) | 2.8 MB | Doom II | Boom |
| 2023 | Runner-up | Jaded | [`levels/doom2/Ports/j-l/jaded.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/j-l/jaded.zip) | 4.2 MB | Doom II | MBF21 |
| 2023 | Runner-up | Machete | [`levels/doom2/Ports/megawads/machetefinal.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/machetefinal.zip) | 7.1 MB | Doom II | Requires Boom compatible source port |
| 2023 | Runner-up | What Lies Beneath | [`levels/doom2/Ports/v-z/whatlies.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/v-z/whatlies.zip) | 9.7 MB | Doom II | MBF21 |
| 2023 | Top Twelve p3 | Sepia | [`levels/doom2/Ports/megawads/sepia.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/sepia.zip) | 17.8 MB | Doom II | MBF21, UMAPINFO |
| 2023 | Runner-up | Corruption | [`levels/doom2/Ports/megawads/corruption.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/corruption.zip) | 24.1 MB | Doom II | MBF21, Decorate compatible |
| 2023 | Top Twelve p2 | Hardfest 2 | [`levels/doom2/Ports/megawads/hardfest2.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/hardfest2.zip) | 39.4 MB | Doom II | Boom-compatible (-cl 9) |
| 2023 | Runner-up | 1x1 | [`levels/doom2/Ports/megawads/1x1.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/1x1.zip) | 45.4 MB | Doom II | MBF21 Compatible Port |
| 2024 | Runner-up | YouDoVoodoo | [`levels/doom2/Ports/s-u/uduvudu.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/uduvudu.zip) | 1.1 MB | Doom II | Limit-removing (-complevel 9) |

### Wave 10 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2024 | Runner-up | Grand Canyon | [`levels/doom/Ports/g-i/gcanyon.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/g-i/gcanyon.zip) | 2.1 MB | Ultimate Doom | limit-removing port such as crispy-doom |
| 2024 | Top Twelve p2 | House of Dead Skin | [`levels/doom2/Ports/d-f/deadskin.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/deadskin.zip) | 2.1 MB | Doom II | MBF21 |
| 2024 | Runner-up | Bitter Dream | [`levels/doom2/Ports/a-c/bitter.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/bitter.zip) | 8.1 MB | Doom II | MBF21 |
| 2024 | Top Twelve p1 | Osterity | [`levels/doom2/m-o/osterity.zip`](https://www.gamers.org/pub/idgames/levels/doom2/m-o/osterity.zip) | 8.2 MB | Doom II | none |
| 2024 | Top Twelve p1 | Sleepwalking | [`levels/doom2/Ports/s-u/sleepwalking.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/sleepwalking.zip) | 8.8 MB | Doom II | MBF21, UMAPINFO |
| 2024 | Top Twelve p4 | Liminal Doom 2 | [`levels/doom2/Ports/j-l/ld2.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/j-l/ld2.zip) | 8.8 MB | Doom II | MBF21 |
| 2024 | Runner-up | Firefly | [`levels/doom2/Ports/d-f/firefly.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/firefly.zip) | 10.7 MB | Doom II | MBF21-compatible |
| 2024 | Runner-up | Oops... All Ghosts! | [`levels/doom2/Ports/m-o/oops.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/m-o/oops.zip) | 17.9 MB | Doom II | Vanilla (cl2) |
| 2024 | Runner-up | TNT: Overcharged | [`levels/doom2/Ports/megawads/tnto.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/tnto.zip) | 36.0 MB | TNT: Evilution | Anything Boom Compatible (CL9) |
| 2024 | Runner-up | Formless Mother | [`levels/doom2/Ports/d-f/formless.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/formless.zip) | 36.5 MB | Doom II | MBF21 |

### Wave 11 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2025 | Runner-up | Forlorn | [`levels/doom2/Ports/d-f/forlorn.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/forlorn.zip) | 407 kB | Doom II | Boom-compatible |
| 2025 | Runner-up | Witching Hour | [`levels/doom2/Ports/v-z/witchinghour.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/v-z/witchinghour.zip) | 3.4 MB | Doom II | Limit Removing |
| 2025 | Top Twelve p4 | Axolotl: Regenerated | [`levels/doom2/Ports/megawads/axodius.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/axodius.zip) | 13.1 MB | Doom II | DSDA-Doom or equivalent port with MBF21 |
| 2025 | Top Twelve p1 | In the Doghouse | [`levels/doom2/Ports/megawads/doghouse.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/doghouse.zip) | 22.5 MB | Doom II | Boom (complevel 9) |
| 2025 | Runner-up | Remanence | [`levels/doom2/Ports/p-r/remanence.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/p-r/remanence.zip) | 29.8 MB | Doom II | MBF21 |
| 2025 | Top Twelve p1 | Neon Overdrive | [`levels/doom2/Ports/megawads/neonover.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/neonover.zip) | 31.2 MB | Doom II | MBF21 + UMAPINFO Compatible Source port |
| 2025 | Top Twelve p4 | Crimson Horror | [`levels/doom2/Ports/a-c/crimhor.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/crimhor.zip) | 42.6 MB | Doom II | MBF21 compatible sourceport (-cl 21) |
| 2018 | Runner-up | Demonastery | [`levels/doom2/Ports/d-f/demonastery.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/demonastery.zip) | 2.5 MB | Doom II | Doom 2, Boom-compatible |
| 2018 | Top Ten p2 | Struggle - Antaresian Legacy | [`levels/doom2/Ports/megawads/strg.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/strg.zip) | 20.7 MB | Doom II | Doom 2, limit-removing / DeHackEd |
| 2020 | Top Ten p2 | Mutabor | [`levels/doom2/m-o/mutabor.zip`](https://www.gamers.org/pub/idgames/levels/doom2/m-o/mutabor.zip) | 1.5 MB | Doom II | Doom 2, vanilla / DeHackEd, 1 map |

### Wave 12 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2020 | Runner-up | Return to Hadron E3: Slayer's Swansong | [`levels/doom/Ports/g-i/hadrone3.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/g-i/hadrone3.zip) | 2.1 MB | Ultimate Doom | Ultimate Doom, limit-removing, 9 maps |
| 2020 | Top Ten p3 | Three Is a Crowd | [`levels/doom2/megawads/3iac.zip`](https://www.gamers.org/pub/idgames/levels/doom2/megawads/3iac.zip) | 3.6 MB | Doom II | Doom 2, vanilla-compatible, 32 maps |
| 2021 | Runner-up | Arceon | [`levels/doom2/Ports/a-c/arceon.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/arceon.zip) | 7.2 MB | Doom II | Doom 2, Boom-compatible, 6 maps |
| 2021 | Top Twelve p2 | Fractured Worlds | [`levels/doom2/Ports/d-f/fw.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/fw.zip) | 14.3 MB | Doom II | Doom 2, Boom-compatible, 9 maps |
| 2022 | Top Ten p2 | Don't Turn Your Back on the City | [`levels/doom2/Ports/d-f/dtybotc.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dtybotc.zip) | 1.7 MB | Doom II | Doom 2, Boom-compatible, 1 map |
| 2022 | Runner-up | Pagodia | [`levels/doom2/p-r/pagodia.zip`](https://www.gamers.org/pub/idgames/levels/doom2/p-r/pagodia.zip) | 4.7 MB | Doom II | Doom 2, vanilla-compatible, 9 maps |
| 2022 | Runner-up | Altars of Madness | [`levels/doom2/Ports/a-c/aom.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/aom.zip) | 8.9 MB | Doom II | Doom 2, Boom-compatible, 12 maps |
| 2022 | Runner-up | Solar Struggle | [`levels/doom/Ports/megawads/sstruggle.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/megawads/sstruggle.zip) | 10.9 MB | Ultimate Doom | Ultimate Doom, vanilla-compatible, 36 maps |
| 2022 | Runner-up | 2022: A Doom Odyssey | [`levels/doom/Ports/megawads/2022ado.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/megawads/2022ado.zip) | 14.2 MB | Ultimate Doom | Ultimate Doom, Boom-compatible, 50 maps |
| 2022 | Runner-up | Dance on the Water | [`levels/doom2/Ports/d-f/dotw.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/d-f/dotw.zip) | 14.3 MB | Doom II | Doom 2, Boom-compatible, 10 maps |

### Wave 13 (10)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2022 | Top Ten p3 | Knee-Deep in Knee-Deep in ZDoom | [`levels/doom2/megawads/kdikdizd.zip`](https://www.gamers.org/pub/idgames/levels/doom2/megawads/kdikdizd.zip) | 14.4 MB | Doom II | Doom 2, vanilla-compatible, 13 maps |
| 2023 | Runner-up | Tetanus | [`levels/doom2/s-u/ttns.zip`](https://www.gamers.org/pub/idgames/levels/doom2/s-u/ttns.zip) | 5.2 MB | Doom II | Doom 2, vanilla-compatible, 10 maps |
| 2023 | Runner-up | BOOMER: Beyond Vanilla | [`levels/doom2/Ports/a-c/boomer.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/a-c/boomer.zip) | 11.1 MB | Doom II | Doom 2, limit-removing, 6 maps |
| 2023 | Top Twelve p4 | Ultimate Doom in Name Only | [`levels/doom/Ports/megawads/udino.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/megawads/udino.zip) | 14.9 MB | Ultimate Doom | Ultimate Doom, limit-removing, 36 maps |
| 2023 | Top Twelve p4 | TNT2: Devilution | [`levels/doom2/megawads/tnt2.zip`](https://www.gamers.org/pub/idgames/levels/doom2/megawads/tnt2.zip) | 18.5 MB | Doom II | TNT: Evilution, vanilla-compatible, 32 maps |
| 2023 | Top Twelve p1 | Piña Colada | [`levels/doom2/Ports/megawads/pinacola.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/pinacola.zip) | 29.9 MB | Doom II | Doom 2, Boom-compatible, 24 maps |
| 2024 | Runner-up | Forsaken Compound | [`levels/doom/Ports/a-c/compound.zip`](https://www.gamers.org/pub/idgames/levels/doom/Ports/a-c/compound.zip) | 1.9 MB | Ultimate Doom | Ultimate Doom, limit-removing, 9 maps |
| 2024 | Top Twelve p2 | Squenched Avacausage | [`levels/doom2/Ports/megawads/squench.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/squench.zip) | 7.2 MB | Doom II | Doom 2, Boom-compatible, 13 maps |
| 2024 | Runner-up | Vogel im Kafig | [`levels/doom2/Ports/v-z/vik.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/v-z/vik.zip) | 10.1 MB | Doom II | Doom 2, MBF21, 1 map |
| 2024 | Top Twelve p2 | Simulacrum | [`levels/doom2/Ports/megawads/simulacrum.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/simulacrum.zip) | 41.0 MB | Doom II | Doom 2, MBF21, 32 maps |

### Wave 14 (6)

| Year | Award | Title | /idgames | Size | IWAD | Format (evidence) |
| --- | --- | --- | --- | --- | --- | --- |
| 2024 | Top Twelve p4 | Doom 2 in City Only | [`levels/doom2/Ports/megawads/d2ico.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/d2ico.zip) | 51.2 MB | Doom II | Doom 2, MBF21, 36 maps |
| 2025 | Top Twelve p2 | Dead Sea Scrolls | [`levels/doom/d-f/dss.zip`](https://www.gamers.org/pub/idgames/levels/doom/d-f/dss.zip) | 4.9 MB | Ultimate Doom | Ultimate Doom, vanilla-compatible, 14 maps |
| 2025 | Runner-up | Earthless | [`levels/doom2/Ports/megawads/earthless.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/earthless.zip) | 5.2 MB | Doom II | Doom 2, limit-removing, 32 maps |
| 2025 | Runner-up | Sunless Empire | [`levels/doom2/Ports/s-u/sunless.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/s-u/sunless.zip) | 13.7 MB | Doom II | Doom 2, MBF21, 10 maps |
| 2025 | Runner-up | When Them Demons Cry | [`levels/doom2/Ports/m-o/onicry.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/m-o/onicry.zip) | 23.9 MB | Doom II | Doom 2, MBF21, 1 map |
| 2025 | Top Twelve p4 | Doom 2 in Rural Only | [`levels/doom2/Ports/megawads/d2iro.zip`](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/d2iro.zip) | 43.2 MB | Doom II | Doom 2, MBF21, 33 maps |

### Playable but not wave-ready (no verified `/idgames` path)

**37** playable award entries have no verified archive path in this pass. Most link to a Doomworld release thread; they need either a new download host in `catalog/hosts.toml` (D-028 terms) or manual source resolution before a recipe can be written. They are deliberately **not** put in a wave.

| Year | Award | Title | Source link | Evidence |
| --- | --- | --- | --- | --- |
| 2018 | Top Ten p1 | Avactor | [https://www.doomworld.com/forum/topic/101647-avactor-release/](https://www.doomworld.com/forum/topic/101647-avactor-release/) | Doom 2, Boom-compatible |
| 2018 | Top Ten p2 | Dimension of the Boomed | [https://www.doomworld.com/forum/topic/92574-dimension-of-the-boomed/](https://www.doomworld.com/forum/topic/92574-dimension-of-the-boomed/) | Doom 2, Boom-compatible |
| 2019 | Top Ten p2 | Finely Crafted Fetish Film | [https://www.doomworld.com/forum/topic/104608-finely-crafted-fetish-film-7-puzzle-maps-for-boom/](https://www.doomworld.com/forum/topic/104608-finely-crafted-fetish-film-7-puzzle-maps-for-boom/) | Doom 2, Boom-compatible, 7 maps |
| 2019 | Runner-up | Infraworld: The Hatehammer | [https://www.doomworld.com/forum/topic/107367-infraworld-the-hatehammer/](https://www.doomworld.com/forum/topic/107367-infraworld-the-hatehammer/) | Doom 2, Boom-compatible, 1 map |
| 2019 | Top Ten p3 | Lost Civilization | [https://www.doomworld.com/forum/topic/105215-lost-civilization-22-map-fun-and-nonlinear-megawad-now-with-download-link/](https://www.doomworld.com/forum/topic/105215-lost-civilization-22-map-fun-and-nonlinear-megawad-now-with-download-link/) | Doom 2, Boom-compatible, 22 maps |
| 2019 | Runner-up | Lost Civilization | [https://www.doomworld.com/forum/topic/105215-lost-civilization-22-map-fun-and-nonlinear-megawad-now-with-download-link/](https://www.doomworld.com/forum/topic/105215-lost-civilization-22-map-fun-and-nonlinear-megawad-now-with-download-link/) | Doom 2, Boom-compatible, 22 maps |
| 2020 | Runner-up | Gods & Guardians | [https://www.doomworld.com/forum/topic/100195-godsguardians-classic-6-map-episode-v10/?tab=comments#comment-1863761](https://www.doomworld.com/forum/topic/100195-godsguardians-classic-6-map-episode-v10/?tab=comments#comment-1863761) | Doom 2, vanilla-compatible, 7 maps |
| 2021 | Top Twelve p4 | 1000 Lines Community Project 3 | [https://www.doomworld.com/forum/topic/125479-1000-lines-3-community-project-rc-2-is-released](https://www.doomworld.com/forum/topic/125479-1000-lines-3-community-project-rc-2-is-released) | Doom 2, vanilla-compatible, 32 maps |
| 2021 | Runner-up | Down the Drain | [https://www.doomworld.com/forum/topic/124197-down-the-drain-32-vanilla-maps-rc2/](https://www.doomworld.com/forum/topic/124197-down-the-drain-32-vanilla-maps-rc2/) | Doom 2, vanilla, 32 maps |
| 2021 | Runner-up | Intergalactic Xenology 2 | [https://www.doomworld.com/forum/topic/122180-doom-2-intergalactic-xenology-2/](https://www.doomworld.com/forum/topic/122180-doom-2-intergalactic-xenology-2/) | Doom 2, Boom-compatible, 5 maps |
| 2021 | Runner-up | Old Still Life | [https://www.doomworld.com/forum/topic/124976-old-still-life-wad/](https://www.doomworld.com/forum/topic/124976-old-still-life-wad/) | Doom 2, Boom-compatible, 12 maps |
| 2022 | Runner-up | Anomaly Report | [https://www.doomworld.com/forum/topic/128408-anomaly-report-released-a-new-33-level-megawad-for-vanilla-doom-ii/](https://www.doomworld.com/forum/topic/128408-anomaly-report-released-a-new-33-level-megawad-for-vanilla-doom-ii/) | Doom 2, vanilla-compatible, 33 maps |
| 2022 | Top Ten p1 | JUMPWAD | [https://www.doomworld.com/forum/topic/129577-jumpwad-7-platforming-maps-by-ribbiks-and-me-d/](https://www.doomworld.com/forum/topic/129577-jumpwad-7-platforming-maps-by-ribbiks-and-me-d/) | Doom 2, MBF-compatible, 7 maps |
| 2022 | Top Ten p2 | Malevolence Episode 1 | [https://www.doomworld.com/forum/topic/131061-malevolence-episode-1-9-maps-boom-compatible/](https://www.doomworld.com/forum/topic/131061-malevolence-episode-1-9-maps-boom-compatible/) | Doom 2, Boom-compatible, 9 maps |
| 2023 | Runner-up | Ad Mortem | [https://www.doomworld.com/forum/topic/125461-ad-mortem-a-mbf21-halloween-themed-megawad-new-update-phase-3-is-out/](https://www.doomworld.com/forum/topic/125461-ad-mortem-a-mbf21-halloween-themed-megawad-new-update-phase-3-is-out/) | Doom 2, MBF21-compatible, 34 maps |
| 2023 | Top Twelve p2 | Austrian Avian Association | [https://www.doomworld.com/forum/topic/135951/](https://www.doomworld.com/forum/topic/135951/) | Doom 2, Boom-compatible, 8 maps |
| 2023 | Runner-up | Head Trauma | [https://www.doomworld.com/forum/topic/140572-head-trauma-v-03-looking-for-playtesters/](https://www.doomworld.com/forum/topic/140572-head-trauma-v-03-looking-for-playtesters/) | Doom 2, Boom-compatible, 6 maps |
| 2023 | Runner-up | The Diseases, and Casualties of This Year Being 1632 | [https://www.doomworld.com/forum/topic/133295-boom-the-diseases-and-casualties-this-year-being-1632-11-maps-v11/](https://www.doomworld.com/forum/topic/133295-boom-the-diseases-and-casualties-this-year-being-1632-11-maps-v11/) | Doom 2, Boom-compatible, 11 maps |
| 2023 | Top Twelve p3 | Wormwood V | [https://www.doomworld.com/forum/topic/141190-wormvwood/](https://www.doomworld.com/forum/topic/141190-wormvwood/) | Doom 2, Boom-compatible, 5 maps |
| 2024 | Runner-up | Curse of Blood | [https://www.doomworld.com/forum/topic/146785-curse-of-blood-a-32-map-megawad-rc2-released/](https://www.doomworld.com/forum/topic/146785-curse-of-blood-a-32-map-megawad-rc2-released/) | Doom 2, MBF21, 32 maps |
| 2024 | Runner-up | Eraser | [https://www.doomworld.com/forum/topic/147486-eraser-rc1-update-a-single-level-partial-conversion-boom/](https://www.doomworld.com/forum/topic/147486-eraser-rc1-update-a-single-level-partial-conversion-boom/) | Doom 2, Boom-compatible, 1 map |
| 2024 | Top Twelve p3 | Necromantic Thirst | [https://www.doomworld.com/forum/topic/149103-necromantic-thirst-rc1/](https://www.doomworld.com/forum/topic/149103-necromantic-thirst-rc1/) | Doom 2, Boom-compatible, 32 maps |
| 2024 | Top Twelve p3 | Nostalgic Entropy: Neo Retro | — | Doom 2, MBF21, 1 map |
| 2024 | Runner-up | Outpost 13 | [https://www.doomworld.com/forum/topic/142279-release-outpost-13/](https://www.doomworld.com/forum/topic/142279-release-outpost-13/) | Doom 2, Boom-compatible, 1 map |
| 2024 | Top Twelve p1 | Pirate Doom 2 | [https://www.doomworld.com/forum/topic/143805-pirate-doom-ii-finished/](https://www.doomworld.com/forum/topic/143805-pirate-doom-ii-finished/) | Doom 2, MBF, 32 maps |
| 2024 | Runner-up | Realm of Chaos 2 | [https://www.doomworld.com/forum/topic/125506-realm-of-chaos-2-the-revenge-of-96-rc3-released-testers-needed/](https://www.doomworld.com/forum/topic/125506-realm-of-chaos-2-the-revenge-of-96-rc3-released-testers-needed/) | Doom 2, Boom-compatible, 40 maps |
| 2024 | Top Twelve p3 | Umbra | [https://www.doomworld.com/forum/topic/148700-umbra-rc46-mbf21/](https://www.doomworld.com/forum/topic/148700-umbra-rc46-mbf21/) | Doom 2, MBF21, 1 map |
| 2025 | Runner-up | Beyond the Zero | [https://www.doomworld.com/forum/topic/155082-beyond-the-zero-surreal-lynchian-puzzle-map/](https://www.doomworld.com/forum/topic/155082-beyond-the-zero-surreal-lynchian-puzzle-map/) | Doom 2, Boom-compatible, 4 maps |
| 2025 | Top Twelve p2 | Blues Brothers 2025 | [https://www.doomworld.com/forum/topic/155711-blues-brothers-2025-mbf21-neo-noir-neo-horror-megawad-rc4/](https://www.doomworld.com/forum/topic/155711-blues-brothers-2025-mbf21-neo-noir-neo-horror-megawad-rc4/) | Doom 2, MBF21, 23 maps |
| 2025 | Runner-up | CPD 2 | [https://www.doomworld.com/forum/topic/154633/](https://www.doomworld.com/forum/topic/154633/) | Doom 2, MBF21, 11 maps |
| 2025 | Runner-up | Chilling Winds of Dis | [https://www.doomworld.com/forum/topic/151566-rc3-the-chilling-winds-of-dis-40-maps-boom-format/](https://www.doomworld.com/forum/topic/151566-rc3-the-chilling-winds-of-dis-40-maps-boom-format/) | Doom 2, Boom-compatible, 40 maps |
| 2025 | Top Twelve p3 | Drake O'Brien Ovational Megawad | [https://www.doomworld.com/forum/topic/153776-drake-o%E2%80%99brien-ovational-megawad-rc2/](https://www.doomworld.com/forum/topic/153776-drake-o%E2%80%99brien-ovational-megawad-rc2/) | TNT: Evilution, limit-removing, 32 maps |
| 2025 | Runner-up | Entangled | [https://www.doomworld.com/forum/topic/151288/](https://www.doomworld.com/forum/topic/151288/) | Doom 2, limit-removing, 1 map |
| 2025 | Top Twelve p3 | Gossip | [https://www.doomworld.com/forum/topic/127080-gossip-boom-version-14-final-update/](https://www.doomworld.com/forum/topic/127080-gossip-boom-version-14-final-update/) | Doom 2, Boom-compatible, 9 maps |
| 2025 | Runner-up | Lemoncholia | [https://www.doomworld.com/forum/topic/153418-lemoncholia-6-maps-by-grain-of-salt-limit-removing/](https://www.doomworld.com/forum/topic/153418-lemoncholia-6-maps-by-grain-of-salt-limit-removing/) | Doom 2, limit-removing, 6 maps |
| 2025 | Top Twelve p3 | Lonesome Road | [https://www.doomworld.com/forum/topic/151858-lonesome-road-chapter-1/](https://www.doomworld.com/forum/topic/151858-lonesome-road-chapter-1/) | Doom 2, vanilla-compatible, 9 maps |
| 2025 | Runner-up | The Warp | [https://www.doomworld.com/forum/topic/153124-the-warp-gold-edition-out-now/](https://www.doomworld.com/forum/topic/153124-the-warp-gold-edition-out-now/) | Doom 2, MBF21, 39 maps |

## Tag-citation gaps in existing recipes

- **`going-down` (2014 Top Ten winner) is not tagged `cacowards`.** Its notes say the award category could not be confirmed; the 2014 `best1` page lists it in the Top Ten. It needs `cacowards` plus a citation to `https://www.doomworld.com/cacowards/2014/best1/` in `[verification]`.
- The nine other matching recipes (`ancient-aliens`, `auger-zenith`, `back-to-saturn-x-e1`, `back-to-saturn-x-e2`, `eviternity`, `eviternity-ii`, `sigil`, `sunlust`, `valiant`) carry the tag and a citation. Their citations are mostly secondary (DoomWiki/search) because the authors could not read doomworld.com; the Wayback links in the research file can replace them with primary evidence if the coordinator wants that cleanup.
- No other existing recipe matches a Cacoward entry. The spec 037 Doomer Boards batch (76 recipes) contains only one Cacoward: `auger-zenith` (2021 runner-up).

## New dependencies

None. The research used `curl`, `python3` (standard library), `zstd` and `gzip` on the dev host. No crate was added.

## Open questions

1. **1994 wording in R1.** The Cacowards archive starts in 2004 (its own index: "Since 2004"); no 1994 Cacowards page exists in Wayback. Should the 1994–2003 retrospective lists (*5 Years of Doom*, *10 Years of Doom* top 100) be inventoried as a separate task, or is 2004–2025 the intended scope? **Recommendation:** keep 2004–2025; the retrospective lists are not Cacowards.
2. **Unmatched old entries.** A handful of 2004–2017 award winners are on `/idgames` but the archive filename does not match the award title, and the numeric `?id=` link could not be resolved for every page. They are marked `not matched`, not `not on /idgames`. A follow-up can resolve them from the archive search frontend or by reading each `.txt`.
3. **Runner-ups are included.** The `cacowards` tag description says "winner or runner-up", so the inventory includes 151 runner-ups. If the later recipe spec wants winners only, filter the Runner-up rows out; the tables mark them explicitly.
4. **2016 has eleven winners.** The 2016 edition lists eleven Top Ten entries (ten plus John Romero's *Tech Gone Bad* special icon). This is what the page says, not a duplicate; decide whether the later catalog treats it as a Top Ten or a special.
5. **Special awards scope.** Multiplayer winners are Zandronum/Odamex and are not DSDA-Doom single-player targets; they are inventoried but not proposed for recipes. Worst Wad and joke awards are included only when the winner is a single release, and are not proposed for waves.
6. **Deck validation.** Every `Playable` verdict is a text-file/page compatibility claim, not a gameplay test. All 137 wave candidates still need the standard `Needs Deck Validation` treatment (IWAD detection, complevel, load order, audio).

## Spec items left open

- R4 is proposed, not executed: no recipes were written (the spec says they come later).
- The A1/A2 checkboxes are left for the coordinator; this report and `docs/research/cacowards.md` are the evidence.
- `docs/status.md` and `docs/journal.yml` were not edited; the coordinator should record the completion and the `going-down` tag gap.


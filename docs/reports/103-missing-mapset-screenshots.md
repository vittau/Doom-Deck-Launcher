# 103 — Missing mapset screenshots

## Outcome

**12 of the 26 baseline recipes gained a verified real in-game image; 14 remain
unresolved and unchanged.** All bundled files are JPEG, 640×360 or larger, at most
1280×800 and 192 KiB, and every one reproduces byte-for-byte from the preserved original
with `scripts/lean-image.sh` at the documented quality. The final batch was linted once
offline after merging `main` (`e280c25`) into the task branch: `Catalog lint passed
(offline)`.

- Added: caustic-industry, doom-ii-the-way-id-did, going-down-turbo, hell-revealed-3-pt-1,
  jolly-junkyard, no-end-in-sight, plutonia-2, rekkr, scythe-ii, terminal-voltage,
  the-10x10-project, tnt-revilution.
- Unresolved: deathless, headless-chicken, japanese-valentines, lunatic, memento-mori,
  memento-mori-ii, micro-slaughter-community-project, mindblood-genesis,
  santas-outback-bender, spaceballs-the-quest-for-more-dbp-maps, speed-of-doom, sunder,
  tomatomania, vanguard.
- Bundled bytes: **1,323,209 bytes** across 12 files (largest 189,870 B; smallest 27,653 B).
- Review contact sheet (local, not a desktop capture): `.scratch/103-contact-sheet.png`
  (12 frames, 1314×568). It is intentionally ignored and not committed.

## Added images

Every row records the exact source URL, the contextual page, a truthful credit and the
original dimensions in the `SOURCES.toml` beside the image. No separate image licence was
found on any source page; each record states that, bundles the view under D-024, and says
the image is removed on the author's request. Doomworld/static.doomworld returned HTTP 403
to automated fetches, so those pages and images were read through Wayback Machine captures
on 2026-10-22; the recorded source URLs remain the original hosts.

| Recipe | Source page (context) | Credit | Original → bundled | Bytes |
| --- | --- | --- | --- | --- |
| caustic-industry | <https://doomer.boards.net/thread/3856/dbp69-caustic-industry-cl2> | Doomer Boards Project 69 team | 640×360 → 640×360 | 87,798 |
| doom-ii-the-way-id-did | <https://www.doomworld.com/20years/best1.php> (2013 Cacowards) | Doom 2 The Way id Did team | 1280×1024 → 1000×800 | 93,000 |
| going-down-turbo | <https://www.doomworld.com/forum/topic/149336-going-down-turbo-10th-birthday-special-edition-rc1-7> | Cyriak Harris (mouldy) | 800×396 → 800×396 | 27,653 |
| hell-revealed-3-pt-1 | <https://doomer.boards.net/thread/2328/dbp41-revealed-iii-pt-1> | Doomer Boards Project 41 team | 1600×900 → 1280×720 | 147,149 |
| jolly-junkyard | <https://doomer.boards.net/thread/3050/jolly-junkyard> | Doomer Boards Project 55 team | 1920×1080 → 1280×720 | 132,775 |
| no-end-in-sight | <https://www.doomworld.com/24years/best1.php> (2017 Cacowards) | NaturalTvventy, Xaser and Lutz | 1280×720 → 1280×720 | 132,438 |
| plutonia-2 | <https://www.doomworld.com/16years/best3.php> (2009 Cacowards) | Plutonia 2 Team | 640×480 → 640×480 | 47,837 |
| rekkr | <https://www.doomworld.com/cacowards/2018/best2/> | Revae and the REKKR team | 1920×1080 → 1280×720 | 189,870 |
| scythe-ii | <https://www.doom2.net/erik/scythe2.html> | Erik Alm | 640×400 → 640×400 | 49,387 |
| terminal-voltage | <https://doomer.boards.net/thread/3421/terminal-voltage> | Doomer Boards Project 63 team | 1920×1080 → 1280×720 | 168,026 |
| the-10x10-project | <https://www.doomworld.com/cacowards/2022/best3/> | lunchlunch | 1600×1200 → 1066×800 | 123,983 |
| tnt-revilution | <https://www.doomworld.com/24years/best1.php> (2017 Cacowards) | TNT: Revilution team | 1920×1061 → 1280×707 | 123,293 |

Notes on specific choices:

- **caustic-industry** is a four-view in-game collage that includes the Doomer Boards
  Projects logo; the permission text says so. **going-down-turbo**, **plutonia-2** and
  **scythe-ii** were already inside the dimension bound and were not enlarged.
- **rekkr** was saved at JPEG quality 55 instead of 60 to stay under the 192 KiB cap; its
  `SOURCES.toml` records that choice.
- The **no-end-in-sight** and **tnt-revilution** records were corrected from “2016” to
  “2017” Cacowards after reading the archived page title
  (“Doomworld -- The 2017 Cacowards - Top Ten Page 1”). The other year claims were checked
  against their page versions (`/16years/` = 2009, `/20years/` = 2013, 2018 and 2022
  Cacowards pages named explicitly).

## Unresolved images

These recipes were left unchanged. In each case the bounded research found no verified
in-game image of the specific mapset/edition at the lint's 640×360 minimum. Candidates
were inspected locally; nothing was invented, enlarged or replaced by a different
mapset's image. The primary page below is the recipe's own listing or release page.

| Recipe | Primary page | Reason left unresolved |
| --- | --- | --- |
| deathless | <https://www.doomworld.com/idgames/levels/doom/Ports/megawads/deathless> | No verified usable in-game image secured; `/idgames` returned HTTP 403 to automated fetches and no archived image at usable size was confirmed. |
| headless-chicken | <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/hdlschkn> | Same `/idgames` HTTP 403 blocker; no attributable archived in-game image at the minimum size. |
| japanese-valentines | <https://doomer.boards.net/thread/2432/japanese-valentines-dbp44> and <https://www.doomworld.com/idgames/levels/doom2/Ports/d-f/dbp44> | The DBP44 thread images inspected were 61×27 logo/UI fragments, far below the minimum; no verified in-game view. |
| lunatic | <https://www.doomworld.com/idgames/levels/doom2/Ports/j-l/lunatic> | `/idgames` HTTP 403; the DoomWiki article (<https://doomwiki.org/wiki/Lunatic>) contains only a 320 px title-screen thumbnail, and title screens are excluded by spec 023. |
| memento-mori | <https://www.doomworld.com/idgames/themes/mm/mm_allup> | `/idgames` HTTP 403; no archived in-game image at the minimum size was confirmed. |
| memento-mori-ii | <https://www.doomworld.com/idgames/themes/mm/mm2> | Same `/idgames` blocker; no usable archived image found. |
| micro-slaughter-community-project | <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/mscp_v1b> | Same `/idgames` blocker; no usable archived image found. |
| mindblood-genesis | <https://www.doomworld.com/idgames/levels/doom2/Ports/d-f/dbp8_mbg> | Same `/idgames` blocker; no verified in-game image at the minimum size was secured. |
| santas-outback-bender | <https://doomer.boards.net/thread/1982/dbp31-santas-outback-bender-released> | Candidates were a 124×64 logo, a 180×180 fragment and a 1366×768 title card (not an in-game view); nothing met the rule. |
| spaceballs-the-quest-for-more-dbp-maps | <https://doomer.boards.net/thread/1749/dbp24-spaceballs-sequel-quest-more> | The usable-looking candidates were 284×177 title cards and a Spaceballs film still, below the minimum and not in-game. |
| speed-of-doom | <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/sodfinal> | `/idgames` HTTP 403; the DoomWiki article (<https://doomwiki.org/wiki/Speed_of_Doom>) has only a 320 px title-screen thumbnail. |
| sunder | <https://www.doomworld.com/forum/topic/46002-sunder-map21-actually-exists-right-now/> | Doomworld forum page returned HTTP 403 to automated fetches; no verified archived in-game image at usable size. |
| tomatomania | <https://doomer.boards.net/thread/2035/dbp32-tomatomania> | The thread's candidate image is 320×200, below the 640×360 minimum; it was not enlarged. |
| vanguard | <https://www.doomworld.com/idgames/levels/doom2/Ports/v-z/vanguard> | `/idgames` HTTP 403; the DoomWiki article (<https://doomwiki.org/wiki/Vanguard>) has only a 320 px title-screen thumbnail. |

The thread links above were taken from the saved pages' canonical URLs or in-page links.
The recipe `page` value for each of the 14 remains the authoritative catalogue link.

## Validation

All checks were offline and local; no Rust suite, Clippy, SDK build, network e2e, visible
window or desktop capture was run.

- **Format and bounds:** `file`/`sips` confirmed all 12 assets are JPEG, between 640×360
  and 1280×800, and at most 192 KiB. Bundled total: 1,323,209 bytes.
- **Deterministic derivation:** each asset was regenerated from its preserved original
  with `scripts/lean-image.sh` (quality 60, REKKR 55) and compared with `cmp`; **12/12
  matched byte-for-byte**.
- **Source records:** each `SOURCES.toml` carries `source_url`, `source_page`, `credit`,
  `original_size`, `retrieved` and honest `permission` text; each recipe's
  `[[screenshots]]` entry matches the directory, asset name and credit.
- **Recipe scope:** `git diff` against the baseline shows only added `[[screenshots]]`
  blocks in those 12 recipes; packages, hashes, tags, engine fields and ordering are
  untouched.
- **Final lint (once, after merging `main` `e280c25`):**

  ```sh
  /Users/vitor/orca/workspaces/Doom-Deck-Launcher/flatpak-metainfo-version/target/debug/ddl \
      catalog lint catalog/
  # Catalog lint passed (offline).
  ```

  Log: `.scratch/103-catalog-lint.log`.

## Boundaries honoured

No IWAD or game/engine was downloaded or executed; no downloaded archive was executed;
no generated gameplay image, video, desktop capture or visible window was used. Downloads
were public image assets only, with bounded size/timeout/retry behaviour. No external
posting or permission request was made. No new dependency was added. `OVERVIEW.md`,
`docs/decisions.md`, `docs/status.md`, `docs/journal.yml`, Rust/Slint code, workflows,
versions, hosts and existing screenshots/fixtures were not changed.

## Remaining questions

- **Needs Deck Validation:** how the new cards/band render the 12 images on the owner's
  Deck in Gaming Mode (crop, caption legibility, perceived load) is not verified here.
- The unresolved 14 may be revisitable later through author-hosted pages or direct author
  contact; this task deliberately did not request permissions or post publicly.
- No image licence text was found for any source; bundling relies on D-024 attribution and
  removal on request. Any author objection should remove the affected image.

# Reference research — No Rest for the Living

**Status:** desk research for the D-010 catalog recipe `no-rest-for-the-living` (spec 094)
**Gathered:** 2026-10-22 (UTC) · **Workstream:** spec 094 worker A
**Scope:** the owner-supplied `/Users/vitor/Downloads/NERVE.WAD`, its read-only identity, the
Doom II + `-file` launch model, the reviewed BFG and Steam/KEX variants, release/tag evidence and
screenshot fallback. The commercial file is never downloaded, copied into the repository or bundled.

Labels used throughout:

- **Confirmed** — verified from the supplied bytes, the reviewed project tables/research, or pinned
  DSDA-Doom v0.30.1 source; the evidence is recorded below.
- **Proposed** — a DDL interpretation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## 1. Supplied file identity (Confirmed, read-only)

`/Users/vitor/Downloads/NERVE.WAD` was hashed and parsed as data with Python's `hashlib`, `zlib` and
`struct`; nothing was executed, copied into the repository, imported into a fixture or bundled.

| | Value |
| --- | --- |
| Size | 3,819,855 bytes |
| SHA-256 | `2d234de8700ee403375fda278e50ea17739b36703b3316e0429b05aca4458fa9` |
| MD5 | `967d5ae23daf45196212ae1b605da3b0` |
| SHA-1 | `3451288383fb16e196f273d9f85d58c1fda97bf4` |
| CRC-32 | `ad7f9292` |
| Header | `PWAD` |
| Lumps / maps | 108 lumps / `MAP01`–`MAP09` |
| Other lumps | `CWILV00`–`CWILV08` plus the normal map groups |
| Patches / metadata | no `DEHACKED`, `COMPLVL`, `UMAPINFO` or `MAPINFO` |

This is exactly the `nerve-bfg` row in `crates/ddl-core/data/iwads.toml` (BFG Edition / XBLA / PSN;
also the version bundled with Doom + Doom II). The separate `nerve-kex` row is 4,039,441 bytes and
117 lumps; it is **not** the supplied file. **Confirmed.**

## 2. Steam / KEX variant (published metadata, not locally inspected)

The reviewed table records the DOOM + DOOM II `rerelease/nerve.wad` variant as:

| Variant | Size | MD5 | SHA-1 | CRC-32 |
| --- | ---: | --- | --- | --- |
| `nerve-kex` | 4,039,441 | `23422eb42833ac7b0dd59c0c7ae18a6f` | `4522eaec7a13aac24b456e29520e99ac879e6989` | `07d9faab` |

No SHA-256 is published for it, and it was not supplied for local inspection. The recipe therefore
lists only the BFG SHA-256 in `known_sha256`; an imported KEX file is accepted by the existing
import flow for the player's file but is honestly reported as an unrecognized hash, never as
verified. The KEX size/MD5 are recorded here and in the recipe comments; no KEX SHA-256 is invented.
**Confirmed from the reviewed table; not locally inspected.**

## 3. Doom II + `-file` launch model

- `NERVE.WAD` is a PWAD, not an IWAD. The catalog's `base_game.game` is therefore `doom2`; NERVE is
  never modeled as a base game or downloaded as an IWAD. **Confirmed.**
- `base_game.variants` is empty, the existing convention that accepts every recognized Doom II
  variant: BFG, classic 1.x, French and both reviewed Steam/KEX variants. `Game::Doom2` still
  excludes TNT and Plutonia, and NERVE itself is never modeled as an IWAD. The reviewed table in
  `crates/ddl-core/data/iwads.toml` supplies the classic, BFG and KEX identities. **Confirmed.**
- DSDA-Doom's `D_AddFile()` forces `gamemission = pack_nerve` when the added file name ends in
  `nerve.wad`; the pinned v0.30.1 code uses `strnicmp`, so the owner's uppercase `NERVE.WAD` works.
  The file name must be preserved through import.
  [d_main.c:914–940](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L914-L940).
  **Confirmed.**
- **Port family: limit-removing; complevel: `vanilla` (2).** The maps are documented as requiring
  a limit-removing port (Doomworld forum: *"nerve.wad requires a limit removing port"*,
  <https://www.doomworld.com/forum/topic/101831-is-there-a-mod-that-adds-the-console-exclusive-levels-into-vanilla-doom/>),
  while the Doom II target and the IWAD table's `default_complevel = 2` keep the complevel at
  vanilla/2. These are separate facts: limit-removing is the minimum port family, not a Boom
  complevel. No `COMPLVL`/`UMAPINFO` is present and no gameplay test was performed. **Confirmed
  for the family from the reviewed statement; complevel Proposed; Needs Deck Validation.**

## 4. Release, maps and permissions

- **Original public release:** No Rest for the Living shipped with Doom II on Xbox Live Arcade on
  **2010-05-26** (day precision). Wikipedia's Doom II release table gives the Xbox 360 release date;
  the archived DoomWiki page (2011-09-28, `oldid` snapshot) says the expansion was developed for the
  XBLA release and cites a Shacknews comment dated 26 May 2010. Later releases (Doom 3: BFG
  Edition, Doom Classic Complete, the 2019 Unity add-on and the 2024 KEX release) are variants, not
  the original date. **Confirmed for 2010-05-26 from those sources; day precision.**
- **Maps:** nine total — eight standard maps plus *March of the Demons* as the secret ninth
  (DoomWiki 2011; the WAD has `MAP01`–`MAP09`). The publisher's 2022 Add-ons page also calls it
  "Nerve Software's nine-map Add-on" and says it debuted on the Xbox Live Arcade port of DOOM II
  (<https://bethesda.net/en-US/news/add-ons-for-doom-doom-ii-2022>). **Confirmed.**
- **Cacowards:** the 2010 Top Ten p1 winner, per the archived inventory
  `docs/research/cacowards.md` §2010 (source page
  <https://web.archive.org/web/2026id_/https://www.doomworld.com/17years/best1.php>). The recipe
  cites <https://www.doomworld.com/cacowards/2010/> and notes that the live page was not read in
  this session. **Confirmed from the archived inventory.**
- **Earned tags:** `episode` (nine maps, at most nine), `cacowards` (2010 winner), `classics`
  (2010 release with Cacoward recognition). No `megawad` (nine is under thirty), no `challenging`
  claim (no reviewed difficulty statement), no `community-projects` (commercial id/Nerve Software
  release).
- **Permission:** commercial id Software/Nerve Software content. The recipe claims no redistribution
  permission and states that DDL never downloads, bundles or redistributes `NERVE.WAD`; the player
  imports their own legally obtained copy for local use. **Confirmed policy statement, not a licence
  grant.**

## 5. Recipe shape and capability dependency

- One required, default-on `user_supplied` `main` component with a single `maps` member, no package
  or member reference, nonempty `obtain` instructions and the BFG SHA-256 in `known_sha256`.
- `packages = []`: there is no download, dummy package or remotely acquired commercial data.
- The existing catalog currently rejects an empty package list in `catalog/lint.rs`, and Setup marks
  required rows as locked/non-actionable. Spec 094 R3/R4 assigns those narrow capability changes to
  worker D; this worker does not edit shared model/lint/runtime/UI code.
- The recipe is authored and tested now, but final offline lint/embedding must wait for D's
  capability merge and the coordinator's signal.

## 6. Artwork

One real in-game screenshot is committed under
`crates/ddl/ui/assets/screenshots/no-rest-for-the-living/01.jpg` (800×600 JPEG, 80,715 bytes),
converted with `scripts/lean-image.sh` from the archived DoomWiki map image
`No_Rest_for_the_Living_Map01_Castle2.PNG` (800×600 PNG).
`SOURCES.toml` records the exact Wayback image URL, the canonical DoomWiki map page, the
`Nerve Software (screenshot via DoomWiki)` credit, the original size, the retrieval date and an
honest permission note: DoomWiki text is CC BY-SA 4.0, the in-game imagery is commercial
id Software/Nerve Software content, and the image is bundled as a representative in-game view under
D-024, removed on request. No image was synthesized, cropped from a title screen or captured from a
window.

## 7. Open Deck checks

- Import the supplied BFG `NERVE.WAD` on the Deck and confirm the required main becomes playable
  without an off switch.
- Confirm DSDA-Doom selects the NERVE mission (`pack_nerve`) and the nine-map progression with a
  Doom II IWAD and `-file`.
- Import the Steam/KEX NERVE variant over a recognized KEX Doom II IWAD; confirm the file is
  accepted with the unknown-hash warning and is not labelled verified.
- Confirm no network request occurs during metadata-only install and that a missing file blocks Play
  with a clear Import action.
- Confirm vanilla/2 behavior, audio and mission selection in actual play.

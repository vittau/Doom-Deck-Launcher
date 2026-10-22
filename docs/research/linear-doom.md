# Linear Doom — author publication and policy assessment (spec 091, R3–R4)

**Verdict: no recipe.** No v1.2 archive bytes could be obtained through the tested
policy-compliant routes, and without the bytes the map/lump inventory needed to claim
DSDA-Doom compatibility cannot be inspected. The evidence below records
the identity we could confirm and the exact blocker. Nothing was executed, no IWAD was
downloaded and no anti-bot/authentication control was bypassed.

## 1. Identity (Confirmed from primary ModDB page metadata)

| Field | Value | Evidence |
| --- | --- | --- |
| Title | Linear Doom | ModDB mod page |
| Author / uploader | DeXiaZ | ModDB mod and file pages |
| Publication | https://www.moddb.com/mods/linear-doom | ModDB |
| Description | "a jokewad which makes all classic Doom 1 levels straightened into one line" | ModDB mod page |
| Original release | 2019-03-31 | ModDB original file page "Added Mar 31st, 2019" (Wayback capture `20240913200632`); the mod header rounds it to "Released Apr 1, 2019" |
| Original archive | `Linear_Doom.7z`, 275,143 bytes, MD5 `488637b2a879ea933b62f819a1fba47f` | ModDB original file page |
| Original notes | "All Doom Levels are Straight Line now. No more mazes. Tested with GZDoom 2.4.0" | ModDB original file page |
| Update 1.1 | 2019-04-02, "Fixed version (E1M8 HOM and E3M6' exit door). All Doom Levels are Straight Line now. No more mazes. Tested with GZDoom 2.4.0" | ModDB downloads page |
| Selected update 1.2 | 2019-04-06, "Fixed HOM in E1M4, added chaingun in E2M4 (there were no chainguns in Episode 2 before, somehow)" | ModDB v1.2 file page |
| Archive | `Linear_Doom_v1.2.7z` (7z format — not supported by the recipe schema, which accepts ZIP or a bare WAD) | ModDB v1.2 file page |
| Size | 274,927 bytes (listed as 268.48kb) | ModDB v1.2 file page |
| Published hash | MD5 `4f758b008649e7424ea634c615f34de7` | ModDB v1.2 file page |
| Download entry point | `/downloads/start/176643?referer=...` | ModDB v1.2 file page |
| Preview image | `https://media.moddb.com/images/downloads/1/177/176643/Screenshot_Doom_20190331_023937.png` | ModDB v1.2 file page |

The v1.1/v1.2 notes mention `E1M4`, `E1M8`, `E2M4` and `E3M6`, so the mapset covers at
least E1–E3 of Doom 1. The exact slot count (27 or 36), IWAD variant and lump set are
**not** confirmed: they require the archive. The page says "Tested with GZDoom 2.4.0";
that is a test report, not a port requirement, and it proves neither a GZDoom dependency
nor DSDA-Doom compatibility.

## 2. Retrieval attempts (2026-10-22, no bypass)

| URL | Result |
| --- | --- |
| `https://www.moddb.com/mods/linear-doom` | HTTP 403, Cloudflare "Just a moment…" challenge |
| `https://www.moddb.com/mods/linear-doom/downloads` | HTTP 403, same challenge |
| `https://www.moddb.com/mods/linear-doom/downloads/linear-doom-v12` | HTTP 403, same challenge |
| `https://www.moddb.com/mods/linear-doom/downloads/linear-doom-v12/start` | HTTP 403, same challenge |
| `https://www.moddb.com/members/dexiaz/downloads` | HTTP 403, same challenge |
| `https://dsdarchive.com/wads/linear-doom` | HTTP 404 (no DSDA entry) |
| Wayback capture `20190407110150` of the mod page | HTTP 200 HTML, metadata usable |
| Wayback capture `20190406152828` of the downloads page | HTTP 200 HTML, metadata usable |
| Wayback capture `20240913200609` of the v1.2 file page | HTTP 200 HTML: filename, size, MD5, dates, notes |
| Wayback capture `20240913200632` of the original file page | HTTP 200 HTML: filename, size, MD5, "Added Mar 31st, 2019" |
| Wayback capture of `https://www.moddb.com/downloads/start/176643` | not archived (Wayback's generic "not archived" page) |
| Wayback CDX search for the archive/file name | CDX endpoint reported "Temporarily Offline" during the check; direct snapshot URLs above were used instead |

The newest Wayback snapshots of the mod pages are themselves Cloudflare challenge pages;
the 2019/2024 captures used here predate that and contain the real metadata. No request
used browser emulation, cookies, JS execution, CAPTCHA solving or any other anti-bot
workaround. **No archive bytes were obtained**: the live pages answered 403 and the Wayback
captures contain HTML metadata only.

## 3. Why no recipe can be added under current policy (R4)

0. **Unsupported archive format.** The author publishes a **7z** archive
   (`Linear_Doom_v1.2.7z`). The recipe schema accepts only `kind = "zip"` or a bare WAD, and
   DDL's archive reader extracts ZIP; it has no 7z reader. A 7z package therefore cannot be
   declared or installed even if a durable URL existed. Adding 7z support would mean a new
   archive dependency and downloader/engine work, which is out of scope. This is independent
   of host access and of engine compatibility.
1. **Interactive gate.** ModDB serves the file only after an HTML "start download"
   endpoint (`/downloads/start/176643`). DDL's downloader is a bounded HTTPS client: it
   follows at most five redirects, validates HTTPS and an exact download host at each hop,
   and cannot solve a JS challenge. The gate answered HTTP 403 to direct requests.
2. **No observed direct target.** The start endpoint is an HTML gate; none of the tested
   pages exposed a direct file URL and no attempt was made to bypass the challenge. The form
   or lifetime of any eventual download target was therefore **not determined**, so no exact
   host or URL can be recorded as a recipe URL. Adding `www.moddb.com` to the allowlist
   would not by itself give DDL's bounded HTTPS client a fetchable file. No host change was
   made.
3. **No alternate source tested.** DSDA returned HTTP 404 for `linear-doom` and no
   `/idgames` path was found on the tested mirrors. Other mirrors were not searched, so this
   is not a claim that no alternate source exists; a third-party mirror would in any case be
   a new trust decision rather than an existing-host mirror verification.

The MD5 and byte size published on the file page are useful identity pins, but the schema
requires SHA-256 for the package and every named member, plus a fetchable package URL; no
member can be named or hashed without the archive.

## 4. DSDA-Doom 0.30.1 compatibility

**Not established.** DSDA-Doom v0.30.1 runs vanilla, Boom, MBF and MBF21 Doom-format data
plus UMAPINFO; it does not run GZDoom-only content (ZScript, ACS, slopes/3D floors, and it
does not read `ZMAPINFO`/`MAPINFO`/`DECORATE`). Whether Linear Doom is a plain map
replacement or depends on GZDoom-only lumps can only be decided from its WAD directory,
which is unavailable. The author's GZDoom test note is not evidence of a requirement, and
conversely it is not evidence of DSDA compatibility. Labelling the recipe "works in
DSDA-Doom" would therefore be invented.

## 5. Chronology input for spec 090

- Original public release: **2019-03-31** (day precision; ModDB file "Added" date, with the
  page header rounding to April 1).
- Selected update: **2019-04-06** (v1.2). No recipe was added, so this is research-only and
  does not affect the catalog or any date field.

## 6. Artwork

The v1.2 file page embeds a preview at
`https://media.moddb.com/images/downloads/1/177/176643/Screenshot_Doom_20190331_023937.png`.
No image was downloaded or bundled: without an accepted recipe there is nothing to attach
it to, and the preview's licence was not established. The painted fallback remains for any
future entry.

## 7. What would unblock a recipe

These are separate needs; solving one does not solve the others:

- **A local/supplied copy of the v1.2 archive** would let us inspect the WAD directory,
  compute package/member SHA-256 and settle the map count, lump inventory and engine
  compatibility. On its own it would **not** provide the automatic catalog download (a
  durable URL on an allowlisted host is still required) and would **not** solve the 7z
  limitation.
- **An author republish of the same bytes as a ZIP or bare WAD on an already-reviewed host**
  would address the format and download blockers together; fetch lint could then verify the
  pinned bytes.
- **An owner-approved policy able to handle ModDB's interactive gate** is a separate policy
  decision and remains out of scope for this task.

Until then the acceptance criterion is met by this blocker report: no `catalog/linear-doom.toml`
is added and the playable catalog is unchanged.

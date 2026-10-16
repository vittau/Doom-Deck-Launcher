# Upstream sources research — DSDA-Doom releases/AppImage, Doomworld /idgames, AUGER;ZENITH

Scope: facts DDL needs to (a) discover/verify/cache DSDA-Doom releases (D-007, `OVERVIEW.md` §5),
(b) model /idgames-sourced packages in catalog recipes (D-010, §9), and (c) build the reference
recipe for **AUGER;ZENITH** (DBP37) including its optional add-on.

All claims are labelled **Confirmed** (observed or quoted from a primary source in this session),
**Proposed** (a DDL decision, not an upstream fact) or **Needs Deck Validation** (cannot be
verified on the macOS arm64 dev host).

## Method and evidence

- Research date: **2026-10-16** (UTC). Host: macOS 27.0.1 arm64; tools `gh` (authenticated as a
  normal user token), `curl`, `file`, `shasum`, `xxd`, `unzip`, `python3`. `unsquashfs` is **not**
  installed and no tool was installed (**Confirmed**).
- Everything is read-only: no file from any archive was executed; archives were only listed,
  hashed and parsed.
- Downloads and scratch artifacts live in `.scratch/` (git-ignored) and are never committed.
- Total outbound requests ≈ 45 (a few over the ~40 guidance): 13 GitHub REST calls, 2 GitHub raw
  file fetches, 6 live Doomworld attempts (all rejected), 10 archive/mirror/AppImage downloads and
  HEADs, 7 Wayback/CDX fetches, 2 doc pages, plus 4 GitHub release-asset/rate-limit probes.
- Scripts written for this research (kept in `.scratch/`, not committed): `wadinfo.py` (WAD lump
  directory reader) and `squashls.py` (minimal squashfs 4.0 lister, stdlib only).

---

## 1. DSDA-Doom GitHub releases (`dsda-org/dsda-doom`)

Command used: `gh api -X GET /repos/dsda-org/dsda-doom/releases -f per_page=10`
(equivalent to `curl -H 'Accept: application/vnd.github+json'
https://api.github.com/repos/dsda-org/dsda-doom/releases?per_page=10`). Raw output saved to
`.scratch/releases.json`.

### 1.1 Last 10 releases

| tag | published (UTC) | prerelease | Linux asset | other assets |
| --- | --- | --- | --- | --- |
| v0.30.1 | 2026-10-09T19:16:04Z | false | `dsda-doom-0.30.1-linux-x86_64.appimage` (14608888 B) | `dsda-doom-0.30.1-mac-uni.zip` (11490973 B), `dsda-doom-0.30.1-win-x64.zip` (7201902 B) |
| v0.30.0 | 2026-09-29T00:14:49Z | false | `dsda-doom-0.30.0-linux-x86_64.appimage` (14600696 B) | `dsda-doom-0.30.0-mac-uni.zip`, `dsda-doom-0.30.0-win-x64.zip` |
| v0.29.4 | 2025-09-26T14:57:21Z | false | `dsda-doom-0.29.4-linux-x86_64.appimage` (14567928 B) | `dsda-doom-0.29.4-mac-arm64.zip`, `dsda-doom-0.29.4-mac-x86_64.zip`, `dsda-doom-0.29.4-win-x64.zip` |
| v0.29.3 | 2025-07-03T00:00:43Z | false | `dsda-doom-0.29.3-Linux.appimage` (14842360 B) | `…-mac-arm64.zip`, `…-mac-x86_64.zip`, `…-win64.zip` |
| v0.29.2 | 2025-06-26T21:36:13Z | false | `dsda-doom-0.29.2-Linux.appimage` (14842360 B) | `…-mac-arm64.zip`, `…-mac-x86_64.zip`, `…-win64.zip` |
| v0.29.1 | 2025-06-26T12:57:24Z | false | `dsda-doom-0.29.1-Linux.appimage` (14842360 B) | `…-mac-arm64.zip`, `…-mac-x86_64.zip`, `…-win64.zip` |
| v0.29.0 | 2025-04-08T22:32:05Z | false | `dsda-doom-0.29.0-Linux.appimage` (14293496 B) | `…-mac-arm64.zip`, `…-mac-x86_64.zip`, `…-win64.zip` |
| v0.28.3 | 2025-01-17T17:39:09Z | false | `dsda-doom-0.28.3-Linux.appimage` (16790720 B) | `…-mac-arm64.zip`, `…-mac-x86_64.zip`, `…-win64.zip` |
| v0.28.2 | 2024-10-25T14:03:48Z | false | `dsda-doom-0.28.2-Linux.appimage` (14456000 B) | `…-mac-arm64.zip`, `…-mac-x86_64.zip`, `…-win64.zip` |
| v0.28.1 | 2024-08-18T14:37:14Z | false | **none** | single `dsda-doom-0.28.1.zip` (13121468 B) |

**Confirmed:** none of the last 10 releases is flagged prerelease; every release publishes exactly
one Linux AppImage except v0.28.1, which has no Linux asset at all.

### 1.2 Linux AppImage naming pattern and its stability

- `dsda-doom-<version>-linux-x86_64.appimage` — **v0.29.4, v0.30.0, v0.30.1** (lower-case,
  architecture in the name). **Confirmed**
- `dsda-doom-<version>-Linux.appimage` — v0.28.2 … v0.29.3 (capital `L`, no architecture).
  **Confirmed**
- v0.28.1: no AppImage (one `.zip` for all platforms). **Confirmed**
- Other platforms also churn: `win64.zip` → `win-x64.zip` at v0.29.4, and `mac-arm64.zip` +
  `mac-x86_64.zip` → a single `mac-uni.zip` at v0.30.0. **Confirmed**
- **Proposed:** match assets by *tag + OS/arch*, never by a hard-coded filename. A safe selector is
  "the newest non-prerelease asset whose name matches `(?i)-linux.*\.(appimage)$`", with a
  per-version allowlist entry in DDL's reviewed list (D-007) carrying the exact expected name and
  SHA-256.

### 1.3 Asset `digest` field and upstream checksums/signatures

- The REST API **does** return `assets[].digest`, e.g. v0.30.1 Linux:
  `sha256:b06adf3f0c7d4e82dc1870baf97e67e4dc4e663e37f69070ddc02fa3e97608b0`. **Confirmed**
- It is **absent (`null`) for older releases**: present for every asset of v0.29.1 … v0.30.1,
  `null` for v0.29.0, v0.28.3, v0.28.2 and v0.28.1. **Confirmed**
  (So a recipe for an older release cannot rely on the API for a digest.)
- **No release publishes any checksum or signature file.** In the last 10 releases every asset name
  ends in `.appimage`/`.zip`; there is no `.sha256`, `.sha256sum`, `.asc`, `.sig` or `.minisig`
  asset, and no release body mentions checksums or signatures. **Confirmed**
- Release tags are **not signed**: `gh api /repos/dsda-org/dsda-doom/commits/v0.30.1` returns
  `{"reason":"unsigned","verified":false,"signature":null,"payload":null}`. **Confirmed**
- A newer API field, `immutable`, appears on releases from v0.29.1 onwards and is absent on older
  ones. Meaning is undocumented in the payload; treat it as informational only. **Confirmed
  (observed) / Open question (semantics).**
- The AppImage itself carries **no embedded signature and no update information**: in the
  ELF section table, `.sha256_sig` (1024 B) and `.sig_key` (8192 B) are all zero bytes, and
  `.upd_info` (1024 B) is all zero bytes (see §2). **Confirmed**
- Build provenance: `.github/workflows/continuous_integration.yml` (fetched from `master` on
  2026-10-16) builds the Linux artifact on `ubuntu-22.04` with GCC/Ninja
  (`-DCMAKE_INSTALL_PREFIX=/usr`), runs `cpack`, then publishes `build/*.appimage` with
  `ncipollo/release-action`. There is no signing or hashing step. **Confirmed**

**Conclusion (Confirmed):** GitHub's `digest` is the only machine-checkable hash upstream provides,
and it only exists for recent releases. This matches D-007 ("GitHub's asset digest proves transfer
integrity, not independent trust").

### 1.4 Rate limits, conditional requests and caching

Measured on 2026-10-16 with `curl https://api.github.com/rate_limit` and the response headers.

| item | value | evidence |
| --- | --- | --- |
| Unauthenticated primary limit | **60 requests/hour per IP** | `x-ratelimit-limit: 60`; docs <https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api> |
| Authenticated primary limit | **5000 requests/hour per user** | `{"limit":5000,…}` from `/rate_limit` |
| Unauthenticated search limit | 10 requests/minute | `/rate_limit` → `resources.search = {limit: 10}` |
| Secondary limits | 100 concurrent requests; ~900 points/min; 90 s CPU/60 s | docs (link above) |
| `GET /rate_limit` | free (does not count) | docs |
| Cache headers on releases | `cache-control: public, max-age=60, s-maxage=60` | response header |
| Validator | **weak** ETag, e.g. `W/"b4857d16…cbec9a"`; also `last-modified` | response header |
| 304 with `Authorization` | **does not count** against the limit (measured 4956 → 4956) | docs + measurement |
| 304 **without** `Authorization` | **does count** (measured 54 → 53) | measurement; docs scope the exemption to authenticated requests |
| Exceeded limit | `403`/`429`, `x-ratelimit-remaining: 0`, wait until `x-ratelimit-reset` | docs |

**Recommended DDL behaviour (Proposed):**
1. Prefer cache-first: never poll on every launch. Refresh at most once per day, and only on an
   explicit user action ("Check for updates").
2. Send `If-None-Match` with the stored weak ETag for `GET /repos/dsda-org/dsda-doom/releases` and
   treat `304` as "no change". Because DDL is unauthenticated by default, assume the `304` **costs**
   a request; budget accordingly (60/hour is shared by every DDL user behind the same NAT/IP).
3. Always request `per_page=10` (or a fixed page size) and the same query string, otherwise the
   ETag changes and every poll is a 200.
4. Persist the whole ETag/JSON pair plus a `fetched_at` in JSON state (D-006), so the app works
   fully offline with a stale-but-labelled list.
5. On `403`/`429`, back off until `x-ratelimit-reset` and show the cached list; never retry in a
   loop (docs: repeated requests while limited can get the client banned).
6. The asset download URL redirects to a signed, expiring
   `release-assets.githubusercontent.com` URL (observed `se=2026-10-16T16:01:31Z`). Follow the
   redirect at download time; never cache the redirect target. **Confirmed**
7. Optional (Proposed): if a `GITHUB_TOKEN` is present in the environment, use it for the metadata
   call only — it raises the limit to 5000/hour and makes `304` free — but never require it.

---

## 2. The v0.30.1 Linux x86-64 AppImage

Downloaded with:

```sh
curl -sL -o .scratch/dsda-doom-0.30.1-linux-x86_64.appimage \
  https://github.com/dsda-org/dsda-doom/releases/download/v0.30.1/dsda-doom-0.30.1-linux-x86_64.appimage
shasum -a 256 .scratch/dsda-doom-0.30.1-linux-x86_64.appimage
```

### 2.1 Size and hash vs the API digest

| quantity | value |
| --- | --- |
| Bytes downloaded | **14608888** |
| SHA-256 (local) | `b06adf3f0c7d4e82dc1870baf97e67e4dc4e663e37f69070ddc02fa3e97608b0` |
| API `digest` | `sha256:b06adf3f0c7d4e82dc1870baf97e67e4dc4e663e37f69070ddc02fa3e97608b0` |
| API `size` | 14608888 |

**Confirmed:** the two independent values agree exactly (hash and size). This validates the
"D-007 integrity check" path for a release that has a `digest`.

### 2.2 Container format (inspected, never executed)

- `file` → `ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), static-pie linked,
  BuildID[sha1]=6068f2cec943069c072c32fe680557d1e5f7ec6f, stripped`. **Confirmed**
- Bytes 0–3 are the ELF magic `7f 45 4c 46`; bytes 8–9 are `41 49` (`AI`) followed by `02` at
  offset 10 → **AppImage type 2** (magic `AI\x02` at offset 8). **Confirmed**
- Embedded squashfs payload: the real superblock is at **offset 944632** (`0xE6A78`), magic
  `hsqs`, squashfs **4.0**, `block_size` 131072, `bytes_used` 13660197, **compression zstd**,
  216 inodes, 18 fragments. Payload ends at 14604829; the file is padded with ~4059 zero bytes
  and has no trailing `.appimage` signature block. **Confirmed**
  (Because the runtime is a *static-pie* ELF, the offset must be read from the file, not assumed:
  `--appimage-offset` is the runtime's own way to report it, and DDL can locate `hsqs` the same way
  this research did.)
- Runtime identification from strings inside the ELF: AppImage **type2-runtime**
  (<https://github.com/AppImage/type2-runtime>), which statically embeds **squashfuse** (MIT) and
  **libfuse** (LGPL-2.1) and states so in its own help text. **Confirmed (strings)**
- Runtime options visible in the binary: `--appimage-extract [pattern]`,
  `--appimage-extract-and-run`, `--appimage-mount`, `--appimage-offset`, `--appimage-portable-home`,
  `--appimage-portable-config`, `--appimage-signature`, `--appimage-updateinfo`,
  `--appimage-version`, `--appimage-help`, plus the `APPIMAGE_EXTRACT_AND_RUN=1` environment
  variable. **Confirmed (strings)**

### 2.3 FUSE requirements

- The runtime logs "Cannot mount AppImage, please check your FUSE setup.", "Error: No suitable
  fusermount binary found on the $PATH" and uses `/dev/fuse`, `fusermount` and squashfuse.
  **Confirmed (strings)** → an unprivileged in-container/Flatpak run needs `/dev/fuse` and a
  `fusermount`/`fusermount3` binary; otherwise the runtime falls back to extraction
  (`--appimage-extract-and-run` / `APPIMAGE_EXTRACT_AND_RUN=1`).
- Whether SteamOS Gaming Mode with the Steam Linux Runtime satisfies that is
  **Needs Deck Validation** (`OVERVIEW.md` §5 already lists this as a hypothesis).
- **Proposed:** implement the documented fallback — extract once into a versioned directory and
  exec the payload entry point — rather than a private fork, and test both paths on the Deck.

### 2.4 Payload listing (`unsquashfs` unavailable — listed with an own read-only parser)

`unsquashfs` is not installed on this host, so the payload was listed by a purpose-written
stdlib-only squashfs reader (`.scratch/squashls.py`, zstd via `compression.zstd`). It decodes the
metadata blocks and walks the directory tree; it executes nothing (**Confirmed**). Result summary
(`.scratch/appimage-listing.txt`, 216 lines = 216 inodes): **131 files, 81 directories,
3 symlinks**.

- `AppRun` is a **symlink → `usr/bin/dsda-doom`** (not a shell script). `.DirIcon` and
  `dsda-doom.svg` point at `usr/share/icons/hicolor/scalable/apps/dsda-doom.svg`; there is a
  `dsda-doom.desktop` (327 B) at the root and `usr/share/applications/dsda-doom.desktop` (301 B).
  **Confirmed**
- Game data and docs in the payload: `usr/bin/dsda-doom` (3396665 B),
  `usr/share/games/doom/dsda-doom.wad` (3676513 B — required `dsda-doom.wad`),
  `usr/share/doc/dsda-doom/COPYING` (18077 B = the GPL-2.0 text, same size as
  `prboom2/COPYING` in the repo). **Confirmed**
- `usr/lib/` holds the bundled libraries (all take precedence over system libs when the AppImage
  mounts). Grouped (**Confirmed**):
  - **SDL / windowing / input**: `libSDL2-2.0.so.0`, `libSDL2_image-2.0.so.0`,
    `libSDL2_mixer-2.0.so.0`, `libX11`-family (`libXau`, `libXcursor`, `libXdmcp`, `libXext`,
    `libXfixes`, `libXi`, `libXinerama`, `libXrandr`, `libXrender`, `libXss`, `libXxf86vm`),
    `libwayland-{cursor,egl,server}.so.0`, `libxkbcommon.so.0`, `libdecor-0.so.0`,
    `libxcb-randr.so.0`, `libXcursor`; **no `libasound`** is bundled (audio goes through PulseAudio
    or SDL2's runtime ALSA probing).
  - **Audio / music**: `libfluidsynth.so.3`, `libinstpatch-1.0.so.2`, `libportmidi.so.0`,
    `libxmp.so.4`, `libmodplug.so.1`, `libmad.so.0`, `libmpg123.so.0`, `libsndfile.so.1`,
    `libvorbis*`, `libopus*`, `libogg.so.0`, `libFLAC.so.8`.
  - **Graphics / image**: `libGLU.so.1`, `libpng16.so.16`, `libwebp.so.7`, `libtiff.so.5`,
    `libjpeg.so.8`, `libjbig.so.0`, `libdeflate.so.0`.
  - **System / utility**: `libzip.so.4`, `libzstd.so.1`, `liblzma.so.5`, `libbz2.so.1.0`,
    `liblz4.so.1`, `libcrypto.so.3`, `libgcrypt.so.20`, `libdb-5.3.so`, `libreadline.so.8`,
    `libtinfo.so.6`, `libdbus-1.so.3`, `libsystemd.so.0`, `libpulse*.so.0` (+
    `libpulsecommon-15.99.so`), `libglib-2.0.so.0`, `libgobject-2.0.so.0`, `libgmodule-2.0.so.0`,
    `libffi.so.8`, `libpcre.so.3`, `libgomp.so.1`, `libcap.so.2`, `libapparmor.so.1`,
    `libasyncns.so.0`, `libbsd.so.0`, `libmd.so.0`, `libapparmor`, `libgomp`.
  - `usr/share/doc/<deb-package>/copyright` files confirm these are unpacked Debian/Ubuntu
    packages (libsndfile1, libfluidsynth3, libzip4, libzstd1, libapparmor1, …). **Confirmed**
- **Proposed:** DDL's "extract once" fallback must preserve `AppRun`, `usr/bin`, `usr/lib`,
  `usr/share/games/doom/dsda-doom.wad`, the icon and the desktop entry — i.e. extract the payload
  as-is and exec `AppRun`; do not copy the inner binary out of the tree
  (`OVERVIEW.md` §5 says the same).

---

## 3. Doomworld /idgames API

### 3.1 Availability — live calls are blocked from this host

- `GET https://www.doomworld.com/idgames/api/`,
  `GET https://www.doomworld.com/idgames/api/api.php?action=search&query=scythe&out=json`,
  `…?action=search&query=scythe` and `…?action=get&id=15507&out=json` all return **HTTP 403 with a
  Cloudflare "Just a moment…" interstitial**
  (`cf-mitigated: challenge`), with `curl` (multiple user agents, HTTP/1.1 and HTTP/2), with the
  `doomworld.com` host without `www`, and through the harness HTTP client. **Confirmed**
- Consequence for DDL: the API is **not reliably machine-callable**; a `403` challenge must be
  treated as a normal, expected failure. **Proposed:** never make idgames API access a hard
  dependency (D-010 recipes stay usable offline and from their own allowlisted mirror).
- The archived API responses used below were captured by the Wayback Machine **from the real
  service**, most recently on **2026-08-17** (see URLs in §3.3), so the field sets are current.
  **Confirmed**

### 3.2 Contract (idGames Public API, version 3, doc updated 2016-11-12)

Source: <https://web.archive.org/web/20250816213631id_/https://www.doomworld.com/idgames/api/>
(capture of `https://www.doomworld.com/idgames/api/`, 2025-08-16; doc "Version 3, Updated:
Nov. 12, 2016"; "idGames API created by Matt Tropiano. © 2012–2016 Doomworld.com. All Rights
Reserved."). **Confirmed (quoted)**

- Single endpoint: `GET|POST https://www.doomworld.com/idgames/api/api.php`. Parameters:
  `action` (required), `out` = `xml` (default) or `json`.
- Envelope: JSON is `{"content": …, "meta": {"version": 3}}`; XML wraps everything in
  `<idgames-response>`. Errors use `error.type`/`error.message`; warnings use
  `warning.type`/`warning.message`. Version is an integer since 2014-01-02.
- Actions: `ping`, `dbping`, `about`, `get` (`id=` or `file=`), `getparentdir`,
  `getdirs`, `getfiles`, `getcontents`, `latestfiles` (`limit`, `startid`), `latestvotes`
  (`limit`), `search` (`query` ≥ 3 chars, `type` ∈ {filename, title, author, email, description,
  credits, editors, textfile}, `sort` ∈ {date, filename, size, rating}, `dir` ∈ {asc, desc}).
- **File record fields — compact form** (`search`, `getfiles`, `getcontents`, `latestfiles`):
  `id`, `title`, `dir` (path without filename), `filename`, `size` (bytes), `age` (Unix seconds),
  `date` (`YYYY-MM-DD`), `author`, `email`, `description`, `rating`, `votes`, `url`
  (human page), `idgamesurl` (machine URL). **Confirmed**
- **`get` adds**: `credits`, `base`, `buildtime`, `editors`, `bugs`, `textfile` (the full
  accompanying TXT) and `reviews` (array with `username`, `vote`, `text`). **Confirmed**
- **No checksum/hash field exists in any response** — only `size`. **Confirmed**
- No rate limits, usage limits, quota, or required attribution are documented anywhere in the API
  page; the only statement is the blanket copyright line above. Note "There is a limit to how much
  is returned" for `search`/`latestfiles` (a `Limit Warning` is returned when it is hit) — the
  value is not published. **Confirmed (documented) / Open question (actual limit).**

### 3.3 Sample responses (trimmed; real captures, not hand-made)

`getcontents` for the directory that holds AUGER;ZENITH — capture 2026-08-17T20:38:57Z,
<https://web.archive.org/web/20260817203857id_/https://www.doomworld.com/idgames/api/api.php?action=getcontents&name=levels%2Fdoom2%2FPorts%2Fmegawads%2F&out=json>
(545 files; trimmed to the AUGER;ZENITH record):

```json
{ "content": { "file": [
  { "id": 19987, "title": "AUGER;ZENITH", "dir": "levels/doom2/Ports/megawads/",
    "filename": "dbp37_augzen.zip", "size": 12771737, "age": 1627275600,
    "date": "2021-07-26", "author": "Doomer Boards Community", "email": null,
    "description": "AUGER;ZENITH is a limit-removing project brought to you by us the Doomer
      Boards planet hackers. AUGER;ZENITH is also the ULTIMATE CYBERPUNK DOOM PROJECT. …",
    "rating": 0.0, "votes": 0,
    "url": "https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/dbp37_augzen",
    "idgamesurl": "idgames://levels/doom2/Ports/megawads/dbp37_augzen.zip" } ] },
  "meta": { "version": 3 } }
```

`search` — capture 2024-07-15T13:46:20Z,
`…?action=search&query=chest&type=filename&sort=date&out=json` (6 results; first two trimmed):

```json
{ "content": { "file": [
  { "id": 12021, "title": "The Community Chest Project", "dir": "levels/doom2/Ports/megawads/",
    "filename": "cchest.zip", "size": 4645046, "age": 1055480400, "date": "2003-06-13",
    "author": "Dale E. Harris", "description": "…", "rating": 3.2045, "votes": 132,
    "url": "https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/cchest",
    "idgamesurl": "idgames://levels/doom2/Ports/megawads/cchest.zip" },
  { "id": 13024, "title": "Community Chest 2", "filename": "cchest2.zip", "size": 8915095,
    "date": "2004-12-06", "author": "Various Authors (See below)", "rating": 4.0465,
    "votes": 172, "url": "…/megawads/cchest2", "idgamesurl": "idgames://…/cchest2.zip" } ] },
  "meta": { "version": 3 } }
```

`get` — capture 2024-07-15T14:47:37Z, `…?action=get&id=15156&out=json` (32 KB, 79 reviews;
`textfile`/`credits`/`reviews` elided):

```json
{ "content": { "id": 15156, "title": "Community Chest 3",
  "dir": "levels/doom2/Ports/megawads/", "filename": "cchest3.zip", "size": 12913516,
  "age": 1195624800, "date": "2007-11-21", "author": "Various (see Credits)", "email": "…",
  "description": "…", "credits": "…", "base": "…", "buildtime": "…", "editors": "…",
  "bugs": "…", "textfile": "<full TXT, ~11 KB>", "rating": 4.0, "votes": 106,
  "url": "https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/cchest3",
  "idgamesurl": "idgames://levels/doom2/Ports/megawads/cchest3.zip",
  "reviews": { "review": [ { "username": "…", "vote": 4, "text": "…" } ] } },
  "meta": { "version": 3 } }
```

### 3.4 Download URLs, mirrors, checksums

- Canonical human page: `<url>` = `https://www.doomworld.com/idgames/<dir>/<stem>`; the raw file is
  `https://www.doomworld.com/idgames/<dir>/<filename>`. The `idgamesurl` value
  (`idgames://<dir>/<filename>`) is the protocol form, not an HTTP URL. **Confirmed**
- The archive master is `ftp.gamers.org` / <https://www.gamers.org/pub/idgames/>; the same
  `<dir>/<filename>` path exists on every mirror, so a mirror URL is
  `<mirror-base>/<dir>/<filename>`. **Confirmed**
  (`https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/dbp37_augzen.zip` and
  `https://dsdarchive.com/files/wads/doom2/3565/dbp37_augzen.zip` both return
  `Content-Length: 12771737`.)
- Mirror list published by the archive itself
  (<https://www.gamers.org/pub/idgames/README>, fetched 2026-10-16) — **Confirmed (quoted)**:

  ```text
  Germany (*)   ftp://ftp.fu-berlin.de/pc/games/idgames/        New York    https://youfailit.net/pub/idgames/
  Germany (*)   https://ftp.fu-berlin.de/pc/games/idgames/      Kentucky    https://lethe.chinstrap.org/idgames/
  Michigan      https://mirrors.lug.mtu.edu/idgames/            Texas       https://planetzdoom.com/idgames/
  Sweden        https://ftpmirror.infania.net/pub/idgames/      Sweden      https://mirror.braindrainlan.nu/pub/idgames/
  New Zealand   https://files.xvertigox.com/idgames/
  (*) Primary download servers - please use these instead of the master server - ftp.gamers.org for uploads only
  ```
- **No checksum is provided by idgames, in the API or alongside the files.** **Confirmed**
  → recipes must pin SHA-256 values DDL computed at review time (D-010), and the review has to
  re-hash if the archive ever re-uploads a file.
- Usage guidance relevant to DDL (same README, quoted): *"*** ATTENTION: To reduce the load on this
  machine and provide faster access for all users, please check the following list for a mirroring
  site closer to you and USE it."* and *"remember to turn on binary mode"*. The archive also
  publishes `README.legal`, `README.mirrorers` (542 B), `README.INCOMING` (8.9 KB), `UPLTEMPL.TXT`
  and rolling `LAST.24hours`/`LAST.7days`/`LAST.14days`/`LAST.30days`/`LAST.60days` file lists.
  **Confirmed (listing; only README and README.legal were read).**

---

## 4. Terms and licensing

### 4.1 DSDA-Doom

- The source repository has **no root `LICENSE`/`COPYING` file** (`raw.githubusercontent.com/…/master/LICENSE`
  → 404) and GitHub reports `license: null` for `dsda-org/dsda-doom`; the licence text lives at
  **`prboom2/COPYING`**, which is the **GNU General Public License, Version 2, June 1991**
  (18077 B). **Confirmed**
- The shipped AppImage contains that same file at `usr/share/doc/dsda-doom/COPYING` (18077 B).
  **Confirmed**
- **Proposed:** because upstream ships GPL-2.0 *unmodified in the archive payload* and DDL merely
  downloads and executes it, DDL's own GPL-3.0-or-later licence (D-001) is compatible for
  distribution of DDL itself; DDL must not relicense or repackage the engine. DDL should credit
  DSDA-Doom/PrBoom+ and link to the upstream release page (`OVERVIEW.md` §18). Legal review of the
  "GPL-2.0-only vs GPL-2.0-or-later" reading of the `prboom2/COPYING` boilerplate is an **open
  question** for the owner.

### 4.2 Doomworld / idgames

- idgames archive legal notice (<https://www.gamers.org/pub/idgames/README.legal>, fetched
  2026-10-16) — **Confirmed (quoted in full)**:

  ```text
  *  The material at this ftp site is not placed on CDROM or sold for profit, but we have no control over mirroring
  *  sites making such use of the materials in this archive.  Do not upload anything to this site that has
  *  distribution restrictions that prohibit this.
  *  If you upload any software that is not completely your own work,
  *  please ensure that the file can be freely redistributed.
  *  If you upload any software that does not meet these conditions
  *  you are BREAKING THE LAW, and either the author or 3D Gamers
  *  Archives may take legal action against you.
  ```
- idgames API: "© 2012–2016 Doomworld.com **All Rights Reserved**", created by Matt Tropiano; no
  terms of use, quota or attribution clause is documented on that page. **Confirmed**
- Doomworld's site-wide terms/FAQ pages could not be read from this host (Cloudflare 403), so
  Doomworld's own ToS for API use and hotlinking is an **Open question**. **Confirmed (unavailable)**
- **Proposed (DDL policy):** treat /idgames as a *download source* and a *metadata provider*, not as
  a bundled dependency: always link the `url` page for attribution, prefer a mirror for the actual
  bytes, send a descriptive `User-Agent`, cap request rate (e.g. ≤ 1 request/second, no retries
  before a minute), cache metadata on disk, and never present idgames data as DDL's own.
  Hotlinking: the archive's own README asks clients to spread load across mirrors, so DDL should
  ship *several* allowlisted mirror bases per file and pick one — do not hard-code
  `www.doomworld.com/idgames/…` as the only URL. **Proposed**

### 4.3 AUGER;ZENITH

The release TXT (`DBP37_AUGZEN.txt`, in the zip) carries the permission text verbatim
(**Confirmed (quoted)**):

```text
* Copyright / Permissions *
Authors MAY use the contents of this file as a base for modification or
reuse. Provide credit if you do use anything from this WAD, and feel free to
ask about any of it. Permissions have been obtained from original authors
for any of their resources modified or included in this file.
You MAY distribute this file, provided you include this text file, with no
modifications.  You may distribute this file in any electronic format (BBS,
Diskette, CD, etc) as long as you include this file intact.
```

**Proposed:** DDL must (a) never modify or repackage the archive, (b) always ship
`DBP37_AUGZEN.txt` next to the extracted data and show the credits in the UI (the TXT lists 30+
external credits: id Software, Raven, 3D Realms, Team TNT, NeoWorm, Princess Prozac, …), and
(c) if the game data is ever displayed on the Deck, keep the credit screen reachable.

---

## 5. Reference mapset — AUGER;ZENITH (Doomer Boards Project 37)

Primary source thread: <https://doomer.boards.net/thread/2176/dbp37-auger-zenith> (post 1,
2021-07-03, "Optional Blaster pistol & chaingun weapon — Download Add-on (from TSPG)"; the thread
is locked, 5 pages; only page 1 was read). **Confirmed**

### 5.1 Download locations

| location | URL | status |
| --- | --- | --- |
| /idgames entry (link in thread post 1; "Added to the archive" by 40oz) | <https://www.doomworld.com/idgames/levels/doom2/Ports/megawads/dbp37_augzen> | **canonical** — file `levels/doom2/Ports/megawads/dbp37_augzen.zip`, id **19987**, dated **2021-07-26**, size **12771737 B** (API + mirror agree) |
| idgames mirrors | `https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/dbp37_augzen.zip` (downloaded and hashed here); any host from §3.4 | **mirror** (byte-identical length at gamers.org and dsdarchive) |
| DSDA archive | <https://dsdarchive.com/files/wads/doom2/3565/dbp37_augzen.zip> | mirror, `Content-Length: 12771737`, `Last-Modified: 2021-11-10` |
| Doomshack | <https://doomshack.org/uploads/DBP37_AUGZEN.zip> | third-party host (not fetched) |
| ModDB page | <https://www.moddb.com/mods/dbp37-augerzenith> | third-party host (not fetched) |
| TSPG (single WAD) | `https://allfearthesentinel.com/zandronum/download.php?file=dbp37_augzen.wad` | third-party; serves one `.wad`, not the zip (not fetched) |
| Optional add-on, TSPG (link in thread post 1) | `https://allfearthesentinel.com/zandronum/download.php?file=dbp37_alternateblaster.wad` | **fetched**: 20145 B, `sha256=8c2fe3a4…` — identical to the copy inside the archive |
| Optional add-ons, bundled | `DBP37_Addons.zip` inside the canonical zip | **authoritative** (see §5.4) |
| Add-on, MediaFire (post by 40oz, page 1) | `DBP37_AZblaster.wad` (MediaFire) | not fetched; same description as the TSPG file |

**Proposed:** canonical source for DDL = the /idgames entry (stable, mirrored, has an id and a
documented path); DDL should still store 2–3 allowlisted mirror bases and verify the SHA-256 after
download. Note that third-party hosts (TSPG, Doomshack, MediaFire, ModDB) may serve *loose WADs*
rather than the archive; if DDL ever uses them it must pin the hash of that exact loose file.

### 5.2 Files in the release (all hashes SHA-256, computed here)

| file | bytes | SHA-256 |
| --- | --- | --- |
| `dbp37_augzen.zip` (archive as published) | 12771737 | `7dfb36408a9cd33a6ef429ada3aa68b4745be6abba03b0a3236dacb03872aeb9` |
| ├ `DBP37_Addons.zip` (2021-07-13) | 62421 | `ff867d9d0af2ad8205c881472e9e4092cb4e864b23b01c1f31b8d8bd45c5d6a3` |
| ├ `DBP37_AUGZEN.txt` (2021-07-23) | 6212 | `674c33641557b18aa17d10628ba3d567970025b2d916dad3a87d1e9eb5dc3e90` |
| └ `DBP37_AUGZEN.wad` (2021-07-22) | 48797931 | `d356de75acdb9c969175bdc3910839fb805415b7244fd7e65d05a9110f53c95a` |

Zip CRCs as reported by `unzip -v`: `DBP37_Addons.zip` `04929a5e`, `DBP37_AUGZEN.txt` `cb830b19`,
`DBP37_AUGZEN.wad` `4447ef97`. **Confirmed** The release is a single PWAD plus its TXT — there is
no separate music/graphics/resource PWAD and no IWAD data. **Confirmed**

### 5.3 IWAD, load order, lump contents, target port and complevel

- **IWAD: `doom2.wad` (DOOM II).** The TXT says `Game: DOOM 2: Hell on Earth`, `Map #: MAP01 -
  MAP22`, and the WAD's map markers are `MAP01`…`MAP22` (22 maps: 21 levels + the credits map).
  **Confirmed**
- **Load order:** exactly one mandatory file, so the base order is `-iwad doom2.wad -file
  DBP37_AUGZEN.wad` (plus the optional add-on last, §5.4). No DEH/BEX file is loaded separately —
  the patch is an internal `DEHACKED` lump. `Other files required: None`. **Confirmed**
- Lump directory (`wadinfo.py`, read-only): 1132 lumps, **PWAD**. Before `MAP01`:
  `PLAYPAL` (10752 B — custom palette), `DECORATE` (3639 B), `SNDINFO` (32 B),
  `DEHACKED` (6599 B), `GLDEFS` (6855 B), `MAPINFO` (5326 B), `UMAPINFO` (4963 B),
  `DEMO1/2/3`, `TITLEPIC`, `INTERPIC`, `CREDIT`, `HELP`, menu graphics, `CWILV00`…`CWILV21`,
  status-bar graphics, `D_*` music lumps, `DS*` sounds, then `SS_START`…`SS_END` sprite range,
  `FF_START`…`FF_END` flats and `PP_START`…`PP_END` patches. **Confirmed**
- Every map `MAP01`…`MAP22` contains exactly the standard ten lumps
  (`THINGS, LINEDEFS, SIDEDEFS, VERTEXES, SEGS, SSECTORS, NODES, SECTORS, REJECT, BLOCKMAP`) — no
  UDMF `BEHAVIOR`, no Boom-only map lumps. **Confirmed**
- Unusual but legal: `TEXTURE1` (52764 B) and `PNAMES` (7588 B) sit **after** `MAP22`'s
  `BLOCKMAP`, i.e. at the very end of the WAD. **Confirmed** → **Proposed:** DDL's recipe
  validator must not assume a fixed lump order or that textures precede maps.
- `COMPLVL` lump: **absent**. `UMAPINFO`: present; its keys are only
  `levelname, levelpic, next, skytexture, partime, music, intertext, interbackdrop` — **no
  `complevel`**. (A parallel ZDoom-style `MAPINFO` also exists; `dsda-doom` **removed ZDoom
  MAPINFO support in v0.30.0** per that release's notes, so it is inert here, as are `DECORATE`,
  `GLDEFS` and `SNDINFO`, which DSDA-Doom does not implement.) **Confirmed**
- `DEHACKED` (6599 B): "Patch File for DeHackEd v3.0", "Created with WhackEd4 1.2.0 BETA",
  `Doom version = 19`, `Patch format = 6`; sections `Short/Long/Trigger Ambience`,
  `Miscellanous`, `Normal Text`; it retasks existing things into ambience emitters (e.g.
  `Thing 25`, `Thing 116` "Broken Glass" as a dead lost soul with `Height = 2031616`), defines
  frames 745/766/777, and rewrites `HUSTR_*` level names. No MBF21 codepointers are introduced.
  **Confirmed (dump read, not executed)**
- Author-stated target (`DBP37_AUGZEN.txt`): `Advanced engine needed: Limit Removing`;
  `May Not Run With: Vanilla and Chocolate Doom`; `Tested With: Crispy Doom, GZDoom, PRBoom+,
  Doom Retro, Zandronum, dsda-doom, LZDoom` → **DSDA-Doom is an author-tested target**
  (**Confirmed**). The TXT does not name a compatibility level.
- **Open question / Needs Deck Validation:** the recipe must pick a complevel because the WAD
  declares none (no `COMPLVL`, no `UMAPINFO complevel`). A limit-removing DOOM II mapset is
  normally played at **complevel 2**; DDL should record `2` as a **Proposed** default, label it in
  the recipe's evidence field and confirm it on the Deck before the recipe is promoted.

### 5.4 The optional add-on (owner-requested UI toggle)

The release is **not** documented as having an add-on inside its own TXT — `DBP37_AUGZEN.txt` says
`Other files required: None` and never mentions add-ons. The add-on evidence lives in the thread
(post 1: *"Optional Blaster pistol & chaingun weapon"* → TSPG link) and in
`DBP37_Addons.zip` + its `readme.txt`, which is bundled **inside** the canonical zip.
**Confirmed (both sources quoted below)**

`DBP37_Addons.zip` (readme.txt, verbatim, 388 B):

```text
DBP37_AZblaster: Includes Zombieman with red muzzle flash. Chaingun's muzzle flash is changed to
match the pistol's style as well as including a new bullet sound.
DBP37_EDGEBreakingGlass: Broken glass effects for the EDGE sourceport.
DBP37_DeckardBlaster: Same as DBP37_AlternateBlaster but with a high-quality sound bite of
Deckard's blaster gun from Blade Runner for the pistol.
```

| add-on file (inside `DBP37_Addons.zip`) | bytes | SHA-256 | lumps |
| --- | --- | --- | --- |
| `DBP37_AlternateBlaster.wad` | 20145 | `8c2fe3a4c65838e333dd9233b806a0e469e4ad9ccab704113de10b05220274e3` | `DSPISTOL` (8013), `SS_START`…`POSSF1, POSSF2F8, POSSF3F7, POSSF4F6, POSSF5, CHGFA0, CHGFB0`…`SS_END` |
| `DBP37_DeckardBlaster.wad` | 81484 | `5e9db49fd849708b61df6fd9826eb99435b16bfde1779f6f7f5f14e3bb9437e5` | same sprites as above + `DSPISTOL` (69352) |
| `DBP37_EDGEBreakingGlass.wad` | 2181 | `fdee1d5a99d467216659c3108f17e360b0275db6c708cba377b43a30611828db` | `DDFATK, DDFTHING, DDFSFX, DDFLINE, DDFSWTH` (EDGE DDF only) |
| `readme.txt` | 388 | `f16f2c555e654240e73dca5cdce5f96254a1a9624f271da2ebd8048b006e28bc` | — |
| TSPG `dbp37_alternateblaster.wad` (fetched separately) | 20145 | `8c2fe3a4…` — **byte-identical to the bundled `DBP37_AlternateBlaster.wad`** | — |

- **Identity of "the" add-on:** the thread links exactly one add-on, the blaster/chaingun weapon
  package, and the file it serves is byte-identical to the bundled `DBP37_AlternateBlaster.wad`
  (also called `DBP37_AZblaster.wad` in the readme and in 40oz's MediaFire post). **Confirmed** →
  the add-on DDL should expose is `DBP37_AlternateBlaster.wad`. Which name to show, and whether the
  owner wants `DBP37_DeckardBlaster.wad` as a *second* toggle, is an **open question**.
- **Confirmation of "loaded after the main PWAD":** neither the release TXT nor the add-on readme
  states a load order in words; the technical evidence is that all three add-ons are pure
  *replacement* PWADs (sprites `POSSF*`/`CHGF*` and the `DSPISTOL` sound that `DBP37_AUGZEN.wad`
  itself provides) and contain no maps, no `DEHACKED`, no `MAPINFO`/`UMAPINFO`. A later PWAD
  overrides an earlier one's lumps, so **`-file DBP37_AUGZEN.wad DBP37_AlternateBlaster.wad`**
  (add-on last) is the correct order. **Confirmed (lump analysis) / Proposed (the exact order in
  the recipe) / Open question (authors never wrote it down).**
- **Does it change the IWAD or complevel?** No — it has no maps, no `COMPLVL`, no `UMAPINFO`, no
  `DEHACKED`; it only replaces sprites and one sound. **Confirmed**
- **Conflicts:** `DBP37_AlternateBlaster.wad` and `DBP37_DeckardBlaster.wad` replace the *same*
  sprites and `DSPISTOL`, so they are mutually exclusive — DDL must not enable both.
  `DBP37_EDGEBreakingGlass.wad` is EDGE-only (DDF lumps) and is inert under DSDA-Doom; it should
  not be offered as a DSDA-Doom toggle. The add-on intentionally overrides the base WAD's pistol
  sound and zombieman/chaingun muzzle-flash graphics — that is its purpose, not a conflict.
  **Confirmed**

### 5.5 Cropped facts for the recipe

| recipe field | value |
| --- | --- |
| Title | AUGER;ZENITH (Doomer Boards Project 37) |
| Package | `dbp37_augzen.zip`, 12771737 B, `7dfb3640…` |
| Source | /idgames id 19987, `levels/doom2/Ports/megawads/dbp37_augzen.zip` (+ mirrors) |
| Files to load (order) | `DBP37_AUGZEN.wad`, then optional `DBP37_AlternateBlaster.wad` |
| IWAD | `doom2.wad` (DOOM II) |
| Complevel | not declared upstream → **Proposed** 2, **Needs Deck Validation** |
| Maps | MAP01–MAP22 (21 levels + credits map) |
| Credits/licence | `DBP37_AUGZEN.txt` (must ship intact, unmodified) |
| Authors' tested ports | Crispy Doom, GZDoom, PRBoom+, Doom Retro, Zandronum, **dsda-doom**, LZDoom |

---

## 6. Implications for DDL (Proposed)

These are DDL decisions, not upstream facts; each one is deliberately *deducible* from the evidence
above.

### 6.1 Engine manager (D-007, `OVERVIEW.md` §5)

1. **Discovery.** `GET /repos/dsda-org/dsda-doom/releases?per_page=10`, filter
   `draft == false && prerelease == false`, then select the Linux x86-64 AppImage by pattern
   (`-linux.*\.appimage$`, case-insensitive) rather than by an exact filename, because upstream has
   renamed the asset at least twice (§1.2). **Proposed**
2. **Support window.** Offer only releases on DDL's reviewed list (D-007). Practically that means
   v0.29.4+ / v0.30.x, because **only releases ≥ v0.29.1 have a `digest`** (§1.3) — older ones
   would need a hash pinned from a manual review. **Proposed**
3. **Verification order.** (a) check the downloaded size against `assets[].size`; (b) compute
   SHA-256 and compare with the *DDL-reviewed* hash; (c) if the release is only in GitHub's
   "unvalidated" bucket, additionally compare with the API `digest` if it is non-null; (d) never
   compare only against the API `digest` — it comes from the same origin as the file. There is no
   signature to check: tags are unsigned and the AppImage's `.sha256_sig`/`.upd_info` sections are
   zero (§1.3, §2.2). **Proposed**
4. **Caching/offline.** Store the releases JSON + weak ETag + `fetched_at` in JSON user state and
   always render from cache first; treat HTTP 403/429 and any network error as "offline with
   cached data". Refresh on demand, at most once a day, with `per_page` fixed (§1.4). **Proposed**
5. **Downloads.** Follow the redirect to the signed `release-assets.githubusercontent.com` URL at
   download time, write to a temp file, hash, then atomically move into
   `…/engines/dsda-doom/<version>/`, `chmod +x`, and keep the previous version for rollback
   (`OVERVIEW.md` §5 steps 3–6). **Proposed**
6. **Smoke test.** `--help`/`--version` are *not* confirmed here (nothing was executed); confirm
   the safe flag on the Deck before wiring the smoke test. **Needs Deck Validation**
7. **FUSE fallback.** Implement `APPIMAGE_EXTRACT_AND_RUN=1` / `--appimage-extract` extraction into
   a versioned directory and exec `AppRun`, preserving `usr/lib` and
   `usr/share/games/doom/dsda-doom.wad` (§2.4). **Proposed / Needs Deck Validation**

### 6.2 Catalog recipes (D-010)

1. **Packages, not files.** A recipe pins *packages* (`dbp37_augzen.zip`) with `url`, mirror
   bases, `size`, `sha256`, and the list of files it contains with *their* sizes/hashes, so the
   validator can prove that an extract produced the expected member bytes. **Proposed**
2. **Mirrors and hashes.** For /idgames packages: store the idgames `id`, `dir` and `filename`,
   the human `url` for attribution, and 2–3 allowlisted mirror bases; because idgames supplies no
   checksum (§3.4), the SHA-256 is computed by the reviewer from the canonical zip and re-checked
   by CI/tests from fixtures. **Proposed**
3. **Optional components.** D-010 already allows optional components with their own load-order
   position. AUGER;ZENITH shows the shape it must support: an optional component that (a) comes
   *from inside the main package* (`DBP37_Addons.zip` → `DBP37_AlternateBlaster.wad`), (b) is also
   downloadable separately with an identical hash, (c) declares an exclusive group with
   `DBP37_DeckardBlaster.wad`, and (d) is appended after the mandatory files. Suggested fields:
   `id`, `label` (from the add-on readme), `enabled_by_default = false`, `source` (either
   `package_member` with a path or a `url` with a hash), `load_order = "after:main"`,
   `exclusive_group`, and `notes` (e.g. "EDGE-only, not applicable to DSDA-Doom"). **Proposed**
4. **Never model a name.** The same add-on appears as `DBP37_AlternateBlaster.wad`,
   `dbp37_alternateblaster.wad` and `DBP37_AZblaster.wad`; recipes must key on hashes and
   `package_member` paths, and the UI must show the final load order before launch (D-010).
   **Proposed**
5. **Evidence field.** Each recipe should record *how* it was verified (this document's commands,
   the idgames id, the tested port list) and any unresolved item — for AUGER;ZENITH, the complevel
   and the identity of "the" add-on. **Proposed**
6. **Validator rules learned here.** Do not assume WAD lump order (`TEXTURE1`/`PNAMES` at the end
   of AUGER;ZENITH), do not require a `COMPLVL` lump, tolerate ZDoom-only lumps (`MAPINFO`,
   `DECORATE`, `GLDEFS`, `SNDINFO`) which modern DSDA-Doom ignores, and treat edits to
   `Copyright / Permissions` text as a recipe error (§4.3). **Proposed**
7. **Linking vs downloading.** /idgames' own guidance is to spread load across mirrors (§3.4) and
   its API may be Cloudflare-challenged at any time (§3.1), so the catalog must keep working
   without the API: the recipe is static, and the API — if used at all — is only a
   discovery/refresh aid. **Proposed**

---

## Open questions

1. **Complevel for AUGER;ZENITH.** Not declared by the WAD (`COMPLVL` absent, no `UMAPINFO
   complevel`) and not stated in the TXT, which only says "Limit Removing". Proposed default 2;
   needs a Deck test and owner sign-off.
2. **Which add-on is "the" optional add-on.** The thread links one blaster/chaingun add-on
   (`DBP37_AlternateBlaster.wad`, a.k.a. `DBP37_AZblaster.wad`, hash `8c2fe3a4…`); the bundled
   `DBP37_Addons.zip` also contains `DBP37_DeckardBlaster.wad` (same sprites, different pistol
   sound) and an EDGE-only glass add-on. Is Deckard a second toggle, or out of scope?
3. **Author's load-order statement for the add-on.** No upstream text says "load after the main
   WAD"; the conclusion here rests on lump-level analysis. If the owner wants an author quote, the
   remaining thread pages (2–5) and the Trello board linked in post 2 should be read.
4. **Doomworld's own terms for API use and hotlinking.** The site pages are behind a Cloudflare
   challenge from this host, so the only written terms found are the API page's copyright line and
   the archive's `README.legal`. Someone with browser access should confirm the Doomworld ToS and
   any API rate limit before DDL ships an /idgames-driven feature.
5. **`search`/`latestfiles` result limit** and whether the API sets any rate limit — undocumented.
6. **API availability.** The API returned 403 (bot challenge) for every live attempt; is that
   host-specific, or would Deck users be blocked too? Needs a check from a normal desktop browser
   and from the Deck.
7. **`dbp37_augzen.wad` on Doomshack/TSPG** (loose single WAD) — not fetched; if DDL ever offers
   those hosts as alternatives, their hashes must be pinned separately.
8. **`immutable` release flag** — semantics undocumented; do not branch on it.
9. **GPL-2.0-only vs GPL-2.0-or-later** reading of `prboom2/COPYING`, for DDL's credit/licence
   text.
10. **Safe smoke-test flag** for DSDA-Doom (`--help`?) — nothing was executed on this host
    (`AGENTS.md`: never execute downloaded archives), so this must be confirmed on the Deck.
11. **Whether the AppImage can mount in SteamOS Gaming Mode / Steam Linux Runtime** (FUSE,
    `fusermount`), and whether the extract-and-run fallback is needed there.
12. **Third-party hosts** (ModDB, Doomshack, MediaFire, TSPG) — terms, stability and hotlinking
    policy not reviewed.

## Reproducing this research

```sh
mkdir -p .scratch && cd .scratch
gh api -X GET /repos/dsda-org/dsda-doom/releases -f per_page=10 > releases.json
curl -sL -o dsda-doom-0.30.1-linux-x86_64.appimage \
  https://github.com/dsda-org/dsda-doom/releases/download/v0.30.1/dsda-doom-0.30.1-linux-x86_64.appimage
shasum -a 256 dsda-doom-0.30.1-linux-x86_64.appimage     # b06adf3f…  (== API digest)
file dsda-doom-0.30.1-linux-x86_64.appimage
xxd -l 16 dsda-doom-0.30.1-linux-x86_64.appimage          # ELF + "AI\x02" at offset 8
python3 squashls.py dsda-doom-0.30.1-linux-x86_64.appimage 944632 12   # payload listing
curl -sL -o dbp37_augzen.zip \
  https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/dbp37_augzen.zip
shasum -a 256 dbp37_augzen.zip                            # 7dfb3640…
unzip -o dbp37_augzen.zip -d augzen && (cd augzen && unzip -o DBP37_Addons.zip -d ../addons)
python3 wadinfo.py augzen/DBP37_AUGZEN.wad
```

No files outside `.scratch/` were created or modified by these commands, and no archive content was
executed.

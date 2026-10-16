# Reference research — SIGIL and SIGIL II (John Romero)

**Status:** desk research for D-010 reference recipes · **Gathered:** 2026-10-16 (UTC)
**Author workstream:** Content Schema / Content Discovery (D-010 reference recipes)
**Scope:** facts needed to author a DDL recipe for **SIGIL** and **SIGIL II**, and their optional
soundtracks. No code and no catalog entry is produced here.

Labels used throughout:

- **Confirmed** — verified from an official source, upstream source code, or a downloaded archive
  (evidence recorded below).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method (reproducible)

All downloads went to `.scratch/` in this worktree (git-ignored, never committed). Nothing was
executed from the archives; the WADs were only parsed as data by `.scratch/wadinfo.py` (reads the
12-byte WAD header and the lump directory) and by `unzip`/Python `zipfile`. Raw evidence kept in
`.scratch/`: the two official ZIPs, the two idgames-mirror ZIPs, their extracted trees, and the
lump inspector.

```sh
curl -sSL -o .scratch/SIGIL_V1_23.zip   "https://romero.com/s/SIGIL_V1_23-8fh4.zip"
curl -sSL -o .scratch/SIGIL_II_V1_0.zip "https://romero.com/s/SIGIL_II_V1_0.zip"
shasum -a 256 .scratch/*.zip
```

## Quick reference

| | SIGIL | SIGIL II |
| --- | --- | --- |
| Official page | <https://romero.com/sigil> | same page (no separate SIGIL II page) |
| Free version (current) | **1.23**, released 2025-08-07 | **1.0**, released 2023-12-10 |
| Free archive | `https://romero.com/s/SIGIL_V1_23-8fh4.zip` | `https://romero.com/s/SIGIL_II_V1_0.zip` |
| Archive size | 3,270,225 bytes | 2,012,026 bytes |
| Archive SHA-256 | `57c35a6bcd722433fc97344314e52c06c91fdfde33b18c6d1c14a4d29e229105` | `41741ce797e6faf9ebd0e3d577b5eaf9a648b64d850fbd741c1b9c28d8461017` |
| Main WAD (free) | `SIGIL_V1_23.wad` (E5M1–E5M9, MIDI) | `SIGIL_II_V1_0.WAD` (E6M1–E6M9, MIDI) |
| Compat WAD | `SIGIL_COMPAT_V1_23.wad` (replaces E3M1–E3M9) | none |
| IWAD | `doom.wad` (registered 1.9) **or** `doomu.wad` (Ultimate 1.9ud) | same |
| Complevel (Proposed) | 3 (Ultimate Doom) | 3 (Ultimate Doom) |
| Episode recognised via | UMAPINFO (DSDA-Doom) | UMAPINFO (DSDA-Doom) |
| Optional soundtrack | paid Buckethead MP3 WAD (gated) | paid THORR MP3 (gated) |
| Redistribution | permitted with the text file intact | permitted with the text file intact |

---

# SIGIL

## 1. Official sources

### Download pages

- **Canonical page:** <https://romero.com/sigil> (**Confirmed**, fetched 2026-10-16). The page header
  is simply "SIGIL"; it also hosts SIGIL II. `https://si6il.com` currently **redirects** to
  `https://romero.com/sigil` (same content) — no separate terms page was found there.
- The page offers the free megawad, two paid bundles, wallpapers, and links to older free versions.

### The current free release — v1.23

- Page text: *"The free SIGIL v1.23 Megawad is now available! Download this small 3.3MB file…"*
  (**Confirmed**).
- User-facing URL: <https://romero.com/s/SIGIL_V1_23-8fh4.zip> (**Confirmed** — note the `-8fh4`
  suffix, not `SIGIL_V1_23.zip`).
- Resolves (HTTP 302 → 200) to a Squarespace CDN URL with a changeable token path:

  ```text
  https://static1.squarespace.com/static/5fef3318f8ce9b562500206e/t/6895223f13bb800a47db1c1f/1754604095555/SIGIL_V1_23.zip
  ```

  The `static1.squarespace.com` path contains a site id, a content hash and a timestamp. **Proposed:**
  DDL must treat the token URL as unstable and always re-verify the SHA-256; never hardcode the CDN
  URL — follow the redirect from `romero.com/s/…`.

### Version history (as published)

| Version | Date | Notes |
| --- | --- | --- |
| 1.0 | 2019-05-01 | first free release |
| 1.1 | 2019-05-31 | |
| 1.2 | 2019-09-20 | |
| 1.21 | (free archive still linked) | the version the paid "SIGIL + Buckethead" bundle advertises (see open questions) |
| **1.23** | **2025-08-07** | **current free release** |

Dates are **Confirmed** from the bundled `SIGIL_V1_23.txt` ("Release date"). The page links older free
archives: `SIGIL_V1_0.zip`, `SIGIL_V1_1.zip`, `SIGIL_V1_2.zip`, `SIGIL_v1_21.zip` (**Confirmed**; only
v1.23 was downloaded here).

### idgames mirror

- idgames hosts both titles at `…/idgames/levels/doom/Ports/s-u/` (**Confirmed** via the mirror
  <https://youfailit.net/pub/idgames/levels/doom/Ports/s-u/>):
  `sigil_v1_23.zip` (3,270,168 bytes) and `sigil_ii_v1_0.zip` (2,012,056 bytes), plus `.txt` entries.
- The idgames ZIPs are **not byte-identical** to the romero.com ZIPs:
  - `sigil_v1_23.zip` SHA-256 `4e3fef70a8a20abbf91c48562334f69d631c7b483a020642044dd4da9df7cf76`
  - The **WAD payloads are byte-identical** (see §2), but the bundled `SIGIL_V1_23.txt` differs —
    idgames replaces the author e-mail with `[redacted]`.
  - **Confirmed** by extracting both archives and comparing SHA-256 per file (table below).
- **Implication (Proposed):** a recipe must bind the SHA-256 to a specific source package; the two
  mirrors are interchangeable for loading but not for hash verification. Prefer romero.com.

### Archive format (what DDL's extractor must handle)

**Confirmed** from `zipfile` inspection of `SIGIL_V1_23.zip`:

- Standard ZIP, "at least v2.0 to extract"; **all four entries are Deflate** (`compress_type = 8`),
  one stored CRC each; no encryption (general-purpose bit 0 clear).
- **Flat layout** — no nested folders, no directory entries, no symlinks, no path traversal.
- **ASCII-only file names**, mixed case preserved exactly. No spaces.
- No executable payloads.
- Timestamps are 2025-08-07 12:00 (ZIP local time).

### Contents (full internal paths)

| Path in archive | Size (bytes) | SHA-256 | Role |
| --- | ---: | --- | --- |
| `SIGIL_V1_23.wad` | 4,640,555 | `29c3b8400657be4546eab1878e2883ffc50771c6be575677b1be5a0d0430f55d` | main episode (E5M1–E5M9), MIDI music |
| `SIGIL_COMPAT_V1_23.wad` | 4,634,503 | `a79416b5e1a64b2946c09dcc5704f5d1448728b1afa2d761ccc067cd9ecce2b5` | compat: maps replace E3M1–E3M9 |
| `SIGIL_V1_23.txt` | 5,001 | `d1445fb883296f69dde7dbd300032bb0c8804f6a6a4b944f8fbb5231bcb79b9e` | README/copyright (romero.com copy) |
| `SIGIL_README_V1_23.txt` | 10,580 | `9d3877d88eb49a0475300f465b1c2771c847ac0e83550d31c74de24018fbef6c` | install/play instructions |

(The idgames copy differs only in `SIGIL_V1_23.txt`, SHA-256
`502c288e13783d8b97aa1116874c039ef8b5368a1ba77da5a00fd7830dd1ee3f`.)

## 2. Variants

Every WAD in the release and its purpose (**Confirmed** from the bundled texts and lump inspection):

| WAD | Where | Purpose |
| --- | --- | --- |
| `SIGIL_V1_23.wad` | free (both mirrors) | main: adds **episode 5** (E5M1–E5M9), free **MIDI** soundtrack |
| `SIGIL_COMPAT_V1_23.wad` | free (both mirrors) | compat: the same levels as **E3M1–E3M9**, i.e. an **Episode 3 replacement** for ports that do not support E5 |
| `SIGIL_V1_23_REG.wad` | paid only | same levels with the **Buckethead MP3** soundtrack embedded |
| `SIGIL_SHREDS_COMPAT.wad` | paid only | MP3 soundtrack package for the **compat** path (load after the compat WAD) |

Bundled `SIGIL_README_V1_23.txt` states the REG/SHREDS files come with the paid fan boxes / digital
download; the free archive ships only the two free WADs (**Confirmed**).

### Lump-directory inspection (read-only, `.scratch/wadinfo.py`)

`SIGIL_V1_23.wad` — PWAD, 146 lumps:

- Maps **E5M1…E5M9**, each a complete vanilla map group (THINGS, LINEDEFS, SIDEDEFS, VERTEXES, SEGS,
  SSECTORS, NODES, SECTORS, REJECT, BLOCKMAP). **No map format beyond vanilla** (no BEHAVIOR/ZNODES).
- `DEHACKED` (289 bytes): `Patch File for DeHackEd v3.0`, `Doom version = 21`, `Patch format = 6`;
  only `[PARS]` (E5M1–E5M9 par times). **No thing/property changes.**
- `UMAPINFO` (1,859 bytes): declares `map E5M1 … E5M9` and, on E5M1,
  `Episode = "M_EPI5", "SIGIL", "S"`. **This is how a modern port adds the episode.**
- Also `MAPINFO` (ZDaemon), `ZMAPINFO` (ZDoom), `EMAPINFO` (Eternity) — alternative metadata for
  other ports, not used by DSDA-Doom.
- `LANGUAGE`, `TEXTURE1/2`, `PNAMES`, graphics, and `DEMO1…DEMO4` (built-in demos).
- Music lumps `D_E5M1…D_E5M9`, `D_INTRO`, `D_INTER` — **all MIDI** (`MThd` magic). No MUS, no
  OGG/MP3 in the free WAD.
- **No `COMPLVL` lump.**

`SIGIL_COMPAT_V1_23.wad` — PWAD, 158 lumps: maps **E3M1,E3M2,…,E3M7,E3M9,E3M8**, same vanilla map
groups; `DEHACKED` (997 bytes) with `[CODEPTR] FRAME 631 = NULL`, `[PARS]` and `[STRINGS]`
(E3 interludes/level names); MIDI lumps `D_E3M1…D_E3M9`. **No `UMAPINFO`/`MAPINFO`** — it relies on
replacing the IWAD's own E3 episode.

## 3. Requirements

- **IWAD (Confirmed, from both bundled texts):** the registered `DOOM.WAD` **or** `DOOM.WAD` from
  *The Ultimate Doom*. SIGIL is a DOOM 1 episode, not DOOM II.
- **Accepted IWAD variants and documented hashes (Confirmed, Doom Wiki/Fandom `DOOM.WAD`):**

  | Variant | Filename | Size | MD5 | SHA-1 | CRC32 |
  | --- | --- | ---: | --- | --- | --- |
  | Registered Doom 1.9 | `doom.wad` | 11,159,840 | `1cd63c5ddff1bf8ce844237f580e9cf3` | `7742089b4468a736cadb659a7deca3320fe6dcbd` | `723e60f9` |
  | The Ultimate Doom 1.9ud | `doomu.wad` | 12,408,292 | `c4fe9fd920207691a9f493668e0a2083` | `9b07b02ab3c275a6a7570c3f73cc20d63a0e3833` | `bf0eaac0` |

  Older registered builds (1.666, 1.8) are also 11,159,840 bytes but different MD5/SHA-1 — **do not
  accept them** (the WAD is documented as "May Not Run With… Versions of Doom prior to 1.9",
  **Confirmed**). **No SHA-256 of any IWAD is published** by these sources (see open questions).
- **Port family / complevel (Confirmed + Proposed):** the text says *"Advanced engine needed:
  Limit-removing"*. The maps are vanilla-format but exceed vanilla engine limits, so they need a
  limit-removing port (DSDA-Doom qualifies). **Proposed: run at complevel 3 (Ultimate Doom).**
  DSDA-Doom treats "Ultimate Doom" as UD compatibility and this is the complevel that permits
  episode 5: its source comment reads *"Now you can play it with `-warp 5 1 -complevel 3`"* and it
  clamps episodes > 3 below that level (`prboom2/src/g_game.c`, v0.30.1, lines ~3010–3041). The
  precise default still **Needs Deck Validation**.
- **Episode recognition (Confirmed):** DSDA-Doom v0.30.1 implements UMAPINFO including the `episode`
  key — `prboom2/src/umapinfo.cpp` calls `dsda_AddEpisode(...)` and `dsda_ClearEpisodes()`; the
  episode logic lives in `prboom2/src/dsda/episode.c`. Therefore the **main WAD's UMAPINFO adds the
  SIGIL episode** and no compat WAD is needed in DSDA-Doom. `SIGIL_COMPAT_V1_23.wad` exists only for
  strict/older ports that lack E5 support (**Confirmed** from the bundled text).
- **May Not Run With:** pre-1.9 Doom (**Confirmed**).

## 4. Composition (candidate recipe data — **Proposed**)

Components and load positions. The schema does not exist yet, so this is illustrative TOML shaped
around D-006/D-010, not a final schema.

```toml
# ILLUSTRATIVE — not a final schema, not a catalog entry
id = "sigil"
release = "1.23"
requirements = { iwad = "doom", iwad_variants = ["doom-1.9-registered", "doom-1.9ud-ultimate"],
                 compatibility = { family = "vanilla-limit-removing", complevel = 3 } }
packages = [
  { id = "sigil-free", url = "https://romero.com/s/SIGIL_V1_23-8fh4.zip",
    sha256 = "57c35a6bcd722433fc97344314e52c06c91fdfde33b18c6d1c14a4d29e229105" },
]
components = [
  { id = "main",      file = "SIGIL_V1_23.wad",        load_position = 1, default = true },
  { id = "compat",    file = "SIGIL_COMPAT_V1_23.wad", load_position = 1, default = false,
    exclusive_with = "main" },                        # E3 replacement alternative
  { id = "soundtrack", file = "SIGIL_V1_23_REG.wad",   load_position = 1, optional = true,
    source = "user-provided", exclusive_with = "main" }, # paid; DDL must not download
  { id = "soundtrack-compat", file = "SIGIL_SHREDS_COMPAT.wad", load_position = 2,
    optional = true, source = "user-provided", requires = "compat" },
]
```

- **(a) Standard play:** `SIGIL_V1_23.wad` (position 1).
- **(b) With soundtrack:** the paid `SIGIL_V1_23_REG.wad` is a **replacement** for the free WAD
  (it embeds the MP3 music), not an add-on loaded after it. Older paid packages also shipped
  `SIGIL_SHREDS.wad` as a soundtrack WAD loaded after the main WAD (referenced in the bundled co-op
  instructions). **Needs validation** which file names the current paid package actually ships.
- **(c) Compat variant:** `SIGIL_COMPAT_V1_23.wad` (position 1) instead of the main WAD; if paid,
  add `SIGIL_SHREDS_COMPAT.wad` after it.

## 5. Licence and redistribution

**Confirmed** from `SIGIL_V1_23.txt`:

- *"You MAY distribute this file, provided you include this text file, with no modifications… as long
  as you include this file intact."*
- *"Authors may NOT use the contents of this file as a base for modification or reuse without
  explicit permission…"*

**Confirmed** from the romero.com page: the megawad is a **free download** ("Free SIGIL Megawad");
DOOM/id Software names are trademarks of Microsoft/ZeniMax and the mod is "in no way affiliated".
The page does **not** publish an explicit redistribution clause beyond the bundled text.

- **May DDL download from romero.com? (Proposed: yes.)** The files are free and redistribution is
  explicitly permitted when the text file travels intact. DDL should still **download from
  romero.com** (or the `/idgames` mirror) rather than rehost, credit John Romero, and keep
  `SIGIL_V1_23.txt` in the install so the permission notice is preserved. DDL does not re-publish
  content, so the "include this file intact" clause is satisfied by not stripping it.
- **Paid soundtracks (Buckethead / THORR) are gated** behind a `sowl.co` payment link on the page.
  **Do not attempt to fetch them.** They must be a user-supplied optional component.
- **DOOM + DOOM II re-release (Confirmed, Bethesda release notes 2024-08-08):** the enhanced
  *DOOM + DOOM II* "What's Included" list contains **SIGIL**. This is the KEX remaster's own campaign
  data — **it is not a loadable PWAD in the user's filesystem**, so DDL cannot assume a user who owns
  that product has a `SIGIL_V1_23.wad` to load. Treat it as context, not as a source of files.

## 6. Implications for DDL (Proposed)

- **IWAD gate:** require DOOM 1 (`doom.wad`/`doomu.wad`), refuse `doom1.wad` (shareware) and pre-1.9
  builds; check the documented MD5/SHA-1 sizes where available and let the user confirm ambiguity
  (OVERVIEW §6).
- **Episode selection:** the launch recipe must run at **complevel 3** so episode 5 is valid. DDL can
  either let the player choose the SIGIL episode (menu shows SIGIL) or pass a validated `-warp 5 1`;
  both need confirmation on the Deck against the pinned DSDA release.
- **Optional soundtrack toggle:** expose a per-profile toggle, but mark it **user-supplied** (paid
  file); never download it. Because the paid REG WAD *replaces* the main WAD, the UI must make the
  resulting load order explicit (D-010) and must not load both.
- **Compat variant choice:** surface `SIGIL_COMPAT_V1_23.wad` only as an advanced alternative
  (or auto-select it for ports that need E3). It is mutually exclusive with the main WAD; the UI
  should hide it by default since DDL only targets DSDA-Doom.
- **Hash binding per source:** allow one recipe to list more than one package (romero.com and
  `/idgames`) each with its own SHA-256, since the ZIPs differ.

---

# SIGIL II

## 1. Official sources

### Download pages

- Same canonical page: <https://romero.com/sigil> (the page is titled "SIGIL" but advertises
  *"Free SIGIL II Megawad"*; there is no `romero.com/sigil2` page — **Confirmed**, 2026-10-16).
- `https://si6il.com` redirects to the same page (**Confirmed**).

### The current free release — v1.0

- Page text: *"The free SIGIL II v1.0 Megawad is now available. This small 2MB file is packed with
  some hardcore classic DOOM punishment…"* (**Confirmed**).
- User-facing URL: <https://romero.com/s/SIGIL_II_V1_0.zip> (**Confirmed**).
- Resolves (302 → 200) to a Squarespace CDN token URL:

  ```text
  https://static1.squarespace.com/static/5fef3318f8ce9b562500206e/t/65751ebf19a4803f0153f416/1702174402423/SIGIL_II_V1_0.zip
  ```

### Version history

- Only **v1.0**, released **2023-12-10** (bundled `SIGIL_II_V1_0.txt`); the page presents no older
  archive and no newer version (**Confirmed**). The free and paid versions are both labelled v1.0;
  the paid one adds the THORR soundtrack only.

### idgames mirror

- idgames hosts it at `…/idgames/levels/doom/Ports/s-u/sigil_ii_v1_0.zip`
  (2,012,056 bytes) (**Confirmed** via the mirror).
- Not byte-identical to the romero.com ZIP
  (`3bb9a9e0d170c7ede6a6e6cca018d03ff675b532e9cab8d1eddd5bae2bbc555f`); the **WAD is identical**,
  the `SIGIL_II_V1_0.txt` differs (author e-mail replaced with `[redacted]`).

### Archive format (what DDL's extractor must handle)

**Confirmed** from `zipfile` inspection of `SIGIL_II_V1_0.zip`:

- Standard ZIP, all three entries **Deflate** (`compress_type = 8`), no encryption.
- **Flat layout** — no folders, no symlinks, no traversal.
- **ASCII-only names**, case preserved. **Note the uppercase `.WAD` extension on the main file** —
  DDL must match it exactly on case-sensitive Linux.
- Timestamps: WAD 2023-12-06, texts 2023-12-09.

### Contents (full internal paths)

| Path in archive | Size (bytes) | SHA-256 | Role |
| --- | ---: | --- | --- |
| `SIGIL_II_V1_0.WAD` | 5,632,323 | `9cadd239ea960ad40eeb655757941313cc8cb4f5f1c07a701ac986be4710a359` | main episode (E6M1–E6M9), MIDI |
| `SIGIL_II_V1_0.txt` | 3,586 | `7c6dab2977c157311d945fb2afa7f8a27399d9a5040e768a82a506a333733f4a` | README/copyright (romero.com copy) |
| `SIGIL_II_README_V1_0.txt` | 9,442 | `0db9b8f0ffdad6a36e48d9dbad1be2c72885f1164a75b52665327c5396700706` | install/play instructions |

## 2. Variants

| WAD | Where | Purpose |
| --- | --- | --- |
| `SIGIL_II_V1_0.WAD` | free (both mirrors) | main: adds **episode 6** (E6M1–E6M9), free **MIDI** soundtrack by James Paddock |
| THORR-soundtrack build | paid only | same episode with the **THORR MP3** soundtrack |

There is **no compat WAD and no separate music WAD** in the free release (**Confirmed**). The
bundled text says SIGIL II "was not made for strict vanilla-compatible DOOM ports, but it may work
with some of them" — no E6 replacement variant is provided.

### Lump-directory inspection (read-only)

`SIGIL_II_V1_0.WAD` — PWAD, 171 lumps:

- Maps **E6M1…E6M9**, complete vanilla map groups (THINGS … BLOCKMAP). No BEHAVIOR/ZNODES.
- `DEHACKED` (244 bytes): `Patch File for DeHackEd v3.0`, `Doom version = 21`, `Patch format = 6`;
  sets `Thing 20 (SpiderMastermind) Hit points = 9000` and `[PARS]` (E6M1–E6M9 par times).
- `UMAPINFO` (1,851 bytes): `map E6M1 … E6M9` and on E6M1
  `Episode = "M_EPI6", "SIGIL II", "S"`; `E6M8` has `BossAction = clear` and ending intertext;
  each map carries `partime`.
- Also `MAPINFO` (ZDaemon), `ZMAPINFO` (ZDoom), `EMAPINFO` (Eternity), `LANGUAGE`, `OPTIONS`
  (`comp_dropoff 1`), `F_START`, `TEXTURE1/2`, `PNAMES`.
- Music lumps `D_E6M1…D_E6M9`, `D_INTRO`, `D_INTER` — **all MIDI** (`MThd`). No MUS/OGG/MP3.
- `DEMO1…DEMO4` present.
- **No `COMPLVL` lump.**
- Map names (from UMAPINFO): E6M1 Cursed Darkness, E6M2 Violent Hatred, E6M3 Twilight Desolation,
  E6M4 Fragments of Sanity, E6M5 Wrathful Reckoning, E6M6 Vengeance Unleashed, E6M7 Descent Into
  Terror, E6M8 Abyss of Despair, E6M9 Shattered Homecoming.

## 3. Requirements

- **IWAD (Confirmed, bundled text):** registered `DOOM.WAD` or the *Ultimate Doom* `DOOM.WAD`
  (`doom.wad` / `doomu.wad`, same hashes as §SIGIL-3).
- **Port family / complevel (Confirmed + Proposed):** text says *"Advanced engine needed:
  Limit-removing. Developed using GZDoom"* and *"Tested on DSDA-Doom, Crispy Doom, Eternity,
  ZDaemon"*. **Proposed: complevel 3 (Ultimate Doom)**, matching the E6 content and SIGIL.
  Complevel is **Needs Deck Validation**.
- **Episode recognition (Confirmed):** main WAD carries `UMAPINFO` with `Episode = "M_EPI6"`; DSDA-Doom
  v0.30.1 supports UMAPINFO episodes (source §SIGIL-3), so no variant is needed. The bundled text
  warns that *"Some engines do not support episode 6"* — DSDA-Doom is not one of them.
- **Note:** SIGIL II ships **no** `SIGIL_II_COMPAT`-type WAD, so there is no official E-replacement
  fallback if a port rejects E6.

## 4. Composition (candidate recipe data — **Proposed**)

```toml
# ILLUSTRATIVE — not a final schema, not a catalog entry
id = "sigil-ii"
release = "1.0"
requirements = { iwad = "doom", iwad_variants = ["doom-1.9-registered", "doom-1.9ud-ultimate"],
                 compatibility = { family = "vanilla-limit-removing", complevel = 3 } }
packages = [
  { id = "sigil2-free", url = "https://romero.com/s/SIGIL_II_V1_0.zip",
    sha256 = "41741ce797e6faf9ebd0e3d577b5eaf9a648b64d850fbd741c1b9c28d8461017" },
]
components = [
  { id = "main",       file = "SIGIL_II_V1_0.WAD", load_position = 1, default = true },
  { id = "soundtrack", file = "SIGIL_II_THORR.wad", load_position = 1, optional = true,
    source = "user-provided", exclusive_with = "main" }, # paid; filename unconfirmed
]
```

- **(a) Standard play:** `SIGIL_II_V1_0.WAD` (position 1, case-sensitive).
- **(b) With soundtrack:** the purchased THORR build (Windows `.WAD` vs `.wad` case unconfirmed — see
  open questions). Presumed to **replace** the free WAD, mirroring SIGIL.
- **(c) Compat variant:** none exists.

## 5. Licence and redistribution

**Confirmed** from `SIGIL_II_V1_0.txt` — the same clauses as SIGIL:

- *"You MAY distribute this file, provided you include this text file, with no modifications…
  as long as you include this file intact."*
- *"Authors may NOT use the contents of this file as a base for modification or reuse without
  explicit permission…"*

The romero.com page describes it as a free download (**Confirmed**).

- **May DDL download from romero.com? (Proposed: yes.)** Same reasoning as SIGIL: free, redistribution
  permitted with the text intact, download from source and preserve `SIGIL_II_V1_0.txt`.
- **THORR soundtrack is gated** (paid via `sowl.co`); do not fetch.
- **DOOM + DOOM II re-release (Confirmed, Bethesda Update 3 release notes, 2025-04-10):**
  *"SIGIL II is now officially included as part of the DOOM + DOOM II package."* Again, this is the
  remaster's own campaign data, **not** a loadable PWAD for DSDA-Doom.

## 6. Implications for DDL (Proposed)

- Same IWAD gate and complevel-3 requirement as SIGIL.
- **Episode selection:** episode 6 is only reachable via the WAD's UMAPINFO (or a validated
  `-warp 6 1`); verify the menu label "SIGIL II" and the episode behaviour on the Deck.
- **Optional soundtrack toggle:** user-supplied only; the THORR WAD filename and whether it replaces
  or augments the free WAD must be confirmed from a purchased copy before the recipe ships.
- **No compat variant** to expose — the UI should simply not offer one for this title.
- Because SIGIL and SIGIL II share an IWAD and complevel, DDL can reuse the same IWAD check and a
  shared "vanilla limit-removing / cl3" compatibility profile.

---

# Open questions

1. **Paid soundtrack file names.** SIGIL: the v1.23 README names `SIGIL_V1_23_REG.wad` and
   `SIGIL_SHREDS_COMPAT.wad`, but the site's paid bundle is described as "v1.21" and older co-op
   instructions mention a plain `SIGIL_SHREDS.wad`. Which files does the **current** paid package
   actually ship, and does the REG WAD replace or augment the free WAD? SIGIL II: the THORR
   package's filename/casing and load position are unknown. (Gated content — to be supplied by the
   owner or a purchaser; do not bypass.)
2. **Redistribution of the paid soundtracks.** The bundled "MAY distribute" clause is written from the
   free megawad's perspective; it is unclear whether it covers the paid MP3 WADs. DDL treats them as
   user-supplied regardless.
3. **SHA-256 of accepted IWADs.** Only MD5/SHA-1/CRC32 are published for `doom.wad`/`doomu.wad`.
   DDL's IWAD identification should follow DSDA-Doom's content-based detection, or the owner should
   supply SHA-256s of the IWADs DDL will accept.
4. **Exact complevel.** cl3 is proposed from DSDA-Doom source semantics, but the *correct tested*
   value (and whether the episode menu needs cl3) is **Needs Deck Validation**.
5. **Registered vs Ultimate.** Whether the SIGIL/SIGIL II episode menu appears identically under the
   registered `doom.wad` (only 3 IWAD episodes) versus `doomu.wad` must be tested; the SIGIL UMAPINFO
   path is reported to behave differently in some GZDoom builds.
6. **Source URL stability.** The `romero.com/s/…` short URLs are stable, but they redirect to
   Squarespace token URLs that change on re-upload. Confirm DDL's downloader follows redirects and
   re-verifies SHA-256, and decide whether to allow the `/idgames` mirror as a fallback (different
   archive hash).
7. **Which mirror to pin.** The romero.com and idgames ZIPs differ (text-file e-mail redaction). Pick
   romero.com as primary; decide whether `/idgames` is an accepted alternate package.
8. **DOOM + DOOM II IWAD.** Whether the IWAD shipped with the KEX remaster can be used by DSDA-Doom,
   and whether its `doom.wad` matches the documented 1.9ud hashes, is unknown. DDL must not assume
   the bundled SIGIL/SIGIL II are available as files.
9. **Compat variant exposure.** Whether DDL should ever surface `SIGIL_COMPAT_V1_23.wad` (DSDA-Doom
   does not need it) as an advanced option, or hide it entirely.

# IWAD identification research

**Status:** desk research · **Gathered:** 2026-10-16 (UTC)
**Workstream:** Content Schema / Library (D-010 base games; OVERVIEW §3 step 3 and §6)
**Scope:** facts needed so DDL can identify user-supplied IWADs by content, never by filename
alone. No code and no catalog entry is produced here.

Labels used throughout:

- **Confirmed** — verified from an upstream source, published metadata, or a reputable
  documentation page (evidence linked inline).
- **Proposed** — a DDL interpretation/recommendation that still needs review.
- **Needs Deck Validation** — must be exercised on the physical Steam Deck before it is trusted.

## Method

All facts come from published metadata and upstream source; **no IWAD was downloaded or
distributed**. Network use was read-only and bounded (no SHA-256 was computed from a commercial
IWAD). DSDA-Doom source was read from a shallow clone of tag `v0.30.1` in the git-ignored
`.scratch/dsda-doom` (HEAD `8ed581ad5c868b4dba1c39eb21d29e4ddbed9fad`, agreeing with
[`docs/research/dsda-engine.md`](dsda-engine.md)). `doomwiki.org` returned HTTP 403 to this
research client, so those pages were read through the
[Wayback Machine](https://web.archive.org/) on 2026-10-16; the links below are the canonical live
URLs. The Fandom mirror (`doom.fandom.com`) was reachable directly and agrees with DoomWiki.

---

## 1. Known IWAD versions

Hashes are reproduced exactly as published. "SHA-256?" marks whether a SHA-256 is published by a
source I could reach; **the DoomWiki and Fandom tables publish MD5, SHA-1 and CRC-32 only** (no
SHA-256), so most rows below are MD5/SHA-1 only. See the summary at the end of this section.

Sizes are in bytes. "Entries" is the lump-directory count where published.

### 1.1 DOOM (episode games)

**Shareware — `DOOM1.WAD` 1.9** ([Fandom](https://doom.fandom.com/wiki/DOOM1.WAD),
[Akbar30Bill](https://github.com/Akbar30Bill/DOOM_wads/blob/master/README.md))

| Size | MD5 | SHA-1 | CRC-32 | SHA-256? |
| ---: | --- | --- | --- | --- |
| 4,196,020 | `f0cefca49926d00903cf57551d901abe` | `5b2e249b9c5133ec987b3ea77596381dc0d6bc1d` | `162b696a` | no |

Older shareware builds (MD5 only, Fandom): 1.0 `90facab21eede7981be10790e3f82da2`, 1.2
`30aa5beb9e5ebfbbe1e1765561c08f38`, 1.666 `762fd6d4b960d4b759730f01387a50a1`, 1.8
`5f4eb849b1af12887dec04a2a12e5e62`.

**Registered — `DOOM.WAD`** ([DoomWiki](https://doomwiki.org/wiki/DOOM.WAD),
[Fandom](https://doom.fandom.com/wiki/DOOM.WAD))

| Version | Entries | Size | MD5 | SHA-1 | CRC-32 | SHA-256? |
| --- | ---: | ---: | --- | --- | --- | --- |
| 1.9 | 2,194 | 11,159,840 | `1cd63c5ddff1bf8ce844237f580e9cf3` | `7742089b4468a736cadb659a7deca3320fe6dcbd` | `723e60f9` | no |
| 1.8 | 2,194 | 11,159,840 | `11e1cd216801ea2657723abc86ecb01f` | `2c8212631b37f21ad06d18b5638c733a75e179ff` | `8d242df9` | no |
| 1.666 | 2,194 | 11,159,840 | `54978d12de87f162b9bcc011676cb3c0` | `2e89b86859acd9fc1e552f587b710751efcffa8e` | `f756aab5` | no |
| 1.1 | 2,074 | 10,396,254 | `981b03e6d1dc033301aa3095acc437ce` | `df0040ccb29cc1622e74ceb3b7793a2304cca2c8` | `66457ab9` | no |
| 1.2 | 2,045 | 10,399,316 | `792fd1fea023d61210857089a7c1e351` | `b5f86a559642a2b3bdfb8a75e91c8da97f057fe6` | `a5da8930` | no |

Note: 1.9 registered and 1.9ud (below) are **different sizes**; older registered builds share the
1.9 size but differ by hash. Only 1.9 and 1.9ud run DOOM-1 mods that require v1.9
(OVERVIEW §7; [`reference-sigil.md`](reference-sigil.md) §SIGIL-3).

**Ultimate Doom — `DOOMU.WAD` 1.9ud** ([DoomWiki](https://doomwiki.org/wiki/DOOM.WAD),
[Lutris installer](https://lutris.net/games/install/35796/view))

| Size | Entries | MD5 | SHA-1 | CRC-32 | SHA-256 |
| ---: | ---: | --- | --- | --- | --- |
| 12,408,292 | 2,306 | `c4fe9fd920207691a9f493668e0a2083` | `9b07b02ab3c275a6a7570c3f73cc20d63a0e3833` | `bf0eaac0` | `6fdf361847b46228cfebd9f3af09cd844282ac75f3edbb61ca4cb27103ce2e7f` |

Both the registered and the Ultimate Doom 1.9 WADs are commonly named `doom.wad`; Ultimate Doom is
sometimes renamed `doomu.wad` (DSDA-Doom accepts `doomu.wad`, DoomWiki "which?" note). Identification
must therefore be by content, not name.

**DOOM (1993) in the 2024 DOOM + DOOM II re-release (KEX)** ([DoomWiki](https://doomwiki.org/wiki/DOOM.WAD),
[included-files list](https://doomwiki.org/wiki/Doom_%2B_Doom_II))

| File date | Entries | Size | MD5 | SHA-1 | CRC-32 | SHA-256? |
| --- | ---: | ---: | --- | --- | --- | --- |
| 2024-08-06 | 2,308 | 12,733,492 | `4461d4511386518e784c647e3128e7bc` | `997bae5e5a190c5bb3b1fb9e7e3e75b2da88cb27` | `cff03d9f` | no |
| 2024-10-01 | 2,308 | 12,996,515 | `3b37188f6337f15718b617c16e6e7a9c` | `87651324502044f9a6eed403e48853aa16c93e49` | `d5f8c089` | no |

The KEX `doom.wad` is Ultimate Doom content (episodes 1–4) with the re-release's map fixes and
widescreen assets; it has 2,308 entries versus 2,306 for 1.9ud. The re-release ships an
**unmodified** 1.9/1.9ud copy under `base/` and this updated copy under `rerelease/` (§4).

Related published variants (for completeness, not DDL targets): BFG Edition `doom.wad` 12,487,824
(`fb35c4a5a9fd49ec29ab6e900572c524` / `e5ec79505530e151ff0e6f517f3ce1fd65969c46` / `5efa677e`) and
Doom I Enhanced (2019 Unity) 12,702,921 (`8517c4e8f0eef90b82852667d345eb86` /
`2a8a1ce0f29497a2781b2902c76115fd60d8bbf8` / `75c3b7bf`) — both from
[DoomWiki DOOM.WAD](https://doomwiki.org/wiki/DOOM.WAD).

### 1.2 DOOM II (`DOOM2.WAD`)

([DoomWiki](https://doomwiki.org/wiki/DOOM2.WAD), [Fandom](https://doom.fandom.com/wiki/DOOM2.WAD),
[Lutris installer](https://lutris.net/games/install/35797/view))

| Version | Entries | Size | MD5 | SHA-1 | CRC-32 | SHA-256 |
| --- | ---: | ---: | --- | --- | --- | --- |
| 1.9 | 2,919 | 14,604,584 | `25e1459ca71d321525f84628f45ca8cd` | `7ec7652fcfce8ddc6e801839291f0e28ef1d5ae7` | `ec8725db` | `10d67824b11025ddd9198e8cfc87ca335ee6e2d3e63af4180fa9b8a471893255` |
| 1.8 | 2,919 | 14,612,688 | `c236745bb01d89bbb866c8fed81b6f8c` | `79c283b18e61b9a989cfd3e0f19a42ea98fda551` | `31bd3bc0` | no |
| 1.7a | 2,919 | 14,612,688 | `d7a07e5d3f4625074312bc299d7ed33f` | `70192b8d5aba65c7e633a7c7bcfe7e3e90640c97` | `952f6baa` | no |
| 1.7 | 2,919 | 14,612,688 | `ea74a47a791fdef2e9f2ea8b8a9da13b` | `78009057420b792eacff482021db6fe13b370dcc` | `47daeb2e` | no |
| 1.666 (1994-08-29) | 2,956 | 14,943,400 | `30e3c2d0350b67bfbf47271970b74b2f` | `6d559b7ceece4f5ad457415049711992370d520a` | `e2a683bd` | no |
| 1.666g | 2,934 | 14,824,716 | `d9153ced9fd5b898b36cc5844e35b520` | `a4ce5128d57cb129fdd1441c12b58245be55c8ce` | `c08005f7` | no |
| 1.8f (French release, `doom2f.wad`) | 2,914 | 14,607,420 | `3cb02349b3df649c86290907eed64e7b` | `d510c877031bbd5f3d198581a2c8651e09b9861f` | `27eaae69` | no |

There was also a 1994-08-25 "1.666" build DoomWiki marks as a pirated copy (same size/MD5 as the
1994-08-29 1.666). DDL should reject pre-1.9 builds for v1.9-only content (as in SIGIL's "May Not
Run With" note).

**DOOM II in the 2024 DOOM + DOOM II re-release (KEX)** ([DoomWiki](https://doomwiki.org/wiki/DOOM2.WAD))

| File date | Entries | Size | MD5 | SHA-1 | CRC-32 | SHA-256? |
| --- | ---: | ---: | --- | --- | --- | --- |
| 2024-08-06 | 2,928 | 14,802,506 | `9aa3cbf65b961d0bdac98ec403b832e1` | `c745f04a6abc2e6d2a2d52382f45500dd2a260be` | `09b8a6ae` | no |
| 2024-10-01 | 2,928 | 14,951,361 | `64a4c88a871da67492aaa2020a068cd8` | `2921cf667359fd3a80aba3c0cf62ab39297e7e9e` | `151b8a96` | no |

Split as for DOOM: `base/` holds the unmodified 1.9 `doom2.wad`, `rerelease/` the KEX copy with the
map fix (MAP20 sky) and widescreen assets. Related published variants: BFG Edition `doom2.wad`
14,691,821 (`c3bea40570c23e511a7ed3ebcd9865f7` / `a59548125f59f6aa1a41c22f615557d3dd2e85a9` /
`927a778a`) and Doom II Enhanced (2019 Unity) 14,802,843 (`8ab6d0527a29efdc1ef200e5687b5cae` /
`9574851209c9dfbede56db0dee0660ecd51e6150` / `f1d1ad55`).

### 1.3 Final Doom — TNT: Evilution (`TNT.WAD`)

([DoomWiki](https://doomwiki.org/wiki/TNT.WAD), [Fandom](https://doom.fandom.com/wiki/TNT.WAD))

| Variant | Entries | Size | MD5 | SHA-1 | CRC-32 | SHA-256? |
| --- | ---: | ---: | --- | --- | --- | --- |
| 1.9 (Final Doom Steam package) | 3,101 | 18,195,736 | `4e158d9953c79ccf97bd0663244cc6b6` | `9fbc66aedef7fe3bae0986cdb9323d2b8db4c9d3` | `903dcc27` | no |
| id Anthology (later revision) / Mac / GOG, still labelled 1.9 | 3,106 | 18,654,796 | `1d39e405bf6ee3df69a8d2646c8d5c49` | `4a65c8b960225505187c36040b41a40b152f8f3e` | `d4bb05c0` | no |
| DOOM + DOOM II (KEX), 2024-08-08 | 3,109 | 18,304,630 | `8974e3117ed4a1839c752d5e11ab1b7b` | `9820e2a3035f0cdd87f69a7d57c59a7a267c9409` | `15f18ddb` | no |
| DOOM + DOOM II (KEX), 2024-10-03 | 3,109 | 18,507,237 | `ad7885c17a6b9b79b09d7a7634dd7e2c` | `ab4e71e360a1e64e262d77a53e4bb202450d0f6e` | `6d7a8eec` | no |

The Anthology/GOG variant fixes the MAP31 yellow-keycard bug, removes some secret specials on MAP10
and rebuilds its nodes, adds a `_DEUTEX_` lump and `P1_START`/`P1_END` · `F1_START`/`F1_END`
markers. It carries **no revision number change**, so both are "v1.9" and can only be told apart by
hash (or by those extra lumps). Demos recorded on one desync on the other
([DoomWiki](https://doomwiki.org/wiki/TNT.WAD)).

### 1.4 Final Doom — The Plutonia Experiment (`PLUTONIA.WAD`)

([DoomWiki](https://doomwiki.org/wiki/PLUTONIA.WAD),
[Fandom](https://doom.fandom.com/wiki/PLUTONIA.WAD))

| Variant | Entries | Size | MD5 | SHA-1 | CRC-32 | SHA-256? |
| --- | ---: | ---: | --- | --- | --- | --- |
| 1.9 (Final Doom Steam package) | 2,984 | 17,420,824 | `75c8cf89566741fa9d22447604053bd7` | `90361e2a538d2388506657252ae41aceeb1ba360` | `48d1453c` | no |
| id Anthology (later revision) / Mac / GOG, still labelled 1.9 | 2,988 | 18,240,172 | `3493be7e1e2588bc9c8b31eab2587a04` | `f131cbe1946d7fddb3caec4aa258c83399c21e60` | `15cd1448` | no |
| DOOM + DOOM II (KEX), 2024-08-08 | 2,991 | 17,531,493 | `24037397056e919961005e08611623f4` | `816c7c6b0098f66c299c9253f62bd908456efb63` | `650b998d` | no |
| DOOM + DOOM II (KEX), 2024-10-03 | 2,991 | 17,704,206 | `e47cf6d82a0ccedf8c1c16a284bb5937` | `fbcc140825b507ab88d844c6b1635c60c4b8bcd6` | `e82fbfd2` | no |

The Anthology/GOG variant adds the missing deathmatch starts on MAP12 and MAP23; again no revision
change, so hashes (or content) are the only discriminator ([DoomWiki](https://doomwiki.org/wiki/PLUTONIA.WAD)).

### 1.5 Extras shipped with DOOM + DOOM II (NERVE and others)

**`NERVE.WAD` (No Rest for the Living)** ([DoomWiki](https://doomwiki.org/wiki/NERVE.WAD))

| Variant | Entries | Size | MD5 | SHA-1 | CRC-32 | SHA-256? |
| --- | ---: | ---: | --- | --- | --- | --- |
| BFG Edition / XBLA / Xbox 360 / PSN (also bundled with Doom + Doom II) | 108 | 3,819,855 | `967d5ae23daf45196212ae1b605da3b0` | `3451288383fb16e196f273d9f85d58c1fda97bf4` | `ad7f9292` | no |
| DOOM + DOOM II `rerelease/` version | 117 | 4,039,441 | `23422eb42833ac7b0dd59c0c7ae18a6f` | `4522eaec7a13aac24b456e29520e99ac879e6989` | `07d9faab` | no |

`nerve.wad` is a **PWAD** (`MAP01`–`MAP09`), not an IWAD; v0.30.1 ships a `nerve.wad`-by-name
special case (§2). Its later Doom Classic Unity add-on `nerve_i.wad` was reworked into an actual
IWAD, so DDL must not assume a `nerve.wad` name is unambiguous ([DoomWiki](https://doomwiki.org/wiki/NERVE.WAD)).

**Other files in the re-release** (inventory only; no published hashes captured).
[DoomWiki "Doom + Doom II" included-files list](https://doomwiki.org/wiki/Doom_%2B_Doom_II) and a
[Steam discussion](https://steamcommunity.com/app/2280/discussions/0/6741412385247917763/) name:
`extras.wad`, `id1.wad` (Legacy of Rust PWAD), `id1-mus.wad`, `id1-res.wad`, `id1-tex.wad`,
`id1-weap.wad`, `id24res.wad`, `iddm1.wad`, `masterlevels.wad`, `sigil.wad`, `sigil2.wad`,
`Common.kpf`, plus the four IWADs above.

### 1.6 SHA-256 availability

**Only two of the IWAD variants listed in §1.1–§1.5 have a published SHA-256** that this research
could reach: `DOOM2.WAD` 1.9 and `DOOMU.WAD` 1.9ud, both from Lutris community installer scripts. Every other row
is **MD5/SHA-1/CRC-32 only**. DoomWiki, Fandom and the Akbar30Bill table publish no SHA-256 for any
classic IWAD. See "Open questions" — DDL's TOML table should be able to match on the hashes that are
published and capture/verify SHA-256 locally when the owner supplies it (D-014 pattern).

---

## 2. Lump-based identification that does not depend on the hash list

**Confirmed — how DSDA-Doom v0.30.1 decides the game.** `CheckIWAD()` in
[`prboom2/src/d_main.c`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c)
reads the WAD directory and counts lumps, then sets the game **mode** purely from content:

| Counter | Source condition | Meaning |
| --- | --- | --- |
| `sw` | lump name `E1Mx` | one shareware episode map |
| `rg` | lump name `E2Mx` or `E3Mx` | registered episodes |
| `ud` | lump name `E4Mx` | Ultimate Doom's fourth episode |
| `cm` | lump name `MAPxx` | a Doom II-style map |
| `sc` | lump name `MAP31` or `MAP32` | the two secret slots |
| `hx` | lump name starting `HACX` | Hacx marker |
| — | `DMENUPIC` present and `TITLEPIC` size ≤ 68168 | Unity/BFG-edition marker |
| — | lump name starting `FREEDM` | FreeDM marker |

Resolution ([`d_main.c` L1058–1075](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1058-L1075)):

```
if (cm >= 30 || (cm >= 20 && hx))  gmode = commercial; hassec = (sc >= 2);
else if (ud >= 9)                  gmode = retail;      // Ultimate Doom
else if (rg >= 18)                 gmode = registered;  // Doom 1.9
else if (sw >= 9)                  gmode = shareware;   // DOOM1.WAD
```

So the **marker lumps that actually matter** are the presence/count of `E1M1…E1M9`, `E2M1…`, `E3M1…`,
`E4M1…` and `MAP01…MAP32`, plus `DMENUPIC`/`TITLEPIC` size and `FREEDM*`/`HACX*`. A single `E4M1`
lump confirms Ultimate Doom versus registered Doom; a lone `E1M1…E1M9` set is shareware; any
`MAP01…MAP32` set means the Doom II family.

**Confirmed — mission is then chosen from the filename, not the content.** `AddIWAD()` in the same
file ([L1094–1164](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L1094-L1164))
sets `gamemission` by suffix: `doom2f.wad`→French Doom II, `tnt.wad`→`pack_tnt`,
`plutonia.wad`→`pack_plut`, `chex3d2.wad`/`chex3v.wad`→Chex 3, `hacx.wad`→Hacx,
`freedoom1.wad`/`freedoom2.wad`/`freedm.wad`→Freedoom, `chex.wad`→Chex, `rekkrsa.wad`→REKKR.
`D_AddFile()` additionally forces `gamemission = pack_nerve` for any file ending `nerve.wad`
([L914–940](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L914-L940)), and
`standard_iwads[]` ([L168–197](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L168-L197))
lists the accepted names (`doom.wad`, `doom1.wad`, `doomu.wad`, `doom2.wad`, `doom2f.wad`,
`tnt.wad`, `plutonia.wad`, `chex.wad`, `chex3v.wad`, `chex3d2.wad`, `hacx.wad`, `rekkrsa.wad`,
`bfgdoom.wad`, `bfgdoom2.wad`, `freedoom1/2.wad`, `freedm.wad`, plus Raven names).

**The consequence (Confirmed, this research).** DSDA-Doom is *content-aware for mode* and
*filename-driven for mission*. Two facts follow for DDL:

1. `DOOM.WAD` and `DOOM2.WAD` are told apart by content; **DOOM II 1.9, TNT, Plutonia, FreeDM and
   Freedoom Phase 2 are not** — they all satisfy `cm >= 30` and are separated only by name. There is
   no single "TNT lump" or "Plutonia lump" in v0.30.1. Reliable discrimination needs hashes
   (or the extra Anthology lumps noted in §1.3).
2. If a user renames `tnt.wad` to, say, `finaldoom1.wad`, DSDA-Doom will classify it as ordinary
   Doom II (and `vanilla` complevel changes meaning — §3). DDL must therefore either preserve the
   canonical basename or place files at a DDL-managed path with the canonical name before launch.

**Header magic caveat (Confirmed).** Two DoomWiki pages note that the BFG Edition `DOOM.WAD`/
`DOOM2.WAD` and the European PSN `TNT.WAD`/`PLUTONIA.WAD` are *incorrectly tagged as `PWAD`*.
DSDA-Doom only warns on a missing `IWAD` tag and continues
([`d_main.c` L989–992](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c#L989-L992)).
So a content identifier must not reject a file solely because its 4-byte magic is not `IWAD`.

**On the lump names in the task brief (`CAVERN5`, `MC11`, `FREEDOOM`).** A full-text search of the
v0.30.1 tree found no `CAVERN5` or `MC11` reference anywhere; `CAVERN` appears only inside the Chex
DeHackEd string table (`E1M5: Caverns of Bazoik`). `FREEDOOM` appears only as filenames. I could not
confirm these as IWAD identification markers in DSDA-Doom; treat them as unverified (open questions).

---

## 3. Complevel / "vanilla" resolution that depends on the IWAD

Cross-checked against [`dsda-engine.md`](dsda-engine.md) §1 (Confirmed there; re-read in source here).

**Confirmed.** In
[`prboom2/src/dsda/settings.c` L59–75](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L59-L75),
the word `vanilla` is resolved **from the game mission DSDA-Doom assigned the IWAD** (which, per §2,
comes from the filename):

```
vanilla → 4  if gamemode == commercial and gamemission is TNT or Plutonia
        → 2  if gamemode == commercial otherwise (e.g. Doom II)
        → 3  otherwise (Doom 1 / Ultimate Doom)
```

Numeric meanings (Confirmed enum,
[`doomtype.h` L113–140](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomtype.h#L113-L140)):
`0` Doom 1.2, `1` Doom 1.666, `2` Doom/Doom II 1.9, `3` Ultimate Doom/Doom95, `4` Final Doom,
`7–9` Boom, `10–11` MBF, `21` MBF21. Omitting `-complevel` reads a winning `COMPLVL` lump, otherwise
falls back to the configured default (upstream `21`), not automatic vanilla
([`settings.c` L76–170](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L76-L170)).

**Implications (from this IWAD dependency):**

- **Confirmed:** the *number* a given content family needs is IWAD-specific — Doom 1 vanilla is 3,
  Doom II vanilla is 2, Final Doom (TNT/Plutonia) vanilla is 4. A recipe that targets "vanilla" must
  know which IWAD the user actually has.
- **Confirmed:** resolving the *word* `vanilla` depends on DSDA-Doom's filename-based mission guess,
  so a renamed IWAD silently shifts the complevel. **Proposed:** DDL always passes an explicit
  numeric `-complevel` (consistent with D-011) and derives it from the identified IWAD variant, never
  from the `vanilla` keyword.
- **Proposed:** the known-IWAD table should carry a suggested default complevel per family
  (`doom`→3, `doom2`→2, `tnt`/`plutonia`→4) for recipe defaults; the recipe may still override.

---

## 4. Where IWADs live on a Steam Deck

**Confirmed — Steam's model** ([ArchWiki: Steam](https://wiki.archlinux.org/title/Steam)): the
default Steam root is `~/.local/share/Steam`, with `~/.steam/root` as a symlink to it. Every game
lives under `LIBRARY/steamapps/common/<install-dir>/`, where `<install-dir>` comes from the
`installdir` property of `<LIBRARY>/steamapps/appmanifest_<appid>.acf` (KeyValues format). Users may
add further libraries (Steam → Settings → Storage). The library list itself is kept in a
`libraryfolders.vdf` KeyValues file under `steamapps/`; its exact current schema is **Needs Deck
Validation** (see open questions).

**Confirmed — legacy classic-Doom Steam folders** (read from Chocolate Doom's IWAD search code,
[`src/d_iwad.c`](https://github.com/chocolate-doom/chocolate-doom/blob/master/src/d_iwad.c)). On
Linux, Chocolate Doom looks under `~/.steam/root/steamapps/common/`:

```
Doom 2/base                    Final Doom/base
Doom 2/finaldoombase           DOOM 3 BFG Edition/base/wads
Ultimate Doom/base             Heretic Shadow of the Serpent Riders/base
Master Levels of Doom/doom2    Hexen/base, Hexen Deathkings.../base
```

The Windows registry list in the same file adds `steamapps\common\final doom\base`. These are the
DOSBox-era releases; folder spelling/casing is Chocolate Doom's, not guaranteed to match the current
shipping client (**Needs Deck Validation**).

**Confirmed — DOOM + DOOM II (2024, KEX; Steam app 2280)** ships the WADs in two subfolders of its
install directory:

- `base/` — the **classic** DOS-era files (Ultimate Doom `doom.wad`, `doom2.wad` 1.9, `tnt.wad`,
  `plutonia.wad`) plus other original WADs.
- `rerelease/` — the **KEX** copies (`doom.wad`, `doom2.wad`, `tnt.wad`, `plutonia.wad`,
  `nerve.wad`, `masterlevels.wad`, `sigil.wad`, `sigil2.wad`, `id1*.wad`, `iddm1.wad`, `extras.wad`,
  `id24res.wad`, `Common.kpf`). These add widescreen assets and the map fixes hashed in §1; some
  assets are censored relative to the originals.

  Sources: [WadFusion maintainer discussion](https://github.com/Owlet7/wadfusion/discussions/51)
  (states the `base/` vs `rerelease/` split explicitly) and the
  [Steam "wads in rerelease folder" thread](https://steamcommunity.com/app/2280/discussions/0/6741412385247917763/).
  **Proposed:** DDL should prefer `base/` IWADs for vanilla Doom/Doom II content (uncensored, matches
  published 1.9 hashes) and treat `rerelease/` copies as the KEX variants below. The literal
  install-dir name (`DOOM + DOOM II`) and the on-Deck presence of both folders are
  **Needs Deck Validation**.

**Proposed GOG and other locations** (all **Needs Deck Validation**):

- GOG `DOOM + DOOM II` (GOG slug `doom_doom_ii`, per the Lutris `gogslug` field) typically installs
  to `~/GOG Games/DOOM + DOOM II/` with a `base/`/`rerelease/` split like Steam; GOG's Final Doom
  copy uses the **Anthology/GOG revisions** of `tnt.wad`/`plutonia.wad` (DoomWiki §1.3/§1.4).
- Common non-store conventions, also searched by Chocolate Doom and DSDA-Doom: `/usr/share/games/doom`,
  `~/.local/share/games/doom`, and the `DOOMWADDIR` / `DOOMWADPATH` environment variables
  ([`d_iwad.c`](https://github.com/chocolate-doom/chocolate-doom/blob/master/src/d_iwad.c);
  [`dsda-engine.md`](dsda-engine.md) §1 search paths). DDL should mirror these as *candidate roots*
  only when the user opts in, never by default.

**Needs Deck Validation** for all of the above: exact folder casing, whether the Steam client still
creates the legacy separate apps' folders, the SD-card library mount path
(commonly `/run/media/mmcblk0p1/…`, but the label can differ), where GOG installs on the Deck, and
Flatpak Steam's path. DDL must read Steam files only, never write them.

---

## 5. Implications for DDL (Proposed)

### 5.1 Identification algorithm

**Proposed** (satisfies OVERVIEW §6 "identify by content, detect ambiguity, let the user choose"):

1. **Cheap pre-filter.** Read only the 12-byte WAD header and the directory table (bounded size).
   Do **not** reject on magic alone: accept `IWAD` and, for a small allow-list of known BFG/PSN
   hashes, `PWAD` too (§2). Reject unreadable/oversized/zip-bomb-sized inputs.
2. **Size + hash.** Compute SHA-256 always (and MD5/SHA-1 when the matched entry only has those) and
   look the file up in the known-IWAD table. Match on the strongest published hash for each entry.
3. **Lump heuristics (hash-list fallback).** If no hash matches, classify the family with DSDA's own
   logic (§2): count `E1Mx`/`E2Mx`/`E3Mx`/`E4Mx`/`MAPxx`, check `DMENUPIC` + `TITLEPIC` size,
   `FREEDM*`/`HACX*`. This reliably yields shareware / registered / Ultimate Doom / Doom II-family
   and flags FreeDM/Hacx/Unity.
4. **Ambiguity → ask.** Within the Doom II family there is no single-lump discriminator between
   Doom II / TNT / Plutonia / Freedoom Phase 2 (§2), so if only heuristics fired, present the
   candidates and let the user choose (OVERVIEW §6). Never auto-launch on a guess.
5. **Canonical naming.** Because DSDA-Doom resolves mission and `vanilla` complevel from the
   filename (§2, §3), after identification DDL should expose/launch the IWAD under its canonical
   basename (`doom.wad`, `doomu.wad`, `doom2.wad`, `tnt.wad`, `plutonia.wad`) — e.g. via a
   DDL-managed symlink or copy — and always pass an explicit `-complevel`.

### 5.2 Known-IWAD table format (TOML, D-006)

**Proposed.** Hand-authored, reviewed data → TOML, e.g. `data/iwads.toml` (or a section of the
catalog). One entry per *variant*, with the game identity separate from the file variant, so a
recipe can require a game and accept several variants:

```toml
# ILLUSTRATIVE — not final schema, not shippable data
schema_version = 1

[[iwad]]
id = "doom2-1.9"
game = "doom2"                 # abstract game id a recipe requires
title = "Doom II: Hell on Earth v1.9"
filename = "doom2.wad"         # canonical basename for DSDA-Doom mission detection
size = 14604584
sha256 = "10d67824b11025ddd9198e8cfc87ca335ee6e2d3e63af4180fa9b8a471893255"
md5 = "25e1459ca71d321525f84628f45ca8cd"
sha1 = "7ec7652fcfce8ddc6e801839291f0e28ef1d5ae7"
crc32 = "ec8725db"
default_complevel = 2
source = "https://doomwiki.org/wiki/DOOM2.WAD"

[[iwad]]
id = "plutonia-1.9-steam"
game = "plutonia"
filename = "plutonia.wad"
size = 17420824
md5 = "75c8cf89566741fa9d22447604053bd7"   # only MD5/SHA-1/CRC-32 published
sha1 = "90361e2a538d2388506657252ae41aceeb1ba360"
crc32 = "48d1453c"
default_complevel = 4
source = "https://doomwiki.org/wiki/PLUTONIA.WAD"

[[iwad]]
id = "doom-1.9ud-kex"
game = "doom"
title = "Ultimate Doom (DOOM + DOOM II rerelease, 2024-10-01)"
filename = "doom.wad"
size = 12996515
md5 = "3b37188f6337f15718b617c16e6e7a9c"
sha1 = "87651324502044f9a6eed403e48853aa16c93e49"
crc32 = "d5f8c089"
default_complevel = 3
source = "https://doomwiki.org/wiki/DOOM.WAD"
```

Rules implied by §1/§2: fields are optional where no source publishes them; every hash field must
cite its source; `game` is the stable key recipes use; `default_complevel` follows §3. SHA-256 is
preferred and DDL computes it locally even when the table has none (then matches on the strongest
published hash).

### 5.3 How recipes reference accepted IWADs (D-010)

**Proposed.** A recipe keeps its abstract requirement and an optional accepted-variant allow-list
that references `id`s from the table (extending the shape already sketched in
[`reference-sigil.md`](reference-sigil.md) §4):

```toml
requirements = {
  iwad = "doom",
  iwad_variants = ["doom-1.9", "doom-1.9ud", "doom-1.9ud-kex"],  # optional; empty = any variant of `game`
  compatibility = { family = "vanilla-limit-removing", complevel = 3 },
}
```

The resolver maps the user's detected file to a table `id` (or "unknown"), checks membership, and
**blocks Play** while no accepted IWAD is present, naming what is needed (D-010). A recipe that
accepts "any `doom`" must still know the detected variant's complevel (D-011 passes an explicit
numeric complevel).

### 5.4 Safe auto-discovery

**Proposed** (OVERVIEW §6; AGENTS.md hard rules):

- Discover Steam roots **read-only** from `libraryfolders.vdf`, then probe only
  `<library>/steamapps/common/<known-install-dirs>` (`DOOM + DOOM II`, `Ultimate Doom`, `Doom 2`,
  `Final Doom`, `DOOM 3 BFG Edition`, …) — never a full `$HOME` walk.
- Otherwise scan only directories the user picks with the controller-first picker, using the same
  bounded identifier (header + size + hashes).
- Never bundle or download IWADs; never execute anything from an IWAD directory; persist the
  selected IWAD *reference* and re-verify size/hash before each launch (OVERVIEW §6).

---

## Open questions

1. **SHA-256 gap.** Only 2 IWAD versions have a published SHA-256 (§1.6). Should DDL's reviewed table
   store owner-computed SHA-256 for each accepted IWAD (with the published MD5/SHA-1 kept for
   matching), or accept MD5/SHA-1 matching until SHA-256 is supplied?
2. **`CAVERN5`, `MC11`, `FREEDOOM` markers.** Not found as DSDA-Doom markers. Is there a reputable
   content marker for TNT vs Plutonia vs Doom II (e.g. a texture/patch or lump-count fingerprint)
   we can rely on instead of hashes? Freedoom's `FREEDOOM`-named lump (if any) is unverified.
3. **KEX rerelease inventory hashes.** DoomWiki hashes the four KEX IWADs and `nerve.wad`, but not
   `extras.wad`, `id1*.wad`, `id24res.wad`, `iddm1.wad`, `masterlevels.wad`, `sigil*.wad`. Do we need
   them, and can the owner supply them from a purchased copy?
4. **Which KEX copy to use.** Is it safe to point DSDA-Doom (via DDL) at the `rerelease/` KEX IWADs,
   or should DDL always prefer the `base/` originals? Needs a Deck test (censorship, demo compat,
   map fixes, node differences).
5. **`libraryfolders.vdf` schema.** Confirm the exact current file location(s) and KeyValues layout
   on SteamOS (and Flatpak Steam) and how to enumerate SD-card libraries robustly. Needs Deck
   Validation.
6. **Header-magic acceptance.** Confirm the specific BFG/PSN variant hashes whose files are actually
   tagged `PWAD`, so the pre-filter can allow-list exactly those and still reject junk.
7. **Canonical-naming mechanics.** Should DDL symlink, copy, or hard-link the identified IWAD under
   its canonical basename? Effects on DSDA-Doom's filename-based organized saves/data should be
   tested (see [`dsda-engine.md`](dsda-engine.md) §6).
8. **Freedoom/Chex as base games.** Are these in scope as DDL "base games" (they are freely
   redistributable, unlike commercial IWADs)? OVERVIEW §6 lists only DOOM/Ultimate/DOOM II/Final Doom.
9. **v1.666 / 1.8 registered builds.** DDL's SIGIL research says "do not accept them"; should the
   known-IWAD table list them as *recognised-but-unsupported* (so the UI can explain the mismatch)
   rather than ignore them?

## Sources

**DoomWiki / DoomWiki.org** (read via the Wayback Machine on 2026-10-16 because the live site
returned HTTP 403 to this client):

- [DOOM.WAD](https://doomwiki.org/wiki/DOOM.WAD) · [DOOM2.WAD](https://doomwiki.org/wiki/DOOM2.WAD) ·
  [TNT.WAD](https://doomwiki.org/wiki/TNT.WAD) · [PLUTONIA.WAD](https://doomwiki.org/wiki/PLUTONIA.WAD) ·
  [NERVE.WAD](https://doomwiki.org/wiki/NERVE.WAD) · [Doom + Doom II](https://doomwiki.org/wiki/Doom_%2B_Doom_II)

**Fandom Doom Wiki** (reached directly):

- [DOOM.WAD](https://doom.fandom.com/wiki/DOOM.WAD) · [DOOM1.WAD](https://doom.fandom.com/wiki/DOOM1.WAD) ·
  [DOOM2.WAD](https://doom.fandom.com/wiki/DOOM2.WAD) · [TNT.WAD](https://doom.fandom.com/wiki/TNT.WAD) ·
  [PLUTONIA.WAD](https://doom.fandom.com/wiki/PLUTONIA.WAD)

**Hash/metadata tables:**

- [Akbar30Bill/DOOM_wads README](https://github.com/Akbar30Bill/DOOM_wads/blob/master/README.md)
  (sizes + MD5)
- Lutris community installer scripts: [The Ultimate Doom v1.9ud](https://lutris.net/games/install/35796/view),
  [Doom II v1.9](https://lutris.net/games/install/35797/view) (SHA-256)

**Source code (pinned):**

- DSDA-Doom v0.30.1:
  [`prboom2/src/d_main.c`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/d_main.c) ·
  [`prboom2/src/dsda/settings.c`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c) ·
  [`prboom2/src/doomtype.h`](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/doomtype.h)
- [Chocolate Doom `src/d_iwad.c`](https://github.com/chocolate-doom/chocolate-doom/blob/master/src/d_iwad.c)

**Platform / community:**

- [ArchWiki: Steam](https://wiki.archlinux.org/title/Steam) (library folders, `steamapps/common`,
  appmanifest layout)
- [Steam discussion "wads in rerelease folder" (app 2280)](https://steamcommunity.com/app/2280/discussions/0/6741412385247917763/)
- [WadFusion discussion #51](https://github.com/Owlet7/wadfusion/discussions/51) (`base/` vs
  `rerelease/`)

**In-repo:** [`OVERVIEW.md`](../../OVERVIEW.md) §3, §6; [`docs/decisions.md`](../decisions.md)
D-006, D-010, D-011, D-014; [`docs/research/dsda-engine.md`](dsda-engine.md);
[`docs/research/reference-sigil.md`](reference-sigil.md).

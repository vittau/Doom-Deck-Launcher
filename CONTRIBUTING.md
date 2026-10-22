# Contributing to DoomDeckLauncher

## Add a mapset

Add one `catalog/<id>.toml` file in a pull request. Use a stable lowercase slug
for `id` and the filename. No Rust changes are needed for ordinary catalog additions.
Optional images need explicit permission, a safe relative `path`, and a nonempty
`permission` in `[[images]]`; attach only images you may redistribute.

1. Read [spec 002](docs/specs/002-recipe-catalog.md),
   [the decisions](docs/decisions.md), and the relevant research. Download from
   the official source or a reviewed /idgames mirror, inspect without executing,
   and record exact byte counts and SHA-256 for each package and named member.
2. Copy the complete, lint-clean SIGIL example below to your new file. Replace
   its identity, sources, members and evidence with values you have actually
   checked. Never retain another release's hashes or invent an unknown value.
3. Keep permission and credit files unmodified: list them as `keep = true`.
   Package members use case-exact paths. Nested ZIPs use `outer.zip!/inner.wad`.
   Nothing loads implicitly, and embedded DEHACKED lumps are never separate members.
4. Declare a required, default-on `main`, then optional components. Every
   component has a positive explicit `position`; members within it retain their
   declared order. Ordinarily, add-ons have positions after main. Components in
   the same `exclusive_group` cannot coexist. A `replaces = "main"` choice
   suppresses main; only one replacement may be enabled. Default selections must
   obey these rules too. Load order validation proves every allowed combination.
5. Record honest evidence: `indexed`, `inspected`, `verified`, `partial` or
   `incompatible`. Include tester, date, engine version, notes and limitations.
   `inspected` means archive/source/lump inspection, **not** a gameplay test.
   Mark the engine range `inspected` too; do not promote a recipe just because
   fetch lint succeeds. The complevel has no status of its own: pick it by
   [common sense](#choosing-the-complevel).
6. Run these from the repository root (Rust comes from `rust-toolchain.toml`):

   ```sh
   cargo run -p ddl -- catalog lint catalog/your-mapset.toml
   cargo run -p ddl -- catalog lint --fetch catalog/your-mapset.toml
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

Lint is offline by default and exits unsuccessfully with `file:line`, an error
and a fix hint. With no paths it checks `catalog/`; multiple files/directories
are supported. `--fetch` tries the primary direct URL, then mirrors on failure;
it verifies downloaded size/hash and **all named members**, including kept text
and optional members. It follows at most five redirects, validating HTTPS and
an exact download host at each hop. It never extracts to disk or executes content.
Resource limits are 1 MiB per TOML document, 512 MiB per package/member and total
decompressed bytes per package check, 10,000 ZIP entries, and four path layers.
A download attempt times out after 5 minutes. Tests are offline with fake fetches.

Host review is a separate change to `catalog/hosts.toml`. `pages` permits canonical
attribution pages; `downloads` permits direct files **and redirect targets**.
Doomworld is pages-only. /idgames URLs are direct mirror paths, never API calls.
Mirrors for one package must serve byte-identical archives: Romero and /idgames
SIGIL ZIPs differ and cannot share one package hash. Never download IWADs or
paid soundtracks. Known imported hashes may be absent when genuinely unknown.

### Schema 1 fields

Unknown keys and unknown enum values are errors at every level. Required fields
are shown in the complete example. Optional fields and constraints:

| Table | Optional fields / accepted values |
| --- | --- |
| Root | `images` (`[[images]]`: `path`, `permission`); `screenshots` (`[[screenshots]]`: `asset`, `credit`; see [Screenshots](#screenshots)); `tags` (list of ids from `catalog/tags.toml`, absent = none; see [Tags](#tags)); `original_release` (first public release, optional; see [Original release date](#original-release-date)) |
| `base_game` | `game`: `doom`, `doom2`, `tnt`, `plutonia`; `variants` defaults empty (any variant of that game). IDs refer to the known-IWAD table; currently lowercase letters, digits, dots and hyphens. The runtime must block Play without a matching IWAD; that gate is outside catalog lint. |
| `packages` | `kind`: `zip` or bare `wad`; optional canonical `idgames_path`; `urls`: primary followed by mirrors. Bare WADs name one member with identical package/member size/hash. |
| `packages.members` | `keep` defaults false; unique `id` and `path` within a package. All members have exact positive `size` and `sha256`. |
| `components` | Optional `exclusive_group`, `replaces = "main"`, `user_supplied = true`, `obtain`, `known_sha256`. IDs are unique within the recipe. |
| `components.members` | For downloaded content, `package` and `member` refer to declared IDs. Roles: `maps`, `music`, `graphics`, `sounds` (WAD) or `dehacked` (DEH/BEX). Kept members cannot be loaded. Each user-supplied component has exactly one imported member with a role, no package/member reference, nonempty `obtain` instructions, and optional known hashes. No guessed filename or download URL. |
| `compatibility` | `family`: `vanilla`, `limit-removing`, `boom`, `mbf`, `mbf21`; mandatory `complevel`: `"vanilla"`, `"boom"`, `"mbf"`, `"mbf21"` or a number 0–17 or 21 that no alias names for the recipe's `base_game` (see [Choosing the complevel](#choosing-the-complevel)); 18–20 forbidden. There is no `complevel_status`: lint rejects the key. |
| `compatibility.engine` | Semantic version requirement `range`, evidence `status`. Include the exact evidence version; a range is only as strong as its evidence status. |
| `compatibility.runtime` | Optional typed `warp = [episode, map]` (or `[map]`, or `[]`; up to two integers 0–99) and `pistol_start = true/false` (player profile suggestion only). No arbitrary flags or shell strings. |
| `verification` | Real `YYYY-MM-DD` date, semantic engine version. `inspected` lists what was inspected; nonempty when status is inspected. `limitations` can be empty only when there are no known limitations. |

### Choosing the complevel

Write the DSDA-Doom alias whenever one names the level (D-030). DSDA-Doom v0.30.1 accepts
`vanilla`, `boom` (9), `mbf` (11) and `mbf21` (21) case-insensitively, and `vanilla` resolves
by IWAD to 2 on Doom II, 3 on Doom/Ultimate Doom and 4 on TNT/Plutonia
([research](docs/research/dsda-engine.md)). DDL passes the alias word to `-complevel` and the
player sees it with its number, for example "Vanilla (2)". A number stays only for a level no
alias names on that recipe's `base_game`, such as 2 on Ultimate Doom; `catalog lint` rejects any
other number and suggests the alias.

Pick the level in this order, and stop at the first source that states one:

1. What the author states: the release text file, a `COMPLVL` lump or `UMAPINFO`.
2. Otherwise the port family's community convention: vanilla or limit-removing →
   `vanilla`, Boom → `boom`, MBF → `mbf`, MBF21 → `mbf21`.

Say which source you used in the recipe's `verification.notes`. The level carries no
verification status; gameplay evidence stays in `[verification]`.

### Tags

`catalog/tags.toml` is the reviewed vocabulary: per tag an `id`, an English `label`, a
short `description` and an `order` (the position of its Discover category). A recipe lists
the tags it earns in an optional root key, before the first table:

```toml
tags = ["episode", "cacowards"]
```

Lint rejects unknown and repeated tags. Every tag is a claim and needs evidence in
`[verification]` (`notes` or `inspected`) — never tag from memory:

| Tag | Evidence to record |
| --- | --- |
| `featured` | Why the maintainers feature it (for example a required reference recipe, D-010). |
| `beginner-friendly` | A quoted statement from the authors or a reviewed source about gentle difficulty. |
| `classics` | A release date at least five years old plus a cited sign of lasting recognition. |
| `cacowards` | **Required by lint:** link that year's page, `https://www.doomworld.com/cacowards/<year>/`, and say winner or runner-up. Say if you could not read the page and relied on a secondary source. |
| `community-projects` | The credited team (for example a Doomer Boards Project's contributors list). |
| `megawad` | At least 30 maps, counted from the WAD or the authors' map list. |
| `episode` | Nine maps or fewer, counted the same way. |
| `challenging` | A quoted statement from the authors or a reviewed source aimed at experienced players. |
| `gimmick` | A quoted author/project statement or reviewed source naming the unusual explicit design constraint or playful central concept the set is organised around. |

Discover shows a category for each tag that at least one recipe uses, in `order`, between
"All" and "Favorites". To propose a new tag, add its `[[tags]]` entry to `catalog/tags.toml` in
the same pull request that first uses it; give it a unique lowercase `id`, a unique `order`,
an English `label` and a one-sentence `description` of the rule a recipe must meet.

### Original release date

Chronological browsing reads one optional root field (spec 090):

```toml
original_release = { year = 1998, month = 5, day = 12 }
```

Record the **first public release**, not the version/update named by `release` and not the
`/idgames` upload date when they differ. Keep only the precision the evidence supports: a day,
a month (`year` + `month`) or a year alone (`year`). Never invent January 1 or a first of the
month, and leave the field out when the first release is unknown. Lint rejects an impossible
calendar value and a day without a month. Lists sort ascending with unknown dates last and ties
by title then recipe id; a coarser date sorts after the dated entries of the same period, so a
year-only date follows that year's months and a month-only date follows that month's days.
Explain the source in a comment or in `verification.inspected`.

### Screenshots

Every mapset card, detail page and art band shows a **bundled screenshot** of the mapset
(D-024); the painted concept art is only the fallback for a recipe that has none. Screenshots are
the mapset authors' material: DDL commits them, credits them and removes one on the author's
request. Add them in the same pull request as the recipe.

1. Commit a representative **in-game** view (a recognisable map, not a title screen, logo or box
   art) under `crates/ddl/ui/assets/screenshots/<id>/`, named `01.jpg`, `02.jpg`, …. The rule
   (spec 035; every bundled image is embedded in the `ddl` binary): **JPEG only, 640×360 up to
   1280×800, ≤ 192 KiB**, saved at quality 60 (lower if the file is still over the cap). Prefer
   a 16:9 crop; keep the original colours, add no border or text. One to three per recipe. Never
   commit the full-size original. One command produces a compliant file (macOS `sips`, or
   ImageMagick on Linux):

   ```sh
   scripts/lean-image.sh original.png crates/ddl/ui/assets/screenshots/<id>/01.jpg
   ```
2. Record where each file came from in `crates/ddl/ui/assets/screenshots/<id>/SOURCES.toml`
   (schema 1), one `[[screenshot]]` entry per file: `file`, `source_url`, `source_page`,
   `credit` (who made the mapset, or who took the shot), `original_size` (`WxH` before the
   resize), `retrieved` (`YYYY-MM-DD`) and `permission` (any licence or permission text you
   found, or that none was found). Never invent a URL, size or permission.
3. Reference each file from the recipe's `[[screenshots]]` (put it before the first `[table]`):

   ```toml
   [[screenshots]]
   asset = "<id>/01.jpg"
   credit = "Author or team"
   ```

   `asset` is the path under `crates/ddl/ui/assets/screenshots/`; `credit` is the short caption
   the UI shows next to the image. Omit the field for a mapset you could not source — the painted
   fallback art stays.
4. Run the lint (see step 6 above). It rejects a missing image, a duplicate or traversing
   `asset`, a missing `credit`, a file over 192 KiB, an image smaller than 640×360 or larger than
   1280×800, a file that is not a JPEG (PNG and WebP are rejected), and a file with no
   `SOURCES.toml` entry carrying a credit and source URL.

Prefer the mapset's own site or release page, the project repository, an author page, or a
community screenshot posted in the official release thread. Doomworld and DoomWiki block
automated fetches; when a page is reachable only through the Wayback Machine, record the page you
read and the exact image URL you downloaded.

The same rule covers the UI art in `crates/ddl/ui/assets/`: raster art with no real transparency
is a JPEG of at most 1280×800 (`scripts/lean-image.sh`). The exceptions are art that uses its alpha
channel, and the 9-slice `frame.png` and the four `ornament-*.png` sprites in `ember/` (each under
8 KB). Those stay lossless PNG: JPEG would save about 12 KB in total and would blur their 1-pixel
bevels. Art with alpha is also capped at 1280×800 and recompressed losslessly.

### Copy-paste template and annotated example

This is the actual [SIGIL recipe](catalog/sigil.toml), backed by
[reference research](docs/research/reference-sigil.md). Its hashes are **Confirmed**;
gameplay **Needs Deck Validation**. Start with
this structure, replace the researched data, then delete components your release
does not provide. The compat and soundtrack components replace main at the same
position; they do not add a second map WAD. The menu selects the episode, so no
variant-dependent `warp` is prescribed here.

```toml
# Confirmed bytes and release metadata: docs/research/reference-sigil.md §1–§5; gathered 2026-10-16.
schema_version = 1
id = "sigil"
title = "SIGIL"
authors = ["John Romero"]
release = "1.23"
page = "https://romero.com/sigil"
description = "Episode 5 for DOOM, with a MIDI soundtrack and an optional Episode 3 compatibility replacement."
permission = "Redistribution permitted with the bundled permission text intact and unmodified. See kept credits/readme members for full terms and resource credits."

[base_game]
game = "doom"
variants = ["doom-1.9ud"]

[compatibility]
family = "limit-removing"
complevel = "vanilla"

[compatibility.engine]
range = "=0.30.1"
status = "inspected"

[compatibility.runtime]
# Optional typed defaults: warp = [episode, map], pistol_start = false.

[verification]
status = "inspected"
tester = "DDL desk research"
date = "2026-10-16"
engine_version = "0.30.1"
inspected = ["docs/research/reference-sigil.md §1–§5", "Archive sizes and SHA-256; WAD lump listings; DSDA-Doom v0.30.1 source"]
notes = "Confirmed archive/member identities and source inspection. Complevel vanilla by the limit-removing convention; no gameplay test performed."
limitations = ["Needs Deck Validation: IWAD/complevel, load order, gameplay, audio and episode menu.", "Paid soundtrack identity and replacement behavior need purchaser validation; no filename/hash is prescribed."]

[[packages]]
id = "free"
page = "https://romero.com/sigil"
urls = ["https://romero.com/s/SIGIL_V1_23-8fh4.zip"]
size = 3270225
sha256 = "57c35a6bcd722433fc97344314e52c06c91fdfde33b18c6d1c14a4d29e229105"
kind = "zip"

[[packages.members]]
id = "maps"
path = "SIGIL_V1_23.wad"
size = 4640555
sha256 = "29c3b8400657be4546eab1878e2883ffc50771c6be575677b1be5a0d0430f55d"
keep = false

[[packages.members]]
id = "compat"
path = "SIGIL_COMPAT_V1_23.wad"
size = 4634503
sha256 = "a79416b5e1a64b2946c09dcc5704f5d1448728b1afa2d761ccc067cd9ecce2b5"
keep = false

[[packages.members]]
id = "credits"
path = "SIGIL_V1_23.txt"
size = 5001
sha256 = "d1445fb883296f69dde7dbd300032bb0c8804f6a6a4b944f8fbb5231bcb79b9e"
keep = true

[[packages.members]]
id = "readme"
path = "SIGIL_README_V1_23.txt"
size = 10580
sha256 = "9d3877d88eb49a0475300f465b1c2771c847ac0e83550d31c74de24018fbef6c"
keep = true

[[components]]
id = "main"
label = "SIGIL"
description = "Main episode/mapset with bundled MIDI music."
optional = false
default = true
position = 1

[[components.members]]
package = "free"
member = "maps"
role = "maps"

[[components]]
id = "compat"
label = "Episode 3 compatibility WAD"
description = "Replaces E3 instead of adding E5; DSDA-Doom supports the main UMAPINFO episode."
optional = true
default = false
position = 1
replaces = "main"
exclusive_group = "edition"

[[components.members]]
package = "free"
member = "compat"
role = "maps"

[[components]]
id = "soundtrack"
label = "Buckethead soundtrack"
description = "Proposed paid soundtrack replacement; user supplied and unvalidated."
optional = true
default = false
position = 1
replaces = "main"
exclusive_group = "edition"
user_supplied = true
obtain = "Purchase the Buckethead soundtrack version through romero.com/sigil and import your purchased WAD. Filename/hash are unconfirmed; validate the replacement before play."
known_sha256 = []

[[components.members]]
role = "maps"
```

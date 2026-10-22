# Report 091 — Gimmick vocabulary and Linear Doom assessment (worker B)

**Date:** 2026-10-22. **Branch:** `vittau/pwads-087-b`. **Spec:** 091 (commit `d18842b`).
**Outcome:** the `gimmick` vocabulary entry and its standalone offline tests are added;
**Linear Doom is deliberately not added** because its author archive has no reproducible,
policy-compliant download and its bytes (hence its DSDA-Doom compatibility) cannot be
inspected. A precise primary-evidence blocker report is in
`docs/research/linear-doom.md`.

## What changed

| File | Change |
| --- | --- |
| `catalog/tags.toml` | Added `gimmick` (label `Gimmick`, order 75, description "Mapsets organized around an unusual explicit design constraint or playful central concept") between Challenging (70) and Featured (80); updated the header comment's sequence to 10–75 |
| `crates/ddl-core/tests/catalog_091.rs` | New standalone offline tests: reviewed label/order/description, unique orders, vocabulary validation, Discover ordering (after Challenging, before Featured/Beginner Friendly), embedded-vocabulary acceptance, a real recipe carrying `gimmick` passing full validation, unknown/duplicate still rejected, and a forward-compatible assertion that The 10x10 Project carries the tag once Worker A's recipe is integrated |
| `docs/research/linear-doom.md` | Primary-evidence identity/date/size/MD5 table, retrieval attempts and statuses, policy analysis, compatibility limits and unblock options |
| `docs/reports/091-gimmick-and-linear-doom.md` | This report |
| `docs/research/reference-087-b.md` | Chronology correction: Deathless original full release **2018-11-09** (author post and DoomWiki), selected v1.1 update 2020-05-25 (see §5) |

Not changed: any of the six settled recipes, `catalog/hosts.toml`, `catalog/linear-doom.toml`
(not created), shared `crates/ddl-core/src/catalog/tests.rs`, runtime/UI, CONTRIBUTING,
dependencies or coordinator docs. No screenshot directory was created.

## Linear Doom — verdict and blocker (summary)

The author is DeXiaZ; the ModDB v1.2 file page records `Linear_Doom_v1.2.7z`, 274,927 bytes,
MD5 `4f758b008649e7424ea634c615f34de7`, added 2019-04-06, described as a jokewad that
"makes all classic Doom 1 levels straightened into one line", with notes referencing E1M4,
E1M8, E2M4 and E3M6. The original file page records `Linear_Doom.7z`, 275,143 bytes, MD5
`488637b2a879ea933b62f819a1fba47f`, "Added Mar 31st, 2019" (the mod header rounds it to
"Released Apr 1, 2019"), so both the original and v1.2 are 7z archives.

Three independent blockers; resolving one does not remove the others:

1. **Host/access — no downloadable bytes observed.** The tested live ModDB URLs (mod page,
   downloads page, v1.2 file page, start endpoint, member downloads) all answered HTTP 403
   with a Cloudflare challenge on 2026-10-22. The start endpoint is an HTML gate; no direct
   file URL appeared on the tested pages, and no bypass was attempted. The form or lifetime
   of any eventual download target is **not determined**, so no exact host or durable URL can
   be recorded. Adding `www.moddb.com` to the allowlist would not by itself make the file
   fetchable by DDL's client; `catalog/hosts.toml` is unchanged.
2. **Archive format — 7z is not supported.** The published file is
   `Linear_Doom_v1.2.7z`. The recipe schema accepts `kind = "zip"` or a bare WAD only, and
   DDL's reader has no 7z support. Even with a perfect URL, the package could not be
   declared without a new archive dependency and downloader/engine change, which is out of
   scope. This is separate from host access and from engine compatibility.
3. **Engine compatibility — not established.** Without the archive, the map count, IWAD
   variant and lump inventory are unknown. DSDA-Doom 0.30.1 runs Doom-format/Boom/MBF/MBF21
   data plus UMAPINFO and ignores GZDoom-only `ZMAPINFO`/`MAPINFO`/`DECORATE`/ACS content;
   the author's "Tested with GZDoom 2.4.0" note is neither a requirement nor proof of DSDA
   support, and no GZDoom dependency is inferred. DSDA has no `linear-doom` entry (HTTP 404)
   and no `/idgames` path was found, but other sources were not searched — this is not a
   claim that none exists.

The coordinator's independent read of the same file page agrees with the identity recorded
here: DeXiaZ, 2019-04-06, `Linear_Doom_v1.2.7z`, 274,927 bytes, MD5
`4f758b008649e7424ea634c615f34de7`.

Published MD5 and byte size are recorded as identity evidence, but the schema needs SHA-256
for the package and every named member plus a fetchable URL, so the recipe cannot be pinned.

Unblock options, which are separate needs: a **local/supplied copy** of the v1.2 archive
would allow byte inspection, hashing and a compatibility verdict, but would not by itself
provide the automatic catalog download and would not solve the 7z format limitation; an
**author republish as ZIP or bare WAD on an already-reviewed host** would solve the download
and format blockers together; an **owner-approved policy for ModDB's gate** is a separate
decision outside this task. "All sources ruled out" is not claimed.

## Dates for spec 090

- Linear Doom original public release: **2019-03-31** (day precision); selected update v1.2
  **2019-04-06**. Research only — no recipe, so no chronology metadata is exposed.

## Deathless chronology correction (final)

The earlier research row used the selected v1.1 upload date (2020-05-25) as the original
release, and a first correction wrongly said 2019-11-09. Primary evidence fixes the year as
2018: the author's release post is dated **"November 9, 2018"**
(`https://www.doomworld.com/forum/topic/103075-deathless-an-ultimate-doom-megawad-rc3/`,
Wayback copy read 2026-10-22), and DoomWiki's infobox reads `Year 2018` with "the initial
version of the full megawad … shared on November 9th" during the first NaNoWADMo. The row now
says **2018-11-09** (day precision) and keeps 2020-05-25 as the selected v1.1 update; the
July 2018 E4-only predecessor *Griefless* is a different release. The recipe's version label
and hashes are unchanged.

## Verification

```sh
# New standalone vocabulary/lint tests
CARGO_TARGET_DIR=/Users/vitor/git/Doom-Deck-Launcher/target CARGO_BUILD_JOBS=2 \
  cargo test -p ddl-core --test catalog_091 -- --test-threads=2
# -> 5 passed; 0 failed.

# The whole catalog still lints against the new vocabulary
/Users/vitor/git/Doom-Deck-Launcher/target/debug/ddl catalog lint catalog/
# -> Catalog lint passed (offline).

cargo fmt --all --check
# -> clean.

CARGO_TARGET_DIR=/Users/vitor/git/Doom-Deck-Launcher/target CARGO_BUILD_JOBS=2 \
  cargo clippy -p ddl-core --all-targets -- -D warnings
# -> clean.
```

No fetch lint was run because no recipe was added. The two failing shared unit tests in
`crates/ddl-core/src/catalog/tests.rs` (`shipped_vocabulary_is_valid_and_has_the_spec_015_tags`
and `embedded_catalog_loads_the_reference_recipes_without_skips`) are the hardcoded
vocabulary/id expectations the coordinator owns; as instructed, I did not edit them. My
standalone test proves the new vocabulary itself is valid and ordered.

## Left open (owned elsewhere)

- `the-10x10-project` carries `gimmick`: worker A owns the recipe edit; my test asserts it
  once that recipe is embedded, and does not touch A's file now.
- Discover category rendering/navigation: worker D owns the runtime ordering change.
- Coordinator: combined vocabulary/id assertions, combined fmt/clippy/workspace tests.
- No gameplay validation is relevant for Linear Doom because no recipe exists; for the
  future entry it would be Needs Deck Validation.

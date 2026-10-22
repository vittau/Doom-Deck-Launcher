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
| `docs/research/reference-087-b.md` | Chronology correction: Deathless original full release 2019-11-09, selected v1.1 update 2020-05-25 (see §5) |

Not changed: any of the six settled recipes, `catalog/hosts.toml`, `catalog/linear-doom.toml`
(not created), shared `crates/ddl-core/src/catalog/tests.rs`, runtime/UI, CONTRIBUTING,
dependencies or coordinator docs. No screenshot directory was created.

## Linear Doom — verdict and blocker (summary)

The author is DeXiaZ; the ModDB v1.2 file page records `Linear_Doom_v1.2.7z`, 274,927 bytes,
MD5 `4f758b008649e7424ea634c615f34de7`, added 2019-04-06, described as a jokewad that
"makes all classic Doom 1 levels straightened into one line", with notes referencing E1M4,
E1M8, E2M4 and E3M6. The original publication is 2019-03-31; the page header rounds it to
"Released Apr 1, 2019".

Why no recipe:

1. **No fetchable URL.** Every live ModDB page (mod, downloads, v1.2 file, start endpoint,
   member downloads) answered HTTP 403 with a Cloudflare challenge on 2026-10-22. The file
   is served only through an interactive `/downloads/start/176643` gate; the eventual CDN
   address is per-request and expiring and was not observable without bypassing the
   challenge, which is explicitly out of scope.
2. **No host change would help.** Adding `www.moddb.com` would still hit the 403 gate; adding
   a CDN host without observing its exact name would violate the exact-host rule, and
   expiring links/wildcards/downloader exceptions are forbidden by R4. `catalog/hosts.toml`
   is therefore unchanged.
3. **No alternate compliant source.** DSDA has no `linear-doom` entry (HTTP 404) and no
   `/idgames` path was found; a third-party mirror would be a new trust decision.
4. **Compatibility cannot be established.** Without the archive, the map count, IWAD variant
   and lump inventory are unknown. DSDA-Doom 0.30.1 runs Doom-format/Boom/MBF/MBF21 data
   plus UMAPINFO and ignores GZDoom-only `ZMAPINFO`/`MAPINFO`/`DECORATE`/ACS content; the
   author's "Tested with GZDoom 2.4.0" note is neither a requirement nor proof of DSDA
   support. Claiming either would be invented.

Published MD5 and byte size are recorded as identity evidence, but the schema needs SHA-256
for the package and every named member plus a fetchable URL, so the recipe cannot be pinned.

Unblock options (any one): a durable author-hosted URL on an already-reviewed host or the
archive supplied for hashing; an owner-approved policy able to handle ModDB's interactive
gate; or a reviewed byte-identical mirror on an existing allowlisted host.

## Dates for spec 090

- Linear Doom original public release: **2019-03-31** (day precision); selected update v1.2
  **2019-04-06**. Research only — no recipe, so no chronology metadata is exposed.

## Deathless chronology correction

The earlier research row used the selected v1.1 upload date (2020-05-25) as the original
release. DoomWiki (Wayback copy read 2026-10-22) records that the full megawad's initial
version was shared on **2019-11-09**, after the July 2018 E4-only predecessor *Griefless*,
which is a different release. The row now says 2019-11-09 (day precision) and keeps
2020-05-25 as the selected v1.1 update; the recipe's version label and hashes are unchanged.

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

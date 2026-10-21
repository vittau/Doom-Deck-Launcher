# 073 — Memento Mori classics and the Doomer Boards category

## Scope and result

Implemented spec 073 R1–R5 as the catalog worker: two new verified Classics recipes
(`memento-mori`, `memento-mori-ii`), Scythe verified in place (no change, no duplicate), the
requested Discover vocabulary order plus a new `doomer-boards` tag, `doomer-boards` membership for
the 77 verified DBP recipes, focused offline tests, and the research behind every byte. Local
`main` (`d1ca482`, the latest spec commit with the owner's explicit category order) was merged
before the final checks. No UI, runtime, services, screenshots, config, audio, workflow or
coordinator-owned file was touched.

## Recipes

| | `memento-mori` | `memento-mori-ii` |
| --- | --- | --- |
| Archive | `themes/mm/mm_allup.zip` | `themes/mm/mm2.zip` |
| Bytes / SHA-256 | 2,950,178 / `c8607a7a32d4f6547d492e0f734324320b955e2b11691b02b107de09ac2e7bfb` | 4,061,135 / `f4e7a10b035b123647f631348d63fc46392d8d8e21fab8a5158cd9ef35027451` |
| Mirrors verified byte-identical | `www.gamers.org`, `youfailit.net`, `ftp.fu-berlin.de`, `mirrors.lug.mtu.edu` | same four |
| Members (size / SHA-256) | `MM.WAD` 7,825,282 / `3e34522f…a22e`; `MMMUS.WAD` 885,387 / `3e1618f7…5e73`; `MM.TXT` 2,893 / `33f67e46…5bee` (+ `AUTHORS`, `MMHELP`, `CHANGES`, `FILE_ID.DIZ`) | `MM2.WAD` 11,453,000 / `810c3eb9…859b`; `MM2MUS.WAD` 1,075,404 / `d0935061…22d5`; `MM2.TXT` 42,580 / `87067853…54e5` (+ `FILE_ID.DIZ`) |
| Base game / format | Doom II, vanilla, `complevel = "vanilla"` (cl2) | same |
| Load order | `MM.WAD` then `MMMUS.WAD` — MMHELP.TXT Q4: `doom2 -file mm.wad mmmus.wad` | `MM2.WAD` then `MM2MUS.WAD` — `MM2.BAT`: `doom2 -file mm2.wad mm2mus.wad` |
| Music | `MMMUS.WAD`, 35 `D_*` lumps, Mark Klem | `MM2MUS.WAD`, 35 `D_*` lumps, Mark Klem and David Shaw |
| Tags | `classics`, `community-projects`, `megawad` | same |
| Release | Updated version, February 1996 (idgames file dated 1996-02-21; first release 1995-12-10) | 1996-07-27 |

Both map WADs carry `MAP01`–`MAP32`, a custom `TEXTURE1`/`PNAMES` and **no** `DEHACKED`,
`COMPLVL`, `UMAPINFO` or `MAPINFO` lump (read as data, nothing executed), so the complevel is
declared by the recipe from the releases' Doom II v1.9 target. The music WADs are separate files in
the official packages: each recipe's single required `main` component loads the map WAD and then the
music WAD, exactly as the authors' own launch lines do (`doom2 -file mm.wad mmmus.wad`,
`doom2 -file mm2.wad mm2mus.wad`). No extra component or toggle was added.

Full evidence, quoted readme sections, per-member hashes and the reproduction commands:
`docs/research/reference-memento-mori.md`.

## Category work (R5, owner expansion)

- `catalog/tags.toml`: orders 10–70 are now exactly Cacowards, Megawads, Community Projects,
  Doomer Boards, Classics, Episodes, Challenging. `featured` (80) and `beginner-friendly` (90) keep
  their vocabulary and description but sit outside that sequence; spec 072 hides them from the
  strip. New tag: `doomer-boards` / "Doomer Boards", citing the forum project.
- `doomer-boards` added to **all 77** recipes named by the verified DBP inventory in
  `docs/research/doomer-boards-projects.md` (DBP 01–77, one row each, including `auger-zenith` =
  DBP37 and `a-doomer-boards-christmas-carol` = DBP19). No membership comes from name guessing.
- Every DBP recipe keeps `community-projects`. Multi-author recipes that are **not** Doomer Boards
  Projects keep `community-projects` and were not relabelled: `alien-vendetta`,
  `back-to-saturn-x-e1`, `back-to-saturn-x-e2`, `eviternity`, `eviternity-ii`, plus the two new
  Memento Mori classics.
- Scythe already carried `classics` (and `megawad`); nothing changed, and a test now pins that there
  is exactly one Scythe recipe with its reviewed package size.

## Verification

All commands run on the macOS arm64 dev host, `CARGO_BUILD_JOBS=2`, `--test-threads=2`.

| Check | Command | Result |
| --- | --- | --- |
| Formatting | `cargo fmt --all --check` | clean |
| Core clippy | `cargo clippy -p ddl-core --all-targets -- -D warnings` | clean |
| Focused catalog tests | `cargo test -p ddl-core --lib -- --test-threads=2 catalog` | 110 passed, 0 failed |
| Core suite (offline) | `cargo test -p ddl-core --lib -- --test-threads=2` | 632 passed, 0 failed |
| Offline catalog lint | `ddl catalog lint catalog/memento-mori.toml catalog/memento-mori-ii.toml` | `Catalog lint passed (offline)` |
| Real-fetch catalog lint | `ddl catalog lint --fetch catalog/memento-mori.toml catalog/memento-mori-ii.toml` | `Catalog lint passed (fetched packages verified)` |

New tests in `crates/ddl-core/src/catalog/tests.rs`:

- `shipped_vocabulary_is_valid_and_has_the_spec_015_tags` — updated to the new nine-tag order.
- `requested_discover_sequence_leads_the_vocabulary` — the seven requested categories are the
  vocabulary prefix, in order; Featured/Beginner Friendly stay in the vocabulary outside it.
- `doomer_boards_covers_every_researched_dbp_recipe` — the tagged set equals the 77-entry inventory
  list (and nothing else), each still carrying `community-projects`.
- `non_dbp_community_projects_stay_out_of_doomer_boards` — the general community projects and both
  new classics are not DBP-tagged.
- `memento_mori_recipes_keep_their_source_identity` — package size/SHA-256/kind, map+music+readme
  member identities, `doom2`, vanilla family/complevel, one required `main` component whose load
  order is maps then music (via `Recipe::load_order`), `classics`+`megawad` reachability, the kept
  credits text, and Scythe's single `classics` entry.
- The shipped-id list and the recipe/file-hash loop in
  `embedded_catalog_loads_the_reference_recipes_without_skips` gained both new ids.

## Cross-worker integration note (spec 072)

Changing the vocabulary order changes what `tag_labels` returns for recipes with several tags. The
existing `crates/ddl/src/runtime.rs` test
`cards_show_at_most_two_tag_chips_led_by_the_open_category` asserts vocabulary order
`["Classics", "Cacowards"]` for `eviternity`, which is now `["Cacowards", "Classics"]`; the same
file's chip fixtures also list the old strip labels. Those are the 071/072 worker's files, so they
were **not** edited here — the UI worker needs to update that expectation while doing the 072
strip change. Recipe-count-wide lists in `crates/ddl` (if any) will also see two more recipes.

## Deck limitations and open items

- **Needs Deck Validation** for both recipes: IWAD selection, complevel 2, `-file` order (map WAD
  then music WAD), single-player gameplay, music playback/MIDI device and map selection were not
  played; only desk research and byte inspection were possible on this host.
- No screenshot was added: the 1995/96 archives ship text-only credits, the crew's homepage is
  gone, and the task allows only verified licensed upstream images. The UI falls back to the
  painted concept art for these two recipes.
- Not added (out of scope, proposed for a follow-up if wanted): the optional NightFright widescreen
  status bar that `scythe` offers, which would suit `memento-mori` because its `MM.WAD` ships no
  `STBAR` (`memento-mori-ii` ships its own).
- The MM2 top-secret outtake levels live in the separate DOS infopack `mm2info.zip`
  (`MM2INFO.DAT`), not in `mm2.zip`; DDL does not run DOS programs, so they are out of scope.
- `MM2.BAT` (32 bytes) is not a member and is never extracted or executed.

## Files

New: `catalog/memento-mori.toml`, `catalog/memento-mori-ii.toml`,
`docs/research/reference-memento-mori.md`, this report.
Changed: `catalog/tags.toml`, the 77 DBP recipes (`tags` line only), and
`crates/ddl-core/src/catalog/tests.rs`.

# Spec 090 report — release order, focused Community Projects, and the player-supplied capability (spec 094)

**Worker:** D (same worktree, `pwads-087-d`) · **Date:** 2026-10-22 · **Status:** implementation,
evidence and focused checks complete; combined workspace verification and physical Deck checks
are the coordinator's / owner's remaining steps.

## What changed

| Area | Change |
| --- | --- |
| Model | `Recipe.original_release: Option<ReleaseDate>` (`year`, optional `month`, optional `day`), validated and serialized; `Recipe.packages` is now optional-empty for import-only entries; machine-written recipe copies (`catalog::persisted`) read both fields, defaulting old copies. |
| Lint | Present dates must be real calendar values (1990–2100, month 1–12, real day, no day without month). A zero-package recipe is valid only when its required `main` is the single player-supplied import; every other member still needs a declared package. |
| Ordering | `catalog::compare_browsing_order` (ascending first-public-release, unknown last, title/id tie-break, coarser precision after the same period's finer dates) and `catalog::shows_in_category` (Community Projects hides `doomer-boards`; the dedicated tab keeps them). |
| Runtime | Discover (every category and Favorites) and Library use the chronology comparator, never favorite-first. The strip omits a category that would be empty after filtering. `gimmick` follows `challenging` (spec 091). Setup rows: a required user-supplied main is focusable and opens Import/Replace; its sheet offers Import + a safe Done, never On/Off. |
| Compatibility | `DEFAULT_MIN_DDL_VERSION` and the workspace version are `0.0.12` (same gate for the `original_release` field and the zero-package capability), with the bundle refusing pre-0.0.12 clients whole. |
| Recipes | `original_release` on all 118 shipped recipes with cited evidence and precision; no package/member/component/description edits. |
| Docs | `CONTRIBUTING.md` documents the field and the `gimmick` evidence rule; `docs/research/catalog-release-dates.md` holds the per-recipe table and policy. |
| Tests | `crates/ddl-core/tests/catalog_090.rs` (10 offline tests); focused headless journeys for category order/favorites/filtering, Library rows and the spec 094 import row/sheet. |

## R1 — Community Projects

`shows_in_category(recipe, tag)` is the one membership rule: a recipe carries the tag, except
that `community-projects` hides `doomer-boards` recipes. The tags stay accurate, so the Doomer
Boards tab, other earned tabs, favorites, installations and profiles are unchanged.
`catalog_090.rs` proves every embedded DBP is hidden from Community Projects and still present
under Doomer Boards, and that ordinary community projects (Eviternity, BTSX E1) remain visible.
The existing `015` production journey now compares each category against the filtered set.

## R2/R3 — Chronology and dates

- Sorted key: `(year, month.unwrap_or(13), day.unwrap_or(32))`, so a year-only date follows the
  year's dated entries and a month-only date follows that month's days; never an invented
  January 1. Unknown dates sort after all dated ones. Ties: case-insensitive title, then id.
- Favorites no longer reorder anything; the `015` journey records the card position before and
  after favoriting and requires it unchanged.
- `release` remains a free-form identity string and is never parsed: the version-label test
  sorts `9.0`/1998 before `10.0`/2001.
- All 118 recipes carry a date: 115 day precision, 3 year precision (Abandon 2020, BTSX E1 2013,
  BTSX E2 2014 — each source stops at that granularity), 0 month-only. The evidence and the
  first-release-vs-update reasoning are in `docs/research/catalog-release-dates.md`.
- Date normalisation recorded: Stardate 20X6 = 2013-06-02 (readme `06/02/2013`, ZIP text member
  2013-06-02 17:50:00, WAD 2013-05-31), Abandon = 2023-05-07 (text member), Going Down /
  Overboard / Headless Chicken use the authors' day-first convention (proved by inline
  `19/01/2026` for Headless Chicken). Deathless uses 2018-11-09 (author release post) while the
  pinned identity is the 2020 update; Eviternity 2018-12-10 (thread) while the pinned build is
  2019-02-10; and so on per recipe.

## R4 — Compatibility and the spec 094 capability

- Old recipes, cached bundles and machine-written recipe copies missing the optional fields load
  (`serde(default)` in `Recipe` and `catalog::persisted::StoredRecipe`). A legacy bundle/cache
  round-trip is a test.
- Malformed present dates are lint errors (`original_release.month/day/year`).
- **Gate:** pre-0.0.12 readers use `deny_unknown_fields`, so the published bundle must not reach
  them — it would silently drop every dated recipe. `DEFAULT_MIN_DDL_VERSION` and
  `Cargo.toml` are now `0.0.12`; a 0.0.11 client is refused whole and keeps its embedded/cached
  catalog. D-038 permits raising the gate only for a real capability change, which this is.
- **Spec 094:** a zero-package recipe is valid only with a required, user-supplied `main` and no
  package references in its members; all other validation is unchanged, and a downloaded recipe
  with no packages is still rejected. Installing is metadata-only (no package exists to fetch),
  Play is blocked with `ComponentNotProvided`, and the required Setup row imports/replaces with
  no Off toggle. A's real NERVE import/hash and argv tests remain in `catalog_094.rs`.

## R5 — Checks run (focused)

```text
$ cargo test -p ddl-core --test catalog_090 -- --test-threads=2
10 passed; 0 failed

$ ddl catalog lint catalog/
Catalog lint passed (offline).          # 118 recipes, including the zero-package entry

$ cargo fmt --all --check
clean

$ cargo clippy -p ddl-core --lib -- -D warnings
no warnings
$ cargo clippy -p ddl-core --test catalog_090 -- -D warnings
no warnings
# The merged A-owned tests/catalog_094.rs has one `needless_lifetimes` warning for
# its own test helper; it is outside this worker's ownership and is reported for
# the coordinator/A to fix before the combined clippy run.

$ cargo test -p ddl --bin ddl categories_favorites_and_screenshots -- --test-threads=1
ok   # category order, DBP exclusion, favorite does not move a card, screenshot fallbacks

$ cargo test -p ddl --bin ddl library_owner_scene_renders -- --test-threads=1
ok   # Library rows align by displayed position under chronology

$ cargo test -p ddl --bin ddl player_supplied_expansion_imports_its_required_file -- --test-threads=1
ok   # metadata-only install, blocked Play, focusable Import row, file picker

$ cargo test -p ddl --bin ddl required_import_sheet_has_no_toggle -- --test-threads=1
ok   # required main: Import + Done, never On/Off; confirming does not toggle
```

Fetch lint is not applicable to the import-only entry (no downloads) and was not re-run for
unchanged packages. The coordinator runs the combined fmt / workspace clippy / workspace tests
and the final 118-recipe embedding count after integration.

## Open questions and Deck checks

- **Needs Deck Validation:** controller navigation and focus after the order/filter change
  (Discover strip, Favorites, Library), the import picker focus after replacing a required file,
  and long-list scrolling on the 1280×800 panel.
- **Needs Deck Validation:** NRFTL with each accepted NERVE variant (BFG/XBLA, Steam/KEX),
  filename preservation for DSDA's case-insensitive mission selection, and the unknown-hash
  warning for the KEX variant.
- No unresolved date is fabricated: the three year-precision entries are the only ones whose
  sources stop short of a day.

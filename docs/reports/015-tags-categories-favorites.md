# Spec 015 — Recipe tags, Discover categories and favorites

Status: R1–R6 implemented and verified headlessly. A4 (controller on a real Deck) is open.
No visible window was opened and no screen was captured: every PNG comes from Slint's
off-screen software renderer.

## What changed

- **R1 vocabulary** — `catalog/tags.toml` (`schema_version = 1`, `[[tags]]` with `id`, `label`,
  `description`, `order`): `featured`, `beginner-friendly`, `classics`, `cacowards`,
  `community-projects`, `megawad`, `episode`, `challenging`. `crates/ddl-core/build.rs` embeds it
  as `EMBEDDED_TAGS` (and no longer treats it as a recipe); `Catalog` now carries `tags`.
  `Catalog::from_sources` takes the vocabulary text as a new second argument.
- **R2 recipe field** — optional root key `tags = ["…"]` (schema stays 1; absent = none; empty
  lists are not serialized, so untagged recipes keep their canonical form). Lint
  (`validate_tags`, `validate_vocabulary`, wired into `lint_paths` and the embedded loader):
  unknown tag, duplicate tag, and `cacowards` without a `doomworld.com/cacowards/<year>` citation in
  `verification.notes` or `verification.inspected`. `lint_paths` reads `tags.toml` beside
  `hosts.toml` and never lints it as a recipe; an invalid vocabulary is reported with file and
  line. The loader skips a recipe with an unknown tag (logged reason), like any other lint failure.
  `CONTRIBUTING.md` documents tags, the evidence each one needs and how to propose a new one.
- **R3 tags on the recipes** — see the table below; every tag claim is backed in the recipe's
  `verification.notes` (and `inspected` for Cacowards).
- **R4 Discover strip** — "All", each vocabulary tag some recipe uses (in `order`), "Favorites";
  empty Favorites shows "No favorites yet" with the instruction and a "Show all mapsets" button.
- **R5 favorites** — `X` toggles on a focused Discover/Library card and on Mapset detail (hint bar
  shows "Favorite"/"Unfavorite"; detail also has an "Add to favorites" button); persisted in
  `app.json`; favorites sort first in every category and in Library; Library shows the star.
- **R6 chips** — at most two chips on a card (the open category's tag leads), all on detail; the
  star and chips use Ember tokens (`deep`/`raised`/`line` on bronze, original `favorite.svg`).

### Controller bindings (also in `docs/design/ui.md`)

| Input | Where | Effect |
| --- | --- | --- |
| Up | first grid row | enters the strip at the **selected** chip |
| Left / Right | strip | moves the chip and switches the category at once; no wrap |
| Down | strip | first card (or "Show all mapsets" when Favorites is empty) |
| A | chip | switches (already done by focus) and enters the grid |
| L2 / R2 (keyboard `[` `]`) | Discover, strip or grid | previous/next category, no wrap |
| X | card, detail | toggle favorite |

The category is kept in the `Runtime` for the session (it survives tab switches and Back from
detail). A strip wider than the grid (9–10 chips at 20 px) scrolls, keeps the selected/focused chip
visible and shows `‹`/`›` for hidden chips. The grid moved down by 72 px on Discover only; the
card text rows were tightened so the card stays 410 px high.

## State: `app.json` schema 2 with migration

`AppState` gains `favorites` (sorted set of recipe ids) and `schema_version` becomes 2.
`AppState::parse` loads schema 1 (migrated in memory; the file is rewritten as schema 2 by the
next save) and schema 2; anything else is still rejected. Spec 021's additive
`search_standard_folders` (merged from `main` while this branch was open) is carried unchanged in
schema 2, as that merge anticipated. Toggling an id the catalog does not know is refused.
Favorites go through the existing worker/`Command::Favorite` path (state lock, atomic write); the
UI flips the mark optimistically and the returned snapshot replaces it.

Install snapshots in `app.json` pre-date tags, so `snapshot()` copies the **catalog's current tags**
onto each item for display; launch data stays frozen in the snapshot (D-019).

## Tags on the recipes

| Recipe | Tags | Evidence |
| --- | --- | --- |
| `auger-zenith` | featured, classics, cacowards, community-projects | Cacowards 2021 **runner-up**; released 2021-07-26; DBP37 with 30+ credited contributors; D-010 reference recipe. 22 maps → not a megawad by the 30-map rule. |
| `eviternity` | classics, cacowards, community-projects, megawad | Cacowards 2019 **winner**; final 2019-02-10; 12 credited authors; 32 maps. |
| `sigil` | featured, classics, cacowards, episode | Cacowards 2019 **runner-up**; first free release 2019-05-01; E5M1–E5M9; D-010 reference recipe. |
| `sigil-ii` | featured, episode, challenging | E6M1–E6M9; romero.com: "some hardcore classic DOOM punishment"; D-010 reference recipe. No Cacoward found. |
| `scythe` | classics, megawad | Released 2003-04-08; 32 maps. No Cacoward for the original (its sequel won 2005). |
| `alien-vendetta` | classics, community-projects, megawad | Compet-N edition 2002-12-24; 13 contributors in `AV.TXT`; 32 maps. |
| `going-down` | classics, megawad | First complete release 2014-09-14; 32 maps. |
| `valiant` | classics, cacowards, megawad | Cacowards 2015 **winner**; first released 2015-02-07; 32 maps. |

Rules I applied (written into `tags.toml` descriptions and CONTRIBUTING): `classics` = release
at least five years before the review plus a cited sign of recognition; `megawad` ≥ 30 maps;
`episode` ≤ 9 maps; `featured` = maintainer pick with a stated reason.

### Evidence caveat (coordinator-confirmed, option A)

`doomworld.com` and `doomwiki.org` answer every automated fetch with a Cloudflare 403, so the
Cacowards year pages were **not read directly**. The years and roles come from web-search snippets
of DoomWiki's Cacowards pages (secondary evidence) and each recipe says so. The coordinator
confirmed that `cacowards` means "honoured in the Cacowards (winner **or** runner-up)", with the
year page and the role cited. Cited URLs follow `https://www.doomworld.com/cacowards/<year>/`
(Eviternity also `best1/`, as returned by the search); **please click through each once** — a human
browser reaches them.

### Candidates deliberately not tagged (need evidence first)

- `going-down`: DoomWiki links it from its Cacowards 2014 page, but the category could not be
  confirmed. `alien-vendetta`, `scythe`: no Cacoward confirmed.
- `sigil`: widely described as hard, but no primary statement is in the repo → no `challenging`.
- `beginner-friendly` is in the vocabulary but unused, so Discover does not show its category yet.

## Requirement → verification

| Item | Evidence |
| --- | --- |
| R1 | `catalog::tests::shipped_vocabulary_is_valid_and_has_the_spec_015_tags`, `vocabulary_rules_reject_bad_ids_text_and_order`; embedded vocabulary equals the file. |
| R2 / A1 | `unknown_and_duplicate_tags_are_lint_errors`, `cacowards_tag_needs_a_year_page_citation_in_notes_or_inspected`, `lint_paths_reports_unknown_tags_with_file_and_line` (file:line:hint), `tags_round_trip_and_are_optional_in_schema_one`, `loader_skips_recipes_with_unknown_tags_and_keeps_known_ones`; `ddl catalog lint` passes on the whole catalog; CONTRIBUTING updated. |
| R3 | all 8 recipes lint clean with tags (`reference_recipes_round_trip_and_lint_offline`, `embedded_catalog_loads_…`, `ddl catalog lint`). |
| R4 / A2 | `headless_production::categories_journey`: strip labels/order vs the vocabulary, Up/Left/Right/Down/L2/R2 navigation, filtering per tag, no wrap, empty Favorites with "Show all mapsets", category kept across Back and tab switches; `runtime::category_tests` (strip windowing at every selection, chip packing/rows, tag order, author line). |
| R5 / A2 | `services::tests::favorites_toggle_persist_and_reject_unknown_recipes`, `schema_one_state_migrates_to_two_keeping_folders_and_installs`, `install_snapshots_without_tags_show_the_catalog_tags`, `an_existing_state_file_without_the_setting_loads_off_and_round_trips` (now asserts schema 2); journey: X persists and sorts first, Library star (`015-library-favorite.png`), detail X and button. |
| R6 / A2 | journey: ≤ 2 chips on cards (unit-tested), all chips on detail equal the vocabulary labels of the recipe's tags. |
| A3 | fmt, clippy `-D warnings`, `cargo test --workspace` — tails below. |
| A4 | **Open — Needs Deck Validation.** |

## Headless renders (inspected)

`DDL_HEADLESS_DIR=<dir> cargo test -p ddl --bin ddl headless` writes, besides the existing set:
`015-discover-all.png` (strip + cards, focus ring), `015-discover-category.png` (Cacowards),
`015-discover-favorites-empty.png`, `015-discover-all-favorite.png` (starred card first),
`015-discover-favorites.png` (non-empty), `015-detail-tags.png` (all chips, star, favorite button),
`015-library-favorite.png`. I looked at each. Findings fixed during review: "Megawads" wrapped in its
chip (dedicated centred non-wrapping chip node, wider estimate); Eviternity's 12 authors overflowed
on detail and cards (authors elided at whole names); a card's second chip hid the active category's
tag (the open category's tag now leads); the chip count/rows helper over-reported rows (caught by a
unit test). The existing rail-alignment assertion for the focused Discover card moved down 72 px.

## Verification output

```text
cargo fmt --all --check                                  → exit 0
cargo clippy --workspace --all-targets -- -D warnings    → Finished `dev` profile, no warnings
cargo test --workspace --no-fail-fast
  ddl (unit)            59 passed; 0 failed   (headless journeys included)
  ddl tests             catalog_cli 2, cli_help 2, logo_assets 2, ui_strings 1 passed
  ddl-core (unit)      290 passed; 0 failed
  ddl-core tests        dsda_launch 16 passed;  doc-tests 3 passed
cargo run -p ddl -- catalog lint                         → Catalog lint passed (offline).
```

## Notes for the coordinator

- **Recipe files changed beyond the `tags` line.** R2 requires the Cacowards citation inside the
  recipe (`verification.notes`/`inspected`), and every tag claim needs evidence, so I appended
  sentences to `notes` (and one `inspected` entry) of each tagged recipe. Any tags edit changes a
  recipe's canonical content hash (`library::install::recipe_content_sha256`); that is only
  compared in `install()` against the **install snapshot kept in `app.json`**, so existing installs
  do not trip `RecipeChanged`. Spec 020 (recipe change after install) should keep tags out of that
  hash if it wants tag edits to be free.
- **Other files outside the stated ownership:** `crates/ddl-core/src/library/tests.rs` and
  `profiles/mod.rs` only gained `tags: Vec::new()` in their test `Recipe` literals (needed to
  compile). `crates/ddl/tests/ui_strings.rs` allows the new `[`/`]` key literals.
- **Batch B is not on `main` yet** (Sunlust, Ancient Aliens, Back to Saturn X E1/E2,
  Eviternity II): those recipes are **untagged**; the coordinator should tag them in a follow-up
  after merging them. Candidates from what I found: Ancient Aliens — Cacowards 2016 (skillsaw's
  Cacowards list says so; read the year page first); Eviternity II — Cacowards 2024 (secondary).
  Both tests and the strip are data-driven, so new recipes need no Rust change.
- **Needs Deck Validation (A4):** L2/R2 are mapped to gilrs `LeftTrigger2`/`RightTrigger2`; on a Deck
  without Steam Input these may arrive as axes and never fire (keyboard `[`/`]` always work, and
  Left/Right on the strip covers the controller). Chip text at 20 px and the 72 px grid offset need a
  look on the LCD. No Steam Input code was added.
- **Proposals (not made):** a `docs/status.md`/journal line for spec 015; a decision note that
  `cacowards` includes runner-ups and that `classics` needs an age + recognition citation.
- **Known, pre-existing:** the top tab bar highlights "Home" while a Detail screen is open
  (`tab_index` defaults to 0 outside root screens).
- **Dependencies:** none added.

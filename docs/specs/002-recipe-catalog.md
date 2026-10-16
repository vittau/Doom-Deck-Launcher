# 002 — Recipe schema, catalog and lint

**Status:** Done  ·  **Milestone:** M2 (schema pulled forward)  ·
**Depends on:** 001; evidence in `research/dsda-engine.md`, `research/upstream-sources.md` §5–§6,
`research/reference-sigil.md`; decisions D-010, D-011, D-014, D-015, D-016

## Goal

A mapset becomes playable by adding **one human-readable TOML file** to `catalog/` through a pull
request. The file says where to download each package, which files inside it to load, in what
order, on which base game and complevel, with which optional components, and who verified it.
`ddl catalog lint` validates it locally and in CI, so contributors get precise feedback and
maintainers can review with confidence. End users only ever see the curated result.

## Requirements

- **R1** — One recipe per file: `catalog/<id>.toml`, where `<id>` is a stable lowercase slug that
  matches the file name. Versioned by `schema_version`.
- **R2** — **Identity and credits:** title, authors, release/version label, canonical page URL,
  description, optional image references with stated permission, license or permission text.
- **R3** — **Base game:** the required game id (`doom`, `doom2`, `tnt`, `plutonia`) and, when
  the content needs it, the accepted IWAD variants (e.g. SIGIL requires Ultimate DOOM 1.9 and
  refuses shareware). Variant ids refer to DDL's known-IWAD table (separate spec); until that
  table exists, the lint accepts any variant id matching `^[a-z0-9][a-z0-9.-]*$` (the research ids contain dots, e.g. `doom2-1.9`). Shown to the user; Play is
  blocked without a matching IWAD (D-010).
- **R4** — **Packages:** one or more downloads, each with id, canonical page (e.g. the /idgames
  entry), direct URLs on allowlisted hosts (primary plus mirrors, D-015), exact size and SHA-256,
  and type (`zip` or a bare `wad`). Each file DDL needs is a **member** named by its exact,
  case-sensitive path inside the package, crossing nested ZIPs with `!/`
  (`DBP37_Addons.zip!/DBP37_AlternateBlaster.wad`), and carries its own size and SHA-256.
  Permission/credit text files (e.g. `SIGIL_V1_23.txt`) are listed as `keep` members so they
  stay in the install unmodified.
- **R5** — **Components:** a required `main` component and zero or more **optional components**,
  each with id, label, description, default on/off, its members and their roles (maps, music,
  graphics, sounds, dehacked), and an optional `exclusive_group` (at most one component of a
  group is enabled). A component may **replace** the main one (SIGIL's E3 compat WAD; a paid
  soundtrack WAD that replaces the main WAD) instead of adding to it. A component may be
  `user_supplied` (D-014): no URL, a label telling the user where to obtain it, and optional
  known SHA-256 values.
- **R6** — **Load order:** explicit position for every component (`main`, then optional
  components in declared order unless a component says otherwise), producing one ordered `-file`
  list and one ordered `-deh` list (D-011) for every allowed combination. DEHACKED lumps inside
  WADs load automatically and are not listed. Nothing in a package is loaded unless named.
- **R7** — **Compatibility:** port family (`vanilla`, `limit-removing`, `boom`, `mbf`, `mbf21`)
  and an explicit complevel from the reviewed set (`vanilla`, 0–17, 21, `boom`, `mbf`, `mbf21`;
  never the placeholders 18–20, never omitted; D-011); optional start map (`-warp`); the tested
  engine version range; extra runtime options only from a typed allowlist (no free-form command
  lines).
- **R8** — **Verification evidence:** status (`verified`, `partial`, `incompatible`, ...),
  tester, date, engine version, notes and known limitations (OVERVIEW §7 stages).
- **R9** — **Lint** (`ddl catalog lint [paths]`): schema validity, unknown keys rejected, id/file
  name match, allowlisted hosts, HTTPS, well-formed hashes, load order covers every combination,
  file roles consistent, no duplicate ids. Errors cite file, line and fix hint. Offline by
  default; `--fetch` downloads packages and checks size, hash and that every named path exists in
  the archive.
- **R10** — **Contributor experience:** `CONTRIBUTING.md` section "Add a mapset" with a
  copy-paste template and an annotated example; a pull request template asking for verification
  evidence; CI runs the lint on every pull request that touches `catalog/`.
- **R11** — Reference recipes AUGER;ZENITH (with add-on), SIGIL and SIGIL II are written and lint
  clean, and serve as test fixtures.

## Out of scope

Downloading and installing at runtime (download/extraction spec), resolver internals beyond
order validation, UI, `/idgames` search.

## Acceptance criteria

- [x] **A1** (R1–R8) — typed Rust model in `ddl-core` with serde, round-trip tests on the
  reference recipes.
- [x] **A2** (R9) — lint unit tests per rule, including failing fixtures with expected messages.
- [x] **A3** (R9) — `ddl catalog lint --fetch` passes for the three reference recipes.
- [x] **A4** (R10) — CI job lints `catalog/` on pull requests.
- [x] **A5** (R11) — three reference recipes merged.

## Reference recipe data

Take every value from the research documents; do not invent any.
- **AUGER;ZENITH** — D-016 and `research/upstream-sources.md` §5. Status `inspected` (not yet
  Deck-verified); complevel 2 marked Proposed.
- **SIGIL** v1.23 and **SIGIL II** v1.0 — `research/reference-sigil.md` §1–§5: Ultimate DOOM,
  complevel 3 marked Proposed, SIGIL's compat WAD as an exclusive replacement, the paid
  soundtracks as `user_supplied` replacements. Status `inspected`.

## Initial host allowlist

`www.doomworld.com` (pages only), `www.gamers.org`, `dsdarchive.com` and other /idgames mirrors
listed in `research/upstream-sources.md` §3.4, `romero.com` (and the exact download host it
uses, per `research/reference-sigil.md` §1). Kept in `catalog/hosts.toml`.

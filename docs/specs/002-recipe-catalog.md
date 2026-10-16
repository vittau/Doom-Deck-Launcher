# 002 — Recipe schema, catalog and lint

**Status:** Draft (waiting on `research/dsda-engine.md`, `research/upstream-sources.md`,
`research/reference-sigil.md`)  ·  **Milestone:** M2 (schema pulled forward)  ·  **Depends on:** 001

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
- **R3** — **Base game:** the required IWAD by game id (e.g. `doom` Ultimate DOOM, `doom2`,
  `tnt`, `plutonia`) plus the accepted IWAD versions by hash. It is shown to the user;
  Play is blocked without a matching IWAD (D-010).
- **R4** — **Packages:** one or more downloads, each with id, URL(s) from allowlisted hosts
  (primary and mirrors), exact size and SHA-256, and archive type (ZIP first; a bare WAD is a
  package without an archive). Each file DDL needs is named by its exact path inside the archive.
- **R5** — **Components:** a required `main` component and zero or more **optional components**
  (e.g. AUGER;ZENITH's add-on PWAD, a SIGIL soundtrack WAD), each with a label, description,
  default on/off, its files and their roles (maps, music, graphics, dehacked, ...), and any
  conflicts with other components. Variant choices that exclude each other (e.g. SIGIL's compat
  WAD) are modelled explicitly.
- **R6** — **Load order:** explicit and total for every allowed combination of components.
  `-file` and `-deh` placement follow the engine research. Nothing in an archive is loaded unless
  the recipe names it.
- **R7** — **Compatibility:** port family and complevel using only values DSDA-Doom accepts; the
  tested engine version range; required runtime arguments limited to a typed allowlist (no
  free-form command lines).
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

- [ ] **A1** (R1–R8) — typed Rust model in `ddl-core` with serde, round-trip tests on the
  reference recipes.
- [ ] **A2** (R9) — lint unit tests per rule, including failing fixtures with expected messages.
- [ ] **A3** (R9) — `ddl catalog lint --fetch` passes for the three reference recipes.
- [ ] **A4** (R10) — CI job lints `catalog/` on pull requests.
- [ ] **A5** (R11) — three reference recipes merged.

## Open questions (resolved by research before Ready)

- Exact DSDA CLI handling of `-file` vs `-deh`, and whether DEHACKED lumps inside WADs load
  automatically.
- Complevel values and aliases to allow; how to express "let the engine detect it" (COMPLVL lump).
- Whether the official sources (romero.com, Doomer Boards hosts, /idgames mirrors) allow direct
  download, and the host allowlist that follows.

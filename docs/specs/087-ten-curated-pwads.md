# 087 — Ten curated PWAD additions

**Status:** Ready. **Origin:** owner requests every title from the proposed ten-PWAD selection, implemented by parallel OpenCode workers.

## Goal

Expand the curated catalog from 92 to 102 recipes with Scythe II, Plutonia 2,
Doom the Way id Did, Arrival, Vanguard, Lunatic, No End in Sight, UAC Ultra,
Japanese Community Project and Stardate 20X6. Preserve the existing install/play
workflow and make every addition reproducible from inspected upstream bytes.

## Requirements

- **R1 — Exact selection.** Add one recipe per title, using ids `scythe-ii`,
  `plutonia-2`, `doom-the-way-id-did`, `arrival`, `vanguard`, `lunatic`,
  `no-end-in-sight`, `uac-ultra`, `japanese-community-project`, `stardate-20x6`.
  Do not duplicate or modify existing recipes. Verify author, release, base game,
  map slots, actual source-port requirements, permissions and credits from the
  canonical release/readme and archive contents. In particular Plutonia 2 requires
  Plutonia, not Doom II; the path-based tentative IWAD in old research is not evidence.
- **R2 — Reproducible recipe.** Follow CONTRIBUTING.md, current schema and pinned
  DSDA research. Compute package/member SHA-256 and byte counts from actual downloads;
  inspect every declared member's case-exact path, map/lump inventory and DEH/BEX
  requirements. Keep permission/credit text. Declare mandatory maps, soundtrack
  and patches in the author's intended order. List only verified byte-identical
  HTTPS mirrors already allowed by catalog/hosts.toml. No archive execution, IWAD
  acquisition, host-policy expansion, dependencies or runtime/UI changes.
- **R3 — Honest discovery metadata.** Use existing tags only with cited evidence,
  respecting their definitions (episode is at most nine maps, megawad at least
  thirty). Cite the actual Cacowards year and winner/runner-up role; preserve the
  distinction between inspected data and gameplay validation. Set inspected
  evidence and explicit Needs Deck Validation limitations, not verified gameplay.
- **R4 — Artwork.** Source one to three real in-game upstream images when available
  under existing D-024/CONTRIBUTING conventions. Commit only compliant JPEGs and
  complete SOURCES.toml evidence under the assigned title's directory. Do not open
  visible windows or capture the screen. If a title has no obtainable source,
  record that limitation and use the existing fallback; do not invent permission,
  synthesize artwork, or block an otherwise valid recipe on an inaccessible image.
- **R5 — Verification.** Add focused offline integration tests per worker freezing
  source identities, base game/complevel, required file/patch order and meaningful
  category memberships. Verify embedding without skipped recipes. Run offline and
  real-fetch lint on every new recipe. Full combined fmt/clippy/workspace tests
  run once at integration; workers iterate with focused core checks, jobs=2 and
  test threads=2. No network in tests.
- **R6 — Evidence and integration.** Each worker records upstream links, inspected
  release identity, archive/member hashes, compatibility reasoning, permission and
  tag evidence, screenshot provenance, commands/results and open Deck checks in its
  research/report. Commit small imperative-subject changes, merge main before final
  checks, and report worker_done. Workers never push/tag/release or merge into main.

## Parallel ownership

Four isolated Orca worktrees avoid simultaneous build.rs/embedded-catalog changes
and shared test edits. No nested delegation. Coordinator owns this spec and any
combined registration/count test fixes plus docs/status.md and docs/journal.yml.

| Worker | Titles | Owned evidence and test |
| --- | --- | --- |
| A | Scythe II; Plutonia 2 | docs/research/reference-087-a.md; docs/reports/087-a.md; crates/ddl-core/tests/catalog_087_a.rs |
| B | Doom the Way id Did; No End in Sight | docs/research/reference-087-b.md; docs/reports/087-b.md; crates/ddl-core/tests/catalog_087_b.rs |
| C | Arrival; Vanguard; Lunatic | docs/research/reference-087-c.md; docs/reports/087-c.md; crates/ddl-core/tests/catalog_087_c.rs |
| D | UAC Ultra; Japanese Community Project; Stardate 20X6 | docs/research/reference-087-d.md; docs/reports/087-d.md; crates/ddl-core/tests/catalog_087_d.rs |

Each worker also owns only its assigned catalog files and matching screenshot
directories. Existing research is a starting index, not a replacement for bytes.

## Acceptance criteria

- [ ] **A1 (R1–R3):** exactly 102 unique embedded recipes, all ten selected titles
  present with reviewed IWAD/complevel/tags and no invalid skips.
- [ ] **A2 (R2, R5):** offline and real-fetch lint pass for all ten; tests pin the
  inspected hashes, composition and compatibility, with no network access.
- [ ] **A3 (R4):** each new image passes existing size/format/provenance lint, or
  its unavailable source and fallback are recorded in the report.
- [ ] **A4 (R5):** merged result passes cargo fmt --all --check,
  cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace.
- [ ] **A5 (R6):** four evidence reports and a combined coordinator report state
  verification results and remaining physical Deck validation honestly.

## Out of scope

New application features, engine changes, new tag definitions, other PWADs,
new dependencies, automatic Steam Input changes, a new application release.

## Open questions

None. Routine archive/version/mirror choices are delegated to evidence-based review.

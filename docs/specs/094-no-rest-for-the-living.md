# 094 — Player-supplied No Rest for the Living

**Status:** Ready. **Origin:** owner requests the expansion and supplies
`/Users/vitor/Downloads/NERVE.WAD` for analysis, with identity references
https://doomwiki.org/wiki/No_Rest_for_the_Living and https://doomwiki.org/wiki/NERVE.WAD.

## Goal and requirements

- **R1 — Catalog entry.** Add `catalog/no-rest-for-the-living.toml`, requiring the
  player's Doom II IWAD and a required, default-on `user_supplied` main map
  component. No downloadable packages, dummy package, IWAD substitution or
  remotely acquired commercial data. The user imports their own `NERVE.WAD`.
  State obtain instructions plainly. Preserve its filename because DSDA uses
  `nerve.wad` (case-insensitive) for expansion mission/progression selection.
- **R2 — Evidence.** Inspect the supplied file as data, never execute or commit
  it. Coordinator inspection: 3,819,855 bytes, PWAD, 108 lumps, MAP01–MAP09,
  MD5 `967d5ae23daf45196212ae1b605da3b0`, SHA-256
  `2d234de8700ee403375fda278e50ea17739b36703b3316e0429b05aca4458fa9`.
  This matches the existing BFG/XBLA variant metadata in the project research
  and IWAD identification table. Independently confirm the exact fingerprint
  for the component's known hashes; never download a different commercial
  variant to obtain a hash. Cite pinned DSDA source for Doom II + `-file`
  filename-based mission behavior and the supported complevel. Record release,
  map count, permissions (local player use, no redistribution claim), earned tags
  and original-date precision with honest sources. Gameplay remains Needs Deck
  Validation. Sourced screenshots follow D-024, otherwise document fallback.
  The owner explicitly also requires the DOOM + DOOM II Steam/KEX variant:
  support importing that player's file over the already recognized Steam Doom II
  IWAD, preserving its filename. Existing reviewed metadata records 4,039,441
  bytes and MD5 `23422eb42833ac7b0dd59c0c7ae18a6f`. Do not invent its SHA-256 or
  claim local byte/gameplay inspection when only the supplied BFG file was read.
  Distinguish accepted imports and published identification evidence from actual
  inspected/known SHA-256 status in the recipe and report.
- **R3 — Import-only capability.** Allow a recipe with no packages only when its
  required main component is user-supplied and every non-user-supplied member
  still has a valid declared package reference. Retain all current validation
  for downloadable recipes and invalid/missing main components. Existing recipes,
  install manifests, user imports and profiles remain compatible. No fake
  download or skipped validation. Use the existing schema and import machinery
  where sufficient; justify a capability version gate together with spec 090.
- **R4 — Controller journey.** Installing the metadata-only entry must make no
  network request. Play stays blocked with a clear Import action until the
  required local file is provided. Its required component is importable through
  Setup using the controller, rather than a locked/unfocusable row. Importing,
  replacing or removing that file must retain required-main semantics and
  preserve the source's safe filename. Do not offer an off toggle that silently
  launches plain Doom II in its place. Use existing unknown-hash warning behavior
  honestly; do not label an unrecognized file as verified. No automatic scan or
  import of additional private files is requested.
- **R5 — Tests and report.** Add offline tests for recipe evidence, zero-network
  install, missing-file blocker, import and launch argv/order/filename, plus
  regression coverage that downloaded recipes still require valid packages.
  Drive the affected production import/navigation journey headlessly with
  synthetic fixture data (never commit commercial bytes). Offline lint applies;
  fetch lint is not applicable because this entry has no downloads. No new
  dependencies, engine changes, visible windows/screen captures, push/tag/release
  or nested agents. Record checks and physical Deck limitations.

## Parallel ownership

Extend current worker A Task: own the new recipe, matching screenshot directory,
`crates/ddl-core/tests/catalog_094.rs`, `docs/research/no-rest-for-the-living.md`
and `docs/reports/094-no-rest-for-the-living.md`. Finish the 10x10 tag independently.
A may prepare recipe/tests/evidence but must wait for D's capability integration
before its final lint/embedding checks. Do not edit shared runtime/model/lint.

Extend worker D's spec 090 Task with R3/R4 and focused core/app/headless tests;
it already owns the relevant model/lint/runtime code. Preserve B's vocabulary and
C's incoming Haste/Whales files. Coordinator owns shared id assertions, source
integration, final combined checks and status/journal reporting. D adds this
recipe's chronological field only after A's evidence is integrated.

## Acceptance

- [ ] Player-supplied entry loads, uses reviewed local identity evidence and
  never downloads or redistributes `NERVE.WAD` or any IWAD.
- [ ] Zero-network install, mandatory import and correct filename/argv are tested;
  controller import remains reachable, and existing downloaded recipes/imports
  retain their validation and behavior.
- [ ] Research/report, chronology and screenshot provenance/fallback are complete;
  combined format/workspace clippy/workspace tests pass after all integration.

The total is 118 recipes with all requested additions except conditional Linear
Doom, or 119 if that source/compatibility assessment supports a recipe. This
supersedes earlier cumulative totals without removing any prior requested title.

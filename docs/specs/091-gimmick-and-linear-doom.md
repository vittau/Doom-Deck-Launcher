# 091 — Gimmick category and Linear Doom assessment

**Status:** Ready. **Origin:** owner requests Linear Doom from ModDB and the
`gimmick` tag for both The 10x10 Project and Linear Doom.

## Requirements

- **R1 — Vocabulary and navigation.** Add `gimmick`, English label `Gimmick`,
  describing mapsets organized around an unusual explicit design constraint or
  playful central concept. Place its Discover category after Challenging and
  before Favorites, preserving the existing category order. Use vocabulary order
  75, keeping Featured and Beginner Friendly as metadata-only tags. Cover lint,
  navigation and the two requested assignments with meaningful offline tests.
- **R2 — The 10x10 Project.** Add the tag and source-backed evidence of the
  ten-texture/ten-flat constraint to its verification. Preserve the existing
  verified bytes, compatibility and release identity. No other retroactive tag
  assignments are requested.
- **R3 — Linear Doom.** Inspect the author's ModDB publication and actual v1.2
  archive without executing it. Confirm identity, permission, exact archive and
  member sizes/hashes, IWAD, map format/lumps and DSDA-Doom compatibility from
  primary evidence. Being tested with GZDoom alone does not prove either a
  requirement for GZDoom or compatibility with DSDA. Add `catalog/linear-doom.toml`
  only when the evidence supports the existing engine and a reproducible public
  download. Assign `gimmick` for its straight-line reinterpretation of Doom maps.
  Record original public release separately from the selected update for spec
  090, preserving honest date precision. Do not download or bundle an IWAD.
- **R4 — Download policy.** If required, narrowly add only observed exact ModDB
  page/download hosts to the reviewed allowlist, with documented redirect-chain
  evidence. Do not introduce wildcards, bypass anti-bot/authentication controls,
  expiring links as durable recipe URLs, arbitrary mirror trust or new downloader
  exceptions. If a correct recipe requires an unsupported policy or cannot be
  justified, report the concrete blocker and leave it out of the playable catalog.
- **R5 — Evidence and checks.** Keep product descriptions plain English, technical
  evidence in verification/research. Offline tests and catalog lint must pass; run
  fetch lint if a recipe is added. Label gameplay as Needs Deck Validation. Any
  missing sourced artwork uses the existing fallback with a stated reason.

## Parallel ownership

Reuse worker B for Linear Doom research/conditional recipe, `catalog/tags.toml`,
any justified `catalog/hosts.toml` change, `crates/ddl-core/tests/catalog_091.rs`,
`docs/research/linear-doom.md` and `docs/reports/091-gimmick-and-linear-doom.md`.
It must not change the shared exact-id/vocabulary assertions or runtime code.
Worker A adds the 10x10 tag/evidence and adjusts its own test after settlement of
its current review. Worker D (spec 090) adds the category to runtime ordering,
its focused navigation assertions and the relevant CONTRIBUTING tag guidance.
The coordinator updates shared vocabulary/id assertions and integrates all work.
Worker B tests vocabulary through its own standalone test, without changing A's
active recipe. No nested agents, push, tag, release or main merge.

## Acceptance

- [ ] `gimmick` validates and appears after Challenging in Discover when used.
- [ ] The 10x10 Project carries the tag, with its existing byte pins preserved.
- [ ] Linear Doom has a tested, reproducible DSDA-compatible recipe tagged
  `gimmick`, or a precise primary-evidence report explains why it cannot be added.
- [ ] Source/date/permission/host evidence and test results are recorded; final
  combined workspace checks pass after integration.

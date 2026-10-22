# 092 — Swim With The Whales

**Status:** Ready. **Origin:** owner explicitly requests
https://doomwiki.org/wiki/Swim_With_The_Whales.

## Requirements

- Add `catalog/swim-with-the-whales.toml`, one recipe for Ribbiks's release.
  Apply spec 087 R2–R6: inspect actual author/readme/archive bytes, pin package
  and member SHA-256/sizes/case-exact paths, required composition, kept credits,
  correct user-provided IWAD, DSDA-supported port family/complevel and precise
  inspected status. No archive execution, IWAD acquisition, new engine or hosts.
- Initial source lead is `/idgames/levels/doom2/Ports/s-u/swtw.zip`; the existing
  Cacowards inventory links the 2013 Top Ten page 3. These are leads, not verified
  compatibility, permission, map-count or download-byte facts. The coordinator's
  DoomWiki fetch returned 403; do not invent its contents.
- Use only tags with evidence and respect their map-count definitions. The
  owner's `gimmick` request covered The 10x10 Project and Linear Doom only; do
  not infer further assignments. Keep product descriptions about the experience,
  theme and campaign rather than engine/lump/load-order implementation details.
- Record source-backed original first-public date and its honest precision,
  separately from the selected archive's update identity. Worker D owns the
  optional chronology field for spec 090 after integration; provide evidence
  rather than adding a competing date schema.
- Add real sourced screenshot(s) with provenance if obtainable under D-024;
  otherwise document the fallback. Never open visible windows or capture the
  screen. Actual gameplay/performance remains Needs Deck Validation.
- Add focused offline tests and run offline/fetch lint; shared workspace checks
  run once at final integration. No new dependencies, push, tag, release or nested
  agents. Preserve all earlier settled work.

## Ownership and verification

Reuse idle OpenCode worker C in `pwads-087-c`. Own only the new recipe, its matching
screenshot directory, `crates/ddl-core/tests/catalog_092.rs`,
`docs/research/swim-with-the-whales.md` and `docs/reports/092-swim-with-the-whales.md`.
Coordinator owns shared id registration and combined reporting. D owns chronology
and navigation; B owns Linear Doom/vocabulary/hosts; A owns the 10x10 tag.
Merge main before final checks; worktree may merge `verify/087-pwads` for current
integration context. Use shared target, jobs=2 and test threads=2.

## Acceptance

- [ ] One reproducible, lint-clean recipe has inspected source/permission/byte/
  compatibility evidence and an original-release date decision.
- [ ] Offline tests cover source identities, composition, base game/complevel,
  tag claims and embedding without skipped assigned recipes; fetch lint passes.
- [ ] Artwork passes provenance checks or fallback is documented; report states
  exact checks and remaining physical Deck validation.
- [ ] Final catalog contains 116 recipes (117 if Linear Doom is also supported),
  all loaded with no skips; combined fmt/clippy/workspace checks pass.

# 088 — Five additional curated PWADs

**Status:** Done — 2026-10-22. All five extra titles integrated (catalog reached 107); combined checks green. **Origin:** owner explicitly approved the five candidates named
while spec 087's four OpenCode workers were running.

## Goal

Include Doom II the Way id Did, Deathless, Overboard, Ozonia and TNT: Revilution
alongside spec 087's ten additions: 107 total curated recipes after integration.
The owner's "include these too" refers to those five named candidates; no other
research candidate is authorized for implementation by this spec.

## Requirements

- **R1:** Add exactly one recipe each with ids `doom-ii-the-way-id-did`,
  `deathless`, `overboard`, `ozonia` and `tnt-revilution`. Follow spec 087 R2–R6
  unchanged: actual upstream package/member hashes, case-exact files, credits,
  mandatory load order and patches, supported hosts, honest inspected status,
  evidence-based tags, offline tests and real-fetch lint. No new dependencies,
  host-policy changes, runtime/UI changes or archive execution.
- **R2:** Verify each release rather than inheriting metadata from a port's
  repackaging. The original readmes are the initial evidence: D2TWID declares
  32 Doom II maps; Deathless v1.1 declares Ultimate Doom E1M1–E4M9 and a
  limit-removing engine; Overboard declares Boom/complevel 9, six main maps plus
  New Game Plus and 15 total slots; Ozonia declares 32 Doom II maps/complevel 9;
  TNT: Revilution declares TNT, 32 maps and vanilla-engine compatibility.
- **R3:** Artwork remains best effort under current contribution rules. Never
  open visible windows or capture the screen. Record inaccessible sources and
  retain existing fallback artwork where needed.
- **R4:** Extend each worker's existing unique integration test, research and
  report to include its extra title(s). Coordinator owns shared expected-id
  coverage and combined fmt/clippy/workspace verification. Full integrated
  catalog must have 107 unique recipes and no skips.

## Parallel ownership extension

Keep the four existing OpenCode Dispatches; this extends their task scope and
does not start duplicate editors or exceed four workers. Merge main to receive
this spec before final checks. No nested delegation, push, tag or release.

| Existing worker | Additional titles and owned files | Evidence/test extension |
| --- | --- | --- |
| A | catalog/tnt-revilution.toml and matching screenshot directory | reference-087-a.md, reports/087-a.md, catalog_087_a.rs |
| B | catalog/doom-ii-the-way-id-did.toml; catalog/deathless.toml and matching screenshot directories | reference-087-b.md, reports/087-b.md, catalog_087_b.rs |
| C | catalog/overboard.toml and matching screenshot directory | reference-087-c.md, reports/087-c.md, catalog_087_c.rs |
| D | catalog/ozonia.toml and matching screenshot directory | reference-087-d.md, reports/087-d.md, catalog_087_d.rs |

## Initial primary sources

- [D2TWID readme](https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/megawads/d2twid.txt)
- [Deathless readme](https://ftp.fu-berlin.de/pc/games/idgames/levels/doom/Ports/megawads/deathless.txt)
- [Overboard readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/overboard.txt)
- [Ozonia readme](https://www.gamers.org/pub/idgames/levels/doom2/Ports/megawads/ozonia.txt)
- [TNT: Revilution readme](https://ftp.fu-berlin.de/pc/games/idgames/levels/doom2/megawads/tntr.txt)

These are source leads, not evidence that this coordinator verified the archives.

## Acceptance criteria

- [x] **A1 (R1–R2):** all five extra recipes pass offline and real-fetch lint;
  archive/member identities, IWAD/complevel and file order are covered by tests.
- [x] **A2 (R3):** sourced artwork passes format/provenance lint, or absent source
  and fallback are explicitly documented.
- [x] **A3 (R4):** all 107 recipes embed without skips; combined format, workspace
  clippy and workspace tests pass.
- [x] **A4:** four expanded worker reports and coordinator report distinguish
  confirmed inspection from gameplay/audio/progression Needs Deck Validation.

## Open questions

None. Research candidates outside these five remain proposals only.

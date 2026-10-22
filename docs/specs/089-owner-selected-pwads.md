# 089 — Further owner-selected PWADs

**Status:** Ready. **Origin:** owner explicitly requested Going Down: Turbo,
Headless Chicken, Abandon, Sunder, The 10x10 Project and Micro Slaughter Community
Project and Speed of Doom while the four OpenCode workers were running.

## Goal

Add the seven named releases alongside specs 087 and 088: twenty-two new recipes,
114 total curated PWADs. Going Down: Turbo is its own recipe and must preserve
the original Going Down entry, package identity and installed profiles.

## Requirements

- **R1:** Add `going-down-turbo`, `headless-chicken`, `abandon`, `sunder`,
  `the-10x10-project`, `micro-slaughter-community-project` and `speed-of-doom`, one
  self-contained recipe per exact requested release. Use the user-provided pages
  below to establish identity, then inspect canonical release/readme/archive
  bytes. Record a specific release/version/date and package/member SHA-256;
  never silently follow an unpinned evolving archive or substitute a similarly
  named fork/mod. For Sunder, inspect the actual selected public release and
  clearly describe unfinished/released-map scope if applicable.
- **R2:** Apply spec 087 R2–R6 unchanged: correct IWAD and author-supported
  complevel, mandatory map/music/patch composition, kept credits and permissions,
  verified byte-identical mirrors on existing hosts, inspected evidence and
  offline tests plus real-fetch lint. Do not infer compatibility from screenshots
  or another port's repackaging. No archive execution, IWAD acquisition, new
  dependencies, runtime/UI/engine behavior or broadening host policy.
- **R3:** Record author-documented limitations and inspect map scope/complexity.
  In particular, make Abandon/Sunder performance and gameplay Needs Deck Validation
  explicit; any qualitative map-complexity description must cite inspection or
  the author. Do not invent Steam Deck FPS, battery usage or tested performance.
  Do not change graphics defaults or impose arbitrary launch flags.
- **R4:** Use only evidence-supported existing tags. Match map counts to the
  project's definitions rather than the title's conventional "megawad" label.
  Artwork is best effort under CONTRIBUTING.md; never open visible windows or
  capture the screen, and document unavailable sources/fallback honestly.
- **R5:** Extend the existing worker-specific integration tests, research and
  reports. Coordinator owns global expected-id registration, combined offline
  lint, fmt/clippy/workspace checks and final 114-entry embedding verification.

## Parallel ownership extension

Extend the same four active OpenCode Dispatches without duplicate editors,
nested agents or exceeding four workers. This spec may initially be read from
the coordinator's absolute checkout path; the coordinator integrates the durable
spec commit together with the worker branches. Merge local main before final checks.

| Existing worker | Additional recipe and screenshot ownership | Existing evidence/test |
| --- | --- | --- |
| A | catalog/going-down-turbo.toml; catalog/the-10x10-project.toml; catalog/speed-of-doom.toml and matching screenshot directories | reference-087-a.md, reports/087-a.md, catalog_087_a.rs |
| B | catalog/sunder.toml and matching screenshot directory | reference-087-b.md, reports/087-b.md, catalog_087_b.rs |
| C | catalog/headless-chicken.toml; catalog/micro-slaughter-community-project.toml and matching screenshot directories | reference-087-c.md, reports/087-c.md, catalog_087_c.rs |
| D | catalog/abandon.toml and matching screenshot directory | reference-087-d.md, reports/087-d.md, catalog_087_d.rs |

## Source leads

- [Going Down: Turbo](https://doomwiki.org/wiki/Going_Down:_Turbo);
  [DSDA identity/download page](https://dsdarchive.com/wads/gdturbo).
- [Headless Chicken](https://doomwiki.org/wiki/Headless_Chicken);
  [DSDA identity/download page](https://dsdarchive.com/wads/hdlschkn).
- [Abandon](https://doomwiki.org/wiki/Abandon); existing Cacowards research
  confirms the Boom release at `levels/doom2/Ports/megawads/abandon.zip` and
  links its 2020 award source. Verify actual selected bytes and release.
- [Sunder](https://doomwiki.org/wiki/Sunder);
  [DSDA identity/download page](https://dsdarchive.com/wads/sunder).
- [The 10x10 Project](https://doomwiki.org/wiki/The_10x10_Project);
  [DSDA identity/download page](https://dsdarchive.com/wads/10x10).
- [Micro Slaughter Community Project](https://doomwiki.org/wiki/Micro_Slaughter_Community_Project).
- [Speed of Doom](https://doomwiki.org/wiki/Speed_of_Doom); existing Cacowards
  research points to `levels/doom2/Ports/megawads/sodfinal.zip`. Author readme
  declares Boom-compatible ports, Doom II and 33 challenging maps; inspect bytes.

DoomWiki returned HTTP 403 to the coordinator. Do not invent its contents;
use the actual author's release text and accessible archive mirrors. Source leads
are not claims that this coordinator has inspected these seven archives.

## Acceptance criteria

- [ ] **A1 (R1–R2):** all seven exact releases have passing offline and real-fetch
  lint and offline tests for their byte identities, composition and IWAD/complevel.
- [ ] **A2 (R3–R4):** scope, tags, permissions, artwork and limits have sources;
  no unmeasured Deck-performance claims or gameplay verification badges.
- [ ] **A3 (R5):** 114 unique recipes embed without skips; combined fmt, workspace
  clippy and workspace tests pass. Original Going Down is preserved.
- [ ] **A4:** all four expanded worker reports and coordinator report cover
  these additions and remaining physical Deck validation honestly.

## Open questions

None. Choose and pin an actual public compatible release from the original
author/archive evidence; raise an issue to the coordinator if host, size or engine
constraints prevent inclusion rather than changing application policy silently.

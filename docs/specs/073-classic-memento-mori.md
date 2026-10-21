# 073 — Add Memento Mori classics

**Status:** Complete — published in v0.0.11-polish. **Origin:** owner requests Memento Mori, Memento Mori 2 and Scythe under Classics.

## Requirements

- R1: Add one reviewed catalog recipe each for Memento Mori and Memento Mori II. Use the
  release's actual author/readme and archive contents to verify title, authors, base game,
  map slots, components/music/load order, compatibility and redistribution terms. Verify exact
  archive/member sizes and SHA-256 from upstream bytes. Cite upstream sources and distinguish
  confirmed metadata from gameplay that still needs Deck validation. Never invent facts.
- R2: Both new recipes appear under the existing Classics category. Scythe already has a recipe:
  verify its Classics membership and add the tag if missing; never create a duplicate recipe.
  Preserve all other existing Scythe behavior and identities.
- R3: Follow current catalog conventions, bounded download/host policy and recipe validation.
  No IWADs, executable archives, paid payloads, bundled mapsets or unreviewed new dependencies.
  If a host/redirect addition is needed, keep it narrow and justify from actual fetch evidence.
  No new screenshots unless verified licensed upstream map images are already available; no
  generated artwork is required by this task.
- R4: Offline tests freeze both recipes' source identities/composition and category reachability;
  use synthetic fixtures where needed, no test network access. Separately run appropriate
  real-fetch/release lint and record results, blocks and any unplayed compatibility limits.

- R5 (owner expansion): add a specific Doomer Boards category with verified DBP recipe
  memberships from existing research, retaining their general Community Projects membership.
  Set vocabulary order to Cacowards, Megawads, Community Projects, Doomer Boards, Classics,
  Episodes, Challenging. Preserve Featured/Beginner Friendly metadata vocabulary but place
  outside that requested sequence; 072 hides them from the strip. Add offline tests for exact
  order, DBP coverage and excluding non-DBP projects from the new category.

## Ownership and verification

One catalog worker owns catalog/memento-mori*.toml, Scythe tags only if necessary, catalog/tags.toml,
verified DBP recipe tag additions, corresponding research/report and focused catalog tests. No UI/runtime/audio/config/workflow or coordinator
changes. 071/072 worker is changing header/Discover navigation and producing screenshots; it
owns UI and docs/screenshots. Coordinate recipe-list/count tests at integration; do not edit its
files. Merge main before final checks. Jobs2/threads2, focused core tests/fmt/clippy, no full Slint
suite. Never open visible windows or capture the screen. No subagents/push/tag/release/mainmerge.
Report docs/reports/073-classic-memento-mori.md with exact source/hashes/permissions/checks and
remaining physical Deck validation. Commit small changes and worker_done.

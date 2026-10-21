# 066 — Add Antaresian Reliquary to the curated catalog

**Status:** Ready · **Origin:** owner, 2026-10-21:
`https://doomwiki.org/wiki/Antaresian_Reliquary`.

## Requirements

- **R1:** Add one complete `catalog/antaresian-reliquary.toml` recipe. Confirm the release,
  author, IWAD, engine compatibility, complevel alias, map slots and all launch dependencies
  from the bundled author's TXT, inspected archive/WAD and pinned DSDA source as necessary.
  Existing Cacowards inventory points to `levels/doom2/Ports/a-c/anta_req.zip`; treat it as a
  lead to verify, not a substitute for checking the downloaded bytes.
- **R2:** Prefer canonical /idgames downloads from already allowlisted mirrors. Compute archive
  and member SHA-256 and sizes from actual bytes; list only byte-identical mirrors. Preserve
  bundled permission/credits text. Inspect DEHACKED/COMPLVL/UMAPINFO or separate patches so the
  recipe loads the correct files exactly once and in the right order. Never execute downloads.
- **R3:** Include representative real in-game screenshot(s) with source/credit/provenance under
  the existing image policy and lint limits. Use only vocabulary tags supported by evidence.
  Do not claim a gameplay/Deck test based on archive inspection or demo listings.
- **R4:** Document findings and links in `docs/research/reference-antaresian-reliquary.md` with
  Confirmed / Proposed / Needs Deck Validation labels. DoomWiki/Doomworld may block automation;
  label search/Wayback evidence appropriately and prefer the author's bundled text for facts.

## Ownership and acceptance

Own the new recipe, research file, `crates/ddl/ui/assets/screenshots/antaresian-reliquary/**`,
necessary catalog-membership/count test expectations, and
`docs/reports/066-antaresian-reliquary.md`. No new hosts/dependencies or application behavior
changes without coordinator review. Do not edit coordinator-owned docs. The concurrent
button-actions worker owns base-game screenshot/UI/config/launch changes; Flatpak worker owns
packaging/workflows. Keep their files untouched; resolve catalog test-list conflicts as a union.

Acceptance: offline lint, real fetch lint for the new recipe (including hash/member validation),
targeted catalog tests, then final fmt/clippy/workspace tests after merging main. Report exact
commands/results and any gaps. Coordinator runs Linux e2e after integration. Commit small
imperative changes; never push/tag/release. Heartbeat per milestone and report via worker_done.

Never open visible windows or capture the screen — headless renders only. Use fetched source
images or off-screen methods, never run the engine visibly to take a screenshot.

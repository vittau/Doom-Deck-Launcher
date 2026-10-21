# 072 — Discover uses specific categories

**Status:** Ready. **Origin:** owner: remove All, then also Featured, from Discover before
refreshing the screenshots.

## Requirements

- R1: Remove All and Featured from the Discover category strip. The owner specifies this order:
  Cacowards, Megawads, Community Projects, Doomer Boards, Classics, Episodes, Challenging;
  retain Favorites after these as the personal filter. Omit other categories from this strip;
  preserve catalog metadata and recipe compatibility/downloads. The 073 catalog worker owns
  the new Doomer Boards vocabulary/memberships and vocabulary ordering.
- R2: New Discover sessions select the first available specific category. If a selected category
  disappears after a snapshot/catalog refresh, fall back to the first available remaining
  category. Handle an empty catalog safely (Favorites is still available); no empty-vector
  indexing, modulo by zero or stale focus.
- R3: Preserve D-pad, category-trigger wrap/hold-repeat, focus restoration and favorite sorting.
  The empty-Favorites action must lead to an available specific category rather than removed
  All; use accurate player-facing copy. No unrelated layout or input changes.
- R4: Every currently embedded mapset remains reachable through at least one specific category.
  Test current recipe coverage, omitted strip labels, initial/fallback selection, wrapping,
  favorites-empty recovery and focused controller journeys. Use synthetic/offline state.
- R5: Complete this together with the header branding change before regenerating the five
  website/README images. Update the existing screenshot producer/reference if it names the
  removed All view; captures must show current actual UI, with honest sample-state credits.

## Ownership and verification

Same bounded UI worker as 071 may own runtime category selection/rendering, category-related
strings, focused category/headless tests and current screenshot assets. No catalog/schema,
audio/config/download/launch behavior, workflow or coordinator-document changes. Read AGENTS,
071 and this task before implementing. Never open visible windows or capture the screen.
Focused tests/headless producers with jobs2/threads2, final appropriate fmt/clippy. Do not rerun
all unrelated headless journeys. Report docs/reports/072-discover-specific-categories.md (may
reference the 071 report and shared render paths). Commit and worker_done; no push/tag/release,
subagents or direct main merge. Coordinator publishes website assets and subsequent app build.

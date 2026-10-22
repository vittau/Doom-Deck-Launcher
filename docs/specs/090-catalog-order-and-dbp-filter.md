# 090 — Release order and a focused Community Projects tab

**Status:** Ready. **Origin:** owner requests that Doomer Boards entries stop
appearing in Community Projects and that everything be ordered by ascending release.

## Goal

PWAD browsing follows release chronology, oldest first. Community Projects no
longer duplicates the large Doomer Boards collection; those recipes remain in
their dedicated category and any other earned categories.

## Requirements

- **R1 — Community Projects view.** Exclude every recipe tagged `doomer-boards`
  from the Community Projects tab. Preserve that dedicated category, other tags,
  favorites, installations and profiles. The taxonomy may retain the technically
  accurate community-projects tag; the owner requested what appears in the tab.
  Add tests covering all existing DBPs and proving ordinary community projects
  remain visible.
- **R2 — Chronological browsing.** Sort PWAD lists in Discover (every category
  and Favorites) and Library by original public release, ascending. Favorite
  status must not move a newer release before an older one. Equal dates use a
  deterministic title/id tie-breaker. Unknown dates sort last, deterministically.
  Continue/recent-play history keeps its own last-played semantics; base-game
  and setup-control sections are not PWAD browsing lists.
- **R3 — Explicit metadata.** Use an explicit validated optional original-release
  date field rather than extracting chronology from the free-form version label.
  Keep the current `release` identity/version text intact. Source dates from the
  release/readme and existing cited research; distinguish first public release
  from later ZIP/repackaging/update dates. Support only the precision actually
  evidenced (year, month or day); never invent January 1 or a first-of-month day.
  State and test the ordering for partial precision. Record a date or an honest
  unknown for every shipped recipe, including the owner-expanded additions.
- **R4 — Compatibility.** Existing manifests, cached catalogs, fixtures and local
  content missing the optional field must still load. Validate present values.
  Any remote-catalog minimum-client or application-version adjustment genuinely
  required by the new recipe capability must be coherent: older apps must not
  silently receive a bundle that causes their recipes to be dropped. D-038
  permits raising the minimum only for an actual capability change. Do not tag,
  publish or release anything. No new dependency unless genuinely necessary and
  justified; prefer the existing date-validation utilities/small typed logic.
- **R5 — Navigation and verification.** Sorting/filtering must preserve focus and
  selected mapset identity as snapshots/categories change; no arbitrary card
  moves due solely to favorite status. Add meaningful offline tests for ascending
  dates (including version labels that would sort incorrectly), equal/unknown/
  partial dates, favorite independence, DBP exclusion, Library, legacy recipes
  and present-field validation. Drive at least the affected production journey
  headlessly. Never open visible windows or capture the screen.
- **R6 — Evidence.** Report the metadata source and precision decisions, affected
  views, legacy/bundle handling, checks and remaining Deck navigation validation.
  Application UI copy stays plain English; technical evidence stays in research.

## Parallel ownership and integration

Reuse the existing idle OpenCode worker D in `pwads-087-d` for this new Task.
The three other agents continue catalog additions; no extra worker slot is used.
This worker owns the release-date model/lint/serialization and ordering/filtering
logic, the corresponding core/app/headless tests, CONTRIBUTING.md changes needed
to author the field, and justified package/bundle version compatibility changes.
Use `crates/ddl-core/tests/catalog_090.rs` for standalone new catalog tests.
The coordinator owns the shared exact shipped-id registration and status/journal/
decisions/overview; do not rewrite that id list.

For recipe files, this worker may add ONLY the chronological metadata field and
its cited evidence, preserving package/member hashes, components, descriptions
and other workers' researched facts. Start with the 92 original recipes plus its
five already settled additions; other agents still own their not-yet-merged files.
Stage source/date decisions while they work. The coordinator will signal when all
new catalog branches are merged on `verify/087-pwads`; then commit own work and
merge that integration branch before filling the remaining dates and final checks.
Never edit another active worktree. All test fixtures stay offline.

Iterate with focused tests, jobs=2 and test threads=2; the coordinator runs final
combined fmt/workspace clippy/workspace tests after the full integration. Commit
small imperative-subject changes. No nested agents, push, tag, release or main merge.
Report `docs/reports/090-catalog-order-and-dbp-filter.md`, with chronology evidence
in `docs/research/catalog-release-dates.md`. Check inbox and heartbeat by milestone.

## Acceptance criteria

- [x] **A1 (R1):** Community Projects contains no DBP; ordinary community recipes
  and the complete Doomer Boards category remain reachable.
- [x] **A2 (R2–R3):** all shipped PWAD browsing lists use the same evidence-based
  ascending chronology, with tested ties, partial/unknown dates and favorites.
- [x] **A3 (R4):** old recipe/manifests/cache/fixtures load; malformed present dates
  fail validation; bundle/client capability gating is coherent.
- [x] **A4 (R5):** focused core/app/headless checks pass and the final combined
  format, workspace clippy and workspace tests pass after integration.
- [x] **A5 (R6):** source/precision metadata and verification report are complete;
  any unresolved original date is explicitly unknown rather than fabricated.

## Open questions

None. Make routine schema/order/source choices within these requirements and
record them; escalate only an actual conflict with the owner's requested behavior.

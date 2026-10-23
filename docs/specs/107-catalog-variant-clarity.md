# 107 — Explain recipe variants without implying a complevel override

**Status:** Ready · **Depends on:** 010, 026, 106
**Origin:** owner, 2026-10-23: the setup's apparent complevel/approval control is confusing;
what approval is required, and how can the player change the complevel? Clicking it seems
to do nothing. Owner correction: an unavailable button should stay visible and greyed out;
active-looking controls that do nothing are poor UX.

## Confirmed current behavior

Catalog setup's Advanced row toggles visibility of components with `replaces.is_some()` and
`!user_supplied`. It does not provide free-form or numeric complevel override. Choosing a
replacement component uses the existing player confirmation sheet; no external approval is
involved. Custom setup detail has a separate Compatibility selector (Auto/numeric).

## Requirements

- **R1:** keep the catalog variant row visible but **greyed out, non-actionable and skipped
  by controller focus** when the effective recipe has none of those hidden replacement
  components. The owner explicitly prefers a disabled control over removing it. Explain
  **No recipe variants available** in its help. Use the same availability condition as the
  component loop, not a guessed tag/title list. Preserve installed-snapshot recipe behavior.
  No active-looking control may accept clicks and silently do nothing.
- **R2:** replace the vague Advanced/approval presentation with **Show recipe variants** /
  **Hide recipe variants** and concise help explaining alternatives supplied by the recipe
  and the player's confirmation on selection. Never imply a generic complevel selector,
  outside approval or a permissions process. New text stays translatable, fits the current
  style/readable size, and must not reintroduce the known bottom-edge subtitle collision.
- **R3:** make the catalog's displayed complevel clearly recipe-controlled, e.g. a concise
  Set by recipe note beside/below the existing value. Custom setup Compatibility remains
  available and unchanged. No new catalog complevel override, recipe changes, engine behavior,
  component selection rules or public download changes.
- **R4:** update guidance explaining that catalog variants may include their own compatibility
  settings; direct complevel selection belongs to custom setups. Do not recommend an override
  that does not exist or promise an arbitrary mod's compatibility.

## Acceptance and scope

- Focused headless tests: a no-variant catalog setup has a visibly disabled row without an actionable/focus node; a variant recipe
  has the correctly labeled toggle, expands/collapses and still asks the player before replacing
  content. Cancel preserves choices. Focus skips disabled rows and remains valid when variants expand/collapse; existing
  skill/pistol/load-order navigation still works. The complevel is visibly recipe-controlled;
  custom setup numeric/Auto selection retains its regression coverage.
- Off-screen production frames for no-variant, collapsed and expanded variant states; no
  visible window or desktop capture. Coordinator reviews text, clipping and focus.
- Focused fmt/Clippy/tests appropriate to the touched presentation and navigation; reuse
  spec106's authoritative ordinary workspace suite when unchanged behavior is covered. Do not
  replay that suite merely for this follow-up. No network e2e, SDK/package build or release.
- Update the guide, this spec and `docs/reports/107-catalog-variant-clarity.md` with precise
  evidence and any remaining physical checks. No dependency needed.

Ownership: one UI worker, reused after spec106 settles, owns runtime/copy/strings/headless
navigation tests and guide for these requirements. Coordinator owns protected status/journal.
No other editor overlaps. Merge main before final checks. Commit small imperative subjects.
Never push, tag, release, use Free models, open visible windows or capture the screen.

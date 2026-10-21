# 071 — Match the application header to the website brand

**Status:** Complete — published in v0.0.11-polish. **Origin:** owner: use the website logo arrangement inside DDL too.

## Requirements

- R1: In the application header, use the existing approved stained-glass emblem on the left,
  with outlined DDL lettering above a smaller DoomDeckLauncher subtitle on the right. Match
  the compact website layout at main 0518500: both text lines share a left edge, their gap is
  about 2 px, and the combined text block is vertically centered against the emblem.
- R2: Reuse approved SVG artwork and existing local fonts. No new art, redesign, dependency,
  navigation target or input behavior. Keep the 1280x800 header clear of tabs; do not move tabs
  or reduce their targets to make room. Preserve icon, loading/About branding and other screens.
- R3: Verify off-screen with Slint software rendering, inspect the header PNG, and cover visible
  bounds/spacing and controller tab navigation with meaningful focused tests. Never open a
  visible window or capture the owner's screen.
- R4: Refresh the five shared website/README screenshot assets from current headless UI after
  this header change, using the existing lossless script. Keep sample-state provenance honest;
  no screenshot styling changes. Run only the focused producing tests, not all headless journeys.
  The existing website renderer hashes image URLs, so publication needs no new cache mechanism.

## Ownership and verification

One worker owns the header in crates/ddl/ui/app-window.slint, a small reusable branding component
or derived vector asset only if needed, focused headless assertions, docs/screenshots and the
screenshot regeneration script only if necessary. No runtime/services/audio/catalog/workflow
changes or coordinator-owned docs. Existing accepted header reference: /tmp/ddl-brand-1280.png
and /tmp/ddl-brand-390.png (website). Read AGENTS and relevant design decisions. Use jobs2/threads2;
focused tests and appropriate fmt/clippy, no repeat of the full unchanged 870-test suite. Merge
main before final checks. Report docs/reports/071-app-header-brand.md with PNG/source/checks and
remaining physical Deck limitation. No subagents, push, tag, release or direct main merge.

# 065 — Launcher-themed project and Flatpak download page

**Status:** Complete — published 2026-10-21; physical Deck installation remains unvalidated · **Depends on:** 063 and a working signed remote
**Origin:** owner, 2026-10-21: page at `https://www.vitormach.dev/Doom-Deck-Launcher/`, similar to
README but with a more polished layout matching the launcher; include a Deck install link.

## Requirements

Mission: a Steam DOOM owner should quickly understand the value, find the right download,
install on Deck and reach their first modded game with minimal friction. Lead with the player
outcome and one clear install action; put repository/build details in secondary links. Describe
necessary setup honestly and do not advertise unmeasured performance claims.

- **R1:** Replace the provisional installation landing page with a responsive static project
  page at the approved URL, using README's accurate product/install content and existing
  Ember Cathedral identity, logo, screenshots, typography and license/credit conventions.
  Work from existing repository assets; no unrelated hosted-site platform or new domain.
- **R2:** A prominent Install on Steam Deck link targets the signed
  `io.github.vittau.Doom-Deck-Launcher.flatpakref` at the same site. Explain the expected
  Discover/open-file install flow, with a manual command fallback; validate the actual Deck
  interaction before claiming it works. A desktop entry is not the download/install artifact.
- **R3:** Explain the self-hosted experimental channel, updates, legal IWAD ownership and manual
  Add to Steam/Compatibility-tool-none setup. Link the tarball alternative, setup guide,
  source repository and license. Never imply Flathub listing, affiliation or verification.
- **R4:** Preserve signed remote URLs/key handling and publication workflow behavior. Render
  the actual version rather than inventing a release. Support desktop/mobile/Deck-sized
  viewports with readable contrast, visible keyboard focus and accessible link/button labels.
- **R5:** Make the page discoverable and shareable with a descriptive document title, concise
  search description, canonical approved URL and social preview using existing licensed assets.
  Explain that it helps Steam DOOM owners play community modifications on Deck; do not imply
  official Valve/id affiliation. The primary install path should use the graphical handler for
  `.flatpakref`, with CLI details secondary and actual Deck validation clearly distinguished.

- **R6 (owner's direct worker request, 2026-10-21):** include a visible, responsive support link
  to `https://buymeacoffee.com/vittau` in a secondary section before the footer. Keep install
  as the primary action. Use a local inline icon and neutral support copy; no external widget,
  script, tracking or invented claims about how contributions are spent.

## Acceptance

Validate static HTML and links, inspect headless renders at desktop and narrow widths and
verify the download link resolves to the intended signed ref. Review the design with the owner
before merging a large visual change. Report at `docs/reports/065-project-download-page.md`.

The first signed remote build succeeded and the site is served (2026-10-21); design may now
proceed independently of the button-actions implementation. Keep its application files untouched.
Publication still depends on reviewed, passing deployment changes from 063 and a headless design
review with the owner. The provisional page remains functional until integration.
Never open visible windows or capture the screen — headless renders only.

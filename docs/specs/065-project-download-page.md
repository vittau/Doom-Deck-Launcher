# 065 — Launcher-themed project and Flatpak download page

**Status:** Planned · **Depends on:** 063 and a working signed remote
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

## Acceptance

Validate static HTML and links, inspect headless renders at desktop and narrow widths and
verify the download link resolves to the intended signed ref. Review the design with the owner
before merging a large visual change. Report at `docs/reports/065-project-download-page.md`.

This is later work: stabilize current Flatpak CI and button-actions first. The provisional page
remains functional until this task is dispatched. Never open visible windows or capture screen.

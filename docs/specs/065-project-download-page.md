# 065 — Launcher-themed project and Flatpak download page

> **Current endpoint:** [doomdecklauncher.com](https://doomdecklauncher.com/). Spec 079 / D-039
> supersede the original domain, URL layout and Flatpak ID recorded below. Original scope is
> retained as historical context.

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

- **R6 (owner's latest revision, 2026-10-21):** include a compact support button in the
  header using the owner-supplied official Buy Me a Coffee embed: slug `vittau`, text `Donate`,
  Cookie font, yellow `#FD0`, black text/outline and white coffee. Retain the official logo and
  font, overriding only dimensions to fit the header. This explicitly permits the official
  external button script and its Google Fonts stylesheet, superseding the earlier local-only
  constraint. Include a no-JavaScript donation link. Version the local stylesheet by content
  to avoid stale browser styling. Keep install as the primary page action.

- **R7 (owner, 2026-10-21):** Publish page-only edits independently of the Flatpak build.
  Automatically deploy relevant main-branch page/assets changes, with a manual trigger too.
  Preserve the published app version/date, signed repository, descriptors and key. Serialize
  both writers to the same Pages branch; retain deployment verification and prefix cache purge.

## Acceptance

Validate static HTML and links, inspect headless renders at desktop and narrow widths and
verify the download link resolves to the intended signed ref. Review the design with the owner
before merging a large visual change. Report at `docs/reports/065-project-download-page.md`.

The first signed remote build succeeded and the site is served (2026-10-21); design may now
proceed independently of the button-actions implementation. Keep its application files untouched.
Publication still depends on reviewed, passing deployment changes from 063 and a headless design
review with the owner. The provisional page remains functional until integration.
Never open visible windows or capture the screen — headless renders only.

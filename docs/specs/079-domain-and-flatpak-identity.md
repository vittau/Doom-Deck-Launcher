# 079 — Adopt doomdecklauncher.com and its Flatpak identity

**Status:** Complete — published in v0.0.11-polish. **Origin:** owner bought doomdecklauncher.com and authorized changing the
Flatpak ID; only the owner uses the current package and no migration is required.

## Requirements

- R1: Use `https://doomdecklauncher.com/` as the canonical public website and installation
  endpoint everywhere current configuration and user-facing documentation calls for one.
  Preserve GitHub repository/source, release-download and catalog transport URLs where they
  actually identify those services. Do not rewrite historical journal entries/reports as if
  the domain had always existed.
- R2: Change the Flatpak/application desktop identity consistently to
  `com.doomdecklauncher.DoomDeckLauncher`: manifest and metadata filenames, manifest app-id,
  exported desktop/icon/metadata names, sandbox expectations, build/publication workflows,
  descriptor/repository refs, tests and current setup instructions. Keep binary `ddl` and the
  product name. No old-ID alias, migration or old-domain redirect is required for this change;
  do not remove unrelated state-schema/release-fixture validation or existing legacy-import
  features merely because the owner waived migration of this one installation.
- R3: Both independent website deployment and full signed Flatpak publication must create or
  preserve a root `CNAME` containing exactly `doomdecklauncher.com`, so a later workflow cannot
  erase the custom-domain setting. Keep the shared publication lock, signature verification
  and separation between page-only updates and Flatpak builds.
- R4: Website assets/links must work at the new domain root, without the former repository
  path prefix. Canonical/OG metadata, installation descriptor and signed remote URLs must agree.
  Use the existing hosting and signing key. No hosting migration or Flathub submission.
- R5: Cache invalidation remains a single prefix purge, scoped exactly to
  `doomdecklauncher.com/`, never `purge_everything`. Update the strict URL/prefix validator,
  tests and current setup documentation for the new Cloudflare zone. No credentials in Git,
  output or reports. The coordinator/owner provides the new zone/token configuration.
- R6: Inspect all old identity/domain references, classifying necessary historical and upstream
  source references separately from active configuration. Test URL rendering, root-path served
  file checks, prefix allow/reject behavior, CNAME preservation and packaging consistency.
  Run fmt and relevant existing checks; report the exact remaining DNS/Pages/Cloudflare setup
  in `docs/reports/079-domain-and-flatpak-identity.md`. Cite current primary hosting docs for
  configuration claims. Physical Deck installation is still Needs Deck Validation.

## Ownership and coordination

Worker owns packaging/Flatpak assets, site/scripts and their tests, relevant workflows,
current README/setup docs and narrow sandbox identity constants/tests. It must not edit
coordinator-owned overview/decisions/status/journal; propose changes in the report. The other
worker owns headless recipe assertions, services catalog-update locking and resolver test copy;
do not touch those functions or unrelated runtime behavior. Merge the coordinator's current
verification checkpoint before implementation. No visible windows or screen capture, no new
dependencies, no DNS/Pages setting mutation, no push/tag/release, no subagents. The owner is
configuring DNS/Pages; the coordinator handles publication after integration checks.

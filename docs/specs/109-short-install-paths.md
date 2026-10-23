# 109 — Short install directory names

**Status:** In progress · **Depends on:** 009, 020, 022
**Origin:** owner, 2026-10-23, while investigating a Headless Chicken crash: the install
directory used the whole `release` citation as its name, producing long paths with spaces,
commas and parentheses. The owner asked for shorter path logic and offered to test it on the
Deck.

## Goal

Keep every install path short and shell-unfriendly characters out of it, without changing what
is downloaded, verified or launched, and without losing installs that already exist under the
old names.

## Requirements

- **R1 — Bounded directory name.** The per-release install directory is derived from the
  recipe's `release` label: keep the leading label, drop a trailing citation (from the first
  `(`, `;` or `,`), reduce it to ASCII letters, digits, `.`, `_` and `-`, collapse separators,
  trim edges and bound it to 32 bytes. A label with nothing usable becomes `release`.
  Plain labels (`1.23`, `RC14`) are preserved exactly, so existing short paths stay put.
- **R2 — Nothing else changes.** Member paths, sizes, SHA-256 identities, download caching,
  recipe freezing, profiles and launch resolution are untouched. Existing installs are not
  migrated: D-042 allows breaking layout changes while the project is experimental, and the
  owner reinstalls mapsets after the update.
- **R3 — Tests.** Unit tests cover the slug (citation cut, punctuation, empty/`.`-only labels,
  length bound, plain labels) and an integration test covers the canonical install and manifest
  paths.
- **R4 — Publication.** Ship as the `v0.0.16-path` prerelease through the existing release and
  signed-Flatpak pipelines, with the ordinary Linux CI run on the tagged source. The owner
  tests the shorter path on the Deck; whether it changes the Headless Chicken crash is recorded
  honestly, not assumed.

## Out of scope

- Migrating existing installs to the new directory; reinstalling a mapset adopts the short
  name.
- Any change to catalogue release labels, the engine, or the launch arguments.
- Claiming a fix for the Headless Chicken crash before the Deck result.

## Acceptance criteria

- [ ] **A1** (R1) — `release_dir_name` tests pass; `install_dir` ends in the short slug for a
      citation-style release.
- [ ] **A2** (R2) — the focused library tests pass and no other install behaviour changed.
- [ ] **A3** (R4) — CI, release and Flatpak runs succeed on the same tag SHA and the served
      version is `0.0.16`.
- [ ] **A4** (R4) — owner's Deck result recorded for the short path and the crash
      *(Needs Deck Validation)*.

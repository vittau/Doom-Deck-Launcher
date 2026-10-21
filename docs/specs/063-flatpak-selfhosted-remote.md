# 063 — Publish a signed Flatpak repository at vitormach.dev/ddl

**Status:** Ready · **Depends on:** 062
**Origin:** operator, 2026-10-20: "instalação por URL simples para o Deck, com caminho pronto para
Flathub depois se achar que vale." Decisions taken in that conversation: a dedicated repository
GPG key signs the remote, publication happens on `v*` tags plus manual runs, and the remote lives
in this repository's `gh-pages` branch — no extra repository — which GitHub Pages serves at
`https://www.vitormach.dev/Doom-Deck-Launcher/`.

This spec is drafted under the operator's direction because the orchestrator is offline. It does
not change `docs/decisions.md`; the entry that records the channel is proposed in the report.

## Context

Spec 062 produces a `.flatpak` bundle as a workflow artifact. Installing it on a Deck still means
downloading a zip from a CI run, unpacking it and running `flatpak install` on a file. A signed
OSTree repository behind a stable HTTPS URL replaces that with one command and gives real updates
through `flatpak update`/Discover. Flathub remains a later, policy-bound option; this task must not
make it harder.

## Scope and authorization

Owner-authorized packaging and CI work under D-034's exception for spec 062. Still out of scope: a
Flathub submission, a GitHub Release, a public announcement, and any change to application code.

## Requirements

- **R1 — Signed repository.** Export the existing manifest to an OSTree repository, sign the commit
  and the summary with a dedicated GPG key, and publish a `.flatpakref` that embeds the public key.
  Runtimes stay on Flathub through `RuntimeRepo`. No unsigned install path is offered.
- **R2 — Publication pipeline.** Add a workflow triggered by `v*` tags and `workflow_dispatch` that
  builds the tagged commit, regenerates and verifies the Cargo sources, validates the metadata,
  builds offline, exports to `branch=stable`, generates static deltas, prunes old versions and
  publishes the site to this repository's `gh-pages` branch. Keep the existing artifact-only
  `Flatpak` workflow unchanged.
- **R3 — Stable URLs.** The published layout must be, at minimum:
  `…/Doom-Deck-Launcher/io.github.vittau.Doom-Deck-Launcher.flatpakref`,
  `…/Doom-Deck-Launcher/ddl.flatpakrepo`, `…/Doom-Deck-Launcher/repo/` (the OSTree remote), and a
  landing page at `…/Doom-Deck-Launcher/` with the install command.
- **R4 — Key handling.** A helper script generates the repository key outside the owner's default
  keyring, writes the private key to a single-line file for `gh secret set`, and prints the exact
  commands. The workflow imports the key from secrets into a temporary `GNUPGHOME`, never echoes
  it, and fails early when a secret is missing.
- **R5 — Documentation and report.** Document setup, publishing, installing on a Deck, updating,
  uninstalling, key rotation and the Flathub migration path. Record what was verified statically
  and what still needs a real run.

## Acceptance

- **A1 (R2/R3):** workflow YAML parses and passes `actionlint`; every external action is pinned by
  commit; the published paths match `R3`.
- **A2 (R1/R4):** the key script passes `shellcheck`, refuses to overwrite an existing key, and the
  workflow references only the documented secret names.
- **A3 (R5):** docs match the workflow behaviour, including the tag caveat (a tag created before
  this change has no packaging files) and the fact that no run has happened yet.
- **A4:** report at `docs/reports/063-flatpak-selfhosted-remote.md`.

Never open visible windows or capture the screen; verification is static plus CI.

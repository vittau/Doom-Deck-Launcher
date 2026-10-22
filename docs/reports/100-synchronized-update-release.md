# Spec 100 report — synchronized launcher and Flatpak update

**Coordinator:** Codex, following the owner's direct takeover and publication request.
**Status:** integrated and verified locally; Linux CI and publication pending.

## Integration and publication plan

The old coordinator's terminal had exited. Rebound the existing Run
`run_705f06678595` to this terminal; the completed spec 099 Dispatch remains explicitly
retained at the owner's request. No other active worker was present. The original
worktree `/Users/vitor/git/Doom-Deck-Launcher` remains on its unmodified
`spec-098-catalog-update-needs-newer-ddl` branch; its three commits were merged into the
verification branch rather than discarded. `docs/status.md` is not edited, as requested.

Integrated spec 098's actionable catalog-version message with spec 099's build-time
AppStream synchronization, on top of main's shared Cards/List browsing and state fixtures.
Prepared workspace/lockfile version `0.0.13`, matching checked-in AppStream metadata and
release notes at `docs/releases/v0.0.13-update.md`. No historical release tag changes.

The signed publication smoke test now checks the installed `flatpak list` display version
against Cargo and the binary, verifies installed/exported OSTree commits match, and checks
the commit body for the checked-out Git SHA. This builds on the export's existing
`--body` provenance. The CI tooling install adds the maintained OSTree command-line tool
to read that provenance; no application, Cargo or Python dependencies were added. Existing
AppStream validation, signing, cache purge and canonical-file gates are retained.

Confirmed command references: [Flatpak command reference](https://docs.flatpak.org/en/latest/flatpak-command-reference.html)
and [OSTree show](https://ostreedev.github.io/ostree/man/ostree-show.html).

The GitHub connector does not offer tag-push/release-publication commands; authenticated
Git and `gh` are used for publication and workflow monitoring.

## Verification

- Spec 099 before integration: formatting and denied-warning Clippy passed; **985 Rust
  tests**, four helper tests and local `appstreamcli validate --no-net --explain` passed.
  Its actual commits are `8e9d39f`, `5a76cf4` and `048e74b` (the original completion
  message mistakenly named the last commit `b6d1800`).
- Combined `0.0.13` source: formatting, denied-warning Clippy, four helper tests,
  AppStream validation and workflow lint passed. Actionlint's only excluded diagnostic
  is the pre-existing unknown `ubuntu-26.04` runner label. Full workspace tests:
  **988 passed, 0 failed, 0 ignored** (239 app, 640 core, 106 integration, 3 doctests).
- Remote Linux CI, tag source SHA, paired release/publication runs, release checksum,
  canonical served repository provenance and post-release serializer fixture: pending.

## Remaining physical validation

**Needs Deck Validation:** install/update the published Flatpak on the Deck and compare
Discover, `flatpak info`, `flatpak list --columns=version` and `ddl --version`; all should
show `0.0.13`. Verify retained state and the new browsing mode as listed in
`docs/deck-testing.md`. No physical Deck check is claimed here.

Permanent orchestration that prevents every future GitHub/Flatpak release divergence
remains a separate follow-up. This publication will be considered complete only once
both workflows succeed for the same source commit and the served files are verified.

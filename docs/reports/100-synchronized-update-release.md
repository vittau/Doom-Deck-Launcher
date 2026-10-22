# Spec 100 report — synchronized launcher and Flatpak update

**Coordinator:** Codex, following the owner's direct takeover and publication request.
**Status:** published and verified; physical Deck checks remain pending.

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
- [Linux CI 39824796061](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39824796061):
  **988 passed, 0 failed, 0 ignored**, with denied-warning Clippy and Steam Runtime build/smoke.
  It completed just before the owner asked to stop redundant testing; it was not repeated.
- [Flatpak artifact 39824795822](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39824795822):
  passed metadata synchronization/validation, source build, install and binary version smoke.
- [Manual network diagnostic 39824824023](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39824824023),
  attempts 1 and 2: **failed**, both after about 79 seconds waiting for the response from
  `ia600903.us.archive.org/.../dbp63_terminal-voltage.zip`. The downloader is unchanged.
  The failure is not hidden or relabeled as passing. Per the owner's subsequent direction,
  network e2e becomes an opt-in diagnostic, not a metadata/copy/UI release gate (spec 101).
- [GitHub Release 39827799850](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39827799850)
  and [signed Flatpak publication 39827799967](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39827799967)
  both **passed** for immutable `v0.0.13-update`, source
  **`7bf51c1bfc0f3d2b424778f9a9e1e71025a93947`**. Release notes were updated from the
  committed English notes; both the tarball and signed channel are available.
- The signed smoke's fresh install reports **`ddl 0.0.13`** and Flatpak's installed
  display version **`0.0.13`**; installed/exported commits match and the commit body names
  the tag's exact Git SHA. Pages, Cloudflare prefix purge and canonical byte checks passed.
- Independently fetched the canonical served stable ref and its raw OSTree commit object,
  checked the object's SHA-256 against its ref, and decoded its normal-form GVariant with
  the host's existing GLib library. Served OSTree commit:
  **`b849d066dfb9c212f4e2bc1f19b803c4cf51454c4c3d7a69e474e6e5e32ea471`**;
  its source body equals the GitHub tag SHA above. No dependency was installed for this read.
- Downloaded the release tarball and verified its accompanying checksum:
  **`4275b097987ee55e54cf90b679596fc4d4a3bf78652a4078e41794860a190e48`**.
  The downloaded archive was not executed on the development host.
- Generated `tests/fixtures/state/v0.0.13-update/` using the immutable tag's own serializers,
  including explicit Default audio and List in `config/view.json`. Generation and the
  focused `services::release_state_tests::release_state_trees_remain_readable` guard
  both passed. The guard also confirms older releases default to Cards. Existing fixture
  bytes were unchanged; this test-only follow-up did not repeat the full suite.

## CI follow-up and cleanup

The owner explicitly requested fewer redundant checks during publication. Spec 101 removes
routine duplicate delivery builds and makes the ten exhaustive visual cases manual, keeps
ordinary offline checks, and uses lightweight metadata checks for packaging/workflow edits.
Its workflow/documentation changes were reviewed and statically validated; no Cargo suite,
SDK build or e2e was repeated for them. See `101-lean-ci.md` for the retained coverage.
The aggregation push is marked `[skip ci]`: after the verified tag it contains only the
focused-test-verified fixture/guard, reviewed workflows and documentation, so another Rust
suite would duplicate the evidence. No release/tag signing or publication gate was skipped.

The spec 098 branch was merged and pushed before release. The owner's cleanup request refers
to the primary checkout, not a disposable Git worktree: its Orca context is marked completed
and has no active terminals; the repository directory and remote branch are preserved.
`docs/status.md` remains untouched. The supervised CI worker is released after review.

## Remaining physical validation

**Needs Deck Validation:** install/update the published Flatpak on the Deck and compare
Discover, `flatpak info`, `flatpak list --columns=version` and `ddl --version`; all should
show `0.0.13`. Verify retained state and the new browsing mode as listed in
`docs/deck-testing.md`. No physical Deck check is claimed here.

Permanent orchestration that prevents every future GitHub/Flatpak release divergence
remains a separate follow-up. This publication is complete: both workflows succeeded for
the same source commit and the canonical served files were verified.

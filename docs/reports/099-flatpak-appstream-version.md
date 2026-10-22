# Spec 099 report — Flatpak AppStream version

**Worker:** Codex · **Branch:** `vittau/flatpak-metainfo-version`
**Status:** implemented and locally verified; publication and physical Deck validation pending.

## Change

Added `scripts/flatpak/set-metainfo-release.py`, using only Python's standard library
(`tomllib`, ElementTree, argparse, datetime and pathlib). It reads the checked-out
`[workspace.package].version`, inserts or promotes that release to the first position,
sets the explicitly supplied UTC build date and keeps historical releases and any
existing description for the promoted version. Repeated calls with the same inputs are
byte-identical; calls with another date update the entry without adding duplicates.

Both Flatpak workflows now invoke it with `date -u +%Y-%m-%d` immediately before their
existing desktop/AppStream validation steps, and therefore before `flatpak-builder`
copies the source tree. Both `appstreamcli validate --no-net --explain` gates are
unchanged. The manifest still installs that metainfo at the existing `/app/share/metainfo`
destination. No bundle/catalog logic or application source, binary behavior, permissions,
signing, publication or smoke-test logic changed (R1/R3/R4).

The checked-in XML now starts with `0.0.12`, dated `2026-10-22`; the complete `0.0.11`
and `0.0.9` entries remain. ElementTree also normalizes the XML declaration's quote style.
No dependencies were added.

## Verification

- **R2:** `crates/ddl/tests/flatpak_metainfo.rs` reads `Cargo.toml` and the metainfo,
  requiring the first release to equal the workspace version. Compiled and run directly
  with `rustc --test` before synchronization: failed with `0.0.11 != 0.0.12`; the same
  executable passed after running the helper. It also runs under `cargo test --workspace`.
- **R1/R2:** `python3 -m unittest discover -s scripts/flatpak -p
  test_set_metainfo_release.py -v`: **4 tests passed**, covering insertion/history and
  unrelated XML preservation, repeated-run byte equality/date updates without duplicates,
  promotion with preserved description and invalid dates leaving the file unchanged.
- **R1/R4:** ran `python3 scripts/flatpak/set-metainfo-release.py --date 2026-10-22`
  locally and then `appstreamcli validate --no-net --explain
  packaging/flatpak/com.doomdecklauncher.DoomDeckLauncher.metainfo.xml`: **passed**
  (`Validation was successful: pedantic: 1`). AppStream tooling is available on this host.
- **R1/R3/R4:** a standard-library XML comparison against the pre-change Git version
  confirmed both historical release elements are identical; a workflow ordering check
  confirmed helper → unchanged validation → builder in both workflows. Scope review
  verifies no application, bundle or catalog code changes; no runtime test is warranted
  for that absence of changes.
- **R5:** `cargo fmt --all --check`: **passed**.
- **R5:** `cargo clippy --workspace --all-targets -- -D warnings`: **passed**.
- **R5:** `cargo test --workspace`: **985 passed, 0 failed, 0 ignored**, including the new
  AppStream regression guard and all existing headless rendering tests.
- `actionlint .github/workflows/flatpak.yml .github/workflows/flatpak-remote.yml` found
  only its pre-existing unknown `ubuntu-26.04` runner-label diagnostics. Repeating with
  `-ignore 'label "ubuntu-26.04" is unknown'` passed with no other findings.
- `git diff --check`: **passed**.

All tests and validation are offline/headless; no visible window or screenshot was opened.
No push, tag, release or remote workflow dispatch was performed.

## Remaining validation and coordinator handoff

**Needs Deck Validation:** a publication run from a reviewed ref containing this helper
and the workflow changes must build/export the package, followed by a physical Deck check
of `flatpak info`, `flatpak list --columns=version` and Discover against `ddl --version`.
Those real package/display checks were not performed here and are not claimed as passing.
The old `v0.0.12-catalog` tag does not contain this repair; merely republishing its
unchanged tree does not pick up the helper or the corrected metadata.

No implementation questions remain. `OVERVIEW.md`, `docs/decisions.md`, `docs/status.md`
and `docs/journal.yml` are untouched. Proposed coordinator-owned status/journal note:
Flatpak builds now synchronize the AppStream release from the workspace version, with
an offline regression guard; publication and physical Deck display validation remain open.

## Owner requirement — align GitHub releases and Flatpak publication

The coordinator relayed this owner requirement on 2026-10-22: every GitHub release must
be accompanied by a Flatpak publication built from the same source commit; the two must
never diverge. A release is not complete until both exist and are verified against the
same source commit.

**Proposed follow-up:** make the release workflow coordinate both outputs from one
resolved tag SHA, record that source SHA in the Flatpak export's provenance, and verify
that the served app ref resolves to an export whose recorded source SHA equals the tag
SHA before marking the release complete. An OSTree app commit and a Git source commit
are different hashes; verification must compare the export's source provenance with the
tag, rather than compare those unrelated hashes literally. Add a release checklist that
requires both published artifacts, matching source provenance and version checks.

This task does not implement release orchestration, provenance metadata or a checklist,
change publication/signing gates, create a GitHub release, publish a Flatpak, or amend
coordinator-owned decision/status/journal files. The proposal needs its own spec.

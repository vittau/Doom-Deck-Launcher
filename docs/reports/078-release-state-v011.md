# 078 — Preserve the v0.0.11 release state

## Result and scope

Added the `v0.0.11-polish` release-state fixture, generated offline from the
immutable tag with its own embedded catalog and serializers. The generator now
uses an explicit reviewed audio-capable tag set, and the release-state guard
requires the new tree and checks its explicit Default audio choice. No product
behaviour, dependency, older fixture or coordinator-owned document changed.

Only `tests/fixtures/state/generate.py`, the new fixture tree, the fixture
README and `services::release_state_tests` were touched, plus this report.
Preparation was committed separately as `730ba0b`; the generated fixture and
report follow it.

## Tag confirmation and provenance

The coordinator confirmed the tag and exact source commit through the single
blocking orchestration question for this task (question `msg_dc831cae419d`,
answer `msg_e3963a28d76d`, no timeout): the immutable tag is
`v0.0.11-polish` at source commit
`7772da84f9dfe4de8d517fe34f2ca6c4b2f172d1`.

I fetched only that tag with `git fetch --no-tags origin tag v0.0.11-polish`
and verified:

| Object | Value |
| --- | --- |
| Annotated tag object | `8c23bec9b8682da57a464df7770a1d0a42de6934` (type `tag`) |
| Peeled commit | `7772da84f9dfe4de8d517fe34f2ca6c4b2f172d1` |
| Tag annotation | “Polish Deck browsing, setup and independent catalog updates” |
| Commit subject | “Clarify current domain and verified release checkpoint” |
| Commit parent | `42041df2c64f3d78f2f77e9b0dbd34bf8a7fb45e` |
| Tagger / author date | 2026-10-21T20:28:19-03:00 / 2026-10-21T20:27:47-03:00 |

**Confirmed:** the peeled commit matches the coordinator’s exact commit. The
tag contains the dedicated domain and Flatpak identity transition, but
`crates/ddl/src/services.rs` is unchanged between the integration checkpoint
`705a3d9` and the tag. The path/sandbox differences in that range are app-ID
test constants, not persisted state fields.

## Tagged serializer review

Reviewed the actual tagged source before generating anything:

- `AppState` remains schema 2 (`STATE_SCHEMA` in the tagged
  `crates/ddl/src/services.rs`); its folders, recipes, favorites, profiles,
  kept recipes and settings fields are unchanged from v0.0.10.
- `soundfonts::Preferences` remains schema 1 with an optional `choice`, and
  `Choice::Default` is still the production selection value. The tagged
  `Soundfonts::select` writer produced `config/audio.json`.
- `DEFAULT_SKILL` remains 4, the DSDA template remains
  `template_version = "5"`, and the persisted DSDA state carries template
  version 5.
- The domain/identity work changes packaging, paths tests and sandbox
  constants; it adds no saved-state fields or serializers.

The fixture therefore uses the established scenario with v0.0.11 included in
the explicit audio-capable set. No JSON was authored or edited by hand.

## Generator and guard changes

`tests/fixtures/state/generate.py`:

- adds `v0.0.11-polish` to the reviewed tag allowlist;
- replaces the exact `v0.0.10-audio` audio condition with
  `AUDIO_TAGS = {"v0.0.10-audio", "v0.0.11-polish"}` and
  `if tag not in AUDIO_TAGS`;
- keeps the earlier-tag behaviour unchanged: tags outside that set still have
  the audio block removed before the scenario is compiled.

`services::release_state_tests`:

- requires the `v0.0.11-polish` fixture;
- treats both `v0.0.10-audio` and `v0.0.11-polish` as audio-capable for the
  file, loaded preferences, snapshot and no-bundled-font assertions;
- leaves the existing launch-recipe, installed-metadata, display-tag and
  current-shape save assertions unchanged.

The existing wildcard still expects DSDA baseline 5 and default skill 4 for
newer tags; the new fixture verifies both values through the production
open/snapshot path. No validation was weakened and no older tree was rewritten.

The fixture README now lists the tag at commit `7772da8`, documents the
audio-capable set and explains how the next release should extend the
allowlist and reviewed audio set.

## Fixture provenance and contents

Generated with the tagged code in a detached scratch checkout:

```sh
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 \
  python3 tests/fixtures/state/generate.py v0.0.11-polish
```

The generator created `.scratch/v0.0.11-polish` at
`7772da84f9dfe4de8d517fe34f2ca6c4b2f172d1`, injected only the test scenario
into that checkout’s `services.rs`, and ran exactly
`services::release_state_fixture::generate_release_state_fixture` with Cargo
offline and locked. Production serializers and types were not patched. The
scratch checkout is dirty only in its injected `services.rs` and is preserved
for review.

The generated tree has 15 files (60,433 bytes; 108 KB on disk):

```text
config/app.json
config/audio.json
config/dsda/active.cfg
config/dsda/base/5.cfg
config/dsda/state.json
config/iwads.json
config/profiles/fixture-alternate.json
config/profiles/fixture-auger-zenith.json
config/profiles/fixture-sigil.json
data/custom/fixture-custom.json
data/engines/dsda-doom/installed.json
data/library/auger-zenith/active.json
data/library/auger-zenith/generations/fixture/install.json
data/library/sigil/active.json
data/library/sigil/generations/fixture/install.json
```

Key provenance values read back from the generated tree:

- `config/app.json`: `schema_version` 2, two embedded recipe snapshots, two
  synthetic IWAD folders (`/ddl-fixture/iwads-one`, `/ddl-fixture/iwads-two`);
- `config/audio.json`: `{"schema_version":1,"choice":"default"}`;
- `config/dsda/state.json`: template version 5;
- `config/profiles/fixture-auger-zenith.json`: default skill 4.

All files are JSON or plain CFG text; there are no symlinks, archives,
executables, IWADs, mapset payloads, soundfonts, credentials or host-specific
user paths. The only URLs are catalog metadata. The tree remains a state-only
fixture, not a runnable installation.

## Verification

- Generator run: `generate_release_state_fixture` passed against the tag
  (1 passed, 222 filtered out, exit 0).
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test --offline --locked -p ddl
  --bin ddl services::release_state_tests::release_state_trees_remain_readable
  -- --exact`: passed (1 test, 221 filtered out). The guard discovered every
  `v*` tree, opened each through production `Services::open`/`snapshot`, and
  passed the new audio and metadata assertions.
- `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo clippy --offline --locked
  -p ddl --bin ddl --tests -- -D warnings`: passed.
- Git status showed the new fixture, the fixture README and this report as the
  only changed or added paths; no existing `tests/fixtures/state/v0.0.*` tree
  was modified.

## Limitations and open work

No engine, game, content download, visible window, screenshot or Deck/Flatpak
runtime was used. The guard proves that saved state loads through the current
production service path; it does not prove game execution or real Flatpak
filesystem behaviour, which remain **Needs Deck Validation**.

The scratch checkout `.scratch/v0.0.11-polish` must stay until the fixture and
provenance are reviewed; after that it can be removed with
`git worktree remove --force .scratch/v0.0.11-polish`. No spec 055 R4 work
remains for this release. The release checklist should continue to require the
next tag’s generated fixture and required-tag guard entry before the following
release. Coordinator-owned status, decisions, journal, overview, workflows and
release version were not edited.

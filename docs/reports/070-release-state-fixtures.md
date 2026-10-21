# 070 — Release-state fixtures through v0.0.10-audio

## Result and scope

Extended spec 055 R4's saved-state compatibility guard with fixtures for
v0.0.8-upgrade, v0.0.9-focus and v0.0.10-audio. Only fixture generation,
fixture documentation and the Services release compatibility test changed;
no application behavior or dependencies changed.

## Tagged-code inspection and provenance

**Confirmed in tagged source:** before extending the generator allowlist,
compared `services.rs` (`AppState`), catalog model, profile, setup, library
active/manifest, IWAD registry, engine installed-state and DSDA config-state
types at the tags below, including changes from v0.0.7-deck. Inspected the
v0.0.10 soundfont preferences type and production writer separately.

| Release | Exact commit | Scenario / serialized-state differences |
| --- | --- | --- |
| v0.0.8-upgrade | `99258bb8fd7969343ba894c4718d4290b78d0047` | AppState remains schema 2; other scenario records remain schema 1. Persisted recipe decoding accepts legacy recipes without changing the writer's shape. Profile constructor now defaults to skill 4, while the alternate stays explicitly skill 2. DSDA template is version 4. No audio preferences API/file exists. |
| v0.0.9-focus | `f452c1361438b440d38dbaf2a25fdc96e9508132` | Scenario record types/serialized shapes unchanged from v0.0.8. Flatpak service wiring adds no fields to these records. Profile default remains 4, DSDA template remains 4; no audio preferences API/file exists. |
| v0.0.10-audio | `d1a39cc8c2d6780a8176171166dabd580c85480a` | Adds defaulted optional `Component.requires` to recipe copies in app/active state; state schema numbers unchanged. Profile default remains 4; DSDA template is version 5. New schema-1 `soundfonts::Preferences` writes `config/audio.json` with `schema_version` and optional `choice`. |

The engine state, IWAD registry, generation manifest, custom setup and DSDA
state record shapes are unchanged across these tags. The scenario creates
profiles through each tag's constructor, preserving its real default rather
than forcing the older skill 3. Earlier checked-in v0.0.5/v0.0.6/v0.0.7 fixture
bytes are retained unchanged (including their skill 3 defaults).

Generation uses `tests/fixtures/state/generate.py <tag>`: create a detached
`.scratch/<tag>` checkout, read that tag's original `services.rs` with
`git show`, append the shared `generate.rs` scenario, then run exactly
`services::release_state_fixture::generate_release_state_fixture`. All
serializers, types and embedded catalog/template content come from the tag;
none are patched. Cargo runs `--offline --locked` with two build jobs and two
test threads and reuses a target directory across tags. No generated JSON is
hand-authored or edited.

Each tree preserves one inert registered engine, two synthetic IWAD folders
and registry entries, two installed recipes (`auger-zenith` and `sigil`) with
active and generation manifests, two AUGER;ZENITH profiles and its optional
blaster selection, one SIGIL profile, the selected alternate, a favorite,
kept recipe identities, settings, one custom setup and a provisioned DSDA
baseline. Package/member metadata comes from the tagged embedded recipes.
Paths are synthetic `/ddl-fixture/...` references, not real user state.

For v0.0.10 only, the scenario invokes the actual production
`Soundfonts::select(Choice::Default)` writer. This records an explicit Default
choice without needing a soundfont installation or download. Earlier tags
omit this block because they predate the type/API; the v0.0.5 custom-setup
exception remains unchanged. The guard checks both the loaded preferences
and Settings snapshot, no unreadable-preferences warning, and no installed
font. An installed Arachno generation and selection are not represented;
there are no font, IWAD, engine or mapset payloads in these state-only trees.

## Verification

Merged local `main` before final checks: already up to date at `d1a39cc`.
Generation completed offline against each tag:

| Tag | Generator test | Emitted files | Total bytes |
| --- | --- | --- | --- |
| v0.0.8-upgrade | 1 passed, 156 filtered out | 14 | 52,474 |
| v0.0.9-focus | 1 passed, 179 filtered out | 14 | 52,474 |
| v0.0.10-audio | 1 passed, 200 filtered out | 15 | 59,498 |

- `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test --offline --locked -p ddl --bin ddl services::release_state_tests::release_state_trees_remain_readable -- --exact`: passed; all six trees loaded through production Services and snapshots (1 test passed, 199 filtered out).
- Negative controls ran the just-built test binary with that exact filter and two test threads. Temporarily moving the v0.0.10 tree outside the fixture root failed with exit 101 and `missing release fixture v0.0.10-audio`. Restoring it, then temporarily moving only `config/audio.json` out failed with exit 101 at the new audio-file assertion. Both moves were restored in `finally` blocks; no JSON contents were edited.
- After restoration, the same guard passed again. SHA-256 checks of every fixture file before/after confirmed identical contents; the three earlier fixture trees are unchanged in Git.
- `cargo fmt --all --check`: passed. `git diff --check`: passed.
- `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo clippy --offline --locked -p ddl --bin ddl --tests -- -D warnings`: passed (includes ddl-core dependencies and test-code linting; no tests executed).

All three detached generator worktrees were removed with `git worktree remove
--force` after generation; `git worktree list` confirms no nested scratch
checkouts remain.

## Limitations and open work

No network content downloads, paid assets, engine execution, visible windows
or screenshots were used. Tests exercise actual state loading and snapshots;
they do not prove game execution or Steam Deck/Flatpak filesystem behavior.
Those remain **Needs Deck Validation**. Full workspace/UI tests were not rerun,
as requested. No new dependency or product functionality was introduced.

No spec 055 R4 work remains for these three tags. The existing release
checklist should continue requiring the next tagged serializer fixture tree
and required-tag guard entry before the following release. Coordinator-owned
status, decisions, journal, overview, workflows and release version are untouched.

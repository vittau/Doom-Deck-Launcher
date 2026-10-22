# 096 — Preserve the v0.0.12 release state

## Result and scope

Extends spec 055 R3/R4 with an immutable `v0.0.12-catalog` state fixture,
using the same offline scenario as report 078. The generator's reviewed tag
allowlist and audio set, the guard's required-tag/audio sets, and the fixture
README include the release. No product code, dependencies, old fixture bytes
or coordinator-owned documents are changed.

## Authoritative tag and serializer review

**Confirmed locally**, without fetching or modifying any tag:

| Object | Value |
| --- | --- |
| Annotated tag | `v0.0.12-catalog` |
| Tag object (type `tag`) | `2b1bcb9ecb739ed38d64cb86363100a35008462c` |
| Peeled commit | `409d6c504da02cc69f1c4b39030293b44acfb40a` |
| Commit subject | Record the combined verification results |
| Tagger date | 2026-10-22T11:14:25-03:00 |
| Commit date | 2026-10-22T11:14:14-03:00 |

Commands: `git rev-parse v0.0.12-catalog v0.0.12-catalog^{}`,
`git cat-file -t v0.0.12-catalog`, and
`git show --no-patch --format=fuller v0.0.12-catalog`.
The peeled commit matches the task's authoritative commit exactly.

Reviewed actual tagged source with `git show` and the diff from
`v0.0.11-polish` to `v0.0.12-catalog`:

| Persisted area | Tagged change since v0.0.11 | Handling |
| --- | --- | --- |
| `config/app.json`, `AppState` | No top-level field or meaning change; `STATE_SCHEMA` remains 2. Nested recipe snapshots gain optional metadata. | Unchanged production writer; production open/snapshot guard. |
| Stored `Recipe` in app/active/previous-recipe copies | `original_release: Option<ReleaseDate>` added; omitted when unknown. `ReleaseDate` preserves year/month/day precision. `StoredRecipe` defaults absent dates to unknown and retains present dates. | New tree serializes the tagged embedded recipes, including their actual date metadata. Full recipe equality remains in the guard. |
| Recipe package serialization | Empty `packages` omitted; machine-written reader defaults omission to empty (spec 094's player-supplied main). | Established AUGER;ZENITH/SIGIL scenario is unchanged; both have real package metadata. No fabricated package-free scenario. |
| `config/audio.json`, `Preferences` | No change; schema 1, optional `Choice`; `Choice::Default` serializes as `default`. | Add exact tag to both explicit audio sets; tagged `Soundfonts::select` writes the file. |
| Profile, custom setup, IWAD registry, engine state, DSDA state/config | Serializer sources unchanged; profile schema 1/default skill 4 and DSDA baseline 5 retained. | Existing scenario and guard assertions retained. |
| Library active/generation types | No production source change (only library tests changed). Nested recipe metadata changes as above. | Existing manifest, member and frozen-recipe assertions retained. |
| Catalog bundle/cache | Bundle schema remains 1; minimum client becomes 0.0.12 because earlier strict recipe readers reject `original_release`. | This is a catalog capability gate, distinct from saved-state schema meaning. The established scenario does not generate a catalog cache. |

Pinned source evidence: [AppState and writers](https://github.com/vittau/Doom-Deck-Launcher/blob/409d6c504da02cc69f1c4b39030293b44acfb40a/crates/ddl/src/services.rs),
[recipe model](https://github.com/vittau/Doom-Deck-Launcher/blob/409d6c504da02cc69f1c4b39030293b44acfb40a/crates/ddl-core/src/catalog/model.rs),
[machine-state recipe reader](https://github.com/vittau/Doom-Deck-Launcher/blob/409d6c504da02cc69f1c4b39030293b44acfb40a/crates/ddl-core/src/catalog/persisted.rs),
[bundle gate](https://github.com/vittau/Doom-Deck-Launcher/blob/409d6c504da02cc69f1c4b39030293b44acfb40a/crates/ddl-core/src/catalog/bundle.rs),
and [audio writer](https://github.com/vittau/Doom-Deck-Launcher/blob/409d6c504da02cc69f1c4b39030293b44acfb40a/crates/ddl-core/src/soundfonts/mod.rs).
These are reviewed repository sources, not claims about upstream game execution.

The additive original-release metadata changes serialized bytes and canonical
recipe identities, but does not reinterpret player choices or existing state
fields. No state schema bump or migration backup is called for here. Strict
catalog validation, legacy-state conversion and every existing guard assertion
remain intact; only the explicit required-tag/audio sets change in services.

## Generation and provenance

```sh
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 \
  python3 tests/fixtures/state/generate.py v0.0.12-catalog
```

The generator creates `.scratch/v0.0.12-catalog` detached at the verified
commit, appends the existing test scenario to its `services.rs`, and invokes
only `services::release_state_fixture::generate_release_state_fixture` with
Cargo `--offline --locked`. The tagged serializers and types are not modified.
`generate.rs` remains byte-for-byte unchanged (SHA-256
`6c2f74c5fd234e313317d26be144fce33a1f55e907bb00a2928e9c5427a53656`).
No generated JSON is authored or edited by hand.

The tagged generator passed (1 passed, 226 filtered out; 3m 13s build,
0.19s test). It wrote **15 files, 60,975 bytes**:

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

Readback confirms AppState schema 2, explicit schema-1 Default audio choice,
DSDA template 5 and profile default skill 4. The tagged AUGER;ZENITH recipe
contains `original_release = {year: 2021, month: 7, day: 26}` and SIGIL contains
`{year: 2019, month: 5, day: 31}`. Those are values serialized from the tagged
catalog, not independently researched release-date claims. Recipe identities
are therefore recomputed by the tag's canonical recipe hash function; older
kept identities are not rewritten.

All files are JSON or CFG text, with no symlinks or bundled engine, IWAD,
mapset or soundfont payloads. The fixture uses stable synthetic
`/ddl-fixture/...` references. It preserves the established engine, two IWAD
folders/entries, two installed recipes with active/generation metadata, three
profiles (two on AUGER;ZENITH, selected alternate with optional blaster),
favorite, kept identities, settings and custom setup scenario.

The scratch checkout was verified at the peeled commit with only the 91-line
scenario injection changed. Its diff was saved locally in
`.scratch/v012-injected-scenario.patch`, and it was removed with
`git worktree remove --force .scratch/v0.0.12-catalog` after generation.
The command, immutable source commit and unchanged scenario hash above preserve
reproducible provenance independently of scratch cleanup.

## Verification

Merged local `main` before final checks (`git merge main`: already up to date
at `a8ebba83fb58233a33a60a6d7867f57cd20d19b4`). Preparation commit:
`19a5ca2` (reviewed generator/README/guard sets).

- Tagged offline generator: passed, 1 test, 226 filtered out.
- `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test --offline --locked -p ddl
  --bin ddl services::release_state_tests -- --test-threads=2`: passed, 1 test,
  225 filtered out. This is the full module; it opens all eight fixture trees
  through production `Services::open`/`snapshot` and preserves all prior
  recipe, profile, registry, engine, settings, manifest and audio assertions.
- `cargo fmt --all --check`: initially requested one line-wrap correction in
  the audio-tag expression; corrected that expression and the final check passed.
- `git diff --check`: passed.
- Byte checks: all 99 files in the seven older release trees match their
  SHA-256 baseline and `git show HEAD:<path>`; an explicit `git diff --exit-code`
  against the starting main commit also shows no old fixture changes.
- The new tree contains exactly 15 regular JSON/CFG files with valid JSON,
  no symlinks, and no bundled content payloads.

- `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo clippy --offline --locked
  -p ddl --bin ddl --tests -- -D warnings`: passed (1m 45s), no warnings.

## Limitations and remaining work

No dependencies were added. No visible window, screen capture, network test,
engine execution, game launch, tag, push, release or main merge was performed.
The task branch only merged main into itself. This task establishes saved-state
readability, not Deck/Flatpak runtime behavior (**Needs Deck Validation**).
The coordinator runs combined workspace checks after integrating compact-list
work; the heavy full workspace suite was deliberately not duplicated here.

No R3/R4 work remains for this release after successful checks. Keep the release
checklist requirement: review the next immutable tag's serializers, generate its
new state tree with that tag's serializers and the offline scenario, add it to
the required-tag/audio sets as appropriate, and run the release-state guard
without rewriting any previous tree. No coordinator-owned document was edited.

# 055 — Legacy state upgrade and recovery

## Result

Implemented state-only legacy recipe decoding and a dedicated unreadable-state
screen. Catalog TOML remains strict. No new dependencies.

**Confirmed:** before enabling the adapter, a real `v0.0.6-local` `app.json`,
written by that tag's serializers in `.scratch/v0.0.6-local`, failed the current
`AppState::parse` test (`release_v006_app_state_loads`, exit 101). The persisted
`compatibility.complevel_status` field is refused by the current strict recipe
type. The same recipe copies also occur in `library/*/active.json`, including
`previous_recipe`; fixing only `app.json` would leave installed content broken.

## Serialized-state audit (R1)

Compared tagged type definitions and writers at `9187df3` (`v0.0.5-screens.1`),
`c5c1294` (`v0.0.6-local`), and `716bee5` (`v0.0.7-deck`). “Changed” means
serialized shape, not unrelated implementation edits or different saved values.
Paths below are relative to the XDG roots resolved by DDL.

| File | Serialized type / format | Changed 5 → 6 | Changed 6 → 7 | Handling in 055 |
| --- | --- | --- | --- | --- |
| config/app.json | AppState schema 2, nested Recipe/Compatibility | No | Yes, nested `complevel_status` removed | Typed stored-recipe decoder ignores the removed evidence field and converts numeric levels to their game-aware alias when one names the number, **only for copies carrying the removed `complevel_status` marker** (v0.0.5/v0.0.6 shape). Current-schema copies keep their exact representation. Other fields and player choices survive. |
| data/library/<id>/active.json | ActiveInstall schema 1; current/previous Recipe; InstallManifest | No | Yes, nested Recipe only | Same decoder on both recipe fields; retains manifests, pending profiles, notices, retired directories and uninstall marker. |
| data/library/<id>/<release>/install.json and generations/<generation>/install.json | InstallManifest schema 1; InstalledPackage/InstalledMember | No | No | Existing bounded loader; exact generation manifests asserted by fixture guard. |
| config/profiles/<id>.json | Profile schema 1 | Yes, additive `extras` | Yes, additive `iwad_path` | Existing serde defaults read absent fields; profile ids, skill, components and selection asserted. |
| data/custom/<id>.json | CustomSetup schema 1; CustomIwad, Compatibility, UserFile | New in 6 | No | Existing loader; compatibility is still `auto` or numeric. v5 cannot supply a feature it predates (coordinator-approved exception). |
| config/iwads.json | Registry schema 1; RegistryEntry/FileTime/Source | No | No | Existing bounded loader; folder references and registry entries asserted. |
| data/engines/dsda-doom/installed.json | InstalledState/InstalledEngine schema 1 | No | No | Existing loader; selected engine and version asserted. |
| data/catalog/catalog.json | CachedBundle/Bundle containing catalog TOML text | No | No outer change; recipe text changed | Existing bounded cache loader validates hand-authored TOML strictly; unusable old cache falls back to embedded catalog. Installed recipes are recovered separately, so this cache fallback loses no player installation. |
| cache/engines/releases.json | CachedReleases schema 1, raw upstream response body | No | No | Existing cache behavior; corruption falls back to a fresh check, never blocks player-state recovery. |
| cache/downloads/<hash>.part.json and .part | Sidecar schema 1 + partial bytes | Not present in 5 or 6 | New in 7 | Existing bounded/resumable cache rules; invalid sidecars discard partials, not installed content. |
| cache/downloads/<hash> and staged/extracted members | Raw bytes, not a state schema | No | No | Hash/size verification remains unchanged; no migration. |
| config/dsda/state.json | PersistedState schema 1, Conflict | No | No | Coordinator-approved nonfatal reset notice; explicit Reset now first preserves corrupt bytes as `state.json.unreadable-<UTC timestamp>`. Stored template versions remain readable. |
| config/dsda/active.cfg, base/<version>.cfg, backups/*.cfg | DSDA text config | No template change | Yes, baseline template 1 → 3 | Existing three-way template migration; tagged baseline/state files included. Config edits remain player data. Spec 056 owns the subsequent baseline migration. |
| data/profiles/<id>/{save,data,shots,run}/ | Engine-owned opaque files | No DDL serialization | No DDL serialization | Never modified by recovery. |
| config/state.lock, config/dsda/run.lock | OS lock / PID text | Lock implementation only | No shape change | Existing lock protocol; Start fresh takes the normal state lock. |
| state/logs/* | Plain log text | No schema | No schema | Diagnostics computes the log location without opening Services. |
| data/iwads/<variant>/<canonical>.wad | Canonical link to player IWAD | No | No | Rebuilt by existing resolution; never bundled, downloaded, or removed by recovery. |
| data/engines/dsda-doom/<version>/ and library payloads | Engine/AppDir and verified mapset bytes | No state schema | No state schema | Recovery renames only invalid records, preserving payloads. |
| Temporary JSON writes and unpublished generations | Interrupted transaction artifacts | No relevant schema | No relevant schema | Existing atomic publication and cleanup rules remain in effect. |

**Confirmed:** an alias names the same DSDA level for the recipe's base game,
as recorded in [DSDA v0.30.1 settings.c](https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c#L60-L170)
and `docs/research/dsda-engine.md` §1. No field meaning changes. Schema numbers therefore remain unchanged, and no `.v<N>.bak` schema
migration is needed. Saves use the existing atomic writers and emit current
recipe fields, without `complevel_status`.

### Regression found by the full workspace run, and its fix

Normalizing every numeric complevel made a current-schema test catalog
(`Number(11)`) read back as `Alias(Mbf)`, so the spec-020 comparison reported a
spurious update. The adapter now maps a number to an alias only when
`complevel_status` is present. Two tests in `catalog::persisted` pin both sides:
a v0.0.6 copy (`11` + status) equals a catalog recipe that now says `mbf`, so no
update is reported after migration; and a current-schema `Number(11)` round-trips
unchanged.

## Fixtures and guard (R2–R4)

The fixture trees and reproducible generator live at `tests/fixtures/state/`.
Each was made by appending the scenario test to **the tag's original**
`services.rs` in its detached `.scratch/<tag>` worktree and running:

```sh
DDL_FIXTURE_OUTPUT="$PWD/tests/fixtures/state/<tag>" \
CARGO_TARGET_DIR="$PWD/target" \
cargo test --manifest-path .scratch/<tag>/Cargo.toml -p ddl --bin ddl \
  services::release_state_fixture::generate_release_state_fixture -- --exact --nocapture
```

`python3 tests/fixtures/state/generate.py <tag>` reproduces the process. No JSON
was authored by hand. These are complete serialized state trees with inert
synthetic engine/IWAD references, package/member metadata from each tag's
embedded catalog, and provisioned DSDA baseline files; game payloads, actual
IWADs and executable engines are intentionally absent. See the fixture README
for scenario details and the v5 custom-setup exception.

The guard discovers every `v*` tree, opens it with production `Services::open`
and `snapshot`, and asserts engine, folders, IWAD registry, two installs and
their generation manifests, optional component, profiles and selection, skills,
favorite, kept identities, settings and custom setup. It also verifies a next
atomic save emits current recipe shapes.

Negative control: temporarily changed the v6 fixture's `last_profiles` field
from its tagged map to integer `42`, ran the guard, and restored the original
fixture bytes in a `finally` block. The guard exited 101 with:

```text
v0.0.6-local: .../config/app.json: invalid type: integer `42`, expected a map
services::release_state_tests::release_state_trees_remain_readable ... FAILED
```

The restored trees then passed the same guard as part of the 10 targeted
`state_` tests. Checked-in fixtures remain byte-for-byte serializer output.

## Recovery and Settings (R5–R7)

Startup inventories unreadable machine-state records and returns their paths
and one-line causes. The recovery screen offers Retry, Start fresh and
Diagnostics. A second failed read displays an explicit “Retry failed” result.
Start fresh confirms first, revalidates under the state lock, and renames only
still-unreadable files to `.unreadable-<UTC timestamp>` backups, adding a suffix
on collision. Corrected files and existing backups stay intact.

Surviving active install records are discovered on disk even when `app.json`
is missing, so engine/install/profile/registry/custom metadata remains
available after app-state recovery. Content payloads and save directories are
untouched. Automatic IWAD discovery continues through the normal first-run
path.

The empty Settings panels were caused by gating **all** panel content, including
About/Diagnostics, on a successful startup snapshot. Diagnostics now computes
version, `logs/ddl.log`, config/data paths and failing files directly from the
runtime environment; state-dependent panels display an explanation. The old
Diagnostics action also selected focus id 1, which immediately selected Engine
again during synchronization; it now selects the Diagnostics section.

Per coordinator direction, `dsda/state.json` retains its previous nonfatal
reset notice because this bookkeeping can be rebuilt. The existing regression
test now also confirms an explicit reset preserves its corrupt bytes in an
unreadable backup. No spec-056-owned config module was edited.

Recovery actions and confirmation use existing D-pad/A/B focus handling.
Long recovery text sizes the common node viewport and keeps the action row
reachable by focus scrolling. Right-stick input belongs to parallel spec 057;
its generic viewport integration must retain `Screen::Recovery` when the
coordinator merges that branch. This branch merged current `main` (`d16e9b2`)
as `a9fbdba` before final checks; spec 057 was not yet on main.

## Verification and acceptance

| Acceptance | Result / evidence |
| --- | --- |
| A1 audit | Done: table above covers persisted state, caches and opaque files. |
| A2 tagged fixtures / preservation | Pass: all three tag generators and production load/snapshot guard; v5 feature exception documented. |
| A3 release guard / negative control | Pass: broken map field exits 101; original bytes restored and all fixture trees pass. |
| A4 headless recovery and key events | Pass: `headless::recovery::saved_state_recovery_with_controller_keys`; four required PNGs rendered and visually inspected. |
| A5 workspace fmt / clippy / tests | Pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` (659 passed, 0 failed) after the complevel fix. |

Targeted checks passed (including `cargo clippy -p ddl --all-targets -- -D warnings`): `cargo test -p ddl-core catalog::persisted` (4 tests),
`cargo test -p ddl --bin ddl state_` (10 tests), and
`cargo test -p ddl --bin ddl unreadable_config` (1 test).

Off-screen artifacts: `target/headless/saved_state_recovery_with_controller_keys/`
contains `055-recovery.png`, `055-start-fresh-confirm.png`,
`055-retry-still-unreadable.png`, and `055-diagnostics-unreadable.png`. All four
were opened as generated image files and checked for clipping/readability.

No visible application window or screen capture was used. UI evidence is
rendered off-screen by Slint's software renderer.

## Follow-up proposals and open items

- **Proposed release-checklist line:** “Before tagging a release, add its
  serializer-produced `tests/fixtures/state/<tag>/` tree (document unsupported
  features for old tags) and run
  `cargo test -p ddl --bin ddl release_state_trees_remain_readable`; keep all
  earlier fixture trees.”
- **Needs Deck Validation:** set up `v0.0.6-local`, upgrade in place, and verify
  engine, IWAD folders, installed generations, favorite, profiles/selection and
  custom setup remain available; launch AUGER;ZENITH. Coordinator should add
  this check to `docs/deck-testing.md`.
- No unresolved implementation items. `main` had not advanced past `d16e9b2` at the final check, so no further merge was needed.

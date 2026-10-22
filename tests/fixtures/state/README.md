# Tagged state regression fixtures (spec 055)

These trees were serialized by the real tagged Rust code, offline, using a test
injected into `services.rs` in detached git worktrees under this worker's
`.scratch/`. No JSON was authored by hand. `generate.rs` is the scenario;
`generate.py <tag>` creates the scratch checkout and invokes only that scenario.
The original serializers and types remain unchanged. Cargo runs offline and
locked, with two build jobs and two test threads; `CARGO_TARGET_DIR` can point
to an existing build cache. Remove the detached `.scratch/<tag>` git worktree
after generation (`git worktree remove --force .scratch/<tag>`).

| Tag | Commit | Scenario difference |
| --- | --- | --- |
| v0.0.5-screens.1 | 9187df3 | No custom setups: this tag predates that feature (coordinator-approved exception). |
| v0.0.6-local | c5c1294 | Includes one custom setup. |
| v0.0.7-deck | 716bee5 | Includes one custom setup and this tag's DSDA baseline. |
| v0.0.8-upgrade | 99258bb | Profile default skill 4; DSDA baseline 4. |
| v0.0.9-focus | f452c13 | Same scenario, profile default skill 4 and DSDA baseline 4. |
| v0.0.10-audio | d1a39cc | Profile default skill 4, DSDA baseline 5, component dependency metadata and explicit Default audio choice. |
| v0.0.11-polish | 7772da8 | Same audio-capable scenario: profile default skill 4, DSDA baseline 5 and explicit Default audio choice; the domain/Flatpak identity work does not change state serializers. |
| v0.0.12-catalog | 409d6c5 | Same scenario and audio choice; embedded recipe snapshots include optional `original_release` date metadata, without changing saved-state field meanings or AppState schema 2. |
| v0.0.13-update | 7bf51c1 | Same scenario and explicit Default audio; the tag's production view-preference writer saves List in the isolated schema-1 `config/view.json`. |

Each config/data state tree contains a registered engine, two IWAD folder
references and registry entries, two installed recipe snapshots with active and
generation manifests (including package/member metadata), two profiles on
AUGER;ZENITH including an optional blaster, one SIGIL profile, a selected
alternate profile, a favorite, kept recipe identities, settings, and provisioned
DSDA configuration. User-content paths are intentionally stable synthetic
`/ddl-fixture/...` references. No real IWAD, engine executable, mapset payload,
savegame or downloaded archive is bundled; these are complete *state* trees,
not runnable game installations. Package/member identities come from each tag's
embedded recipes. The engine and IWAD references are inert test metadata.

`services::release_state_tests::release_state_trees_remain_readable` discovers
all `v*` directories and opens them through production `Services::open` and
`snapshot`, then checks the preserved player choices, engine, registry,
installed generations, and current-shape save. Resolver failures for absent
synthetic content are expected and do not erase installed metadata.

`config/audio.json` exists only in the reviewed audio-capable fixtures
(`v0.0.10-audio`, `v0.0.11-polish` and `v0.0.12-catalog`). Each scenario calls that tag's
production `Soundfonts::select(Choice::Default)` serializer, so the guard checks
an explicit choice survives loading. Earlier tags predate audio preferences and
correctly load with no explicit choice. No soundfont is installed or bundled,
and no download/engine/UI code is run.

For a new release, review the tagged serializers, add the tag to the generator's
allowlist and, only if it is audio-capable, to its explicit `AUDIO_TAGS` set;
then generate a new tree without changing existing ones and run this guard.
Keep the scenario's ids and explicitly document any features that the tag
predates.

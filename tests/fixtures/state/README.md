# Tagged state regression fixtures (spec 055)

These trees were serialized by the real tagged Rust code, offline, using a test
injected into `services.rs` in detached git worktrees under this worker's
`.scratch/`. No JSON was authored by hand. `generate.rs` is the scenario;
`generate.py <tag>` creates the scratch checkout and invokes only that scenario.
The original serializers and types remain unchanged.

| Tag | Commit | Scenario difference |
| --- | --- | --- |
| v0.0.5-screens.1 | 9187df3 | No custom setups: this tag predates that feature (coordinator-approved exception). |
| v0.0.6-local | c5c1294 | Includes one custom setup. |
| v0.0.7-deck | 716bee5 | Includes one custom setup and this tag's DSDA baseline. |

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

For a new release, review and update the generator for that tag, generate a new
tree without changing existing ones, then run this guard. Keep the scenario's
ids and explicitly document any features that the tag predates.

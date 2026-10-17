# 021 — GOG and other store IWAD paths

## Result and rationale

**Confirmed:** R1–R5 are implemented; **A1–A4 pass**, **A5** stays *Needs Deck Validation*.
DDL now finds GOG IWADs the way it finds Steam ones: read-only, bounded, tolerant, and
through the same content-identification pipeline. A new `crates/ddl-core/src/iwads/stores.rs`
reads the launcher records researched in `docs/research/iwad-identification.md` §4.1 and
probes the documented default folders; candidates carry a **source** that the IWAD list and
`ddl iwad scan` both show. R4's generic roots are a new persisted setting, default **off**.

**The one design conservative choice:** research could not pin a source-backed *internal*
folder layout for each GOG DOOM product (does GOG's *DOOM + DOOM II* use Steam's
`base/` + `rerelease/` split? where does the classic Final Doom installer put `tnt.wad`?).
Rather than invent a path, DDL treats the **launcher-recorded install directory** as a root
and looks for `*.wad` non-recursively in that directory and one level of its subfolders. The
content identifier then decides. A wrong or empty directory simply yields no candidates.

**Platform note:** the app-state `schema_version` moved **1 → 2** for the new setting (see
R4); v1 files migrate on load. This is a `crates/ddl` change the coordinator asked to be
sequenced with spec 015's app-state work.

## R1 — Research (A1)

`docs/research/iwad-identification.md` gained **§4.1 "GOG and other store IWAD locations
(spec 021)"**, plus two new open questions and a pinned-sources block. Every path and JSON
key is cited; nothing is invented:

- **Heroic Games Launcher v2.22.4** — `gog_store/installed.json` and its `install_path` key
  (`src/backend/storeManagers/gog/electronStores.ts`, `…/library.ts`); config dir
  `~/.config/heroic`, Flatpak `~/.var/app/com.heroicgameslauncher.hgl/config/heroic`
  (`package.json` name `heroic`, `src/backend/constants/paths.ts`); default install dir
  `~/Games/Heroic` (`paths.ts`, `config.ts`).
- **minigalaxy** — `~/.config/minigalaxy/config.json`, key `install_dir`, default
  `~/GOG Games` (`minigalaxy/paths.py`, `config.py`); Flatpak
  `~/.var/app/io.github.sharkwouter.Minigalaxy/…`.
- **GOG offline installers** — default `$HOME/GOG Games`
  ([GOG Support](https://support.gog.com/hc/en-us/articles/213039625-Where-is-my-game-installed)).
- **Lutris** — install dirs live only in the binary SQLite DB `~/.local/share/lutris/pga.db`
  (`lutris/game.py` `directory` column). **Researched but not implemented** (open question
  10): a SQLite reader is a new dependency the project has so far avoided (cf. D-018).
- **Revisions** — GOG *DOOM + DOOM II* carries the 1.9 originals **and** the 2024 KEX copies;
  GOG *Final Doom* ships the Anthology/GOG `tnt.wad`/`plutonia.wad`. The slug
  `doom_doom_ii` is confirmed by the Lutris `gogslug` field on installers 35796/35797.

## R2 — Discovery (A2)

`crates/ddl-core/src/iwads/stores.rs`:

- `heroic_installed_files(home)`, `minigalaxy_config_files(home)` — native and Flatpak paths.
- `install_paths_from_heroic(text)` — tolerant: parses JSON, collects every absolute
  `install_path` value anywhere in the tree (the record's wrapper is not asserted); garbled
  text yields nothing.
- `install_dir_from_minigalaxy(text)` — reads the absolute `install_dir`; relative/absent
  yields nothing.
- `discover(home) -> StoreDiscovery` — reads the records, adds the documented defaults
  `~/Games/Heroic` and `~/GOG Games`, and walks each root: a recorded install dir one level
  deep, a store default two levels deep (game → subfolder). Every path is bounded
  (`MAX_RECORD_BYTES` 4 MiB, `MAX_CANDIDATES` 4096, 4096 entries/level), read-only, missing
  paths and symlinked directories are normal, and a file that exists but cannot be read or
  parsed is reported in `StoreDiscovery::unreadable` (logged by the caller) — never an error
  and never a dialog.
- `standard_roots(home, env)` / `standard_paths(home, env)` — R4's `DOOMWADDIR`/`DOOMWADPATH`
  (colon-split), `/usr/share/games/doom`, `~/.local/share/games/doom`; the environment is a
  closure so tests never touch the process environment.

`crates/ddl-core/src/iwads/registry.rs` gained `Source::GogHeroic`, `GogMinigalaxy`,
`GogOffline`, `StandardFolder` (stable wire names `gog_heroic`, …) and `Source::label()`
("GOG via Heroic", …). Existing entries keep their names (`steam`, `userfolder`, `manual`),
so old `iwads.json` files still load.

## R3 — Same pipeline, source shown

`Services::scan_impl` (shared by startup, the UI Scan command and `ddl iwad scan`) adds the
store candidates and, only when the setting is on, the standard roots; it logs each garbled
record with `tracing::warn!`. Candidates then go through `Identifier::identify` unchanged —
ambiguous files stay ambiguous and are never auto-chosen. The source is shown in:

- the **IWAD list** (Review screen) — the Ember row's label is now the source, the detail the
  variant and path;
- **`ddl iwad scan`** — each line is prefixed with the source label, and `ddl doctor`'s
  `iwads:` rows print `[source]` too.

## R4 — Opt-in generic roots

`AppState` gained `search_standard_folders: bool` (`#[serde(default)]`, off), the schema
moved to `APP_SCHEMA_VERSION = 2`, and `Services::open` migrates a v1 file (no field →
`false`) and rejects anything else. A new `Command::SetSearchStandardFolders(bool)` persists
the toggle (it takes the state lock like other mutations). The Settings → **IWAD folders**
section gained one row (id `72`) built with the existing `Nodes::row` component: label
"Search standard Linux folders", detail `On`/`Off`; the existing Add-folder/Rescan buttons
and the folder list moved down one row to make space, and the section panel height follows.
No new visual design.

## R5 — Deck checks

`docs/deck-testing.md` gained a **"GOG and other stores (spec 021)"** table (G1–G7): Heroic
(G1, also A5), offline installer, minigalaxy, the opt-in toggle, the `iwad scan` source
label, Flatpak Heroic, and Lutris-by-hand.

## Verification

macOS arm64, repository toolchain. **361 tests passed, zero failures** across the workspace.

`cargo fmt --all --check` — exit 0, no output.

`cargo clippy --workspace --all-targets -- -D warnings` — exit 0, tail:

```text
    Checking i-slint-renderer-software v1.18.1
    Checking i-slint-renderer-femtovg v1.18.1
   Compiling ddl v0.0.4 (.../crates/ddl)
    Checking slint v1.18.1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 36s
```

`cargo test --workspace` — exit 0, result lines:

```text
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 89.21s   (ddl)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s    (cli_help)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s    (catalog_cli)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s    (logo_assets)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s    (ui_strings)
test result: ok. 283 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s  (ddl-core)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s    (probe bin)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s   (dsda_launch)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s    (doctests)
```

Tests added (12 ddl-core + 5 ddl + the headless render assertions):

- `iwads::stores::tests` (11): Heroic record parse, malformed/garbled tolerance, minigalaxy
  parse and tolerance, record-over-default preference, missing/garbled records still probing
  defaults, Flatpak (Heroic + minigalaxy), the one-subfolder depth limit, `standard_roots`
  and `standard_paths`, and an empty home.
- `iwads::registry::tests::store_sources_keep_stable_names_and_labels` — the new `Source`
  wire names and labels.
- `services::store_wiring_tests` (3): a Heroic GOG install is found and labelled
  `GogHeroic`; a garbled record fails nothing and adds nothing; the standard roots stay off
  until enabled (then labelled `StandardFolder`).
- `cli_exec::tests::iwad_scan_labels_a_gog_install_from_heroic` — the CLI prints
  `GOG via Heroic: …`.
- `services::tests::app_state_migrates_from_v1_to_v2_and_persists_the_new_setting` — the
  v1→v2 migration and default; the unknown-schema test now uses version 3.
- The production headless journey (`headless_production.rs`) renders the Settings row,
  asserts it defaults to `Off`, toggles it (row detail becomes `On`) and toggles back.

Fixtures: `crates/ddl-core/tests/fixtures/gog/heroic-installed.json`,
`heroic-installed-malformed.json`, `minigalaxy-config.json`. No test touches the network,
opens a window or captures the screen. **New dependencies: none.**

## Proposed changes outside my ownership

- `docs/decisions.md` is untouched. If the coordinator wants it recorded, a new decision
  could state "store discovery reads launcher records only, never a SQLite DB", or the new
  `Source` variants' wire names.

## Open questions / items left open

- **A5** (Deck): a GOG install via Heroic is found on the real Deck — Deck check **G1**.
  The Flatpak route and the exact product folder layout are also unvalidated (G6).
- **Lutris**: not implemented (open question 10) because install paths live in a binary
  SQLite DB. Lutris installs remain a manual "Add IWAD folder" job (Deck check G7). Owner
  decision wanted: add SQLite, write a tiny reader, or leave it.
- **GOG Final Doom revisions** (open question 11): the Anthology/GOG `tnt.wad`/`plutonia.wad`
  hashes are published in research §1.3/§1.4 but are **not** in `data/iwads.toml`, so a GOG
  Final Doom pair identifies only as `Ambiguous`. Follows spec 007 R1's "do not invent
  hashes"; adding them is a separate change if wanted.

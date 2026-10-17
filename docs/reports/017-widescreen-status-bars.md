# 017 — Widescreen status bars — final report

**Status:** local implementation complete; A4 CI and A6 physical Deck validation remain open.
Continued the paused WIP (`32fb9e3`, `5cf163b`), merged `main` through spec 016 and spec 018
(`e24f027`, plus status-only `f18eff4`), and finished local acceptance on 2026-10-17. No visible window was opened,
no screen was captured, no IWAD was fetched, and no downloaded WAD was executed.

## What changed

- R1/R2: nine optional `wide-status-bar` graphics components (five mapset bars, four IWAD
  bars), pinned to **`eb3773ce9e4563c69a092386bf2e4f8d740dfd4d`**, the upstream GPL-3.0
  licence commit. Exact package/member sizes and SHA-256 values are in each recipe.
- R3/R4: reviewed GitHub page/raw hosts; fetch lint parses bounded WAD directories and rejects
  non-status-bar lumps. Offline synthetic fixtures exercise clean bars, `DEHACKED`, map,
  music and sound rejection, malformed directories and the fetch-lint integration. Release
  labels reject `/` and `\` as decided by the coordinator.
- R5: all nine bars default on for fresh profiles. Existing choices persist; spec 018 copies
  retain a disabled bar and its new-default action enables it. The existing Play setup toggle
  and launch summary expose the choice and bar-last order.
- R6: About displays the complete NightFright/contributors/GPL-3.0 credit and repository URL,
  inside an expanded panel. README keeps spec 016's rewrite plus only the credit entry.
  Component descriptions and verification notes carry each recipe's credit (schema 1 has no
  dedicated credits field).
- R7: the e2e workflow installs all nine components with `--with wide-status-bar` and checks
  the final `-file` member for Eviternity and SIGIL. Running it on main belongs to A4.
- Added explicit headless assertions for the enabled, actionable toggle, credit text/link and
  panel bounds; inspected both off-screen PNGs. Added final argv assertions and profile-default
  integration assertions after merging spec 018. No new dependencies.

## Per-recipe coverage

**Confirmed** by pinned upstream tree/readme and release WAD directory inspection; the complete
pin, matching-game citation, R4 allowlist and per-mapset release-date comparison are in
[research §4.1](../research/widescreen-statusbars.md#41-spec-017-implementation-coverage-2026-10-17).
Each added bar contains only `STBAR`. The four IWAD-bar mapsets replace **none** of the R4
allowlist families. Upstream explicitly lists one `doom_uwide.wad` for Doom/Doom II in
[`readme.txt:50`](https://github.com/NightFright2k19/doom_widehud/blob/eb3773ce9e4563c69a092386bf2e4f8d740dfd4d/readme.txt#L50).

| Recipe | Decision | Upstream path | Basis |
| --- | --- | --- | --- |
| alien-vendetta | **none** | — | Only `pwad/av_uwide_fix.wad` exists and it carries `DEHACKED`+`MAPINFO`+`STBAR`, so the cosmetic-only lint rejects it; `AV.WAD` replaces `STBAR` → no IWAD bar. |
| ancient-aliens | mapset bar | `pwad/aaliens_uwide.wad` | `STBAR` only (23,570 B). |
| auger-zenith | **none** | — | No DBP37 file in the collection; `DBP37_AUGZEN.wad` replaces `STBAR`+`STARMS`+`STTNUM*`. |
| back-to-saturn-x-e1 | mapset bar | `pwad/btsx1_uwide.wad` | `STBAR` only (23,570 B). |
| back-to-saturn-x-e2 | mapset bar | `pwad/btsx2_uwide.wad` | `STBAR` only (23,570 B). |
| eviternity-ii | **none** | — | No Eviternity II file; `Eviternity II.wad` replaces the whole status bar. |
| eviternity | mapset bar | `pwad/eviternity_wide.wad` | `STBAR` only (19,962 B). |
| going-down | **IWAD bar** | `iwad/doom_uwide.wad` | No mapset bar; `gd.wad` defines no status-bar lump. |
| scythe | **IWAD bar** | `iwad/doom_uwide.wad` | No Scythe 1 bar (only `scythe2_uwide.wad` = Scythe II); `SCYTHE.WAD` defines no status-bar lump. |
| sigil | **IWAD bar** | `iwad/doom_uwide.wad` | No SIGIL bar; `SIGIL_V1_23.wad` defines no status-bar lump. |
| sigil-ii | **IWAD bar** | `iwad/doom_uwide.wad` | No SIGIL II bar; `SIGIL_II_V1_0.WAD` defines no status-bar lump. |
| sunlust | **none** | — | No Sunlust file; `sunlust.wad` replaces the whole status bar. |
| valiant | mapset bar | `pwad/valiant_wide.wad` | `STBAR` only (19,962 B). |


The five mapset-bar content dates post-date the recipe releases: Ancient Aliens 2024-08-05
vs 2023-03-16; both BTSX episodes 2022-03-11 vs 2020-08-13; Eviternity 2022-03-10 vs
2019-02-10; Valiant 2022-03-11 vs 2015-05. These are source/date checks, not gameplay validation.

## Verification

All commands ran on macOS arm64. Tests use fakes, loopback fixtures and scratch XDG roots;
no external network is used by tests. Explicit fetch lint verifies remote package identities
(or their verified cache) and all named archive members, including the cosmetic-only check.

```text
$ ./target/debug/ddl catalog lint catalog
Catalog lint passed (offline).

$ ./target/debug/ddl catalog lint --fetch catalog/ancient-aliens.toml catalog/back-to-saturn-x-e1.toml catalog/back-to-saturn-x-e2.toml catalog/eviternity.toml catalog/valiant.toml catalog/going-down.toml catalog/scythe.toml catalog/sigil.toml catalog/sigil-ii.toml
Catalog lint passed (fetched packages verified).

$ cargo test -p ddl-core widescreen_status_bar_loads_last_after_the_mapset
test resolver::tests::widescreen_status_bar_loads_last_after_the_mapset ... ok
test result: ok. 1 passed; 0 failed
```

The resolver test checks exactly one `-file`, followed by the main WAD and the bar, then
`-complevel`. Both main and SIGIL's compatibility replacement retain the final IWAD bar.
The final workspace run below also executes these tests.

```text
$ cargo fmt --all --check
(no output; exit 0)

$ cargo clippy --workspace --all-targets -- -D warnings
    Checking ddl-core v0.0.4
    Checking ddl v0.0.4
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.83s
(exit 0)

$ DDL_HEADLESS_DIR=/tmp/ddl-017-headless cargo test --workspace
test result: ok. 65 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 106.72s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 307 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit 0; 398 tests/doc tests passed in total)
```

A3 render command: `DDL_HEADLESS_DIR=/tmp/ddl-017-headless cargo test -p ddl
fixture_screens_navigation_and_focus_render_headlessly` (also included in the full suite).
Inspected `/tmp/ddl-017-headless/010-play-setup-ready.png`: the toggle reads **On**, and the
summary shows `SIGIL_V1_23.wad` then `doom_uwide.wad`. Inspected
`/tmp/ddl-017-headless/013-about.png`: the full author/licence credit and repository URL wrap
legibly inside the panel. These are Slint off-screen software renders at 1280×800.
Logs for this run are `/tmp/ddl-017-{fmt,clippy,tests,lint,fetch,order,headless-test}.log`.

## Open items and limitations

- **A4:** coordinator must run the e2e workflow on `main`; workflow edits and local checks
  do not establish a successful Linux e2e run.
- **A6 — Needs Deck Validation:** confirm the actual classic status bar fills the Deck's
  16:10 panel correctly in DSDA-Doom. Fullscreen HUD modes may hide the classic status bar.
- No unresolved implementation question. Credits use existing schema fields, with no schema
  extension; cosmetic checking happens when fetch lint has bytes, with offline fixture tests.
- The research retains the original licence/derivative-art discussion as historical evidence;
  the pinned GPL-3.0 update and coordinator's licence decision govern this implementation.
  Date evidence does not prove pixel-perfect compatibility with a particular mapset release.

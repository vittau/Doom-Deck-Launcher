# 017 — Widescreen status bars (NightFright's doom_widehud) — report

**Status: PAUSED by coordinator (2026-10-17).** Work on branch `vittau/spec-017-widehud` was
stopped mid-task at the coordinator's request. Commit `32fb9e3` is a WIP commit; nothing is
finished and no Deck validation has happened. This file is a handoff: what is done, what is left,
the coverage decisions already made, the pinned commit, and open doubts.

## Handoff

### Pinned upstream commit

**`eb3773ce9e4563c69a092386bf2e4f8d740dfd4d`** (2026-10-17, "Add GNU General Public License v3"),
the tip of `main` and the commit that added the GPL-3.0 `LICENSE`. It is later than the content
pin in `docs/research/widescreen-statusbars.md` (`78d962173334aecdb8bd974a1ba9bac95ab6cde4`), so
the same content plus the licence is referenced by every raw URL:
`https://raw.githubusercontent.com/NightFright2k19/doom_widehud/eb3773ce9e4563c69a092386bf2e4f8d740dfd4d/<path>`.

### Per-recipe coverage determined (all **Confirmed** by parsing the pinned tree)

WADs were inspected with the lump directory only; nothing was executed. "Replaces STBAR" means
the mapset's main WAD defines its own `STBAR`/status-bar graphics, so loading the base-game IWAD
bar would clobber the mapset's art and it is not offered (R2 guard).

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

Sizes/SHA-256 are computed from the pinned checkout (not invented); e.g. `aaliens_uwide.wad`
23,570 B `1bda814a7bcce46f5b1abbddfde5817e0471405ebb007134c84bc5351b6cde36`; `doom_uwide.wad`
23,652 B `e1d29d0ed7db6e7adbac97ffd34018cb35c26d8fb607ba7d57f7a1155f46d0c5` (matches the research
doc). `iwad/doom_uwide.wad` is offered for both Doom and Doom II: the upstream `readme.txt` lists
`Doom/Doom II ... doom_uwide.wad / doom_uwide_v2.wad` as one entry.

### Done (compiles; catalog/resolver tests passed at the time of pause)

- **R1** — five mapset bars (`ancient-aliens`, `back-to-saturn-x-e1`, `back-to-saturn-x-e2`,
  `eviternity`, `valiant`) as commit-pinned bare-WAD packages with one `graphics` member and one
  optional `wide-status-bar` component at position 2, label "Widescreen status bar".
- **R2** — four IWAD bars (`going-down`, `scythe`, `sigil`, `sigil-ii`) duplicated per recipe.
- **R3** — `catalog/hosts.toml`: `raw.githubusercontent.com` → `downloads`, `github.com` → `pages`.
- **R4** — `lint::wad_lump_names` + `lint::check_status_bar_lumps` (allowlist derived from the
  added WADs and DSDA v0.30.1's status-bar code), wired into `--fetch` for "status bar" components;
  unit + integration tests with tiny synthetic WAD fixtures (no network, no real WADs committed).
- **R4 (extra)** — `release` must not contain `/` or `\` (install path component); lint rule +
  test.
- **R5** — `Recipe::default_components()` and a seeded `default_profile` in the UI, so the bar is
  on by default for all nine recipes (coordinator confirmed IWAD bars default ON); resolver tests
  prove the bar is last in the single `-file` list.
- **R6** — About screen credit label, README credit line, and each affected recipe's
  `verification.notes` + component description credit NightFright and contributors (GPL-3.0).
- **R7** — `e2e.yml` installs each new component once with `--with wide-status-bar` and asserts
  the bar is the last `-file` entry for `eviternity` and `sigil`.
- `ddl catalog lint catalog` (offline) and `ddl catalog lint --fetch` on all nine changed recipes
  passed.

### Left / not finished

- **Headless render test (A3, R5/R6) is not green.** Paused while updating
  `crates/ddl/src/headless_production.rs`: the journey's synthetic SIGIL install now serves a
  synthetic `widehud` WAD offline (`status_bar_wad()`, `/wide.wad`), and the SIGIL assertions were
  updated for the default-on bar, but the final run had not completed when work stopped. Verify the
  "2 content files" / order assertions and re-run
  `DDL_HEADLESS_DIR=<scratch> cargo test -p ddl fixture_screens_navigation_and_focus_render_headlessly`,
  then inspect `010-play-setup-ready.png` and `013-about.png`.
- **Research doc not updated** — the coverage table above and the pinned commit must be recorded in
  `docs/research/widescreen-statusbars.md` (R1 explicitly asks for it).
- **Spec 017 acceptance boxes A1–A3, A5 not ticked.**
- **Full checks not run to completion:** `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. Only
  `cargo test -p ddl-core catalog`, `cargo test -p ddl-core resolver`, and the offline/`--fetch`
  catalog lint had passed before the pause.
- **A6 (Deck look)** remains *Needs Deck Validation* and was not attempted.

### Open doubts / decisions to confirm

- R5 for IWAD bars was ambiguous; the coordinator ruled **default ON**, with the extra guard that
  an IWAD bar is only offered when the mapset replaces **no** status-bar lump in the R4 allowlist
  (the four IWAD-bar mapsets were checked against the full allowlist, not just `STBAR`).
- The cosmetic-only check runs only in the `--fetch` path (offline lint cannot read WAD bytes);
  reading this as consistent with R4's "reads the WAD directory from a fixture in tests".
- "Each recipe's credits list the bar": the schema has no `credits` field, so the credit lives in
  the component description and `verification.notes` (plus the package `page`). Confirm that is
  the intended surface.
- The widehud `--fetch` cosmetic check confirms the *declared* bytes are status-bar-only; it does
  not re-verify that a mapset bar matches the exact shipped mapset release beyond the date evidence
  (upstream file dates post-date each recipe's release).

# 017 — Widescreen status bars (NightFright's doom_widehud)

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 002, 009, 010, catalog batch (Scythe…Eviternity II)
**Owner request (2026-10-16):** "suportar Widescreen statusbars … detectar o PWAD e já baixar e
aplicar o widescreen statusbar de acordo." Licence unblocked 2026-10-16: the collection is
GPL-3.0 ([LICENSE @ `eb3773ce9e`](https://github.com/NightFright2k19/doom_widehud/blob/eb3773ce9e/LICENSE),
[issue #6](https://github.com/NightFright2k19/doom_widehud/issues/6)); the author asks for credit.

## Goal

When a catalog mapset has a matching wide status bar in `doom_widehud`, DDL offers it as an
optional component — downloaded, verified and loaded in the right order like any add-on — so the
HUD fills the Deck's 16:10 screen. Research: `docs/research/widescreen-statusbars.md` (§3 engine
support, §5.2 Option A).

## Requirements

- **R1** — **Recipe components (Option A, no schema change):** for every catalog recipe that has a
  bar in the collection, a `[[packages]]` entry with a commit-pinned
  `raw.githubusercontent.com/NightFright2k19/doom_widehud/<commit>/…` URL, exact `size` and
  `sha256`, one `wad` member, and one optional `[[components]]` entry (`role = "graphics"`,
  label "Widescreen status bar", loaded after the mapset and any gameplay add-on). Pin one commit
  for all recipes and record it in the research doc. Cover the whole catalog as merged at the
  time (including the new batch); list per recipe "bar found / none" with the upstream path.
- **R2** — **IWAD bars:** where a recipe has no mapset-specific bar but plays on an IWAD whose bar
  is in the collection (`iwad/doom_uwide.wad`, Doom II's, etc.), offer that bar per research §5.2
  (a) — duplicated per recipe, no new global mechanism. Only if the bar is valid for that mapset
  (a mapset that replaces STBAR graphics gets no IWAD bar; check its lumps and say so).
- **R3** — **Hosts:** add `raw.githubusercontent.com` to `catalog/hosts.toml` `downloads` (and
  `github.com` to `pages` if a page link is used).
- **R4** — **Cosmetic-only lint:** a component with `role = "graphics"` whose members are status
  bars may contain only status-bar/HUD graphic lumps (allowlist from the research, e.g. `STBAR`,
  `STARMS`, `STF*`, `STT*`, `STG*`, `STK*`, `STYS*`, `STC*`, `STDISK`, `STCDROM`, `BRDR_*` — derive
  the exact list from the WADs you add); any `DEHACKED`, `MAPINFO`, `UMAPINFO`, map, music or
  sound lump fails lint. Lint reads the WAD directory from a fixture in tests (no network).
- **R5** — **Default and UI:** the component defaults to **on** for recipes whose bar targets the
  current release of that mapset (cite how you checked), otherwise off; the existing component
  toggle shows it on Play setup with the load order (D-010). No new screen.
- **R6** — **Credit:** About and README credit "Widescreen status bars: doom_widehud by
  NightFright and contributors (GPL-3.0)" with the repo link; each recipe's credits list the bar.
- **R7** — **e2e:** `e2e.yml` installs each new component once (`--with`), like AUGER's add-ons.

## Out of scope

Automatic hash matching of arbitrary PWADs (research §5.1, rejected). A global "cosmetic add-on"
schema. Bars for mapsets not in the catalog.

## Acceptance criteria

- [ ] **A1** (R1–R3) — recipes lint clean; per-recipe coverage table in the report and the
  research doc; dry-run argv shows the bar last in `-file` when enabled.
- [ ] **A2** (R4) — lint tests: a cosmetic WAD with `DEHACKED`/map lumps is rejected; a clean one passes.
- [ ] **A3** (R5, R6) — headless render of Play setup with the toggle; About shows the credit.
- [ ] **A4** (R7) — e2e run on `main` passes with the new components (coordinator triggers).
- [ ] **A5** — fmt, clippy `-D warnings`, tests green.
- [ ] **A6** — *Needs Deck Validation:* bar fills the 16:10 screen correctly in DSDA.

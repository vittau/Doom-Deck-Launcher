# 037 — Every Doomer Boards Project in the catalog

**Status:** In progress  ·  **Milestone:** M3  ·  **Depends on:** 002, 014, 015, 023, 035 (rule), D-010, D-028
**Origin:** owner, 2026-10-19: "Eu quero que todos os PWADs do catálogo do Doomer Boards estejam no
DDL: https://doomer.boards.net/page/dbp. AUGER;ZENITH já está." Dropbox links are allowed as a
fallback, "mas se conseguir achá-los no idgames seria ideal" (D-028).

## Facts gathered by the coordinator (2026-10-19)

- The live page sits behind a proof-of-work bot challenge; a Wayback snapshot works:
  `https://web.archive.org/web/20260929225359id_/https://doomer.boards.net/page/dbp` lists
  **DBP 01–99**, each linking to its project thread on `doomer.boards.net`.
- The /idgames index (`https://www.gamers.org/pub/idgames/ls-laR.gz`) holds about 30 of them
  (e.g. `levels/doom2/Ports/d-f/dbp01.zip` … `dbp50.zip`, `levels/doom2/Ports/megawads/dbp37_augzen.zip`
  (already in the catalog), `dbp47.zip`, and some under `themes/xmas/` and `themes/chex/`).
- doomworld.com returns Cloudflare 403 to automation; use idgames mirrors (gamers.org and the
  others in `catalog/hosts.toml`) and Wayback snapshots of forum threads.

## Requirements

- **R1** — **Inventory:** `docs/research/doomer-boards-projects.md` lists every DBP 01–99 with:
  title, thread URL, release date, IWAD, target port / format (vanilla, Boom, MBF21, UDMF/ZDoom…),
  /idgames path if published there, other download links (Dropbox etc.), and a **DSDA-Doom
  verdict**: playable in DSDA-Doom v0.30.1 (with the complevel it needs) or not (GZDoom-only,
  UDMF, ZScript, missing download), each with its evidence link. Label claims Confirmed /
  Proposed / Needs Deck Validation. Never invent hashes or complevels.
- **R2** — **Recipes:** one `catalog/<id>.toml` per DSDA-playable DBP, following existing recipes
  and spec 002: sources with SHA-256 and size from a real download, files and load order,
  IWAD, complevel (Proposed unless the text file states it), credits, tags from
  `catalog/tags.toml` (add the tag "Doomer Boards Projects" or reuse "Community Projects"),
  verification evidence. Source priority (D-028): an /idgames mirror first; Dropbox (`?dl=1`)
  only when the project is not on /idgames; never another host without asking.
- **R3** — **Hosts:** add only the hosts actually used. Dropbox needs `www.dropbox.com` and its
  redirect target `dl.dropboxusercontent.com` in `downloads` (check the real redirect chain and
  the downloader's redirect rules); thread pages go in `pages` if not already there.
- **R4** — **Screenshots:** each recipe gets 1–2 in-game screenshots per spec 023 (source and
  credit in `SOURCES.toml`), saved as JPEG at most 1280×800, quality ≈70 (spec 035 rule). Thread
  or /idgames screenshots are preferred; if none can be found, leave the recipe without one and
  list it in the report (the painted fallback shows).
- **R5** — **Waves:** commit in batches of about 10 recipes; each batch passes `ddl catalog lint`
  with real fetches before the next starts. DBPs that are not DSDA-playable get no recipe; the
  inventory says why.

## Acceptance criteria

- [ ] **A1** (R1) — inventory complete for 01–99 with evidence links.
- [ ] **A2** (R2, R3, R5) — lint with real fetch passes for every new recipe; `cargo test
  --workspace` green (update any recipe-count test as the union).
- [ ] **A3** (R4) — screenshots present or listed as missing.
- [ ] **A4** — the coordinator runs the Linux e2e (`gh workflow run e2e.yml`) on the merge.

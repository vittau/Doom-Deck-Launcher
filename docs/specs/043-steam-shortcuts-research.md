# 043 — Research: optional Steam shortcuts for installed mapsets

**Status:** Ready  ·  **Milestone:** M4  ·  **Depends on:** 013, D-002
**Origin:** OVERVIEW §13 M4 ("Optional Steam shortcuts … after feasibility investigation").
Owner, 2026-10-19: advance M4 while Deck testing waits. AGENTS.md: no Steam Input code of any
kind — this research is about library shortcuts and artwork only.

This is a **feasibility research** task. No code.

## Requirements

- **R1** — **Mechanism:** how a non-Steam shortcut is added on SteamOS: `userdata/<id>/config/
  shortcuts.vdf` (binary VDF) and its fields, app-id derivation for artwork in `config/grid/`,
  whether Steam must be closed (it rewrites the file), what Gaming Mode does on the next start,
  and any supported alternative (e.g. `steam://` URLs, Steam's own "Add a Non-Steam Game").
  Cite upstream sources (Steam, Valve docs, maintained open-source tools such as Steam ROM
  Manager or BoilR) with pinned versions.
- **R2** — **Fit for DDL:** what a shortcut for one mapset would launch (`ddl play <id>
  --profile …`, spec 014 CLI), how DDL could add/remove it safely (atomic write, backup, never
  while Steam runs), multi-user/`userdata` selection, SD card, and the risks (file corruption,
  Steam updates changing the format).
- **R4** — **SteamGridDB** (owner, 2026-10-19: "quero que as artes apareçam no SteamGridDB"):
  how DDL's own artwork (spec 013 / D-029 Steam set in `docs/design/logo/steam/`: grid/capsule,
  wide capsule, hero, logo, icon) gets listed on <https://www.steamgriddb.com> for the
  non-Steam "DoomDeckLauncher" entry so tools that search it (the Decky SteamGridDB plugin,
  Steam ROM Manager) find it. Cover: whether a non-Steam app can have a game page there and how
  one is created, the upload path (web upload or API, account requirements), size/format/style
  rules for each asset type, and whether our exports already meet them (list any re-export
  needed). A search for `DDL`, `ddl` or `DoomDeckLauncher` (the shortcut name) must lead to the
  art: cover how SGDB matches names (aliases, case, short-name collisions) and how the Decky
  plugin and Steam ROM Manager pick a game for a shortcut name. Uploading is an owner action on the owner's account: write the step-by-step for them;
  do not create accounts or upload anything.
- **R3** — **Recommendation:** go / no-go / "README only" with reasons, and if go, a proposed
  spec (requirements + acceptance) in the report.

## Acceptance criteria

- [ ] **A1** — `docs/research/steam-shortcuts.md` covers R1–R2, every claim labelled
  **Confirmed** / **Proposed** / **Needs Deck Validation**, sources linked and pinned.
- [ ] **A2** — `docs/reports/043-steam-shortcuts-research.md` with R3 and the SteamGridDB
  upload steps for the owner (R4).

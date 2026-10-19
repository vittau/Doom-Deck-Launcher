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
- **R3** — **Recommendation:** go / no-go / "README only" with reasons, and if go, a proposed
  spec (requirements + acceptance) in the report.

## Acceptance criteria

- [ ] **A1** — `docs/research/steam-shortcuts.md` covers R1–R2, every claim labelled
  **Confirmed** / **Proposed** / **Needs Deck Validation**, sources linked and pinned.
- [ ] **A2** — `docs/reports/043-steam-shortcuts-research.md` with R3.

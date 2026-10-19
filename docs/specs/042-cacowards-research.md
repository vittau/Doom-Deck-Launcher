# 042 — Research: Cacowards inventory for the curated catalog

**Status:** Ready  ·  **Milestone:** M4  ·  **Depends on:** 002, 015, 037, D-010, D-028
**Origin:** OVERVIEW §13 M4 ("Curated Cacowards … selections where rights/source details
permit"); owner, 2026-10-19: advance M4 while Deck testing waits. Spec 015 defined the
`cacowards` tag and its citation rule.

This is a **research** task. No recipes yet: they come in a later spec, in waves, after 037 and
038 are merged.

## Requirements

- **R1** — **Inventory** every Cacoward *winner* (Top 10 and named special awards that are
  mapsets) from 1994 retro years through the latest edition, from Doomworld's Cacowards pages via
  Wayback `id_` snapshots (doomworld.com blocks automation). For each: year, title, author(s),
  source page link.
- **R2** — **Availability and fit:** for each entry, the /idgames path (from the mirrors'
  `ls-laR.gz`) or other host, size, target port and IWAD from its txt, and a DSDA-Doom verdict:
  playable (limit-removing, Boom, MBF, MBF21 — with the evidence), not playable (needs GZDoom,
  Eternity, ZDoom features…), or unknown. Never guess a complevel.
- **R3** — **Already in DDL:** mark entries that already have a recipe (`catalog/*.toml`,
  including the Doomer Boards set from 037) and whether their recipe carries the `cacowards` tag
  with a citation.
- **R4** — **Proposed waves:** order the playable, not-yet-cataloged entries into waves of ~10
  for a later recipe spec, with any host that would need allowlisting.

## Acceptance criteria

- [ ] **A1** — `docs/research/cacowards.md` with the full table (R1–R3), sources dated, verdicts
  labelled **Confirmed** / **Proposed**.
- [ ] **A2** — `docs/reports/042-cacowards-research.md` with R4, the tag-citation gaps for
  existing recipes, and open questions.

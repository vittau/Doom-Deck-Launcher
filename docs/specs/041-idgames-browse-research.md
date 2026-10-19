# 041 — Research: browsing /idgames as an unverified tier

**Status:** Ready  ·  **Milestone:** M4  ·  **Depends on:** 002, 006, 019, D-010
**Origin:** OVERVIEW §13 M4 ("`/idgames` search and browse through a separate unverified metadata
tier"). Owner, 2026-10-19: advance M4 work while Deck testing waits. Prior facts:
`docs/research/upstream-sources.md` §3 — the Doomworld idGames API sits behind a Cloudflare
challenge (HTTP 403) and is not reliably machine-callable; it has no hashes.

This is a **research and proposal** task. No code.

## Requirements

- **R1** — **Data source:** compare the realistic ways to get /idgames metadata without the API:
  the mirrors' `ls-laR.gz`, per-file `.txt` files on mirrors already in `catalog/hosts.toml`, and
  a DDL-built index published like the catalog bundle (spec 019, `catalog` branch) by a CI job.
  For each: size, freshness, what fields it gives (title, author, date, size, IWAD, port line of
  the txt), licensing/terms, failure modes. Measure real sizes and timings; cite every URL with a
  date. Recommend one.
- **R2** — **What "unverified" means:** propose how DDL infers a launch from /idgames data alone
  (IWAD from directory and txt, complevel only from evidence, never guessed; files that need
  GZDoom/ZDoom features flagged), and what the player sees: labels, warnings, what is disabled
  (e.g. no "verified" badge, no screenshots unless the txt/zip ships one, pistol-start off).
  It must not imply compatibility (OVERVIEW M4 exit criterion).
- **R3** — **Safety:** how untrusted archives are handled when no hash is pinned (size caps,
  trust-on-first-download hash recording, extraction rules from spec 006, never executing
  anything), and how an unverified entry can be promoted to a catalog recipe by PR.
- **R4** — **UX sketch in words:** where browse/search lives (Discover tab section or its own
  tab), controller-only search (Steam keyboard, spec 031 OSK), paging, offline behaviour.
- **R5** — **Proposed specs:** split the build into 2–4 implementation specs with requirements
  and acceptance criteria, in the report, for the coordinator to write.

## Acceptance criteria

- [ ] **A1** — `docs/research/idgames-browse.md` covers R1–R4 with every claim labelled
  **Confirmed**, **Proposed** or **Needs Deck Validation**, sources linked and dated.
- [ ] **A2** — `docs/reports/041-idgames-browse-research.md` holds R5 and the open questions for
  the owner (if any), each with a recommendation.

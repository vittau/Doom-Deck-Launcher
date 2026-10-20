# 052 — Unverified install and launch

**Status:** Ready (after 051)  ·  **Milestone:** M4  ·  **Depends on:** 006, 009, 051, D-030
**Origin:** spec 041 research (`docs/research/idgames-browse.md`, report R5), 2026-10-19; owner: advance M4 while Deck testing waits.

**Goal:** download and launch an indexed mapset honestly and safely, with no pinned hash and no
reviewed recipe.

## Requirements

- **R1** — URL construction from a **per-host base path** table (gamers/youfailit/infania/
  braindrainlan `/pub/idgames`, fu-berlin `/pc/games/idgames`, lethe/mtu `/idgames`, planetzdoom
  `/idgames`); allowlist from `catalog/hosts.toml`; redirects re-checked (add
  `idgames.planetzdoom.com` or drop planetzdoom). `files.xvertigox.com` is not used.
- **R2** — download with `expected_size` from the index, 512 MiB hard cap, 100 MB warning, progress,
  cancel, cache keyed by SHA-256 (spec 006).
- **R3** — TOFU as described in the research §7.2: record the first hash, refuse later mismatches,
  treat a changed date/size as a new version and ask, abort on mirror disagreement.
- **R4** — extraction allowlist (`.wad`, `.deh`, `.bex`, `.txt`/`.nfo`; never executables), spec 006
  limits, hash-and-record every kept member; no symlinks, no execution.
- **R5** — launch plan: IWAD from path (+ TNT/Plutonia override, conflict = "unclear"), one main WAD
  (largest) plus optional extra WADs off by default, `-deh` order, COMPLVL detected after
  extraction (omit `-complevel`), author-declared family mapped with **D-030 aliases**
  (`vanilla`/`boom`/`mbf`/`mbf21`, shown with the resolved number), otherwise omit and show
  "unknown"; other-port-required entries blocked; no pistol start; no `-noautoload` change beyond
  the recipe defaults.
- **R6** — `ddl idgames draft <path>` writes a lint-clean recipe TOML with `status = "indexed"`,
  computed hashes, member list, inferred IWAD and evidence quotes, for a human-reviewed PR.

## Acceptance criteria

- [ ] **A1** (R1) — base-path/redirect/allowlist tests with a fake server.
- [ ] **A2** (R2, R3) — size cap, TOFU match/mismatch, changed date/size, mirror disagreement.
- [ ] **A3** (R4) — adversarial zip fixtures (executables, traversal, bomb, multi-WAD, no-WAD) and the
  selected-member assertions.
- [ ] **A4** (R5) — launch-plan tests: COMPLVL present, `MBF21` txt, no evidence, ZDoom-only, TNT
  override, multi-WAD confirmation; arguments built as separate argv (never a shell string).
- [ ] **A5** (R6) — draft lints clean and round-trips through the recipe model.
- [ ] **A6** — fmt, clippy `-D warnings`, tests green; Deck smoke of one unverified launch
  (**Needs Deck Validation**).

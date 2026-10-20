# 041 — /idgames browse research: report

**Status:** Done  ·  **Spec:** `docs/specs/041-idgames-browse-research.md`
**Deliverable:** `docs/research/idgames-browse.md` (R1–R4) — research and proposal only, no code.

## What changed

- **Added `docs/research/idgames-browse.md`.** It compares the three realistic mirror-based data
  sources, measures them on 2026-10-19, and recommends a DDL-built index published like the spec 019
  catalog bundle, with per-file `.txt` fetched on demand. It then defines the `Indexed` status, IWAD
  and complevel inference, port gating, download/extraction/TOFU safety, recipe promotion and a
  controller-only UX sketch.
- **Added this report** with the R5 build-spec split and the owner questions.
- No code, catalog recipe, workflow, decision, status or journal file was touched. Research scratch
  data stayed in `target/research-041/` (git-ignored) and is not committed; nothing was executed.

## Verification of the acceptance criteria

### A1 — research file covers R1–R4, every claim labelled, sources linked and dated

| R | Where | Evidence |
| --- | --- | --- |
| R1 data source | §2–§5 | measured `ls-laR.gz` (419,389 B / 2.79 MB / 43,741 files / daily 05:07 UTC) and `fullsort.gz`; 40-file sidecar sample (mean 3.2 KB, 0.64 s, 95 % title/author, 87.5 % game, 47.5 % engine); index-size and build-cost estimates; terms from `README`, `README.legal`, `README.INCOMING`, `robots.txt`; failure modes measured (Cloudflare 403, master burst timeouts, per-host base paths, mirror lag) |
| R2 “unverified” | §6 | `VerificationStatus::Indexed` (existing model) + OVERVIEW §7 stage; IWAD from path + txt; COMPLVL / author-family mapped to D-030 aliases / unknown policy (DSDA default 21 cited from `dsda-engine.md`); port gating table with quoted sample txts; what is shown/disabled |
| R3 safety | §7 | size cap 512 MiB from the measured distribution (max 426,662,505 B), TOFU keyed on path+size+date, spec 006 extraction rules plus a WAD/DEH/TXT allowlist, no execution, main-WAD selection, promotion by reviewed PR |
| R4 UX | §8 | Discover section + pushed screen (four tabs kept), Steam OSK search, typeless filters, 50-row paging, offline behaviour and copy |

All claims carry **Confirmed / Proposed / Needs Deck Validation**. Sources are linked inline with
the observation date (2026-10-19) or the Wayback `id_` capture date (`doomworld.com/idgames/`
2026-09-26, API doc 2025-08-16, terms 2025-06-22).

Key measurements (all 2026-10-19):

- **Confirmed:** Doomworld page and API still return 403 `Just a moment...`; the mirrors do not.
- **Confirmed:** every upload must ship a same-named `.txt` outside and inside the zip
  (`README.INCOMING`), and 15,054/15,063 `levels/` zips have one (10,877/10,885 DOOM-family
  single-player).
- **Confirmed:** title/author parsing is good (95 % of the sample), game 87.5 %, engine requirement
  only 47.5 %, and free-text `complevel` is unsafe (a `-complevel 17` demo hint is not the map's
  target).
- **Confirmed:** the index copies are byte-identical on seven mirrors; the master rate-limits bursts
  (≈ 10 min of timeouts after ≈ 20–30 requests), so a builder must use a mirror, not gamers.org.

### A2 — report holds R5 and the open questions

Below.

## R5 — proposed implementation specs

Four specs, in dependency order. Numbers follow 046 (taken on `main` by the complevel-alias spec);
the coordinator may renumber.

### 047 — `/idgames` index build and publish

**Goal:** one small, versioned index that gives DDL titles, authors, dates, sizes, base game and a
port hint for the archive, built from `ls-laR.gz` + sidecar txts, without the Doomworld API.

**Requirements (proposed):**

- **R1** — `ddl-core::idgames` builder: parse `ls-laR.gz` (and accept `fullsort.gz` as an
  alternative inventory); keep only `levels/doom` and `levels/doom2`; drop `/deathmatch/`,
  `incoming/`, `newstuff/`, heretic/hexen/strife/hacx/doom64; parse the lossy `ls` date (year
  inference for recent files, server-local, day precision).
- **R2** — fetch the same-stem `.txt` for each entry with a bounded GET (64 KiB), ETag/
  `If-Modified-Since`, UTF-8 → Windows-1252/CP437 fallback; parse `Title`, `Author`, `Game`/
  `Episode and Level #`, `Advanced engine needed`, `May Not Run With`, `Primary purpose`,
  `Map #`; a bad/missing txt keeps the entry with a filename fallback and is counted, never fatal.
- **R3** — output `index.json`: `schema`, `generated_at`, `source {index URL, index sha256, mirror}`,
  `min_ddl_version`, `entries` grouped by directory (compact arrays or short keys), carrying stem,
  title, author, date, size, game, port class and txt flag. Deterministic ordering; **no
  descriptions, credits, ratings or txt bodies**; ≤ 2 MiB; atomic write.
- **R4** — CLI: `ddl idgames index build --ls <file> --mirror <host> --out <file>
  [--previous <old.json>]`, `ddl idgames index validate <file>`. Incremental mode re-fetches only
  added/changed entries (path+size+date).
- **R5** — workflow `.github/workflows/idgames-index.yml`: daily schedule + `workflow_dispatch`;
  builds/validates, publishes `index.json` to the `idgames-index` branch as one ordinary commit,
  keeps the last good index on failure, `contents: write` only for this job; no force push.
- **R6** — politeness: one mirror at a time, ≤ 4 requests in flight, `DoomDeckLauncher/<version>`
  UA, stop on 403/429/5xx, never crawl the master for the bulk pass.

**Acceptance criteria (proposed):**

- **A1** (R1–R3) — fixture `ls-laR` snippets + txt fixtures: exclusion rules, recent-date year
  inference, missing/malformed/non-UTF-8 txt, field extraction, schema round-trip, size bound,
  deterministic order. No network.
- **A2** (R4) — CLI build/validate round-trip; incremental build fetches only the diff (fake
  fetcher counts requests).
- **A3** (R5) — workflow reviewed; first publish triggered by the coordinator after merge and the
  resulting index size/entry count recorded.
- **A4** — fmt, clippy `-D warnings`, tests green.

### 048 — `/idgames` browse core (fetch, cache, search)

**Goal:** the app can list, search and page the archive offline from the cached index, and fetch a
single `.txt` on demand for the entry page.

**Requirements (proposed):**

- **R1** — typed schema + validation: known `schema`, `min_ddl_version` ≤ running version,
  `generated_at` parseable, entries sane (path under `levels/doom{,2}/`, no `..`, size > 0, hash
  format when present), ≤ 2 MiB; malformed index rejected whole.
- **R2** — fetch `…/idgames-index/index.json` over HTTPS with the existing download stack; ETag +
  `If-Modified-Since`; cache atomically at `<XDG data>/ddl/idgames/index.json` with fetch time;
  background refresh ≤ once per 24 h; manual refresh; a failed refresh keeps the old index; a
  corrupt cache is ignored and logged.
- **R3** — search API: case/diacritic-insensitive substring over title, author and filename
  (2-character minimum), filters (game, era, port class, txt availability), sort (newest, title,
  size), offset/limit paging with a deterministic order; pure `ddl-core`, no network.
- **R4** — entry detail: on-demand `.txt` fetch (64 KiB, ETag, encoding fallback) with a small
  on-disk cache; the list never blocks on it; the parsed fields and the raw permission text are
  returned to the UI.
- **R5** — offline and stale-index behaviour are explicit (age shown, no empty list).
- **R6** — tests use the local HTTP server only.

**Acceptance criteria (proposed):**

- **A1** (R1) — fixtures for each rejection case; partial acceptance is not allowed for a
  malformed index.
- **A2** (R2) — loopback server: 200, 304, 403, oversized, corrupt JSON, cache precedence, 24 h
  throttle with a fake clock.
- **A3** (R3) — fixture index: search, diacritics, filters, paging boundaries, stable order.
- **A4** (R4) — txt fetch/cache/encoding and offline fallback.
- **A5** — no network in tests; fmt, clippy `-D warnings`, tests green.

### 049 — Unverified install and launch

**Goal:** download and launch an indexed mapset honestly and safely, with no pinned hash and no
reviewed recipe.

**Requirements (proposed):**

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

**Acceptance criteria (proposed):**

- **A1** (R1) — base-path/redirect/allowlist tests with a fake server.
- **A2** (R2, R3) — size cap, TOFU match/mismatch, changed date/size, mirror disagreement.
- **A3** (R4) — adversarial zip fixtures (executables, traversal, bomb, multi-WAD, no-WAD) and the
  selected-member assertions.
- **A4** (R5) — launch-plan tests: COMPLVL present, `MBF21` txt, no evidence, ZDoom-only, TNT
  override, multi-WAD confirmation; arguments built as separate argv (never a shell string).
- **A5** (R6) — draft lints clean and round-trips through the recipe model.
- **A6** — fmt, clippy `-D warnings`, tests green; Deck smoke of one unverified launch
  (**Needs Deck Validation**).

### 050 — Browse UI and promotion hand-off

**Goal:** a controller-only, couch-readable browse/search flow that never implies compatibility.

**Requirements (proposed):**

- **R1** — Discover section + pushed screen; four root tabs unchanged; tab bar disabled on the
  browse/detail screens; B = Back (spec 030).
- **R2** — controller: Y opens the Steam OSK search (spec 031) with the physical-keyboard fallback,
  typeless filters, 50-row pages via L1/R1 or "Load more", focus restored to the previous row.
- **R3** — entry page as in research §6.5: "Unverified" chip, author/date/size/source, engine quote,
  IWAD availability, no rating/screenshots/verified badge, other-port-required = Play disabled,
  warning sheet before the first download.
- **R4** — progress, cancel, offline/stale states, index-missing state; errors are specific
  (offline, host refused, size changed, hash mismatch, missing IWAD).
- **R5** — every string in `strings.slint` (English), no process jargon (spec 045); headless renders
  only.
- **R6** — "Export recipe draft" action for installed unverified entries, with the CLI instructions
  and a lint-clean draft path.

**Acceptance criteria (proposed):**

- **A1** (R1, R3) — headless renders of the browse list, entry page with each gating case, and the
  warning sheet; reviewed by the coordinator.
- **A2** (R2) — navigation test: open browse, type via a fake OSK, filter, page, open entry, back
  restores focus.
- **A3** (R4) — fake-download progress/cancel/error renders; offline and no-index states.
- **A4** (R6) — draft action test with a fake index/install.
- **A5** — fmt, clippy `-D warnings`, tests green; Deck check of OSK/focus/readability
  (**Needs Deck Validation**).

## Proposed decision and allowlist changes (not edited here)

- **New decision (proposed, for `docs/decisions.md`):** *`/idgames` unverified tier.* The Doomworld
  API is never used; DDL builds a derived index from mirror listings and sidecar txts, publishes it
  on a CI branch, marks entries `Indexed`, never claims compatibility, downloads via mirrors with
  size + TOFU hashes, extracts only WAD/DEH/TXT, and promotes only through a reviewed catalog PR.
- **`catalog/hosts.toml` (proposed):** add `idgames.planetzdoom.com` (redirect target) or drop
  planetzdoom; keep an explicit per-host base path in code (not in the allowlist); stop using
  `files.xvertigox.com` for automation (Cloudflare 403), though it may stay listed.

## Open questions for the owner (with recommendations)

1. **Publish a derived index at all?** Recommended **yes**: a small daily `index.json` on an orphan
   `idgames-index` branch, same trust model as D-025 (HTTPS, no signature yet). It is the only way
   to get search without making every Deck crawl 10 885 files or depend on the blocked API.
2. **Scope.** Recommended **DOOM-family single-player only**: 10,885 entries under `levels/doom`
   and `levels/doom2`, excluding `/deathmatch/` and heretic/hexen/strife/hacx/doom64. That matches
   DDL's four base games and keeps the index under 2 MiB.
3. **Entries that need ZDoom/GZDoom/EDGE (or another port):** keep them browsable but disable
   Download & Play, showing the author's quoted requirement. Recommended; alternative is to hide
   them entirely, which loses discovery and the "known incompatible" state OVERVIEW asks for.
4. **Complevel when there is no evidence.** D-030 already fixes this for recipes (author's
   TXT/COMPLVL/UMAPINFO, else the port-family convention, expressed as aliases). Recommended for
   the unverified tier: apply the same mapping when the author declares a family; when there is no
   family evidence at all, omit `-complevel` and say compatibility is unknown (DSDA then uses
   COMPLVL or its default 21), with an advanced override. Do **not** pass a number silently.
5. **Crawl permission and load.** Recommended: ask the owner to contact the archive's mirror
   coordinator (`ftpadmin@gamers.org`, per `README.mirrorers`) and the chosen mirror's admin before
   the first full pass; then crawl daily incrementally (≈ 2–3 files/day), 2–4 requests in flight,
   descriptive UA, never the master. Also ask Doomworld about its API terms for completeness (it
   remains unused either way).
6. **Legal sign-off for derived metadata.** Recommended: store facts only (path, size, date, title,
   author, port flag), no descriptions/credits/txt bodies; credit the archive and author and link
   the canonical page; publish it under the repo's licence unless the owner prefers a data-only
   notice. This needs an owner/legal decision because it is a new public artefact.
7. **Player-facing wording.** Recommended: "Unverified" chip + "From the /idgames archive. DDL has
   not tested this mapset." Avoid "Indexed"/"Inspected"/"Needs Deck Validation" in the UI.
8. **Index refresh cadence.** Recommended: build daily; the app refreshes at most once per 24 h and
   keeps the old index on failure. Weekly would also work (2–3 uploads/day) but makes new releases
   feel stale.
9. **Allowlist/base paths.** Recommended: add `idgames.planetzdoom.com` or drop planetzdoom; stop
   automated use of `files.xvertigox.com`; keep per-host base paths in code; never default to
   `www.gamers.org` for bulk fetches.

## New dependencies

None. This task is research only. The proposed specs need no new crates: the existing download
stack, `serde`/`serde_json`, and stdlib gzip are enough (the index is plain JSON; a compact
grouped-array schema avoids a compression dependency). If a later spec chooses gzip, prefer the
already-vendored dependency chain per spec 006.

## Spec items left open

- A1 and A2 are satisfied by this document and the report; the spec's requirement text is fully
  covered.
- Nothing in the spec asked for code; no Rust build was run (not needed), so the normal full-check
  gate is not applicable to this task. `git status` shows only the two new docs.
- Deck validation items are explicitly listed as **Needs Deck Validation** in the research
  (OSK/focus, default-21 launch behaviour, COMPLVL detection, external link in Gaming Mode,
  readability, one end-to-end unverified launch and one recipe promotion).

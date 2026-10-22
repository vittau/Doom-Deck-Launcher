# 103 — Fill missing mapset screenshots from real public sources

**Status:** Implemented — 12 of 26 added, 14 unresolved · **Depends on:** 023, 035, 037
**Origin:** owner, 2026-10-22: try to find screenshots for mapsets that lack them,
using an OpenCode worker.

## Goal and baseline

Reduce the existing 26 screenshot-less recipes with representative real in-game images.
Inventory the checked-out catalog before editing and try each missing recipe. Existing
screenshots are not replacements for another mapset or edition. No generated gameplay
images, video/game execution, IWAD download or local screen capture.

Baseline missing IDs: caustic-industry, deathless, doom-ii-the-way-id-did, going-down-turbo,
headless-chicken, hell-revealed-3-pt-1, japanese-valentines, jolly-junkyard, lunatic,
memento-mori, memento-mori-ii, micro-slaughter-community-project, mindblood-genesis,
no-end-in-sight, plutonia-2, rekkr, santas-outback-bender, scythe-ii,
spaceballs-the-quest-for-more-dbp-maps, speed-of-doom, sunder, terminal-voltage,
the-10x10-project, tnt-revilution, tomatomania and vanguard.

## Requirements

- **R1 — Real, attributable sources.** Prefer official author/release pages and their
  image attachments, then established community mapset pages with clear provenance.
  Verify the image depicts the specific mapset/edition; inspect downloaded images.
  Record exact image URL, contextual source page, truthful creator/source credit,
  original dimensions, retrieval date and permission text (or honestly that none was
  found) in the existing `SOURCES.toml` schema, following D-024 and spec 023. Do not
  invent permission, attribution, gameplay facts or compatibility claims. Do not post
  to external channels or request permissions on the owner's behalf.
- **R2 — Lean bundled assets.** Add one useful screenshot per previously missing recipe
  when found, using the established JPEG-only assets and recipe `[[screenshots]]` schema.
  Use `scripts/lean-image.sh`/existing deterministic tools, quality around 60, preserve
  aspect ratio, no enlargement, at least the lint's minimum card dimensions and at most
  1280×800 / 192 KiB. Do not replace existing good images or add decorative thumbnails.
- **R3 — Bounded investigation.** Download only public image assets with bounded size,
  timeouts and retries. Never execute downloaded archives or launch the game/engine.
  If blocked, ambiguous, too small, or genuinely unavailable after reasonable official
  and community searches, leave the recipe unchanged and record sources tried and why.
  Do not silently omit any baseline recipe from the outcome inventory.
- **R4 — Proportionate validation.** Verify asset dimensions/format/size, source records,
  references and the offline catalog lint once on the final batch. Use available existing
  binaries/caches; do not run full Cargo fmt/Clippy/tests, exhaustive visual matrices,
  network e2e or SDK builds for image/TOML additions. Produce a local contact sheet of
  downloaded game images for coordinator review; it is not a desktop screen capture.
- **R5 — Durable report.** In `docs/reports/103-missing-mapset-screenshots.md`, list each
  of the 26 IDs as added or unresolved with links/reasons, credit evidence, final asset
  byte totals, validation commands and remaining questions. No new dependency.

## Ownership and boundaries

One supervised OpenCode worker owns only the baseline recipes' screenshot entries,
their new `crates/ddl/ui/assets/screenshots/<id>/` image/source records, this spec's status
and its report. A contact sheet may live in ignored `.scratch/`. Do not change packages,
hashes, engine compatibility, recipe ordering/tags, hosts, Rust/Slint, versions, workflows,
status/journal/decisions/overview or existing release fixtures. A Codex worker separately
owns scroll/Library UI code (102). Merge main before final focused lint; commit small
changes. Never push, tag, release, spawn another worker, open visible windows or capture
the desktop. Send worker_done with the honest achieved coverage and unresolved searches.

# 023 — Representative screenshots on mapset cards

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 002, 006, 012, 013 · **Decision:** D-024 (Proposed)
**Owner request (2026-10-17):** "os cards dos PWADs devem mostrar alguma screenshot
representativa do mod, e não 'original concept art'."

## Goal

Each mapset card, detail page and art band shows a real screenshot of that mapset, credited to
its source, so players recognise what they are choosing. The painted concept art stays only as
the fallback when no screenshot is available or downloaded yet.

## Decision D-024 (Proposed)

Screenshots are third-party material (the mapset authors' art and levels). DDL does **not**
commit them to this repository. A recipe **references** screenshots on official or
author-controlled sources (the author's site or release page, the project's repository, the
/idgames mirrors, ModDB/Doomworld attachments only if fetchable), each with exact size, SHA-256
and a credit line, exactly like packages (D-010, D-015); DDL downloads them on demand into its
cache, verifies them and shows them with the credit. Where a mapset's text grants permission to
redistribute screenshots, the recipe notes say so. Hosts go through `catalog/hosts.toml`.

## Requirements

- **R1** — **Recipe field** (schema 1 compatible, optional): `[[screenshots]]` with `url`
  (HTTPS, allowlisted `images` host list in `hosts.toml`), `size`, `sha256`, `width`, `height`,
  `credit` (who made the mapset / who took the shot), `source_page` (where a human can see it),
  optional `role = "card" | "band" | "gallery"`. Lint: formats PNG/JPEG/WebP only, ≤ 4 MiB,
  ≥ 640×360, hashes well-formed, host allowlisted; `--fetch` verifies bytes and dimensions.
- **R2** — **Research per recipe** (all 13): find 1–3 representative official screenshots,
  record source, permission/licence text found, and why it is representative (a recognisable
  map, not a title screen). Cite links; Doomworld/DoomWiki are Cloudflare-blocked for
  automation — prefer author sites, GitHub/GitLab, /idgames mirrors. No source → keep the
  fallback art and say so.
- **R3** — **Fetch and cache:** images download through the existing download stack (bounded,
  timed, verified) into `<XDG cache>/ddl/images/<sha256>`; decoded with bounded dimensions
  (reject decompression bombs); failures fall back silently to the concept art. Downloads start
  when Discover/detail first needs them, never block the UI, and are retried at most once per
  session.
- **R4** — **UI:** cards, detail and the art band use the screenshot (cropped/scaled to fill,
  focal point kept centred), with a small credit caption instead of "Original concept art"; the
  painted fallback keeps its current caption. Ember style, legible caption on a dark plate.
- **R5** — **Offline:** cached images show offline; a never-fetched image shows the fallback.
- **R6** — CONTRIBUTING.md documents how to add screenshots to a recipe.

## Out of scope

Taking our own screenshots (needs IWADs, which DDL never downloads). User-supplied images.
Videos.

## Acceptance criteria

- [ ] **A1** (R1, R6) — lint tests (format, size, dimensions, host, hash); CONTRIBUTING updated.
- [ ] **A2** (R2) — research note per recipe; every recipe with a found source has screenshots
  and passes `catalog lint --fetch`.
- [ ] **A3** (R3, R5) — loopback-fake tests: verify, cache hit, bad hash, oversize, bomb
  dimensions, offline fallback.
- [ ] **A4** (R4) — headless renders: Discover with screenshots + one fallback, detail with
  credit caption.
- [ ] **A5** — fmt, clippy `-D warnings`, tests green; Linux e2e fetches every screenshot.

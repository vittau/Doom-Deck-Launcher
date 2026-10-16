# 012 — Production UI polish: back to the 008 fidelity

**Status:** Ready  ·  **Milestone:** M2  ·  **Depends on:** 010
**Evidence:** `docs/design/ui.md`, `docs/design/visual/*.png`, spec 008 fixture PNGs (the bar to
match), `docs/reports/010-ui-wiring.md` (current production screens).

## Goal

The production screens wired in 010 work, but they lost density and context the 008 fixture
screens had. Bring every production state back to the 008/visual-reference fidelity with real
data, so DDL looks clearly better than UZDeck on a 1280×800 Deck screen. No new features.

## Requirements

- **R1** — **Context in titles:** Play setup, Mapset detail and Download/install titles name the
  mapset (`Play setup / SIGIL`, as in 008); subtitles say something new, never repeat the panel
  body (first run currently repeats the engine sentence twice).
- **R2** — **Home readiness:** the readiness panel shows real checks (engine installed + version,
  each base game found/missing for the last-played mapset, verification state) as in 008's
  "Ready to play" panel; the "add a folder" call to action appears only when something is
  missing. Home keeps a Recently played row from real profiles' last-played times.
- **R3** — **Will launch panel:** no large empty gaps; base game with found/missing status
  colour, numbered load order with component labels, complevel, engine, skill/pistol, then the
  primary action — compact and top-aligned like 008. Correct plurals ("1 content file").
- **R4** — **Art and cards:** Discover cards, Mapset detail and Play setup header use the bundled
  concept art with the "Original concept art" plaque, as in 008.
- **R5** — **Install progress:** the page subtitle and the panel stage agree; show the stage
  sequence (Download → Verify → Extract → Install) with the current one highlighted, real bytes
  in human units (KB/MB), and the mapset art; no invented speed or ETA.
- **R6** — **Directory browser:** folders sorted case-insensitively with hidden (dot) folders
  after normal ones, dimmed; long paths elide from the start (keep the tail visible); Places
  lists Home, each Steam library found and removable media, with icons or clear labels.
- **R7** — **Settings:** a section's right panel never leaves a large blank area; the status
  line in the subtitle reflects the selected section; "Check for updates" before any check reads
  neutrally (no "No reviewed update available" until a check ran).
- **R8** — **Consistency pass:** every production state is compared side by side with its 008
  fixture or visual reference; list remaining intentional differences in the report.

## Out of scope

New features (favorites, categories, profiles CRUD, reorder). ddl-core behaviour. Copy that
changes product meaning.

## Acceptance criteria

- [ ] **A1** (R1–R8) — headless `010-*.png` regenerated and `comparison-010-*.png` side by side
  with the 008 fixture or `docs/design/visual/` reference for every state; report lists fixes
  and remaining differences.
- [ ] **A2** — headless tests assert the R1 titles, the R3 plural, the R6 sort order and the R7
  pre-check copy.
- [ ] **A3** — fmt, clippy `-D warnings`, tests green.
- [ ] **A4** — *Needs Deck Validation:* text readability on the LCD (`docs/deck-testing.md` S3).

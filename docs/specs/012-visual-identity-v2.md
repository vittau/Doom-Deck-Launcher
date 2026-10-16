# 012 — Visual identity v2 (DOOM-flavoured) and production UI polish

**Status:** Phase B in progress  ·  **Milestone:** M2  ·  **Depends on:** 010
**Evidence:** D-020; `docs/design/ui.md` (interaction contract stays), `docs/research/prior-art.md`
(UZDeck critique), spec 008 fixture PNGs and `docs/reports/010-ui-wiring.md` (current screens).

## Goal

DDL looks unmistakably DOOM — the mood UZDeck goes for (DOOM-style display lettering, hellish
red/brown palette, wall-texture atmosphere) — but clearly more beautiful, detailed and tasteful
than UZDeck, and legible on the Deck's 1280×800 LCD. Interaction rules (`ui.md` focus, hints,
navigation) do not change; only the visual layer does. Two phases: **A** mockups the owner picks
from, **B** implementation of the chosen direction.

## Requirements — phase A (design)

- **R1** — Three distinct visual directions, each with **Home** and **Play setup / AUGER;ZENITH**
  as 1280×800 hi-fi mockups (real DDL content: SIGIL, SIGIL II, AUGER;ZENITH, real component
  names, Will launch panel, footer hints with button badges, amber focus ring visible).
  Examples of directions: "UAC tech-base" (metal panels, hazard stripes, green/grey + red),
  "Hell gothic" (red marble, carved stone frames, fire glow), "Classic 1993 menu" (bevelled red
  display lettering, brown/olive status-bar panels, pixel-art accents at high resolution).
- **R2** — For each direction: a one-page style sheet PNG (palette with hex, type specimens,
  panel/button/focus states, texture swatches) and a short rationale.
- **R3** — **Original art only** (D-020): textures, emblems and backgrounds are generated or
  drawn for DDL; nothing traced or copied from id Software material; no DOOM logo imitation.
- **R4** — **Fonts:** for each direction name the display face and the body face with licence
  and source link (OFL or equivalent, embeddable in a GPL binary). Body text stays a legible UI
  face; the display face is for titles, logo and large labels only.
- **R5** — **Legibility:** body text on solid/darkened panels with ≥4.5:1 contrast (state the
  measured ratios for the main pairs); texture never directly behind body text.

## Requirements — phase B (implementation, after the owner picks)

- **R6** — Apply the chosen direction to every production screen and state (tokens, fonts,
  textures as bundled assets, panels, buttons, cards, focus ring, footer badges), keeping the
  Slint software renderer fast enough on the Deck (prefer pre-rendered PNG textures and 9-slice
  frames over many layered effects).
- **R7** — **Context and density fixes** found in the 010 review: titles name the mapset
  (`Play setup / SIGIL`); subtitles never repeat panel text; Home readiness shows real checks
  (engine + version, base game found/missing, verification) and a Recently played row; Will
  launch is compact and top-aligned with correct plurals ("1 content file"); Discover/detail/
  setup use art with the "Original concept art" plaque; install progress shows the stage
  sequence and human byte units with subtitle and panel in agreement; directory browser sorts
  case-insensitively, hidden folders last and dimmed, long paths elided from the start; Settings
  never leaves a big blank panel, and "Check for updates" reads neutrally before any check.

## Out of scope

New features (favorites, categories, profile CRUD, reorder). `ddl-core` behaviour. Changing
navigation or focus rules.

## Acceptance criteria

- [x] **A1** (R1–R5) — `docs/design/visual-v2/` holds, per direction, Home + Play setup mockups,
  a style sheet and a `README.md` with rationale, fonts + licences + links and contrast ratios.
- [x] **A2** — owner picks a direction (coordinator records it in `decisions.md`): Ember Cathedral, intensified (D-021).
- [ ] **A3** (R6–R7) — headless PNGs of every production state with side-by-side comparisons
  against the chosen mockups; headless tests assert the R7 titles, plural, sort order and
  pre-check copy; fmt, clippy `-D warnings`, tests green.
- [ ] **A4** — *Needs Deck Validation:* readability and rendering speed on the LCD
  (`docs/deck-testing.md` S3).

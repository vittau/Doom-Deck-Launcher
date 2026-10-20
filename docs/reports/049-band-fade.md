# 049 — Fade the title panel into the art band

**Spec:** `docs/specs/049-band-fade.md` · **Branch:** `vittau/band-fade`
**Verified:** 2026-10-20

## Changes

- R1: added a 120 px, left-to-right Slint linear-gradient overlay inside the
  existing conditional art band. It starts at opaque `Tokens.deep` and ends
  at the same colour with zero alpha. Both screenshot and painted sources use
  the overlay. The heading, status, title panel and band-free screens retain
  their existing layout; the panel frame component is untouched.
- Added `headless::journeys::title_band_fades_from_panel`, which renders the
  production Discover screenshot, generic painted fallback, Eviternity, and
  SIGIL Mapset detail states. It checks every pixel row across the joint and
  bounds image emergence throughout the 120 px transition, checks visible art
  at the fade endpoint, and compares the title/status pixels with a band-free
  render. The source assertions explicitly distinguish screenshot and fallback.
- R2: regenerated all four README screenshots after merging verified main
  `68c4e5e` (specs 045 and 047 included), via `python3 scripts/readme-screenshots.py`.
  Visually reviewed the refreshed Discover and Mapset detail images.
  Subsequently merged documentation-only main `cdb0958` before final verification.

## Verification

- A1: `cargo test -p ddl title_band_fades_from_panel` passed. Visually reviewed
  the off-screen SIGIL detail and generic fallback PNGs: both fade smoothly from
  the dark panel, and the title retains its contrast and position. Evidence is
  under `target/headless/title_band_fades_from_panel/049-*.png` (regenerable,
  ignored artifacts). Every row from y=120 through y=207 passes the seam check.
  A separate PNG pixel audit measured a maximum channel difference of **0/255**
  between x=839 and x=840 across all 88 rows in each of the four captures.
  The first test run caught an unanchored overlay; adding explicit x=0/y=0 fixed
  it and demonstrates the regression test rejects a visible seam.
- A2: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
  warnings`, and `cargo test --workspace` passed with `set -e` and
  `set -o pipefail`, in the coordinator-granted build slot. The tests include
  140 DDL tests, 408 core tests, integration tests and doctests, with no failures.
  The full check sequence was rerun after the overnight Orca restart.
  `python3 scripts/readme-screenshots.py --check` passed: all four committed
  1280×800 images exactly match the regenerated production frames.
  Reproducible logs are under ignored `target/049-{fmt,clippy,tests,screenshots-check}.log`.

No new dependencies or band-art assets. No visible windows or desktop captures.

## Open questions / remaining items

None. R1, R2, A1 and A2 are complete. No new dependencies and no spec items left open.

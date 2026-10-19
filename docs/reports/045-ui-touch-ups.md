# Spec 045 — UI touch-ups

## Changes

- Crop the wall quarter at `(0, 0, 510, 433)` inside the original PNG atlas border, preserving the mirrored-quarter construction. Add `build_assets.py --wall-only` so this correction never re-encodes unrelated art. Regenerated with Pillow 12.3.0 in a scratch venv; no runtime or Rust dependency added.
- Remove the production Home verification check and compact Discover upward by one 72px row. Remove the same row in the legacy shell and compact its button by 42px (coordinator explicitly approved the shell.rs ownership extension).
- Remove the unconditional verification footer on mapset detail, preserving the actionable content-update/save warning.
- Refresh all four README images through the existing off-screen production journeys. No desktop windows or screen captures used.

## Player-copy audit (R2)

All `crates/ddl/ui/*.slint` translation resources and Rust display call sites were searched case-insensitively for verification, validation and research qualifiers. No UI reads a recipe's real `[verification]` status today; no catalog evidence was changed or invented.

| Original string / context | Result and reason |
| --- | --- |
| `Verification pending · Needs Deck Validation` — production Home and unconditional detail footer | Removed key, enum variant and both usages; project status carries no player action. |
| `Verification pending` — legacy Home / detail label | Home row removed; detail label now `Compatibility` next to its complevel recommendation. Existing indexed translation slot retained for legacy key alignment. |
| `Ultimate DOOM · Installed · Verification pending · Today` | `Ultimate DOOM · Installed · Today`. |
| `DOOM II · Installed · Verification pending · Yesterday` | `DOOM II · Installed · Yesterday`. |
| `Installed · Verification pending` | `Installed`. |
| `Not installed · Verification pending` | `Not installed`. |
| `Reset inventory at each map · needs Deck validation` | `Reset inventory at each map`; retain option explanation. |
| `Reviewed transfer digest · gameplay needs Deck validation` | `Reviewed transfer digest`; retain actual transfer-trust information. |
| `DSDA-Doom v0.30.1 · research reference` | `DSDA-Doom v0.30.1`. |
| `Installed: v0.30.1 · research reference` | `Installed: v0.30.1`. |
| `Complevel 3 — recipe recommendation (not yet verified)` | `Complevel 3 — recipe recommendation`. |
| `Complevel 2 — recipe recommendation (not yet verified)` | `Complevel 2 — recipe recommendation`. |
| `Complevel {} · recipe recommendation (not yet verified)` | `Complevel {} · recipe recommendation`. |
| `Custom — not verified` | Kept: warns about user-imported content's real compatibility limitation. |
| `Your files stay in place. Compatibility is not verified.` | Kept: actionable explanation of local imports. |
| `Verifying…`, `Checking the files before use.`, `Download → Extract → Verify → Install`, `Download → Verify → Extract → Install` | Kept: actual installation integrity work and progress, not project-process labels. |
| `Recipe evidence` | Kept: legacy detail section heading, not a verification-status claim. |
| `Release trust and gameplay checks are separate.` | Kept: explains why transfer trust does not establish gameplay compatibility. |
| `Not validated · explicit warning before installation` | Kept: warns about an unlisted engine release before installing it. |

Coordinator approved stripping hard-coded research/validation qualifiers from engine and complevel copy as part of R2.

## Verification

### A1 — wall

Using Pillow `convert('L')` and `ImageStat.Stat(...).mean[0]` on one-pixel rows/columns of decoded `wall.jpg`:

| Axis/index | Before | After |
| --- | ---: | ---: |
| row 217 | 14.43 | 26.25 |
| row 218 | 6.48 | 26.38 |
| row 219 | 5.59 | 26.62 |
| row 220 | 15.12 | 26.90 |
| column 254 | 28.59 | 30.48 |
| column 255 | 12.82 | 29.06 |
| column 256 | 12.83 | 29.06 |
| column 257 | 28.59 | 30.48 |

Repeat edges also pass: rows 437/438 = 31.36/31.27; columns 511/512 and 1023/1024 = 26.57/26.60; columns 767/768 = 29.06/29.06. All sampled seam means exceed 20. Running `--wall-only` again produces identical bytes; every other PNG/JPEG under `ember/` matches HEAD byte for byte. The original PNG atlas is the sole input to the wall path; the existing JPEG is never read.

### A2 — readiness and audit

The production Home journey asserts exactly three checks at y=100/172/244, tones 2/2/4, Discover at y=330, and no forbidden process labels. The resource lint rejects the removed process phrases in every Slint string resource. The legacy shell navigation test now follows the compacted Discover control and verifies its remembered focus; the isolated shell journey passes. The audit above accounts for retained real limitations and installation verification. `cargo test -p ddl headless::journeys::library_favorite_and_home` and `cargo test -p ddl --test ui_strings` pass. Visually inspected the off-screen Home PNG: continuous wall and no fourth row.

### A3 — final checks

`python3 scripts/readme-screenshots.py --check` passes for all four 1280×800 images: Home 307,043 bytes; Discover 516,183; Play setup 297,068; Mapset detail 549,635. After merging `main` (through `b112365`) and obtaining the coordinator build slot, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` all pass. The suite includes 139 launcher unit/headless tests, 408 core unit tests, all integration tests and doctests. The first full run identified the stale legacy spatial-navigation expectation after compaction; updated it and verified the isolated journey before the final all-green run.

## Open questions / remaining items

None; R1–R3 and A1–A3 are complete. Physical Deck validation is outside this visual/copy task; no new device-behavior claims are made.

## Post-review fix — expanded spec 037 catalog

Merged `main` through `b369b7b` (merge `54fa870`) following CI run 39304816147. The category/favorites/screenshot journey had two remaining assumptions tied to the earlier catalog:

- The screenshot assertion required exactly 640×360. It now checks both dimensions against the exported `ddl_core::catalog::{MIN,MAX}_SCREENSHOT_{WIDTH,HEIGHT}` bounds, retaining rejection of undersized/oversized images and the existing embedded-image, card/band/detail and credit checks for every recipe declaring screenshots.
- Favoriting the second recipe happened to select a multi-tag recipe before spec 037. The journey still checks Right navigation to a non-first card, then explicitly selects a non-first recipe with at least two tags for the favorite/detail flow. It retains the exact tag comparison, multiple-tag assertion, persisted favorite identity, first-position sort assertion, and X/button unfavorite checks.

The complete targeted journey passes against the expanded catalog (58.37s), with no further failures. Refreshed README Discover and detail images on the 89-mapset catalog; Home and Play setup remain byte-identical. Visually inspected the off-screen Discover image. `readme-screenshots.py --check` passes: Home 307,043 bytes, Discover 547,647, Play setup 297,068, detail 489,703.

All follow-up validation commands run under `set -o pipefail`, chained with `&&`, with direct stdout/stderr redirection rather than output-filtering pipelines so Cargo failures propagate. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` all pass on the merged catalog. The complete command chain exited 0; no acceptance item remains open. No dependencies or product behavior changed in this follow-up.

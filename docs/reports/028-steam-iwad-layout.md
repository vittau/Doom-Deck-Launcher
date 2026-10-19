# 028 — Find every IWAD in the real Steam DOOM + DOOM II layout — worker report

**Status:** R1–R4 implemented; A1–A3 verified by tests, A4 green locally with the Deck re-check
still **Needs Deck Validation**. The Steam scan now probes the confirmed `Ultimate Doom/` tree
(classic `base/` copies plus KEX `rerelease/` copies), bounded and case-insensitive, and keeps the
KEX resource/Master Levels PWADs out of the candidate list. No new dependencies.

## What changed

- `crates/ddl-core/src/iwads/steam.rs`
  - `KNOWN_INSTALL_DIRS` now names the confirmed Deck folders explicitly:
    `Ultimate Doom/base`, `Ultimate Doom/base/doom2`, `.../tnt`, `.../plutonia` and
    `Ultimate Doom/rerelease`, alongside the previous `DOOM + DOOM II/…` and legacy entries.
    Every entry is still scanned flat, so there is no recursive walk and no `$HOME` search;
    `base/master/wads/` (the 20 Master Levels PWADs) is deliberately not scanned.
  - `candidate_iwads` skips names in the new `KEX_RESOURCE_WADS` list when scanning a
    `rerelease/` folder (`id1*.wad`, `extras.wad`, `id24res.wad`, `iddm1.wad`,
    `masterlevels.wad`, `nerve.wad`, `sigil*.wad`). The filter runs before identification, so
    even an IWAD-tagged or hash-identical resource cannot be offered as a base game.
  - Folder and file matching stays case-insensitive (the Deck's classic files are upper case and
    its KEX files lower case).
- `crates/ddl-core/src/resolver/tests.rs` — new resolver test for classic-over-KEX preference.
  The production ranking already lived in `Registry::best_for`/`preference`; it is now proven
  against the real `base/` vs `rerelease/` path shape.
- `crates/ddl/src/cli_exec.rs` — new end-to-end `ddl iwad scan` test over the Deck tree (test
  only; no production CLI change was needed).
- `docs/research/iwad-identification.md` — §4 corrected: the Deck layout is **Confirmed
  (Deck, 2026-10-19)**, the classic/KEX casing is stated, and `DOOM + DOOM II/` is marked
  unconfirmed on Steam (possibly GOG's). §4's closing `Needs Deck Validation` list now covers
  only the legacy folders and GOG/other locations.

Fixtures are built in the tests from synthetic WADs: a valid 12-byte header plus the identifying
lump names only (E1M1…E4M9 or MAP01…MAP32, PWADs for the resources). No real IWAD content and no
network access.

## Acceptance criteria → named tests

| ID | Test | What it proves |
| --- | --- | --- |
| **A1** (R1, R2) | `iwads::steam::tests::deck_layout_finds_the_four_games_in_base_and_rerelease_in_every_library` | A fixture tree copying the Deck layout in two libraries (internal + SD card, second one fully upper-cased) yields exactly the four classic and four KEX candidates per library; each identifies as `Ultimate` or `Commercial`. Master Levels live under `base/master/wads/` and are not candidates; all resource WADs identify as `Unknown`. |
| **A1** (R2) | `iwads::steam::tests::rerelease_resources_are_filtered_by_name_before_identification` | Resource names are excluded even when the file has `IWAD` magic and base-game-looking lumps, proving the name filter (not only PWAD detection) keeps them out. |
| **A2** (R3) | `resolver::tests::resolver_prefers_the_classic_steam_copy_over_the_kex_copy` | With KEX entries inserted first, SIGIL's plan link resolves to `base/DOOM.WAD` and AUGER;ZENITH's to `base/doom2/DOOM2.WAD`; both KEX copies stay registered (the existing `best_for` test also shows an accepted KEX variant is still selectable). |
| **A3** | `cli_exec::tests::iwad_scan_lists_the_real_steam_layout_as_steam` | `ddl iwad scan` over the fixture prints eight `Steam: …` lines, one per file, and reports no unrecognized files. |
| **A4** | `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` | All green on this branch after merging `main` (workspace test: 379 `ddl-core` unit tests plus the integration/UI suites, 0 failures). Deck re-check of F2 remains **Needs Deck Validation** (no Deck access here). |

## Verification notes

- Targeted runs while iterating: `cargo test -p ddl-core iwads::steam`, the resolver test, and
  `cargo test -p ddl iwad_scan_lists_the_real_steam_layout_as_steam`.
- Full checks were run once at the end. The coordinator's hold on the workspace-build slot arrived
  after the run had already completed; no further workspace command was started.
- The existing Steam tests still pass unchanged: flat scanning, odd casing, missing folders, no
  recursion into unrelated subfolders.

## New dependencies

None.

## Open questions / follow-ups

1. **Nerve as an expansion.** R2 requires `nerve.wad` not to be offered as an IWAD from Steam, so
   the scan now filters it. The known-IWAD table still lists `nerve-kex` (as a PWAD) so a
   manually added copy can be identified and symlinked. If Nerve should also be discoverable
   from the Steam tree later, that needs a separate decision (it is not a base game).
2. **Exact variant identities in fixtures.** Synthetic WADs cannot carry the published hashes, so
   A1 asserts the identified *families*; exact `doom-1.9ud`/KEX variant matching remains covered
   by the table/identification tests (and by A2, which registers real variant ids).
3. **`DOOM + DOOM II/` folder.** Kept in `KNOWN_INSTALL_DIRS` defensively per R1, but it was not
   present on the Deck and may be GOG's layout. A GOG Deck check would settle it.
4. **Legacy `Master Levels of Doom/doom2` folder.** Still scanned flat because it is a legacy
   install entry; its PWADs are rejected by the identifier. No change made.

## Proposed coordinator-owned edits

- `docs/status.md`: record spec 028 as implemented, with F2 awaiting the Deck re-check.
- `docs/journal.yml`: add the spec 028 entry.
- No decision change proposed: D-017 and the research §4 *Proposed* classic-over-KEX preference
  already match the implemented behaviour.

## Spec items left open

- A4's Deck re-check (F2): install/update on the Deck, run `ddl iwad scan`, and confirm all eight
  files appear with source "Steam" and that a Doom II mapset resolves without "Base game missing".

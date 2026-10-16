# 007 — IWAD table, identification, discovery and canonical links

**Status:** Ready (after 001 merges)  ·  **Milestone:** M1  ·  **Depends on:** 001
**Evidence:** `docs/research/iwad-identification.md` (all sections); decisions D-010, D-017.

## Goal

DDL knows which base games the user legally owns, by content and not by filename, finds them in
Steam libraries and user-chosen folders without scanning the whole home directory, and hands
DSDA-Doom a path whose filename makes it pick the right game mission.

## Requirements

- **R1** — **Known-IWAD table** `crates/ddl-core/data/iwads.toml` following the research's §5.2
  shape: per variant `id`, `game` (`doom`, `doom2`, `tnt`, `plutonia`, plus `nerve` for the
  expansion), `title`, canonical `filename`, `size`, whichever of `sha256`/`md5`/`sha1`/`crc32`
  have a published source, `default_complevel`, `source` URL. Populate it **only** with values
  from the research document (do not invent hashes). Include the shareware DOOM entry marked
  `playable = false` (it is not accepted by commercial content).
- **R2** — **Identifier** `identify(path) -> Identification`: bounded header + directory read
  (reject non-WAD, directory larger than limits, files over 64 MiB); size pre-filter; hash match
  against the strongest published hash (MD5/SHA-1 only as identification, D-017); compute and
  return SHA-256 always. Do not reject files whose magic is `PWAD` if their hash matches a
  known variant (BFG/PSN quirk). Without a hash match, classify the family with DSDA's lump
  rules from research §2 and return `Ambiguous { candidates }` within the Doom II family, or
  `Unknown`.
- **R3** — **Steam discovery:** find Steam roots (`~/.local/share/Steam`, `~/.steam/root`,
  Flatpak `~/.var/app/com.valvesoftware.Steam/.local/share/Steam`), parse
  `steamapps/libraryfolders.vdf` (Valve KeyValues; write a small tolerant read-only parser,
  no Steam writes ever), and list candidate IWAD files under each library's
  `steamapps/common/` for the known install dirs from research §4 (`DOOM + DOOM II/base`,
  `DOOM + DOOM II/rerelease`, `Ultimate Doom/base`, `Doom 2/base`, `Doom 2/finaldoombase`,
  `Final Doom/base`, `Master Levels of Doom/doom2`), case-insensitively. Missing paths are
  normal, not errors.
- **R4** — **User folders:** identify every `*.wad` (case-insensitive) directly inside a
  user-chosen folder, non-recursively, plus one level of subfolders; never follow symlinks
  out of the chosen folder; never scan `$HOME` as a whole.
- **R5** — **IWAD registry** (JSON state, atomic writes): identified files with path, variant id,
  SHA-256, size, mtime and source (steam/user folder/manual); user choices for ambiguous files;
  `refresh()` re-checks existence and size+mtime, re-hashing only changed files. The
  `best_for(game, accepted_variants)` query prefers `base/` (classic) over `rerelease/` (KEX)
  copies for the same game unless the recipe asks otherwise.
- **R6** — **Canonical link:** `canonical_path(entry)` creates or refreshes
  `<data>/iwads/<variant-id>/<canonical filename>` as a symlink to the user's file, verifying the
  target still matches the recorded size and SHA-256 (full re-hash if mtime changed), and returns
  the link path for 003's `LaunchPlan`.

## Out of scope

The controller folder-picker UI (UI spec). Recipe resolution (later). GOG paths (research marks
them unvalidated; add after Deck checks).

## Acceptance criteria

- [x] **A1** (R1) — table parses; every hash value has a `source`; a test fails on duplicate ids.
- [x] **A2** (R2) — synthetic WAD fixtures built in tests (header + directory with marker lumps,
  no copyrighted data): E1–E4 family, MAPxx family → Ambiguous, PWAD-magic tolerance via a
  test-only table entry, oversize rejection.
- [x] **A3** (R3) — fixture Steam trees with `libraryfolders.vdf` samples (two libraries, odd
  casing, missing folders); parser fuzz-style tests on malformed VDF.
- [x] **A4** (R4, R5) — symlink-escape refusal; refresh only re-hashes changed files;
  `best_for` preference.
- [x] **A5** (R6) — link creation, refresh, target-changed detection.
- [ ] **A6** — *Needs Deck Validation:* on the Deck, DDL finds the owner's DOOM + DOOM II
  install and identifies its IWADs (tracked in `docs/deck-testing.md`).

## Tasks

- [x] Table + identifier
- [x] Steam VDF + discovery + user folders
- [x] Registry + canonical links

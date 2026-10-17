# 021 — GOG and other store IWAD paths

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 007
**Backlog item:** "GOG IWAD paths". Spec 007 discovers Steam libraries only; research
`docs/research/iwad-identification.md` §4 lists GOG and generic locations as Proposed.

## Goal

A Deck owner who bought DOOM + DOOM II (or Final Doom, etc.) on GOG and installed it through the
usual Deck routes gets the IWADs found automatically, like Steam users.

## Requirements

- **R1** — **Research first** (extend `docs/research/iwad-identification.md` with a GOG section,
  cited and pinned): where GOG games land on a Steam Deck through **Heroic Games Launcher**
  (Flatpak and native: its `gog_store/installed.json` or equivalent that records install paths,
  default `~/Games/Heroic/<title>`), **Lutris** (its GOG install dirs/DB), **minigalaxy**, and
  the offline GOG Linux/Windows installers (`~/GOG Games/…`); the folder layout of the GOG
  DOOM + DOOM II, The Ultimate DOOM, DOOM II, Final Doom and DOOM 3 BFG-era products where they
  hold IWADs; which IWAD revisions they ship (link the hash table). Label each claim Confirmed /
  Proposed / Needs Deck Validation. Never invent paths: no source → not implemented.
- **R2** — **Discovery:** a `gog` (or `stores`) module beside `iwads/steam.rs` that reads the
  launcher records found in R1 (read-only, bounded file sizes, tolerant of missing/garbled files)
  and yields candidate IWAD files, also probing the confirmed default folders. Flatpak paths
  (`~/.var/app/<app id>/…`) included where R1 confirms them.
- **R3** — **Same pipeline:** candidates go through the existing identification (D-017);
  ambiguous files are listed, never auto-chosen; the source ("GOG via Heroic", "Steam") is shown
  in the IWAD list and in `ddl iwad scan`.
- **R4** — **Generic roots (opt-in):** `DOOMWADDIR`/`DOOMWADPATH`, `/usr/share/games/doom`,
  `~/.local/share/games/doom` as candidate roots only when the user enables "Search standard
  Linux folders" in Settings (default off), per research §4.
- **R5** — `deck-testing.md` gains GOG checks (Needs Deck Validation).

## Out of scope

Installing or launching GOG games. Windows-only paths outside Proton prefixes unless R1 confirms
a Deck route uses them. Epic/Bethesda.net.

## Acceptance criteria

- [ ] **A1** (R1) — research section committed with links.
- [ ] **A2** (R2, R3) — tests with fixture launcher records and temp homes: found, missing,
  garbled, Flatpak; source label in scan output.
- [ ] **A3** (R4) — opt-in default-off test; Settings row headless render.
- [ ] **A4** — fmt, clippy `-D warnings`, tests green.
- [ ] **A5** — *Needs Deck Validation:* a GOG install via Heroic is found on the Deck.

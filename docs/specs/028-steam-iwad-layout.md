# 028 — Find every IWAD in the real Steam DOOM + DOOM II layout

**Status:** Ready  ·  **Milestone:** M3  ·  **Depends on:** 007, 021
**Origin:** Deck test 2026-10-19 on `v0.0.6-local` (check F2 ❌): the Steam scan found only
`Ultimate Doom/base/DOOM.WAD`, so every Doom II mapset showed "Base game missing".

## Facts from the Deck (Confirmed, owner's `find` on a Steam Deck LCD, 2026-10-19)

Steam app 2280 (*DOOM + DOOM II*) installs to `steamapps/common/Ultimate Doom/`, not
`DOOM + DOOM II/` as `docs/research/iwad-identification.md` §4 assumed:

```text
Ultimate Doom/base/DOOM.WAD
Ultimate Doom/base/doom2/DOOM2.WAD
Ultimate Doom/base/tnt/TNT.WAD
Ultimate Doom/base/plutonia/PLUTONIA.WAD
Ultimate Doom/base/master/wads/{ATTACK,BLACKTWR,…,VIRGIL}.WAD      (20 Master Levels PWADs)
Ultimate Doom/rerelease/{doom,doom2,tnt,plutonia,nerve,masterlevels,sigil,sigil2,
                         extras,id1,id1-mus,id1-res,id1-tex,id1-weap,id24res,iddm1}.wad
```

Classic files are upper case, KEX files lower case. Today `KNOWN_INSTALL_DIRS`
(`crates/ddl-core/src/iwads/steam.rs`) probes `Ultimate Doom/base` one level deep only and
probes `rerelease/` only under `DOOM + DOOM II/`.

## Requirements

- **R1** — **Probe the real layout:** the Steam scan finds `DOOM.WAD`, `DOOM2.WAD`, `TNT.WAD` and
  `PLUTONIA.WAD` under `Ultimate Doom/base/` (including its `doom2/`, `tnt/`, `plutonia/`
  subfolders) and the KEX copies under `Ultimate Doom/rerelease/`, in every Steam library
  (internal and SD card), matched case-insensitively. Keep the existing `DOOM + DOOM II/…` and
  legacy entries.
- **R2** — **Bounded:** still no `$HOME` walk; only named subfolders or a depth limit of at most two
  levels below an install dir. The 20 Master Levels PWADs and the KEX resource WADs
  (`id1*.wad`, `extras.wad`, `id24res.wad`, `iddm1.wad`, `masterlevels.wad`, `nerve.wad`,
  `sigil*.wad`) are not offered as IWADs. The identifier already rejects PWADs; prove it with
  the fixture.
- **R3** — **Prefer classic over KEX:** when a game is found in both `base/` and `rerelease/`, the
  automatic pick for a recipe stays the classic `base/` file (research §4 *Proposed*, unchanged).
  The KEX copy stays listed and selectable.
- **R4** — **Research update:** correct `docs/research/iwad-identification.md` §4 with the layout
  above, labelled **Confirmed (Deck, 2026-10-19)**. Mark the old `DOOM + DOOM II/` folder name as
  unconfirmed on Steam (it may still be GOG's).

## Acceptance criteria

- [ ] **A1** (R1, R2) — a fixture tree that copies the Deck layout (zero-byte or tiny synthetic
  WADs with the right headers and lumps, never real IWAD content), with Master Levels and KEX
  resource files included, yields exactly the expected IWAD candidates and identities; the same
  tree under a second library (SD card) is found too.
- [ ] **A2** (R3) — a resolver test: with both copies present, SIGIL resolves to `base/DOOM.WAD`
  and AUGER;ZENITH to `base/doom2/DOOM2.WAD`.
- [ ] **A3** — `ddl iwad scan` against the fixture lists each file with source "Steam".
- [ ] **A4** — fmt, clippy `-D warnings`, tests green. Deck re-check: F2 (Needs Deck Validation).

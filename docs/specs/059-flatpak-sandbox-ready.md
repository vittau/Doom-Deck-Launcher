# 059 — Make DDL sandbox-ready for its Flathub Flatpak

**Status:** Ready  ·  **Milestone:** M4 (distribution)  ·  **Depends on:** 004, 007, 021, 026, 031, 058
**Origin:** owner, 2026-10-20: "Quanto ao flatpak, quero no Flathub direto." Decision D-034;
evidence `docs/research/flatpak.md` (spec 058).

Application code only. **Never write or edit a Flatpak manifest, AppStream metainfo, desktop
entry, `flathub.json`, or any Flathub submission text** — the owner writes those by hand
(Flathub's Generative AI policy, D-034). Do not add a manifest even as a test fixture.

## Requirements

- **R1 — Detect the sandbox.** One small platform probe: running under Flatpak when
  `/.flatpak-info` exists (research §3). Expose it behind the existing platform trait so tests use
  a fake; log it at startup and show it in Settings → Diagnostics / About ("Flatpak sandbox:
  yes/no", the app id from `FLATPAK_ID` when set).
- **R2 — Engine: always extract in the sandbox.** When sandboxed, never try FUSE: extract the
  verified AppImage with DDL's own extraction path (spec 004 already supports extraction) into the
  engine directory and launch its `AppRun` with the unchanged D-011 argv. Outside the sandbox keep
  today's behaviour. Version pinning, verification and rollback (spec 004) work the same.
- **R3 — Paths.** Confirm `paths.rs` resolves config/data/state/cache from the sandbox's `XDG_*`
  (inside `~/.var/app/<id>/`) with no hard-coded home; test with a fake environment.
- **R4 — Content visibility.** IWAD discovery (Steam incl. `/run/media` SD cards and the Steam
  Flatpak's `~/.var/app/com.valvesoftware.Steam`, GOG/Heroic incl. Heroic's Flatpak dir, user
  folders) and the controller file picker (spec 026) keep working through read-only host
  access. When a configured or discovered path exists on the host but is not visible from the
  sandbox (permission error or missing mount), say so in one line instead of showing an empty
  list ("<path> is not visible to the Flatpak — see the guide"). Tests with fake roots.
- **R5 — One-time import of a pre-Flatpak install.** When sandboxed and DDL's own data dir is
  empty, look for a tarball-era install at the host's `~/.config/ddl` and `~/.local/share/ddl`
  (readable via read-only home access). If found, offer once: "Import your existing setup?" —
  copy (never move) app state, profiles, library and engine metadata into the sandbox dirs,
  rewriting absolute paths that pointed into the old data dir; IWAD references stay pointing at
  the player's files. Uses spec 055's readers, so any supported older state imports. Decline is
  remembered. Controller-first dialog, headless render.
- **R6 — Steam keyboard.** When sandboxed, `steam` is not on `PATH`: open
  `steam://open/keyboard` through `xdg-open` (OpenURI portal) instead of `Command::new("steam")`;
  keep the visible "Steam + X" hint as the fallback (spec 031). Outside the sandbox, unchanged.
- **R7 — Updates.** When sandboxed, hide any app self-update path and say updates come from
  Discover/Flathub (About shows DDL version and engine version separately). Catalog (D-025) and
  engine downloads are unchanged.
- **R8 — Guide.** Add a "Flatpak (Discover)" section to `docs/steam-deck-guide.md`: install from
  Discover, add to Steam (the exported launcher or `/usr/bin/flatpak run
  io.github.vittau.Doom-Deck-Launcher` — label **Needs Deck Validation**), Compatibility tool
  none, artwork, no Steam overlay, where data lives (`~/.var/app/…`), importing a tarball-era
  setup. Mark it as applying once the app is on Flathub.

## Constraints

Never open visible windows or capture the screen — headless renders only. No new permissions
are decided here; R4's messages must not assume any specific `finish-args` beyond what the
research lists. Keep non-sandbox behaviour byte-for-byte identical where possible.

## Acceptance criteria

- [ ] **A1** (R1–R3) unit tests with a fake sandbox marker and fake `XDG_*`; engine plan test shows
  extract-then-`AppRun` when sandboxed and today's path otherwise.
- [ ] **A2** (R4) tests for the "not visible" message with fake roots; picker still lists visible
  roots.
- [ ] **A3** (R5) import test from a spec 055 fixture tree into an empty fake sandbox dir (paths
  rewritten, IWAD links untouched, original files unchanged); headless render of the offer.
- [ ] **A4** (R6, R7) tests for the keyboard command selection and the hidden update path;
  Diagnostics render showing the sandbox line.
- [ ] **A5** fmt, clippy `-D warnings`, `cargo test --workspace` green; grep in the report showing
  no manifest/metainfo/desktop/flathub.json file was added.
- [ ] **Needs Deck Validation** (after the owner's first local Flatpak build): extracted engine
  runs with GL, audio and pad; IWADs found; import works; keyboard opens in Gaming Mode.

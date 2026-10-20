# 058 — Research: ship DDL as a Flatpak installable from Discover

**Status:** Ready  ·  **Milestone:** M4 (distribution)  ·  **Depends on:** 001, 004
**Origin:** owner, 2026-10-20: "Quero também investigar a possibilidade de disponibilizar o DDL
como flatpak, viabilizando install por Discover store. Precisamos ver a questão de invocar o
AppImage a partir do flatpak. Quero reduzir ao máximo o atrito para uso da ferramenta."

Research only: no code, no manifest committed to the repo, no submission anywhere. Output is
`docs/research/flatpak.md` with pinned links and **Confirmed / Proposed / Needs Deck Validation**
labels, plus the report.

## Questions

1. **Engine inside the sandbox.** DDL launches the official DSDA-Doom AppImage (D-001/D-011).
   Can a Flatpak app run an AppImage it downloaded? Cover FUSE inside the sandbox,
   `--appimage-extract-and-run` / extracting with DDL's own safe extractor and running the
   binary, the libraries the engine needs vs what the Flatpak runtime (which one: freedesktop,
   GNOME, KDE; version) provides, OpenGL/Vulkan and audio (PipeWire/PulseAudio) access, and
   gamepad access (`--device=all`, `/dev/input`, `/dev/hidraw`). Cite Flatpak docs and sources.
2. **Alternatives for the engine** — compare with evidence:
   (a) download the AppImage, extract and run inside the sandbox;
   (b) `flatpak-spawn --host` to run it on the host (needs `--talk-name=org.freedesktop.Flatpak`;
   what Flathub's policy says about that permission);
   (c) build DSDA-Doom from source as a module of DDL's Flatpak (GPL-2.0+ engine, GPL-3.0+
   DDL: licensing, update cadence, who maintains it), or depend on an existing DSDA-Doom
   Flatpak/extension if one exists on Flathub (check, with the app id and maintainer);
   (d) anything else found. Which keeps DDL's "one engine: official DSDA-Doom" rule and engine
   version pinning/rollback (spec 004)?
3. **Files and paths.** IWAD discovery (Steam library incl. SD card under `/run/media`, GOG/Heroic
   incl. Heroic's own Flatpak dir, user folders), the controller file picker of spec 026,
   XDG paths inside `~/.var/app/<id>/`, and whether existing installs migrate. Which
   `--filesystem` permissions are needed and whether Flathub accepts them; portals as an
   alternative.
4. **Steam / Gaming Mode.** Adding a Flatpak app to Steam (Discover's "Add to Steam" vs manual),
   launching in Gaming Mode, the `steam steam://open/keyboard` call of spec 031 from inside the
   sandbox, `SteamAppId`/overlay, and the launch-option advice in `docs/steam-deck-guide.md`.
5. **Flathub path.** Submission requirements (app id under `io.github.vittau.*`, AppStream
   metainfo, screenshots, OARS, license, `x-checker-data`/update flow, network downloads at
   runtime, build from source with offline cargo vendoring via `flatpak-cargo-generator`), review
   timeline, and the alternative of a self-hosted Flatpak repo / `.flatpakref` (does Discover on
   SteamOS install those?). SteamOS specifics: Flathub preconfigured as a user remote in Discover.
6. **Self-update and catalog.** How DDL's catalog updates (spec 019) and engine downloads behave
   when the app itself updates through Flathub.
7. **Recommendation.** One recommended architecture with the friction for a new Deck user
   (steps from Discover to playing) compared with today's GitHub tarball, the work it implies
   (draft spec outlines), risks, and what needs Deck validation. Flag anything that would change
   a decision in `docs/decisions.md` (D-001 engine, D-011 launch) as a question for the owner.

## Acceptance criteria

- [ ] **A1** `docs/research/flatpak.md` answers 1–7 with pinned links (tags/commits for sources,
  dated pages for docs and policy); every claim labelled.
- [ ] **A2** the recommendation includes a friction comparison table and draft spec outlines.
- [ ] **A3** report at `docs/reports/058-flatpak-research.md` with open questions.

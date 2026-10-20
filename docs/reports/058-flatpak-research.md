# 058 — Flatpak research — worker report

**Status:** research complete 2026-10-20. **No code, no manifest, no submission.** All findings
and sources are in [`docs/research/flatpak.md`](../research/flatpak.md); this report answers Q7
(recommendation) and records the open questions.

**Recommendation — yes, with caveats.** DDL can ship as a Flatpak that Discover installs, and the
official DSDA-Doom AppImage can run inside the sandbox **if it is always extracted first**. Keep
the engine downloaded at runtime (D-001/D-007/D-011/D-013 intact); the Flatpak is a delivery and
update channel for DDL only. The technical path is not the hard part: Flathub's **Generative AI
policy**, permission exceptions and a set of Deck checks are.

## What was delivered

| Item | Deliverable |
| --- | --- |
| Q1 — engine inside the sandbox | Research doc §3: three independent reasons FUSE cannot work, the two extraction flavours, the runtime library audit against `org.freedesktop.Platform` 26.08, graphics/audio/input (including `/dev/input` vs `/dev/hidraw`), process behaviour. |
| Q2 — alternatives | §4: five options compared on "one engine", pinning/rollback, sandbox story and Flathub fit; (b)–(e) each with the evidence that rules it in or out. |
| Q3 — files and paths | §5: default sandbox view, DDL's discovery paths mapped to permissions, the proposed `finish-args`, linter consequences and the exception reasons, portals, existing installs. |
| Q4 — Steam / Gaming Mode | §6: Discover has no Add-to-Steam, the manual shortcut, the missing overlay, the OSK through OpenURI, launch advice. |
| Q5 — Flathub path | §7: submission checklist, AI policy, "thin wrapper"/"host-dependent" clauses, review and publication, SteamOS's preconfigured Flathub remote, self-hosted repo alternative. |
| Q6 — self-update and catalog | §8: `/app` is replaced by the Flatpak ref, data survives, in-app self-update must be disabled when sandboxed, catalog and engine downloads are unaffected. |
| Q7 — recommendation | §9 here and in the research doc: architecture, friction table, draft specs, risks, decision questions. |
| A1 | Every claim carries **Confirmed**, **Proposed** or **Needs Deck Validation**; sources are pinned in §2 (tags for Flatpak 1.18.4, AppImage runtime `fedfad5`, Discover `v6.7.3`, linter `e38c396`; dated pages for Flathub policy and Flatpak docs). |
| A2 | §9.2 is the friction comparison; §9.3 lists draft specs 059–063. |
| A3 | This report. |

No file outside the two deliverables changed, no dependency was added and no build was run.
Nothing was submitted to Flathub, and no manifest was written at all.

## Recommendation

### Chosen: (a) download the AppImage at runtime and extract it inside the sandbox

**Confirmed** — the mount path is not a "sometimes fails" fallback; it cannot work:

1. the type-2 runtime only mounts through a root-owned **setuid** `fusermount`, and Flatpak maps
   only the calling user;
2. Flatpak runs `--unshare-user --disable-userns` and seccomp returns `EPERM` on
   `clone(CLONE_NEWUSER)`, so no unprivileged FUSE mount can be created;
3. the platform runtime ships no `fusermount` at all.

Even `--device=all` does not lift the first two. **Confirmed precedent:** Lutris' Flathub manifest
sets `APPIMAGE_EXTRACT_AND_RUN=1` with the comment "Needed for appimages".

**Proposed design.** When `/.flatpak-info` exists, DDL skips FUSE entirely and extracts with
`--appimage-extract` into its engine cache, then runs the AppDir's `AppRun` with the D-011 argv.
`--appimage-extract` puts `squashfs-root/` in DDL's chosen cwd; `APPIMAGE_EXTRACT_AND_RUN`
extracts to `$TMPDIR` and deletes the tree after each launch, which would cost a re-extract on
every start. Keeping the AppDir also preserves `-v` smoke tests and engine rollback (spec 004).

Why this option:

- it keeps the unmodified official engine, the reviewed-hash list and multiple installed engine
  versions — nothing in D-001/D-007/D-011/D-013 changes except that extraction is now the primary
  path rather than the fallback;
- the engine runs against the runtime's libraries, so the sandbox still describes what runs;
- Flathub already accepts apps that download or drive components at runtime from inside the
  sandbox (Lutris, Heroic, DoomRunner).

### Fallbacks, in order

| Option | When to use it | Cost |
| --- | --- | --- |
| (c) build DSDA-Doom as a manifest module | Flathub refuses a runtime-downloaded engine | **Changes D-013**: engine version becomes the app version, in-app engine choice and rollback disappear; GPL-2.0-or-later into GPL-3.0-or-later is permitted and satisfies corresponding-source. |
| (b) `flatpak-spawn --host` | Extraction is rejected on Deck | Needs `--talk-name=org.freedesktop.Flatpak` (a linter review finding, already accepted for Lutris and DoomRunner); the engine leaves the sandbox, and every path must be re-mapped. |
| (d) existing DSDA-Doom Flatpak | — | **Confirmed absent** from Flathub on 2026-10-20 (search API). |
| (e) `extra-data` | — | Rejected: Flathub reserves `extra-data` for non-redistributable sources; also couples the engine version to the app release. |

### Friction: what actually improves

The research doc §9.2 has the eight-step table. In short:

- **Removed:** getting the app (browser → GitHub Releases → download) and installing it (extract) —
  two of the eight steps, both file management — plus manual re-download/replace on update
  (Discover updates the app; user data survives).
- **Not removed:** the Desktop Mode trip, the manual Steam shortcut, the artwork work, the first
  desktop launch. Discover has **no** "Add to Steam" in the SteamOS 3.10.0 build (Discover 6.7.3),
  and its icon is not a Steam capsule.
- **Honest claim:** "no terminal, no archive, updates handled" — not "install from Gaming Mode".

## Key findings that drive the plan

| Finding | Label | Consequence |
| --- | --- | --- |
| `org.freedesktop.Platform` 26.08 provides every non-bundled `DT_NEEDED` of the engine, including `libjack.so.0` via PipeWire's JACK shim | Confirmed (composition) | No extra engine libraries in the manifest; verify with `ldd` on Deck. |
| `XDG_DATA_HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `XDG_STATE_HOME` are all set to `~/.var/app/<id>/…` | Confirmed (`flatpak-context.c` L4111–4116) | `paths.rs` needs no change; no hardcoded home. |
| `home:ro`, `/run/media:ro` and two `~/.var/app` grants are what IWAD discovery needs | Proposed + Confirmed linter | Three exception requests; DDL never writes outside its XDG dirs. |
| `--device=input` + `/run/udev:ro` covers Steam Input pads and `gilrs` | Confirmed (code/docs) | Avoid `--device=all`; requires `--require-version=1.16.0`. |
| Flatpak unsets `LD_PRELOAD` (run.c L669) | Confirmed (code) | No Steam overlay in DDL; Steam Input still works. |
| Discover 6.7.3 (SteamOS 3.10.0) has no "Add to Steam" | Confirmed (source) | The Steam shortcut stays manual; the guide keeps the Desktop Mode step. |
| `steam` is not on `$PATH` in the sandbox | Confirmed (code) | OSK must go through the OpenURI portal (`xdg-open steam://open/keyboard`); `Steam + X` stays the fallback. |
| Flathub's Generative AI policy forbids AI-generated/AI-assisted **manifest** content and AI-agent submission PRs | Confirmed (policy text, read 2026-10-20) | The owner must write the manifest, metainfo and every submission message; agents may research and test only. This is the top process risk. |
| App id `io.github.vittau.Doom-Deck-Launcher` maps to `github.com/vittau/doom-deck-launcher`, which resolves to the actual repo (a dash is allowed in the last component; GitHub URLs are case-insensitive) | Confirmed (policy/tooling) | No exception and no repository rename; use this id, not `…DoomDeckLauncher`. |
| No automatic migration from `~/.local/share/ddl` | Confirmed (code) | `home:ro` makes the old tree readable; offer a one-time import. |

## Draft specs (outlines in research doc §9.3)

| Draft | Scope | Size |
| --- | --- | --- |
| 059 — packaging and manifest | app id `io.github.vittau.Doom-Deck-Launcher`, manifest, runtime 26.08 + rust-stable, `cargo-sources.json`, desktop file/icon/metainfo/OARS, license, `flathub.json`, finish-args, local builder + linter in CI | L |
| 060 — sandbox-aware paths and content | `/.flatpak-info` detection, extraction-first, IWAD discovery under the grants, "path not visible" message, one-time import, fake-sandbox tests | M |
| 061 — Steam, Gaming Mode and the guide | portal OSK path, Flatpak install/shortcut/artwork/no-overlay guide text, Deck checklist | M |
| 062 — update channel split | hide app self-update when sandboxed, separate app/engine versions in About, optional "Open Discover" | S |
| 063 — Flathub submission and maintenance | human-owned PR and exception texts, verification, `x-checker-data` on the DDL release, release/runtime cadence, AI-policy rule | M |

## Risks

Research doc §9.4 has the full table with mitigations. The four that can stop the project:

1. **Flathub AI policy** — a mechanically produced manifest or PR is a rejection and a process
   change. Mitigation: owner authors, agents research/test.
2. **"Thin wrapper"/"host-dependent" inclusion clauses** — a launcher is the kind of submission
   these target. Mitigation: point at Lutris/Heroic/DoomRunner and at DDL's catalog, downloader,
   IWAD identification and profiles; only a human-written PR settles it.
3. **Permission exceptions** (`home:ro`, two `~/.var/app` grants) — can be refused or delayed.
   Mitigation: written reasons; drop the Steam/Flatpak grant first, then Heroic's.
4. **Deck runtime behaviour** — extraction, GL/audio/pad and the OSK portal are unproven on
   hardware. Mitigation: the Deck checklist in draft spec 061 before advertising the Flatpak.

## Open questions

### For the owner (decisions, not research)

1. **Channel.** Flathub as the recommended channel with the GitHub tarball as the offline/advanced
   one? Or a self-hosted `.flatpakrepo` first (no review, fast iteration, but no Discover search
   result)? Research doc §7.5.
2. **Engine.** Accept a runtime-downloaded engine (keeps D-013) and treat (c) as the fallback, or
   go straight to a source-built engine (simpler Flathub story, loses rollback)? Research doc §4.
3. **AI process.** Is the owner willing to write the manifest, metainfo and submission PR by hand?
   Without that, Flathub is not a viable channel under the current policy.
4. **OSK fallback.** If the OpenURI portal fails in Gaming Mode, accept `Steam + X` only, or add
   `--talk-name=org.freedesktop.Flatpak` and use `flatpak-spawn --host`?
5. **Runtime floor.** `--require-version=1.16.0` (needed by `--device=input`) excludes older
   distros. Accept for Deck, or keep a second build with `--device=all`?

### Needs Deck Validation

- Extracted `AppRun` inside `flatpak run`: `ldd` shows no `not found`; GL, audio and pad work.
- Gaming Mode: shortcut launch, gamescope/Wayland negotiation, controller focus, suspend/resume.
- Whether Steam's "Add a Non-Steam Game" lists the Flatpak-exported desktop entry, or the
  `/usr/bin/flatpak run <id>` recipe is required.
- Whether the OpenURI portal has a working backend in Gaming Mode.
- What the missing Steam overlay changes for focus, the `Steam + X` keyboard and the layout picker.
- Whether Discover on SteamOS opens a `.flatpakref` link and whether "Install from file" is
  reachable in Desktop Mode.

### Needs Build Validation

- `org.freedesktop.Sdk.Extension.rust-stable` version versus DDL's Rust 1.99.1 pin; vendor the
  exact toolchain as a source if the extension is older.
- A real `flatpak-builder` run: Slint build time, vendored crate size, linter clean with the three
  exceptions.

## Verification

- Read the pinned sources listed in research doc §2: Flatpak 1.18.4 `flatpak-run.c` and
  `flatpak-context.c`, Flatpak sandbox/metadata/builder docs, AppImage type-2 runtime `fedfad5`,
  AppImage docs `b29179c`, freedesktop-sdk `master` + tag `26.08.2`, the SteamOS 3.10.0 package
  manifest (`20261015.1000`), Discover `v6.7.3` source, Flathub requirements/metainfo/submission/
  maintenance pages (read 2026-10-20), and linter `finish_args.py` @ `e38c396`.
- Checked line citations against the saved sources while writing (e.g. `platform.bst` lines 8–9,
  89, 98–99, 158, 185, 201–203; `run.c` 411–416, 552–558, 669, 2068, 2415; `flatpak-context.c`
  4111–4116).
- Inspected DDL at `1edd4ec`: `platform/paths.rs`, `iwads/steam.rs`, `iwads/stores.rs`,
  `screenshots.rs`, `Cargo.lock` (slint 1.18.1, gilrs → libudev-sys, ureq/rustls, Rust 1.99.1).
- Queried Flathub's search API on 2026-10-20 for `dsda`, `dsda-doom` and `doom`: no DSDA-Doom app.
- Checked the app-id rule against the requirements page, the linter's `appid.py` and a live
  counter-example (`ai.lmstudio.lm-studio`): a dash in the last component is accepted, and
  `io.github.vittau.Doom-Deck-Launcher` resolves to the repository URL without an exception.
- Read four precedent manifests in full (Lutris, Heroic, DoomRunner, Odamex), plus the GZDoom
  `finish-args` excerpt, for permission and runtime-download practice.
- `git diff --check` before committing. No `cargo` run, no build and no test: the change is
  research-only and touches no Rust code.
- No visible window was opened, no screen was captured, and nothing was uploaded or submitted.
  Throwaway downloads (manifests, HTML, sources) live only in the gitignored `.scratch/`.

## New dependencies

None.

## Files changed

- `docs/research/flatpak.md` (new)
- `docs/reports/058-flatpak-research.md` (new)

## Spec items left open

A1–A3 are met. Nothing in the spec is unimplemented; the items that cannot be closed here are the
Deck and build validations above, which are explicitly out of scope for research. Three findings
go beyond the spec's questions and are recorded rather than acted on: the Flathub AI policy,
Discover's missing Add-to-Steam, and the missing Steam overlay.

## Proposed coordinator-owned updates

- `docs/decisions.md` **D-013**: add one clause — "inside a Flatpak (or any sandbox without
  namespaces/root mapping) DDL never attempts FUSE and always extracts first." No change to what
  is downloaded or verified.
- `docs/decisions.md` **D-002**: a second release channel (Flathub) means two artifacts from one
  tag, built by different toolchains and not byte-identical. The GitHub tarball stays the
  offline/advanced channel.
- `docs/decisions.md`: a **new decision** on distribution (Flathub recommended, GitHub tarball
  offline/advanced) and on the human-only Flathub process (AI policy). Both need the owner, not a
  worker.
- `docs/status.md` / `docs/journal.yml`: "058 research complete 2026-10-20; Flatpak is feasible
  with extraction-first engine launch; Flathub submission requires owner-authored manifest and
  PR; Deck validation pending."
- `docs/steam-deck-guide.md` (**later, after Deck validation**): a Flatpak install section, the
  `/usr/bin/flatpak run io.github.vittau.Doom-Deck-Launcher` shortcut recipe, "Compatibility tool:
  none", the manual artwork step, and the no-overlay/no-`Steam`-binary consequences.
- `README.md`: at most a pointer to that guide section once it exists; do not advertise Discover
  before the submission is accepted.

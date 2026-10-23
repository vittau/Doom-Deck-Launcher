# DoomDeckLauncher (DDL) — Project Overview

> **Status:** Architecture and agent implementation brief (revised decisions incorporated)  
> **Audience:** AI orchestrator, implementation agents, maintainers  
> **Primary platform:** Steam Deck / SteamOS (Gaming Mode first)  
> **Primary engine:** DSDA-Doom (official Linux x86-64 AppImage)  
> **Last updated:** 2026-10-23<br>
> **Revision:** 1.2 — repo/docs conventions; decisions in force live in `docs/decisions.md`

## 1. Vision

### Mission (owner, 2026-10-21)

Make DOOM with community modifications easy to find, install and play on Steam Deck,
especially for people who already own DOOM on Steam. Minimize the steps between discovering
DDL and playing a compatible mapset, with well-chosen performance and configuration defaults
from the first launch. This is DDL's central mission, not an optional polish pass.

Judge the complete journey: find DDL → install it → find the player's Steam-owned games →
choose a modification → play. Prefer automatic discovery and sensible defaults over manual
paths, technical choices and repeated setup. Keep the advanced controls available without
making them prerequisites. Measure the actual experience on the Deck; do not claim optimal
performance or a seamless install from build-only smoke tests. Existing ownership, safety,
single-engine and documentation-only Steam Input boundaries remain in force.

**DoomDeckLauncher (DDL)** is a lightweight, native, controller-first DOOM launcher and curated PWAD discovery platform designed specifically for Steam Deck. Its guiding experience is:

**Open DDL → discover a compatible mapset → install → play, entirely with the controller.**

DDL makes historically technical DOOM ecosystem concerns—IWAD choice, source-port compatibility, complevel, multiple files, loading order, configuration, and pistol start—transparent in the default path. Advanced players can inspect and override those choices without making the beginner workflow depend on them.

DDL is **not** a DOOM engine, an Electron application, a general-purpose cross-source-port frontend, a replacement for Steam, or a source of commercial IWAD assets.

### Product outcomes

- A player can use the launcher in Steam Deck Gaming Mode without switching to Desktop Mode, connecting a keyboard, opening a web browser, or entering shell commands.
- A player with legally obtained IWADs can discover vetted PWADs, download them, and launch them with known-good settings.
- DDL manages acquisition and updates of an **unmodified official DSDA-Doom AppImage**; compilation of DSDA-Doom on the Deck is never required.
- A player can toggle supported play preferences (including pistol-start behavior where technically implementable), select difficulty, and save independent profiles for different mapsets.
- DDL provisions a consistent, isolated **DSDA-Doom configuration baseline** for Steam Deck while preserving user changes; **Steam Input mappings and gyro are configured manually using README instructions**, not by DDL.
- Users can understand clearly whether a mapset is verified, unverified, or known to be incompatible with the installed DSDA-Doom version.

## 2. Decisions and non-goals

| Area | Decision |
| --- | --- |
| Name | **DoomDeckLauncher**, abbreviated **DDL**, executable `ddl` |
| Repository | New standalone GitHub repository, **not** a GitHub fork of UZDeck/GZDeck |
| Original inspiration | UZDeck and GZDeck, credited in README; avoid copying source/assets without license review |
| Platform | Steam Deck / SteamOS first, Linux x86-64; Gaming Mode is the primary interaction model |
| UI | Native UI, **Rust + Slint**, no Electron, embedded Chromium, React, or WebView |
| Controller in DDL | First-class gamepad navigation; evaluate SDL3 versus gilrs early on physical Deck |
| DSDA controls/config | **DDL-managed, versioned DSDA configuration baseline** and upgrade-safe user preferences through a `DsdaConfigManager` |
| Steam Input | **Documentation only in `README.md`**: manual recommended controls / gyro setup. No import, install, publishing, automation, API integration, or UI wizard. |
| Engine | **DSDA-Doom**, intentionally single-engine at initial release |
| Engine distribution | Official upstream DSDA-Doom Linux x86-64 **AppImage** from GitHub Releases |
| Engine lifecycle | Detect/download/verify/install/update/launch/rollback, managed inside DDL's per-user data directory |
| DDL distribution | Native Linux x86-64 application, built/tested against a suitable Steam Linux Runtime SDK; tarball initially |
| PWAD discovery | Curated catalog first; general `/idgames` browser deferred without current priority (D-041) |
| Persistence | Local files and typed versioned manifests; SQLite only if demonstrated necessary |
| Backend service | None required for initial release; host versioned catalog as static files |

### Explicit non-goals for the MVP

- Supporting UZDoom/GZDoom/PrBoom+/other engines at runtime.
- Installing, sharing, or redistributing commercial IWADs.
- Auto-playing every WAD found on the internet without verification.
- Running arbitrary installation scripts from downloaded archives.
- Auto-updating the engine in the middle of a game session.
- Integrating Proton or installing Windows-only tooling.
- Programmatically managing Steam Input, installing community layouts, importing/exporting `.vdf`, modifying Steam settings/shortcuts files, publishing a Steam Input preset, or implementing a controller setup wizard.
- Shipping a background daemon, hosted account system, or cloud saves.

> **Historical context:** UZDoom 5.x was initially considered because of its modern mod support and Flatpak availability. The selected baseline is now DSDA-Doom because the product prioritizes classic community mapsets, complevel accuracy, Boom/MBF/MBF21, and reproducible PWAD execution. Do not silently reintroduce UZDoom support.

## 3. Target user journey

1. **First run** — DDL detects usable input, checks for DSDA-Doom and IWADs in user-authorized/configured locations, explains missing prerequisites, and prepares the versioned **DDL-managed DSDA configuration** without modifying a separately installed DSDA-Doom configuration. Steam Input/gyro setup is optional and explained in `README.md`, not in an in-app wizard.
2. **Engine setup** — DDL offers an explicit **Install DSDA-Doom** action, downloads the official supported AppImage, verifies it, and stores it in its application data directory.
3. **Find IWADs** — DDL scans safe user-selected paths for IWADs. It identifies games from content/known hashes where possible, rather than trusting filename alone. User supplies commercial data.
4. **Discover** — DDL shows compatible mapsets in the specific curated categories, with cover/screenshot, credits, required base game, target complevel, and compatibility confidence. The independent GitHub catalog is checked at each startup and its source date is shown in Discover (D-038).
5. **Install** — DDL downloads from the author-approved/original mirror, verifies archive integrity when a trusted digest is known, safely extracts selected resources, resolves dependencies/load order, and creates an immutable installation manifest.
6. **Play** — DDL selects the correct IWAD, prepares a clean launch profile and effective DSDA config, supplies the correct ordered arguments (including verified `-config` behavior), and starts DSDA-Doom. The player never needs to edit command lines.
7. **Return** — DDL shows the installed item, offers **Continue / Play**, allows profile settings and replays, and preserves the exact engine/content version details used.

### Core screens

- **Home / Continue:** recently played, continue, favorites, setup status.
- **Discover:** curated categories in order: Cacowards, Megawads, Community Projects, Doomer Boards, Classics, Episodes, Challenging; Favorites follows as a personal filter. All and Featured are not navigation categories (072). General `/idgames` search is deferred without current priority (D-041); occasional other content uses manual custom setups/extras.
- **Detail:** description, authorship, previews, source, dependencies, requirements, badges for verified/support status, Install/Play.
- **Library:** installed mapsets and user-imported PWADs; filter/search with on-screen keyboard only if needed.
- **Profile / Game Options:** game and skill selection, play mode, pistol-start availability, gameplay flags, advanced load order and compatibility information.
- **Settings:** engine version/update/rollback, IWAD directories, downloads/storage, logs/diagnostics, DSDA configuration defaults/restore, and Audio with Default/installed MIDI soundfont selection. **Do not add Steam Input setup or Steam layout management screens.**

### Interaction standards

- All core actions operable via D-pad/analog stick, face buttons, triggers and shoulder buttons.
- Persistent focus indication, consistent Back/Confirm mappings, no mouse-hover dependencies.
- Design for 1280×800 resolution and readable text at handheld viewing distance.
- Correctly handle launching a game, loss/return of window focus, and controller hotplug.
- Downloads/installation are asynchronous from the UI; cancellation and error recovery must be visible.
- Ensure Steam Input translation and direct Linux gamepad APIs do not cause double input; test real Deck behavior. The user configures Steam Input via Steam itself, guided only by `README.md`.

## 4. Architecture

```text
DoomDeckLauncher (Rust binary; Slint UI)
├── ui/                 Screens, focus/navigation, localization, controller UX
├── app/                Application state, commands, user journeys
├── catalog/            Static curated manifest provider; /idgames provider later
├── recipes/            Schema, validation, versioning, compatibility metadata
├── resolver/           Dependency graph, load ordering, IWAD/engine requirements
├── downloads/          Fetching, checksum verification, caching, safe extraction
├── library/            Installed content and import indexing
├── profiles/           User preferences + immutable launch recipe snapshots
├── dsda_config/        Versioned DSDA defaults, bindings, user overrides, migration
├── engines/dsda/       Official release discovery, install/update, launch builder
├── platform/linux/     XDG paths, process management, Steam Deck integration
└── diagnostics/        Structured logs, troubleshooting/export
```

Modules map onto a Cargo workspace (`crates/ddl-core` for logic, `crates/ddl` for the Slint binary; D-005). The end-user `README.md` (including the manual Steam Input/gyro walkthrough) lives at the repository root; project docs live in `docs/`.

Separation principles:

- **Content metadata describes a requirement; the runtime applies it.** Do not embed launch commands or shell scripts inside remote manifests.
- **Catalog source ≠ verified compatibility.** A record found through the `/idgames` API is discoverable, not automatically approved for one-click play.
- **Content identity ≠ filename.** Track immutable versions, archive digests where known, extracted file hashes, and author identity.
- **User preferences ≠ mapset requirements ≠ input mapping.** Keep recipe-required IWAD/complevel/order separate from player preferences and DSDA config, and keep Steam Input completely outside the DDL config model. Allow overrides, but show when doing so deviates from author-verified conditions.
- **Engine lifecycle ≠ content lifecycle.** Updating DSDA-Doom must not silently alter installed mapsets' verified status.
- Keep the architecture modular, but **do not build a generic engine-plugin framework prematurely**; use DSDA-specific typed interfaces for the first release.
- **DDL may configure DSDA-Doom, not Steam Input.** This is an intentional product boundary, not a postponed auto-install feature.

### Candidate Rust libraries

- Slint for native declarative interface.
- `serde`, `serde_json`, `toml` or `serde_yaml` for versioned local/curated metadata (choose one format and standardize).
- `reqwest` (or a similarly mature HTTP client) for downloads/API queries.
- `tokio`/background worker strategy as necessary for UI responsiveness.
- `zip`/archive libraries with strict extraction bounds.
- `sha2` for integrity checks.
- SDL3 or `gilrs` for gamepad input; choose only after an early Deck probe.
- `tracing` for structured diagnostics.

These are starting candidates; pin maintained compatible versions during implementation.

## 5. DSDA-Doom engine management

### Upstream

- Official repository: <https://github.com/dsda-org/dsda-doom>
- Official releases: <https://github.com/dsda-org/dsda-doom/releases>
- At planning time (2026-10-14), the inspected latest release is **v0.30.1**, published **2026-10-09**, with `dsda-doom-0.30.1-linux-x86_64.appimage`.
- **Do not hardcode 'latest' or this release version.** Discover release and assets dynamically; pin/test supported versions.

### Desired lifecycle

1. Query/cache upstream GitHub release metadata (observe rate limits and offline mode).
2. Select the official Linux x86-64 AppImage for a supported release.
3. Download to a temporary file and validate final size/digest/signature **where upstream provides trusted verification material**. HTTPS alone is not a substitute for independently supplied checksums. For DDL-curated versions, maintain vetted SHA-256 hashes in a signed or review-controlled catalog where possible.
4. Atomically move the file into a versioned per-user engine directory and make it executable.
5. Perform a smoke test (`--help` or another confirmed safe option; verify upstream CLI semantics first).
6. Record installed version/path/hash/source and retain at least one known-good version for rollback.
7. Launch without a shell via `std::process::Command`, passing arguments as separate typed tokens.
8. Prompt for updates; do not replace an engine currently running. Inform users when a new version has not yet been validated for curated recipes.
9. Before promotion, validate the corresponding DSDA configuration schema/baseline and run a migration dry-run on an existing user configuration; retain a rollback path for both binary **and config**.

### AppImage execution

- **Primary path:** launch the official AppImage directly, forwarding DSDA-Doom arguments.
- **Fallback:** when FUSE/AppImage runtime mounting is unavailable, extract once into a versioned directory and execute its `AppRun` entry point; preserve bundled libraries/resources. Do not strip out an ELF and assume it will work alone.
- Validate both approaches in SteamOS Desktop and Gaming Mode, with and without the Steam compatibility/runtime environment used for DDL.
- Direct AppImage execution and compatibility with Steam Linux Runtime are **hypotheses to test**, not guarantees.
- Keep the engine as an independent upstream artifact rather than a patched or privately compiled fork.

## 6. IWAD detection and game ownership

- Support legal, user-provided IWADs for supported DSDA-Doom games (start with DOOM, Ultimate DOOM, DOOM II, Final DOOM; validate expansions and alternative IWADs individually).
- Use configurable roots with safe defaults; never assume a commercial IWAD is present.
- Identify known IWADs via signatures/hashes/content metadata. Detect ambiguity and let the user choose.
- Persist the selected IWAD reference, but recheck existence and digest before launch.
- Show friendly remediation when required base-game data is missing. Never bundle copyrighted IWAD data.
- Avoid scanning the entire home directory indiscriminately; request directories through an accessible controller-first picker.

## 7. PWAD recipes and compatibility

### Fundamental policy

**A verified catalog entry is a reproducible, tested launch recipe, not merely a download link.** Recipe semantics must distinguish:

- Mandatory IWAD and accepted IWAD variants.
- Target source-port family and map format (vanilla, limit-removing, Boom, MBF, MBF21, etc.).
- **Complevel** where applicable, using actual DSDA-Doom supported values/aliases.
- All downloadable packages/resources and their trusted sources.
- Internal archive files to load, in exact intended order.
- Required DEH/BEX patches and optional patches.
- Explicit dependencies, incompatible mods, engine minimum/maximum tested versions.
- Known required runtime arguments and validated defaults.
- Compatibility evidence: tested engine version, tester, date, limitations.
- Author credits, license/redistribution policy, canonical page, and optional screenshots.

**Compatibility target and engine configuration are separate notions.** DSDA-Doom's `-complevel` is meaningful, but it cannot be inferred safely from filename or a vague “Boom-compatible” description. Test the actual behavior and requirements of each verified title. Confirm all CLI flags against the supported DSDA release.

### Example recipe — **fictional; DO NOT treat as real catalog content**

```yaml
schema_version: 1
id: example-megawad
release: "1.0"
title: Example Megawad
creators: ["Example Author"]
source_page: https://example.invalid/megawad
verification:
  status: verified
  engine: dsda-doom
  tested_engine_version: "0.30.1"
  date: "2026-10-14"
requirements:
  iwad: doom2
  compatibility:
    family: mbf21
    complevel: 21
packages:
  - id: main
    url: https://example.invalid/example.zip
    sha256: "<trusted-sha256-goes-here>"
    files:
      - path: example.wad
        role: maps
      - path: example.deh
        role: dehacked
load_order:
  - main/example.wad
  - main/example.deh
profile_defaults:
  skill: 3
  pistol_start: off
```

A production schema must formalize asset references and separate `-file` and `-deh` where required by actual DSDA-Doom CLI behavior. Do not translate this sample directly into an executable shell command.

### Dependency / load-order resolver

- Model resources as a directed acyclic graph with `requires`, `load_before`, `load_after`, and incompatible combinations.
- Topologically sort with deterministic tie-breaking; fail gracefully on cycles/missing nodes.
- Respect explicit author instructions over guesses.
- Separate IWAD position, PWAD `-file` sequence, DEH/BEX options, and auxiliary assets where DSDA-Doom CLI requires different handling.
- Track resolved launch manifest; reruns should not silently change order following catalog updates.
- Downloads can contain multiple WADs, text files, demo files, patches, and optional extras; never auto-load every file from an archive.

### Validation stages

| Stage | Meaning | User-facing outcome |
| --- | --- | --- |
| Indexed | Found in external metadata | Browse/details; launch configuration not guaranteed |
| Inspected | Archive metadata checked, candidate IWAD/files inferred | Manual review/confirmation; no unsupported claim of compatibility |
| Verified | Exact files/order/IWAD/complevel tested on a pinned DSDA-Doom version | **Install & Play** |
| Partial | Known limitations or mismatches | Visible warning / advanced override |
| Incompatible | Proven unsupported in current engine/version | Explain why; do not pretend it can run |

Unknown metadata must remain unknown; do not fabricate compatibility from heuristics.

## 8. Gameplay profiles, DSDA configuration management, and Steam Input boundaries

### Three distinct layers (never conflate them)

| Layer | Owner | Responsibilities | Delivery |
| --- | --- | --- | --- |
| PWAD launch recipe/profile | **DDL** | IWAD, exact content files/order, complevel, gameplay flags, difficulty, save/profile identity, pistol start where verified | Versioned catalog + per-user launch profile |
| **DSDA-Doom runtime configuration** | **DDL `DsdaConfigManager`** | DSDA key/button bindings, gamepad/mouse enablement, analog response/deadzones, mouse response, video/audio/HUD defaults, user changes and migrations | Isolated DSDA config file selected via supported DSDA CLI |
| **Steam Input layout** | **User, using Steam UI** | Mapping Deck buttons/triggers/trackpads, gyroscope mode, activation behavior, Steam-side sensitivity/action sets | **Step-by-step `README.md` instructions only** |

DDL must **not** install or select Steam Input layouts, publish them, invoke undocumented Steam Input APIs, modify Steam's shortcut or controller `.vdf` files, or offer a Steam Input onboarding wizard. These are **out of scope by decision**, including for the MVP. The Steam Input section of the README is guidance, not a hidden product dependency.

### 8.1 DSDA-Doom configuration policy (`DsdaConfigManager`)

**Goal:** consistent, controller-friendly DSDA-Doom behavior on a fresh Steam Deck, while respecting user customization and independently installed DSDA setups.

Responsibilities:

1. Ship **versioned template(s)** pinned to a tested DSDA release/range. Include a `steam-deck-default` baseline; specialized baseline(s) only if a real engine-specific requirement is confirmed. Do not assume Steam Input profiles correspond 1:1 with separate DSDA configs.
2. On initial setup, provision a private DDL-owned DSDA configuration file rather than changing a user's global/default DSDA configuration. Launch the AppImage using DSDA's `-config <path>` **after testing the exact flag in the supported release**.
3. Map recommended bindings (attack, use, weapon cycling, automap, menus, save/load, etc.) to DSDA's **actual** config/key names; audit defaults and ensure no duplicate/conflicting bindings when gamepad input and mouse input coexist.
4. Configure DSDA-native stick behavior (sensitivity, deadzones, inversion, acceleration, controller selection), DSDA mouse settings relevant to trackpad/gyro-as-mouse, and independently tested display/audio/HUD defaults. Prefer stable upstream defaults where practical; **do not present uncalibrated values as validated Steam Deck tuning**.
5. Allow the DSDA process to persist user adjustments made in its menus. Preserve those changes on future launches rather than regenerating the config on every run.
6. For template/engine updates, detect changes against the previous template, **three-way merge** only compatible values where feasible, back up before migration, show conflicts if needed, and support **Reset to Recommended** with confirmation. Never silently discard user changes.
7. Keep configuration profiles and WAD launch recipes separate. Prefer one global DDL-owned DSDA configuration plus explicitly supported per-profile overrides; do not clone a DSDA config per PWAD without evidence of need.
8. Record DSDA version, template version, and effective config identifier in diagnostic launch metadata; do not leak private paths in exported logs.
9. Document what is controlled by DDL versus DSDA in-game menus. DDL should not require a separate calibration UI for Steam Input; any DSDA tuning UI is optional and not a blocker for initial release.

**Config baseline example — illustrative starting point ONLY, not Deck-calibrated:** the following key names and initial values are grounded in DSDA-Doom v0.30.1 `prboom2/src/dsda/configuration.c`, but actual template format, binding lines, effectiveness and Steam Deck UX still require an on-device test. Preserve any additional required settings generated by DSDA.

```text
use_game_controller          1
use_mouse                    1
left_analog_deadzone         6556
right_analog_deadzone        6556
left_analog_sensitivity_x    100
left_analog_sensitivity_y    100
right_analog_sensitivity_x   1536
right_analog_sensitivity_y   768
analog_look_acceleration     0
dsda_mouse_acceleration      0
mouse_sensitivity_horiz      10
mouse_sensitivity_mlook      10
swap_analogs                 0
invert_analog_look           0
```

Source to validate against during implementation:
- <https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/configuration.c>
- <https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/game_controller.c>
- <https://github.com/dsda-org/dsda-doom/blob/v0.30.1/prboom2/src/dsda/settings.c>

Use supported syntax as produced/accepted by the installed DSDA-Doom version; the fragment above is a **parameter inventory**, not a complete drop-in configuration file. Independently test any preferred button mapping in DSDA and in Steam Gaming Mode.

### 8.2 Manual Steam Input recommendations: README only

`README.md` must contain **Recommended Steam Deck Controller Setup** with clear, controller-only steps in Steam: open the non-Steam `DoomDeckLauncher` shortcut → controller icon → **Edit Layout** → assign controls → configure gyro/trackpad → save/apply. Steam UI names may change; check and illustrate based on currently tested Gaming Mode. Do not require community layout discovery or provide a fictitious layout link.

Document two **suggested manual layouts**, not two configurations managed by DDL:

| Input | Classic (suggested) | Modern + Gyro (suggested) |
| --- | --- | --- |
| Left stick | Native movement | Native movement |
| Right stick | Native camera turn | Native camera turn |
| Gyroscope | Off | **Gyro to Mouse**, ideally activated by right-stick touch (test Steam UI/support) |
| Right trackpad | Mouse/menus | Mouse/precision aim/menus |
| Triggers | Fire / Use (verify assignment) | Fire / Use (verify assignment) |
| Face / shoulders / rear buttons | Contextual menus, weapon cycle, map, quick save/load | Same, with gyro toggle option if desired |

These are recommendations to document and validate, **not a promise of automatic installation**. Steam Input can emit virtual controller input and mouse input simultaneously. The DDL's DSDA baseline must enable/handle both for everyday play, and the README must explain Steam-side versus DSDA-side sensitivity and how to avoid double-mapped actions.

**Strict Mode / demo recording caveat:** in DSDA-Doom v0.30.1 source, strict-mode input restrictions apply while demo recording. Mixing actual gamepad input with gyro-as-mouse can be blocked/restricted under those conditions. Test explicitly. Document a manual alternative using Steam Input to emulate **keyboard + mouse** for players recording strict-compatible demos rather than silently disabling strict mode or promising that hybrid inputs always work.

**Steam child-process handoff:** verify that a Steam Input layout assigned to the non-Steam DDL shortcut remains active when DDL starts the DSDA AppImage and when focus returns to DDL; Steam behavior is a hardware integration acceptance criterion, **not** an implementation mandate to manipulate Steam Input.

### 8.3 Pistol start, gameplay flags, and profile isolation

- Expose friendly controls for difficulty and DSDA-Doom-supported gameplay options.
- **Pistol start requires explicit design/verification.** Initial-map start with basic inventory and **automatic reset on every subsequent map** are different behaviors. Check DSDA-Doom's actual features/CLI and persistence semantics before presenting a global toggle. If automatic per-map pistol start cannot be implemented reliably without modifying game behavior, label the limitation rather than simulating it.
- Keep recipe-required compatibility flags separate from player-chosen gameplay preferences; do not let toggling an option silently change a verified complevel or file order.
- Profiles must be scoped to a specific mapset/install and may override defaults, without corrupting underlying verified metadata.
- Keep DSDA-Doom config/autoload and save-data behavior isolated or controlled sufficiently to prevent one profile's mods/config from polluting another. Verify actual DSDA-Doom configuration flags/directories rather than assuming isolation.
- Do not indiscriminately overwrite global DSDA-Doom settings; use the DDL-owned configuration and backups.
- A launch preview/diagnostic view should expose the fully resolved engine, DSDA config source/version, IWAD, file order, complevel and flags (not needed in normal play).

### 8.4 Configuration acceptance gates

- Fresh DDL DSDA install starts with functional gamepad input using Steam Deck's standard Steam Input/controller behavior; mouse input also works as documented.
- Manually configured **Modern + Gyro** can aim using gyro-as-mouse while the Deck's left stick moves in normal (non-recording) play, without unusable double input.
- The recommended Steam Input settings can be completed from Gaming Mode using only Deck controls and README instructions, with **no integration code in DDL**.
- Player edits a DSDA option in-game, exits, relaunches: the option persists.
- Engine upgrade and baseline revision preserve user changes or present a backed-up, recoverable conflict; reset to recommended is explicit.
- DSDA-Doom independent installation/configuration remains untouched.
- With demo recording + strict mode, the known mixed-input restriction is accurately surfaced in docs/test results.

## 9. PWAD discovery, sourcing, and distribution ethics

### Sources

- **Curated DDL catalog (MVP):** versioned static manifests in the repository / release artifacts or separately published catalog. Start with roughly **20–30 manually verified and tested mapsets**, not thousands of unverified entries.
- **Doomworld /idgames:** potential future search and metadata provider: <https://www.doomworld.com/idgames/>. Validate current API contract, usage limits, file URLs and attribution before implementation.
- **Cacowards:** editorial recommendations / discovery inspiration: <https://www.doomworld.com/cacowards/>; do not imply an official catalog API or reuse images without permission.
- **Doomer Boards:** community projects and curated discovery: <https://doomer.boards.net/>; many entries need manually maintained metadata/download links.

### Safe, legitimate distribution

- Credit creators and link to canonical release pages.
- Download from authorized original sources or permitted mirrors; do not rehost files without rights.
- Respect licenses, hotlinking restrictions, service terms, and site traffic limits.
- Each download source must be allowlisted/reviewed for verified one-click recipes.
- Treat all remote catalog/archives as untrusted input. Enforce HTTPS when available, archive path traversal prevention, decompression limits, archive size caps, timeouts, redirect limits, hashes where available, and executable/script blocking.
- Cache network metadata responsibly and support offline use of installed mapsets.
- Images/screenshots in the UI must have appropriate permissions and attribution.

## 10. Local storage model (initial proposal)

Use XDG Base Directory conventions, resolving paths through Rust platform APIs rather than hardcoding username/home paths.

```text
~/.config/ddl/
  settings.json
  profiles/*.json
  dsda/active.cfg                  # DSDA-managed by its menus; originally provisioned by DDL
  dsda/backups/*.cfg              # restore/merge points
  dsda/templates/*.json           # template version/migration metadata
~/.local/share/ddl/
  engines/dsda-doom/<version>/...
  library/<content-id>/<release>/...
  catalogs/<catalog-version>/...
  installed-manifests/*.json
~/.cache/ddl/
  downloads/
  thumbnails/
  api/
~/.local/state/ddl/
  logs/
```

- DSDA config must use a **DDL-owned path** and verified `-config` invocation, with change-preserving upgrades; do not use the separately installed DSDA default config.
- Decide early whether the launch/save working directory and DSDA-Doom user config need additional per-profile paths.
- Atomic writes and schema migrations are required for user config and installation records.
- Updates should be transactional: stage, verify, promote; retain prior version until validated.
- Preserve a user's manually imported content even if a catalog entry is removed.

## 11. Build, release, and Steam Deck integration

- Produce a Linux x86-64 optimized Rust/Slint executable with required assets and any justified dynamic libraries.
- Use a **Valve Steam Linux Runtime SDK** build/test environment and document exact SDK/image digest and toolchain versions; confirm the currently appropriate runtime/version rather than assuming all runtimes are interchangeable.
- Building with a Steam Linux Runtime SDK **does not automatically activate** a runtime for a non-Steam game. Test whether a Deck can run the release directly and what happens when launched through Steam.
- First delivery: `ddl-linux-x86_64.tar.gz`, checksum, installation guide, `.desktop` entry if helpful. A Flatpak for DDL is not required.
- The player should only need to extract DDL, add it as a non-Steam game (or use an installer/desktop entry), and launch in Gaming Mode.
- **Steam Input stays manual:** `README.md` documents Classic and Modern + Gyro settings. DDL does not read/write Steam layout files, ship an automatic layout installer, or implement a Steam Input wizard.
- Test Steam Input child-process handoff and return of focus from DSDA-Doom, without depending on undocumented Steam APIs.
- CI: formatting, clippy, unit/integration tests, manifest schema tests, DSDA config template/migration fixtures, Linux builds, checksums, release artifacts.
- Testing on physical Steam Deck is mandatory for acceptance of input, graphics, AppImage/FUSE and game-launch behavior.

## 12. Reliability, security, observability

- Do not execute shell-interpreted strings assembled from filenames, catalog data, or user input.
- Never execute scripts bundled with PWAD download archives.
- Require integrity/size checks and controlled extraction paths.
- No root/sudo requirement in default path; manage only per-user files.
- Clearly distinguish network errors, missing IWAD, unsupported complevel, corrupt archive, wrong load order, DSDA launch failure, missing AppImage/FUSE, and controller-input failure.
- Use structured local diagnostic logs; redact sensitive paths/tokens when exporting.
- Report engine exit status and point users to logs without requiring a terminal.
- Keep UI responsive during downloads/hash verification/decompression.
- Cache verification metadata; allow rollback to previously working engine + manifest **and DSDA config template** combinations.

## 13. Implementation roadmap for agent orchestration

### Milestone 0 — Repository and feasibility spikes

**Goal:** resolve assumptions before building substantial UI.

Tasks:

1. Initialize independent repository, license selection, code standards, architecture ADRs, credits for UZDeck/GZDeck.
2. Validate current DSDA-Doom upstream AppImage asset naming, release API, licensing, and distribution terms.
3. Test official AppImage **on actual Steam Deck** from Gaming Mode, and FUSE vs extraction/AppRun fallback.
4. Test minimal Slint fullscreen binary built against target Steam runtime on Steam Deck.
5. Test controller with SDL3 vs gilrs and Steam Input focus/input behavior.
6. Verify DSDA-Doom 0.30.x CLI details (`-iwad`, `-file`, `-deh`, `-complevel`, `-skill`, config paths, pistol-start controls) using upstream source/docs and sample WADs.
7. Validate `/idgames` API and download licensing assumptions with a small read-only proof of concept.
8. **DSDA config spike:** verify `-config` support, initial config generation/format, exact input/button mapping keys, config persistence, and isolation from standalone DSDA installs on v0.30.x.
9. **Input interoperability spike:** test Steam Input native controller + gyro-as-mouse, strict-mode/demo recording restrictions, DSDA AppImage Steam child-process handoff, and document manual configuration steps for `README.md`.

**Exit criteria:** successful button-driven UI → official DSDA-Doom AppImage **with isolated config** → game start/exit → DDL restored, with verified input/handoff behavior and a draft README controller configuration guide.

### Milestone 1 — Core offline launcher

- Fully navigable Slint UI and Deck controller input.
- Manual engine path + automated official AppImage install.
- IWAD directory picker/discovery, local PWAD import.
- Explicit file order and complevel selection, difficulty, recent profile, last-played.
- **DsdaConfigManager v1:** create versioned DDL-owned DSDA configuration, bind it to launch via verified CLI, provision input defaults, and preserve in-game user edits.
- Stable launch-profile/config persistence and clear error states.

**Exit criteria:** launch a known-good multi-file PWAD under proper IWAD/complevel using only the controller, while using an isolated DSDA configuration whose in-game edits persist.

### Milestone 2 — Recipe engine and curated catalog

- Formal versioned recipe schema and validation tools.
- Content dependency/load-order resolver and install record hashes.
- Download manager, safe extraction, engine compatibility statuses.
- First curated **small** batch of verified titles with credits and documented test results; grow toward 20–30.
- `Install & Play` available only for fully verified entries.

**Exit criteria:** install and play a representative vanilla/Boom/MBF21 set without manual configuration; incompatibilities signaled honestly.

### Milestone 3 — Polished Gaming Mode UX

- Featured/Library/Continue navigation, images, feedback, onboarding.
- Optional game options/pistol-start support **after verification**.
- Engine update and rollback, **DSDA config template migrations/backups/reset-to-recommended**, offline-mode and interrupted-download recovery.
- Publish and test `README.md` manual Steam Input instructions (Classic and Modern + Gyro), including strict-mode demo caveats; no Steam Input code or UI assistant.
- Controller-only usability test on real hardware.

**Exit criteria:** a fresh Steam Deck user can install DSDA-Doom, select legally held IWADs, discover a featured mapset, download, and launch without mouse/keyboard/Desktop Mode.

### Milestone 4 — Broader discovery

- `/idgames` search and browse through a separate unverified metadata tier.
- Curated Cacowards and Doomer Boards selections where rights/source details permit.
- Community-submitted recipes through reviewed pull requests with reproducible verification evidence.
- Optional Steam shortcuts and improved per-mapset progress/continue behavior, after feasibility investigation.

**Exit criteria:** broad discovery does not falsely imply universal compatibility or permit unsafe arbitrary download execution.

## 14. Workstreams and agent delegation

The orchestrator should create independent tasks with explicit interfaces and tests, rather than asking multiple agents to mutate the same files.

| Workstream | Scope | Deliverable |
| --- | --- | --- |
| A. Platform/Runtime | Slint, Gaming Mode, input, Steam Runtime | runnable controller-operated prototype and test notes |
| B. DSDA Integration | Release discovery, AppImage installation, CLI launch, fallback | engine manager crate/module and smoke tests |
| C. DSDA Config | Versioned config baseline, bindings, migrations, backups, user change preservation | `DsdaConfigManager`, template fixtures, migration tests |
| D. Content Schema | Recipe definitions, IWAD/complevel metadata, validation | versioned schema, fixtures, compatibility taxonomy |
| E. Resolver/Library | Dependency graph, file order, installs, profiles | deterministic resolution + persistent state |
| F. Content Discovery | Static curated catalog, providers, author/source links | catalog provider + sample vetted records |
| G. Downloads/Security | network, validation, zip extraction, atomic updates | robust download pipeline and adversarial tests |
| H. Product UX | controller navigation, detail cards, flows, copy | Slint screens and usability test plan |
| I. CI/Release/QA | build environment, checksums, packaging, Deck tests | repeatable release pipeline and acceptance checklist |
| J. Controller Documentation | README-only Steam Input (Classic + Modern/Gyro) instructions, strict-mode caveats | tested reader-friendly README section; **no Steam API/layout management code** |

### Orchestrator rules

1. **Start with Milestone 0**, and record findings in `docs/research/`, decisions in `docs/decisions.md` and changes in `docs/journal.yml` (see `docs/README.md`).
2. Create one spec (`docs/specs/NNN-*.md`) per independently testable outcome with dependencies, acceptance criteria, and test evidence. No external issue tracker.
3. Stabilize the core types/contracts (`EngineInstall`, `LaunchProfile`, `ContentRecipe`, `ResolutionPlan`, `InstalledContent`, `DsdaConfigTemplate`, `DsdaConfigState`) before parallel implementation.
4. Do not implement guessed engine flags, unsupported complevel mappings, or invent curated compatibility data. When uncertain, open a research/validation issue.
5. Label every item **Confirmed**, **Proposed**, or **Needs Deck Validation**; prohibit converting proposals into verified facts without evidence.
6. Use mocks/fixtures in CI but require real-device acceptance for hardware integration.
7. Prefer incremental vertical slices over creating broad architecture with no playable path.
8. All remote inputs are untrusted; test extraction and command-building boundaries.
9. Record release versions and hashes for reproducibility; update version numbers intentionally.
10. Maintain `README.md` for end users and keep this `OVERVIEW.md` as the implementation north star.
11. **Do not implement Steam Input integration.** Update README instructions if necessary; treat Steam-side control layouts as manual user configuration.
12. When changing DSDA config defaults, preserve user values and test compatibility with pinned DSDA versions rather than blindly overwriting configuration files.

## 15. Initial backlog (suggested sequence)

1. `SPIKE-001` — Verify Steam Deck execution of official DSDA AppImage and fallback.
2. `SPIKE-002` — Choose Slint renderer and controller backend after real-device spike.
3. `SPIKE-003` — Enumerate DSDA CLI/config/pistol-start behavior and actual complevel support.
4. `SPIKE-004` — Validate isolated `-config`, DSDA input/binding keys, Steam Input handoff and hybrid gyro-as-mouse; record strict-mode demo behavior.
5. `ARCH-001` — Define typed core model and schema versioning.
6. `ENGINE-001` — Upstream release metadata provider and download verification.
7. `ENGINE-002` — Versioned AppImage install/launch/rollback.
8. `CONFIG-001` — DSDA config template + isolated `-config` launch + preserved user edits.
9. `LIB-001` — IWAD identification and controller-first directory picker.
10. `LIB-002` — PWAD import and ordered launch profile.
11. `UI-001` — Gamepad focus system and full-screen shell.
12. `UI-002` — Library → profile → launch vertical slice.
13. `RECIPE-001` — Validated manifest specification and test fixtures.
14. `RECIPE-002` — Deterministic dependency and load-order resolver.
15. `DL-001` — Safe archive downloads/extraction with progress/cancel.
16. `CAT-001` — Static reviewed catalog provider and small starter collection.
17. `UI-003` — Discover/detail/install/play controller flow.
18. `PROFILE-001` — Verified gameplay options and isolation.
19. `CONFIG-002` — Migration/backup/recovery/reset logic for DSDA config templates.
20. `DOC-001` — README manual Classic and Modern + Gyro Steam Input instructions, tested on Deck.
21. `CI-001` — Steam Linux Runtime build/release/checksum pipeline.
22. `QA-001` — Real Deck end-to-end test matrix and onboarding validation.

## 16. Acceptance test matrix

At minimum, validate:

- Deck Gaming Mode cold start and navigation with built-in controls, without mouse or keyboard.
- First install, interrupted install, corrupt download, offline start, update available, update rollback.
- Official AppImage direct execution and extract/AppRun fallback when FUSE is unavailable.
- Vanilla DOOM, DOOM II, Boom complevel 9, MBF complevel 11 and MBF21 complevel 21 **using actual tested fixtures and supported engine syntax**.
- Missing IWAD and incorrect IWAD, multiple archives, required DEH patch, ordering constraints/cycles.
- Unverified content accurately labeled; verified content launchable without manual arguments.
- Profile persistence after reboot/DDL update; user override does not rewrite catalog recipe.
- Initial isolated `DsdaConfigManager` setup, correct DSDA `-config` use, in-game setting persistence, template migration and backups, explicit reset to defaults, and no mutation of standalone DSDA config.
- Manual Classic and Modern + Gyro Steam Input README steps on real Deck; no wizard, layout downloads, `vdf` editing, or Steam Input APIs.
- Gamepad + gyro-as-mouse mixed-input behavior in normal play, strict-mode demo-recording caveat, and controller/gyro input during child-AppImage launch/focus return.
- DSDA-Doom clean exit, crash, fast relaunch, and return of controller focus to DDL.
- Safe extraction rejects `../` paths, absolute paths, symlink escapes, oversized archives and executable payload policies.
- Logging and useful failure messages while offline.

## 17. Open questions / validation gates

- Which Steam Linux Runtime SDK and packaging method best supports Slint and SteamOS on current Deck hardware?
- Does DSDA's official AppImage run reliably in Gaming Mode with the chosen Steam launch settings? Are FUSE and graphics libraries available?
- Which controller API avoids conflicts with Steam Input and works consistently with Slint?
- What **exact** DSDA settings/CLI mechanisms support pistol start on *every map*, and what limitations apply?
- Which DSDA v0.30.x bindings/config defaults yield a comfortable Steam Deck baseline, and which user edits must be merged across engine upgrades?
- Which tested Steam Input action mappings/gyro activation methods work in current Deck Gaming Mode, and what manual README instructions are simplest?
- Does the Steam Input gyro-as-mouse + native gamepad combination work in the chosen non-strict play modes and preserve expected demo-recording restrictions?
- What do DSDA's actual configuration paths/autoload semantics mean for per-profile isolation?
- Does a particular WAD require fixes even when the claimed complevel matches? How will testing verify this?
- How do we curate download URLs and immutable checksums when upstream mirrors change or publish no digests?
- Are the chosen screenshots/cover arts licensed for redistribution or remote display?
- Do we need a durable DB for search/library once content volume grows?
- How does Steam re-focus DDL after DSDA exits under every supported input/runtime configuration?

These must be answered empirically, not by assumptions in implementation prompts.

## 18. Project identity and attribution

- Product name: **DoomDeckLauncher**
- Short name / binary: **DDL / `ddl`**
- GitHub repository: `vittau/Doom-Deck-Launcher`.
- Tagline: **A native DOOM mapset launcher for Steam Deck.**
- Positioning: **The easiest way to discover, configure, and play community DOOM maps on Steam Deck.**
- Acknowledge inspiration from [UZDeck](https://github.com/karkusviktor/UZDeck) and [GZDeck](https://github.com/flegald/GZDeck); DDL is an independent reimplementation. Do not claim to have reused original code unless that actually occurs; review source and asset licenses before copying.
- DOOM and Steam Deck names/trademarks belong to their respective owners. Do not imply official affiliation with id Software, Bethesda/Microsoft, Valve, Doomworld, or DSDA-Doom maintainers.

---

### One-sentence instruction for the orchestrating agent

**Deliver an incrementally playable, native Steam Deck experience in Rust and Slint that downloads and launches the official DSDA-Doom AppImage with a safe, versioned, DDL-managed DSDA configuration; makes curated community PWADs reliably playable using verified IWAD/complevel/dependency/load-order recipes; and provides readable README-only manual Steam Input/gyro guidance. Do not implement Steam Input APIs, profile import/export, or a controller setup wizard. The normal supported user journey must not require a keyboard, mouse, Desktop Mode, or manual command-line setup.**

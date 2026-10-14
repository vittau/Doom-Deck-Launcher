# Doom Deck Launcher (DDL) — Project Overview

> **Status:** Initial architecture and execution brief  
> **Audience:** AI orchestrator, implementation agents, maintainers  
> **Primary platform:** Steam Deck / SteamOS (Gaming Mode first)  
> **Primary engine:** DSDA-Doom (official Linux x86-64 AppImage)  
> **Last updated:** 2026-10-14

## 1. Vision

**Doom Deck Launcher (DDL)** is a lightweight, native, controller-first DOOM launcher and curated PWAD discovery platform designed specifically for Steam Deck. Its guiding experience is:

**Open DDL → discover a compatible mapset → install → play, entirely with the controller.**

DDL makes historically technical DOOM ecosystem concerns—IWAD choice, source-port compatibility, complevel, multiple files, loading order, configuration, and pistol start—transparent in the default path. Advanced players can inspect and override those choices without making the beginner workflow depend on them.

DDL is **not** a DOOM engine, an Electron application, a general-purpose cross-source-port frontend, a replacement for Steam, or a source of commercial IWAD assets.

### Product outcomes

- A player can use the launcher in Steam Deck Gaming Mode without switching to Desktop Mode, connecting a keyboard, opening a web browser, or entering shell commands.
- A player with legally obtained IWADs can discover vetted PWADs, download them, and launch them with known-good settings.
- DDL manages acquisition and updates of an **unmodified official DSDA-Doom AppImage**; compilation of DSDA-Doom on the Deck is never required.
- A player can toggle supported play preferences (including pistol-start behavior where technically implementable), select difficulty, and save independent profiles for different mapsets.
- Users can understand clearly whether a mapset is verified, unverified, or known to be incompatible with the installed DSDA-Doom version.

## 2. Decisions and non-goals

| Area                 | Decision                                                                                                    |
| -------------------- | ----------------------------------------------------------------------------------------------------------- |
| Name                 | **Doom Deck Launcher**, abbreviated **DDL**, executable `ddl`                                               |
| Repository           | New standalone GitHub repository, **not** a GitHub fork of UZDeck/GZDeck                                    |
| Original inspiration | UZDeck and GZDeck, credited in README; avoid copying source/assets without license review                   |
| Platform             | Steam Deck / SteamOS first, Linux x86-64; Gaming Mode is the primary interaction model                      |
| UI                   | Native UI, **Rust + Slint**, no Electron, embedded Chromium, React, or WebView                              |
| Controller           | First-class gamepad support; evaluate SDL3 versus gilrs early on physical Deck                              |
| Engine               | **DSDA-Doom**, intentionally single-engine at initial release                                               |
| Engine distribution  | Official upstream DSDA-Doom Linux x86-64 **AppImage** from GitHub Releases                                  |
| Engine lifecycle     | Detect/download/verify/install/update/launch/rollback, managed inside DDL's per-user data directory         |
| DDL distribution     | Native Linux x86-64 application, built/tested against a suitable Steam Linux Runtime SDK; tarball initially |
| PWAD discovery       | Small curated/verified catalog initially; general `/idgames` browser later                                  |
| Persistence          | Local files and typed versioned manifests; SQLite only if demonstrated necessary                            |
| Backend service      | None required for initial release; host versioned catalog as static files                                   |

### Explicit non-goals for the MVP

- Supporting UZDoom/GZDoom/PrBoom+/other engines at runtime.
- Installing, sharing, or redistributing commercial IWADs.
- Auto-playing every WAD found on the internet without verification.
- Running arbitrary installation scripts from downloaded archives.
- Auto-updating the engine in the middle of a game session.
- Integrating Proton or installing Windows-only tooling.
- Shipping a background daemon, hosted account system, or cloud saves.

> **Historical context:** UZDoom 5.x was initially considered because of its modern mod support and Flatpak availability. The selected baseline is now DSDA-Doom because the product prioritizes classic community mapsets, complevel accuracy, Boom/MBF/MBF21, and reproducible PWAD execution. Do not silently reintroduce UZDoom support.

## 3. Target user journey

1. **First run** — DDL detects Steam Deck controls, checks for DSDA-Doom, checks for IWADs in user-authorized/configured locations, and explains missing prerequisites.
2. **Engine setup** — DDL offers an explicit **Install DSDA-Doom** action, downloads the official supported AppImage, verifies it, and stores it in its application data directory.
3. **Find IWADs** — DDL scans safe user-selected paths for IWADs. It identifies games from content/known hashes where possible, rather than trusting filename alone. User supplies commercial data.
4. **Discover** — DDL shows featured beginner-friendly mapsets and verified collections with cover/screenshot, credits, required base game, target complevel, and compatibility confidence.
5. **Install** — DDL downloads from the author-approved/original mirror, verifies archive integrity when a trusted digest is known, safely extracts selected resources, resolves dependencies/load order, and creates an immutable installation manifest.
6. **Play** — DDL selects the correct IWAD, prepares a clean launch profile, supplies the correct ordered arguments, and starts DSDA-Doom. The player never needs to edit command lines.
7. **Return** — DDL shows the installed item, offers **Continue / Play**, allows profile settings and replays, and preserves the exact engine/content version details used.

### Core screens

- **Home / Continue:** recently played, continue, favorites, setup status.
- **Discover:** curated categories (Featured, Beginner Friendly, Classics, Cacowards, Doomer Boards Projects); full online search added after curated browsing works.
- **Detail:** description, authorship, previews, source, dependencies, requirements, badges for verified/support status, Install/Play.
- **Library:** installed mapsets and user-imported PWADs; filter/search with on-screen keyboard only if needed.
- **Profile / Game Options:** game and skill selection, play mode, pistol-start availability, gameplay flags, advanced load order and compatibility information.
- **Settings:** engine version/update/rollback, IWAD directories, downloads/storage, logs/diagnostics, controls.

### Interaction standards

- All core actions operable via D-pad/analog stick, face buttons, triggers and shoulder buttons.
- Persistent focus indication, consistent Back/Confirm mappings, no mouse-hover dependencies.
- Design for 1280×800 resolution and readable text at handheld viewing distance.
- Correctly handle launching a game, loss/return of window focus, and controller hotplug.
- Downloads/installation are asynchronous from the UI; cancellation and error recovery must be visible.
- Ensure Steam Input translation and direct Linux gamepad APIs do not cause double input; test real Deck behavior.

## 4. Architecture

```text
Doom-Deck-Launcher (Rust binary; Slint UI)
├── ui/                 Screens, focus/navigation, localization, controller UX
├── app/                Application state, commands, user journeys
├── catalog/            Static curated manifest provider; /idgames provider later
├── recipes/            Schema, validation, versioning, compatibility metadata
├── resolver/           Dependency graph, load ordering, IWAD/engine requirements
├── downloads/          Fetching, checksum verification, caching, safe extraction
├── library/            Installed content and import indexing
├── profiles/           User preferences + immutable launch recipe snapshots
├── engines/dsda/       Official release discovery, install/update, launch builder
├── platform/linux/     XDG paths, process management, Steam Deck integration
└── diagnostics/        Structured logs, troubleshooting/export
```

Separation principles:

- **Content metadata describes a requirement; the runtime applies it.** Do not embed launch commands or shell scripts inside remote manifests.
- **Catalog source ≠ verified compatibility.** A record found through the `/idgames` API is discoverable, not automatically approved for one-click play.
- **Content identity ≠ filename.** Track immutable versions, archive digests where known, extracted file hashes, and author identity.
- **User preferences ≠ mapset requirements.** Allow preference overrides, but show when doing so deviates from author-verified conditions.
- **Engine lifecycle ≠ content lifecycle.** Updating DSDA-Doom must not silently alter installed mapsets' verified status.
- Keep the architecture modular, but **do not build a generic engine-plugin framework prematurely**; use DSDA-specific typed interfaces for the first release.

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

| Stage        | Meaning                                                               | User-facing outcome                                               |
| ------------ | --------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Indexed      | Found in external metadata                                            | Browse/details; launch configuration not guaranteed               |
| Inspected    | Archive metadata checked, candidate IWAD/files inferred               | Manual review/confirmation; no unsupported claim of compatibility |
| Verified     | Exact files/order/IWAD/complevel tested on a pinned DSDA-Doom version | **Install & Play**                                                |
| Partial      | Known limitations or mismatches                                       | Visible warning / advanced override                               |
| Incompatible | Proven unsupported in current engine/version                          | Explain why; do not pretend it can run                            |

Unknown metadata must remain unknown; do not fabricate compatibility from heuristics.

## 8. Pistol start, gameplay flags, and profile isolation

- Expose friendly controls for difficulty and DSDA-Doom-supported gameplay options.
- **Pistol start requires explicit design/verification.** Initial-map start with basic inventory and **automatic reset on every subsequent map** are different behaviors. Check DSDA-Doom's actual features/CLI and persistence semantics before presenting a global toggle. If automatic per-map pistol start cannot be implemented reliably without modifying game behavior, label the limitation rather than simulating it.
- Keep recipe-required compatibility flags separate from player-chosen gameplay preferences.
- Profiles must be scoped to a specific mapset/install and may override defaults, without corrupting underlying verified metadata.
- Keep DSDA-Doom config/autoload and save-data behavior isolated or controlled sufficiently to prevent one profile's mods/config from polluting another. Verify actual DSDA-Doom configuration flags/directories rather than assuming isolation.
- Do not indiscriminately overwrite the user's global DSDA-Doom settings.
- A launch preview/diagnostic view should expose the fully resolved engine, IWAD, file order, complevel and flags (not needed in normal play).

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
- Integrate with Steam Input gracefully; ensure normal handoff and return from DSDA-Doom.
- CI: formatting, clippy, unit/integration tests, manifest schema tests, Linux builds, checksums, release artifacts.
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
- Cache verification metadata; allow rollback to previously working engine + manifest combinations.

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

**Exit criteria:** successful button-driven UI → official DSDA-Doom AppImage → game start/exit → DDL restored; documented constraints.

### Milestone 1 — Core offline launcher

- Fully navigable Slint UI and Deck controller input.
- Manual engine path + automated official AppImage install.
- IWAD directory picker/discovery, local PWAD import.
- Explicit file order and complevel selection, difficulty, recent profile, last-played.
- Stable profile/config persistence and clear error states.

**Exit criteria:** launch a known-good multi-file PWAD under proper IWAD/complevel using only the controller.

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
- Engine update and rollback, offline-mode and interrupted-download recovery.
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

| Workstream            | Scope                                                          | Deliverable                                           |
| --------------------- | -------------------------------------------------------------- | ----------------------------------------------------- |
| A. Platform/Runtime   | Slint, Gaming Mode, input, Steam Runtime                       | runnable controller-operated prototype and test notes |
| B. DSDA Integration   | Release discovery, AppImage installation, CLI launch, fallback | engine manager crate/module and smoke tests           |
| C. Content Schema     | Recipe definitions, IWAD/complevel metadata, validation        | versioned schema, fixtures, compatibility taxonomy    |
| D. Resolver/Library   | Dependency graph, file order, installs, profiles               | deterministic resolution + persistent state           |
| E. Content Discovery  | Static curated catalog, providers, author/source links         | catalog provider + sample vetted records              |
| F. Downloads/Security | network, validation, zip extraction, atomic updates            | robust download pipeline and adversarial tests        |
| G. Product UX         | controller navigation, detail cards, flows, copy               | Slint screens and usability test plan                 |
| H. CI/Release/QA      | build environment, checksums, packaging, Deck tests            | repeatable release pipeline and acceptance checklist  |

### Orchestrator rules

1. **Start with Milestone 0**, and record findings as short ADRs under `docs/adr/`.
2. Create one issue per independently testable outcome with dependencies, acceptance criteria, and test evidence.
3. Stabilize the core types/contracts (`EngineInstall`, `LaunchProfile`, `ContentRecipe`, `ResolutionPlan`, `InstalledContent`) before parallel implementation.
4. Do not implement guessed engine flags, unsupported complevel mappings, or invent curated compatibility data. When uncertain, open a research/validation issue.
5. Label every item **Confirmed**, **Proposed**, or **Needs Deck Validation**; prohibit converting proposals into verified facts without evidence.
6. Use mocks/fixtures in CI but require real-device acceptance for hardware integration.
7. Prefer incremental vertical slices over creating broad architecture with no playable path.
8. All remote inputs are untrusted; test extraction and command-building boundaries.
9. Record release versions and hashes for reproducibility; update version numbers intentionally.
10. Maintain `README.md` for end users and keep this `OVERVIEW.md` as the implementation north star.

## 15. Initial backlog (suggested sequence)

1. `SPIKE-001` — Verify Steam Deck execution of official DSDA AppImage and fallback.
2. `SPIKE-002` — Choose Slint renderer and controller backend after real-device spike.
3. `SPIKE-003` — Enumerate DSDA CLI/config/pistol-start behavior and actual complevel support.
4. `ARCH-001` — Define typed core model and schema versioning.
5. `ENGINE-001` — Upstream release metadata provider and download verification.
6. `ENGINE-002` — Versioned AppImage install/launch/rollback.
7. `LIB-001` — IWAD identification and controller-first directory picker.
8. `LIB-002` — PWAD import and ordered launch profile.
9. `UI-001` — Gamepad focus system and full-screen shell.
10. `UI-002` — Library → profile → launch vertical slice.
11. `RECIPE-001` — Validated manifest specification and test fixtures.
12. `RECIPE-002` — Deterministic dependency and load-order resolver.
13. `DL-001` — Safe archive downloads/extraction with progress/cancel.
14. `CAT-001` — Static reviewed catalog provider and small starter collection.
15. `UI-003` — Discover/detail/install/play controller flow.
16. `PROFILE-001` — Verified gameplay options and isolation.
17. `CI-001` — Steam Linux Runtime build/release/checksum pipeline.
18. `QA-001` — Real Deck end-to-end test matrix and onboarding validation.

## 16. Acceptance test matrix

At minimum, validate:

- Deck Gaming Mode cold start and navigation with built-in controls, without mouse or keyboard.
- First install, interrupted install, corrupt download, offline start, update available, update rollback.
- Official AppImage direct execution and extract/AppRun fallback when FUSE is unavailable.
- Vanilla DOOM, DOOM II, Boom complevel 9, MBF complevel 11 and MBF21 complevel 21 **using actual tested fixtures and supported engine syntax**.
- Missing IWAD and incorrect IWAD, multiple archives, required DEH patch, ordering constraints/cycles.
- Unverified content accurately labeled; verified content launchable without manual arguments.
- Profile persistence after reboot/DDL update; user override does not rewrite catalog recipe.
- DSDA-Doom clean exit, crash, fast relaunch, and return of controller focus to DDL.
- Safe extraction rejects `../` paths, absolute paths, symlink escapes, oversized archives and executable payload policies.
- Logging and useful failure messages while offline.

## 17. Open questions / validation gates

- Which Steam Linux Runtime SDK and packaging method best supports Slint and SteamOS on current Deck hardware?
- Does DSDA's official AppImage run reliably in Gaming Mode with the chosen Steam launch settings? Are FUSE and graphics libraries available?
- Which controller API avoids conflicts with Steam Input and works consistently with Slint?
- What **exact** DSDA settings/CLI mechanisms support pistol start on *every map*, and what limitations apply?
- What do DSDA's actual configuration paths/autoload semantics mean for per-profile isolation?
- Does a particular WAD require fixes even when the claimed complevel matches? How will testing verify this?
- How do we curate download URLs and immutable checksums when upstream mirrors change or publish no digests?
- Are the chosen screenshots/cover arts licensed for redistribution or remote display?
- Do we need a durable DB for search/library once content volume grows?
- How does Steam re-focus DDL after DSDA exits under every supported input/runtime configuration?

These must be answered empirically, not by assumptions in implementation prompts.

## 18. Project identity and attribution

- Product name: **Doom Deck Launcher**
- Short name / binary: **DDL / `ddl`**
- Proposed GitHub slug: `Doom-Deck-Launcher` (confirm availability).
- Tagline: **A native DOOM mapset launcher for Steam Deck.**
- Positioning: **The easiest way to discover, configure, and play community DOOM maps on Steam Deck.**
- Acknowledge inspiration from [UZDeck](https://github.com/karkusviktor/UZDeck) and [GZDeck](https://github.com/flegald/GZDeck); DDL is an independent reimplementation. Do not claim to have reused original code unless that actually occurs; review source and asset licenses before copying.
- DOOM and Steam Deck names/trademarks belong to their respective owners. Do not imply official affiliation with id Software, Bethesda/Microsoft, Valve, Doomworld, or DSDA-Doom maintainers.

---

### One-sentence instruction for the orchestrating agent

**Deliver an incrementally playable, native Steam Deck experience in Rust and Slint that downloads and launches the official DSDA-Doom AppImage and makes curated community PWADs reliably playable through verified IWAD/complevel/dependency/load-order recipes, with no keyboard, mouse, Desktop Mode, or manual command-line setup required for the normal user journey.**

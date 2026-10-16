# 009 — Mapset install, profiles and launch resolution (core)

**Status:** Ready  ·  **Milestone:** M2  ·  **Depends on:** 002 (catalog model), 003 (LaunchPlan),
005 (config + run guard), 006 (downloads/extraction), 007 (IWAD registry)

## Goal

Connect the core modules into the path the UI will call: from a catalog recipe and the player's
choices to installed files on disk and a validated `LaunchPlan`. After this spec, "install
AUGER;ZENITH with the AlternateBlaster add-on and play it" works end to end in `ddl-core`
(except actually running the engine, which tests fake), with every user-visible state
explainable.

## Requirements

- **R1** — **Installer** (`ddl_core::library`): `install(recipe, components)` downloads each
  needed package through spec 006's fetcher (catalog host allowlist, mirrors, cache), extracts
  exactly the members the selected components need plus every `keep = true` member, verifies
  member size and SHA-256, and places them under
  `<data>/library/<recipe id>/<release>/` atomically. Installing more components later reuses
  the cached packages. User-supplied components are never downloaded (D-014).
- **R2** — **Install manifest** (JSON, versioned, atomic): recipe id, release, recipe file
  SHA-256 at install time, packages and members with hashes and local paths, install time. A
  later catalog update never silently changes an installed manifest (OVERVIEW §7).
- **R3** — **User-supplied import:** `import_user_component(recipe, component, path)` copies the
  file into the install, records its SHA-256 and matches it against the recipe's known hashes
  when present (`matched` / `unknown`), never rejecting an unknown hash outright.
- **R4** — **Profiles** (`ddl_core::profiles`, JSON per profile): profile id, recipe id, enabled
  optional components (respecting exclusive groups and replacements), skill, pistol start,
  chosen IWAD variant (optional override), last played time. Each profile owns its save, data
  and screenshot directories under `<data>/profiles/<profile id>/` (D-011).
- **R5** — **Resolver:** `resolve(recipe, profile, iwad registry, installed manifest, engine,
  config manager) -> Result<Resolution, Blocker>` producing the ordered `-file` and `-deh`
  lists from component positions, the IWAD canonical path (007), the numeric complevel (recipe
  value; `vanilla`/aliases resolved from the identified IWAD family, D-017), `-warp` if the
  recipe sets one, `-pistolstart` per profile, config path and `-assign` overrides (005), and a
  `LaunchPlan` (003) that passes validation. `Blocker` is typed and user-explainable:
  `MissingIwad { game, accepted }`, `NotInstalled`, `ComponentNotProvided`, `EngineMissing`,
  `IncompatibleChoice`, `IntegrityFailed`.
- **R6** — **"Will launch" summary:** `Resolution::summary()` gives the UI everything the Play
  setup panel shows: base game and whether it is present, numbered load order with component
  labels, complevel with its status (proposed/verified), skill, pistol start, engine version.
- **R7** — **Catalog loading:** load all recipes from the embedded/bundled `catalog/` directory
  (compiled in or shipped next to the binary — choose and justify) and validate them with the
  002 model at startup; invalid recipes are skipped with a logged reason, never crash.
- **R8** — **Refactor:** `ddl catalog lint --fetch` uses spec 006's fetcher instead of its own
  HTTP code, so there is one hardened download path.

## Out of scope

UI (spec 010 wires it). Engine install (spec 004). Running the engine for real (tests use the
fake runner).

## Acceptance criteria

- [ ] **A1** (R1, R2) — end-to-end test with a local HTTP server serving a generated ZIP
  (nested add-on zip inside), a test recipe pointing at it, install of main only, then adding the
  add-on without re-downloading.
- [ ] **A2** (R3) — import with known and unknown hash.
- [ ] **A3** (R4, R5) — resolver tests: AUGER;ZENITH main only and with each add-on (order:
  main then add-on; exclusive group enforced), SIGIL with the compat variant replacing main,
  user-supplied soundtrack missing → `ComponentNotProvided` only when enabled, missing IWAD →
  `MissingIwad` naming the game, `vanilla` resolved per family.
- [ ] **A4** (R6) — summary snapshot for the three reference recipes.
- [ ] **A5** (R7, R8) — catalog loads the three reference recipes; lint `--fetch` tests pass on
  the shared fetcher.
- [ ] **A6** — fmt, clippy `-D warnings`, tests green; no network in tests.

## Tasks

- [ ] Installer + manifest + import
- [ ] Profiles
- [ ] Resolver + summary
- [ ] Catalog loading + lint fetch refactor

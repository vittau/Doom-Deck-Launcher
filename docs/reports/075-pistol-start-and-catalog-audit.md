# 075 — Pistol-start toggle latency and the GitHub catalog contract

Read-only audit (no implementation, no source edits). Evidence is code at `a945f5f` plus local
timings from artifacts that already existed; every number below is labelled **Measured** or
**Inferred**, and nothing is claimed for the physical Deck. No window was opened and no screen was
captured.

## Method

- Code trace only: `crates/ddl/src/runtime.rs`, `crates/ddl/src/services.rs`,
  `crates/ddl-core/src/catalog/{bundle,remote,loader}.rs`, `crates/ddl-core/build.rs`,
  `.github/workflows/{ci,release}.yml`, `crates/ddl/ui/app-window.slint`.
- Timings on the macOS arm64 dev host with an isolated XDG root (`.scratch/075/`, empty state, no
  installs), `DDL_LOG=error`, min/median of 15 runs (9 for release), debug binary `target/debug/ddl`
  and the already-built `ddl-core` debug/release test binaries. No network was used.
- The release `ddl` binary would need a cold Slint build, so its end-to-end numbers are **not
  measured**; the release catalog-parse figure comes from the pre-existing release test binary.

## 1. Pistol start: exact path and cost

### 1.1 Trace (every step is on the per-toggle path)

| Step | Where |
| --- | --- |
| Key `(Setup, 41)` clones the focused item's profile, flips `pistol_start`, calls `self.start(ui, Command::Save(profile))` | `runtime.rs:1015-1023` |
| `start()` returns immediately if `self.busy`, else sets `busy = true` and stores `pending`/`retry` | `runtime.rs:259-266` |
| `start()` **spawns a new OS thread**, then the caller's action handler ends with `self.sync(ui)` (full re-render, status becomes "Working…") | `runtime.rs:303`, `runtime.rs:539` (final `sync`) |
| Worker: `Services::open_with(env, sandbox)` — re-opens *everything*: `create_dir_all` ×4, `state_recovery::unreadable_files` (parses `app.json`, `iwads.json`, engine state and **every profile/setup JSON**), `Catalog::embedded()` (**parses + lints all 92 recipe TOMLs, no memoization**), `load_cache`, `DsdaConfigManager::new`, `ProfileStore`, `SetupStore`, `Registry::load`, `Soundfonts::new` | `runtime.rs:321`, `services.rs:895-1010`, `services.rs:915` |
| `Command::Save`: `profile.validate(recipe)`, `profiles.save` (atomic write), `state.last_profiles.insert`, `self.save()` → writes `app.json` **and** `iwads.json` | `services.rs:1990-2003`, `services.rs:1013-1022` |
| `Services::snapshot()` re-builds the **whole** catalog view: clones all 92 recipes, `profiles.list()` (reads every profile JSON), and per recipe `manifest()`, `active()` (read twice), `resolver::resolve()` for installed items, `recipe_content_sha256` (re-serialises the recipe TOML and hashes it), `classify`, plus `base_games`, `automatic_iwad_folders`, `hidden_paths`, `config.state()`, custom setups, `has_saves` per profile | `runtime.rs:324`, `services.rs:1480-1660` |
| `Event::Finished` replaces the snapshot on the UI thread and `sync()` re-renders the **entire** screen again (chip strip, cards, `FittedText` measurements) | `runtime.rs:407-413`, `runtime.rs:542` |

Nothing on this path is cached and nothing is optimistic: unlike Favorite (`runtime.rs:1310-1329`
flips the item before saving), the pistol-start row shows its old value until the full round trip
finishes. So the user's perceived delay is `open + save + snapshot + two full renders`, and the
first render after the press says "Working…" because `busy` is set (`runtime.rs:1704-1717`).

### 1.2 Measured

| Measurement (dev host, empty state) | min | median |
| --- | --- | --- |
| `ddl --version` (process start only) | 3.90 ms | 4.07 ms |
| `ddl catalog status` (start + `Services::open_with`) | 35.56 ms | 36.09 ms |
| `ddl mapset list` (open + walk 92 recipes, no snapshot) | 36.53 ms | 37.07 ms |
| `ddl doctor` (open + `snapshot()` + engine/mapset extras) | 47.15 ms | 48.27 ms |
| `Catalog::embedded()` alone, debug test binary (delta over a trivial test in the same binary) | **31.2 ms** | 31.4 ms |
| `Catalog::embedded()` alone, **release** test binary (delta over a trivial test) | **4.5 ms** | 5.9 ms |

Reading: on this host the fixed cost of *any* command is dominated by re-parsing the embedded
catalog (≈31 ms debug of ≈32 ms open; ≈4.5–6 ms release), the snapshot adds ≈8–10 ms with an empty
state, and the remainder is process/registry/config setup. The two `sync()` renders are not
measured (**Inferred** cost, UI thread).

### 1.3 What actually explains perceptible lag

1. **Re-parse of the embedded catalog on every command** (`services.rs:915`) — measured, ~5–6 ms
   release on a fast desktop core; **Inferred** 15–40 ms on the Deck's Zen 2 cores, which is the
   single largest fixed term.
2. **Full `Services` reopen per toggle** instead of reusing the open services — same parse, plus a
   complete `state_recovery` re-read of every profile/setup JSON (measured only as part of the
   open total).
3. **Full snapshot rebuild for a one-boolean change** (`services.rs:1480`), including per-recipe
   `manifest()`/`active()` filesystem reads and `recipe_content_sha256` re-serialisation —
   measured ≈8–10 ms here with no installs; **Inferred** to grow with installed mapsets and
   profiles.
4. **Two full re-renders** (busy press frame, then result frame) — **Inferred**, UI thread, the
   part a Deck GPU/CPU makes visible.
5. **`busy` gate**: while any command runs, `start()` drops every further action
   (`runtime.rs:260`); the same gate covers the *network* automatic catalog check (§2), which is a
   separate, larger stall at startup.

### 1.4 Smallest measurable fix (proposal, not implemented)

Scope: `crates/ddl-core/src/catalog/loader.rs` (+ its tests) only.

- Memoise `Catalog::embedded()` behind a `std::sync::OnceLock<Catalog>` (the embedded sources are
  `&'static str`, and `Catalog` is already `Clone`). Every command inherits the saving; no API,
  schema, UI or behaviour changes.
- Regression check (offline, no new dependency): call `Catalog::embedded()` twice, assert
  `ptr::eq`/same `skipped` and that the second call is < 10 % of the first (timing assertion
  bounded generously, or simply assert the memo is shared by comparing a `OnceLock`-backed
  identity). Plus the existing catalog suite unchanged.
- Expected effect: measured debug open 36 → ≈5 ms, release open → ≈1 ms fixed cost on this host;
  the toggle's other terms (save write, snapshot, renders) remain and are listed for a later,
  separate fix (persistent worker thread or optimistic profile patch) if the coordinator wants
  more.

## 2. Catalog pipeline vs the owner request

Owner wants: **check GitHub at every app startup**, auto update, and a Discover header subtitle
like `October 21, 2026 at 13:39 (UTC)`.

### 2.1 What the code does today

- **Trigger**: after the startup snapshot finishes, `should_auto_check()` is evaluated
  (`runtime.rs:437`) and, if true, `start(ui, Command::CatalogCheck { automatic: true })`
  (`runtime.rs:539-541`). It runs on a worker thread, silent on failure.
- **Gate**: `should_auto_check` (`runtime.rs:546-557`) requires `env.is_production()`, the
  persisted `catalog_auto_update` toggle (default on), **and** `catalog_check_due(last_check, now)`
  — i.e. `now - last_check >= CHECK_INTERVAL_SECS` = **24 hours** (`bundle.rs:55`, `bundle.rs:517`).
  `env.is_production()` is true for every real build (`services.rs:716-725`), false under `cfg(test)`.
- **Authoritative re-check**: `Services::catalog_check` repeats the same 24 h gate and the opt-out
  (`services.rs:1765-1775`) and **records the attempt before the network**
  (`services.rs:1779-1781`), so a failure still throttles for 24 h.
- **Cost of the check**: it is dispatched through `start()`, so it pays a second full
  `Services::open_with` (≈30 ms debug here) + `save()` write, sets `busy` for the whole network
  window, and on success stores the cache and rebuilds the snapshot. Fetch limits come from the
  download stack: 15 s connect, 15 s response headers, 60 s body idle, `MAX_BUNDLE_BYTES` 2 MiB
  (`downloads/fetch.rs`, `catalog/bundle.rs`), so an offline Deck can hold `busy` for ~15–60 s at
  startup (**Inferred** from those constants; not measured offline here).
- **Discover header today**: `sync()` sets the status line only for playing/FirstRun/busy/
  Review/Browser (`runtime.rs:1704-1717`); on Discover/Library it is **empty**, per spec 033
  ("a header is its title alone; the status line only reports a state…"). `app-window.slint:385-388`
  renders `heading` + `status` at y=176.

### 2.2 Serial semantics (the part the coordinator asked to verify)

- **Bundle serial** = UTC Unix seconds of the pushed commit: `serial="$(git log -1 --format=%ct)"`
  in the `catalog-bundle` job (`ci.yml:296-309`), passed to `ddl catalog bundle --serial`.
- **Embedded serial** in a release binary = the **whole source commit time** of the release build:
  `serial="$(git log -1 --format=%ct)"` exported as `DDL_CATALOG_SERIAL` (`release.yml:76-86`);
  `build.rs:27-42` reads `DDL_CATALOG_SERIAL`, else `SOURCE_DATE_EPOCH`, else **0** ("unknown").
- **Comparison**: a cached bundle loads only if `bundle.catalog_serial > embedded_serial`
  (`services.rs:931`); a fetched bundle is accepted only if
  `catalog_serial > max(effective serial, embedded serial)`
  (`services.rs:1790-1793`, `bundle.rs`). Equal serial = "NotNewer".
- **Consequence A (provenance)**: the embedded number is the *app commit* time, not the *catalog
  content* time. It equals the catalog's last change only when the catalog changed in that same
  commit. A header formatted from it therefore shows "when this build's source tree was
  committed", which is defensible as "catalog source" but can be older than the catalog content
  the binary actually contains if the release tag sits behind a catalog-only commit — the mismatch
  the coordinator flagged.
- **Consequence B (rejection/downgrade risk)**:
  - Release builds (serial > 0) cannot be downgraded by a cached bundle (strict `>`). A bundle
    published from a commit **earlier** than the release commit but **not contained** in that
    release (e.g. release cut from a branch that skipped the catalog merge) is rejected as
    `NotNewer` even though its content is newer than the embedded catalog → that release only gets
    the next bundle. **Confirmed by code path; probability Inferred.**
  - Dev/local builds (`DDL_CATALOG_SERIAL`/`SOURCE_DATE_EPOCH` unset → serial **0**) accept **any**
    cached bundle with serial > 0, so an older cached bundle can shadow a newer embedded catalog
    (`services.rs:931`). Cosmetic for players, confusing for maintainers. **Confirmed by code.**
- **Fetch time is separate**: `catalog_origin.fetched_at` (`services.rs:1799-1806`) records when
  this device accepted the bundle; it is **not** what the owner asked the header to show.
  **Confirmed.**

### 2.3 Exact changes needed (for the spec; nothing implemented)

1. **Spec 019 R5 rewrite** (durable contract `docs/specs/077-…`, prepared by the coordinator):
   automatic check **on every startup** in the background, keeping the opt-out and the
   cached/built-in offline fallback; failures stay silent.
2. **Gate**: replace the 24 h predicate in both places — `runtime.rs::should_auto_check` and
   `Services::catalog_check` — with the startup policy. Keep `is_production()`; keep
   `catalog_auto_update`; decide a short restart guard (e.g. skip when the last *attempt* was
   < 10 min ago) so a restart loop cannot hammer GitHub; `state.catalog_last_check` keeps its
   "last attempt" meaning either way.
3. **Do not pay a second reopen**: either run the automatic check inside the existing Startup
   worker after the first snapshot is emitted, or give `catalog_check` a path that reuses the open
   `Services` (today the check is a separate command, `runtime.rs:539`). Also stop covering the
   network window with `busy` so navigation/actions are not dropped for 15–60 s at launch.
4. **Header data**: new pure core helper
   `ddl_core::catalog::serial_datetime(serial) -> Option<String>` producing
   `October 21, 2026 at 13:39 (UTC)` — reuse the existing `civil_from_days`/`serial_date` maths
   (`bundle.rs:521-544`) plus `serial % 86_400` for HH:MM; no new dependency; `None` for serial 0.
   UI binds `snapshot.catalog.serial` (already present in `CatalogInfo`) and owns all Slint/header
   rendering; Settings keeps or reuses the date form (`catalog_summary`, `runtime.rs:4073-4096`).
5. **Provenance fix** (backend/build): export the **catalog content** time for the embedded serial,
   i.e. `git log -1 --format=%ct -- catalog/` in `release.yml` (the `catalog-bundle` job's whole
   commit *is* the catalog commit, so it stays as is). Then the header date means "catalog content
   date" for both embedded and updated catalogs, and the floor can no longer reject a bundle newer
   than the embedded content. Decide explicitly whether the header shows serial (content) or
   `fetched_at` (device update); the owner's wording points at serial.

## 3. Scope risks found (reported, not broadened)

- **Card focus ids stop at 99**: cards are addressed `10 + item_index`
  (`runtime.rs:1876-1900`, `runtime.rs:3288`) and the only card-confirm/favorite arms are
  `10..=99` (`runtime.rs:912`, `runtime.rs:732`, `runtime.rs:1315`). The catalog now ships **92**
  recipes, and the largest category (`community-projects`, 84) is inside the 90-card window today,
  so this is **proximity risk, not a live bug**: the 91st item of any single list (or of a future
  "All" view) is unreachable for Confirm/Favorite. Needs its own spec; untouched here.
- `Snapshot.catalog` already carries `serial`, `updated`, `source_commit`, `fetched_at`,
  `auto_update`, `last_check` (`services.rs:1713-1722`), so the header work needs no schema change.

## 4. Not measured / not done

- Deck timings, GPU/compositor render cost, and the Slint re-render share of the toggle: **not
  measured** (no window, no capture; headless renders were not needed for this audit).
- Offline-catalog fetch duration at startup: **Inferred** from the download-stack constants.
- Release end-to-end magnitudes of the toggle: **not measured** (cold Slint release build avoided
  as instructed); only the release `Catalog::embedded()` figure above is measured.
- No implementation, no test changes, no coordinator-owned docs touched; the 073 commits are
  untouched.

## 5. Proposed regression checks (for the eventual specs)

- `ddl-core`: `Catalog::embedded()` memoised (second call shares the first instance) and the whole
  catalog suite still green.
- `ddl-core`: `serial_datetime(0) == None`; `serial_datetime(1_792_589_940) == Some("October 21,
  2026 at 13:39 (UTC)")`; date-only helper unchanged.
- `services`: startup check runs once per startup with the toggle on, not at all with it off, and
  records the attempt before the fetch (existing fake-clock tests adapted); no second
  `Services::open_with` on the automatic path.
- `runtime`: on Discover the status line shows the catalog datetime when idle and the busy text
  while a command runs; Settings text unchanged.
- Manual: on the Deck, measure press→row-change for the pistol-start toggle before/after the memo
  (acceptance: one frame, i.e. < 100 ms).

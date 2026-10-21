# 077 — Per-startup catalog check and Discover date (backend)

**Status:** backend implemented on `vittau/classic-memento-mori` (`36e97e2`, `9ce330c`, `845dd50`,
`ee3c7ee`, `f09c41f`). UI header binding, Slint and screenshots stay with the UI worker.

## Helper contract (UI)

```rust
ddl_core::catalog::serial_datetime(serial: u64) -> Option<String>
// Some("October 21, 2026 at 13:39 (UTC)") for 1_792_589_940; None for serial 0
```

Pure, std-only, UTC, English month, zero-padded `HH:MM`. Bind `snapshot.catalog.serial`; render your
own "unavailable" copy for `None`. Tests cover UTC, the 2026→2027 year boundary, leap day 2028 and
unknown. `serial_date` (Settings, CLI) is unchanged.

## Behaviour

- **Every startup, once (R1):** the automatic check is dispatched after the first usable snapshot by
  a dedicated runtime thread with its own channel — it never sets `busy`, never takes the state lock
  during the network call, and a per-process flag (`catalog_checked`) makes it once per start (no
  24 h and no 10 min throttle). Opt-out (`catalog_auto_update`) and production-only gating stay; a
  manual check is unchanged and always runs.
- **Adoption without stale state (R1 review):** the check returns only `StartupCatalog { updated }`.
  When a newer bundle is adopted the runtime asks for a **fresh** `Services` snapshot
  (`Command::RefreshCatalog`, writes nothing) once no command is running, so a background result can
  never overwrite newer profile/scan state. If a command is running the reload waits for it.
- **State merge (R1 review):** `record_catalog_attempt` re-reads `app.json`, changes only
  `catalog_last_check` and writes **that file only** under the short lock; a malformed or
  newer-schema `app.json` is left untouched, a missing one is created, and `iwads.json` is never
  rewritten from the background instance.
- **Anti-rollback (R1 review):** after the fetch, the accepted serial is compared again with the
  catalog in use; a slower automatic response that is older than a manual adoption in the meantime
  is ignored and not cached.
- **Unchanged (R2):** bounded fetch, host allowlist, `min_ddl_version`, atomic cache, per-recipe drop
  and the frozen install manifest. A recipe-only bundle still arrives without an app release.
- **Stable minimum (R6):** `DEFAULT_MIN_DDL_VERSION` is now `0.0.11` with the card-reachability
  rationale; a `0.0.10` client rejects the bundle whole offline and future recipe-only updates keep
  `0.0.11`.
- **Provenance (R3):** the serial is the **catalog content time** — `git log -1 --format=%ct --
  catalog/` — in the bundle job, the tarball release and both Flatpak builds, and every one of those
  checkouts now uses `fetch-depth: 0` so the history actually reaches that commit. A commit that
  touches no catalog file does not change the displayed date.

## Review fixes (committed-code review of `f09c41f`)

- **Stale cache anti-rollback:** adoption compares the accepted serial with the serial effective on
  disk *at that moment* (cached bundle, this instance, embedded fallback), so a slower automatic
  response can never overwrite a newer catalog another process adopted during the request.
  `an_older_automatic_response_never_rolls_back_a_newer_catalog` proves the cache keeps the newer
  serial.
- **Lost queued reload:** `catalog_reload_pending` is cleared only when `RefreshCatalog` is actually
  dispatched, and the reload goes before any queued Pistol toggle, so a busy moment can never drop
  it.
- **Malformed/newer-schema `app.json`:** the attempt recorder leaves an unreadable or
  unsupported-schema file untouched (it writes only when the file parses or is missing) and never
  rewrites `iwads.json`.

## Verification (focused, offline)

| Check | Result |
| --- | --- |
| `cargo test -p ddl-core --lib -- catalog` | 112 passed (incl. the datetime format tests) |
| `cargo test -p ddl --bin ddl -- startup` | startup tests pass |
| `cargo test -p ddl --bin ddl -- pistol startup` | 11 passed |
| `cargo clippy -p ddl-core/--all-targets -D warnings`, `cargo clippy -p ddl ...` | clean |
| `cargo fmt --all --check` | clean |

New offline tests: two separate startups inside 24 h both check; the opt-out skips; a failure is
silent, keeps the catalog and does not throttle the next startup; a newer bundle is adopted and
cached; a **stalled** fetch (loopback server blocked on a channel) with a favourite and a profile
saved meanwhile leaves both on disk, records the attempt, and never rewrites `iwads.json`; the
minimum-client test rejects `0.0.10` and accepts `0.0.11`; the format tests cover the header string.

## Limitations / open

- The Discover subtitle itself (Slint + header) is the UI worker's; this backend only guarantees the
  helper and `snapshot.catalog.serial`.
- Physical Deck behaviour of the startup check (offline, captive portal, slow network) is **Needs
  Deck Validation**; the code no longer blocks navigation or actions while it runs.
- `headless::journeys::catalog_card_ids_beyond_99_stay_reachable` fails in the merged tree for the
  UI fixture, independent of this backend change (verified with my changes stashed).
- The combined workspace suite is the coordinator's to run; only focused checks were run here.

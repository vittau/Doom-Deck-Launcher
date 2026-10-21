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

## Review fixes (final, after `97cf359`)

- **Publication lock — the earlier report was wrong:** the cache read/compare/store was *not*
  locked in `97cf359`. It now is: `fetch_and_accept_catalog` performs the network call with no lock,
  then takes one short cross-process state lock around the anti-rollback comparison, the cache write
  and the attempt record. `CatalogCheck` is no longer treated as a mutating command at the
  `execute` level, so the manual check does not hold the lock across the network and the publication
  lock is never nested.
- **Delayed-response proof:** `a_delayed_automatic_response_cannot_overwrite_a_newer_publish` stalls
  an automatic fetch, publishes serial 900 from another path, releases the response (serial 500) and
  asserts the cache still holds 900; `an_older_automatic_response_never_rolls_back_a_newer_catalog`
  covers the already-cached case.
- **Metadata is best effort:** the attempt record never erases a successful publication — the
  cached bundle and the adopted serial stay authoritative if the lock or `app.json` write fails.
  `an_unwritable_attempt_never_erases_a_successful_publication` opens services, replaces
  `app.json` with a directory, runs the startup check and asserts serial 500 is adopted, cached and
  reported.
- **Attempt recording:** success, "not newer" and fetch failure all record `catalog_last_check` with
  a read-merge-write of `app.json` only (malformed/newer-schema untouched); `iwads.json` is never
  rewritten from the background instance.

## Earlier review fixes
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

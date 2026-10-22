# 098 — A catalog update that needs a newer DDL says so

**Status:** Done · **Milestone:** M3 · **Depends on:** 019, 077, 090
**Origin:** owner, 2026-10-22, after a catalog update failed on an installed older build
(v0.0.10) and the app only offered a retry. D-025 owns the mechanism, D-038 the startup
cadence and the rule that the version gate advances only for a real capability change.

## Goal

When the published catalog is refused because this DDL is older than the client the bundle
requires, the manual catalog check says **which DDL version is needed and that DDL itself
must be updated**, instead of reporting a generic failure that invites a retry.

## Observed evidence (why this exists)

The `catalog` branch raised `min_ddl_version` twice in two days: to `0.0.11` on 2026-10-21
23:29 UTC (92 recipes) and to `0.0.12` on 2026-10-22 14:15 UTC (118 recipes, the
`original_release` capability, spec 090 R3/R4). `accept_bundle` refuses such a bundle whole
with `BundleRejection::TooNew`, which is correct: a pre-0.0.12 reader would otherwise drop
every dated recipe one by one and show a silently reduced catalog.

What reaches the user is wrong, though. `fetch_and_accept_catalog` collapses every rejection
into `Problem::Catalog`:

- Settings toast: *"Could not check for catalog updates. Your installed mapsets are safe.
  Connect and Retry."* (`crates/ddl/ui/wiring-strings.slint`, `error-catalog`);
- CLI: `ddl: the catalog update check failed`;
- the actual reason exists only in `~/.local/state/ddl/logs/ddl.log`
  (`catalog bundle needs DDL 0.0.12 or newer; this is 0.0.10`).

Retrying can never succeed on such a build, so the copy sends the user to the wrong action.

## Requirements

- **R1 — Classification.** A bundle refused whole because it needs a newer client is
  reported as such: `BundleRejection::TooNew` (naming the required version) and
  `BundleRejection::UnknownSchema` (an unknown bundle format also means this DDL is too old,
  and the design raises `min_ddl_version` with any schema change). Every other failure —
  transport, oversize, malformed JSON, storage — keeps today's generic `Problem::Catalog`
  wording, because a retry can help there.
- **R2 — UI.** Settings surfaces a distinct sentence that names the required version when
  the bundle states one, and never the "Connect and Retry" copy. The translated strings live
  in `crates/ddl/ui/wiring-strings.slint` with the rest.
- **R3 — CLI.** `ddl catalog update` still exits non-zero, and its message names the
  required version and says that DDL must be updated. No new exit status.
- **R4 — Nothing else changes.** Acceptance, per-recipe drops, the anti-rollback floor,
  `NotNewer` as "up to date", the atomic cache write, and the silent automatic startup check
  (which keeps recording the attempt and never shows a toast) all behave exactly as today.
- **R5 — Tests.** The classification is unit-tested, the message is exercised end to end
  through a loopback bundle with a too-new `min_ddl_version` (services, reaching the CLI
  message), and the UI sentence is rendered off-screen. No test touches the network.
- **R6 — No new dependency.**

## Out of scope

Self-update of DDL from inside the app, downloading or offering a newer DDL automatically,
and any change to the bundle format, the version gate, or the `catalog` branch publication.

## Acceptance criteria

- [x] **A1** (R1) — `TooNew` maps to the new problem with the bundle's `min_ddl_version`;
  `UnknownSchema` maps to it without a version; every other rejection stays `Problem::Catalog`.
- [x] **A2** (R2/R3) — the UI and the CLI name the version and ask for a DDL update; the
  retry wording is not used for this case.
- [x] **A3** (R4) — the existing catalog tests (acceptance, cache precedence, delayed
  responses, silent startup failure, `NotNewer`) pass unchanged.
- [x] **A4** (R5) — fmt, clippy `-D warnings` and the workspace tests pass.

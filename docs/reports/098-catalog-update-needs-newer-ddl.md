# Spec 098 report — a catalog update that needs a newer DDL says so

**Worker:** OpenCode, 2026-10-22 · **Branch:** `spec-098-catalog-update-needs-newer-ddl`
· **Status:** implemented and validated locally (fmt, clippy, workspace tests).

## Why

The owner's catalog update failed on an installed v0.0.10 and the only thing the app offered was
"Connect and Retry". Investigation (recorded below) showed URL, transport and acceptance logic are
all correct: the `catalog` branch raised `min_ddl_version` to `0.0.11` on 2026-10-21 23:29 UTC and to
`0.0.12` on 2026-10-22 14:15 UTC, and `accept_bundle` correctly refuses such a bundle whole with
`BundleRejection::TooNew`. The defect was the copy: every rejection collapsed into
`Problem::Catalog` ("the catalog update check failed" / *"… Connect and Retry."*), with the real
reason only in `logs/ddl.log`. Retrying can never succeed on such a build.

## Change

| File | Change |
| --- | --- |
| `crates/ddl/src/services.rs` | New `Problem::CatalogNeedsNewer { min: Option<String> }` variant and its `Display`; new `catalog_needs_newer` classifier; the rejection arm of `fetch_and_accept_catalog` uses it |
| `crates/ddl/src/runtime.rs` | `problem_copy` maps the variant; `problem_text` names the required version when the bundle states one |
| `crates/ddl/src/wiring_copy.rs` | Two `Text` variants wired to the new Slint strings |
| `crates/ddl/ui/wiring-strings.slint` | `error-catalog-needs-newer` and `error-catalog-needs-newer-unknown` |

Classification (R1): `TooNew` → `min: Some(<bundle min_ddl_version>)`; `UnknownSchema` → `min: None`
(an unknown bundle format also means this DDL is too old — the design raises `min_ddl_version` with
every schema change). `Fetch`, `Oversize`, `Malformed` and storage failures keep `Problem::Catalog`,
because a retry can help there.

User-facing text:

- CLI, knowing the version: `ddl: the catalog needs DDL 0.0.12 or newer; update DDL to get it`
- CLI, unknown format: `ddl: the catalog needs a newer DDL; update DDL to get it`
- Settings toast: *"This catalog needs DDL 0.0.12 or newer. Update DDL to get it."*
  (or *"This catalog needs a newer DDL. Update DDL to get it."* for an unknown format)

Nothing else changed (R4): acceptance, per-recipe drops, the anti-rollback floor, `NotNewer` as
"up to date", the atomic cache write and the silent automatic startup check are untouched. The
attempt is still recorded on this path.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test --workspace` | **239 app + 640 core + integration suites, all pass** |
| New: `only_a_newer_client_rejection_asks_for_a_ddl_update` | `TooNew` → version, `UnknownSchema` → no version, `Fetch`/`Oversize`/`Malformed`/`NotNewer` → `None` |
| New: `a_bundle_that_needs_a_newer_ddl_is_reported_as_an_app_update` | loopback bundle with `min_ddl_version = 99.0.0`: the manual check returns the new problem, the CLI message names `99.0.0` and "update DDL", the catalog in use and the cache are untouched, `last_check` recorded, the startup check stays silent |
| New: `a_catalog_that_needs_a_newer_ddl_asks_for_an_app_update` | off-screen render: the new sentence names `0.0.12`, never says "Retry"; `Problem::Catalog` still does |

No test touches the network; the loopback server is the existing `downloads::test_http` fixture.

## Evidence gathered during the investigation

Live checks on 2026-10-22, recorded because they justify the classification:

- `https://raw.githubusercontent.com/vittau/Doom-Deck-Launcher/catalog/catalog.json` → HTTP 200,
  525,430 bytes, `bundle_schema` 1, `catalog_serial` 1792648603 (2026-10-22 05:56 UTC),
  `source_commit` 409d6c5, `min_ddl_version` 0.0.12, 118 recipes + `tags.toml` + `hosts.toml`.
- `ddl catalog update` on a local 0.0.12 build with an empty XDG root → *"Catalog updated to
  2026-10-22 (serial 1792648603, commit 409d6c5…)"*, cache written under `<XDG data>/ddl/catalog/`.
- `catalog/hosts.toml` is byte-identical between `main` and `v0.0.12-catalog`, so no recipe is
  dropped by the embedded allowlist.
- `catalog` branch history: `min_ddl_version` was `0.0.4` (89–90 recipes) until 2026-10-21 13:26 UTC,
  `0.0.11` (92 recipes) from 2026-10-21 23:29 UTC, and `0.0.12` (118 recipes) from 2026-10-22
  14:15 UTC. A v0.0.10 build therefore worked until the 0.0.11 publish and has failed since.

## Open questions

1. **Translations.** Nothing in the repo ships a translation catalogue today (`@tr` resolves to the
   English source). If one is added, the two new strings need entries like everything else.
2. **`UnknownSchema` copy.** It maps to the version-less sentence. If a future bundle ever changes
   `bundle_schema` without raising `min_ddl_version`, the user still learns "update DDL" but not
   which version; the log carries the schema number.

## Notes for the coordinator (not changed here)

- `docs/status.md` should record this fix under the next release, together with the fact that builds
  older than 0.0.12 can no longer receive catalog updates at all (the gate is deliberate, D-038).
- **Bundle size headroom:** 525 KB for 118 recipes (≈4.4 KB per recipe) against the 2 MiB
  `MAX_BUNDLE_BYTES` cap means the gate rejects every bundle at roughly 470 recipes. Not urgent, but
  it is a hard ceiling that will arrive with catalog growth and deserves its own spec.

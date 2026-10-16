# Spec 002 implementation report

Date: 2026-10-16. Branch: `vittau/spec-002-recipes`.

## Result

Implemented schema 1 in `ddl-core::catalog`, offline lint and opt-in HTTPS fetch
verification, headless `ddl catalog lint [--fetch] [paths]`, three reference
recipes and exact host allowlists, contributor instructions/template, PR evidence
checklist and a catalog-lint CI job on every pull request.

The catalog contains AUGER;ZENITH with exclusive Alternate/Deckard Blaster
add-ons, SIGIL with its exclusive E3 compat replacement and user-supplied paid
soundtrack replacement, and SIGIL II with a user-supplied THORR replacement.
All evidence remains **Inspected**. Complevel 2/3 defaults are **Proposed** and
**Need Deck Validation**, exactly as the cited research/decisions require.

## Requirement to verification mapping

| Requirement | Verification |
| --- | --- |
| R1 schema, slug, filename | `reference_recipes_round_trip_and_lint_offline`, `every_semantic_rule_rejects_a_bad_recipe`, duplicate/IO catalog tests |
| R2 identity, credits, permission, images | Round trips; semantic rejection matrix for empty identity/authors/permission and unsafe/unpermitted images; kept credit text hashes checked by fetch |
| R3 game and IWAD variants | Typed enum/schema rejection; variant slug/duplicate tests; reference combinations assert Ultimate Doom variant. Runtime Play gate belongs to later UI/resolver work and is outside this spec's scope. |
| R4 packages/members/kept texts | Round trips; semantic size/hash/path/reference matrix; nested ZIP, exact-case, bare WAD and corrupt ZIP fetch tests; real reference fetch |
| R5 required/optional/exclusive/replacement/import | `reference_combinations_have_explicit_order_and_replacements`, semantic matrix, round trips; unknown paid identities intentionally unspecified |
| R6 every combination/order/file vs deh | Pairwise proof in `validate`; position/reference collision tests; exclusive/default/replacement combinations; `deh_order_is_separate_and_members_are_never_implicitly_loaded` |
| R7 typed compatibility/runtime | Enum/unknown-field rejection; all reviewed numeric complevels tested, placeholders rejected; aliases, warp, suggestion-only pistol_start, range/version tests |
| R8 evidence | Round trips, real/leap/invalid date, missing tester/notes/inspection evidence and inspected-vs-tested status tests |
| R9 lint/diagnostics/fetch | Semantic rule matrix, failing TOML fixtures with expected messages/lines/hints, repeated nested-key span test, duplicate recipe/empty/IO/oversize/hosts cases; fake fetch/mirror failures; traversal/symlink/bomb tests; real CLI subprocess tests with invalid UI backend |
| R10 contributor experience/CI | `CONTRIBUTING.md` includes a complete lint-clean real template plus field guide and annotated explanation; PR template requests evidence; CI job runs offline lint for all PRs (thus every catalog PR). Static review; hosted CI awaits the coordinator's PR. |
| R11 reference recipes | All three files are the round-trip/lint test inputs; real fetch checks every declared member, including optional/keep members. Branch delivery awaits coordinator merge. |

Automated tests never access the network. Live transport timeouts/redirect wiring
are checked by implementation review and the explicit manual `--fetch` run,
including Romero's real redirect to the exact allowlisted Squarespace host.
No visible windows were opened or screens captured. Existing Slint tests render
only off-screen PNGs, as required by AGENTS.md.

## Dependencies

All direct versions are exact workspace pins; Cargo.lock is updated. `ddl-core`
remains free of UI dependencies.

| Added direct dependency | Version | Reason |
| --- | --- | --- |
| serde (derive) | 1.0.229 | Typed schema and serialization round trips |
| toml | 1.1.8 | Human-authored recipes/hosts, schema errors and exact source spans |
| url | 2.5.8 | Structured HTTPS/host/redirect validation |
| semver | 1.0.28 | Exact engine evidence versions and inspected/tested ranges |
| sha2 | 0.11.1 | RustCrypto SHA-256 package/member verification |
| ureq (rustls only) | 3.4.2 | Maintained synchronous HTTPS for opt-in CLI verification; bounded redirects/timeouts |
| zip (deflate-flate2-zlib-rs only) | 9.0.4 | Maintained ZIP reader for exact nested member checks; no extraction/execution |

Maintenance/API sources: <https://docs.rs/serde/1.0.229>,
<https://docs.rs/toml/1.1.8>, <https://docs.rs/url/2.5.8>,
<https://docs.rs/semver/1.0.28>, <https://docs.rs/sha2/0.11.1>,
<https://docs.rs/ureq/3.4.2>, <https://docs.rs/zip/9.0.4>.

## Clarifications and limits

Coordinator-approved clarifications: short snake_case schema fields; only typed
`warp` and `pistol_start` defaults; explicitly inspected engine range; paid
imports without invented filename/hash; dotted known-IWAD IDs (`doom-1.9ud`).
The coordinator is updating R3's variant-id grammar separately.

Defensive implementation bounds are documented in CONTRIBUTING: 1 MiB TOML,
256 MiB package/member and aggregate decompressed bytes per package check,
10,000 ZIP entries, four nested path layers, five redirects, 60 seconds per
mirror attempt. All redirect targets must be exact HTTPS allowlisted hosts;
credential URLs, custom ports, fragments, unsafe paths and symlink entries fail.

Open questions are research/hardware follow-ups, not fabricated schema data:
paid soundtrack identity and replacement behavior need purchaser validation;
all complevels, episode menus, audio and gameplay need Deck testing. No runtime
installer, IWAD acquisition, UI, free-form launch flags or engine processes were
added. No protected coordination/state documents were edited.

## Final command output tails

All commands exited 0. The full test run passed 39 tests (15 binary unit tests,
2 real CLI integration tests and 22 core unit tests); no failures/ignores.

`cargo fmt --all --check`:

```text
(no output; exit 0)
```

`cargo clippy --workspace --all-targets -- -D warnings` (tail):

```text
   Compiling slint-build v1.18.1
   Compiling slint-macros v1.18.1
   Compiling ddl v0.1.0 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-002-recipes/crates/ddl)
    Checking slint v1.18.1
    Checking ddl-core v0.1.0 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-002-recipes/crates/ddl-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 14.46s
```

`cargo test --workspace` (result lines and tail):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4m 45s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.23s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


   Doc-tests ddl_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

`./target/debug/ddl catalog lint --fetch catalog/auger-zenith.toml catalog/sigil.toml catalog/sigil-ii.toml`:

```text
Catalog lint passed (fetched packages verified).
```

The first live fetch probes were terminated by the coordinator's GUI guard,
which the coordinator confirmed and corrected. The unchanged binary then
passed a subprocess retry and a direct CLI rerun; this was not a fetch/lint bug.

## Delivery

- `4c151da` — Add typed recipe catalog and integrity lint.
- `ba21341` — Wire headless catalog lint and contributor workflow.
- Final report/spec checklist commit follows these implementation commits.
- A1–A5 are checked; reference recipes are committed on this task branch, ready
  for the coordinator's main-branch merge. Hosted CI itself runs after PR creation.
- No implementation acceptance items remain open. Hardware/purchased-content
  checks listed above remain evidence limitations and do not claim verified play.


# Spec 020 — Recipe changes after install

Implemented R1–R6 with fixture recipes, synthetic content, loopback HTTP and off-screen Slint
rendering. No visible windows were opened and no screen was captured. No new dependencies,
engine flags, configuration keys, compatibility claims or Steam Input code were added.

## Behavior and storage

- `library::classify` compares the manifest and frozen installed recipe with the current catalog.
  `RecipeStatus` distinguishes Current, MetadataOnly, Update and Removed; Update carries explicit
  independent launch/content flags and added/removed component ids. Package/member declaration
  order does not change content identity; component load order and DEH roles do change launch.
- Metadata changes are adopted silently on startup by atomically writing the new active record,
  retaining the same files and installation time. Explicit CLI Update takes this download-free
  path too. Pending updates keep resolving against their frozen installed recipe.
- Update builds a new generation under `library/<id>/generations/`, verifies archive and member
  integrity, preserves surviving player imports, and stages every profile's legal selection.
  `library/<id>/active.json` (schema 1) contains the manifest and frozen recipe; its atomic rename
  is the publication point. Old release-directory `install.json` records remain readable.
- Pre-publication failure or cancellation leaves the previous active record and content intact.
  Unpublished directories are cleaned on ordinary error exits. After publication, the old
  generation is deleted best-effort; a durable per-install retired-directory queue lets startup
  retry cleanup after a crash or storage failure. Cleanup refuses profile paths and symlinks.
- ProfileStore JSON files remain the source of truth. Publication records pending profile ids and
  the old recipe; profile files are reconciled atomically afterwards. Removing a pending marker
  follows its profile write, so interruption simply repeats the deterministic reconciliation.
  Existing save/data/shot/run directories and their files are retained. New add-ons use their
  defaults without replacing an existing exclusive choice; dropped choices produce a notice.
  Once reconciled, deliberately disabling a new default is respected on later startup.
- `app.json` remains schema 2 with only a defaulted additive `kept_recipes` map for this feature
  (recipe id → canonical recipe hash). Older schema-1/2 files load with no dismissals, following
  spec 018's additive-field convention; favorites, folders and profile selection survive. A
  later recipe hash restores the reminder. Active manifests, profiles and notices are not stored
  in app state.
- Library and detail show Update available / No longer in the catalog. Detail retains Play setup
  as the initial action and offers Update and Keep current, including an explicit Update after
  dismissal. Content changes show “Old saves may not load in the new release.” as requested by
  the spec. Successful updates show dropped-component notices without blocking.
- `ddl mapset status [<id>]` prints classifications and summaries; `ddl mapset update <id>` uses
  the same transaction, sends progress and the save warning to stderr, and reports notices.
  Removed-catalog installs retain the installed recipe and can resolve and launch.

## Requirement evidence

| Acceptance | Evidence |
| --- | --- |
| A1 / R1 | `recipe_change_categories_and_combinations`: unchanged, credits/mirrors/tags, complevel, IWAD, order, DEH, package/member hash, release, added/removed components and combined changes. |
| A1 / R2 | `metadata_adoption_is_atomic_and_download_free`: unchanged members/time, changed identity, no HTTP requests, durable reopening. |
| A2 / R3 | `update_publishes_atomically_preserves_profiles_and_retries_cleanup_after_crash`: progress observer reopens the old record until publication; old directory removed only afterwards. Failure tests cover bad download, member verification, archive extraction, cancellation and active-record write failure. |
| A2 / R4 | Successful update checks profile identity/settings, dropped notice, new default, and sentinel bytes in all four runtime directories; `interrupted_profile_reconciliation_is_retried_from_the_active_record` tests recovery and disabled-default persistence. |
| A2 / R5 | `keep_current_is_per_hash_and_removed_install_stays_resolvable`, `removed_catalog_install_launches_through_the_fake_engine`: persisted dismissal, new-hash reminder, frozen summary and actual fake-engine launch with catalog removed. |
| A2 / cleanup | Crash simulation recreates retired content plus the durable queue and verifies retry; `retired_cleanup_refuses_profile_paths_and_symlink_escapes` checks deletion boundaries. |
| A3 / R6 | `mapset_status_and_update_follow_positional_style`, `cli_status_and_update_use_the_shared_transaction`: optional id, required id, help, bad args, classification output, update output and stderr warning. |
| A3 / UI | Production headless journey uses keys for Library → detail → Keep current → Update, checks focus and state, then renders removed-catalog Library/detail and the update notice. |

## Headless visual review

Inspected 1280×800 off-screen PNGs under `target/headless/`:

- `020-library-update.png`: update label and profile count fit inside the existing carved card.
- `020-detail-update.png`: combined release/complevel/component summary wraps cleanly; Update,
  Keep current and the old-save warning remain visible with the initial Play setup focus.
- `020-library-removed.png` and `020-detail-removed.png`: full removed-catalog label is legible;
  Play setup and existing artwork/metadata remain available.
- `020-update-notice.png`: the non-blocking removed-add-on notice fits on one line above
  Play setup; the underlying profile selector and controller focus remain visible.

Existing detail tab highlight behavior (Home highlighted outside root screens) is unchanged.
Physical controller/LCD and real-engine save behavior remain **Needs Deck Validation**; no local
fixture result is presented as hardware or upstream compatibility evidence.

## Coordinator clarifications and merge

The coordinator confirmed Uninstall does not exist yet and corrected R5: adding that flow is
out of scope (now spec 022). The coordinator required a per-mapset active record, profile-file
reconciliation after publication, and best-effort cleanup with crash recovery; these are implemented.

Merged current main cleanly in `4d17716`, including spec 017's widescreen components, catalog lint,
About credit, and the coordinator's spec correction. Protected overview/decision/status/journal
files and recipes were not edited by this worker; their changes arrived only through that merge.

## Final verification

All commands exited 0 on macOS arm64 after merging main. 411 workspace tests passed, with no
external-network tests or physical Deck validation. The successful ddl unit-test phase, including the headless journey, took 98.78 s.

```text
cargo fmt --all --check
  exit 0 (no output)

cargo clippy --workspace --all-targets -- -D warnings
  Checking ddl v0.0.4 (/Users/vitor/orca/workspaces/Doom-Deck-Launcher/spec-020-recipe-change/crates/ddl)
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.66s

cargo test --workspace
  ddl unit:       test result: ok. 74 passed; 0 failed
  catalog_cli:    test result: ok. 2 passed; 0 failed
  cli_help:       test result: ok. 2 passed; 0 failed
  logo_assets:    test result: ok. 2 passed; 0 failed
  ui_strings:     test result: ok. 1 passed; 0 failed
  ddl-core unit:  test result: ok. 311 passed; 0 failed
  dsda_launch:    test result: ok. 16 passed; 0 failed
  Doc-tests:      test result: ok. 3 passed; 0 failed
```

The nine focused recipe-change tests also pass. After the full run, the successful update fixture
was strengthened to serve a new package hash and replacement bytes over loopback (rather than
reusing the initial cached package); all nine focused tests passed again in 0.65 s. Formatting
and clippy were rechecked after that test-only improvement.

An earlier full attempt hit the existing gated headless engine-download fixture's response-header
timeout while off-screen renders ran under concurrent builds (Network instead of Cancelled).
Production timeouts were not changed; the next UI journey passed. That run exposed the old
real-binary mapset-help heading expectation, which was updated together with new status/update
help and usage-error cases. The final full workspace run above is green.

## Open items

No implementation questions or new dependencies. Physical Steam Deck validation is separate from
spec 020 A1–A4's local acceptance. No push, tag or release was performed.

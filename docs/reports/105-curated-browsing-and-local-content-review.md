# 105 — Curated browsing and local-content UX review

## Scope and outcome

Owner request, 2026-10-23: study polish of Discover, Library and occasional manual content;
consolidate current project documentation. The owner will test the Deck soon and has found
no further compatibility issue. D-041 defers `/idgames` browsing without current priority.

**Confirmed:** this is a source/spec/test review and documentation update. No application
code, UI, catalog recipe, dependency, workflow or release was changed. Proposals below are
not implemented features or a dispatch-ready specification. No visible window or desktop
capture was used. Physical feel and timing remain **Needs Deck Validation**.

## Existing strengths to preserve

- Discover has specific categories, favorite filtering, deterministic chronology and shared
  Cards/List browsing. Library puts installed favorites first. Focus/identity and scroll
  behavior have focused offline tests in `headless_library.rs` (095/102).
- Manual content already has a full route: Library → New custom setup → base IWAD →
  Create setup → File order → Add files → Play. Y/Start shortcuts, multi-selection,
  content-based file checks, separate patch order and profile-scoped extras exist.
- Hash identity, blocked missing/changed files and deletion limited to DDL-owned records
  protect the player's files and reproducibility. These guarantees must survive any polish.
- Existing `local_ui_tests.rs` covers creation, multi-selection, ordering, missing/changed
  files, safe deletion, extra-file scope and Home launch validation. Tests are evidence of
  intended logic; they do not measure a user's ability to discover or operate the workflow.

## Findings and recommendations

### 1. Folder listing performs unnecessary full-file identity work

**Confirmed in source:** `Services::execute_local`, `LocalCommand::Browse`, enumerates up
to 4,096 directory entries and calls `UserFile::read` for each accepted file before emitting
`Event::Directory`. `UserFile::read` validates type and then computes the whole-file SHA-256.
Opening another folder repeats that work. It runs on a background worker, so this is not
proof of a blocked UI event loop; nevertheless the new directory is not published until the
scan finishes. Cancellation is checked between entries, not inside this hashing call.

**Proposed, highest-value code candidate:** separate bounded listing/type inspection from
identity capture. Hash only confirmed selections when adding them, and continue full identity
verification before launch. First reproduce with synthetic valid large WADs/mixed folders;
record directory-ready time, bytes read and cancellation behavior. Measure on the host for
comparison, then on Deck/SD for acceptance. No lag duration or battery claim is established.
A future spec should test that browsing does not hash every file, selection/addition does
record identity, and launch still rejects changed files. Keep all current path/type/size checks.

Source: [services.rs](../../crates/ddl/src/services.rs),
[local/kind.rs](../../crates/ddl-core/src/local/kind.rs),
[local_ui.rs](../../crates/ddl/src/local_ui.rs).

### 2. The manual route needs a user-facing explanation

**Confirmed:** the Steam Deck guide previously described install/setup/troubleshooting but
had no local-content walkthrough. The UI exposes technical labels such as New custom setup,
Base IWAD and File order; creation precedes adding the PWAD. `LocalUi::local_picker` always
starts at Home, including when adding more files later. The New/Detail actions expose no
custom-setup rename control; generated names such as Custom setup 1 can be poor identifiers
once several occasional imports accumulate. These are discoverability/effort observations,
not reported user failures.

**Completed here:** document the existing route, shortcuts, files staying in place,
profile extras and missing-file recovery in `steam-deck-guide.md`.

**Proposed:** after observing the owner perform one manual import, consider a clearer import
entry and automatic title from the selected file, while retaining editable identity and the
advanced editor. Remember the last usable folder with Home fallback; retain the existing
folder/card shortcuts. Do not introduce an import wizard merely to rename the current screens.
No new UI wording or naming/persistence behavior is approved by this report.

Source: [local_ui.rs](../../crates/ddl/src/local_ui.rs),
[local-strings.slint](../../crates/ddl/ui/local-strings.slint),
[spec026](../specs/026-local-content.md).

### 3. Missing-file recovery is safe but indirect

**Confirmed:** the issue sheet names the missing path and directs the player back to the
file list. Changed files offer explicit Use the changed file; a missing file has no direct
Locate/Replace action. Current recovery is restore the old location, or remove the old
reference and add the new one, repairing its position in the load order.

**Proposed, lower priority:** an explicit locate/replace action preserving the slot and
profile, with identity checks and confirmation for different bytes. Do not silently scan for
or substitute another file. Add tests for equal/different bytes, order, cancellation and saved
profile scope if this receives a spec. Since imports are occasional, prioritize only when
real usage makes this recovery cost relevant.

Source: `Dialog::Issue`, `local_sheet` in
[local_ui.rs](../../crates/ddl/src/local_ui.rs).

### 4. Discover already has useful navigation; more controls need evidence

**Confirmed:** `discover_items` filters category/favorites and sorts chronology;
`library_items` groups installed favorites first. No text search or general Ready/game
filter is present in these functions. Current category and favorite flows cover browsing,
but finding a known title can still require scrolling. This is a potential task cost, not
proof that search is needed for 118 recipes.

**Proposed:** observe three tasks on the Deck: find a known title, choose something playable
with an owned game, and return to a favorite. If base-game availability causes repeated
irrelevant browsing, a small owned-game filter is a candidate. If finding a known title is
actually slow, consider local catalog search separately from archive search. Preserve the
owner-requested category order and chronology. Do not add sorting/filter/search controls now
or infer that D-041 asks for local search.

Source: [runtime.rs](../../crates/ddl/src/runtime.rs),
[spec072](../specs/072-discover-specific-categories.md),
[spec090](../specs/090-catalog-order-and-dbp-filter.md).

### 5. Image coverage and releases must be distinguished

**Confirmed at source `fa4eb89`:** 118 recipes, 114 with screenshot declarations; four
without: japanese-valentines, mindblood-genesis,
spaceballs-the-quest-for-more-dbp-maps and tomatomania. At `v0.0.14-browse`, 104 recipes
have images. Subsequent image additions are in `13cade4`, `071839d`, `0754810`.
`screenshots::presentation` accepts only assets embedded in the app. The published catalog
can name the newer files, but an older app uses fallback art when it does not bundle them.

**Proposed:** include those ten assets in the next otherwise justified app release. Do not
rush a new release or add runtime image downloads merely to clear four remaining fallbacks.
The original report103 findings and report104 tag results remain historical evidence.

## Durable project-state changes

- Replaced the obsolete current status checkpoint with release 0.0.14, post-tag source and
  catalog/image distinctions; preserved historical milestone tables.
- Recorded D-041, marked specs051–053 deferred and aligned the overview/orchestration backlog.
- Updated spec/report103 with a clearly separated current screenshot-coverage note.
- Added the existing manual-content route to the Steam Deck guide and this review to the journal.

## Verification and limitations

Recounted source/tag TOML recipes and screenshot declarations; checked the current source CI
and index workflow results, published release tag, site version and remote catalog source
provenance during this assessment. Latest source CI `39866464560` passed ordinary tests,
fmt/Clippy/catalog lint; it excludes the ten exhaustive render cases (101). That run validates
the pre-existing source, not new behavior from this documentation-only change.

Documentation checks passed:

- `git diff --check`.
- All relative Markdown file links in changed documents resolve.
- TOML inventory: source 114/118 images, tagged app 104/118; exact four missing IDs checked.
- Annotated tag peels to the recorded `58e83b8c8dabdb550633faee002fa6ba977da7ab`.
- All three archive-browser specs explicitly have Deferred status.
- Ruby/Psych safe YAML parsing: the journal is a list with 119 entries.

No Cargo suite, network e2e, SDK build or performance benchmark was run; this change only
updates documentation, so those checks cannot establish additional behavior here.

Open: physical Deck checks; measured picker costs; observation of one manual import.
No implementation requirement was opened and left partially implemented. Recommendations
need a scoped spec before code. No dependency, push, tag or release.

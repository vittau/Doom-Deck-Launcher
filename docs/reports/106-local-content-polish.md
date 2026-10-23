# 106 — Responsive local browsing and recoverable custom setups

## Outcome

**Confirmed in source/offline tests:** implemented the four authorized local-content
improvements from [spec106](../specs/106-local-content-polish.md). Listing now publishes
bounded inspections without user-file hashes or a library snapshot tail; selected files
receive fresh, cancellable identities. The picker remembers a usable folder separately from
other preferences. Missing local references can be located in their existing slot with
explicit confirmation for different content. New setups get filename defaults and an
explicit Rename control. No new dependency, archive search, catalog change or engine change.

**Needs Deck Validation:** physical controller/Steam keyboard feel, Flatpak folder visibility,
SD-card/unmount behavior and timings, real-content compatibility and saves. The new LC1–LC7
rows in [deck-testing.md](../deck-testing.md) remain pending. No visible window, desktop
capture, game launch, IWAD download, network e2e, SDK build, push, tag or release was used.

## Requirement evidence

| Requirement | Implementation and evidence |
| --- | --- |
| R1 / A1 | `local::Inspection` has path/size/kind and deliberately cannot become a launch identity. It preserves regular-file/absolute-path/size, suffix/content route and WAD-directory sanity checks, reading at most 64 KiB per candidate. Browse remains on the existing single worker and checks cancellation between entries. `Refresh::None` prevents Runtime from rebuilding a snapshot and hashing unrelated custom/library files after Browse/OpenPicker/PrepareReplacement. `bounded_inspection_reads_only_prefix_and_preserves_wad_checks` uses a reader that rejects a second read or a buffer over 64 KiB. Existing kind/filter tests remain. |
| R1 confirmation / A1 | Add/create-IWAD/change-IWAD/relocation capture fresh identities on the worker, never from listing data. Hash capture checks cancellation between 64 KiB chunks, checks metadata/header again, and on Unix rejects a changed inode/device. `identity_cancel_at_chunk_and_invalid_mutation_after_listing` cancels deterministically at checkpoint three, with no timing assertion; service tests prove a bad second selection, vanished/invalid selection and pre-cancelled add publish no partial list. Existing launch verification still hashes recorded identities and rejects changed bytes. |
| R2 / A3 | Version-1 `local-picker.json` under resolved XDG config, atomic writes only after a successful local browse; independent of app/catalog/view writers. Reads require a regular file at most 16 KiB, then use a bounded reader. Missing/corrupt/future/oversized/nonregular preferences and missing/inaccessible remembered directories fall back to Home with diagnostics, without saved-state recovery. Tests use temporary XDG roots, a real permission-denied directory and Unix socket, sentinel app/view files, services reopen, failed browse, cancelled initial browse, and failed Home fallback. No retry loop or failed-path preference publication. |
| R3 / A4 | A typed target captures the complete expected setup or exact profile plus file/patch/IWAD slot. Prepare is read-only; Apply recaptures the selected identity and compares the persisted target under the state lock. Equal size/SHA-256 and kind/header repair the path without another confirmation; different content opens a Cancel-focused save warning. Tests cover middle-slot equal move/reload, different-content preparation/cancel/accept, wrong kind, non-IWAD for custom IWAD, stale setup/profile, changed replacement between prepare/apply, absent replacement and pre-cancelled apply. Catalog and base-game extras are tested with two profiles and save/player-file sentinels. Recipe members and registered IWAD copies are excluded by UI target construction. |
| R4 / A5 | Library and its Y footer say Add local mapset; New explains base game → create → files. Filename naming is authorized only by a session-local `(setup ID, placeholder name)` token and an empty content group, and uses the first non-patch stem. Unicode/separator/32-character/empty cases, patch-only, custom/existing-name and display-name collision cases are covered. The existing creation controller journey checks automatic naming. Rename reuses text/OSK infrastructure, validates 1–32 characters, supports Save/Cancel/Back, clears rename context on Back and changes only display name, preserving timestamps/IDs/profiles/order/history. Controller tests verify invalid names, cancel/confirm, Library/Home/detail refresh and unchanged profile records. The existing focus walk includes the new two-button Rename/Delete row. |
| R4 / A6 | Local strings include new guidance, actions, validation and replacement messages. Single-file pickers have a Locate file heading, no multi-add/Start hint, and choose-folder/file hints. Missing and replacement sheets focus Cancel. Busy local capture operations expose B Cancel; the controller test verifies that Back reaches their shared cancellation token without a timing race, paired with the deterministic core chunk-cancel test. Software-render frames were reviewed offscreen; clipped detail help was shortened and a clean Rename frame added. |
| A7 | Main merged before final checks. Final fmt, denied-warning workspace Clippy and one ordinary offline workspace suite: result recorded below. Historical release-state fixtures have no diff from task base `9f86c50`. |
| A8 | Updated local-content guide, physical Deck checklist and this report. No dependency additions. Spec status distinguishes host implementation from physical validation. |

Test files: `crates/ddl-core/src/local/kind.rs`, `local/picker.rs`,
`crates/ddl/src/local_polish_tests.rs`, `local_ui_tests.rs`; all player data is synthetic.
Registered-base tests explicitly mock registry identity and make no real IWAD identification claim.

## Host release diagnostic — A2

**Confirmed host diagnostic, not a Deck result or end-to-end UI benchmark.** Baseline was
captured before editing implementation at task base `9f86c50`, using a newly added example
that enumerated the synthetic folder and called the original `UserFile::read` for each file.
The final example calls the same bounded `Inspection::read` as production Browse. Both use
`cargo run --release --locked -p ddl-core --example local-inspection-diagnostic` with
`CARGO_TARGET_DIR=/Users/vitor/git/Doom-Deck-Launcher/target` on this macOS arm64 host
(macOS 27.0.1). Only one code worker used the shared target.

Fixture: eight syntactically valid, sparse 256 MiB PWADs (2 GiB total), zero lumps and a valid
empty directory offset of 12, plus one 29-byte DEH text and one rejected 9-byte `.wad` text.
No game data. The same file names/lengths/content and filesystem location were recreated by
both invocations under the system temporary directory `ddl-106-synthetic`. Nine files were
accepted in every run. Compilation time is excluded. No cache flush; run zero follows fixture
creation and runs one/two immediately repeat against warm filesystem caches. Sparse holes,
small directory size and synthetic zero-lump WADs limit extrapolation to real collections.

| Folder-inspection ready | Run 0 | Run 1 | Run 2 |
| --- | ---: | ---: | ---: |
| Original full identity path | 1121.801 ms | 1048.135 ms | 1047.278 ms |
| Bounded inspection path | 0.250 ms | 0.160 ms | 0.191 ms |

Read evidence is deterministic rather than an OS I/O counter: the original path hashes all
2 GiB of WAD payload, plus its bounded prefixes and repeated WAD headers; the new detector
has a 64 KiB buffer and the instrumented reader test rejects payload continuation. For this
fixture the loop requests/reads 524,326 bytes of prefixes after the change, versus
2,148,008,099 bytes through the original inspection + identity path. Production Browse also
reads a four-byte IWAD header for accepted files and enumerates/sorts folders/shortcuts; those
operations are outside this loop. This measurement excludes opening Services, UI scheduling,
rendering and the original full-snapshot tail. Separate service/runtime tests prove the new
light completion; no complete app-latency number is claimed.

Baseline cancellation was between entries, so a single selected/listed full-file hash had no
in-file checkpoint. After the change listing has no full-file hash; selected identity capture
checks every 64 KiB and the controlled test interrupts on the third checkpoint, after two
chunks, without persisting content. These are source/test bounds, not measured Deck cancellation
latencies. Actual SD-card listing, selection and cancellation remain **Needs Deck Validation**.

Raw logs, absolute paths:

- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/local-content-polish/.scratch/106/before.txt`
- `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/local-content-polish/.scratch/106/after.txt`

## Headless visual artifacts — A6

All are 1280×800 offscreen Slint software renders, inspected with the local image viewer tool.
No desktop screenshots. The coordinator also reviewed and accepted the revised layouts.

Root: `/Users/vitor/orca/workspaces/Doom-Deck-Launcher/local-content-polish/target/headless/`

- `local_polish_controller_rename_guidance_and_identity_refresh/106-library.png`
- `local_polish_controller_rename_guidance_and_identity_refresh/106-detail.png`
- `local_polish_controller_rename_guidance_and_identity_refresh/106-rename.png`
- `local_polish_controller_rename_guidance_and_identity_refresh/106-rename-invalid.png`
- `local_polish_controller_locate_equal_changed_cancel_and_busy_back/106-missing-locate.png`
- `local_polish_controller_locate_equal_changed_cancel_and_busy_back/106-single-picker.png`
- `local_polish_controller_locate_equal_changed_cancel_and_busy_back/106-changed-replacement.png`

## Verification and remaining work

Focused iteration: local core tests, optional preference tests, nine new service/controller
checks and the existing local UI regression journeys. The new Rename row required updating
the existing bidirectional focus-walk expectations; that focused check passed. Initial Clippy
identified a large replacement dialog variant; boxing its captured target resolved it.

Final integrated checks passed after merging main (`a954d44`; production source plus final
boxing/copy corrections checked afterward):

- `cargo fmt --all --check`.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` (shared target).
- Exactly one `cargo test --workspace --locked -- --skip headless::journeys::title_fit_ --skip headless::journeys::categories_favorites_and_screenshots`: **995 passed**, zero failures,
  exactly ten filtered cases; includes 241 app and 645 core unit tests, integration tests and
  three doctests. App unit group completed in 89.27 s; this is test runtime, not app latency.
- `git diff --check`; historical fixture-tree diff from `9f86c50` is empty.

Final logs are under the same absolute `.scratch/106/` root: `clippy.txt` and
`workspace-tests.txt`. No exhaustive/title-matrix, network e2e or SDK replay.
No physical Deck acceptance is claimed. No implementation requirement intentionally deferred;
archive search remains deferred by D-041, outside this task.

# 075 — Pistol start as a light profile change (implementation)

**Status:** implemented on `vittau/classic-memento-mori` (commits `36e97e2`, `9ce330c`, `f09c41f`).
Read-only audit: `docs/reports/075-pistol-start-and-catalog-audit.md` (kept, corrected below).

## What changed

- **`ddl-core` catalog memoization (R1):** `Catalog::embedded()` now parses and validates the
  embedded sources once per process behind a `std::sync::OnceLock` and hands every caller a clone.
  Mutable hosts/tags/recipes stay per caller; `embedded_catalog_is_reused_without_sharing_mutable_state`
  proves a mutated clone never enters the memo and that a fresh parse equals the memoized one.
- **Light Pistol path (R1/R2):** new absolute-value commands `Command::SetPistolStart { recipe_id,
  profile_id, value }` and `LocalCommand::BasePistolStart { profile_id, value }`. `Services::execute`
  intercepts both and calls `set_pistol_start` / `set_base_pistol_start`, which validate the profile
  against its recipe/base game, take the state lock and write **only** the profile file atomically.
  They return `Refresh::Profile(profile)` instead of a snapshot: no `snapshot()`, therefore no
  `resolver::resolve` → `verify_installed` whole-file SHA-256 and no `extras` verification on this
  path, and no `app.json`/`iwads.json` rewrite, so there is no partial-success rollback and no way to
  clobber a concurrent favourite/scan.
- **Runtime (narrow):** `(Setup, 41)` and the base-game `View::BaseSetup` focus 5 both call a new
  `set_pistol_start` / `set_base_pistol_start` that flips the row and the launch summary optimistically,
  dispatches the absolute value, and reverts to the last saved value on failure. A press that lands
  while a command is running is queued (`pistol_queue` / `base_pistol_queue`) and applied when the
  worker frees up, so the last requested value is never lost. `Event::ProfileSaved` replaces the
  optimistic row with the saved profile, mirrors it into the selected profile list, and patches base
  rows too.
- **No other Save path changed:** full `Command::Save`, component changes, extras and profile
  management keep the snapshot path (R3).

## Verification (focused, offline; macOS arm64)

| Check | Command | Result |
| --- | --- | --- |
| Formatting | `cargo fmt --all --check` | clean |
| Core clippy | `cargo clippy -p ddl-core --all-targets -- -D warnings` | clean |
| Binary clippy | `cargo clippy -p ddl --all-targets -- -D warnings` | clean |
| Core catalog tests | `cargo test -p ddl-core --lib -- catalog` | 112 passed |
| Pistol + startup tests | `cargo test -p ddl --bin ddl -- pistol startup` | 11 passed |
| Release-state guard | `cargo test -p ddl --bin ddl -- release_state` | passed |
| Local UI suite | `cargo test -p ddl --bin ddl -- local_ui` | 14 passed |

Deterministic checks (no wall-clock thresholds): per-`Services` counters prove the toggle builds
**0** snapshots and resolves **0** launches (`pistol_start_is_a_light_atomic_profile_change`), both
directions and a repeated press persist the final value, `skill`/`components`/`extras` survive, a
profile from another recipe is refused without writing, a read-only profile directory reports
`Problem::Storage` and leaves the stored value untouched, and the base path refuses an unknown
profile with no snapshot/resolve work. The runtime test
`the_optimistic_pistol_value_is_reverted_when_the_write_fails` drives the patch/revert pair.

**Timings (Measured, dev host, debug/release, empty state):** the audit recorded
`Catalog::embedded()` at ~31 ms debug / ~4.5–6 ms release and `Services::open_with` at ~32 ms debug;
after memoization the parse happens once per process, so every later command's fixed cost drops by
that amount. Deck responsiveness and the Slint render share are **Needs Deck Validation**.

## Corrections to the audit (as instructed)

1. The audit's "card ids stop at 99 → live bug at 92 recipes" is **wrong**: 072 already raised the
   range to `CARD..=CARD_LAST` (199) and added the regression journey. The merged tree still fails
   `headless::journeys::catalog_card_ids_beyond_99_stay_reachable`, but it fails identically with my
   changes stashed — a UI-worker fixture issue, not a backend one.
2. The audit missed that `snapshot()` re-hashes **every installed member** and every extra through
   `resolver::verify_installed` / `UserFile::verify`; that, not the catalog parse alone, dominates
   for large installed WADs. The light path removes the whole snapshot, so both costs are gone on
   the toggle; launch-time verification is untouched (R3).
3. The audit's "10 min restart throttle" suggestion is dropped: the owner wants a check at **every**
   startup (spec 077).

## Review fixes (committed-code review of `f09c41f`)

- **Profile-only atomic write under the lock, no partial success:** both light methods take the
  state lock *before* reading the profile (no read-then-lock lost update) and write only the
  profile file — never `app.json`/`iwads.json` — so a failure changes nothing else.
- **First-ever press without a saved profile:** when the profile JSON does not exist yet (the Setup
  screen shows an unsaved default), the toggle materialises that default and persists the value
  instead of refusing. Covered by `the_first_pistol_press_works_without_a_saved_profile`.
- **Base-game failure rollback:** a failed `LocalCommand::BasePistolStart` now reverts the
  optimistic base row exactly like the catalog path.
- **Base-game first press:** the base profile is materialised from the id (`base-<game>-profile-N`)
  when no JSON exists, so the toggle works on a fresh install.

## Final review round (97cf359 follow-up)

- **Intent vs persisted baseline:** the runtime keeps the latest requested value per profile
  (`pistol_intent`) separately from the last value a save confirmed (`pistol_baseline`). A save
  result first records the baseline and applies the saved profile, then re-applies the intent, so
  an older result can never invert a newer press; a queued press is re-dispatched once the worker is
  free. A failure restores the **actual last persisted value**, drops only that profile's intent
  and queue entry (other profiles keep theirs), and surfaces the error.
- **Real Confirm tests:** `rapid_confirm_presses_on_a_mapset_keep_the_last_intent` and
  `rapid_confirm_presses_on_a_base_game_keep_the_last_intent` drive `action(Confirm)` three times
  plus a failing pair through the real route, asserting the durable value, the row/launch-summary
  state and the failure rollback. They run in the existing headless harness: `headless::isolated`
  re-executes each case in a child process, which is how the suite runs several platform cases —
  **correcting** the earlier claim that "one Slint platform per thread" blocked these tests.
- **First-ever identities (review):** the catalog fallback only materialises the identity the UI
  actually shows (`<recipe>-default`); any other id is refused. The base fallback only accepts the
  identity `fresh_base_profile` mints (`base-<game>-profile-N`); the bare `base-doom2` and
  arbitrary suffixes are refused. Both are tested positively against an identified fixture
  registry and negatively for unknown ids.
## Open

- Actual controller `Confirm` routing for a rapid double press is covered by the queue logic and the
  unit tests above; a full headless journey that presses Confirm twice belongs with the UI worker's
  card/header fixtures to avoid duplicate harness ownership.
- Physical Deck latency and the render share: **Needs Deck Validation**.

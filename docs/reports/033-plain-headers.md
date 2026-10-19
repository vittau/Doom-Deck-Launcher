# 033 — Plain headers: no decorative subtitles, no caption plates

## Result

Every screen header is now its title alone. A line under a header, card or panel stays only if it
tells the player a state, a requirement or the next step. The "Screenshot: <credit>" and "Original
concept art" plates are gone from every image. Credits are plain text: Mapset detail lists
"Screenshots: <credit>" for that mapset, and About (Settings → Diagnostics / About) lists every
credit from the bundled `SOURCES.toml` files. Base-game pages no longer say "Original game" at all
and show home paths with `~`.

What changed, by requirement:

- **R2 — headers.** `app-window.slint`: the `subtitle` property became `status`. With no status the
  title is centred in the header band (no empty strip under it); with one (playing, busy, loading,
  scan help, install stage, config error, folder breadcrumb, local-setup help) the title and the
  line sit where they did. The band, the content origin and spec 030's 20 px Discover margin are
  untouched, so no content moved. `runtime.rs` / `local_ui.rs` / `shell.rs` set `status` only for the
  kept lines; the decorative strings and their `wiring_copy::Text` keys are deleted.
- **R4 — captions.** `ArtCaption`, the `credit` / `full-credit` node fields and the `band-credit`
  property are deleted. `screenshots::presentation` now returns only the image;
  `screenshots::recipe_credits` (detail) and `screenshots::all_credits` (About) return the text.
  `build.rs` reads the `credit = "…"` lines of each `SOURCES.toml` into a generated
  `SCREENSHOT_CREDITS` list (distinct, recipe-id order), so About follows the bundled files with no
  second copy to maintain. Detail's lower block was re-spaced by 28 px steps so the new line fits
  the panel; About's block moved up and its panel grows with the credit text.
- **R5 — base-game pages.** "Original game" is removed from the page header, the Library card, the
  label under the art (page and Play setup) and Home's hero (including the "Original game · Steam"
  line in the readiness panel). Paths on the page, its Play setup and the IWAD copies list go through
  `tilde()` (`~/.local/share/Steam/…`). The Library base cards shrank from 132 to 84 px now that
  they hold only the title.
- **R3 — tone.** `docs/design/ui.md` §2 "Layout and translation" has the rule; the caption paragraph,
  the Home/Discover/detail contracts were updated to match.
- **Preview shell (test-only `shell.rs`, `copy.rs`, `strings.slint` `Strings.values`).** The same
  decorative strings were removed there (enum entries and values together), so nothing removed
  remains under `crates/ddl/ui/`. `tests/ui_strings.rs` has a new test that fails if any removed
  text comes back in a `.slint` file.

## A1 — audit of subtitles, taglines and flavour lines

Ids are the Slint property (`WiringStrings` unless noted). "Preview" = the test-only static shell.

| String id | Text | Verdict | Reason |
| --- | --- | --- | --- |
| `first-sub` | Prepare the engine and bring your own base games. | remove | Restates the two panels below it. |
| `home-sub` | Your last profile, ready again. | remove | Tagline; the Continue button says it. |
| `home-intro` | Build your library and begin a new session. | remove | Slogan. |
| `discover-sub` | Curated worlds. Choose your next descent. | remove | Flavour. |
| `library-sub` | Installed worlds, ready for your next session. | remove | Restates the title. |
| `detail-sub` | Requirements, content and source. | remove | Restates the sections. |
| `setup-sub` | Review components and gameplay before launching. | remove | Restates the screen. |
| `settings-sub` | Manage your engine, base games and settings. | remove | Restates the title. |
| `install-error-sub` | Review the error and choose your next step. | remove | The error panel and Retry already say it. |
| `home-empty` | Your next mapset starts here | remove | Headline with no state; the 36 px headline is now `no-history`. |
| `concept-art` | Original concept art | remove | Caption plate (R4). |
| `screenshot-credit` | Screenshot: {} | replace | Plate removed; now `screenshots-credit` "Screenshots: {}" as plain text (detail, About). |
| `playing-sub` | DDL will be ready when your session ends. | keep | State. |
| `working`, `loading`, `cancel-pending` | Checking files… / Checking prerequisites… / Cancelling at the next safe point… | keep | State. |
| `download-help` | Receiving the archive; checks follow before installation. | keep, shortened | Install state → "Receiving the archive. Checks run before install." |
| `verify-help` | Checking integrity before these files can be used. | keep, shortened | → "Checking the files before use." |
| `extract-help` | Unpacking content into a safe staging directory. | keep, shortened | → "Unpacking the files." |
| `commit-help` | Saving the completed install; cancellation is temporarily unavailable. | keep, shortened | State plus a constraint → "Finishing the install. Cancel is unavailable for now." |
| `scan-help` | Add a folder containing your legally owned IWADs. Steam libraries are checked automatically. | keep, shortened | Requirement / next step → "Add a folder with your IWADs. Steam libraries are checked automatically." |
| `pre-check` | Check when you are ready. No update check has run. | keep, shortened | State → "No update check has run." |
| `order-help` | Recipe-required order is locked. Enabled files are listed under Will launch. | keep, shortened | State → "The recipe's file order is locked." |
| `first-help` | DDL downloads the official DSDA-Doom engine. It does not include game data. | keep | Requirement. |
| `no-history`, `no-recent`, `no-favorites-help`, `no-updates`, `no-folders`, `no-games` | empty-state lines | keep | Each names the state or the next step. |
| `profile-help`, `locked`, `advanced-help`, `pistol-help` | Setup row sublines | keep | Consequence or lock state of the control. |
| `recipe-saves`, `config-notice`, `config-ready`, `trust`, `verification` | warnings / confidence lines | keep | State; confidence must stay honest (UI contract §1). |
| `error-*`, `engine-failed`, `update-error`, `unknown-file` | error copy | keep | Each ends with the next step. |
| `uninstall-*`, `delete-profile-confirm`, `remove-confirm`, `reset-confirm`, `cancel-confirm` | dialog sublines | keep | Scope of a destructive action. |
| `LocalStrings.badge` as the Detail view's header line | Custom — not verified | remove (header only) | The page body already shows the badge; the header line repeated it. |
| `LocalStrings.auto` as the Compat view's header line | Auto — DSDA-Doom decides | remove (header only) | Repeats the first option in the list. |
| `LocalStrings.new-help`, `order-help`, `iwad-help`, `local-help` | local-setup help | keep | Next step / state. |
| Base-game header line (Base, Play setup, IWAD copies, Profiles) | Original game | remove | Decorative (R5). |
| Library base card subline | Original game | remove | R5. |
| Label under the art on the base page and its Play setup | Original game | remove | R5. |
| Home hero label and readiness-panel line for a base game | Original game · {source} | remove | R5; the source is on the base page. |
| Preview `HomeTitle` | Pick up where you left off | remove | Tagline used as a heading; the preview now shows "Home". |
| Preview `HomeSub`, `DiscoverSub`, `LibrarySub`, `SettingsSub`, `DetailSub`, `SetupSub` | Your last profile…; Curated mapsets. Clear requirements…; Installed mapsets, local imports…; Manage your engine, game folders…; Curated recipe · preview preview; Changes are saved to this mapset's profile. | remove | Same family as the production lines. |
| Preview `Missing`, `PistolHelp`, `LockedOrder` | blocker / option help | keep | State or next step. |

## Acceptance

- **A1:** the table above.
- **A2:** headless 1280×800 renders inspected after the change (software renderer to PNG, no window):
  first run, Home (continue, empty, base game), Discover, Library (with Base games), Mapset detail,
  Play setup (ready, Auger, base game), install progress, install error, Settings (Engine,
  Diagnostics / About), Playing, the quit dialog and the base IWAD page. No empty band under any
  title; Discover keeps its 20 px margin. `tests/ui_strings.rs::removed_decorative_copy_does_not_return`
  asserts none of the removed texts is in `crates/ddl/ui/*.slint`.
- **A3:** card, Home, Library, detail and band renders show the image with nothing over it;
  `headless_production.rs` asserts no node label mentions "Screenshot" / "concept art" on cards and
  that Mapset detail has the node `Screenshots: <credit>` for every catalog recipe, and the
  painted-cover fallback has no caption either. The About render shows the ten distinct credits.
  `local_ui_tests.rs` asserts no "Original game" label and a `~/` path on the base page.
- **A4:** see "Final checks" below. README renders were regenerated with
  `scripts/readme-screenshots.py` (`docs/screenshots/*.png`, 1280×800).

## Final checks

After merging `main` (through spec 034) into the branch, with the coordinator's go for the single
full-workspace run: `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets -- -D
warnings` clean; `cargo test --workspace` green (ddl 139 unit + 3 + 2 + 2 + 2 + 2 integration,
ddl-core 398, plus its integration suites). The final-run PNGs were rendered to a scratch directory
and are not tracked; the four README images in `docs/screenshots/` are tracked and refreshed.

## New dependencies

None.

## Open questions and notes for the coordinator

- **Header band height.** I kept the 88 px band and centred a lone title in it, rather than shrinking
  the band and moving the content up 32 px. The art band (400×88 on the right) needs the height and
  every headless test pins content to the current origin. If the owner wants the extra room back on
  the Deck, shrinking is a contained follow-up (band, viewport constants in `runtime.rs` /
  `local_ui.rs`, the pixel checks in `headless_production.rs`).
- **About credits will grow.** The credit text sizes its label and the panel, but About has nothing
  focusable below it, so a very long list (spec 037 adds many mapsets) would be clipped by the
  viewport. Today it is ten distinct credits and fits. A follow-up could make the credits a focusable
  row so the pane scrolls.
- **"Original game" appears nowhere** (the spec allows once at most). Base games are told apart by
  the Library "Base games" group and by their title.
- **Full path.** The `~` form is on the base pages only. The full path stays in Settings → IWAD
  folders and the folder picker; there is no focus reveal on the base page itself.
- **Playing.** The header line "DDL will be ready when your session ends." (kept, as the spec's
  example) sits above the body line and footer that both say "Game running — return here when you
  exit". Dropping one of those two duplicates would be a small copy follow-up.
- `crates/ddl/ui/assets/ember/build_assets.py` still mentions the old "Original concept art plaque"
  in its image-generation prompts. It is under the asset folder owned by spec 035, so I left it.
- Suggested protected-file notes (not edited): `docs/status.md` / `journal.yml` can record spec 033
  done; D-024 stays satisfied through the text credits.

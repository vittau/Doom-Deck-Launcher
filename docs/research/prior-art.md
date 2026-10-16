# Prior art: UZDeck and GZDeck

Research for DDL's Milestone 0 (see `OVERVIEW.md` §18). Sources were read directly from a clone of each
repository and from the GitHub API; every claim carries a link pinned to a commit where practical.

**Pins used below**

| Project | Commit read | Snapshot taken |
| --- | --- | --- |
| GZDeck | [`6fc1f1c`](https://github.com/flegald/GZDeck/tree/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5) (2024-08-13) | 2026-10-16 |
| UZDeck | [`daabebc`](https://github.com/karkusviktor/UZDeck/tree/daabebcfa8a5c21153901d0f797c366532c35b94) (2026-03-27) | 2026-10-16 |

**Labels.** *Confirmed* = observed in the pinned source or upstream metadata. *Proposed* = a DDL
position suggested here. *Needs Deck Validation* = depends on hardware we cannot test from a
development machine. Code paths are cited as `path#Lnn` against the pin above.

---

## 1. Project summaries

### GZDeck — the original

* **Purpose.** A controller-first launcher for the **Flatpak** build of **GZDoom** on Steam Deck: pick an
  IWAD, pick and order a set of mod files, launch the game, and remember the last combination.
  Explicitly **no network functionality** — "This application does not and will not have any network
  functionality" ([README](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/README.md)).
  *Confirmed.*
* **Stack.** Electron (Electron Forge + webpack) with a TypeScript **React 17** renderer,
  `react-bootstrap` 5, `styled-components`, `i18next`, the HTML Gamepad API via `react-gamepad`
  (pinned to a **git URL**, no version) and `await-exec`
  ([`package.json`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/package.json)).
  Main/preload/renderer split with `contextBridge`
  ([`electron/main.ts`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/electron/main.ts),
  [`electron/bridge.ts`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/electron/bridge.ts)).
  *Confirmed.*
* **License.** `"license": "MIT"` in `package.json`, but **no `LICENSE` file exists anywhere in the
  tree** — `git ls-files | grep -i licen` returns nothing at `6fc1f1c` — and the GitHub API reports
  `license: null`. So there is **no SPDX identifier backed by a file**; see §7. *Confirmed.*
* **Activity.** 26 commits, first 2022-07-27, last 2024-08-13 (`6fc1f1c`); 6 releases from `0.0.1`
  (2022-07-27) to `v1.5.0` (2022-10-16) on the [releases page](https://github.com/flegald/GZDeck/releases).
  63 stars, 9 forks, **16 open issues**, README banner "No Longer In Development". Dormant for ~2
  years. *Confirmed.*
* **Distribution.** A single Linux `GZDeck-1.5.0.AppImage` built by
  `electron-builder --linux AppImage` (`package.json` `scripts.package`). No CI workflow in the tree
  at `6fc1f1c`. The README instructs the user to add it as a non-Steam game, set `--no-sandbox` as a
  launch argument, and run it once in Desktop Mode to make it executable. *Confirmed.*

### UZDeck — the fork DDL is compared against

* **Purpose.** The same application retargeted at **UZDoom**: "This is a fork of GZDeck by flegald that
  starts UZDoom instead of GZDoom" —
  [README](https://github.com/karkusviktor/UZDeck/blob/daabebcfa8a5c21153901d0f797c366532c35b94/README.md),
  also marked "No Longer In Developement". The GitHub API confirms `fork: true`,
  `parent: flegald/GZDeck`. *Confirmed.*
* **Stack.** Identical to GZDeck. A recursive diff of the two trees at the pins above shows only
  **seven differing files**, all of them constants, strings or the build pipeline (see §3). There is
  **no independent code base to review**; every UI and content-model finding below applies to both.
  *Confirmed.*
* **License.** Same situation as GZDeck: `"license": "MIT"` in
  [`package.json`](https://github.com/karkusviktor/UZDeck/blob/daabebcfa8a5c21153901d0f797c366532c35b94/package.json),
  no `LICENSE` file, GitHub API reports `license: null`. *Confirmed.*
* **Activity.** 35 commits, but only 9 on top of GZDeck, all dated 2026-03-26/27: rename the Flatpak
  constants, add `ipk3`, rename the error strings, add a build workflow contributed by a third party.
  4 stars, 0 issues, 1 fork, last push 2026-03-26. Dormant. *Confirmed.*
* **Distribution.** `UZDeck-1.0.0.AppImage` / `UZDeck-1.0.1.AppImage` from the repo's
  [releases](https://github.com/karkusviktor/UZDeck/releases), published by the one thing UZDeck adds
  over GZDeck: a
  [GitHub Actions workflow](https://github.com/karkusviktor/UZDeck/blob/daabebcfa8a5c21153901d0f797c366532c35b94/.github/workflows/build.yml)
  that runs `yarn package` on `ubuntu-latest` and attaches the AppImage with
  `softprops/action-gh-release`. *Confirmed.*

---

## 2. Features and UX flows worth knowing

Both projects ship **one screen** — no Home, Discover, Detail or Settings — and no networking. The
whole flow is: scan fixed directories → select → launch.

| Feature | Where it lives | Notes |
| --- | --- | --- |
| Single "Launch" screen: *Available Mods* │ *Active Mods* over an IWAD row + big **Launch** button | [`src/screens/Launch/index.tsx`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/screens/Launch/index.tsx), [`LaunchBar`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/components/LaunchBar/index.tsx), [README screenshot 1](https://user-images.githubusercontent.com/14855999/187118856-defdd4dc-e214-4c2f-8b78-b4599575d92e.png) | Two columns plus a fixed bottom bar; layout is `Row`/`Col` with hard-coded `30rem` heights. |
| Folder browsing inside the mod root, with a "…" parent entry | [`updateCurrPath`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/App.tsx), [`ModList`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/components/ModList/index.tsx) | One level of navigation, breadcrumb-free. |
| Selection by clicking a row; `[+]`/`[-]` prefix shows inclusion, active set mirrored in the right column | [`ModList#L40-L72`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/components/ModList/index.tsx), [`Launch#L34-L40`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/screens/Launch/index.tsx) | Good, immediate feedback on what is selected. |
| Last run is restored automatically from `~/.config/GZDeck/settings.json` (`previousRun.iwad`, `previousRun.mods`) | [`bridge.ts#L31-L58`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/electron/bridge.ts), [`Launch#L25-L32`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/screens/Launch/index.tsx) | Survives restarts; `savedConfigs: []` is written on every launch and never read — dead scaffolding. |
| Engine presence check before showing the launcher | [`bridge.ts#L18-L21`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/electron/bridge.ts) (`flatpak list \| grep GZDoom`), [`App.tsx#L109-L114`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/App.tsx) | Explicit "GZDoom not found / install it via the Discovery app" state. No version, update or rollback. |
| IWAD picker as a modal listing accepted files in the mod root | [`IwadModal`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/components/IwadModal/index.tsx) | Filenames only; `active` marks the current one; closing only through **Ok**. |
| Content directories created on first run, Mods forced under `<engine-config>/mods` | [`bridge.ts#L23-L29`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/electron/bridge.ts) | Path is the hard-coded Flatpak location; no user-configurable root. |
| Gamepad: D-pad + `A`, hand-rolled focus | [`Controller`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/components/Controller/index.tsx), [`focusableElements.ts`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/components/hooks/focusableElements.ts) | Focus is an **index into a flat `document.querySelectorAll` list**, not spatial navigation; only `DPadUp/Down/Left/Right` and `A` are handled — no analog stick, no B/Back. |
| Launch: disable input → run engine synchronously → save settings → re-enable | [`LaunchBar#L30-L56`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/components/LaunchBar/index.tsx) | Failure path is a blocking `alert(e)`. |
| i18n scaffolding | [`public/locales/en/translation.json`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/public/locales/en/translation.json), [`src/i18n.ts`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/i18n.ts) | 10 strings, English only, `fallbackLng: 'en'`. |
| Window: 1200×800 in dev, **fullscreen in production**, `--no-sandbox` required on Steam | [`main.ts#L8-L25`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/electron/main.ts), [README](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/README.md), [README screenshot 2](https://user-images.githubusercontent.com/14855999/187119028-788135e7-33b8-4c47-bfc3-e0f39c37ce81.png) | `--no-sandbox` is a Steam launch argument the user must set; "Failure to set this flag may cause steam to hang". |

**Flows that do not exist** (Confirmed by the file list at the pin — 58 tracked files, no downloader,
no catalog, no settings screen): engine install/update/rollback, downloads or any content acquisition,
content search, metadata of any kind, multiple saved profiles, per-mapset shortcuts, controller
configuration, and a Steam Input story beyond "the HTML Gamepad API works if Steam Input translates
the Deck's controls into a standard gamepad".

**The only UZDeck deltas** (recursive diff at the pins above): the Flatpak constants
([`electron/constants.ts`](https://github.com/karkusviktor/UZDeck/blob/daabebcfa8a5c21153901d0f797c366532c35b94/electron/constants.ts)),
`ipk3` added to `ACCEPTED_FILE_TYPES`
([`src/utils/constants.ts`](https://github.com/karkusviktor/UZDeck/blob/daabebcfa8a5c21153901d0f797c366532c35b94/src/utils/constants.ts)),
two translated strings, the window title/header text, and the new CI workflow. *Confirmed.*

---

## 3. Content model, compared with DDL's TOML recipe (D-010)

**There is no content model.** A "mod" in GZDeck/UZDeck is *any file in one directory whose extension
is on a list*, and identity is the filename. Nothing is described, composed, ordered or verified.

| Concern | GZDeck / UZDeck (pinned) | DDL (D-010, `docs/decisions.md`) |
| --- | --- | --- |
| Where content lives | One hard-coded path: `~/.var/app/org.zdoom.GZDoom/.config/gzdoom`, mods under `.../mods` ([`electron/constants.ts`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/electron/constants.ts)) | Packages fetched from **allowlisted hosts** with SHA-256, extracted into a DDL-owned library; imported local PWADs also supported. |
| How a mapset is described | Not at all. Files are listed by name; the only metadata is the filename and extension. | One self-contained **TOML recipe per mapset** under `catalog/`, with title, creators, source page, credits and verification evidence. |
| File typing | Extension allow-list, `['iwad','wad','pk3','lmp','pk4','pk7','pke','zip']` ([`fileUtils.ts`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/utils/fileUtils.ts)); UZDeck adds `ipk3`. | Explicit `files` with a `role` each (maps, dehacked, …). |
| Composition | `modList.forEach(m => command.push('-file "<abs>"'))` ([`bridge.ts#L74-L80`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/electron/bridge.ts)) — every mod is one more `-file`, concatenated into a shell string. | Typed resolver producing the exact argv: IWAD position, `-file` sequence, `-deh`/`-bex`, aux assets. |
| Load order | **Click order.** `setSelectedMods(selectedMods.concat([chosenMod]))` ([`Launch#L34-L40`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/screens/Launch/index.tsx)); there is no reorder control, and removing/shifting requires deselecting and reselecting. | Explicit ordered `load_order` in the recipe, deterministic topological sort, `load_before`/`load_after`, cycle detection. |
| IWAD selection | Picked by filename from a modal listing every accepted file in the engine config dir ([`IwadModal`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/src/components/IwadModal/index.tsx)); the app does not know which game an IWAD is, and the recipe-level requirement does not exist. | `requirements.iwad` per recipe, identified by signature/hash, ambiguity surfaced, extra IWADs validated per title. |
| Compatibility (complevel, engine version) | None. Whatever the Flatpak GZDoom is, and whatever game data the user points at. | `compatibility.family` + `complevel` + `tested_engine_version` + a five-state validation taxonomy (Indexed → Verified). |
| Optional add-ons | None. An add-on is just another file the user clicks, with no declared dependency and no position. | Recipes may declare **optional components** shown as per-profile toggles, each with its own load-order position and the resolved order made visible before launch (D-010, reference recipe AUGER;ZENITH). |
| Integrity / trust | No hashes, no size checks, extension filter only. | SHA-256 per package, host allow-list, CI recipe linting on every PR. |
| Reproducibility of a run | Reconstructed from `previousRun` (IWAD + mod list) and replayed. | Immutable install manifest + resolved launch manifest; catalog updates do not silently change an existing install. |

Two things GZDeck does that DDL should keep: it **restores the last run**, and it shows the active set
and the chosen IWAD *next to* the Launch action, so the immediate consequence of a selection is
visible. Everything else in the table is a gap DDL closes by design.

---

## 4. Steam Deck lessons

All items are **Confirmed** user reports (public issues) unless marked otherwise. These are the
problems a Deck-first launcher must not repeat.

| # | Problem users hit | Evidence | DDL implication (Proposed) |
| --- | --- | --- | --- |
| 1 | A paired Bluetooth controller makes the app **entirely unresponsive**, including the built-in controls; only the touchscreen worked. Workaround from the community is a third-party Decky plugin ("Input disabler"). | [GZDeck #17](https://github.com/flegald/GZDeck/issues/17) | Own the focus system and the input device enumeration; do not depend on Steam Input translation for DDL's own UI (D-009); test hotplug and dual-controller states (**Needs Deck Validation**). |
| 2 | "No controller input" — only `A` advanced a menu; also reported on other SteamOS handhelds. | [GZDeck #24](https://github.com/flegald/GZDeck/issues/24) | Every core action must be reachable from the D-pad/stick/buttons; no mouse-hover-only affordances (`OVERVIEW.md` §3). |
| 3 | Wrong game launched: selecting `doom2.wad` + a mod brought up **Ultimate Doom** instead, with no warning. | [GZDeck #32](https://github.com/flegald/GZDeck/issues/32) | Identify IWADs by content/hash, declare the required IWAD in the recipe, and show the resolved IWAD before launch (§6, §7 of `OVERVIEW.md`). |
| 4 | Mods marked active simply do not load, so the user sees plain DOOM II / plain Doom / normal Doom. Root causes reported over time: **spaces in filenames** (fixed only in `v1.5.0`), and later **unsupported engine/mod version**. | [#12](https://github.com/flegald/GZDeck/issues/12), [#18](https://github.com/flegald/GZDeck/issues/18), [#22](https://github.com/flegald/GZDeck/issues/22), [#31](https://github.com/flegald/GZDeck/issues/31) | Verified recipes plus a pre-launch summary of exactly what will load; version-pinned compatibility evidence so "verified on engine X" is explicit. |
| 5 | Fixed, non-negotiable paths under the Flatpak sandbox: no way to point the app at IWADs or mods kept elsewhere ("not wanting to deal with the flatpak permission system"; rsync'd WAD collections). | [GZDeck #23](https://github.com/flegald/GZDeck/issues/23) | Configurable content roots with safe defaults, resolved through XDG APIs (`OVERVIEW.md` §§6, 10). |
| 6 | Config dumped outside XDG (a user had to ask for `.config` instead of Documents). | [GZDeck #15](https://github.com/flegald/GZDeck/issues/15) | XDG paths, atomic writes, versioned schemas — already required by `AGENTS.md`. |
| 7 | Uncompressed **directory mods** unsupported; the maintainer's folder navigation does not load a folder's contents, so users must zip mods, which changes how the mod behaves in co-op. | [GZDeck #13](https://github.com/flegald/GZDeck/issues/13) | Recipes name *files to load*, so a directory-shaped package is a packaging detail, not a user problem. |
| 8 | `.ipk3` standalone games not recognised in GZDeck; only UZDeck added the extension. | [GZDeck #20](https://github.com/flegald/GZDeck/issues/20) | Type support follows verified compatibility data, not an extension allow-list that varies per fork. |
| 9 | Under Gaming Mode the **on-screen and external keyboards cannot type into the game console**; the fix found by the user was an engine-side launch argument (`SteamDeck=0`), i.e. a Gaming-Mode-specific windowing/fullscreen interaction. | [GZDeck #26](https://github.com/flegald/GZDeck/issues/26) | Fullscreen/focus/window behaviour must be validated on the Deck in Gaming Mode, not in Desktop Mode (**Needs Deck Validation**). |
| 10 | Launching through Steam requires the user to add `--no-sandbox`, and forgetting it "may cause steam to hang" (an Electron artefact, not a DOOM one). | [README](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/README.md), [README screenshot 2](https://user-images.githubusercontent.com/14855999/187119028-788135e7-33b8-4c47-bfc3-e0f39c37ce81.png) | DDL must work as a non-Steam game with no special launch arguments, and must never hang the Steam UI. |
| 11 | First run needs Desktop Mode only to `chmod +x` the AppImage. | [README](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/README.md) | The primary journey must not require Desktop Mode (`OVERVIEW.md` §1). |
| 12 | Engine version is not selectable, so a mod that needs an older engine is simply unplayable on Deck. | [GZDeck #31](https://github.com/flegald/GZDeck/issues/31) | Engine lifecycle with explicit installed versions and rollback (`OVERVIEW.md` §5), plus per-recipe `tested_engine_version`. |
| 13 | Users ask for per-mapset shortcuts ("run this IWAD + these mods in one click"), custom commands, and host/join toggles — evidence that "one flat selection, re-done every time" is not enough. | [#25](https://github.com/flegald/GZDeck/issues/25), [#27](https://github.com/flegald/GZDeck/issues/27) | Per-mapset profiles and Continue/Play (`OVERVIEW.md` §§3, 8.3). |
| 14 | Users question the value of a Deck-specific launcher and ask why the mature cross-platform launchers were not ported instead. | [GZDeck #33](https://github.com/flegald/GZDeck/issues/33) | DDL's answer is the curated verified recipe + Gaming-Mode-first UX (`OVERVIEW.md` §1), and it must be visible in the product, not only in the README. |

There are no GitHub Discussions in either repository, so issues (plus the two README screenshots) are
the complete public record. *Confirmed.*

---

## 5. UI critique — where DDL must clearly do better

Read from the pinned source and the README screenshots
([screen 1](https://user-images.githubusercontent.com/14855999/187118856-defdd4dc-e214-4c2f-8b78-b4599575d92e.png),
[Steam launch options](https://user-images.githubusercontent.com/14855999/187119028-788135e7-33b8-4c47-bfc3-e0f39c37ce81.png)).
Each row gives a concrete weakness and the bar DDL should meet.

1. **Typography is decorative, not readable.** A bitmap DOOM typeface is applied to *every* element
   with `font-family: 'DooM', sans-serif !important` (`src/index.css`), so `ASHES2063ENRICHED.PK3` and
   `PROJECT_BRUTALITY-MASTER.PK3` render in stylised caps that are slow to scan.
   **Bar:** a legible UI typeface for all text (decorative face limited to the wordmark/logo), filenames
   shown in their original case and not all-caps, body text readable at handheld distance on a 1280×800
   panel (`OVERVIEW.md` §3).
2. **Low-contrast palette over a busy background.** Red text (`#9a0100`) on a semi-transparent pink
   panel (`rgba(228,214,221,0.9)`) over a DOOM texture photograph; several labels are near-illegible.
   **Bar:** ≥4.5:1 contrast for body text on a flat surface (≥3:1 for large text), no photographic
   texture behind text, and a single documented dark/light theme.
3. **Focus feedback is subtle and brittle.** The only focus styling is `.list-group-item:focus
   { border-width: thick; }` — it merely thickens the row's existing Bootstrap border — plus a
   background swap on buttons; the screenshot shows the focused row as a slightly thicker box while the
   Launch bar gives no focus cue at all until you reach it. Focus is also *index-based*: the hook queries
   `[data-inputcategory="left"]`, `"launch"`, `"exit"` and `"modal"` and walks a flat array
   (`focusableElements.ts#L15-L93`), with `DPadRight` hard-coded to jump to the Launch button.
   **Bar:** a visible, high-contrast focus ring on *every* focusable element at all times; real spatial
   navigation (Slint focus chain / geometry-based movement) so D-pad directions match the on-screen
   layout; focus restored deterministically after the game exits.
4. **The active list is a dead end.** The *Active Mods* column renders with
   `data-inputcategory='right'` (`Launch#L57-L69`) but the D-pad pool is built from `"left"` only
   (`focusableElements.ts#L15`), so with a controller alone a user **cannot reach the Active Mods list
   to remove a mod** — only re-click it in the left column. There is no reorder, no numbering, and no
   dependency awareness.
   **Bar:** D-010's requirement — an obvious, numbered, editable load order with per-profile toggles for
   optional components, reachable and reorderable with the controller.
5. **Filenames are the only identity.** Every row is a raw filename with an `[+]`/`[-]` marker; WADs,
   PK3s, ZIPs and IPK3s are visually identical, and there is no title, author, description, preview,
   size, or "which game does this need".
   **Bar:** a content card (title, creators, cover, required IWAD, verification badge, last played) as
   the primary unit, with the raw file list only in an advanced/diagnostic view.
6. **What will actually run is not clear.** The only summary is a header line "Selected IWAD:
   DOOM.WAD" and the file lists. Nothing states the resolved game, complevel, or the order the files
   will be passed in, and there is no pre-flight check that the IWAD matches the recipe.
   **Bar:** a persistent, plain-language "Will launch: DOOM II · 3 files · load order 1→3" summary next
   to Launch, and a full resolved-launch diagnostic view (`OVERVIEW.md` §8.3).
7. **Feedback and error handling are thin.** During a launch the whole UI is disabled
   (`inputDisabled`, `LaunchBar#L30-L56`) and only a spinner is shown, with `alert(e)` for failure
   (blocking, raw exception text, no remediation, no log path). The missing-engine state is one red
   `Alert` with a truncated sentence — "Please install GZDoom via the Discovery app," — and no action
   button (`src/components/ErrorAlert`, `translation.json`).
   **Bar:** distinct, actionable states for network failure, hash mismatch, missing IWAD, wrong IWAD,
   unsupported complevel and missing engine, each with a next step and a log export; non-blocking
   progress with cancel; no `alert()`.
8. **Navigation is shallow but confusing, and has dead ends.** The mod browser can only go into
   subfolders and back via a "…" row; the IWAD picker is a modal in a *separate* focus pool that is
   dismissed only by the **Ok** button; and quitting is a small *Exit* button in the top-right that you
   reach by wrapping up from the top of the list. There is no Back button and no B-button handling
   anywhere in `handleInput`.
   **Bar:** consistent Back/Confirm mappings, a modal that traps focus and closes with Back, an explicit
   Quit, and guidance rather than dead ends — always with one obvious way forward.
9. **No feedback during downloads because there are none.** Content acquisition is "copy files into a
   Flatpak directory in Desktop Mode" ([README](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/README.md)).
   **Bar:** asynchronous download → verify → extract → install with byte progress, cancel, retry,
   resume and an install summary, all usable from Gaming Mode (`OVERVIEW.md` §3).
10. **Layout is not designed for 1280×800.** The window is 1200×800 and fullscreen in production
    (`main.ts#L8-L25`); the panels use a fixed `height: '30rem'` and the launch bar is
    `position: fixed; bottom: 20px` (`Launch#L44`, `LaunchBar#L65`), so the composition does not follow
    the panel.
    **Bar:** an explicit 1280×800 design, no fixed pixel heights for content panes, no clipped or
    overlapping chrome, and no dependence on window decorations or a menu bar.
11. **Visual polish is inconsistent.** A `bg="light"` Bootstrap navbar sits inside an otherwise dark,
    red-on-texture page; the whole UI is one flat file list with no grouping, empty state, or imagery
    ("Active Mods" is just a large empty box at launch).
    **Bar:** one coherent visual language, a designed empty/loading state, and grouping (Featured/
    Library/installed) rather than an undifferentiated list.
12. **No saved configurations, no per-mapset identity.** `savedConfigs` is written as an empty array on
    every launch and never read; the only memory is `previousRun`.
    **Bar:** multiple named profiles per installed mapset, with their own gameplay options, plus
    Continue/Play per item (`OVERVIEW.md` §§3, 8.3).

---

## 6. Adopt / Avoid / Different-by-design

Every row is **Proposed** (nothing here is decided until the owner accepts it in `docs/decisions.md`).

| Verdict | Idea | One-line reason |
| --- | --- | --- |
| **Adopt** | Two-pane "available → active" model with the active set and the chosen IWAD shown *next to* the Launch action | Instant, unambiguous feedback on what the current selection means. |
| **Adopt** | Persist and restore the last run as a first-class "Continue" | It is the single most-used flow in GZDeck and removes the re-selection tax. |
| **Adopt** | Check that the engine is present before showing the main UI, with a blocking, explanatory state when it is missing | Users immediately understand a missing prerequisite instead of pressing Launch at nothing. |
| **Adopt** | Ship one self-contained artifact that needs no installer and no root privileges, and put the platform-specific launch instructions in the README | Matches DDL's chosen distribution (tarball) and the Deck's non-Steam-game workflow. |
| **Adopt** | Thread every user-facing string through a translation resource from day one | Cheap now, expensive later — aligns with D-008. |
| **Adopt** | Document the Steam-side launch-argument/controller caveats in the README | Steam↔child-process behaviour is a real integration hazard; documenting it is the DDL boundary for Steam Input. |
| **Avoid** | Building the engine command by joining arguments into one shell string and running it through `exec` | Injection and quoting bugs (the spaces-in-paths defect, the unquoted `-iwad`); `AGENTS.md` forbids shell strings. |
| **Avoid** | Hard-coded Flatpak paths and extension-only file discovery | Blocked legitimate setups and produced the wrong-game failures; use XDG + user-confirmed roots and content detection. |
| **Avoid** | Filename-as-identity, with no hashes, sizes or metadata | Users had no way to know what was loaded or whether it was the right content. |
| **Avoid** | Index-based focus over ad-hoc DOM attributes plus a WebView's native focus | It is the root cause of the controller and modal dead-ends; a native toolkit gives a real focus system. |
| **Avoid** | Blocking `alert()` for errors and disabling all input for the whole launch | No remediation, no logs, no cancel. |
| **Avoid** | Decorative bitmap font for body text, low-contrast red-on-texture palette | Unreadable at handheld distance and on the 1280×800 panel. |
| **Avoid** | An Electron/Chromium shell and npm dependencies pinned to a git branch | Large download, sandbox flags, and an unpinnable supply chain; DDL is native with reviewed, pinned crates. |
| **Avoid** | Requiring user-side launch arguments so the launcher does not hang Steam | A launcher that needs a workaround to start is not Deck-first. |
| **Avoid** | Requiring one Desktop-Mode session to make the binary executable | Breaks the "Gaming Mode only" promise of `OVERVIEW.md` §1. |
| **Avoid** | Writing configuration outside XDG | The very first thing an experienced Linux user complained about. |
| **Different by design** | Curated, verified TOML recipes with SHA-256 and allow-listed hosts instead of "any file in a folder" | DDL's promise is one-click, known-good play (D-010), which requires describing and validating content. |
| **Different by design** | One engine (DSDA-Doom) with DDL-managed, pin-validated versions and rollback | Reproducible classic-map execution needs complevel accuracy, and users asked for engine versions GZDeck could not offer. |
| **Different by design** | IWAD identified by content/hash, rechecked at launch, and declared per recipe | Prevents the "wrong game launched" class of failure rather than reporting it later. |
| **Different by design** | Complevel, DEH/BEX and explicit ordered load order are first-class in the recipe and the resolver | `-file` concatenation cannot express what classic community mapsets need. |
| **Different by design** | Optional components as per-profile toggles with a visible resolved order | D-010's reference recipe (AUGER;ZENITH + add-on) makes this a product requirement, not a nicety. |
| **Different by design** | Native Rust + Slint, gamepad-first, no embedded browser | Required for a lean, focus-correct, controller-only application. |
| **Different by design** | Steam Input is README-only; DDL never reads or writes layouts | Deliberate product boundary (`OVERVIEW.md` §8; `decisions.md`), and the boundary DDL does not want to depend on for its own UI. |

---

## 7. License note

**Facts.** *Confirmed.* Neither repository contains a `LICENSE`, `COPYING` or `NOTICE` file at the pins
above (`git ls-files` finds none). Both only *declare* `"license": "MIT"` in
[GZDeck `package.json`](https://github.com/flegald/GZDeck/blob/6fc1f1ce2ffab097983bc990ac55d27a9035ffe5/package.json)
and [UZDeck `package.json`](https://github.com/karkusviktor/UZDeck/blob/daabebcfa8a5c21153901d0f797c366532c35b94/package.json),
and the GitHub API reports `license: null` for both — there is no SPDX identifier we can point at an
actual licence file. UZDeck is a Git fork of GZDeck, so it inherits the same status. The bundled
assets — `src/fonts/DooM.ttf`, `src/assets/images/FQJ4izH.png` (background) and `folder-icon.png` —
carry no licence or attribution statement anywhere in the tree, and the typeface and background appear
to be derived from id Software's DOOM material.

**What DDL (GPL-3.0-or-later, D-001) may do.**

* **Reuse the ideas.** Purpose, flows, interaction patterns and the problem catalogue in §4/§5 are not
  copyrightable, and §6 records them as DDL's own proposals. Writing our own Slint screens and Rust
  integration is not a derivative work. Crediting UZDeck and GZDeck in `README.md` as inspiration
  (`OVERVIEW.md` §18) is both honest and sufficient.
* **Do not copy code.** With no `LICENSE` file, the MIT declaration is metadata only and gives us
  nothing we can rely on; absent a licence, the default is all rights reserved. Even if the declaration
  were honoured, an MIT→GPL-3.0 combination would require a retained copyright notice and a copy of the
  MIT terms, and we cannot satisfy that against a repository that ships neither. Treat GZDeck/UZDeck
  source as **read-only reference**.
* **Do not copy assets.** The DOOM typeface and the DOOM-derived background are not the authors' to
  license in the first place. DDL must not ship them, and must not point the UI at the README's
  screenshots as if they were DDL's own media. DDL's own screenshots/cover art need stated permission
  (`OVERVIEW.md` §9).
* **Do not vendor dependencies from either tree.** `react-gamepad` is pulled from a bare git URL with
  no version (`package.json`), which is itself a reason not to inherit that dependency graph.
* **If we ever wanted to reuse more than an idea** — e.g. a specific algorithm transcribed from their
  source — that would require the upstream author to add a real `LICENSE` file first, and a documented
  licence review in DDL. **Default stance (Proposed): ideas, not code or assets.**

DDL's own GPL-3.0-or-later obligation is unaffected by this: everything DDL ships must be
GPL-compatible, which is why the missing upstream licence file, not just the licence name, is the
operative fact.

# DDL UI contract

Status: **Proposed** design for implementation specs, 2026-10-16. Accepted decisions remain
binding; this document does not promote research assumptions into compatibility evidence.
Scope: native Rust + Slint, one engine, controller-only completion of every journey.

Sources: [vision and journey](../../OVERVIEW.md#3-target-user-journey),
[profiles](../../OVERVIEW.md#83-pistol-start-gameplay-flags-and-profile-isolation),
[D-008–D-014](../decisions.md), [UZDeck critique §5](../research/prior-art.md#5-ui-critique--where-ddl-must-clearly-do-better),
[SIGIL composition](../research/reference-sigil.md#4-composition-candidate-recipe-data--proposed),
[recipe requirements](../specs/002-recipe-catalog.md). **Confirmed** factual claims below derive
from these cited research sources and their pinned upstream links. **Needs Deck Validation**:
readability, input repeat/hotplug, game handoff, pistol start behavior and SIGIL episode behavior.

## 1. Principles

- Controller first: visible focus, spatial movement, one obvious primary action, no hover actions.
- Readable at handheld distance on the 7-inch 1280×800 LCD: 22 px body, short lines, generous gaps.
- Clear over clever: name the required game, explain blockers and show the consequence of toggles.
- DOOM atmosphere through charcoal, ember accents and strong hierarchy; no texture behind text,
  copyrighted artwork, franchise logos or bitmap display fonts. Layout-reference art says “cover art”; visual concepts use original abstract generated imagery.
- Progressive disclosure: title and Play first; files, compatibility and diagnostics one action away.
- Honest confidence: “Verified” requires recipe evidence for the selected engine and combination.
  Distinguish “Partially checked”, “Not verified” and “Incompatible”; never rely on color alone.

## 2. Design system

### Color tokens

Ember Cathedral, intensified (D-021). Opaque sRGB surfaces protect all body text. WCAG
relative-luminance ratios below use the brightest normal surface (`raised`).

| Token | Hex | Use |
| --- | --- | --- |
| canvas / deep / on-accent | `#120C0D` | Chrome, separation, primary-button text |
| surface | `#211719` | Opaque information panels |
| raised | `#352527` | Rows, controls, sheets |
| text | `#F8EAD5` | Bone foreground, 12.26:1 on raised |
| muted | `#D0BCA7` | Secondary / hidden-folder text, 7.91:1 on raised |
| focus / warning | `#FFD078` | Amber ring and primary action, 10.06:1 on raised |
| success | `#88D9B0` | Positive checks with explicit text |
| danger | `#FFA69C` | Error with explicit text |
| line / steel-edge | `#80533B` | Decorative bronze separators |
| ember / copper | `#B8583F` | Decorative oxblood trim |

Body ≥4.5:1, large text ≥3:1. Hidden folders use muted text on opaque raised rows, never
reduced opacity. Unavailable controls give explicit reasons and preserve readable text.

### Visual identity: Ember Cathedral

Original carved oxblood wall repeats seamlessly, visibly surrounding smooth information panels.
Pre-rendered bronze / red-stone frames use 28 px nine-slice borders, preserving corners at all
panel sizes. Dedicated top art bands and cover regions carry “Original concept art” plaques.
The original angular broken-circle emblem pairs with a clearly legible bone Barlow DDL logotype;
no franchise lettering, sprites or copied textures. See `visual-v2/ember-cathedral/prompt.txt`
for the retained original generation prompt and `crates/ddl/ui/assets/ember/build_assets.py`
for the offline production compositor. Art does not imply gameplay verification.

### Typography and assets

**Pirata One** for page, hero and section titles (≥26 px); **Barlow Regular** for body,
controls, filenames and metadata. Both unmodified TTFs and SIL OFL 1.1 notices are bundled,
with notices included in release tarballs. Verified sources and notices are retained in
[visual-v2/fonts/SOURCE.txt](visual-v2/fonts/SOURCE.txt); Inter remains bundled for legacy assets.

| Role | Size / line height | Face |
| --- | --- | --- |
| Page title | 36 / 44 px | Pirata One |
| Hero title | 44 / 52 px | Pirata One |
| Section / card title | 26 / 34 px | Pirata One |
| Body / controls | 22 / 30 px | Barlow |
| Metadata / hint bar | 20 / 28 px | Barlow |

Preserve filename case. Wrap titles and descriptions; directory breadcrumbs elide from the
start so the destination remains visible. No essential tooltips. Minimum control height 52 px;
64 px for two-line rows. Long lists scroll inside the content area.

### Geometry, elevation, icons and motion

Spacing: 4, 8, 12, 16, 24, 32, 48 px. Outer inset 40; cards 24; column gap 24.
Opaque flat panels with carved perimeters; bevelled bronze controls and circular / pill footer
badges. One active target: 4 px amber outer ring with 4 px dark separation around its complete
bounds. Active-tab underline and selection never replace the ring. Focus does not change layout.
Original simple icons always pair with text. No continuous animation, pulsing, blur or parallax;
pre-rendered PNG materials and nine-slice frames keep idle work low in the software renderer.

### Layout and translation

Reference canvas exactly 1280×800: header 104 px, content y=128–704, hint bar y=736–800;
32 px gap prevents overlap. Reference coordinates illustrate proportions, not fixed pane heights:
Slint layouts allocate remaining content space and scroll overflow, including at smaller windows.
No window decorations required. Reserve ring gutters and 40 px outer safe inset.

Every visible literal—including status, errors, categories, accessibility labels, hint verbs,
pluralized file counts and confirmation copy—must use Slint `@tr` or translated model resources.
Catalog descriptions/category labels need translation keys and English fallback. Proper names,
filenames and paths are data, not translated. Use complete sentences with named placeholders,
not concatenated fragments; locale-aware counts, sizes and dates. Allow 35% text expansion,
wrap controls to two lines, and test pseudo-localization and RTL layout before shipping a locale.
Button glyphs remain physical labels; their accompanying actions translate. English MVP (D-008).

## 3. Navigation model

```text
First run → Engine install → IWAD folders → Home
Home ↔ Discover ↔ Library ↔ Settings                 (L1 / R1)
Home / Discover / Library → Mapset detail → Play setup → Game → Play setup
Mapset detail → Download / install → Mapset detail
Settings → Engine / IWAD folders / Storage / DSDA config / Diagnostics
Any import / folder action → Controller picker → originating control
Start → menu: Downloads, Settings, Quit DDL
```

D-pad and left stick move to the nearest eligible control in the requested direction; explicit
neighbor links resolve ties. No edge wrap. Vertical lists move by row, grids retain the nearest
column; right from the left column enters the closest right-column row. Up from content enters
the active top tab; down returns to the last content target. Right stick is unused, avoiding
accidental scrolling. Proposed stick threshold/repeat require input-spec tuning on Deck.

A confirms; B returns one level; X/Y only perform the visible contextual action. L1/R1 switches
root tabs, preserving each tab's scroll/focus; within detail/setup they do nothing and are omitted
from hints. Start opens the menu. At a root tab B does nothing; it never opens Menu. The persistent legend always labels B as “Back”, including at root where it is inactive. Start alone opens Menu. Quit requires confirmation with “Keep playing” (cancel) focused.

Persistent hint bar shows available actions in physical order (with the inactive B Back hint retained at root): A, B, X, Y, L1/R1, Start.
A's verb describes the focused action (“Import”, “Toggle”, “Choose folder”), never a misleading
“Play” when focus is elsewhere. Disabled actions display why in the content; no hidden shortcuts.
Sheets trap focus; B cancels, returns to the invoking control and discards uncommitted changes.
Controller picker: breadcrumb, storage roots, parent row, directory/file list, “Use this folder”
or “Import file”, B parent then cancel at root. No text entry needed; profile renaming uses D-pad preset names; no text entry or on-screen keyboard is required (spec 018).

Before launching, store screen, mapset/install/profile id, control id, scroll and current options.
Show “Game running — return here when you exit” while the child owns input; do not interpret
background inputs. On exit restore Play setup and its Play control; if no longer valid focus
first remediation, then heading fallback. Release held buttons and require neutral input before
accepting a new press. Report exit failures inline. Hotplug retains focus and shows “Controller
connected”; disconnect shows “Connect a controller to continue”. Pointer motion is ignored and
cursor hidden (D-009); clicks may focus/activate the same controls. No Steam Input UI or code.

## 4. Screen contracts and copy

Global state rule: loading keeps chrome and B usable, reserves final geometry, and says what is
loading. Empty gives a forward action. Error keeps prior usable data plus Retry/Diagnostics.
Offline uses cached metadata and installed content, with “Offline — showing saved catalog”; no
network retry loop. The following overrides define each screen's exact state and focus behavior.

### Home / Continue — `mockups/home.png`

Purpose: resume the last mapset/profile and expose readiness. Left hero: cover, title, last profile
and “Continue”; right: “Ready to play” prerequisite summary and Discover action. Lower recent
items show title, creators, required game, installed/confidence status and last played.
“Continue” means reopen the last launch profile/start the engine; it **does not promise loading an
engine save automatically**. Copy: “Your last profile, ready again.”
Focus: Continue → hero Details → right readiness actions → recent cards; initial Continue, or
“Discover mapsets” if no history. Loading: “Loading your library…”; empty: “Your next mapset starts
here” / “Discover mapsets”; error: “Could not read recent play” / Retry, library still usable;
offline: installed Continue works, cached Discover remains available. Missing prerequisites replace
Continue with “Finish setup”, naming the blocker.

### Discover — `mockups/discover.png`

Purpose: browse curated categories without typing. **Category strip (spec 015):** “All”, then
each tag of `catalog/tags.toml` that at least one recipe uses, in the vocabulary's `order`, then
“Favorites”. A 3-column grid contains cover placeholders, title, creators, required game + IWAD
status, install state, a favorite star and at most two tag chips (the open category's tag first).
Focused card gets ring; no text over art. Select opens detail.
Bindings: **Up** from the first grid row enters the strip at the selected chip; **Left/Right** on
the strip moves the chip and switches the category at once (no wrap); **Down** from the strip
enters the first card; **L2/R2** (keyboard `[` `]`) switch to the previous/next category from the
strip or the grid, without wrap; **X** on a focused card toggles favorite (the hint bar shows
“Favorite” / “Unfavorite”); L1/R1 remain root navigation. Favorites sort first in every category.
A strip wider than the grid scrolls and shows `‹` / `›` for hidden chips. The last category is
kept for the session. Empty Favorites: “No favorites yet” / “Focus a mapset in Discover or
Library and press X to keep it here.” / “Show all mapsets”.
Loading: “Loading curated mapsets…”; empty: “No mapsets in this collection yet” / Featured;
error: “Catalog could not be refreshed” / Retry + use saved catalog; offline without cache:
“Connect to download the catalog” / Open library. Search is deferred beyond curated MVP.

### Mapset detail — `mockups/mapset-detail.png`

Purpose: decide confidently. Left cover + primary Install or Play setup; right title, creators,
short description, base-game readiness, compatibility confidence + tested engine/date, optional
components summary, credits/source and installed version. No invented “Verified” label in mockups:
“Verification pending” is deliberate. “Install” stays available if the IWAD is missing; no
“Install & Play” promise while blocked. Focus primary → Setup → Favorite (“Add to favorites” / “Remove from favorites”; **X** does the same from anywhere on the screen) → Profiles → requirements →
components → evidence → source/credits. The pane scrolls to keep whole focused row visible.
Loading “Loading mapset details…”; empty/removed recipe “This mapset is no longer in the catalog”
with installed copy still reachable; error “Could not read this recipe” / Retry + Diagnostics;
offline cached detail usable, Install unavailable with “Connect to install”. Source shows URL and
retained text in-app; controller journey never depends on opening a browser.

### Play setup / Will launch — `mockups/play-setup-sigil.png`

Purpose: make the resolved launch concrete. Left: named profile selector, component rows,
variant/advanced controls and gameplay options. Right: base game readiness, engine/confidence,
numbered enabled files, compatibility, skill, pistol start, and Play. Required main cannot be
unchecked without a valid replacement. Optional components show On/Off with A toggling. A on a
missing user-supplied component opens Import, never a download. Copy: “Soundtrack: not provided —
import”; help: “Optional. The included MIDI music works without this file.”
Focus initial Play if ready; left from Play enters last visible options row; column crossing
uses row geometry. Left column order: profile → main → soundtrack Import → Advanced/compat →
skill → pistol start → load-order editor; right column: base-game details → evidence → resolved
launch details → Play. Compat is off in this expanded advanced mockup; normally collapsed.

Required compatibility stays separate from player preferences. Show “Complevel 3 — recipe
recommendation (not yet verified)” for SIGIL; lock recipe-required values until Advanced is
explicitly selected. Unsupported values block Play. Skill picker uses recipe/engine-provided
labels and values; mockup “Skill 3” avoids inventing map-specific text. Pistol start shows Off/On
and “Reset inventory at each map”; if unsupported/unvalidated show “Unavailable for this engine”
with reason, not an ineffective toggle. D-011 confirms the flag; actual transitions/save behavior
remain **Needs Deck Validation**. Recording mode cannot combine with pistol start.

Load-order editor: every enabled row numbered, required/dependency constraints explained. A
enters move mode; up/down selects a valid position, A commits, B cancels. Invalid positions are
marked “Required order”; cannot silently turn a verified recipe into an arbitrary permutation.
Where recipe has one valid ordering (both worked examples), rows are inspectable but locked;
the editor says why. Profiles save independent choices. The named profile row opens a framed picker; Mapset detail
also offers the active profile below Favorite. **D-pad** moves between actions; **A** on Previous /
Next switches the active profile immediately (no wrap), remembered per mapset. **New — copy
settings** clones the current options; **New — defaults** starts with default options. Both select
“Profile 2”, “Profile 3”, … using the first free number and keep saves separate. **Rename profile**
opens D-pad presets: Default, Casual, UV, Pistol start, Co-op, Speedrun, and the first free Profile N.
**Delete profile** opens a confirmation naming the profile and saying its saves and screenshots
will be deleted. Cancel is initially focused; **Right, A** confirms; **B** cancels and returns to
the invoking profile control. The last profile cannot be deleted. The profile document is deleted
before its runtime directories; installed mapset files are retained. **B** closes either picker
without another action; switching is saved immediately. L1/R1 remain inactive within these sheets.
Home Continue shows and launches the remembered profile; Library shows the profile count. Unverified overrides visibly remove
verified confidence. Effective order updates immediately after each change.

Loading “Checking launch requirements…” blocks Play until resolver completes; empty/no profile
“Create a profile to play” / Create profile; resolver error “Could not prepare this launch” /
Retry + Diagnostics; offline installed validated files launch normally. Right preview remains
visible while options scroll. Diagnostic view includes engine, config source/template version,
resolved IWAD, separate ordered PWAD/DEH groups, complevel/flags, profile/save identity and argv
as read-only data, with redacted export. Never allow free-form shell commands.

### Missing IWAD / blocked Play — `mockups/play-blocked-iwad.png`

Same setup screen with warning: “Ultimate DOOM is missing. Add a folder containing your legally
owned IWAD.” Play remains visible, unavailable and explainable; initial focus “Add IWAD folder”.
A opens controller picker → scan → matched-game review → “Use this game” → return and recheck.
No automatic acceptance based on filename. Wrong match: “This is DOOM II. SIGIL needs Ultimate
DOOM.” Provide Choose another folder, Rescan and Diagnostics. No buy/download-IWAD button.
Cancel returns to blocked screen. Install is still allowed from detail. Distinguish checking,
missing, wrong version and ready; offline has no effect on local scan.

### Library — `mockups/library.png`

Purpose: manage installed titles/imported PWADs. Filters All / Ready / Needs setup / Favorites;
two-column rows show identity, game, confidence, last played, profile count and storage. Initial
last selected card, otherwise first; A detail, X favorite (star on the card, favorites sort first; spec 015), Y import local PWAD. Filter first then
rows in spatial order. Import is controller picker → identification summary → create local entry;
unknown compatibility says “Not verified”, never fabricates a curated recipe. Empty “No installed
mapsets yet” / Discover or Import; loading “Reading installed mapsets…”; error “Some installs could
not be read” / inspect affected items + Diagnostics; offline local Library fully usable.

### Download / install — `mockups/download-install.png`

Purpose: asynchronous acquisition with informed cancellation. Title and cover; stages Download →
Verify → Extract → Install, current step, received/total bytes, speed/estimate only when known.
Verification/extraction use indeterminate bars when no trustworthy total exists, never pretend
byte count means installed. Mockup byte counts are illustrative, not package metadata.
Initial focus Cancel; right Details exposes source, digest and logs. B returns to originating
screen with progress indicator; does not cancel. Start → Downloads reopens task. Cancel confirms
“Cancel this install?” / Keep downloading (initial) / Cancel install. During short atomic commit
show “Finishing install…” and hold cancellation until safe completion. No partial install in Library.
Empty/no tasks “No downloads in progress” / Discover; error preserves resumable partial where
safe, “Download interrupted” / Resume or Retry / Cancel; offline “Waiting for connection” with
Resume and Cancel. Resume only when source identity/range support is safely validated; otherwise
“Restart download”. Hash failure discards suspect data. Success “Installed — ready to set up” /
Play setup; when IWAD missing “Installed — add Ultimate DOOM to play” / Add IWAD folder.

### Settings — `mockups/settings.png`

Purpose: maintain the installation. Left sections Engine / IWAD folders / Storage / DSDA config /
Diagnostics / Appearance & language. Right selected section, clear status and rows. Engine:
installed and previous versions, Check for updates, explicit Install update, Roll back, advanced
unlisted release warning per D-007. Mockup v0.30.1 is research reference, not current/latest claim.
Show reviewed digest status separately from gameplay verification. No update while game runs.

IWAD folders: list chosen roots, Add folder, Rescan, Remove folder (does not delete IWAD).
Storage: used/free bytes, install location, download cache and Clear cache confirmation. DSDA config:
baseline version, preserved edits, Reset to recommended with backup/confirm; disable mutations
while game runs. Diagnostics: latest launch result, resolved manifest, View logs, Export redacted
report; controller picker chooses destination, success shows exact saved location. Appearance:
reduced motion and English MVP language status; no unsupported language choices.

Focus first left section → right first action; up/down rows; B right returns left, B left root.
Loading “Checking engine…” preserves local information; empty “DSDA-Doom is not installed” /
Install engine; error “Could not check for updates” / Retry, installed engine preserved; offline
“Connect to check for updates”, rollback/local storage/config/logs remain available. Destructive
confirmation defaults Cancel and describes scope and backup. No Steam Input setup screen.

### First-run setup — `mockups/first-run.png`

Purpose: install prerequisites in Gaming Mode. Two-step page: 1 Engine, 2 Your base games. Engine
explains “DDL downloads the official DSDA-Doom engine. It does not include game data.” / Install
DSDA-Doom opens shared progress. After success focus Add IWAD folder; controller picker and
scan show identified games with confirmation. Finish goes Home, or “Browse without a base game”
allows Discover but Play stays blocked. Do not require keyboard, shell or Desktop Mode.
Initial focus Install engine; then Add folder → Review scan matches → Finish / Browse. B from
picker returns here; B at initial setup does nothing. Start provides Menu and the explicit Quit action. Loading “Checking prerequisites…”;
empty scan “No supported base games found” / Choose another folder; engine error Retry + Diagnostics;
offline “Connect to install the engine” / Retry or Browse saved catalog. A local usable engine may
proceed offline. No automatic IWAD downloads or Steam Input onboarding.

## 5. Worked compositions

### SIGIL 1.23

**Confirmed:** reference research documents `SIGIL_V1_23.wad` (main E5),
`SIGIL_COMPAT_V1_23.wad` (E3 alternative), and paid REG/compat soundtrack arrangements.
**Proposed:** design example uses Ultimate DOOM, free main enabled, compat off, soundtrack not
provided, Skill 3 and pistol start off. Right summary: “Will launch: Ultimate DOOM · 1 content
file”; numbered order 1 `SIGIL_V1_23.wad`. IWAD is listed separately and is not counted as content.
Complevel 3 remains a research recommendation pending Deck evidence.

Variants form a single-selection group. Choosing compat replaces the main, with confirmation
“Use the Episode 3 variant instead?”; never load both. A validated paid REG import replaces the
free main in position 1, not position 2. Compat soundtrack requires compat and loads at position 2.
Until current paid package identities/composition are verified, import can identify/store a file
but cannot enable an unknown paid combination; show “This soundtrack file is not recognized” /
Choose another file / Diagnostics. Do not assume every music WAD is an additive soundtrack.
The mockup expands Advanced to show compat Off as requested; normal flow keeps it collapsed.

### AUGER;ZENITH

**Confirmed product requirement (D-010/task):** DOOM II, main PWAD, optional add-on PWAD after
main. **Proposed UI example:** add-on Off → “1 content file”; On → “2 content files · order 1 → 2”
with rows “1 Main mapset” and “2 Optional add-on”. Main remains required; add-on cannot move before
it. Show title and creators “Doomer Boards Project 37”, DOOM II readiness and profile choices.
Exact add-on label, filename, bytes, complevel and tested engine are intentionally **unresolved**:
no AUGER source research/recipe is present yet. Bind them from reviewed recipe data; never ship
illustrative filenames or claim verification. [Supplementary mockup](mockups/play-setup-auger.png)
shows semantic labels and “Complevel: awaiting recipe evidence” with Play blocked, rather than
inventing compatibility data.

## 6. Actionable error copy

All errors are inline panels/sheets with retained context, an initial safe next action, B back
and “Diagnostics” / redacted log export. Never blocking native alerts or raw exceptions alone.

| Condition | Message | Initial action / other next step |
| --- | --- | --- |
| Network | “Download interrupted. Your installed mapsets are safe.” | Resume if safe / Restart download / Cancel |
| Hash mismatch | “The download did not match the expected checksum. It was not installed.” | Download again / Diagnostics; discard suspect archive |
| Missing IWAD | “Ultimate DOOM is missing. Add your own base-game folder to play SIGIL.” | Add IWAD folder / Rescan |
| Wrong IWAD | “This is DOOM II. SIGIL needs Ultimate DOOM.” | Choose another folder / View supported games |
| Unsupported complevel | “This engine cannot use the recipe's compatibility setting.” | Choose supported engine version / Restore recipe settings |
| Missing engine | “Install DSDA-Doom before playing.” | Install engine / Diagnostics |
| Unknown user file | “This soundtrack file is not recognized. It has not been enabled.” | Choose another file / Diagnostics |
| Unsafe archive | “This archive could not be installed safely.” | Retry from approved source / Diagnostics; no bypass |
| Storage | “Not enough space to finish this install.” | Manage storage / Retry / Cancel |
| Launch failure | “DSDA-Doom could not start. Your profile has been kept.” | Retry / Diagnostics / Engine settings |
| Config conflict | “Your settings need review before this update can finish.” | Review conflicts / Restore backup |

## 7. Better than UZDeck

Each numbered item maps one-to-one to [prior-art §5](../research/prior-art.md#5-ui-critique--where-ddl-must-clearly-do-better).

| §5 | Critique | DDL design answer / evidence |
| --- | --- | --- |
| 1 | Decorative type everywhere | Inter 22 px body; preserved filename case (§2, setup) |
| 2 | Low contrast, busy background | Opaque dark surfaces and measured ≥4.5:1 text (§2, verification) |
| 3 | Brittle, subtle focus | Separated 4 px amber ring, spatial links, deterministic return (§2–3, every PNG) |
| 4 | Active list unreachable, no order/dependencies | Controller component rows, numbered resolver order, constrained move mode (§4 setup) |
| 5 | Filename-only identity | Cover/title/creators/game/confidence/last-played cards (§4 Home, Discover, Library) |
| 6 | Unclear actual launch | Adjacent Will launch, matched IWAD, complevel, full diagnostics (§4–5, setup PNGs) |
| 7 | Spinner/alert, no recovery | Per-cause copy and next steps, logs, cancellable asynchronous work (§6, progress) |
| 8 | Dead ends, inconsistent Back/quit | A/B conventions, trapped sheets, picker breadcrumbs, explicit Quit (§3) |
| 9 | Manual Desktop acquisition | Download → verify → extract → install, bytes, safe resume/retry/cancel (§4 progress) |
| 10 | Wrong canvas, clipped chrome | 1280×800 reference, flexible content area, reserved hint bar (§2, all PNGs) |
| 11 | Inconsistent polish, blank boxes | Shared tokens, grouped categories, deliberate loading/empty states (§2/4) |
| 12 | No saved profiles/identity | Named per-mapset profiles and honest Continue semantics (§4 Home/setup/Library) |

## 8. Handoff, verification and open questions

Layout-reference inventory: Home, Discover, detail, SIGIL setup, missing IWAD, download/install, Settings,
first run, plus Library and AUGER;ZENITH setup. Every image has one focus ring and persistent hint
bar. The [layout renderer](render_mockups.py) draws original flat geometry and Inter text; these reference images contain no generative or
third-party art. Run with Python + Pillow; no application dependencies added. Actual dimensions,
file sizes, contrast output and visual review are recorded in [verification.md](verification.md).
The high-fidelity [visual set](visual/) supplies the final look; `mockups/` remains the layout reference. Original model-generated art and exact composited UI are documented in [visual-report.md](visual-report.md). No Rust/Slint code changed, so cargo checks are not relevant; behavioral acceptance below must be
encoded in future UI specs/tests rather than pretending a raster proves interaction.

Implementation acceptance: controller-only walkthrough of every navigation edge; B on every
sheet/picker; stable focus after game exit/hotplug; resolver-gated Play; SIGIL exclusivity and
missing soundtrack import; AUGER add-on after main; cancellation at each install stage; offline
with/without cache; named profile isolation; no wrong-IWAD launch; pseudo-localization expansion;
1280×800 screenshot comparison with visible ring and no overlapping chrome. Physical Deck LCD
review at handheld distance is required for final readability, input timing and game return.

Open questions for coordinator/owner:

1. Current SIGIL paid package identities and tested recipe combinations, including compat exposure.
2. AUGER;ZENITH upstream research: exact add-on, authorship metadata, filenames and compatibility.
3. Deck proof for SIGIL cl3/episode selection, pistol start per-map/save behavior, controller hotplug
   and focus handoff. The mockups are design evidence, not gameplay verification.
4. Supported engine/template pair for initial release and precise recipe evidence badge vocabulary.

These are data/hardware gates, not missing screen designs. No new runtime dependencies.

## 9. Installed recipe changes (spec 020)

Library cards replace their base-game readiness line with “Update available” when the embedded
recipe changes; profile counts remain visible. Detail keeps Play setup as the initial action and
shows the change summary, with “Old saves may not load in the new release.” for content changes.
The next two actions become **Update** and **Keep current**. D-pad Up/Down selects actions, A
activates, B returns; X still toggles favorites. Keep current persists for this exact recipe hash;
the Library reminder disappears, while detail retains the explicit Update action.

Update uses the existing progress/cancel/retry view, then returns to Play setup. Removed component
selections produce a non-blocking notice. The frame, type, colors and focus ring use existing Ember
components. A removed recipe shows “No longer in the catalog” in Library and detail; Play setup
continues to use the installed snapshot. Metadata-only changes are adopted during startup without
progress or a download. No uninstall action is added by this spec.

## 10. Uninstalling a mapset (spec 022)

Mapset detail offers **Uninstall** below the active-profile action for installed mapsets only —
including installs no longer in the catalog (spec 020, R5) — and it is unreachable while the game
runs. There is no separate Library shortcut: from Library, **A** opens detail, where the action
lives (coordinator clarification, 2026-10-17).

The second detail action is the spec 020 **Update** action and is rendered only when an update is
available (with **Keep current** in the third slot); otherwise the column goes straight from the
primary **Play setup** / **Install** action to the favorite, profile and Uninstall actions, so no
label is ever repeated.

The confirmation is the existing Ember sheet. It names the mapset and its installed size, and has
one controller-toggleable checkbox, **“Also delete profiles and saves (N profiles)”**, which starts
**unchecked** every time the sheet opens. **A** on the checkbox toggles it — no text entry — while
**Cancel** is focused first, so **A** cancels immediately; **B** closes the sheet and keeps
everything. Unchecked, the sheet states that only the install files are removed and profiles and
saves are kept; checked, it states that profiles, saves and screenshots will be deleted too. When
the install includes a user-imported component (D-014), the sheet also says imported components are
deleted. Confirming runs the removal and returns to Library with a non-blocking toast.

Removal is crash-safe and contained: the install record is marked inactive atomically **before** any
file is deleted, so an interrupted uninstall is never a launchable half-install; only paths inside
`<data>/library/<id>` are removed (canonicalised; symlinks are never followed out); leftovers are
cleaned on the next start. With the checkbox, the mapset's profile documents are removed first and
then their save/data/shot/run directories (spec 018 order). Favorites stay — they are a catalog
preference — and the shared download cache and IWAD links are untouched. `ddl mapset uninstall <id>
[--delete-profiles]` performs the same transaction headlessly and refuses while the UI holds the
state lock.

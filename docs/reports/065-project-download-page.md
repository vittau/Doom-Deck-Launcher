# 065 — Launcher-themed project download page

## Outcome

Implemented specs 065 R1–R5 in the existing GitHub Pages site, without publishing.
The page leads with the Steam DOOM owner’s next community-map playthrough, a prominent
signed `.flatpakref` download, and a graphical Desktop Mode → Discover installation path.
The CLI fallback, repository information and short hardware-testing limitation are secondary.

**Design approval:** early 1280×800 headless preview reviewed by the coordinator and sent
to the owner. The owner explicitly approved: “Sim está ótima direção visual.” The follow-up
copy request was applied: research labels and render terminology were removed from the main
player flow; the secondary disclosure says the install flow is expected and physical Deck
installation, controls and gameplay still need testing.

## Changes and scope

- `packaging/flatpak/site/index.html`: responsive project/install page, product features,
  existing launcher previews, Discover open-file steps, manual Add to Steam and compatibility
  tool unset instructions, legal IWAD ownership, signed-channel/update information, tarball
  alternative, source/guide/contribution/license links and independence/trademark statement.
- `packaging/flatpak/site/style.css`: local Barlow/Pirata One typography, Rose Compass and
  Ember Cathedral palette, original cathedral scene, bronze install actions, responsive
  layouts, visible focus, skip link, reduced-motion handling and large primary tap targets.
- `packaging/flatpak/site/credits.html`: original-art licensing, bundled font notices and
  existing mapset-image provenance/permission records. Existing launcher screenshots contain
  community map imagery, not original DOOM title artwork. Community images retain their
  authors’ rights and the repository’s D-024 removal-on-request convention; no new image
  permission is asserted.
- `scripts/flatpak/render-site.py`: standard-library renderer copies existing assets from
  their original repository locations, substitutes escaped publication inputs and rejects
  invalid inputs/unresolved template tokens. It does not generate or rewrite signed artifacts.
- `packaging/flatpak/site/test_render.py`: five offline acceptance/regression tests.
- `.github/workflows/flatpak-remote.yml`: **only Render the site** changed, replacing template
  `sed` with the renderer. Both signed descriptor heredocs remain byte-identical, as do all
  other workflow steps. No key, repository URL, GPG, build, publish or verification changes.

The deployment still derives the actual version from Cargo.toml and UTC publication date from
its existing publish step; local previews use the current version **0.0.9**, dated **2026-10-21**.
Canonical/social URLs use the approved `https://www.vitormach.dev/Doom-Deck-Launcher/` base.
The social preview is the existing licensed project export, not newly generated artwork.

No application, Cargo, catalog, coordinator-document or other workflow-section edits were made.
`git merge main` before the final focused checks returned **Already up to date**, with main at
`29d3645`. No push, merge into main, tag, release or publication was performed.

## Verification

**Confirmed — all final focused checks passed:**

- `python3 -B -m unittest discover -s packaging/flatpak/site -v`: **5 passed**. Checks published
  version/date, canonical/social URL and install target; local asset/link/fragment/CSS paths;
  preservation of signed-ref/repo/summary/signature/Git bytes across repeated renders; rejection
  of invalid publication inputs; HTML escaping of URL attributes.
- Rendered with `python3 -B scripts/flatpak/render-site.py --output /tmp/ddl-site-065
  --version 0.0.9 --date 2026-10-21 --app-id io.github.vittau.Doom-Deck-Launcher
  --site-base https://www.vitormach.dev/Doom-Deck-Launcher`.
- HTML Validate **11.17.0**: rendered `index.html` and `assets/credits.html` pass its default
  recommended rules. Initial doctype/void-tag style findings were fixed before the final run.
- Headless Playwright **1.62.1**, Chromium headless shell: **1440×1000**, **1280×800**,
  **390×844**, **320×700**. All images and local fonts loaded; document width matches viewport
  at every width. Keyboard Tab reaches the visible skip link with a solid focus outline;
  keyboard Enter opens the secondary disclosure.
- Axe **4.14.0**, WCAG 2 A/AA, 2.1 AA and 2.2 AA tags: **zero violations at all four widths**.
  This automated evidence supplements visual inspection; it does not establish complete
  accessibility certification. A capture initially timed out during scroll settling; the final
  complete run used reduced motion and produced all four PNGs successfully.
- Inspected the early desktop/narrow and final Deck-sized headless PNGs. The final PNGs include
  the open secondary disclosure and visible keyboard focus, so reviewers can inspect both.
- Live approved `.flatpakref` endpoint: **HTTP 200**, `application/octet-stream`; parsed app ID,
  `stable` branch and approved `/repo/` URL match. Its decoded GPG key is byte-identical to
  `packaging/flatpak/repo-key.gpg`. A headless click on the hero CTA downloads the live ref with
  the intended filename and bytes identical to the fetched published artifact. This verifies
  download behavior, not Discover installation or a new repository signature verification.
- Every GitHub `blob/main` file link in the rendered page/credits resolves to an existing local
  repository file. Local page links and fragments are covered by the offline tests.
- Programmatically compared the workflow against main: all text outside Render the site is
  identical, and both signed descriptor heredocs inside it are unchanged. Extracted Render
  step passes `bash -n`; `git diff --check` passes.

**No Cargo rebuild/tests:** this change contains static HTML/CSS and a Python page renderer,
with no application logic, Rust dependencies or asset-source changes. Existing application
PNGs are copied unchanged; rebuilding Slint would not validate this page. Python tests,
HTML validation, actual download behavior and headless browser checks exercise the changed
surface directly.

## Preview evidence (outside Git)

Early review artifacts:

- `/tmp/ddl-site-065-desktop-early.png`
- `/tmp/ddl-site-065-deck-early.png`
- `/tmp/ddl-site-065-narrow-early.png`

Final copy and responsive/focus/disclosure evidence:

- `/tmp/ddl-site-065-desktop-final.png`
- `/tmp/ddl-site-065-deck-final.png`
- `/tmp/ddl-site-065-narrow-final.png`
- `/tmp/ddl-site-065-small-final.png`
- `/tmp/ddl-site-065/index.html` (rendered site; approximately **2.7 MB** of page assets)

Temporary QA scripts: `/tmp/ddl-site-preview.cjs`, `/tmp/ddl-site-a11y.cjs`,
`/tmp/ddl-site-download.cjs`. These are development evidence, not production dependencies.

## Dependencies and remaining work

**Production dependencies added: none.** Python standard library only; static page has no
JavaScript runtime, framework, build step, CDN fonts or analytics. Existing images/fonts are
copied on publication. Temporary validation tooling installed outside the repo under
`/tmp/ddl-site-qa`: HTML Validate, axe Playwright integration and Prettier; Chromium headless
shell was downloaded for the pre-existing Playwright tool. No dependency manifests changed.

**Needs Deck Validation:** actual Discover/open-file interaction and the end-to-end install,
Steam shortcut, controller, audio/graphics and gameplay experience. Browser download tests and
CI signing/build checks do not establish hardware validation; the page makes no such claim,
performance benchmark, Flathub listing or official affiliation claim.

The linked guide’s Flathub section still describes a future channel. The page explicitly
points players to this page for the current self-hosted remote; changing the guide is outside
this worker’s ownership. No spec implementation items remain open, and visual direction is
owner-approved. Publication remains gated on the concurrent 063 workflow fixes (coordinator
reported the OSTree refs/remotes roundtrip failure), passing deployment checks and root review.

## Owner revision — compact hero support action (2026-10-21)

Moved the donation link into the hero action row and removed the oversized footer support
band. The secondary bronze treatment is approximately 157 × 44 px; the install action stays
visually primary. Explicit SVG width/height also prevent intrinsic sizing before CSS loads.
No dependencies added. Five offline renderer tests pass. Headless Chromium renders at
1280×800, 1280×720 and 390×844 confirm the support action is fully inside the first viewport,
with no horizontal overflow. Desktop render inspected visually. No application code changed.
The two paused OpenCode conversations remain paused.

## Independent website publication (owner revision, 2026-10-21)

Added `project-site.yml`: automatic page/asset changes on main and manual dispatch now refresh
only page-owned files. The job reuses the already published app version/date, rejects changes
outside index.html/assets, and shares the existing publication lock. Deployment wait, Cloudflare
prefix purge and canonical HTML/CSS/ref/summary verification remain in place. No SDK, Rust
build, GPG private key or new dependency is required. `render-site.py --refresh` fails closed
when existing release metadata is missing, invalid or ambiguous. Seven offline tests pass,
including refresh metadata/signature preservation and rejected-input checks; actionlint passes.
Application release publication still renders the page with its new version as before.

Build CI and artifact-only Flatpak path filters now exclude page-only templates/assets, the
page renderer and (for CI) the website workflow. Application/build inputs retain their checks.
The website job owns the offline renderer checks.

Independent deployment run `39551167942` passed end to end: page refresh/push, Pages build,
Cloudflare prefix purge and canonical HTML/CSS/ref/summary checks. The compact hero support
action is live. Redundant build run `39550620509` was cancelled during compilation, before
export/publication; the previously published app repository remains intact.

## Header placement and browser cache correction (owner revision, 2026-10-21)

Moved the compact donation action to the header navigation, labelled Donate with an explicit
Buy Me a Coffee accessible name. The hero again contains only the install action. Desktop
button bounds are approximately 93×44 px. Headless 1280×800, 390×844 and 320×740 checks show
no horizontal overflow and the donation action remains in the header. Desktop render inspected.
The owner's screenshot showed an unstyled underlined link; the served stylesheet has a one-day
browser cache lifetime. Added a CSS content hash to its URL so browser caches cannot retain
an earlier stylesheet under the same asset URL. Eight offline renderer tests pass, including
verification that the rendered stylesheet URL matches its content hash. No dependencies added.

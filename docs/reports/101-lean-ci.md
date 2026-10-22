# 101 — Keep routine CI proportional to the change

## Result

Implemented R1–R6 with workflow/documentation changes only. No production Rust, source tests,
Cargo files, release workflow, signed Flatpak publication workflow or website workflow changed.
No new dependencies or test framework. `main` at `1ae6f28` was merged before final static
verification (`Already up to date`).

## Work removed and coverage retained

- Replaced broad routine CI exclusions with source inputs: `crates/**`, `catalog/**`,
  `tests/**`, `docs/design/**` reference art, Cargo manifest/lockfile, toolchain pin and
  `.cargo/**`, excluding Markdown. Docs, packaging, scripts and workflow-only edits do not
  start Rust CI. Website publication retains its independent triggers unchanged.
- Removed the entire routine `steamrt-build` job: its separate pinned SDK environment,
  Rust installation/cache, release compilation, version/help smoke tests and `ldd` step.
  The authoritative tagged Steam Runtime build and smoke checks remain in `release.yml`.
- Removed the separate PR `catalog-lint` job, toolchain setup, target cache and system package
  installation. Its offline `cargo run --locked -p ddl -- catalog lint catalog/` now runs
  in `checks`, reusing that job's compiled workspace. Catalog PRs still lint; catalog pushes
  retain bundle publication, changed-path detection and first-publication fallback.
- Ordinary tests skip only the nine `headless::journeys::title_fit_*` cases and
  `headless::journeys::categories_favorites_and_screenshots`. Source inspection confirms
  exactly ten matching cases. All other model, navigation, persistence and focused headless
  tests remain, as do fmt and denied-warning workspace Clippy. No source test was changed.
- CI manual `full_tests=true` runs the unfiltered `cargo test --workspace --locked` and
  cannot publish a catalog, even with `publish_catalog=true`. Explicit manual publication
  is `publish_catalog=true`, `full_tests=false` on `main`. Both inputs default to false.
- Removed all push/PR triggers from artifact-only `flatpak.yml`; its complete build and smoke
  steps remain manually available. Lightweight `metadata.yml` automatically handles Flatpak
  files, helpers, workflow changes and workspace version changes with existing offline helper
  tests, XML well-formedness and first AppStream release == workspace version. It has no
  Rust install/build, SDK, package installation, secrets or publication.
- Preserved every signed/tag publication, integrity, AppStream, vendor verification and smoke
  gate in the unchanged `flatpak-remote.yml` and `release.yml`. Full desktop/AppStream
  validation remains in manual artifact builds and signed publication, beyond the lightweight
  XML/version check.
- Updated developer/Flatpak/orchestration guidance: select focused checks by affected behavior,
  use one authoritative full-suite result when warranted, avoid habitual macOS/Linux repeats,
  and do not rerun a full suite for follow-ups that cannot invalidate it. Linux release/build
  smoke verification remains required. Network e2e is an opt-in relevant download/archive/engine
  diagnostic, not a metadata, copy, catalog-only or UI release gate; its executable YAML is
  unchanged and no download case, timeout, hash or allowlist was weakened.

## Verification

No Cargo fmt/Clippy/tests, Rust build, SDK build, Flatpak build or network e2e was run for
this workflow/documentation task, as explicitly required by the owner.

- `actionlint .github/workflows/ci.yml .github/workflows/flatpak.yml
  .github/workflows/metadata.yml .github/workflows/e2e.yml`: the installed actionlint's runner
  list rejects the pre-existing `ubuntu-26.04` in Flatpak. Re-ran with only that exact known
  diagnostic suppressed using `-ignore '^label "ubuntu-26.04" is unknown'`: passed. No runner
  was changed to accommodate the older validator.
- Ruby standard-library YAML parsing: all nine checked-in workflows parsed. Python PyYAML
  is unavailable; no dependency was installed. Used the parsed workflow data for the static
  assertions below (accounting for Ruby YAML 1.1's boolean representation of `on`).
- Static source/packaging/docs trigger matrix passed for pushes and PRs, including Rust,
  Slint, embedded art/catalog, fixture, reference art, Cargo/toolchain and `.cargo` inputs;
  packaging/helpers/workflow paths get metadata checks without Rust; README/developer/docs,
  orchestration scripts, LICENSE and website paths do not start Rust. Markdown within a
  source/reference directory also stays excluded.
- Static job/step condition checks passed for 72 event/ref/input combinations (push, PR,
  manual; main, topic, tag; both boolean inputs). Verified only `checks` and `catalog-bundle`
  jobs remain, the ordinary command has exactly the two required skip filters, manual full
  tests are unfiltered, catalog lint occurs once in `checks`, and full-suite dispatches never
  enter publication. Flatpak artifact and e2e have only manual triggers.
- `python3 -m unittest discover -s scripts/flatpak -p 'test_*.py'`: **27 passed**.
- `bash scripts/flatpak/test_served_urls.sh`: passed.
- Executed the actual inline workflow XML/version check: XML parsed and first checked-in
  AppStream release matched workspace **0.0.13**. Confirmed metadata steps contain no Rust/SDK
  build commands.
- Compared `release.yml`, `flatpak-remote.yml` and `project-site.yml` byte-for-byte to `main`:
  unchanged. Parsed e2e YAML matches `main` exactly; only explanatory comments changed.
- `git diff --check main`: passed. Static verification scratch files stay untracked/ignored
  under `.scratch/101/`; they are not a new repository test framework.

## Prior evidence and limitations

**Confirmed (coordinator-provided evidence):** on `7bf51c1`, 988 full local tests,
denied-warning Clippy/fmt, [Linux CI 39824796061](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39824796061)
and [Flatpak artifact 39824795822](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39824795822)
passed. [Manual e2e 39824824023](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39824824023),
attempts 1 and 2, failed while receiving archive.org's Terminal Voltage response after about
79 seconds; the downloader is unchanged. This failure is retained as evidence, not presented
as a passing diagnostic.

Actual CI duration improvements have **not yet been measured**. A future relevant GitHub run
must confirm hosted execution and supply timings before any speedup claim. No implementation
items remain open; hosted timing evidence and physical Deck validation are outside this task.

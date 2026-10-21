# 062 — Build a test Flatpak in GitHub Actions

**Status:** Ready · **Depends on:** 059
**Origin:** owner, 2026-10-20: "Certo, agora crie esses arquivos no projeto para compilarmos no GitHub Actions."

## Scope and authorization

The owner explicitly authorizes agent-written packaging files for this task, overriding
059's human-only file-authoring restriction for this CI test package. This does not authorize
a Flathub submission. Record the AI-assisted provenance honestly: the resulting manifest is
not eligible for Flathub under its current AI policy. No submission PR or release is in scope.

## Requirements

- **R1 — Package inputs.** Add a complete Flatpak manifest, desktop entry and AppStream
  metainfo for `io.github.vittau.Doom-Deck-Launcher`. Reuse the existing production icon,
  GPL license and font licenses. Build x86_64 only. Metadata must describe existing behavior,
  not claim Flathub availability, verification or hardware validation. Store packaging under
  `packaging/flatpak/`; use a local directory source so CI builds the exact checked-out commit
  without a self-referential commit pin.
- **R2 — Offline source build.** Use the maintained Freedesktop 26.08 SDK/runtime and Rust
  SDK extension, subject to verifying upstream availability. Build Rust and Slint from source
  with the lockfile and offline mode; prepare all Cargo sources before entering the build
  sandbox using the official Flatpak Cargo generator pinned to a full commit. Verify the
  compiler meets the repository MSRV. Do not change Rust versions or Cargo dependencies just
  to make packaging pass. Preserve the embedded catalog serial from the checkout's commit time.
- **R3 — Sandbox permissions.** Provide the permissions required by existing Slint rendering,
  audio, network downloads, gilrs controller input, IWAD discovery and the controller picker,
  based on `docs/research/flatpak.md` and current upstream docs. Use read-only host-content
  grants; no host shell execution, no Steam Input integration. Explain each grant in the
  report. The official engine remains downloaded at runtime and extract-first under spec 059;
  never download an IWAD or execute archive payloads during packaging.
- **R4 — CI artifact.** Add a separate GitHub Actions workflow for manual runs and relevant
  pull requests/pushes to main, using the Flatpak build environment rather than the Steam
  Linux Runtime tarball job. Pin external tools/images/actions to verified immutable
  references. Prefer existing first-party actions and direct builder commands. Bound the job
  timeout, set read-only workflow permissions, avoid duplicate concurrent builds, use no
  secrets or publication credentials. Validate desktop/metainfo, smoke-test `ddl --version`
  and `--help` inside the built sandbox without opening a window, and upload a `.flatpak`
  bundle, SHA-256 and useful failure logs. Never publish a GitHub Release or submit to Flathub.
- **R5 — Instructions.** Add developer-facing instructions for the manual workflow, downloading
  and installing its artifact, refreshing Cargo sources and understanding local-vs-published
  sources. Clearly label the package experimental and AI-assisted. Explain that Deck checks
  (GL/audio/pad, IWAD visibility, import, OSK) remain open. Keep existing tarball instructions.

## Acceptance

- **A1 (R1/R3):** manifest parses; desktop and AppStream validate; all referenced repository
  files exist and installation destinations match the app ID. Inspect the permission list.
- **A2 (R2/R4):** workflow syntax/static checks and meaningful source-generation validation;
  report the pinned upstream references and compiler handling. A real Linux CI run must prove
  offline build, sandbox CLI smoke checks and bundle creation when remote execution is available.
- **A3 (R5):** documentation describes the actual workflow/artifact and provenance accurately.
- **A4:** report at `docs/reports/062-flatpak-ci-package.md`, including commands/results,
  dependencies, limitations and remaining Deck validation. Existing application behavior is
  unchanged; if no Rust code changes, explain why full macOS workspace tests add no evidence
  for this Linux-only packaging change. Do not claim a CI run passed if it was not run.

Never open visible windows or capture the screen — headless verification only.

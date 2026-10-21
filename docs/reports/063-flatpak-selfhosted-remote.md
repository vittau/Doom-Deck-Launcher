# 063 — Signed Flatpak repository on GitHub Pages

**Spec:** [`docs/specs/063-flatpak-selfhosted-remote.md`](../specs/063-flatpak-selfhosted-remote.md)
**Status:** implementation complete, not yet run · 2026-10-20
**Branch:** `vittau/flatpak-selfhost-remote` (on top of `vittau/flatpak-ci-package`)

## What changed

The Flatpak package now has a real install channel: a signed OSTree repository published to this
repository's `gh-pages` branch and served by GitHub Pages at
<https://www.vitormach.dev/Doom-Deck-Launcher/>.

| File | Purpose |
| --- | --- |
| `.github/workflows/flatpak-remote.yml` | Tag/manual publish job: build, sign, export, smoke-test, render, push `gh-pages` |
| `packaging/flatpak/site/index.html` | Landing page template (`@@VERSION@@`, `@@DATE@@`, `@@APP_ID@@`, `@@SITE_BASE@@`) |
| `packaging/flatpak/repo-key.gpg` | Public half of the repository signing key; the workflow refuses to sign if the secret differs |
| `scripts/flatpak/make-signing-key.sh` | One-time repository key generation, prints the `gh secret set` commands |
| `docs/flatpak-remote.md` | Player install/update/remove, maintainer setup, key rotation, Flathub path |
| `docs/specs/063-flatpak-selfhosted-remote.md` | Requirements and acceptance criteria |
| `docs/flatpak-build.md` | Points at the repository channel; stale "never publishes" claim corrected |

No application code, catalog, dependency, toolchain or manifest change. `packaging/flatpak/**`,
`docs/flatpak-build.md` and the new scripts/workflow are the whole diff.

## Design decisions

1. **Same repository, `gh-pages` branch.** The owner rejected a separate `ddl` repository. GitHub
   Pages serves a project site under the repository name, so the URL is
   `https://www.vitormach.dev/Doom-Deck-Launcher/`. No PAT, no second repository; the workflow uses
   the scoped `GITHUB_TOKEN` with `contents: write` and only ever writes `gh-pages`.
2. **Signed, no unsigned fallback.** `.flatpakref`/`.flatpakrepo` carry `GPGKey` and Flatpak has no
   verification toggle in those files (confirmed in the Flatpak 1.18.4 command reference), so an
   unsigned remote could not offer one-command install. A dedicated sign-only RSA-4096 key is
   generated in a throwaway keyring by the script and stored only as
   `FLATPAK_GPG_PRIVATE_KEY` (+ optional `FLATPAK_GPG_KEY_ID`).
3. **Branch `stable`, not `master`.** Matches the Flathub convention and the `.flatpakref`
   `Branch=stable`. The artifact workflow keeps its existing `master` export untouched.
4. **Incremental repository.** The job clones the previous `gh-pages`, exports into its `repo/`,
   signs, generates static deltas, prunes to three builds (`--prune-depth=2`) and pushes. Users get
   delta updates instead of a full re-download.
5. **Reproducible commit metadata.** `flatpak build-export --timestamp=<tag date> --subject=<commit
   subject> --body=<sha>` ties the OSTree commit to the tagged source instead of the run clock; a
   re-run of the same tag is usually a no-op.
6. **Signature verification before publication.** The exported repository is verified the way a
   client would: the committed `repo-key.gpg` is imported into a fresh keyring and required to verify
   `summary.sig`, then a throwaway remote is added with `--gpg-import=packaging/flatpak/repo-key.gpg`
   and the app is installed **with verification on** (no `--no-gpg-verify`). `ddl --version` is
   checked against `Cargo.toml`. A build that cannot start, or whose signature the committed key
   rejects, never reaches the site.
7. **Runtimes stay on Flathub.** `RuntimeRepo=https://dl.flathub.org/repo/flathub.flatpakrepo`;
   only the app comes from this remote.
8. **`main` untouched.** The publication writes exactly one branch. No GitHub Release is created;
   `release.yml` still owns releases.

## Requirement verification

| Requirement | Evidence |
| --- | --- |
| R1 signed repository | `build-export --gpg-sign`, `build-update-repo --gpg-sign`, `GPGKey` in both generated ref files, and a smoke test that imports `packaging/flatpak/repo-key.gpg`, verifies `summary.sig` and installs with GPG verification enabled |
| R2 tag + manual pipeline | `on: push: tags: v*` and `workflow_dispatch` with a `ref` input; build → verify → sign → export → smoke → render → push |
| R3 stable URLs | Render step writes `<app-id>.flatpakref`, `ddl.flatpakrepo`, `index.html`, `.nojekyll`; `repo/` comes from `build-export`; paths verified by dry-running the render block locally |
| R4 key handling | `scripts/flatpak/make-signing-key.sh`: isolated `GNUPGHOME`, `umask 077`, refuses to overwrite, never prints private material; workflow fails early without the secret and imports into `$RUNNER_TEMP/gnupg` |
| R5 documentation | `docs/flatpak-remote.md` covers install/update/remove, setup, security, rotation, rollback and the Flathub path |

## Verification run

| Check | Command | Result |
| --- | --- | --- |
| Workflow lint | `actionlint -ignore 'label "ubuntu-26.04" is unknown' .github/workflows/flatpak-remote.yml` | pass |
| Shell lint | `shellcheck scripts/flatpak/make-signing-key.sh` | pass |
| YAML parse | `ruby -ryaml -e 'YAML.load_file(...)'` | parses (`on` read as a boolean key by YAML 1.1, as expected) |
| HTML parse | `html.parser` over the rendered template | no unclosed tags, no `@@…@@` placeholders left |
| Render dry run | heredoc + `sed` with sample values | `.flatpakref` key/URL lines correct, no indentation |
| Whitespace | `git diff --check` | pass |

Not run, because there is no Flatpak runtime on the macOS dev host:

- `flatpak install --user --from …flatpakref` on a Linux host or Deck.

The first publish run ([39537960687](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39537960687))
proved the build, the signed export, the existing smoke test and the `gh-pages` push on a Linux
runner. Pages was enabled afterwards: the homepage and `repo/summary` are served (HTTP 200) and the
`.flatpakref` is served with a cache-busting query, with key, URL and app id checked. The bare
`.flatpakref` URL was still returning 404 from a Cloudflare negative cache at the time of writing.

## Execution log (2026-10-21)

The owner authorised the channel, the signing key and the Pages setup, so the work moved from
implementation to execution. No private key material is reproduced here.

| Step | Result |
| --- | --- |
| Repository key | Generated with `scripts/flatpak/make-signing-key.sh`; fingerprint `EF50C91A6703B13327AEAF00962CEACA8DE90556`; public half committed as `packaging/flatpak/repo-key.gpg`; secret stored as the `FLATPAK_GPG_PRIVATE_KEY` repository secret, fingerprint as `FLATPAK_GPG_KEY_ID`; backup kept outside the repository |
| Integration | Spec 062 and this branch merged into `main` (`cfb7b36`, `c7efd6c`) and pushed to `origin/main`; later manifest fixes merged as `2e76897` and `9808e6b` |
| First publish run | [39537960687](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39537960687) — success: build, signed export, smoke test, site render and push to `gh-pages` |
| Pages | Enabled by the coordinator on the `gh-pages` root; the homepage and `repo/summary` return HTTP 200, and the `.flatpakref` returns 200 with a cache-busting query (key, URL and app id checked). The bare URL still returns 404 from a Cloudflare negative cache; that is transient |
| Signature smoke test | Review follow-up: the throwaway remote now imports `packaging/flatpak/repo-key.gpg` and installs with GPG verification on, plus an explicit `gpg --verify summary.sig summary` |

CI history while getting there:

| Run | Workflow | Result | Cause and fix |
| --- | --- | --- | --- |
| [39537456681](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39537456681) | Flatpak repository | failure | `cd rust-x86_64-unknown-linux-gnu` — flatpak-builder strips the archive's top-level directory. Fixed with `./install.sh` at the build root (`c72daf5`) |
| [39537769632](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39537769632) | Flatpak repository | failure | `eu-strip` is not in the 26.08 SDK and debuginfo extraction aborts the module. Fixed with `no-debuginfo: true` on both modules (`84ebd15`) |
| [39537960702](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39537960702) | Flatpak (artifact) | failure | The manifest check imports `yaml`, but the generator venv installed only `aiohttp` and `tomlkit`. Fixed by pinning `PyYAML==6.0.3` (`2fa9dc3`) |
| 39537765626 / 39537765603 | Flatpak / CI | failure / cancelled | Same two manifest causes as above, superseded |

## What the owner still has to do

1. ~~Generate the key and set the secrets.~~ Done on 2026-10-21.
2. ~~Publish once to create `gh-pages`.~~ Done by run 39537960687.
3. **Enable Pages** (*Settings → Pages → Deploy from a branch → `gh-pages` / `/`*) and confirm the
   public URLs. The coordinator took this over.
4. Publish future versions with `git tag vX.Y.Z && git push origin vX.Y.Z`.

## Open questions for the coordinator

1. **Decision record.** This adds a distribution channel. `docs/decisions.md` was not edited; a new
   entry should state: self-hosted signed Flatpak repo as the experimental channel, Flathub still
   not submitted, `gh-pages` as the only write target of the publish workflow.
2. **README.** No Flatpak link was added to `README.md`; the tarball stays the supported install.
   Say the word if the experimental channel should be advertised there.
3. **Branch stack.** This branch sits on `vittau/flatpak-ci-package` (spec 062), which is still
   unreviewed. Merge 062 first, then this one.
4. **Repository size.** Every release keeps `repo/` history in `gh-pages`; pruning bounds growth but
   Pages has a 1 GB published-site limit. If it becomes a problem, move to a dedicated repository or
   to Pages artifact deployment.

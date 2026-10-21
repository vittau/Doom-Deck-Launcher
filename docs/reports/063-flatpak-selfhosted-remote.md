# 063 — Signed Flatpak repository on GitHub Pages

**Spec:** [`docs/specs/063-flatpak-selfhosted-remote.md`](../specs/063-flatpak-selfhosted-remote.md)
**Status:** merged into `main` and exercised through the `gh-pages` push; the incremental-export fix
and the explicit Pages request still need a fresh successful run · 2026-10-21
**Integration:** `origin/main`, currently `1dc6e8e`

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
   delta updates instead of a full re-download. Git cannot store empty directories, so the clone
   loses `tmp/`, `state/`, `extensions/`, `refs/mirrors/` and `refs/remotes/`; OSTree only creates
   that layout for a new repository (`repo_create_at_internal` returns early when `objects/` exists),
   so the job restores the full set before exporting. Without that, `build-update-repo` fails with
   `Listing refs: opendir(refs/remotes): No such file or directory` while generating static deltas.
5. **Reproducible commit metadata.** `flatpak build-export --timestamp=<tag date> --subject=<commit
   subject> --body=<sha>` ties the OSTree commit to the tagged source instead of the run clock; a
   re-run of the same tag is usually a no-op.
6. **Signature verification before publication.** The exported repository is verified the way a
   client would: the committed `repo-key.gpg` is imported into a throwaway remote and the app is
   installed **with GPG verification on** (no `--no-gpg-verify`), so Flatpak/OSTree verify the
   summary and the commit signatures. `repo/summary.sig` is not fed to `gpg` directly: OSTree stores
   it as a GVariant map (the `ostree.gpgsigs` key, confirmed by inspecting the published file), not
   as a raw OpenPGP signature. `ddl --version` is checked against `Cargo.toml`.
7. **Pages deployment is explicit.** Pushes made with `GITHUB_TOKEN` do not trigger a Pages build,
   so the workflow requests one through `POST /repos/{owner}/{repo}/pages/builds` (permission
   `pages: write`, bounded with `--max-time`), waits for the build of the pushed commit and then
   re-checks the served `.flatpakref` and `repo/summary` until the summary is byte-identical to the
   exported one. A stale CDN copy is retried and fails the job on exhaustion.
8. **Runtimes stay on Flathub.** `RuntimeRepo=https://dl.flathub.org/repo/flathub.flatpakrepo`;
   only the app comes from this remote.
9. **`main` untouched.** The publication writes exactly one branch. No GitHub Release is created;
   `release.yml` still owns releases.

## Requirement verification

| Requirement | Evidence |
| --- | --- |
| R1 signed repository | `build-export --gpg-sign`, `build-update-repo --gpg-sign`, `GPGKey` in both generated ref files, and a smoke test that imports `packaging/flatpak/repo-key.gpg` into a throwaway remote and installs with GPG verification enabled, so Flatpak/OSTree verify the summary and the commit |
| R2 tag + manual pipeline | `on: push: tags: v*` and `workflow_dispatch` with a `ref` input; build → verify → sign → export → smoke → render → push → explicit Pages build request → served-file check. `pages: write` is declared for the REST build request |
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
| OSTree layout roundtrip | local git repo with `repo/{config,objects,refs/heads/…,summary}`, committed and cloned | the clone loses `tmp`, `state`, `extensions`, `refs/mirrors`, `refs/remotes` (empty directories); the restore loop recreates all of them |
| Whitespace | `git diff --check` | pass |

Not run, because there is no Flatpak runtime on the macOS dev host:

- `flatpak install --user --from …flatpakref` on a Linux host or Deck.

The first publish run ([39537960687](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39537960687))
proved the build, the signed export, the existing smoke test and the `gh-pages` push on a Linux
runner. Pages was enabled afterwards and all three public URLs (site, `.flatpakref`,
`repo/summary`) return HTTP 200; the earlier bare-URL 404 was a Cloudflare negative cache that has
since expired.

Still unproven in CI:

- the incremental export over the existing `gh-pages` after restoring the OSTree layout;
- the `pages: write` REST request, its wait and the served-file comparison.

The Pages endpoint itself was exercised against this repository: `POST /pages/builds` returned
`queued` and the build for the `gh-pages` head `953ccd5` reached `built`. The most recent manual
dispatch ([39542355868](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39542355868)) did
not reach the build: it passed `0dd4749` (a short SHA) as the `ref` input, which `actions/checkout`
treats as a ref name (`refs/heads/0dd4749*`). Manual runs need a tag, a branch or a full
40-character SHA.

## Execution log (2026-10-21)

The owner authorised the channel, the signing key and the Pages setup, so the work moved from
implementation to execution. No private key material is reproduced here.

| Step | Result |
| --- | --- |
| Repository key | Generated with `scripts/flatpak/make-signing-key.sh`; fingerprint `EF50C91A6703B13327AEAF00962CEACA8DE90556`; public half committed as `packaging/flatpak/repo-key.gpg`; secret stored as the `FLATPAK_GPG_PRIVATE_KEY` repository secret, fingerprint as `FLATPAK_GPG_KEY_ID`; backup kept outside the repository |
| Integration | Spec 062 and this branch merged into `main` (`cfb7b36`, `c7efd6c`); manifest fixes followed as `2e76897` and `9808e6b`; the layout fix was reviewed and merged as `1f66a25`. `origin/main` is now `1dc6e8e` |
| First publish run | [39537960687](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39537960687) — success: build, signed export, smoke test, site render and push to `gh-pages` |
| Pages | Enabled by the coordinator on the `gh-pages` root (`build_type: legacy`, source `gh-pages`). The site, the `.flatpakref` and `repo/summary` all return HTTP 200; the earlier bare-URL 404 was a Cloudflare negative cache that expired |
| Pages trigger | Confirmed that `POST /repos/…/pages/builds` returns `queued` and the build for `953ccd5` (the `gh-pages` head) reached `built` in about 20 s. The workflow now requests this build explicitly, because a `GITHUB_TOKEN` push does not trigger Pages |
| `summary.sig` format | Downloaded the published file: it starts with `ostree.gpgsigs`, an OSTree GVariant map, not a raw OpenPGP signature. The `gpg --verify` approach was removed; verification goes through Flatpak/OSTree with `--gpg-import` and normal GPG verification |
| Signature smoke test | Review follow-up: the throwaway remote imports `packaging/flatpak/repo-key.gpg`, runs `remote-modify --gpg-verify` (which also enables summary verification for a remote without a collection ID) and installs with verification on |
| Manual dispatch input | [39542355868](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39542355868) passed `0dd4749` (short SHA) as `ref`; `actions/checkout` resolved it as `refs/heads/0dd4749*` and failed before any build step. The input accepts a tag, a branch or a full 40-character SHA |

CI history while getting there:

| Run | Workflow | Result | Cause and fix |
| --- | --- | --- | --- |
| [39537456681](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39537456681) | Flatpak repository | failure | `cd rust-x86_64-unknown-linux-gnu` — flatpak-builder strips the archive's top-level directory. Fixed with `./install.sh` at the build root (`c72daf5`) |
| [39537769632](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39537769632) | Flatpak repository | failure | `eu-strip` is not in the 26.08 SDK and debuginfo extraction aborts the module. Fixed with `no-debuginfo: true` on both modules (`84ebd15`) |
| [39537960702](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39537960702) | Flatpak (artifact) | failure | The manifest check imports `yaml`, but the generator venv installed only `aiohttp` and `tomlkit`. Fixed by pinning `PyYAML==6.0.3` (`2fa9dc3`) |
| [39540926300](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39540926300) | Flatpak (artifact) | success | First green artifact run with the PyYAML pin |
| [39540938605](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39540938605) | Flatpak repository | failure | After export: `Listing refs: opendir(refs/remotes): No such file or directory`. The `gh-pages` git roundtrip drops empty OSTree directories; fixed by restoring the layout after the clone |
| [39542355868](https://github.com/vittau/Doom-Deck-Launcher/actions/runs/39542355868) | Flatpak repository | failure | Manual dispatch passed `0dd4749` (short SHA) as `ref`; `actions/checkout` treats it as a ref name and fails before any build step. Input mistake, no code fix |
| 39542511716 | Flatpak repository | cancelled | Mistaken invocation, superseded immediately |
| 39537765626 / 39537765603 | Flatpak / CI | failure / cancelled | Same two manifest causes as above, superseded |

## Current state and remaining work

Setup and first publication are done: the signing key and secrets exist, Pages serves the site on
the `gh-pages` branch, and the signed repository has been published once. The channel remains
experimental; it is not a stable release and has no Flathub listing.

- **Publishing a version is a release decision.** The workflow runs on `v*` tags or by manual
  dispatch with a tag, a branch or a full 40-character SHA. Creating a tag triggers this channel,
  but nothing here approves a stable release.
- **Deck validation is still open:** extracted DSDA-Doom with working GL/audio/pad, IWAD visibility,
  tarball-era import, OSK in Gaming Mode, launch from Steam.

## Open questions for the coordinator

1. **README.** No Flatpak link was added to `README.md`; the tarball stays the supported install.
   Say the word if the experimental channel should be advertised there.
2. **Repository size.** Every release keeps `repo/` history in `gh-pages`; pruning bounds growth but
   Pages has a 1 GB published-site limit. If it becomes a problem, move to a dedicated repository or
   to Pages artifact deployment.

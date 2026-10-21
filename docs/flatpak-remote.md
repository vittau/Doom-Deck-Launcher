# Install DoomDeckLauncher from the Flatpak repository

Status: **experimental and AI-assisted.** The repository is signed with a dedicated project key and
served from GitHub Pages, but it is **not** a Flathub release, has **no** store page and has **not**
been validated on a Steam Deck yet. Spec 063 defines the scope; the release tarball in the
[README](../README.md#install-on-steam-deck) remains the supported channel.

- Site: <https://www.vitormach.dev/Doom-Deck-Launcher/>
- App id: `io.github.vittau.Doom-Deck-Launcher`
- Branch: `stable`
- Runtime: `org.freedesktop.Platform//26.08`, installed from Flathub
- Architecture: `x86_64` only

## Install on a Steam Deck

Desktop Mode → Konsole:

```sh
flatpak install --user --from \
  https://www.vitormach.dev/Doom-Deck-Launcher/io.github.vittau.Doom-Deck-Launcher.flatpakref
```

The `.flatpakref` embeds the repository's public key and points the runtime at Flathub, which
SteamOS already has configured. No `remote-add`, no key import, no zip to download. Flatpak 1.16 or
newer is required (`--device=input`); SteamOS 3.10 ships 1.18.

Then add it to Steam as described in the [Deck guide](steam-deck-guide.md): shortcut to
`/usr/bin/flatpak` with launch options `run io.github.vittau.Doom-Deck-Launcher`, compatibility tool
**none**.

## Update and remove

```sh
flatpak update --user io.github.vittau.Doom-Deck-Launcher
flatpak uninstall --user io.github.vittau.Doom-Deck-Launcher
```

Discover also offers the update on its next check. Configuration, mapsets, engines and profiles live
in `~/.var/app/io.github.vittau.Doom-Deck-Launcher/` and survive updates. `flatpak uninstall
--delete-data` removes them too.

The published repository keeps the last three builds (`--prune-depth=2`); updates always move
forward. There is no downgrade path from the CLI other than reinstalling an old build by hand.

## Troubleshooting

| Symptom | Cause / fix |
| --- | --- |
| `error: No such ref` right after a release | GitHub Pages has not rebuilt yet. Wait a minute; the publish job retries for ~90 s and warns if it is slower. |
| `404` on the `.flatpakref` right after the first publication | A CDN may cache the earlier negative response. Retry with a query string (`…flatpakref?v=1`) or wait for the cache to expire; the file itself is published. |
| `Cloudflare purge skipped` in the log | Neither `CLOUDFLARE_CACHE_PURGE_TOKEN` nor `CLOUDFLARE_ZONE_ID` is set. Publication is still valid; the edge copy expires normally. |
| `Cloudflare purge misconfigured` | Only one of the two settings exists. Set both or neither; the job fails rather than publishing with half a configuration. |
| `Cloudflare purge failed` | The API rejected the request or all bounded retries failed. The message names the HTTP status and Cloudflare error; credentials are never printed. |
| `GPG verification failed` | The repository key was rotated. Remove and re-add the remote (see below). |
| `flatpak install` says the runtime is missing | Add Flathub once: `flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo`. |
| `flatpak remote-ls ddl` is empty | The remote points at the wrong URL. Check `flatpak remotes --show-details` for `ddl`. |
| App installed, no window | Expected permissions are `--socket=wayland`/`fallback-x11`; run `flatpak run io.github.vittau.Doom-Deck-Launcher` from a terminal and read stderr. |

Useful inspection commands:

```sh
flatpak remotes --show-details
flatpak remote-info --user ddl io.github.vittau.Doom-Deck-Launcher
flatpak info --user --show-metadata io.github.vittau.Doom-Deck-Launcher
```

## How publication works

`.github/workflows/flatpak-remote.yml` runs on `v*` tags (and manual dispatch) and:

1. checks out the tagged commit and verifies `cargo-sources.json` matches `Cargo.lock`;
2. validates the desktop entry and metainfo;
3. imports the repository key from the `FLATPAK_GPG_PRIVATE_KEY` secret into a temporary
   `GNUPGHOME`;
4. builds with `flatpak-builder` using the same manifest as the artifact-only job;
5. exports the build as a **signed** commit on branch `stable` with
   `flatpak build-export --timestamp=<tag date>`, then signs the summary and generates static deltas
   with `flatpak build-update-repo --generate-static-deltas --prune --prune-depth=2`;
6. imports `packaging/flatpak/repo-key.gpg` into a throwaway remote with GPG verification enabled,
   then lists and installs from it (Flatpak/OSTree verify the summary and commit signatures; the
   `repo/summary.sig` file itself is an OSTree GVariant map, not a raw OpenPGP signature);
7. renders `index.html`, `<app-id>.flatpakref`, `ddl.flatpakrepo` and `.nojekyll`, and pushes them
   plus `repo/` to the `gh-pages` branch with the workflow's `GITHUB_TOKEN`;
8. requests a GitHub Pages build through the REST API (`pages: write`) — a `GITHUB_TOKEN` push alone
   does not trigger one — and waits for the build of the pushed commit;
9. purges the single Cloudflare prefix `www.vitormach.dev/Doom-Deck-Launcher/` after the deployment,
   when `CLOUDFLARE_CACHE_PURGE_TOKEN` and `CLOUDFLARE_ZONE_ID` are configured, and then checks the
   served `.flatpakref` and `repo/summary`.

Published layout:

```text
/                                  landing page (this repo's gh-pages)
/io.github.vittau.Doom-Deck-Launcher.flatpakref
/ddl.flatpakrepo
/repo/                             OSTree remote: summary, config, objects/, deltas/
```

`main` is never touched, and no GitHub Release is created. The Flatpak repository history lives in
the `gh-pages` branch, which is why old builds consume repository size; pruning keeps it bounded.

## One-time setup (maintainer)

1. **Pages.** After the first publication creates the branch, enable Pages for this repository:
   *Settings → Pages → Build and deployment → Source: Deploy from a branch → `gh-pages` / `/`*.
   The site then appears at `https://www.vitormach.dev/Doom-Deck-Launcher/` (the custom domain and
   user site are already configured).

2. **Signing key.** On a trusted machine with `gpg` (macOS: `brew install gnupg`):

   ```sh
   scripts/flatpak/make-signing-key.sh
   ```

   The script creates a dedicated sign-only RSA-4096 key in a throwaway keyring, refuses to
   overwrite an existing output directory, writes the secret as a single base64 line and prints the
   `gh secret set` commands. It does not touch your personal keyring.

   ```sh
   gh secret set FLATPAK_GPG_PRIVATE_KEY -R vittau/Doom-Deck-Launcher \
     < ddl-flatpak-signing-key/ddl-flatpak-secret.asc.b64
   gh secret set FLATPAK_GPG_KEY_ID -R vittau/Doom-Deck-Launcher \
     --body <fingerprint printed by the script>
   ```

   Back the secret file up offline, then delete it. `FLATPAK_GPG_KEY_ID` is optional, but setting it
   keeps the workflow unambiguous if the imported keyring ever holds more than one key.

3. **Cloudflare cache purge (optional).** The workflow can invalidate the DDL prefix right after a
   Pages deployment, so a stale edge copy cannot survive a release. It needs:

   ```sh
   gh secret set CLOUDFLARE_CACHE_PURGE_TOKEN -R vittau/Doom-Deck-Launcher
   gh variable set CLOUDFLARE_ZONE_ID -R vittau/Doom-Deck-Launcher --body <zone id>
   ```

   Use an API token with **Zone → Cache Purge** restricted to this zone (`vitormach.dev`); the token
   is read from the environment and never printed. With both settings absent the step reports a skip
   and publication continues; with only one present it fails. The purge covers exactly one prefix,
   `www.vitormach.dev/Doom-Deck-Launcher/` — never the zone, another host or `purge_everything` — so
   it also invalidates immutable OSTree objects under that tree, which is what the owner asked for.

4. **Publish.** Push a tag (`git tag v0.1.0 && git push origin v0.1.0`) or run
   *Actions → Flatpak repository → Run workflow* with a tag, a branch or a full 40-character commit
   SHA. Short SHAs are not refs and fail at checkout. Tags created before the Flatpak packaging
   existed (`v0.0.9` and older) fail the manifest check; use a newer tag.

No other secrets are required: the push to `gh-pages` uses the workflow's `GITHUB_TOKEN`, which is
scoped to this repository and cannot touch `main` history.

## Security model

- **Signed commits and summary.** Clients pin the public key from the `.flatpakref`; a compromised
  Pages site alone cannot push an unsigned build. The private key exists only as a repository
  secret and in the maintainer's offline backup. The public half is committed at
  `packaging/flatpak/repo-key.gpg`, and the workflow refuses to sign if the secret does not match it.
- **HTTPS.** GitHub Pages terminates TLS for `www.vitormach.dev`; the OSTree remote uses that URL.
- **No host execution.** The app still runs entirely inside the sandbox; the engine is downloaded at
  runtime and extracted, never mounted.
- **Cloudflare token.** The purge token needs only **Zone → Cache Purge** for this zone. It is read
  from the environment, never printed, and the purge is one prefix under the DDL path.
- **Scope of the token.** `GITHUB_TOKEN` with `contents: write` can push any branch of this
  repository, including `main`, though this workflow only writes `gh-pages`. Protect `main` with a
  branch ruleset if that residual scope matters.

## Key rotation and rollback

The key has no expiry. To rotate it: generate a new key with the same script, replace
`packaging/flatpak/repo-key.gpg` with the new public key, update both secrets, and publish once.
Users then see a verification failure, because the pinned key changed, and must reinstall:

```sh
flatpak uninstall --user io.github.vittau.Doom-Deck-Launcher
flatpak uninstall --user --unused
flatpak install --user --from https://www.vitormach.dev/Doom-Deck-Launcher/io.github.vittau.Doom-Deck-Launcher.flatpakref
```

Losing the private key without a backup has the same effect for every existing user. For a single
version rollback, `flatpak update --user --commit=<sha>` works while the old commit is still in the
pruned window; otherwise rebuild and republish the older tag.

## Path to Flathub

This remote does not conflict with a later Flathub submission: same app id, same manifest, same
runtime. The differences are procedural, not technical:

- Flathub requires the **manifest and submission text to be human-authored**; the current
  `packaging/flatpak/*.yml` is AI-assisted and says so. A submission needs the owner to write the
  manifest, metainfo and PR text by hand, and to disclose AI assistance in the application code.
- Flathub builds from its own `flathub/<id>` repository, so the source pin becomes
  `type: git` at a tag, and the `catalog-serial` sentinel has to be replaced by a value committed at
  release time.
- `home:ro` and the two `~/.var/app` grants need written exceptions; see
  [`docs/research/flatpak.md`](research/flatpak.md) §5.3 and §7.
- Users on this remote who later want Flathub can uninstall without `--delete-data` and reinstall
  from Flathub; the app id — and therefore the data directory — is the same.

## Still to validate on hardware

- Extracted DSDA-Doom starts with working GL, audio and controller input under `flatpak run`.
- IWADs are visible through `home:ro`, `/run/media:ro` and the two Flatpak app-data grants.
- The one-time import of a tarball-era `~/.config/ddl` / `~/.local/share/ddl` works.
- `xdg-open steam://open/keyboard` opens the on-screen keyboard in Gaming Mode.
- Steam launches the Flatpak shortcut from Gaming Mode; the overlay will not attach (Flatpak unsets
  `LD_PRELOAD`).

## Not verified yet

The first publish run succeeded: it built the app, exported and signed the repository, smoke-tested
the exported commit and pushed `gh-pages`. The CI smoke test now also proves the signature, by
importing `packaging/flatpak/repo-key.gpg`, verifying `summary.sig` and installing from a remote with
GPG verification enabled.

Pages is enabled: the homepage and `repo/summary` return HTTP 200, and the `.flatpakref` returns 200
with a cache-busting query (public key, URL and app id checked). The bare `.flatpakref` URL was still
serving a cached 404 from Cloudflare at the time of writing.

Still unverified:

- Installing the published `.flatpakref` on a real Steam Deck or Linux host.
- Runtime behaviour on hardware (GL, audio, controller, IWAD visibility, OSK).

See [`docs/reports/063-flatpak-selfhosted-remote.md`](reports/063-flatpak-selfhosted-remote.md) for
the full run history.

# 079 — Domain and Flatpak identity

## Outcome

Implemented the canonical `https://doomdecklauncher.com/` site and
`com.doomdecklauncher.DoomDeckLauncher` Flatpak identity. Started by fast-forwarding to
verification checkpoint `1bed4cf`, then integrated `verify/071-073` again after the other
worker's UI/CLI fixes landed. No dependencies added, no visible windows or screen capture,
no settings/DNS/remotes changed, and no push, tag, release, publication or secret access.

## Changes and acceptance

| Requirement | Implementation and evidence |
| --- | --- |
| R1 canonical domain | README, Deck setup, build/remote setup use the domain root. AppStream homepage and both descriptor homepages agree; GitHub source, release, screenshot and catalog transports retain their actual GitHub URLs. |
| R2 application identity | Renamed manifest, desktop and metainfo; updated app-id, desktop/icon/metadata/license exports, artifact/signed workflows, sandbox/path/visibility/import fixtures and diagnostics expectation. Binary `ddl`, signing key, developer identity `io.github.vittau`, product name, schema/release checks and tarball legacy import are retained. No old-ID alias/migration added. |
| R3 CNAME | Shared renderer writes exactly `doomdecklauncher.com` plus newline at the publication root on full render and refresh. Page-only changed-path allowlist permits `CNAME`; signed cleanup retains it. Offline tests run the real cleanup command and verify creation, preservation and correction of CNAME; signed artifacts stay byte-identical under page refresh. Shared lock and signing verification remain unchanged. Before page-only mutation, a bounded read-only guard requires the requested ID/stable ref, canonical remote URL, public key and summary/signature artifacts. A missing or mismatched channel yields a notice and successful deferral; all six publication/deployment/purge/verification steps are gated. Tests cover absent/wrong/oversized/malformed metadata, ready transition and byte-identical deferred files. |
| R4 root URLs | Both workflow bases are the apex origin; descriptor heredocs execute in offline tests and produce the expected new application ref, canonical homepage and `/repo/` URL with the supplied public key. Existing HTML canonical/OG/download templates now render at the root; asset/link checks map same-origin URLs to generated files from both pages. |
| R5 prefix purge | Allowlist is exactly `doomdecklauncher.com/`. Tests reject old sites, www, other paths, scheme/port/userinfo tricks, suffix hosts, query/fragment, encoded/double slashes and traversal. Payload remains one prefix; no broad purge. Retry, redaction, size limits, missing/partial settings behavior preserved. No live purge attempted. |
| R6 audit and validation | Remaining references are classified below. DNS, Pages and Cloudflare setup is documented with primary hosting sources. Local checks are recorded below; publication and physical installation remain external validation. |

## Local verification

- `python3 -m unittest discover -s packaging/flatpak/site`: **23 passed**, including eleven identity/publication/transition contract tests and root-resolved assets.
- `python3 -m unittest discover -s scripts/flatpak -p 'test_*.py'`: **23 passed**.
- `bash scripts/flatpak/test_served_urls.sh`: passed (root page, new descriptor, repository summary, canonical/cache-busted variants).
- `desktop-file-validate packaging/flatpak/com.doomdecklauncher.DoomDeckLauncher.desktop`: passed.
- `appstreamcli validate --no-net packaging/flatpak/com.doomdecklauncher.DoomDeckLauncher.metainfo.xml`: passed (one pedantic informational diagnostic).
- `actionlint` on all three changed workflows: installed tool flags only pre-existing `ubuntu-26.04` labels; ignoring that exact unknown-label diagnostic passes. No runner change made for this task.
- `python3 scripts/check-readme-links.py`: **38 links/anchors passed**.
- `cargo fmt --all --check` and `git diff --check`: passed.
- `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test -p ddl-core platform:: -- --test-threads=2`: **25 passed**, including unchanged tarball legacy-import behavior with the new sandbox identity.
- `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test -p ddl-core iwads::visibility:: -- --test-threads=2`: **5 passed**.
- `CARGO_BUILD_JOBS=2 cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 CARGO_TARGET_DIR=/Users/vitor/orca/workspaces/Doom-Deck-Launcher/app-header-brand/target cargo test -p ddl headless::flatpak:: -- --test-threads=2`: **5 passed**, entirely off-screen; reused the existing build cache. No source-wide test rerun or diagnosis of the other worker's known follow-ups.

## Final review follow-up

- Corrected `flatpak build-update-repo --homepage` to `${SITE_BASE}/`; repository metadata,
  AppStream, descriptors and page canonical URL now agree.
- The coordinator confirmed the published obsolete loose ref at
  `repo/refs/heads/app/io.github.vittau.Doom-Deck-Launcher/x86_64/stable`.
  `retire-old-app-ref.py` unlinks **only that exact file** after the new signed export and
  before regenerating/signing AppStream and summary. It is idempotent, rejects symlink
  paths and non-file targets, and does not delete Git refs, objects, other OSTree refs or
  publication history. Existing pruning policy remains unchanged.
- The signed smoke client now lists application IDs explicitly, requires the new ID and
  fails if the obsolete ID remains advertised. GPG verification and fresh-client install
  checks remain enabled. Actual signed publication is still coordinator-owned.
- Four additional tests run the cleanup CLI twice on a bounded OSTree-layout fixture,
  compare all retained files byte-for-byte (including same-ID other branches/architectures,
  similarly prefixed ref, other apps/runtimes, remote refs, objects, summary and Git refs),
  reject symlinks/non-files, check operation order/signing/homepage metadata, and execute
  the real smoke assertion block against new-only, both-ID, old-only and empty listings.
  **Limitation:** neither `ostree` nor `flatpak` is installed on this macOS host; local fixtures
  do not claim to verify signatures or regenerate real AppStream. The Linux signed publisher
  now enforces both application-list assertions before installation.
- Manifest comments now describe the per-startup bundle check and the last `catalog/`
  content commit timestamp; no catalog behavior or generator/release-state fixture changed.
- Follow-up implementation commit: `da772e3`; then merged coordinator checkpoint `8eb2914`
  cleanly. Formatting, all 23 site tests, workflow lint (same existing runner-label exception),
  purge tests and served URL checks pass after integration. No Rust behavior changed in this
  follow-up; the earlier full workspace Clippy and focused Rust checks above remain applicable.

## Exact remaining old-reference audit

Ran `rg -n 'io.github.vittau.Doom-Deck-Launcher|vitormach.dev|vittau.github.io' . --hidden --glob '!.git' --glob '!docs/reports/079-domain-and-flatpak-identity.md'`.
There are **46 matching lines in 18 files**, classified as follows; none is an active old
application export or canonical site setting owned by this worker.

| File(s) | Matching lines | Classification |
| --- | ---: | --- |
| `docs/flatpak-remote.md` | 1 | Necessary upstream DNS CNAME target `vittau.github.io`, not a public website base. |
| `scripts/flatpak/retire-old-app-ref.py` | 1 | Exact obsolete OSTree ref authorized for retirement; never an active export. |
| `.github/workflows/flatpak-remote.yml` | 1 | Negative signed-remote assertion: old application must not be advertised. |
| `scripts/flatpak/test_purge_cloudflare_cache.py` | 3 | Intentional rejection fixtures for both former origins; never allowed purge targets. |
| `docs/decisions.md` | 2 | Historical decisions now explicitly superseded by coordinator-added D-039; retain. |
| `docs/status.md` | 1 | Coordinator-owned earlier release/site evidence; coordinator should refresh status after publication. |
| `docs/journal.yml` | 2 | Historical publication entries; retain. |
| `docs/orchestration-pause-2026-10-21.md` | 1 | Historical pause/handoff snapshot; retain. |
| `docs/reports/058-flatpak-research.md` | 4 | Historical research report; retain. |
| `docs/reports/059-flatpak-sandbox-ready.md` | 2 | Historical sandbox report; retain. |
| `docs/reports/062-flatpak-ci-package.md` | 3 | Historical packaging report and original filenames; retain. |
| `docs/reports/063-flatpak-selfhosted-remote.md` | 8 | Historical live publication/purge evidence; retain. |
| `docs/reports/065-project-download-page.md` | 3 | Historical page implementation evidence; retain. |
| `docs/research/flatpak.md` | 6 | Dated research and original proposal, superseded by spec 079; retain as evidence. Current setup docs use new identity. |
| `docs/specs/059-flatpak-sandbox-ready.md` | 1 | Original accepted scope; retain, superseded for identity by 079. |
| `docs/specs/062-flatpak-ci-package.md` | 1 | Original packaging scope; retain. |
| `docs/specs/063-flatpak-selfhosted-remote.md` | 4 | Original signed-channel scope; retain. |
| `docs/specs/065-project-download-page.md` | 2 | Original page scope; retain. |

The AppStream developer ID `io.github.vittau` is an author identity, preserved deliberately.
GitHub repository, source/screenshot attribution, catalog transport and release/download URLs
are upstream service references, not legacy public website bases; they remain unchanged.
No edits to coordinator-owned overview/decisions/status/journal or the other worker's functions.

## Remaining DNS, Pages and Cloudflare setup

**Confirmed from coordinator checkpoints, not re-queried here:** authoritative nameservers
are `luke.ns.cloudflare.com` and `zara.ns.cloudflare.com`. Initially no public apex A answer
was present and Pages had `cname: null`, publishing from `gh-pages` at `/`. The coordinator
subsequently reports all four apex GitHub A records configured and Pages custom domain set
to `doomdecklauncher.com`; `www` was initially NXDOMAIN and TLS was pending. The latest
coordinator confirmation reports the Pages certificate approved for both apex and www,
`https_enforced: true`, and the owner confirms Pages works. The
coordinator configured the new Cloudflare zone variable; the token is unchanged and the owner
expects it to cover the new zone. This worker did not test credentials or mutate settings.

1. In repository Pages settings keep branch deployment from `gh-pages` / `/`, set custom domain
   `doomdecklauncher.com`, and save. Both generated publications now retain its root CNAME.
2. Configure apex DNS in the new Cloudflare zone with GitHub Pages A records
   `185.199.108.153`, `185.199.109.153`, `185.199.110.153`, `185.199.111.153`, or a supported
   ALIAS/ANAME arrangement. Recommended optional `www` CNAME points to `vittau.github.io`
   (no repository path); the canonical site remains the apex. Use DNS-only during initial
   validation as an operational recommendation, verify DNS in Pages and with `dig`, then enable
   Enforce HTTPS when available. Follow account-specific TXT instructions for domain verification.
3. Coordinator has replaced `CLOUDFLARE_ZONE_ID` for **doomdecklauncher.com**. Confirm that
   `CLOUDFLARE_CACHE_PURGE_TOKEN` authorizes Cache Purge on that zone; the owner expects the
   existing token to cover it. Do not use the old zone ID. Publication must verify API acceptance
   for the exact new prefix; no live purge was attempted by this worker.
4. Publish the signed repository containing the new ID/descriptors before a page-only refresh.
   Page-only deployment intentionally preserves descriptors and cannot transform an old signed
   publication into the new channel; its bootstrap guard defers successfully until that channel exists. Then verify canonical page, stylesheet, descriptor and
   summary bytes, GPG verification and exactly one `doomdecklauncher.com/` prefix purge.
5. **Needs Deck Validation:** install the new descriptor on the physical Deck; verify GL/audio,
   controller input, IWAD visibility, extracted DSDA-Doom, OSK and Steam shortcut. No old-ID
   migration or redirect work is required by the owner.

**Confirmed — primary hosting documentation read for this task:**
[GitHub custom-domain setup](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site)
explains apex records, repository custom domain, branch CNAME and HTTPS availability;
[GitHub CNAME troubleshooting](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/troubleshooting-custom-domains-and-github-pages)
requires one domain-only root CNAME for branch publication and explains how generated pushes can
erase it. These workflows write a branch; they do not use the custom Actions Pages artifact
publishing mode that ignores CNAME. GitHub documents up to 24 hours for DNS propagation and for
Enforce HTTPS to become available. Owner/coordinator owns external completion and publication.

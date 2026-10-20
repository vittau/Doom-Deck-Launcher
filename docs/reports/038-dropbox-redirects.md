# 038 — Dropbox as a fast mirror — worker report

**Status:** Phase 1 (**R1**, **R3**) and phase 2 (**R2**) implemented. A1 verified; A2 verified
(`lint --fetch` for the 12 changed recipes plus an offline lint of the whole catalog). A3 verified
on 2026-10-20 in the shared build slot, on a branch merged with `main` `a7ae1ef`: fmt clean,
workspace clippy clean, workspace tests 581 passed; the Linux e2e is the coordinator's.
No new dependency.

## What changed

### R1 — the narrow Dropbox redirect rule

- `crates/ddl-core/src/downloads/host.rs`
  - New `pub const DROPBOX_DOWNLOAD_SUFFIX: &str = ".dl.dropboxusercontent.com"` and
    `pub const DROPBOX_REDIRECT_ORIGIN: &str = "www.dropbox.com"`.
  - New `pub fn is_dropbox_download_host(host: &str) -> bool`: case-insensitive, true only when
    the host is **exactly one non-empty DNS label** (no dot) followed by the suffix. The bare
    suffix, any deeper label, any `…dl.dropboxusercontent.com.evil.example` lookalike and the
    parent domain are all refused.
  - The module doc names this as the one exception to exact host matching in `host.rs`.
- `crates/ddl-core/src/downloads/fetch.rs`
  - Extends the existing redirect-only mechanism (the `DEFAULT_REDIRECT_EXCEPTIONS` list for
    GitHub's signed asset host) rather than adding a parallel one: `check_target` keeps the exact
    exception list and gains the Dropbox rule beside it, gated on the hop being a redirect.
  - `check_target`'s `is_redirect: bool` became `hop: Hop<'_>` — a tiny private struct with the
    redirecting host (`from`, `None` for a request's first URL) and `dropbox_entry` (whether the
    chain began at an allowlisted `www.dropbox.com` URL). `transfer` tracks both per chain.
  - A hop is permitted when `dropbox_entry && is_dropbox_download_host(host) && (from ==
    "www.dropbox.com" || from == host)`: only a chain that entered at `www.dropbox.com` may land
    on a per-request host, the identity must be exact, and a per-request host may redirect only
    to **itself**. Everything else stays an exact allowlist match.
- `catalog/hosts.toml`
  - `www.dropbox.com` added to `downloads`; the header comment documents the rule (R3). The
    per-request `dl.dropboxusercontent.com` host is deliberately **not** listed — it is code, and
    any one exact label would be stale before use.

#### Amendment to R1 (coordinator, 2026-10-19)

R1 as written describes a *single* redirect. The real chain has two:

```text
$ curl -sI 'https://www.dropbox.com/scl/fi/<id>/DBP78.zip?rlkey=<key>&dl=1'
HTTP/2 302
location: https://uc3b1e38517d45e48a062e295c44.dl.dropboxusercontent.com/cd/0/inline/<token>/file?dl=1#

$ curl -sL '<that target>'
HTTP/2 302
location: /cd/0/inline2/<token>/file?dl=1     ← root-relative, same host
HTTP/2 200
```

Three identical `curl -sI` calls to the same share URL returned **three different** hosts
(`uc3b1e38…`, `uce24a6b2…`, `uc02d3811…`), confirming why the host cannot be allowlisted. A strict
single-hop rule refuses hop 2 (its origin is the per-request host, not `www.dropbox.com`) and no
Dropbox download could ever complete, so I asked the coordinator; the answer was option (a)
**narrowed**: a one-label host may redirect only to the **same** host, only when the chain began
at an allowlisted `www.dropbox.com` URL, with `MAX_REDIRECTS` and the per-hop scheme checks
unchanged. That is what is implemented. The reachable set is still exactly
`{one-label}.dl.dropboxusercontent.com`.

### R2 — Dropbox as the first source for matching DBPs

The 21 recipes pinned to the `dbp-wad-archive` item and not DBP41/60/69 were revisited. The
Dropbox link for each was read from a Wayback snapshot of its release thread (the live board sits
behind a proof-of-work challenge), downloaded to `target/038/files/` (never committed), and
SHA-256 compared with the recipe's pin; nothing was executed.

- **12 match** → `urls = ["https://www.dropbox.com/…?dl=1", "https://ia600903…"]`, so Dropbox is
  tried first and archive.org is the fallback in the recipe format's existing mirror order. Each
  recipe also gained one comment recording the 2026-10-19 hash check. The `st=` signed token some
  links carried was dropped after confirming the link resolves without it (it is short-lived and
  would rot the catalog).
- **9 differ** → recipes unchanged, archive.org only, and listed in
  `docs/research/doomer-boards-projects.md` §Spec 038 phase 2.

### R3 — hosts

`www.dropbox.com` in `downloads`, with the code rule documented in the header comment (R3).

## Verification

### A1 — unit tests, no network

`cargo test -p ddl-core downloads` → **63 passed, 0 failed**. New/updated tests:

| Test | Case |
| --- | --- |
| `host::tests::matches_a_dropbox_per_request_host_with_one_label` | one label, and case-insensitive |
| `host::tests::refuses_dropbox_hosts_without_exactly_one_label` | bare suffix, two labels, empty label, lookalike `…evil.example`, parent domain, empty |
| `fetch::tests::dropbox_chain_allows_the_per_request_host_and_its_same_host_hop` | hop 2 from `www.dropbox.com`, and hop 3 back to the same host |
| `fetch::tests::dropbox_rule_never_applies_to_a_first_url` | per-request host and bare suffix as a first URL |
| `fetch::tests::dropbox_chain_must_start_at_www_dropbox_com` | chain not entering via `www.dropbox.com` |
| `fetch::tests::dropbox_rule_refuses_a_lookalike_or_wrong_origin` | two labels, lookalike suffix, four wrong origins, a different one-label hand-off, arbitrary host from the right origin |
| `fetch::tests::dropbox_rule_is_inert_once_the_host_is_allowlisted_exactly` | exact matching still wins |
| `fetch::tests::rejects_a_dropbox_lookalike_redirect_from_another_origin` | whole `transfer` chain refuses it, no bytes fetched |
| `fetch::tests::redirect_exceptions_apply_only_to_redirects`, `does_not_match_subdomains_or_suffixes` | unchanged in substance, still pass |

### A2 — catalog lint and measured speed

Method: Dropbox speed is wall time of the full `?dl=1` download as `curl` reports it (includes the
redirect and TLS setup per file); archive.org speed is a 30 s timed Range read (`-r 0-8388607`) of
the pinned archive.org copy, so no large archive.org file is downloaded in full. Both were measured
on 2026-10-19 on the dev host, same connection.

| DBP recipe | Dropbox hash matches pin | Dropbox MiB/s | archive.org MiB/s |
| --- | --- | --- | --- |
| avpvtvjvvhvss | **no** | 0.60 | 0.017 |
| chronicles-of-ghost-town | yes | 1.78 | 0.017 |
| forgotten-psychedelics | **no** | 3.68 | 0.017 |
| grief-arbiter | **no** | 0.99 | 0.017 |
| haunting-hollow | yes | 1.08 | 0.017 |
| immortal-warfare | **no** | 1.23 | 0.029 |
| mausoleum-nefarium | yes | 2.41 | 0.017 |
| narcoleptic-strata | yes | 2.27 | 0.033 |
| outposts-of-the-wasteland | **no** | 3.15 | 0.033 |
| patagonian-pilgrimage | yes | 2.18 | 0.017 |
| phantom-processing | yes | 1.09 | 0.033 |
| plutonian-sunrise | yes | 1.19 | 0.022 |
| sanguine-holy-land | yes | 2.73 | 0.020 |
| stagnant-weather | yes | 2.17 | 0.033 |
| tempest-enterprise | yes | 1.80 | 0.033 |
| terminal-voltage | **no** | 1.02 | 0.017 |
| the-economy-of-the-dead | **no** | 1.62 | 0.033 |
| the-vast-silence | yes | 1.92 | 0.019 |
| tomatomania | **no** | 6.30 | 0.033 |
| wretched-morrow | yes | 1.00 | 0.017 |
| zeppelin-armada | **no** | 2.55 | 0.017 |

Totals: Dropbox **201.3 MiB in 86.0 s (2.34 MiB/s)**; archive.org **14.7 MiB in 630 s
(0.023 MiB/s ≈ 1.4 MiB/min)**. Small Dropbox files are dominated by ~2–3 s of setup; the largest
download (53.8 MiB) reached 6.3 MiB/s. For the 12 matching recipes (~87 MiB of packages) Dropbox
takes roughly 40 s versus ~1 h from archive.org.

Why the nine differ (bytes differ in both directions, so they are different builds, not truncated
downloads): DBP32 and DBP70 publish a bare `.wad` on Dropbox while the recipe pins a ZIP; DBP48's
Dropbox file is the 8-map limit-removing build only, not the 156 MB bundle; DBP58/63/68/71/77 are
repacked or different builds; both DBP59 Dropbox files differ from the pin.

`ddl catalog lint catalog/` (whole catalog, offline) → **passed**. `ddl catalog lint --fetch`
was run on each of the 12 changed recipe files individually (not the whole directory, so no other
mapset was re-downloaded); all 12 → **passed (fetched packages verified)** on 2026-10-19.

### A3 — static checks (full workspace, 2026-10-20)

Run in the shared build slot after merging `main` `a7ae1ef` (branch tip `fdfab4c`), in this order
with `set -o pipefail`:

- `cargo fmt --all --check` → clean.
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `cargo test --workspace` → **581 passed, 0 failed** across 12 test binaries (includes the
  408-test `ddl-core` suite, the 140-test `ddl` suite and the edited-recipe catalog lint).

Targeted checks from earlier are unchanged: `cargo test -p ddl-core catalog` → 56 passed (includes
the offline `lint_paths` over the real `catalog/`, so the edited recipes and `hosts.toml` lint).
The Linux e2e is the coordinator's (A3).

## Dependencies

None added.

## Open questions

1. **The nine differing Dropbox files** are newer or differently packaged builds (e.g. DBP48's
   limit-removing-only Zip, DBP32's bare WAD). Re-pinning them would need a fresh size/member
   review and is outside 037/038; left for a follow-up if the owner wants those versions.
2. **Spec 037 R3 wording.** 037 says Dropbox needs `www.dropbox.com` *and its redirect target
   `dl.dropboxusercontent.com`* in `downloads`. 038 (and this implementation) says the redirect
   target is code, not the allowlist, and the bare suffix would never be the real host. Propose
   rewording 037 R3 to match 038.
3. **Spec 038 R1 wording** is being amended by the coordinator to state the narrowed same-host,
   chain-entry rule; the code and the tests above are the record until then.

## Spec items left open

- The Linux e2e (A3, coordinator-run).

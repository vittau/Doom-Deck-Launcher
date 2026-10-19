# 038 — Dropbox as a fast mirror — worker report

**Status:** Phase 1 complete (**R1**, **R3**; A1 verified). **R2 is not started** — it waits for
spec 037 to land in `main` and for the coordinator's go-ahead. No new dependency. No full-workspace
check has run yet (build slot not requested); `ddl-core`-targeted `fmt`/`clippy`/tests are green.

## What changed

### R1 — the narrow Dropbox redirect rule

- `crates/ddl-core/src/downloads/host.rs`
  - New `pub const DROPBOX_DOWNLOAD_SUFFIX: &str = ".dl.dropboxusercontent.com"` and
    `pub const DROPBOX_REDIRECT_ORIGIN: &str = "www.dropbox.com"`.
  - New `pub fn is_dropbox_download_host(host: &str) -> bool`: case-insensitive, true only when
    the host is **exactly one non-empty DNS label** (no dot) followed by the suffix. The bare
    suffix, any deeper label, any `…dl.dropboxusercontent.com.evil.example` lookalike and the
    parent domain are all refused.
  - The module doc now names this as the one exception to exact host matching in `host.rs`.
- `crates/ddl-core/src/downloads/fetch.rs`
  - Extends the existing redirect-only mechanism (the `DEFAULT_REDIRECT_EXCEPTIONS` list for
    GitHub's signed asset host) rather than adding a parallel one: `check_target` keeps the exact
    exception list and gains the Dropbox rule beside it, gated on the hop being a redirect.
  - `check_target`'s `is_redirect: bool` became `hop: Hop<'_>` — a tiny private struct with the
    redirecting host (`from`, `None` for a request's first URL) and `dropbox_entry` (whether the
    chain began at an allowlisted `www.dropbox.com` URL). `transfer` tracks both per chain.
  - A hop is permitted when `dropbox_entry && is_dropbox_download_host(host) && (from ==
    "www.dropbox.com" || from == host)`: only a chain that entered at `www.dropbox.com` may land
    on a per-request host, the identity must be exact, and a per-request host may only hand off to
    **itself**. Everything else stays an exact allowlist match.
- `catalog/hosts.toml`
  - `www.dropbox.com` added to `downloads`; the header comment documents the rule (R3). The
    per-request `dl.dropboxusercontent.com` host is deliberately **not** listed — it is code, and
    any one exact label would be stale before use.

### Amendment to R1 (coordinator, 2026-10-19)

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
**narrowed**: allow a one-label host to redirect only to the **same** host, only when the chain
began at an allowlisted `www.dropbox.com` URL, with `MAX_REDIRECTS` and the per-hop scheme checks
unchanged. That is what is implemented. The reachable set is still exactly
`{one-label}.dl.dropboxusercontent.com`.

## Verification

### A1 — unit tests, no network (§Verification)

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

`does_not_match_subdomains_or_suffixes` keeps passing: there is no suffix matching in
`HostAllowlist`; the Dropbox rule is a separate, redirect-only predicate.

### A3 — lint and static checks (partial, phase 1 scope)

- `cargo fmt --all --check` → clean.
- `cargo clippy -p ddl-core --all-targets -- -D warnings` → clean.
- `cargo test -p ddl-core catalog` → 56 passed (the offline `lint_paths` over the real `catalog/`
  is included, so `hosts.toml` still lints).
- Full `cargo clippy --workspace` / `cargo test --workspace` and the Linux e2e are **open**; the
  build slot will be requested when phase 2 lands.

### Real redirect chain (evidence)

See [Amendment to R1](#amendment-to-r1-coordinator-2026-10-19). Opened 2026-10-19 with `curl -sI`
and `curl -sL` against `https://www.dropbox.com/scl/fi/…/DBP78.zip?rlkey=…&dl=1`.

## Dependencies

None added.

## Open questions

1. **Spec 037 R3 vs spec 038 R1.** 037 says Dropbox needs `www.dropbox.com` *and its redirect
   target `dl.dropboxusercontent.com`* in `downloads`. 038 (and this implementation) says the
   redirect target is code, not the allowlist, and the bare suffix would never be the real host.
   Propose rewording 037 R3 to match 038 when 037's report is folded in. (`docs/decisions.md`
   D-028 is unaffected: it only says Dropbox links are allowed.)
2. **R1 wording.** The coordinator said they will amend spec 038 R1 to state the narrowed
   same-host, chain-entry rule; until then the code is the record.

## Spec items left open

- **R2 (phase 2):** merge `main` after 037, then for each of the 21 Dropbox-only DBPs compare the
  Dropbox `?dl=1` SHA-256 with the pinned archive.org hash, add Dropbox as the first source only on
  a match, and measure both hosts' speed. The report table (DBP · match · Dropbox speed ·
  archive.org speed) will be added then.
- **A2** and the Linux e2e remain for phase 2.

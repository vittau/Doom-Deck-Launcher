# 038 — Dropbox as a fast mirror: allow its per-request download host

**Status:** Ready (after 037)  ·  **Milestone:** M3  ·  **Depends on:** 006, 037, D-028
**Origin:** spec 037 (2026-10-19): `www.dropbox.com/...?dl=1` redirects to a **per-request**
host such as `uc6b74966005c2b22696e15c5150.dl.dropboxusercontent.com` (three identical requests
gave three hosts). The downloader matches hosts exactly (`downloads/host.rs`,
`does_not_match_subdomains_or_suffixes`), so Dropbox recipes can never download. The 21
Dropbox-only Doomer Boards Projects therefore use archive.org copies, which serve at about
2.5 MB/min — a 156 MB mapset takes an hour on the Deck. Owner, 2026-10-19: the downloads from
that host are slow.

## Requirements

- **R1** — **Narrow redirect rule:** a redirect from an allowlisted `www.dropbox.com` URL may land
  on any host whose name is exactly one DNS label followed by `.dl.dropboxusercontent.com`
  (e.g. `ucXXXX.dl.dropboxusercontent.com`), and only as a redirect target of `www.dropbox.com`
  — never as a recipe's first URL, never for any other origin, never deeper subdomains. Keep
  exact matching everywhere else; the existing `does_not_match_subdomains_or_suffixes` test keeps
  passing, and new tests cover the allowed and refused cases (wrong origin, two labels, a lookalike
  suffix such as `dl.dropboxusercontent.com.evil.example`, plain `dl.dropboxusercontent.com` as
  a first URL).
- **R2** — **Mirrors:** for each of the 21 DBPs whose Dropbox link still serves the same file
  (same SHA-256 as the pinned archive.org copy), add the Dropbox `?dl=1` URL as the **first**
  source and keep archive.org as the fallback, using the recipe format's existing mirror order.
  Where the Dropbox file differs (another version), keep archive.org only and note it in
  `docs/research/doomer-boards-projects.md`.
- **R3** — **Hosts:** `catalog/hosts.toml` lists `www.dropbox.com` in `downloads`; the rule of R1 is
  code, documented in `hosts.toml`'s header comment.

## Acceptance criteria

- [ ] **A1** (R1) — downloader unit tests for every allowed/refused case, no network.
- [ ] **A2** (R2) — `ddl catalog lint --fetch` passes for the 21 recipes with Dropbox first; the
  report lists each DBP's outcome and measured download speed from both hosts.
- [ ] **A3** — fmt, clippy `-D warnings`, tests green; the coordinator runs the Linux e2e.

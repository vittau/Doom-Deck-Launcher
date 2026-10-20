# Spec 044 — Weekly catalog link health and a lean e2e install set

## What changed

- **Header probe (`downloads/fetch/probe.rs`):** `Fetcher::probe(url, allowlist)` sends `HEAD`, and
  `GET` with `Range: bytes=0-0` when the host answers `HEAD` with 400/403/405/501 or reports no
  size. It walks the same redirect chain as a download (`check_target`, the GitHub and Dropbox
  exceptions of specs 006/038, at most 5 hops) and never reads a body. Size comes from
  `Content-Length`, the total of `Content-Range` (206), or `bytes */N` (416, empty file). Added
  `Fetcher::headers_only`/`for_probes`, which build a fetcher without creating or cleaning the user's
  download cache, and a 30 s whole-request timeout on top of the 15 s connect/header ones.
  `test_http.rs`: a `HEAD` answer no longer writes the body.
- **Link checks (`catalog/links.rs`, R1–R2):** `LinkChecker::check` probes every mirror of every
  package. Outcomes `ok`, `size-changed`, `gone` (404/410), `host-refused`, `error`. Bounds: 4 probes
  in flight, one per `host:port`. Each distinct URL is probed once and judged against every
  package's own pinned size. Transient failures (5xx, 429, timeouts, connection errors) are retried
  3 times (1 s, 2 s, 4 s) — see "Findings". `Report` renders a Markdown table or JSON; a package with
  at least one `ok` mirror is healthy (a warning if some are bad), a package with none is failed.
- **CLI:** `ddl catalog check-links [DIR] [--json] [--problems-only]` (exit non-zero when a recipe has
  no good source) and `ddl catalog e2e-sample [DIR] [--max-mib N]`. `catalog::load_catalog_dir`
  loads a directory with the embedded catalog's validation.
- **E2E sample (`catalog/sample.rs`, R4):** smallest recipe (by total download) per first-source host,
  plus every recipe with free optional components, each component once; total download ≤ 64 MiB
  unless no recipe for the host fits. Output is `recipe<TAB>component<TAB>reason`, with `-` for a
  plain install (a tab-`IFS` shell `read` would swallow an empty field).
  Over the real catalog: **19 installs of 16 recipes** (was 89): luminous-gloom, monuments-of-mars,
  sigil-ii, terminal-voltage, wretched-morrow for hosts; alien-vendetta, ancient-aliens,
  auger-zenith, back-to-saturn-x-e1/e2, eviternity, going-down, mausoleum-nefarium, scythe, sigil,
  sigil-ii, valiant for components.
- **`.github/workflows/catalog-health.yml` (R3):** weekly (Mon 05:17 UTC) + `workflow_dispatch`;
  builds `ddl` like the `checks` job, runs `check-links`, writes the table to the job summary, opens
  or edits the single `catalog-health` issue on failure and closes it on a clean run. Permissions
  `contents: read`, `issues: write`. The job also ends red on failure.
- **`.github/workflows/e2e.yml`:** step 2 now installs the sample (with the reasoning as a comment
  and a summary table of the chosen ids); the dry-run loops cover the sampled recipes.

## Acceptance criteria

- **A1:** `catalog::links::tests` (25), local test server, no network: every outcome, HEAD refused
  (405/501/403) → range fallback with the `Range: bytes=0-0` request asserted, 200 to a range, 416
  empty file, redirect out of the allowlist, redirect loop, timeout, one dead + one good mirror
  (warning), no good source (failure), failing second package, result order, ≤ 4 in flight across
  hosts, 1 per host, retries, URL dedup, Markdown/JSON. Run 6 times in a row, stable.
- **A4:** `catalog::sample::tests` (10) incl. the real `catalog/`: every first-source host covered,
  every free component and every status-bar component in the sample, cap holds, stable under input
  order, lean. **Not yet done:** the post-merge e2e dispatch and its < 30 min (coordinator).
- **A2:** `actionlint` passes on all three workflows. **Not yet done:** the `workflow_dispatch` run.
- **A3:** `cargo fmt --all --check` ok; `cargo clippy --workspace --all-targets -- -D warnings` ok;
  `cargo test --workspace --no-fail-fast` all green (ddl 144, ddl-core 472, …) on the final tree.
  The first full run had one failure, `cli_exec::tests::launch_profile_selects_isolated_paths…`:
  that existing test installs `sigil` from the real network and got HTTP 503 from
  raw.githubusercontent.com (see below). It passed on the rerun; it is unrelated to this change.
- Manual, headers only: `ddl catalog check-links` against the real catalog — 222 URLs, 222 ok (all
  by `HEAD`, all sizes equal the pins), 25 s.

## Findings

- raw.githubusercontent.com answers a bare 503 (`retry-after: 0`) to roughly every other request
  (curl, 8 tries: 4 of 8 failed). The first manual run reported 9 failed recipes because of it,
  hence the retries and the URL dedup. The same flake hit the existing `cli_exec` test above and can
  hit the e2e's sigil/status-bar downloads.

## Dependencies

None added.

## Open questions / spec items left open

- Interpretation: "no `ok` source left" is judged per package (a recipe with two packages fails if
  either has no good mirror). Say if you want it per recipe.
- A host that answers 2xx but reports no size counts as `ok` ("size not reported" in the detail),
  since there is nothing to compare. This path has no test: the test server always sends
  `Content-Length`.
- `check-links` also exits non-zero when a recipe file fails to load (it would go unchecked).
- The sample's cap is on a recipe's total download, not on each package.
- Suggested for the coordinator (not edited here): note the new workflow and subcommands in
  `docs/status.md` and `docs/journal.yml`.

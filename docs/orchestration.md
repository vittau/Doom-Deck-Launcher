# Orchestration playbook

How DDL is built: one **orchestrator** agent coordinates **worker** agents through Orca ADE.
Read this before taking over as orchestrator. Workers only need `AGENTS.md` and their brief.

## Roles

- **Owner** (human): sets direction, tests release builds on a physical Steam Deck LCD. Wants an
  autonomous end-to-end flow: escalate only questions that change the project's direction
  (scope, platform or engine, legal/licensing risk, conflict with an explicit owner request).
  Decide everything else, record it (`docs/decisions.md`, `docs/journal.yml`), and report it in
  the next status update. Talk to the owner in Brazilian Portuguese; repository text stays English.
  State decisions in plain language (spec/decision ids after the statement, never instead of it).
  - **Asking for input:** play `afplay /System/Library/Sounds/Ping.aiff`, mark the ask in the reply
    with `> 🔴 **PRECISO DE VOCÊ** — <one-line ask>` (where things stand, the ask with a
    recommendation, what proceeds without them), and — when the ask arises during autonomous work
    while the owner may be away — send it on Telegram through the Make MCP connector: scenario
    "Integration Telegram Bot", id `6660765` (team 3207272, zone us2), `scenarios_run` with
    `responsive: true` and data `{question, context, options, request_id}` (Portuguese). Skip
    Telegram when answering a question the owner just asked in chat. Never use it for routine status.
  - When the owner says they are away, keep working autonomously and use Telegram only for real asks.
- **Orchestrator** (Claude Opus 5.5, effort high, running in an Orca terminal): turns research
  into specs, writes task briefs, dispatches and monitors workers, reviews and merges their
  branches, pushes, tags pre-releases, and owns `docs/status.md`, `docs/journal.yml`,
  `docs/decisions.md` and `OVERVIEW.md`. It does not do bulk implementation itself.
- **Workers**: implement one task each on their own branch and report with `worker_done`.

## Worker routing

**Current owner override (2026-10-21, supersedes OpenCode-only routing): prioritize new workers
on GPT-6.1 Sol, effort medium, for today.** Keep existing OpenCode conversations for their current
tasks and review corrections; do not replace them solely because routing changed. OpenCode remains
the fallback when needed. After today, use the default routing table below unless the owner updates
this preference.

| Work | Agent | Launch |
| --- | --- | --- |
| Default development and research, bounded | opencode, **DeepSeek V4.1 Flash Free** (`opencode/deepseek-v4.1-flash-free`), effort high — 200k context | see *Launching opencode* |
| Default work that is long or wide, or when Free credits/endpoint fail | opencode, DeepSeek V4.1 Flash (`opencode-go/deepseek-v4.1-flash`, 1M context), effort high | see *Launching opencode* |
| Wide-context work; **all UI design and implementation** (it generates raster mockups) | Codex, GPT-6.1-Sol, effort medium | `worker-start … --agent codex --model gpt-6.1-sol --effort medium` |
| Fallback when Codex's budget is out | opencode (Flash, paid) — **not Claude** (see below) | see *Launching opencode* |

**Claude workers are paused** (owner, 2026-10-20: "Não use mais Sonnet 5.5 por enquanto, tokens
Claude próximos de acabar. Use apenas OpenCode como fallback"). Do not start Claude Code workers
until the owner lifts this; the orchestrator itself also runs on the owner's Claude quota, so keep
coordinator turns lean (no idle polling loops, short reports). Before the pause the fallback was
Claude Code Sonnet 5.5, effort high.

Switch rules: Codex above ~85 % of its 5-hour window or ~90 % weekly → route new work to opencode
until it resets (two Codex workers can burn a fresh 5-hour window in about 35 minutes). A worker
that hits a usage limit mid-task → snapshot its tree, interrupt it, close its terminal, and start a
replacement on the **same Task and worktree**, then type a continuation note into the new terminal
(what is committed, what is uncommitted, decisions already given). The owner may override routing
for a day (2026-10-17: "Codex first"); such overrides end with the day. A worker above ~80 %
context → its next task goes to a fresh worker. Read Codex usage from the health monitor line, the
`rate_limits` field in `~/.codex/sessions/**/*.jsonl`, or the Codex footer.

**No context rot:** every new task gets a **fresh agent**. Only post-review fixes to the *same*
task may reuse that task's terminal. After settlement: release the worker, close its terminal.

## Task lifecycle

1. **Spec first.** The contract is `docs/specs/NNN-*.md` (or, for research, the brief itself).
2. **Brief** — self-contained: Target, Change, Constraints, Ownership (files the worker may
   touch; it never edits the orchestrator-owned docs), Observable acceptance. Point to
   `AGENTS.md` and the spec rather than restating them. Always include: "never open visible
   windows or capture the screen — headless renders only"; which other workers are editing
   which files, and "merge `main` before your final checks"; iterate with targeted tests, full
   checks once at the end; report at `docs/reports/NNN-*.md`; heartbeat per milestone. Briefs
   live in the orchestrator's session scratchpad and are **lost between sessions** — the spec is
   the durable contract; rebuild a brief from it.
3. **Placement** — one Orca worktree per task, top-level from `main`
   (`--no-parent --base-branch main`); the branch is `vittau/<task-name>`.
4. **Launch** — one Orca Run holds all tasks (reuse it across sessions with `run-use`) (`run-create` once; pass
   `--run <id>` to every `worker-start`). Confirm on screen that the brief landed and a turn
   started, and read the model and effort from the TUI footer.
5. **Monitor** (three Claude Code Monitors, re-armed every 30 minutes):
   - inbox: `orca orchestration check --wait --types "worker_done,escalation,question" --run <id> --json`
     (ack each delivery with `--ack <delivery_id>`, then re-arm);
   - health every 5 minutes: `scripts/orchestration/worker_health.py NAME=HANDLE …` — screen
     state, stalls, prompts, context and token usage.
   - GUI guard: `scripts/orchestration/gui_guard.sh` kills any worker-launched `ddl` window or
     `screencapture` (the owner works on this machine; see AGENTS.md).
   Heartbeats need no action.
6. **Review** the branch: scope limited to its ownership, tests and lint pass, claims cited
   (spot-check one or two against the source), acceptance criteria met, headless PNGs looked at.
   Send fixes back to the **same** terminal *before* releasing: `worker-release` of a
   worker-started Codex/Claude dispatch closes its terminal (`closed_agent_terminal`).
7. **Merge** with `git merge --no-ff`; resolve test-list conflicts (embedded recipe
   ids, item counts) as the union; then run `cargo fmt --all --check`, `cargo clippy --workspace
   --all-targets -- -D warnings` and `cargo test --workspace` on the merge result before pushing.
   Worktrees share the local `main` ref, so a worker that merges `main` picks up an unverified
   merge: while workers are running, merge on a scratch branch (`git switch -c verify/<task>`),
   check there, then fast-forward `main` (`git switch main && git merge --ff-only verify/<task>`). After
   catalog or download changes also run the Linux e2e: `gh workflow run e2e.yml --ref main`.
   Then `worker-release`, close the terminal, `worktree rm`, and update `docs/status.md` and
   `docs/journal.yml`. Keep merged task branches on the remote unless the owner asks
   (deleting remote refs needs the owner's OK).
8. **Pause / resume.** To stop a worker mid-task: snapshot its tree (`git stash create` →
   `git update-ref refs/keep/<name>`), have it commit a WIP and a handoff section in its report,
   push the branch, release it. To resume: a new worktree with `--base-branch <wip branch>` and a
   brief that starts with "read the handoff in the report".
9. **Release for the Deck** when a build is worth testing: push a `v0.0.N-<label>` tag (release
   workflow, spec 001) and add the checks to `docs/deck-testing.md`. After tagging, a worker adds
   the tag's saved-state fixture tree (`tests/fixtures/state/generate.py <tag>`, spec 055) and
   its name to the release guard test, so the next release must still read it.

## Machine load

All workers share the owner's Mac. Cold Rust/Slint builds are heavy: five in parallel pushed the
load average to ~170. Keep at most **four** code workers at once, and tell workers to iterate
with `cargo test -p ddl-core` / `cargo clippy -p ddl-core` (no Slint) and run the full workspace
checks once before `worker_done`.

## Launching opencode

`worker-start --agent opencode` can lose the first prompt, and the opencode TUI takes no
`--model` flag. Instead:

```sh
orca worktree create --repo id:<repo-id> --name <task> --no-parent --base-branch main --json
orca terminal create --worktree path:<worktree> --title opencode-<task> \
  --command "env OPENCODE_CONFIG_CONTENT='{\"model\":\"opencode-go/deepseek-v4.1-flash\"}' opencode --standalone" --json
orca terminal wait --terminal <handle> --for tui-idle --timeout-ms 90000 --json   # footer: "DeepSeek V4.1 Flash … · high"
orca orchestration worker-start --run <run> --spec "$(cat brief.md)" --task-title "<title>" \
  --terminal <handle> --worktree path:<worktree> --json
```

Use `opencode/deepseek-v4.1-flash-free` in place of `opencode-go/deepseek-v4.1-flash` for bounded
tasks (footer: "DeepSeek V4.1 Flash Free OpenCode · high"). The Free endpoint sometimes answers
"Upstream request failed: Endpoint is unavailable" and retries by itself; if it keeps failing,
relaunch on the paid model. Close the fallback shell that a bare `worktree create` opens, after
confirming it is idle. When an opencode task is settled: `worker-release` (it stays `retained`),
`opencode api session.list` → `opencode api session.interrupt --param sessionID=<id>` for its
session, then close the terminal.

## Known stalls

- A worker mid-turn may not read an `orchestration send` until its next checkpoint. For urgent
  corrections, type the instruction straight into its terminal (`terminal send --text … --enter`;
  opencode queues it).

- opencode **"Allow OpenCode to search the web?"** prompt: send Enter (option 1, allow).
- opencode ends a turn without `worker_done`: a plain `terminal send --text "continue" --enter`.
- Codex **"Update available"** menu fails `worker-start` at `agent_readiness`, but the terminal
  survives. Send `3` ("Skip until next version"), wait for `› Ask Codex`, then retry on that
  terminal with `worker-start --task <id> --retry-of <failed dispatch> --terminal <handle>
  --worktree path:<wt>`. Tell the owner an update is pending.
- An Orca or Codex restart can orphan workers; recover per the Orca `orchestration` skill.
- **Closing an opencode terminal does not stop its turn**, even with `--standalone`: the session
  keeps running headless in the shared `opencode serve` and keeps editing the worktree. Before
  handing that worktree to another agent: `opencode api session.list` (match
  `location.directory`), `opencode api session.interrupt --param sessionID=<id>`, and confirm its
  tool processes are gone. Never kill the shared service.
- opencode transient provider errors (`Upstream request failed … rank … not connected`, HTTP
  4xx/5xx) end the turn: nudge with `terminal send --text "continue …" --enter`. If opencode
  itself needs reconnecting the owner does it; then check the screen for the brief.
- Codex `worker-start` failing `agent_readiness` / `terminal_handle_stale` while Codex sits idle:
  `worker-release` that dispatch, then `worker-start --task <id> --retry-of <failed dispatch>
  --worktree path:<wt> --agent codex …` (a `failed` Task needs `--retry-of`; a `ready` one must
  not have it).
- The orchestrator resumed in a **new terminal** is fenced from its Run: `orca orchestration
  run-use --id <run>` first.
- `worker_health.py` decides "working" from the last 8 screen lines; Claude Code's spinner sits
  above a tip line. A false `IDLE` → read the screen before acting.
- doomworld.com and doomwiki.org return Cloudflare 403 to automated fetches; workers fall back
  to search snippets. Label such evidence secondary and spot-check it (the 2014 Cacoward of
  BTSX E2 was missed that way).

## Push and release policy

The orchestrator may push `main` and task branches and create pre-release tags without asking.
Ask the owner first before a force-push, before deleting remote refs, and before any
non-prerelease release.

## Product direction the orchestrator must keep

These come from the owner and are recorded in `OVERVIEW.md` / `docs/decisions.md`; the points
below are the ones most easily lost between sessions.

- **Platform order:** everything working and fully validated on the Steam Deck first; then
  Windows; macOS last (owner, 2026-10-19). Do not start port work or port research before the Deck
  validation is done, but keep new code portable (OS specifics behind `cfg`/traits).
- **Distribution:** GitHub pre-releases and the experimental signed self-hosted Flatpak remote
  at `https://www.vitormach.dev/Doom-Deck-Launcher/` (D-035, owner confirmed 2026-10-21).
  The owner explicitly authorized agent-written packaging for this channel (specs 062–063).
  Flathub is a later, policy-bound option: do not submit these AI-assisted manifests or automate
  submission/review interactions. Spec 065 will replace the provisional page with a launcher-themed
  page and a prominent signed `.flatpakref` install link.
- **UI:** unmistakably DOOM, tasteful and detailed (D-020), better than UZDeck; UI work goes to
  Codex, and large visual changes are shown to the owner as headless renders before merging.
  Amber means focus only; only actions that launch a game are red (D-036/spec 064 supersedes
  spec 060's primary-action rule). Base-game menu/title art is an explicit owner-authorized
  exception to D-020; retain source/credit. The owner's hardware-renderer default is D-037.
- **Catalog:** community-contributable TOML recipes via PR (D-010); AUGER;ZENITH (with add-on),
  SIGIL and SIGIL II are reference mapsets.
- **Tracking** lives only in `docs/` (no Linear, no GitHub Issues except the automated
  `catalog-health` / `idgames-crawl` issues opened by workflows).

## Operational lessons (Orca 1.4.232, Codex 0.162, opencode 2.0.x)

- `orca worktree rm` **deletes the local task branch**: push the branch (or merge it) first.
- A task worktree may contain nested `git worktree`s a worker made (e.g. tag checkouts under
  `.scratch/` or its scratchpad): `git worktree remove --force` them, then `orca worktree rm`;
  finish with `git worktree prune`.
- Closing a worker-started terminal whose Dispatch is still running turns the Dispatch `failed`
  and the Task back to `ready`: restart it with `worker-start --task <id> --worktree path:<wt>`
  **without** `--retry-of`. `worker-start --terminal <h>` with `--spec` on a settled opencode
  terminal is how to dispatch a post-review fix to the same agent.
- Orca self-updates mid-run; afterwards a worker's heartbeat can be rejected ("not the exact
  process owning the dispatch") — later messages usually work again; keep the worker going.
- The typed inbox wait also returns heartbeats on this Orca version: read every message's `type`,
  ack the batch, re-arm.
- Running workers' typed inputs land mid-turn: for urgent corrections use
  `orca terminal send --text … --enter` on the worker's terminal, plus an `orchestration send` to
  `dispatch:<id>` for the record.
- `git ls-remote --heads origin <name>` matches by suffix (it found `vittau/idgames-index` for
  `idgames-index`); always use `refs/heads/<name>`.
- Never start a background `cargo test` on a tree with unresolved merge conflicts; resolve, commit,
  then verify.

## Open follow-ups (keep this list current)

- Add saved-state fixture trees for `v0.0.8-upgrade` and `v0.0.9-focus` to
  `tests/fixtures/state/` and the release guard test (spec 055 R4; small opencode task).
- Owner: Deck checks V1–V6 and U1–U9 in `docs/deck-testing.md`; install/test the signed Flatpak
  from D-035's remote after CI stabilization, including spec 059's Needs Deck Validation items.
- Current integration 2026-10-21: 067 Audio/soundfonts and 068 verified SIGIL editions,
  including COMPAT + SHREDS, are reviewed and merged after combined fmt/clippy and 870 passing tests.
  Specs 062–066 and independent website deployment are complete; the old shutdown pause record
  is historical. See `docs/status.md` for the final integration and CI results.
- Next backlog: specs 051–053 (/idgames browse on top of the published index); the
  "Advanced" card subtitle touching its bottom edge (cosmetic, pre-existing).

## Where state lives

`docs/status.md` (now), `docs/journal.yml` (history), `docs/decisions.md` (decisions in force),
`docs/specs/` (contracts), `docs/research/` (evidence), `docs/reports/` (worker reports),
`docs/deck-testing.md` (owner's Deck checklist and results). An orchestrator taking over starts by
reading these and this file, then `orca orchestration run-use --id <run>` (needed whenever the
coordinator runs in a new terminal) and `orca orchestration worker-list --run <id> --json` for live
workers, and `orca worktree list --repo id:<repo-id> --json` for leftover worktrees.

Current identifiers: repo id `7bde2174-d352-4dd8-9057-f2b433ecbbf2`; Run `run_005c6aad7366`
(holds every task since 2026-10-16). Monitors (inbox wait, `worker_health.py`, `gui_guard.sh`)
are Claude Code Monitors that expire every 30 minutes and must be re-armed while workers run;
stop them when no worker is active. Published branches written by CI: `catalog` (catalog bundle,
spec 019) and `idgames-index` (daily /idgames index, spec 050; first publish 2026-10-20, 10,885
entries).

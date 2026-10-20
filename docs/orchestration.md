# Orchestration playbook

How DDL is built: one **orchestrator** agent coordinates **worker** agents through Orca ADE.
Read this before taking over as orchestrator. Workers only need `AGENTS.md` and their brief.

## Roles

- **Owner** (human): sets direction, tests release builds on a physical Steam Deck LCD. Wants an
  autonomous end-to-end flow: escalate only questions that change the project's direction
  (scope, platform or engine, legal/licensing risk, conflict with an explicit owner request).
  Decide everything else, record it, and report it in the next status update. When the owner's
  input is needed: play `afplay /System/Library/Sounds/Ping.aiff`, mark the ask in the reply with
  `> 🔴 **PRECISO DE VOCÊ** — <one-line ask>`, and notify them on Telegram through the Make MCP
  scenario "Integration Telegram Bot" (inputs `question`, `context`, `options`, `request_id`).
  Talk to the owner in Brazilian Portuguese; repository text stays English.
- **Orchestrator** (Claude Opus 5.5, effort high, running in an Orca terminal): turns research
  into specs, writes task briefs, dispatches and monitors workers, reviews and merges their
  branches, pushes, tags pre-releases, and owns `docs/status.md`, `docs/journal.yml`,
  `docs/decisions.md` and `OVERVIEW.md`. It does not do bulk implementation itself.
- **Workers**: implement one task each on their own branch and report with `worker_done`.

## Worker routing

| Work | Agent | Launch |
| --- | --- | --- |
| Default development and research | opencode, DeepSeek V4.1 Flash, effort high | see *Launching opencode* |
| Wide-context work; **all UI design and implementation** (it generates raster mockups) | Codex, GPT-6.1-Sol, effort medium | `worker-start … --agent codex --model gpt-6.1-sol --effort medium` |
| Fallback when Codex's budget runs low | Claude Code, Sonnet 5.5, effort high | `worker-start … --agent claude --model claude-sonnet-5-5 --effort high` |

Switch rules: Codex above ~85 % of its 5-hour window or ~90 % weekly → route by the default
table (opencode) until it resets; Sonnet takes Codex-only work (UI) that cannot wait. The owner
may override routing for a day (2026-10-17: "Codex first"); such overrides end with the day.
A worker above ~80 % context → its next task goes to a fresh worker. An opencode usage limit →
replace the worker on the same worktree. Read Codex usage from the health monitor line or the
`⚠ 5h limit: N% left · resets at HH:MM` hint in a Codex footer.

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

Close the fallback shell that a bare `worktree create` opens, after confirming it is idle.

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

## Where state lives

`docs/status.md` (now), `docs/journal.yml` (history), `docs/decisions.md` (decisions in force),
`docs/specs/` (contracts), `docs/research/` (evidence), `docs/reports/` (worker reports). An
orchestrator taking over starts by reading these, then `orca orchestration run-use --id <run>`
and `orca orchestration worker-list --run <id> --json` for any live workers, and
`orca worktree list --repo id:<repo-id> --json` for leftover worktrees.

Current identifiers: repo id `7bde2174-d352-4dd8-9057-f2b433ecbbf2`; Run `run_005c6aad7366`
(holds every task since 2026-10-16). The orchestrator's long-lived monitors (inbox, health, GUI
guard) are Claude Code Monitors that expire every 30 minutes and must be re-armed.

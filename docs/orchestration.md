# Orchestration playbook

How DDL is built: one **orchestrator** agent coordinates **worker** agents through Orca ADE.
Read this before taking over as orchestrator. Workers only need `AGENTS.md` and their brief.

## Roles

- **Owner** (human): sets direction, tests release builds on a physical Steam Deck LCD. Wants an
  autonomous end-to-end flow: escalate only questions that change the project's direction
  (scope, platform or engine, legal/licensing risk, conflict with an explicit owner request).
  Decide everything else, record it, and report it in the next status update.
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

Switch rules: Codex above ~85 % of its 5-hour window or ~90 % weekly → new Codex tasks go to
Sonnet. A worker above ~80 % context → its next task goes to a fresh worker. An opencode usage
limit → replace the worker on the same worktree.

**No context rot:** every new task gets a **fresh agent**. Only post-review fixes to the *same*
task may reuse that task's terminal. After settlement: release the worker, close its terminal.

## Task lifecycle

1. **Spec first.** The contract is `docs/specs/NNN-*.md` (or, for research, the brief itself).
2. **Brief** — self-contained: Target, Change, Constraints, Ownership (files the worker may
   touch; it never edits the orchestrator-owned docs), Observable acceptance. Point to
   `AGENTS.md` and the spec rather than restating them.
3. **Placement** — one Orca worktree per task, top-level from `main`
   (`--no-parent --base-branch main`); the branch is `vittau/<task-name>`.
4. **Launch** — one Orca Run per orchestrator session holds all tasks (`run-create` once; pass
   `--run <id>` to every `worker-start`). Confirm on screen that the brief landed and a turn
   started, and read the model and effort from the TUI footer.
5. **Monitor** (two Claude Code Monitors):
   - inbox: `orca orchestration check --wait --types "worker_done,escalation,question" --run <id> --json`
     (ack each delivery with `--ack <delivery_id>`, then re-arm);
   - health every 5 minutes: `scripts/orchestration/worker_health.py NAME=HANDLE …` — screen
     state, stalls, prompts, context and token usage.
   Heartbeats need no action.
6. **Review** the branch: scope limited to its ownership, tests and lint pass, claims cited,
   acceptance criteria met. Send fixes back to the same terminal if needed.
7. **Merge** with `git merge --no-ff` into `main`, push, `worker-release`, close the terminal,
   remove the worktree once merged. Update `docs/status.md` and `docs/journal.yml`.
8. **Release for the Deck** when a build is worth testing: push a `v0.0.N-<label>` tag (release
   workflow, spec 001) and add the checks to `docs/deck-testing.md`.

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

- opencode **"Allow OpenCode to search the web?"** prompt: send Enter (option 1, allow).
- opencode ends a turn without `worker_done`: a plain `terminal send --text "continue" --enter`.
- Codex **"Update available"** menu blocks `worker-start` at agent readiness; update Codex, then
  retry the same task.
- An Orca or Codex restart can orphan workers; recover per the Orca `orchestration` skill.

## Push and release policy

The orchestrator may push `main` and task branches and create pre-release tags without asking.
Ask the owner first before a force-push, before deleting remote refs, and before any
non-prerelease release.

## Where state lives

`docs/status.md` (now), `docs/journal.yml` (history), `docs/decisions.md` (decisions in force),
`docs/specs/` (contracts), `docs/research/` (evidence). An orchestrator taking over starts by
reading these, then `orca orchestration worker-list --run <id> --json` for any live workers.

# DDL documentation

Spec-driven, deliberately small. Each file holds **current** truth; history lives in `journal.yml`.

| Path | Contents |
| --- | --- |
| `decisions.md` | Decisions in force (`D-NNN`). Replace an entry when it changes; log the change in the journal. |
| `status.md` | Where each milestone and spec stands, and what waits on Deck testing. |
| `journal.yml` | Append-only log of decisions, findings and notable changes. |
| `specs/NNN-name.md` | One spec per deliverable: requirements, acceptance criteria, tasks. Start from `specs/_template.md`. |
| `research/*.md` | Evidence gathered from upstream source, docs and APIs, each claim cited and labelled. |
| `orchestration.md` | How the orchestrator and worker agents run the project (Orca ADE). |
| `deck-testing.md` | Manual test checklist for the physical Steam Deck (added with the first release build). |
| `outreach.md` | Outreach plan: channel order and rules, post drafts, and a log of what was posted where. |

## Flow

1. **Research** what is uncertain → `research/` (claims labelled Confirmed / Proposed / Needs Deck Validation).
2. **Specify** → `specs/NNN-*.md` with numbered requirements (`R1`, `R2`…) and observable acceptance criteria.
3. **Implement** against the spec on a task branch, with tests per requirement.
4. **Verify** → tick the spec's acceptance criteria; anything needing hardware goes to `deck-testing.md`.
5. **Record** → update `status.md`, append to `journal.yml`, adjust `decisions.md` if a decision changed.

`OVERVIEW.md` at the repository root stays the product north star; `decisions.md` overrides it where they differ.

# Contributing / Agent Workflow

## Before work

Read, in order:

1. `AGENTS.md`
2. `project/CURRENT_STATE.md`
3. `project/HANDOFF.md`
4. `project/TASKS.md`
5. relevant architecture/security docs under `docs/`
6. relevant ADRs under `docs/decisions/`

Do not rely on previous conversation history.

## Select a task

Choose the highest-priority READY task whose dependencies are DONE.

A task is READY only if:

1. its dependencies are DONE;
2. it has sufficient acceptance criteria;
3. it does not require an unresolved architectural decision.

If no suitable READY task exists, explain the blocker.

## During work

- Keep changes scoped to the selected task.
- If unrelated work is discovered, create or reference another task rather than silently expanding scope.
- Update `project/HANDOFF.md` when meaningful milestones occur.

## Before completion

Run all relevant checks:

- unit tests;
- integration tests;
- type checks;
- lint;
- formatting;
- project/security checks when available.

Review the Git diff. Never commit secrets, keys, or real user data.

## Documentation updates

Update before finishing:

- task status in `project/TASKS.md`;
- `project/CURRENT_STATE.md`;
- `project/HANDOFF.md`;
- ADRs when architectural decisions are made;
- `project/SECURITY_LOG.md` when security-relevant changes are made.

## Git workflow

See `AGENTS.md` section 19 for full details.

Summary:

- One branch per task: `task/TASK-###`
- Commit with task-based messages: `TASK-###: concise description`
- Create a PR targeting `main` before merging
- Full commit history preserved (no squash/rebase)
- Self-merge allowed with a note if human doesn't review

## Handoff

A new agent must be able to continue without conversation history.

`project/HANDOFF.md` is the handoff document. It must include:

- agent/session identifier;
- task worked on;
- work completed;
- what works and what does not;
- tests run;
- files changed;
- important discoveries;
- decisions made;
- known risks;
- blockers;
- exact next recommended task;
- exact instructions for the next agent.

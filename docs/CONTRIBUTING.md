# Contributing / Agent Workflow

## Before work

Read:

- `AGENTS.md`
- `project/CURRENT_STATE.md`
- `project/HANDOFF.md`
- `project/TASKS.md`
- relevant architecture/security docs
- relevant ADRs

## Select a task

Choose the highest-priority READY task whose dependencies are DONE.

## During work

Keep changes scoped.

If unrelated work is discovered, create or reference another task rather than silently expanding scope.

## Before completion

Run:

- relevant unit tests;
- relevant integration tests;
- type checks;
- lint;
- formatting;
- project/security checks when available.

Review the Git diff.

## Documentation

Update:

- task status;
- `CURRENT_STATE.md`;
- `HANDOFF.md`;
- ADRs when needed;
- `SECURITY_LOG.md` when security-relevant.

## Commit

Use:

`TASK-###: concise description`

## Handoff

A new agent must be able to continue without conversation history.

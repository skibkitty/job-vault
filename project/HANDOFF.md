# Agent Handoff

## Current session

Agent:
opencode/big-pickle

Date:
2026-08-18

Task:
TASK-024

## What has been done

- Implemented local job search functionality in SqliteStorage.
- Added search_jobs method with case-insensitive search.
- Added search by title, company_id, and location.
- Added 5 comprehensive tests.
- All 92 tests pass.

## What works

- Search jobs by title, company_id, or location
- Case-insensitive search
- Search returns matching jobs ordered by created_at DESC
- Empty query returns error
- No results returns empty vec

## What does not work

Nothing broken.

## Tests run

- `cargo test` — 92 passed

## Files changed

- companion/src/database/sqlite.rs
- project/TASKS.md
- project/CURRENT_STATE.md
- project/HANDOFF.md

## Important discoveries

- SQLite LIKE is case-insensitive by default for ASCII
- Search uses %query% pattern for partial matching
- Search is performed on title, company_id, and location fields

## Decisions

- Search is case-insensitive (user-friendly)
- Search uses LIKE with % wildcards for partial matching
- Search returns results ordered by created_at DESC (newest first)
- Empty query is rejected with error

## Known risks

None.

## Next recommended action

TASK-030 — Generic extraction framework.

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. Implement TASK-030 — Generic extraction framework.

## Blockers

None.

## Next recommended action

TASK-022 — Job CRUD.

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. Implement TASK-022 — Job CRUD.

## Blockers

None.

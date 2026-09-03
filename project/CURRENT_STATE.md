# Current State

## Project

Job Vault

## Status

IN_PROGRESS

## Current phase

Phase 6 — UI (extension-side); Phase 5 diff engine unblocked via WSL

## Current task

TASK-062 (DONE)

## Recommended next action

Rust tasks are unblocked. Highest-priority READY Rust task:
TASK-051 — Paragraph/sentence diff (P0, depends on TASK-050 DONE).
Then TASK-052 (bullet diff), then TASK-042 (similarity matching).
Phase 6 UI tasks (TASK-063+) remain blocked on Rust milestones (TASK-055).

## Completed

- Product concept defined.
- Local-first privacy model defined.
- Chrome extension + local companion architecture selected.
- LinkedIn, Indeed, and generic company career pages selected as initial sources.
- Encrypted vault requirement established.
- Multi-agent handoff model established.
- Repository initialized with directory structure (TASK-001).
- Agent/project documentation validated (TASK-002).
- Architecture documentation validated (TASK-003).
- Threat model validated (TASK-004).
- Security specification validated (TASK-005).
- Permission policy created (TASK-006).
- Chrome extension scaffolded (TASK-007).
- Rust companion scaffolded and verified (TASK-008).
- Native Messaging protocol implemented (TASK-009).
- Vault cryptographic design finalized (TASK-010).
- Database abstraction created (TASK-011).
- Vault initialization implemented (TASK-012).
- Vault unlock/lock state machine implemented (TASK-013).
- Encrypted persistence tests implemented (TASK-014).
- Job domain model implemented (TASK-020).
- JobSnapshot model implemented (TASK-021).
- Job CRUD operations implemented (TASK-022).
- Snapshot history operations implemented (TASK-023).
- Local job search implemented (TASK-024).
- Generic extraction framework implemented (TASK-030).
- LinkedIn adapter implemented (TASK-031).
- Indeed adapter implemented (TASK-032).
- Generic career-page adapter implemented (TASK-033).
- Manual save fallback implemented (TASK-034).
- URL/canonical matching implemented (TASK-040).
- Job fingerprinting implemented (TASK-041).
- Text normalization implemented (TASK-050).
- Popup job list with IPC fetch, safe rendering, filtering, and states implemented (TASK-060).
- Popup job detail view with IPC fetch, safe rendering, and back navigation implemented (TASK-061).
- Snapshot history UI with clickable snapshots, snapshot detail view, and back-to-job navigation implemented (TASK-062).

## Not yet implemented

- Companion IPC (companion-side handlers for job.list, job.get return NOT_IMPLEMENTED)
- Database encryption
- Browser adapters
- Diff engine (bullet diff, moved/reordered detection, change ranking pending; paragraph/sentence diff exists on blocked branch task/TASK-051)
- Remaining UI (comparison view, change highlighting, keyword view)

## Known blockers

Resolved: Windows Smart App Control previously blocked Rust build/test execution
(os error 4551) on the Windows toolchain. The Rust companion is now built and tested
inside WSL (Ubuntu, mounted at `/mnt/c/Users/test/Downloads/...`), which is unaffected
by Smart App Control. See AGENTS.md section 7b for the supported Rust workflow.
No remaining blocker on the Rust roadmap.

## Known security concerns

Vault crypto design is finalized (ADR-005) and vault code compiles; runtime
verification via `cargo test` is now possible in WSL and should be exercised when
touching vault code.
No telemetry/network surface has been added.

## Last updated

TASK-062: Snapshot history UI implemented; 86 extension tests passing.

# Current State

## Project

Job Vault

## Status

IN_PROGRESS

## Current phase

Phase 3 — Browser integration

## Current task

TASK-051 (BLOCKED)

## Recommended next action

Resolve Windows Smart App Control blocking local build output (human decision), then
run `cargo test` on branch `task/TASK-051` to finish TASK-051. Alternative non-blocked
work: extension-side tasks (e.g. TASK-060) do not depend on executing Rust test binaries.

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

## Not yet implemented

- Extension
- Companion IPC
- Database encryption
- Browser adapters
- Diff engine (paragraph/sentence diff code written but tests not executed; bullet diff, moved/reordered detection, change ranking pending)
- UI

## Known blockers

Windows Smart App Control is On and blocks execution of freshly compiled unsigned
binaries (os error 4551). `cargo test` cannot run any newly built test harness or build
script. Code compiles (`cargo check`/`cargo build` succeed because existing cached
artifacts are reused). Unblocking requires a human to turn off Smart App Control
(Settings > Privacy & security > Windows Security > App & browser control) — note this
is permanent until Windows is reinstalled — or otherwise permit local build output.
Rust tasks cannot be verified until resolved.

## Known security concerns

No implementation exists yet. Security design must be completed before sensitive storage is implemented.

## Last updated

TASK-051: Paragraph/sentence diff implemented; BLOCKED — Windows Smart App Control prevents executing Rust test binaries.

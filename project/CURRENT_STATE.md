# Current State

## Project

Job Vault

## Status

IN_PROGRESS

## Current phase

Phase 4 — Repost detection complete (URL/external-ID matching, fingerprints, similarity matching); Phase 5 diff engine and Phase 6 UI (TASK-060..062) merged

## Current task

TASK-042 (DONE) — Content similarity matching. Merge of TASK-040..042 lands matching complete on main.

## Recommended next action

TASK-063 (comparison view) — P0, dependency TASK-055 (DONE). TASK-043 (match-review UI, P1, deps TASK-042) also eligible.

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
- Paragraph/sentence diff engine implemented (TASK-051).
- Bullet diff engine implemented (TASK-052).
- Moved/reordered detection implemented (TASK-053).
- Requirement/responsibility change detection implemented (TASK-054).
- Change ranking (deterministic significance scoring and ordering) implemented (TASK-055).
- Popup job list with IPC fetch, safe rendering, filtering, and states implemented (TASK-060).
- Popup job detail view with IPC fetch, safe rendering, and back navigation implemented (TASK-061).
- Snapshot history UI with clickable snapshots, snapshot detail view, and back-to-job navigation implemented (TASK-062).
- Content-similarity matching implemented (TASK-042): deterministic token-overlap similarity over normalized posting text (title/company/location/description), combined with URL/external-ID/fingerprint signals into a per-candidate confidence. 263 companion tests pass.

## Not yet implemented

- Companion IPC (companion-side handlers for job.list, job.get return NOT_IMPLEMENTED)
- Database encryption
- Browser adapters
- Match-review UI (TASK-043)
- Remaining UI (comparison view TASK-063, change highlighting TASK-064, keyword view TASK-065)

## Test execution note

Rust tests are executed in WSL (Ubuntu) via `cargo test` because Windows Smart App
Control blocks execution of freshly compiled unsigned binaries on the host. All 263
companion tests pass (247 prior plus 16 similarity-matching tests from TASK-042).
Run with `wsl -d Ubuntu -u root -- bash -lic "cd /root/job-vault/companion && cargo test"`
(always `-u root`: the distro default user is `test`, which cannot read `/root/job-vault`).

## Known security concerns

Vault crypto design is finalized (ADR-005) and vault code compiles, but runtime
verification on the host remains blocked by Smart App Control. No telemetry/network
surface has been added.

## Last updated

TASK-042: Content similarity matching implemented; 263 companion tests passing via WSL.

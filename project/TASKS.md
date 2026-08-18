# Task Registry

## Status vocabulary

- `BACKLOG`
- `READY`
- `IN_PROGRESS`
- `BLOCKED`
- `REVIEW`
- `DONE`
- `ABANDONED`

A task may only be marked `DONE` when all acceptance criteria are satisfied and relevant tests pass.

---

# Phase 0 — Foundation

## TASK-001 — Initialize repository

Status: DONE
Priority: P0
Dependencies: None

Goal:

Create the base Git repository and directory structure described by the architecture documents.

Acceptance criteria:

- [x] Git repository exists.
- [x] Extension directory exists.
- [x] Companion directory exists.
- [x] `docs/`, `project/`, `fixtures/`, and `scripts/` exist.
- [x] Basic README exists.
- [x] Appropriate ignore files exist.
- [x] No secrets or user data are committed.
- [x] Project can be built/tested once implementation scaffolding is added.

Likely next task:

TASK-002.

---

## TASK-002 — Establish agent/project documentation

Status: DONE
Priority: P0
Dependencies: TASK-001 (DONE)

Goal:

Create and validate `AGENTS.md`, task management files, handoff workflow, and project status tooling.

Acceptance criteria:

- [x] Agent instructions are present.
- [x] Task registry is machine-readable enough for humans and agents.
- [x] Handoff format is documented.
- [x] Current-state format is documented.
- [x] Agent workflow is documented.
- [x] No ambiguity exists about how to choose the next task.

---

## TASK-003 — Create architecture documentation

Status: DONE
Priority: P0
Dependencies: TASK-001 (DONE)

Goal:

Turn the architecture plan into implementation-ready documentation.

Acceptance criteria:

- [x] Component boundaries documented.
- [x] Extension/companion boundary documented.
- [x] Adapter architecture documented.
- [x] Storage abstraction documented.
- [x] Diff architecture documented.
- [x] Future extension points documented.

---

## TASK-004 — Create threat model

Status: DONE
Priority: P0
Dependencies: TASK-003 (DONE)

Goal:

Document threats, assets, trust boundaries, mitigations, and residual risks.

Acceptance criteria:

- [x] Assets listed.
- [x] Threat actors listed.
- [x] Browser boundary documented.
- [x] Extension/companion boundary documented.
- [x] Database boundary documented.
- [x] Lost/stolen-device scenario addressed.
- [x] Malicious-page scenario addressed.
- [x] malicious-dependency/agent-change scenario addressed.

---

## TASK-005 — Create security specification

Status: DONE
Priority: P0
Dependencies: TASK-004 (DONE)

Goal:

Define concrete security requirements before implementing encryption or IPC.

Acceptance criteria:

- [x] Password handling requirements defined.
- [x] KDF requirements defined.
- [x] encryption requirements defined.
- [x] key-management requirements defined.
- [x] vault-lock requirements defined.
- [x] logging policy defined.
- [x] backup requirements defined.
- [x] dependency policy defined.
- [x] network policy defined.

Do not invent cryptographic parameters without reviewing current library guidance.

---

## TASK-006 — Create permission policy

Status: DONE
Priority: P0
Dependencies: TASK-003 (DONE)

Goal:

Define the minimum Chrome permissions and host access strategy.

Acceptance criteria:

- [x] Every permission has a documented reason.
- [x] Optional vs required permissions considered.
- [x] `<all_urls>` explicitly rejected unless later justified.
- [x] Site access strategy documented.

---

## TASK-007 — Scaffold Chrome extension

Status: DONE
Priority: P0
Dependencies: TASK-003 (DONE), TASK-006 (DONE)

Goal:

Create a minimal Manifest V3 extension with side-panel/popup foundation and test/build tooling.

Acceptance criteria:

- [x] Extension builds.
- [x] Extension loads in Chrome developer mode.
- [x] No unnecessary permissions.
- [x] No network calls.
- [x] No sensitive storage implemented yet.
- [x] Basic automated test infrastructure exists.

---

## TASK-008 — Scaffold Rust companion

Status: DONE
Priority: P0
Dependencies: TASK-003 (DONE), TASK-005 (DONE)

Goal:

Create the local companion process with clean module boundaries.

Acceptance criteria:

- [x] Companion builds.
- [x] Tests run.
- [x] Vault module boundary exists.
- [x] Database module boundary exists.
- [x] IPC module boundary exists.
- [x] No network server is created.

Note: Rust build toolchain (MSVC) required installation of Visual Studio Build Tools.

---

## TASK-009 — Implement Native Messaging protocol

Status: DONE
Priority: P0
Dependencies: TASK-007 (DONE), TASK-008 (DONE)

Goal:

Create a versioned, explicit extension-to-companion IPC protocol.

Acceptance criteria:

- [x] Protocol version exists.
- [x] Request IDs exist.
- [x] Explicit operation names exist.
- [x] Schema validation exists.
- [x] Malformed requests are rejected.
- [x] Arbitrary command execution is impossible.
- [x] Integration test exists.

---

## TASK-010 — Finalize vault cryptographic design

Status: READY
Priority: P0
Dependencies: TASK-005 (DONE), TASK-008 (DONE)

Goal:

Finalize the exact cryptographic implementation plan using established libraries and current best practices.

Acceptance criteria:

- [ ] KDF selected.
- [ ] AEAD/encryption scheme selected.
- [ ] key hierarchy documented.
- [ ] nonce/IV handling documented.
- [ ] randomness requirements documented.
- [ ] password verification approach documented.
- [ ] library choice documented.
- [ ] no custom cryptography required.

This task requires human review before implementation if the agent encounters ambiguity.

---

# Phase 1 — Vault and database

## TASK-011 — Database abstraction

Status: BACKLOG
Priority: P0
Dependencies: TASK-008, TASK-010

Goal:

Create storage abstraction and schema foundation.

---

## TASK-012 — Vault initialization

Status: BACKLOG
Priority: P0
Dependencies: TASK-010, TASK-011

Goal:

Create a new encrypted vault from a user password.

---

## TASK-013 — Vault unlock/lock

Status: BACKLOG
Priority: P0
Dependencies: TASK-012

Goal:

Implement safe vault lifecycle.

---

## TASK-014 — Encrypted persistence tests

Status: BACKLOG
Priority: P0
Dependencies: TASK-013

Goal:

Test wrong passwords, corruption, lock/unlock cycles, and sensitive-data absence from logs.

---

# Phase 2 — Job model

## TASK-020 — Job domain model

Status: BACKLOG
Priority: P0
Dependencies: TASK-011

## TASK-021 — JobSnapshot model

Status: BACKLOG
Priority: P0
Dependencies: TASK-020

## TASK-022 — Job CRUD

Status: BACKLOG
Priority: P0
Dependencies: TASK-021, TASK-013

## TASK-023 — Snapshot history

Status: BACKLOG
Priority: P0
Dependencies: TASK-022

## TASK-024 — Local job search

Status: BACKLOG
Priority: P1
Dependencies: TASK-022

---

# Phase 3 — Browser integration

## TASK-030 — Generic extraction framework

Status: BACKLOG
Priority: P0
Dependencies: TASK-009, TASK-021

## TASK-031 — LinkedIn adapter

Status: BACKLOG
Priority: P0
Dependencies: TASK-030

## TASK-032 — Indeed adapter

Status: BACKLOG
Priority: P0
Dependencies: TASK-030

## TASK-033 — Generic career-page adapter

Status: BACKLOG
Priority: P0
Dependencies: TASK-030

## TASK-034 — Manual save fallback

Status: BACKLOG
Priority: P0
Dependencies: TASK-030

---

# Phase 4 — Repost detection

## TASK-040 — URL/canonical matching

Status: BACKLOG
Priority: P1
Dependencies: TASK-023

## TASK-041 — Job fingerprinting

Status: BACKLOG
Priority: P0
Dependencies: TASK-023

## TASK-042 — Similarity matching

Status: BACKLOG
Priority: P0
Dependencies: TASK-041

## TASK-043 — Match-review UI

Status: BACKLOG
Priority: P1
Dependencies: TASK-042

---

# Phase 5 — Diff engine

## TASK-050 — Text normalization

Status: BACKLOG
Priority: P0
Dependencies: TASK-023

## TASK-051 — Paragraph/sentence diff

Status: BACKLOG
Priority: P0
Dependencies: TASK-050

## TASK-052 — Bullet diff

Status: BACKLOG
Priority: P0
Dependencies: TASK-050

## TASK-053 — Moved/reordered detection

Status: BACKLOG
Priority: P1
Dependencies: TASK-052

## TASK-054 — Requirement/responsibility changes

Status: BACKLOG
Priority: P0
Dependencies: TASK-052

## TASK-055 — Change ranking

Status: BACKLOG
Priority: P0
Dependencies: TASK-054

---

# Phase 6 — UI

## TASK-060 — Job list

Status: BACKLOG
Priority: P1
Dependencies: TASK-022, TASK-007

## TASK-061 — Job detail

Status: BACKLOG
Priority: P1
Dependencies: TASK-060

## TASK-062 — Snapshot history UI

Status: BACKLOG
Priority: P1
Dependencies: TASK-061, TASK-023

## TASK-063 — Comparison view

Status: BACKLOG
Priority: P0
Dependencies: TASK-055

## TASK-064 — Change highlighting

Status: BACKLOG
Priority: P0
Dependencies: TASK-063

## TASK-065 — Keyword/tailoring view

Status: BACKLOG
Priority: P0
Dependencies: TASK-055, TASK-064

---

# Phase 7 — Hardening

## TASK-070 — Security audit

Status: BACKLOG
Priority: P0
Dependencies: TASK-065

## TASK-071 — Permission audit

Status: BACKLOG
Priority: P0
Dependencies: TASK-070

## TASK-072 — Logging/privacy audit

Status: BACKLOG
Priority: P0
Dependencies: TASK-070

## TASK-073 — Network audit

Status: BACKLOG
Priority: P0
Dependencies: TASK-070

## TASK-074 — IPC robustness testing

Status: BACKLOG
Priority: P1
Dependencies: TASK-009

## TASK-075 — Database corruption testing

Status: BACKLOG
Priority: P0
Dependencies: TASK-014

## TASK-076 — Encrypted backup/restore

Status: BACKLOG
Priority: P1
Dependencies: TASK-014

## TASK-077 — Documentation audit

Status: BACKLOG
Priority: P1
Dependencies: TASK-070

---

# Phase 8 — Future features

These are deliberately outside MVP.

## TASK-100 — Autofill foundation

## TASK-101 — Focused-field text insertion

## TASK-110 — Application tracking

## TASK-120 — Resume version management

## TASK-130 — Cover-letter management

## TASK-140 — Local AI provider interface

## TASK-141 — Local AI change interpretation

## TASK-142 — Local AI tailoring suggestions

## TASK-150 — Interview preparation

---

# Ready-task rule

A task is READY only if:

1. its dependencies are DONE;
2. it has sufficient acceptance criteria;
3. it does not require an unresolved architectural decision.

If no suitable READY task exists, do not invent one. Explain the blocker and ask the user if necessary.

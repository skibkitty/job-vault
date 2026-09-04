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

Note: The Rust build toolchain runs in WSL (see AGENTS.md section 7b). A Windows MSVC installation was previously required for the Rust toolchain but Smart App Control blocks Windows-native build output; use WSL for all Rust build/test.

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

Status: DONE
Priority: P0
Dependencies: TASK-005 (DONE), TASK-008 (DONE)

Goal:

Finalize the exact cryptographic implementation plan using established libraries and current best practices.

Acceptance criteria:

- [x] KDF selected.
- [x] AEAD/encryption scheme selected.
- [x] key hierarchy documented.
- [x] nonce/IV handling documented.
- [x] randomness requirements documented.
- [x] password verification approach documented.
- [x] library choice documented.
- [x] no custom cryptography required.

This task requires human review before implementation if the agent encounters ambiguity.

---

# Phase 1 — Vault and database

## TASK-011 — Database abstraction

Status: DONE
Priority: P0
Dependencies: TASK-008 (DONE), TASK-010 (DONE)

Goal:

Create storage abstraction and schema foundation.

Acceptance criteria:

- [x] Storage trait/abstraction exists.
- [x] SQLite schema for job, job_snapshot, job_changeset.
- [x] Schema creation is idempotent.
- [x] Tests pass.

---

## TASK-012 — Vault initialization

Status: DONE
Priority: P0
Dependencies: TASK-010 (DONE), TASK-011 (DONE)

Goal:

Create a new encrypted vault from a user password.

Acceptance criteria:

- [x] Vault created from password.
- [x] Salt, KEK, DEK generated.
- [x] DEK encrypted under KEK.
- [x] Verification tag computed.
- [x] Vault header written as JSON.
- [x] Wrong password fails.
- [x] Tests pass.

---

## TASK-013 — Vault unlock/lock

Status: DONE
Priority: P0
Dependencies: TASK-012 (DONE)

Goal:

Implement safe vault lifecycle with proper state machine validation.

Acceptance criteria:

- [x] State machine prevents invalid transitions (e.g., unlock while unlocked, lock while locked)
- [x] Valid transitions: Locked→Unlocking, Unlocking→Unlocked, Unlocking→Error, Unlocked→Locking, Locking→Locked, Error→Locked, Error→Unlocking
- [x] lock() returns Result<(), String> to handle invalid transitions
- [x] unlock() uses transition validation
- [x] Error state can recover to Locked or Unlocking
- [x] DEK memory is zeroized on lock() using zeroize crate
- [x] Automatic locking with configurable inactivity timeout
- [x] Vault::open() loads existing vault from directory path
- [x] All existing tests pass
- [x] New tests for invalid state transitions, auto-lock, and vault open exist

---

## TASK-014 — Encrypted persistence tests

Status: DONE
Priority: P0
Dependencies: TASK-013 (DONE)

Goal:

Test wrong passwords, corruption, lock/unlock cycles, and sensitive-data absence from logs.

Acceptance criteria:

- [x] Multiple wrong password attempts fail safely and leave vault in Error state
- [x] Recovery from Error state via lock() works
- [x] Vault header corruption (tampered JSON, invalid fields) detected and fails gracefully
- [x] Multiple lock/unlock cycles work correctly
- [x] Lock/unlock cycles preserve vault integrity
- [x] No sensitive data (DEK, password, KEK) appears in error messages
- [x] Vault state is consistent after failed unlock attempts
- [x] All tests pass

---

# Phase 2 — Job model

## TASK-020 — Job domain model

Status: DONE
Priority: P0
Dependencies: TASK-011 (DONE)

Goal:

Define and implement the core Job domain model with all required fields and validation.

Acceptance criteria:

- [x] Job struct with all required fields (id, title, company, location, url, description, etc.)
- [x] Job fields have appropriate types and validation
- [x] Job can be serialized/deserialized
- [x] Job has proper Debug and Clone implementations
- [x] Tests for Job creation and validation
- [x] All tests pass

## TASK-021 — JobSnapshot model

Status: DONE
Priority: P0
Dependencies: TASK-020 (DONE)

Goal:

Define and implement the JobSnapshot model for capturing job state at a point in time.

Acceptance criteria:

- [x] JobSnapshot struct with all required fields from DATA-MODEL.md
- [x] JobSnapshot has proper validation
- [x] JobSnapshot can be serialized/deserialized
- [x] JobSnapshot has proper Debug and Clone implementations
- [x] Tests for JobSnapshot creation and validation
- [x] All tests pass

## TASK-022 — Job CRUD

Status: DONE
Priority: P0
Dependencies: TASK-021 (DONE), TASK-013 (DONE)

Goal:

Implement Create, Read, Update, Delete operations for Job entities.

Acceptance criteria:

- [x] Job CRUD operations (create, get_by_id, update, delete)
- [x] Jobs are stored in SQLite database
- [x] CRUD operations work through vault abstraction
- [x] Validation before write operations
- [x] Tests for all CRUD operations
- [x] All tests pass

## TASK-023 — Snapshot history

Status: DONE
Priority: P0
Dependencies: TASK-022 (DONE)

Goal:

Implement operations for managing job snapshot history.

Acceptance criteria:

- [x] Create snapshots for jobs
- [x] Get snapshot history for a job
- [x] Get specific snapshot by id
- [x] Snapshots are linked to jobs via job_id
- [x] Tests for snapshot operations
- [x] All tests pass

## TASK-024 — Local job search

Status: DONE
Priority: P1
Dependencies: TASK-022 (DONE)

Goal:

Implement local search functionality for jobs in the vault.

Acceptance criteria:

- [x] Search jobs by title, company, location
- [x] Search is case-insensitive
- [x] Search returns matching jobs
- [x] Search works on unlocked vault
- [x] Tests for search functionality
- [x] All tests pass

---

# Phase 3 — Browser integration

## TASK-030 — Generic extraction framework

Status: DONE
Priority: P0
Dependencies: TASK-009 (DONE), TASK-021 (DONE)

Goal:

Create a generic framework for extracting job data from web pages.

Acceptance criteria:

- [x] Extractor trait/interface for job data extraction
- [x] Support for different extraction strategies
- [x] Basic text extraction from HTML
- [x] Tests for extraction framework
- [x] All tests pass

## TASK-031 — LinkedIn adapter

Status: DONE
Priority: P0
Dependencies: TASK-030 (DONE)

Goal:

Implement job extraction adapter for LinkedIn job pages.

Acceptance criteria:

- [x] LinkedIn extractor implementing JobExtractor trait
- [x] Can detect LinkedIn job URLs
- [x] Extracts job title, company, location, description
- [x] Tests for LinkedIn extraction
- [x] All tests pass

## TASK-032 — Indeed adapter

Status: DONE
Priority: P0
Dependencies: TASK-030 (DONE)

Goal:

Implement job extraction adapter for Indeed job pages.

Acceptance criteria:

- [x] Indeed extractor implementing JobExtractor trait
- [x] Can detect Indeed job URLs
- [x] Extracts job title, company, location, description
- [x] Tests for Indeed extraction
- [x] All tests pass

## TASK-033 — Generic career-page adapter

Status: DONE
Priority: P0
Dependencies: TASK-030 (DONE)

Goal:

Implement a fallback adapter for extracting job data from generic company career pages that do not have site-specific adapters.

Acceptance criteria:

- [x] Generic extractor implementing JobExtractor trait
- [x] Detects career page URLs (e.g., /careers/, /jobs/, /positions/)
- [x] Extracts job title from common HTML patterns (h1, h2, title tags)
- [x] Extracts company name from page metadata or common selectors
- [x] Extracts job description from main content areas
- [x] Uses sensible defaults when selectors fail
- [x] Falls back gracefully when extraction is poor quality
- [x] Tests for generic extraction
- [x] All tests pass

## TASK-034 — Manual save fallback

Status: DONE
Priority: P0
Dependencies: TASK-030 (DONE)

Goal:

Implement a manual save fallback so users can save job data when automatic extraction fails or is not available.

Acceptance criteria:

- [x] Manual extractor implementing JobExtractor trait
- [x] Accepts manually provided job data (title, company, location, description)
- [x] Always returns true for can_extract (manual mode)
- [x] Creates Job and JobSnapshot from provided fields
- [x] Validates required fields are present
- [x] Generates unique IDs for Job and JobSnapshot
- [x] Tests for manual extraction
- [x] All tests pass

---

# Phase 4 — Repost detection

## TASK-040 — URL/canonical matching

Status: DONE
Priority: P1
Dependencies: TASK-023 (DONE)

Goal:

Implement URL-based matching to detect when a new job snapshot is a repost of an existing job.

Acceptance criteria:

- [x] Match jobs by canonical URL equality
- [x] Match jobs by external job ID equality
- [x] Return match confidence (1.0 for exact URL match)
- [x] Handle URL normalization (trailing slashes, query params)
- [x] Store match results for later use
- [x] Tests for URL matching
- [x] All tests pass

## TASK-041 — Job fingerprinting

Status: DONE
Priority: P0
Dependencies: TASK-023 (DONE)

Goal:

Create deterministic fingerprints for job postings to enable similarity matching.

Acceptance criteria:

- [x] Generate fingerprint from job fields (title, company, location, description)
- [x] Fingerprint is deterministic (same input = same output)
- [x] Normalize text before fingerprinting (lowercase, strip whitespace)
- [x] Use content hashing for fingerprint generation
- [x] Store fingerprints with job snapshots
- [x] Tests for fingerprint generation
- [x] All tests pass

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

Status: DONE
Priority: P0
Dependencies: TASK-023 (DONE)

Goal:

Create text normalization utilities for consistent comparison and diffing of job descriptions.

Acceptance criteria:

- [x] Normalize whitespace (collapse multiple spaces, trim)
- [x] Normalize line breaks (convert to consistent format)
- [x] Strip HTML tags and decode entities
- [x] Lowercase text for case-insensitive comparison
- [x] Remove common filler words (optional, for better diffing)
- [x] Preserve meaningful punctuation
- [x] Tests for normalization functions
- [x] All tests pass

## TASK-051 — Paragraph/sentence diff

Status: DONE
Priority: P0
Dependencies: TASK-050 (DONE)

Goal:

Implement deterministic paragraph- and sentence-level diffing of job description text.

Acceptance criteria:

- [x] Split text into paragraphs on blank lines
- [x] Split text into sentences on terminal punctuation
- [x] Detect added paragraphs/sentences
- [x] Detect removed paragraphs/sentences
- [x] Detect modified paragraphs/sentences via similarity-threshold pairing
- [x] Comparison uses TASK-050 normalization (HTML stripping, lowercasing, whitespace collapsing)
- [x] Original un-normalized segment text preserved in results for display
- [x] Fully deterministic: identical inputs produce identical output
- [x] No LLM, no network access, no new external dependencies
- [x] Tests for splitting, diff classification, normalization tolerance, and determinism
- [x] All tests pass (202 passed, 0 failed; run via WSL bypassing Smart App Control)

## TASK-052 — Bullet diff

Status: DONE
Priority: P0
Dependencies: TASK-050 (DONE), TASK-051 (DONE)

Goal:

Implement deterministic diffing of bulleted list items in job descriptions (requirements, responsibilities).

Acceptance criteria:

- [x] Split bulleted lists into individual bullet segments (dash, star, bullet char, numbered markers)
- [x] Detect added bullets
- [x] Detect removed bullets
- [x] Detect modified bullets via similarity-threshold pairing
- [x] Continuation lines (wrapped text) are appended to the owning bullet
- [x] Comparison uses TASK-050 normalization
- [x] Original un-normalized bullet text preserved in results for display
- [x] Fully deterministic: identical inputs produce identical output
- [x] No LLM, no network access, no new external dependencies
- [x] Tests for splitting, add/remove/modify classification, continuation lines, normalization tolerance, and determinism
- [x] All tests pass (218 passed, 0 failed; run via WSL)

## TASK-053 — Moved/reordered detection

Status: DONE
Priority: P1
Dependencies: TASK-052 (DONE)

Goal:

Detect when bullets/segments are moved or reordered between two snapshots rather than reported as removed-and-added.

Acceptance criteria:

- [x] Add a `Moved` change type distinct from Added/Removed/Modified
- [x] Detect a single moved item (same content, changed position)
- [x] Detect full reordering of a list without false Removed/Added pairs
- [x] Item content matched by TASK-050 normalization (case/whitespace/punctuation tolerant)
- [x] Works for bullets (`diff_bullets_reordered`) and generic segments (`diff_segments_reordered`)
- [x] Modified/added/removed classification still honored when items genuinely differ
- [x] Fully deterministic: identical inputs produce identical output
- [x] No LLM, no network access, no new external dependencies
- [x] Tests for swap, rotation, move-to-end, addition+move, normalization tolerance, modification+removal, and determinism
- [x] All tests pass (228 passed, 0 failed; run via WSL)

## TASK-054 — Requirement/responsibility changes

Status: DONE
Priority: P0
Dependencies: TASK-052 (DONE)

Goal:

Detect and classify changes specifically in a job's requirements and responsibilities sections between two snapshots.

Acceptance criteria:

- [x] Diff a requirements section independently (`diff_requirements`)
- [x] Diff a responsibilities section independently (`diff_responsibilities`)
- [x] Detect added requirements/responsibilities
- [x] Detect removed requirements/responsibilities
- [x] Detect modified requirements/responsibilities via similarity pairing
- [x] Handle `Option<&str>` (missing section means empty)
- [x] Segment requirement/responsibility text as bullets, falling back to paragraphs for prose
- [x] `diff_requirement_sections` reports added/removed counts for both sections plus full diffs
- [x] Normalization (case/punctuation) ignored for comparison
- [x] No LLM, no network access, no new external dependencies
- [x] Tests for added/removed/modified, combined counts, empty/missing, prose fallback, normalization tolerance
- [x] All tests pass (239 passed, 0 failed; run via WSL)

## TASK-055 — Change ranking

Status: BACKLOG
Priority: P0
Dependencies: TASK-054

---

# Phase 6 — UI

## TASK-060 — Job list

Status: DONE
Priority: P1
Dependencies: TASK-022 (DONE), TASK-007 (DONE)

Goal:

Display the user's saved jobs in the extension popup.

Acceptance criteria:

- [x] Popup fetches saved jobs via `job.list` IPC routed through the service worker
- [x] Strict validation of IPC payload shape before rendering
- [x] States handled: loading, empty, ready, vault locked, companion unavailable, not implemented, error
- [x] Renders only safe content (no innerHTML with untrusted data; http/https URLs only)
- [x] Client-side filter box narrows the visible list
- [x] Deterministic ordering (newest first, title tiebreak)
- [x] No new permissions, no network access
- [x] Pure data/render-preparation logic separated from DOM wiring
- [x] Tests for parsing, validation, filtering, sorting, and state resolution
- [x] `npm run typecheck` and `npm test` pass

## TASK-061 — Job detail

Status: DONE
Priority: P1
Dependencies: TASK-060 (DONE)

Goal:

Display full job details in the extension popup when a user selects a job from the list.

Acceptance criteria:

- [x] Clicking a job in the list requests `job.get` via IPC and shows a detail view
- [x] Strict validation of `job.get` response payload before rendering
- [x] States handled: loading, detail-ready, not-found, locked, unavailable, not-implemented, error
- [x] Detail view renders: title, company, location, salary, URL, description, requirements, responsibilities
- [x] Snapshot count and most-recent capturedAt shown when available
- [x] Back button returns to job list
- [x] Only safe content rendered (no innerHTML with untrusted data; http/https URLs only)
- [x] Pure data/render-preparation logic separated from DOM wiring
- [x] Tests for parsing, validation, state resolution, and detail rendering
- [x] `npm run typecheck` and `npm test` pass
- [x] No new permissions, no network access

## TASK-062 — Snapshot history UI

Status: DONE
Priority: P1
Dependencies: TASK-061 (DONE), TASK-023 (DONE)

Goal:

Display the full snapshot history for a job in the detail view, allowing users to browse and inspect individual snapshots.

Acceptance criteria:

- [x] Snapshot list in detail view shows each snapshot with capturedAt, sorted newest first
- [x] Clicking a snapshot replaces the detail content with that snapshot's full data (title, company, location, salary, URL, description, requirements, responsibilities)
- [x] Snapshot detail includes a "Back to job" button that returns to the job detail view
- [x] Snapshot parsing validates all snapshot fields from companion payload
- [x] Empty snapshot list shows appropriate message
- [x] All rendering uses createElement/textContent (no innerHTML)
- [x] Tests for snapshot parsing, snapshot list rendering, and snapshot detail state
- [x] `npm run typecheck` and `npm test` pass
- [x] No new permissions, no network access

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

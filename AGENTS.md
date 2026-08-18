# Job Vault — OpenCode Agent Instructions

## 1. Mission

Build a private, local-first personal job-search vault consisting of:

- a Chrome Manifest V3 extension;
- a local companion application;
- an encrypted local database/vault;
- job-site adapters for LinkedIn, Indeed, and generic company career pages;
- historical job snapshots;
- repost detection;
- deterministic change/diff analysis;
- a UI showing meaningful changes and tailoring keywords;
- future extensibility for autofill, resume/cover-letter management, and optional local AI.

The user intends to develop this project across many OpenCode sessions and many independent agents. The repository is the persistent memory of the project.

## 2. Source of truth

Before doing work, read:

1. `AGENTS.md`
2. `project/CURRENT_STATE.md`
3. `project/HANDOFF.md`
4. `project/TASKS.md`
5. relevant documents under `docs/`
6. relevant ADRs under `docs/decisions/`

Do not rely on previous chat/session history.

## 3. User interaction rule

The user may say:

> What is the next task?

Determine the highest-priority READY task whose dependencies are complete.

The user may say:

> Implement the next task. Ask me questions if you need my input or if something is not possible for you to do.

Then:

- select the next READY task;
- read its acceptance criteria;
- inspect dependencies;
- implement only the task and necessary supporting changes;
- ask the user when a decision genuinely requires human input, a security-sensitive decision is ambiguous, required credentials/access are unavailable, or the task is impossible with the available environment;
- do not ask questions merely because the task is difficult;
- prefer a safe, documented blocker over inventing requirements.

## 4. Non-negotiable privacy rules

Treat all job-search data as extremely sensitive personal information.

Never:

- send job-search data to remote services;
- add analytics;
- add telemetry;
- add advertising;
- add cloud synchronization;
- add remote AI APIs;
- add crash-reporting services that transmit data;
- collect unrelated browsing history;
- collect passwords;
- collect cookies;
- collect authentication tokens;
- collect payment information;
- store arbitrary page contents;
- store screenshots of unrelated pages;
- log resumes, cover letters, job descriptions, names, emails, phone numbers, or application notes;
- put real user data into tests, fixtures, commits, or documentation.

The extension should process job pages only as required by explicit user actions/features.

## 5. Security rules

The application is personal-use software, but should target a security posture inspired by technical safeguards in HIPAA/GLBA environments. Do NOT claim legal compliance.

Required principles:

- encryption at rest;
- authenticated encryption;
- strong password-based key derivation;
- secure random generation;
- protected key material;
- vault lock/unlock;
- automatic locking;
- integrity validation;
- least privilege;
- minimal Chrome permissions;
- strict IPC schemas;
- input validation;
- HTML sanitization;
- dependency minimization;
- secure backups;
- no plaintext sensitive logs.

Never implement cryptographic primitives manually.

Never invent a cryptographic protocol.

Use established, reputable cryptographic libraries and current library guidance.

Any change to cryptography, authentication, permissions, network access, IPC, or threat model requires explicit task scope and documentation.

## 6. Network rule

No outbound network request may be added to the extension or companion without explicit human approval and a documented ADR.

Core job functionality must not depend on remote services.

The user's browser naturally connects to websites such as LinkedIn, Indeed, and company career pages. That does not authorize the extension or companion to transmit saved job data elsewhere.

## 7. Architecture rules

Keep these boundaries:

- browser integration separate from domain logic;
- site-specific extraction separate from core job logic;
- UI separate from business logic;
- storage behind an abstraction;
- encryption behind an abstraction;
- IPC behind a versioned protocol;
- features modular;
- deterministic analysis independent of browser technology.

The extension should communicate with the local companion through Chrome Native Messaging rather than creating an unauthenticated localhost HTTP service.

The companion owns the sensitive database.

The extension must not directly manipulate the database.

## 8. Initial technology direction

Preferred:

- Chrome Manifest V3;
- TypeScript for extension;
- Rust for local companion;
- SQLite or another well-supported local database;
- established cryptographic libraries;
- Chrome Native Messaging.

These are architectural recommendations, not permission for an agent to substitute technologies casually.

Changing these choices requires an ADR and human review if the change materially affects security or architecture.

## 9. Initial supported sources

Priority:

1. LinkedIn
2. Indeed
3. generic company career pages

Do not create dozens of company-specific adapters.

Prefer:

- generic career-page adapter;
- platform adapters when useful (e.g. Workday/Greenhouse/Lever later);
- site-specific adapters only where necessary.

Automatic extraction must have a manual fallback.

## 10. Job snapshots

Never overwrite a previous job posting.

Every capture creates a historical `JobSnapshot`.

The system must support:

- first capture;
- later captures;
- snapshot history;
- matching likely reposts;
- comparison between snapshots.

## 11. Job identity

Do not use URL equality as the only identity mechanism.

Use multiple signals, potentially including:

- external job ID;
- canonical URL;
- company;
- title;
- location;
- description similarity.

Represent uncertain matches with confidence rather than silently merging questionable records.

## 12. Diff engine

The core diff engine must be deterministic.

It should identify:

- additions;
- removals;
- modifications;
- moved bullets;
- reordered items;
- added requirements;
- removed requirements.

The diff engine must not depend on an LLM.

AI interpretation is a future optional layer.

## 13. Future extensibility

Design stable interfaces for future:

- autofill;
- application tracking;
- resume versions;
- cover letters;
- snippets;
- local AI;
- encrypted backups;
- interview preparation.

Do not implement these during MVP unless their task is explicitly selected.

## 14. AI agent authority

An OpenCode agent is not automatically trusted with architectural judgment.

An agent may:

- implement a scoped task;
- add tests;
- refactor within scope;
- update documentation;
- fix directly related bugs.

An agent must not silently:

- add network access;
- add permissions;
- weaken encryption;
- replace cryptographic libraries;
- change IPC architecture;
- introduce cloud dependencies;
- add telemetry;
- change the threat model;
- reverse an accepted ADR.

If an architectural change is necessary, document it and ask the user when required.

## 15. Testing

Relevant tests must be run before a task is marked DONE.

Prefer:

- unit tests;
- integration tests;
- browser/IPC tests;
- sanitized HTML fixtures;
- security regression tests;
- malformed-input tests.

Never use real user job data as test fixtures.

## 16. Multi-agent workflow

Each meaningful task has a stable `TASK-###` ID.

Before work:

- read task;
- read dependencies;
- inspect current state;
- inspect relevant ADRs.

During work:

- keep scope narrow;
- update task notes when meaningful milestones occur;
- do not silently take unrelated tasks.

Before completion:

- run tests;
- inspect `git diff`;
- update task status;
- update `CURRENT_STATE.md`;
- update `HANDOFF.md`;
- update relevant documentation;
- create/update ADRs if applicable;
- update security log for security-relevant changes;
- commit in a logical unit.

## 17. Task completion

Never mark a task DONE simply because code exists.

A task is DONE only when:

- all acceptance criteria are satisfied;
- relevant tests pass;
- documentation is updated;
- no known blocker is being hidden;
- the implementation does not violate project rules.

If work is incomplete, use `IN_PROGRESS`, `BLOCKED`, or `REVIEW`.

## 18. Handoff requirement

Every session must leave enough information for a new agent to continue without conversation history.

Update `project/HANDOFF.md` before ending.

Include:

- agent/session;
- task;
- work completed;
- what works;
- what does not work;
- tests run;
- files changed;
- important discoveries;
- decisions made;
- known risks;
- blockers;
- exact next recommended task;
- exact instructions for the next agent.

## 19. Git

Use task-based commits such as:

`TASK-031: add LinkedIn job parser`

Avoid vague commits such as `stuff`, `updates`, or `fixes`.

Do not rewrite shared history unless explicitly instructed.

## 20. When blocked

Do not fake completion.

If blocked:

1. document the blocker;
2. explain what was attempted;
3. explain what is needed;
4. update task status to BLOCKED;
5. identify an alternative task if one is available.

If the blocker requires user input, ask the user.

## 21. Session ending checklist

Before ending:

- [ ] relevant tests run;
- [ ] no sensitive data in logs;
- [ ] no unexpected network access;
- [ ] no unexpected permissions;
- [ ] git diff reviewed;
- [ ] task status updated;
- [ ] CURRENT_STATE updated;
- [ ] HANDOFF updated;
- [ ] security log updated if relevant;
- [ ] ADR added/updated if relevant;
- [ ] logical commit created.

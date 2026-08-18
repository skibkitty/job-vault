# Architecture

## Goal

Private local-first Chrome extension + local companion for job-search tracking.

## High-level system

```text
Browser pages
  |
  v
Chrome Extension
  |
  | Chrome Native Messaging
  v
Rust Local Companion
  |
  +-- Vault
  +-- Database
  +-- Job Matching
  +-- Diff Engine
  +-- Change Analyzer
  +-- Backup
  |
  v
Encrypted Local Storage
```

## Extension responsibilities

- interact with supported job pages;
- extract job information after user action;
- provide UI;
- request operations from companion;
- future autofill.

The extension does not own the sensitive database.

## Companion responsibilities

- vault lifecycle;
- encryption/key management;
- database;
- job identity/matching;
- snapshot persistence;
- diffing;
- future local AI integration;
- encrypted backups.

## Domain boundaries

### Browser integration

Knows about Chrome and webpage DOMs.

### Adapters

Know how to extract job information from particular sites/platforms.

### Domain

Knows about jobs, snapshots, changes, matching, and tailoring signals.

Must not know about Chrome.

### Storage

Owns persistence.

### Security

Owns vault/key/encryption lifecycle.

### UI

Displays domain state but does not implement business logic.

## Initial adapters

1. LinkedIn
2. Indeed
3. Generic career page

Use a common adapter interface.

Conceptual interface:

```text
canHandle(page)
extractJob(page)
```

The exact implementation may evolve.

## Generic fallback

Automatic extraction must have a manual fallback so a broken selector does not make the application unusable.

## Job snapshots

Never overwrite previous job descriptions. Each capture is a snapshot.

## Job identity

Use multiple signals:

- external job ID;
- canonical URL;
- company;
- title;
- location;
- description similarity.

Represent uncertainty with confidence.

## Diff engine

Input:

```text
JobSnapshot A
JobSnapshot B
```

Output:

```text
ChangeSet
```

ChangeSet includes:

- added;
- removed;
- modified;
- moved;
- reordered;
- added requirements;
- removed requirements;
- metadata.

The engine is deterministic.

## Future providers

Use interfaces so future functionality can plug in:

```text
AnalysisProvider
AutofillProvider
```

The initial implementation does not require an LLM.

## Communication

Use Chrome Native Messaging.

Do not create a generic unauthenticated localhost server.

IPC should have:

- protocol version;
- request ID;
- explicit operation;
- schema validation;
- bounded payloads;
- structured errors.

## Database

The companion owns the database.

All sensitive persistence goes through the storage abstraction.

The exact database encryption implementation must follow the security specification and established libraries.

## Future local AI

Potential architecture:

```text
ChangeSet
   |
   v
Local Analysis Provider
   |
   v
tailoring suggestions
```

No remote AI is permitted by default.

# Security Specification

## Scope

This is a personal application, not a regulated HIPAA/GLBA-covered system. It does not claim legal compliance.

The goal is to adopt a strong technical security posture inspired by safeguards such as access control, authentication, encryption, integrity protection, auditability, data minimization, and transmission security.

## Security objectives

1. Confidentiality
2. Integrity
3. Availability
4. Data minimization
5. Least privilege
6. No unintended transmission

## Sensitive data

Treat as highly sensitive:

- job descriptions;
- application history;
- resumes;
- cover letters;
- recruiter information;
- notes;
- salary information;
- job-search URLs where relevant;
- company/job relationships;
- personal contact information;
- future interview notes.

## Data that must not be collected

- passwords;
- cookies;
- authentication tokens;
- browser history;
- unrelated page content;
- payment information;
- clipboard history;
- arbitrary keystrokes.

## Encryption

Required:

- established cryptographic library;
- modern password KDF;
- authenticated encryption;
- secure random generation;
- unique nonce/IV handling;
- protected key material;
- encrypted backups.

Do not implement cryptographic primitives manually.

Do not invent algorithms or key-management protocols.

The exact algorithms and parameters must be selected using current library guidance and documented in an ADR.

## Key hierarchy

Conceptual target:

```text
User password
   |
   v
Password KDF
   |
   v
Key-encryption key
   |
   v
Encrypted vault key
   |
   v
Data encryption
```

The master/data key must not be persisted plaintext.

## Password

- Never log it.
- Never store it plaintext.
- Do not provide a recovery mechanism that defeats encryption.
- Incorrect password must fail safely.
- Password changes must be designed carefully and tested.

## Vault

States:

- LOCKED
- UNLOCKING
- UNLOCKED
- LOCKING
- ERROR

Sensitive operations are unavailable while locked.

Automatic lock should eventually occur after inactivity and system sleep/lock.

## Database

Sensitive data must be encrypted at rest.

Do not invent ad-hoc database encryption.

## Logging

Runtime logs must not contain sensitive user data.

Project engineering logs may describe implementation state but must not contain real user data.

## Network

No application-level outbound transmission of user/job data.

Any new network access requires:

1. explicit human approval;
2. security review;
3. ADR;
4. documentation update;
5. tests.

## Permissions

Use least privilege.

Avoid `<all_urls>` unless a later feature makes it genuinely necessary and the user approves.

Every permission must have a documented purpose.

## IPC

Reject:

- malformed messages;
- unknown operations;
- invalid schemas;
- oversized payloads;
- invalid protocol versions.

Never expose arbitrary command execution.

## Input handling

Treat webpage HTML and text as untrusted input.

Sanitize appropriately.

Never execute page-provided scripts.

## Dependencies

Prefer minimal, established dependencies.

Review new dependencies for:

- maintenance;
- security history;
- transitive dependency size;
- necessity.

## Backups

Future backups must be encrypted.

No automatic cloud backup.

## Security regression

Before MVP:

- permission audit;
- network audit;
- logging audit;
- dependency review;
- IPC robustness tests;
- corrupted database tests;
- wrong-password tests;
- malicious HTML tests.

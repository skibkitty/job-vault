# Threat Model

## Assets

- encrypted job database;
- resumes;
- cover letters;
- application history;
- recruiter information;
- personal notes;
- encryption keys;
- user password;
- browser-extension state.

## Threat actors

### T1 — Attacker obtains database file

Goal:
Read job-search information.

Mitigation:
Strong authenticated encryption and protected keys.

### T2 — Device theft

Goal:
Read stored data from disk.

Mitigation:
Encrypted vault plus OS/device security.

### T3 — Person gains access to unlocked computer

Goal:
Read or modify data.

Mitigation:
Vault lock, inactivity timeout, OS authentication.

Residual risk:
If the vault is unlocked and the host is compromised, local confidentiality cannot be guaranteed.

### T4 — Malicious webpage

Goal:
Exploit extension or inject data.

Mitigation:

- least privilege;
- strict content-script boundaries;
- sanitization;
- validation;
- explicit operations;
- no arbitrary execution.

### T5 — Malicious or compromised dependency

Goal:
Exfiltrate or alter data.

Mitigation:

- dependency minimization;
- lockfiles;
- review;
- tests;
- no unnecessary packages.

### T6 — Coding agent introduces vulnerability

Mitigation:

- AGENTS.md rules;
- task scoping;
- security docs;
- network/permission audits;
- Git review;
- human review of cryptography/security changes.

### T7 — Accidental logging

Mitigation:

- no-sensitive-data logging policy;
- regression tests;
- code review.

## Trust boundaries

```text
Untrusted web page
       |
       v
Chrome extension
       |
       v
Native Messaging
       |
       v
Local companion
       |
       v
Encrypted storage
```

Each boundary validates input.

## Security assumptions

- User controls the computer.
- Operating system is not already fully compromised.
- Chrome is trusted.
- Cryptographic libraries are trusted and maintained.
- The companion executable is trusted.

## Out of scope for MVP

- defending against a fully compromised operating system;
- enterprise device management;
- remote multi-user access;
- cloud synchronization.

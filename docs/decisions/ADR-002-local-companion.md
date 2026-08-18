# ADR-002: Use a local companion application

Status: Accepted

## Context

The extension needs secure local storage, encryption, future backups, and future local AI capabilities.

## Decision

Use a local companion application rather than putting all sensitive functionality inside the Chrome extension.

Preferred implementation: Rust companion.

## Consequences

- Slightly more installation complexity.
- Better separation of browser and sensitive-data responsibilities.
- Easier future local AI integration.
- Easier database and cryptography management.

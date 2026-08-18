# ADR-003: Use Chrome Native Messaging

Status: Accepted

## Context

The extension must communicate with the local companion.

## Decision

Use Chrome Native Messaging rather than an unauthenticated localhost HTTP server.

## Reasons

- Explicit extension-to-host relationship.
- Avoids opening a listening network port.
- Clearer trust boundary.
- Appropriate for a personal local application.

## Consequences

- Companion registration/install process is more involved.
- IPC protocol must be explicitly versioned and validated.

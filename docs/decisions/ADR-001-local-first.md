# ADR-001: Local-first architecture

Status: Accepted

## Context

The application handles highly sensitive job-search information.

## Decision

Store and process job-search data locally.

No cloud database, analytics, telemetry, or remote AI by default.

## Consequences

- Better privacy.
- More control.
- Local backups become the user's responsibility.
- Future remote features require explicit review.

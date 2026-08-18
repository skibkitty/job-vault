# ADR-004: No remote AI by default

Status: Accepted

## Context

Resumes, job descriptions, cover letters, and application history are highly sensitive.

## Decision

Core functionality must not depend on remote AI.

Future AI should preferably run locally.

## Consequences

- Privacy is preserved.
- Core diff functionality remains deterministic.
- Local model integration can be added later behind an interface.

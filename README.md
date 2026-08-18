# Job Vault

Private, local-first personal job-search vault.

## Components

- **extension/** — Chrome Manifest V3 extension
- **companion/** — Rust local companion application
- **docs/** — Architecture, security, and design documentation
- **project/** — Project management (tasks, state, handoffs)
- **fixtures/** — Sanitized test fixtures
- **scripts/** — Utility scripts

## Architecture

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
  +-- Vault (encrypted)
  +-- Database
  +-- Job Matching
  +-- Diff Engine
  |
  v
Encrypted Local Storage
```

## Privacy

All job-search data stays on the local machine. No telemetry, no cloud sync, no remote AI. See `docs/PRIVACY.md` for details.

## Status

Phase 0 — Foundation. See `project/TASKS.md` for current tasks.

## License

TBD

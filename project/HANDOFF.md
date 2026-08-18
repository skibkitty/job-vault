# Agent Handoff

## Current session

Agent:
opencode/big-pickle

Date:
2026-08-18

Task:
TASK-010

## What has been done

- Created ADR-005: Vault Cryptographic Design.
- Selected: Argon2id KDF, AES-256-GCM AEAD, ring/argon2/rand crates.
- Documented: key hierarchy, nonce handling, password verification, vault header format.
- Added crypto dependencies to companion/Cargo.toml.

## What works

ADR-005 documents the complete cryptographic design.

## What does not work

Nothing broken.

## Tests run

None (documentation-only changes).

## Files changed

- docs/decisions/ADR-005-vault-crypto.md (new)
- companion/Cargo.toml
- project/TASKS.md
- project/CURRENT_STATE.md
- project/HANDOFF.md

## Important discoveries

The design follows ADR-005. Password verification uses HMAC-SHA256 rather than encrypting a known plaintext.

## Decisions

- Argon2id over scrypt/PBKDF2 (memory-hard)
- AES-256-GCM over ChaCha20-Poly1305 (hardware acceleration on desktop)
- HMAC-SHA256 for password verification (avoids leaking information)
- Vault header stored as JSON for readability/debuggability

## Known risks

None.

## Next recommended action

TASK-011 — Database abstraction.

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. Read docs/decisions/ADR-005-vault-crypto.md for cryptographic design.
3. Implement TASK-011.

## Blockers

None.

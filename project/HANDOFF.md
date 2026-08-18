# Agent Handoff

## Current session

Agent:
opencode/big-pickle

Date:
2026-08-18

Task:
TASK-012

## What has been done

- Implemented vault initialization: create vault from password.
- Created crypto module: Argon2id KDF, AES-256-GCM AEAD, HMAC-SHA256 verification.
- Vault writes JSON header with salt, verification tag, encrypted DEK.
- Vault unlock derives KEK, verifies password, unwraps DEK.
- 30 tests pass.

## What works

- Vault creation from password
- Password verification via HMAC-SHA256
- DEK wrap/unwrap with AES-256-GCM
- Vault header persistence as JSON
- Wrong password fails safely
- Vault cannot be created twice

## What does not work

Nothing broken.

## Tests run

- `cargo test` — 30 passed

## Files changed

- companion/Cargo.toml
- companion/src/main.rs
- companion/src/crypto/mod.rs (new)
- companion/src/vault/mod.rs
- project/TASKS.md
- project/CURRENT_STATE.md
- project/HANDOFF.md

## Important discoveries

- argon2 crate `Params::new` takes `(m_cost, t_cost, p_cost, output_len)` not `(t_cost, m_cost, ...)`.
- base64ct `encode` returns a `&str` slice into the buffer, not the full buffer.

## Decisions

- Vault header stored as JSON for debuggability.
- base64ct for encoding binary data in JSON header.

## Known risks

None.

## Next recommended action

TASK-013 — Vault unlock/lock.

## Instructions for next agent

1. Read AGENTS.md, project/CURRENT_STATE.md, project/HANDOFF.md, project/TASKS.md.
2. Implement TASK-013.

## Blockers

None.

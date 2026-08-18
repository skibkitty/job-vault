# ADR-005: Vault Cryptographic Design

## Status

Proposed

## Context

TASK-010 requires selecting concrete cryptographic algorithms, libraries, and key-management design for the encrypted vault. The security specification (docs/SECURITY.md) requires:

- established cryptographic library;
- modern password KDF;
- authenticated encryption;
- secure random generation;
- unique nonce/IV handling;
- protected key material;
- no custom cryptography.

## Decision

### Library

Use the `ring` crate for core cryptographic operations. It is:

- audited and maintained by the Rust Project;
- used by `rustls`, `boring`, and other widely-deployed crates;
- backed by BoringSSL's cryptographic primitives;
- no custom cryptography required.

Supporting crates:

- `argon2` — Argon2id password hashing (recommendation: use the standalone `argon2` crate for clearer API);
- `rand` — secure random generation via `rand::rngs::OsRng`;
- `hkdf` — HKDF-SHA256 for key derivation;
- `hmac` — HMAC-SHA256 for password verification.

### KDF: Argon2id

Select Argon2id as the password-based KDF.

Parameters (tunable, but starting point):

- memory: 64 MiB (65536 KiB);
- iterations: 3;
- parallelism: 4;
- output length: 32 bytes.

These parameters are deliberately conservative for a desktop application. They can be adjusted based on benchmarking on target hardware.

Argon2id is preferred over:

- scrypt — less memory-hard in some configurations;
- PBKDF2 — not memory-hard, vulnerable to GPU/ASIC attacks.

### AEAD: AES-256-GCM

Select AES-256-GCM as the authenticated encryption scheme.

- 256-bit key;
- 96-bit (12-byte) nonce;
- 128-bit authentication tag.

AES-256-GCM is:

- NIST-approved (SP 800-38D);
- hardware-accelerated on modern x86/ARM CPUs via AES-NI;
- widely supported in `ring`.

Alternative considered: ChaCha20-Poly1305. Viable but AES-GCM has better hardware performance on desktop. ChaCha20-Poly1305 may be used for future mobile/embedded targets.

### Key hierarchy

```text
User password
   |
   v
Argon2id(salt, password, params)
   |
   v
Key-Encryption Key (KEK) — 32 bytes
   |
   v
HMAC-SHA256(KEK, "verify") → password verification tag (32 bytes)
   |
   v
AES-256-GCM unwrap(KEK, encrypted_vault_key) → Vault Key (DEK) — 32 bytes
   |
   v
AES-256-GCM(DEK, nonce, plaintext) → ciphertext
```

### Vault key (DEK)

- 32-byte random key generated once during vault creation.
- Encrypted under KEK using AES-256-GCM.
- Stored alongside salt and verification tag.
- Never persisted plaintext.
- Re-encrypted when password changes (re-derive KEK from new password, unwrap DEK, re-wrap DEK with new KEK).

### Password verification

- During unlock: derive KEK from password + stored salt.
- Compute HMAC-SHA256(KEK, "verify") and compare to stored verification tag.
- If verification passes, unwrap DEK.
- If verification fails, return error without revealing which step failed.

This avoids encrypting a known plaintext (which would leak information). HMAC verification is a standard approach.

### Nonce/IV handling

- AES-GCM nonces are 12 bytes (96 bits).
- For the vault key encryption: use a nonce derived from a random 12-byte value stored alongside the encrypted DEK.
- For data encryption: use a random 12-byte nonce per encryption operation.
- Nonces must never be reused with the same key. Since nonces are random (not counter-based), collision probability is negligible with 96-bit random nonces and reasonable data volumes.

### Salt

- 16-byte random salt for Argon2id.
- Generated once during vault creation.
- Stored in plaintext alongside the encrypted vault key.

### Random generation

- Use `rand::rngs::OsRng` for all cryptographic randomness.
- OsRng is backed by the OS CSPRNG (`/dev/urandom` on Linux, `BCryptGenRandom` on Windows).
- Do not use `rand::thread_rng()` for cryptographic purposes (it uses a non-cryptographic PRNG internally).

### Password change

1. Prompt for old password.
2. Verify old password (HMAC check).
3. Unwrap DEK with old KEK.
4. Prompt for new password.
5. Derive new KEK from new password + new salt.
6. Wrap DEK with new KEK.
7. Store new salt, verification tag, and encrypted DEK.
8. Zeroize old KEK from memory.

### File layout

The vault header is stored as a JSON file:

```json
{
  "version": 1,
  "kdf": "argon2id",
  "kdf_params": {
    "memory_kib": 65536,
    "iterations": 3,
    "parallelism": 4
  },
  "salt": "<base64 16 bytes>",
  "verification_tag": "<base64 32 bytes>",
  "encrypted_dek_nonce": "<base64 12 bytes>",
  "encrypted_dek": "<base64 encrypted 32 bytes>",
  "created_at": "2026-01-01T00:00:00Z"
}
```

The encrypted database is stored separately as a binary file encrypted with the DEK.

## Consequences

- No custom cryptography. All operations use established libraries.
- Password brute-force is mitigated by Argon2id's memory hardness.
- Data confidentiality and integrity are provided by AES-256-GCM.
- Password change does not require re-encrypting all data (only the DEK is re-wrapped).
- The vault header is small and can be read quickly during unlock.
- Future: consider adding a key-rotation mechanism for the DEK itself.

## Dependencies added

- `argon2` — Argon2id KDF
- `ring` — AEAD, HMAC, HKDF
- `rand` — secure random generation

## Review

This ADR requires human review before implementation begins, as specified by TASK-010.

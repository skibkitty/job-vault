# Testing Strategy

## Unit tests

Test:

- text normalization;
- job identity;
- similarity;
- diffing;
- change ranking;
- serialization;
- database operations;
- encryption wrappers.

## Integration tests

Test:

```text
Extension
  -> Native Messaging
  -> Companion
  -> Database
```

## Fixture tests

Use sanitized HTML fixtures for:

- LinkedIn;
- Indeed;
- generic career pages.

Do not use real personal job-search data.

## Security tests

Must include:

- incorrect password;
- corrupted ciphertext/database;
- repeated lock/unlock;
- malformed IPC;
- invalid IPC schema;
- oversized IPC payload;
- malicious HTML;
- unexpected script content;
- unexpected network access where testable.

## Privacy tests

Core functionality should not make application-level outbound network requests.

No sensitive data should appear in runtime logs.

## Definition of test success

A task cannot be marked DONE when required tests fail.

Do not hide failing tests or weaken assertions just to obtain a green build.

# IPC Specification

## Transport

Chrome Native Messaging.

## Design goals

- explicit operations;
- schema validation;
- versioning;
- request/response correlation;
- bounded payloads;
- structured errors;
- no arbitrary commands.

## Conceptual request

```json
{
  "protocolVersion": 1,
  "requestId": "unique-id",
  "operation": "job.save",
  "payload": {}
}
```

## Conceptual response

```json
{
  "protocolVersion": 1,
  "requestId": "unique-id",
  "success": true,
  "data": {}
}
```

## Conceptual error

```json
{
  "protocolVersion": 1,
  "requestId": "unique-id",
  "success": false,
  "error": {
    "code": "VAULT_LOCKED",
    "message": "Vault is locked."
  }
}
```

## Initial operation families

- `vault.status`
- `vault.unlock`
- `vault.lock`
- `job.save`
- `job.get`
- `job.list`
- `job.compare`
- `job.search`

Future operations may include:

- `backup.create`
- `backup.restore`
- `snippet.list`
- `autofill.*`

## Security requirements

- Reject unknown operations.
- Validate payload schema.
- Reject invalid protocol versions.
- Bound payload size.
- Never expose arbitrary command execution.
- Never return secrets unnecessarily.
- Do not log sensitive payloads.

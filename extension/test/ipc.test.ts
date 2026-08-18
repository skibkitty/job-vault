import { describe, it, expect } from "vitest";
import { IpcRequest, IpcResponse } from "../src/types";

const VALID_OPERATIONS = [
  "vault.status",
  "vault.unlock",
  "vault.lock",
  "job.save",
  "job.get",
  "job.list",
  "job.compare",
  "job.search",
];

describe("IPC protocol", () => {
  it("request has required fields", () => {
    const request: IpcRequest = {
      protocolVersion: 1,
      requestId: "test-1",
      operation: "vault.status",
      payload: {},
    };
    expect(request.protocolVersion).toBe(1);
    expect(request.requestId).toBeTruthy();
    expect(request.operation).toBeTruthy();
  });

  it("response has required fields", () => {
    const response: IpcResponse = {
      protocolVersion: 1,
      requestId: "test-1",
      success: true,
      data: {},
    };
    expect(response.protocolVersion).toBe(1);
    expect(response.requestId).toBeTruthy();
    expect(typeof response.success).toBe("boolean");
  });

  it("error response has error code and message", () => {
    const response: IpcResponse = {
      protocolVersion: 1,
      requestId: "test-1",
      success: false,
      error: { code: "UNKNOWN_OPERATION", message: "Unknown operation: foo" },
    };
    expect(response.success).toBe(false);
    expect(response.error?.code).toBeTruthy();
    expect(response.error?.message).toBeTruthy();
  });

  it.each(VALID_OPERATIONS)("operation '%s' is valid", (op) => {
    expect(VALID_OPERATIONS).toContain(op);
  });

  it("unknown operations are rejected", () => {
    const dangerous = [
      "system.exec",
      "system.spawn",
      "file.read",
      "file.write",
      "eval",
      "shell",
    ];
    for (const op of dangerous) {
      expect(VALID_OPERATIONS).not.toContain(op);
    }
  });
});

import { describe, it, expect } from "vitest";
import {
  parseJobDetail,
  resolveDetailState,
  stateFromError,
  JobDetail,
} from "../src/ui/jobdetail";
import { IpcResponse } from "../src/types";

function detail(overrides: Partial<Record<string, unknown>> = {}): Record<string, unknown> {
  return {
    id: "job-1",
    title: "Software Engineer",
    company: "Example Corp",
    location: "Berlin",
    url: "https://example.com/jobs/1",
    salary: "100k",
    employmentType: "Full-time",
    description: "Build things.",
    requirements: ["TypeScript", "Rust"],
    responsibilities: ["Write code", "Review PRs"],
    snapshots: [
      { id: "snap-1", capturedAt: "2026-08-20T10:00:00Z" },
      { id: "snap-2", capturedAt: "2026-08-19T09:00:00Z" },
    ],
    createdAt: "2026-08-01T00:00:00Z",
    updatedAt: "2026-08-20T10:00:00Z",
    ...overrides,
  };
}

function successResponse(data: unknown): IpcResponse {
  return {
    protocolVersion: 1,
    requestId: "r1",
    success: true,
    data,
  };
}

function errorResponse(code: string, message: string): IpcResponse {
  return {
    protocolVersion: 1,
    requestId: "r1",
    success: false,
    error: { code, message },
  };
}

describe("parseJobDetail", () => {
  it("parses a valid detail payload", () => {
    const parsed = parseJobDetail(detail());
    expect(parsed).not.toBeNull();
    expect(parsed?.id).toBe("job-1");
    expect(parsed?.title).toBe("Software Engineer");
    expect(parsed?.company).toBe("Example Corp");
    expect(parsed?.location).toBe("Berlin");
    expect(parsed?.url).toBe("https://example.com/jobs/1");
    expect(parsed?.salary).toBe("100k");
    expect(parsed?.employmentType).toBe("Full-time");
    expect(parsed?.description).toBe("Build things.");
    expect(parsed?.requirements).toEqual(["TypeScript", "Rust"]);
    expect(parsed?.responsibilities).toEqual(["Write code", "Review PRs"]);
    expect(parsed?.snapshots).toHaveLength(2);
    expect(parsed?.createdAt).toBe("2026-08-01T00:00:00Z");
    expect(parsed?.updatedAt).toBe("2026-08-20T10:00:00Z");
  });

  it("rejects missing id", () => {
    expect(parseJobDetail(detail({ id: "" }))).toBeNull();
  });

  it("rejects missing title", () => {
    expect(parseJobDetail(detail({ title: null }))).toBeNull();
  });

  it("rejects non-object payloads", () => {
    expect(parseJobDetail("string")).toBeNull();
    expect(parseJobDetail(42)).toBeNull();
    expect(parseJobDetail(null)).toBeNull();
    expect(parseJobDetail(undefined)).toBeNull();
  });

  it("defaults optional fields to null when blank", () => {
    const parsed = parseJobDetail(
      detail({ company: "", location: "  ", salary: undefined })
    );
    expect(parsed?.company).toBeNull();
    expect(parsed?.location).toBeNull();
    expect(parsed?.salary).toBeNull();
  });

  it("defaults empty description to empty string", () => {
    const parsed = parseJobDetail(detail({ description: undefined }));
    expect(parsed?.description).toBe("");
  });

  it("defaults empty arrays when missing", () => {
    const parsed = parseJobDetail(
      detail({ requirements: undefined, responsibilities: undefined })
    );
    expect(parsed?.requirements).toEqual([]);
    expect(parsed?.responsibilities).toEqual([]);
  });

  it("filters out non-string array entries", () => {
    const parsed = parseJobDetail(
      detail({ requirements: ["TypeScript", 123, null, "Rust"] })
    );
    expect(parsed?.requirements).toEqual(["TypeScript", "Rust"]);
  });

  it("defaults empty snapshots array", () => {
    const parsed = parseJobDetail(detail({ snapshots: undefined }));
    expect(parsed?.snapshots).toEqual([]);
  });

  it("skips invalid snapshot entries", () => {
    const parsed = parseJobDetail(
      detail({ snapshots: [{ id: "s1", capturedAt: "2026-01-01" }, "bad", { id: "" }] })
    );
    expect(parsed?.snapshots).toHaveLength(1);
    expect(parsed?.snapshots[0].id).toBe("s1");
  });

  it("defaults missing timestamps to empty string", () => {
    const parsed = parseJobDetail(
      detail({ createdAt: undefined, updatedAt: undefined })
    );
    expect(parsed?.createdAt).toBe("");
    expect(parsed?.updatedAt).toBe("");
  });
});

describe("parseJobDetail url safety", () => {
  it("keeps https urls", () => {
    expect(parseJobDetail(detail())?.url).toBe("https://example.com/jobs/1");
  });

  it("keeps http urls", () => {
    expect(parseJobDetail(detail({ url: "http://example.com/x" }))?.url).toBe(
      "http://example.com/x"
    );
  });

  it("drops javascript urls", () => {
    expect(parseJobDetail(detail({ url: "javascript:alert(1)" }))?.url).toBeNull();
  });

  it("drops data urls", () => {
    expect(parseJobDetail(detail({ url: "data:text/html,hi" }))?.url).toBeNull();
  });

  it("drops malformed urls", () => {
    expect(parseJobDetail(detail({ url: "not a url" }))?.url).toBeNull();
  });
});

describe("stateFromError", () => {
  it("maps vault locked codes to locked", () => {
    expect(stateFromError("VAULT_LOCKED", "")).toBe("locked");
    expect(stateFromError("VAULT_UNLOCK_REQUIRED", "")).toBe("locked");
  });

  it("maps lock mentions in messages to locked", () => {
    expect(stateFromError(undefined, "vault must be unlocked")).toBe("locked");
  });

  it("maps NOT_IMPLEMENTED", () => {
    expect(stateFromError("NOT_IMPLEMENTED", "")).toBe("not-implemented");
  });

  it("maps companion connection failures to unavailable", () => {
    expect(
      stateFromError("COMPANION_UNAVAILABLE", "Native host has exited.")
    ).toBe("unavailable");
    expect(
      stateFromError("X", "Access to the specified native messaging host is forbidden")
    ).toBe("unavailable");
  });

  it("maps NOT_FOUND to not-found", () => {
    expect(stateFromError("NOT_FOUND", "")).toBe("not-found");
    expect(stateFromError(undefined, "resource not found")).toBe("not-found");
  });

  it("falls back to error", () => {
    expect(stateFromError("SOMETHING_ELSE", "boom")).toBe("error");
  });
});

describe("resolveDetailState", () => {
  it("detail-ready when detail present", () => {
    const state = resolveDetailState(successResponse(detail()), parseJobDetail(detail()));
    expect(state).toBe("detail-ready");
  });

  it("not-found on success with null detail", () => {
    expect(resolveDetailState(successResponse(null), null)).toBe("not-found");
  });

  it("error response maps through error codes", () => {
    expect(resolveDetailState(errorResponse("VAULT_LOCKED", "Vault is locked"), null)).toBe("locked");
    expect(resolveDetailState(errorResponse("NOT_IMPLEMENTED", ""), null)).toBe("not-implemented");
    expect(resolveDetailState(errorResponse("NOT_FOUND", ""), null)).toBe("not-found");
  });

  it("garbage response is error", () => {
    expect(resolveDetailState(null, null)).toBe("error");
    expect(resolveDetailState("nonsense" as unknown as IpcResponse, null)).toBe("error");
  });

  it("detail-ready requires both success and detail", () => {
    expect(resolveDetailState(successResponse(null), null)).toBe("not-found");
    expect(resolveDetailState(errorResponse("X", ""), parseJobDetail(detail()))).toBe("error");
  });
});

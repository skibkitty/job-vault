import { describe, it, expect } from "vitest";
import {
  parseJobDetail,
  parseSnapshotDetail,
  sortSnapshots,
  resolveDetailState,
  stateFromError,
  SnapshotDetail,
} from "../src/ui/jobdetail";
import { IpcResponse } from "../src/types";

function snapshot(overrides: Partial<Record<string, unknown>> = {}): Record<string, unknown> {
  return {
    id: "snap-1",
    title: "Software Engineer",
    company: "Example Corp",
    location: "Berlin",
    url: "https://example.com/jobs/1",
    capturedAt: "2026-08-20T10:00:00Z",
    description: "Build things.",
    requirements: ["TypeScript"],
    responsibilities: ["Write code"],
    ...overrides,
  };
}

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
      snapshot({ id: "snap-1", capturedAt: "2026-08-20T10:00:00Z" }),
      snapshot({ id: "snap-2", capturedAt: "2026-08-19T09:00:00Z" }),
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

describe("parseSnapshotDetail", () => {
  it("parses a valid snapshot", () => {
    const parsed = parseSnapshotDetail(snapshot());
    expect(parsed).not.toBeNull();
    expect(parsed?.id).toBe("snap-1");
    expect(parsed?.title).toBe("Software Engineer");
    expect(parsed?.capturedAt).toBe("2026-08-20T10:00:00Z");
    expect(parsed?.company).toBe("Example Corp");
    expect(parsed?.location).toBe("Berlin");
    expect(parsed?.url).toBe("https://example.com/jobs/1");
    expect(parsed?.description).toBe("Build things.");
    expect(parsed?.requirements).toEqual(["TypeScript"]);
    expect(parsed?.responsibilities).toEqual(["Write code"]);
  });

  it("rejects missing id", () => {
    expect(parseSnapshotDetail(snapshot({ id: "" }))).toBeNull();
  });

  it("rejects missing title", () => {
    expect(parseSnapshotDetail(snapshot({ title: null }))).toBeNull();
  });

  it("rejects non-object payloads", () => {
    expect(parseSnapshotDetail("string")).toBeNull();
    expect(parseSnapshotDetail(42)).toBeNull();
    expect(parseSnapshotDetail(null)).toBeNull();
  });

  it("defaults optional fields to null when blank", () => {
    const parsed = parseSnapshotDetail(
      snapshot({ company: "", location: "  ", salary: undefined })
    );
    expect(parsed?.company).toBeNull();
    expect(parsed?.location).toBeNull();
    expect(parsed?.salary).toBeNull();
  });

  it("defaults missing capturedAt to empty string", () => {
    const parsed = parseSnapshotDetail(snapshot({ capturedAt: undefined }));
    expect(parsed?.capturedAt).toBe("");
  });

  it("drops javascript urls", () => {
    expect(parseSnapshotDetail(snapshot({ url: "javascript:alert(1)" }))?.url).toBeNull();
  });

  it("keeps http and https urls", () => {
    expect(parseSnapshotDetail(snapshot({ url: "http://example.com/x" }))?.url).toBe(
      "http://example.com/x"
    );
    expect(parseSnapshotDetail(snapshot())?.url).toBe("https://example.com/jobs/1");
  });

  it("filters non-string array entries", () => {
    const parsed = parseSnapshotDetail(
      snapshot({ requirements: ["TypeScript", 123, null] })
    );
    expect(parsed?.requirements).toEqual(["TypeScript"]);
  });
});

describe("sortSnapshots", () => {
  it("sorts newest first by capturedAt", () => {
    const snaps: SnapshotDetail[] = [
      { id: "old", title: "A", capturedAt: "2026-08-01T00:00:00Z", company: null, location: null, url: null, salary: null, employmentType: null, description: "", requirements: [], responsibilities: [] },
      { id: "new", title: "A", capturedAt: "2026-08-20T00:00:00Z", company: null, location: null, url: null, salary: null, employmentType: null, description: "", requirements: [], responsibilities: [] },
    ];
    const sorted = sortSnapshots(snaps);
    expect(sorted.map((s) => s.id)).toEqual(["new", "old"]);
  });

  it("breaks ties by id", () => {
    const snaps: SnapshotDetail[] = [
      { id: "b", title: "A", capturedAt: "2026-08-01T00:00:00Z", company: null, location: null, url: null, salary: null, employmentType: null, description: "", requirements: [], responsibilities: [] },
      { id: "a", title: "A", capturedAt: "2026-08-01T00:00:00Z", company: null, location: null, url: null, salary: null, employmentType: null, description: "", requirements: [], responsibilities: [] },
    ];
    const sorted = sortSnapshots(snaps);
    expect(sorted.map((s) => s.id)).toEqual(["a", "b"]);
  });

  it("sorts empty capturedAt last", () => {
    const snaps: SnapshotDetail[] = [
      { id: "empty", title: "A", capturedAt: "", company: null, location: null, url: null, salary: null, employmentType: null, description: "", requirements: [], responsibilities: [] },
      { id: "dated", title: "A", capturedAt: "2026-08-01T00:00:00Z", company: null, location: null, url: null, salary: null, employmentType: null, description: "", requirements: [], responsibilities: [] },
    ];
    const sorted = sortSnapshots(snaps);
    expect(sorted.map((s) => s.id)).toEqual(["dated", "empty"]);
  });
});

describe("parseJobDetail", () => {
  it("parses a valid detail payload with snapshots", () => {
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
    expect(parsed?.snapshots[0].id).toBe("snap-1");
    expect(parsed?.snapshots[0].title).toBe("Software Engineer");
    expect(parsed?.snapshots[0].description).toBe("Build things.");
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
      detail({ snapshots: [
        snapshot({ id: "s1", capturedAt: "2026-01-01" }),
        "bad",
        snapshot({ id: "" }),
      ] })
    );
    expect(parsed?.snapshots).toHaveLength(1);
    expect(parsed?.snapshots[0].id).toBe("s1");
  });

  it("parses snapshot details with full fields", () => {
    const parsed = parseJobDetail(
      detail({
        snapshots: [
          snapshot({
            id: "s1",
            title: "Updated Role",
            company: "New Corp",
            description: "New description.",
            requirements: ["Python"],
            responsibilities: ["Lead team"],
          }),
        ],
      })
    );
    expect(parsed?.snapshots).toHaveLength(1);
    const snap = parsed?.snapshots[0];
    expect(snap?.title).toBe("Updated Role");
    expect(snap?.company).toBe("New Corp");
    expect(snap?.description).toBe("New description.");
    expect(snap?.requirements).toEqual(["Python"]);
    expect(snap?.responsibilities).toEqual(["Lead team"]);
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

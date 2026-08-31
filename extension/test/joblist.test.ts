import { describe, it, expect } from "vitest";
import {
  filterJobs,
  isRecord,
  parseJobList,
  parseJobSummary,
  resolveListState,
  sortJobs,
  stateFromError,
} from "../src/ui/joblist";
import { IpcResponse } from "../src/types";

function job(overrides: Partial<Record<string, unknown>> = {}): Record<string, unknown> {
  return {
    id: "job-1",
    title: "Software Engineer",
    company: "Example Corp",
    location: "Berlin",
    url: "https://example.com/jobs/1",
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

describe("isRecord", () => {
  it("accepts plain objects", () => {
    expect(isRecord({ a: 1 })).toBe(true);
  });

  it("rejects arrays, null, and primitives", () => {
    expect(isRecord([1, 2])).toBe(false);
    expect(isRecord(null)).toBe(false);
    expect(isRecord("text")).toBe(false);
    expect(isRecord(42)).toBe(false);
    expect(isRecord(undefined)).toBe(false);
  });
});

describe("parseJobSummary", () => {
  it("parses a valid job", () => {
    const parsed = parseJobSummary(job());
    expect(parsed).not.toBeNull();
    expect(parsed?.id).toBe("job-1");
    expect(parsed?.title).toBe("Software Engineer");
    expect(parsed?.company).toBe("Example Corp");
    expect(parsed?.url).toBe("https://example.com/jobs/1");
  });

  it("rejects missing id", () => {
    expect(parseJobSummary(job({ id: "" }))).toBeNull();
  });

  it("rejects missing title", () => {
    expect(parseJobSummary(job({ title: null }))).toBeNull();
  });

  it("rejects non-object entries", () => {
    expect(parseJobSummary("nope")).toBeNull();
    expect(parseJobSummary(42)).toBeNull();
    expect(parseJobSummary(null)).toBeNull();
  });

  it("treats blank optional fields as null", () => {
    const parsed = parseJobSummary(
      job({ company: "", location: "   ", url: undefined })
    );
    expect(parsed?.company).toBeNull();
    expect(parsed?.location).toBeNull();
    expect(parsed?.url).toBeNull();
  });

  it("defaults missing updatedAt to empty string", () => {
    const parsed = parseJobSummary(job({ updatedAt: undefined }));
    expect(parsed?.updatedAt).toBe("");
  });
});

describe("parseJobSummary url safety", () => {
  it("keeps https urls", () => {
    expect(parseJobSummary(job())?.url).toBe("https://example.com/jobs/1");
  });

  it("keeps http urls", () => {
    expect(parseJobSummary(job({ url: "http://example.com/x" }))?.url).toBe(
      "http://example.com/x"
    );
  });

  it("drops javascript urls", () => {
    expect(parseJobSummary(job({ url: "javascript:alert(1)" }))?.url).toBeNull();
  });

  it("drops data urls", () => {
    expect(parseJobSummary(job({ url: "data:text/html,hi" }))?.url).toBeNull();
  });

  it("drops malformed urls", () => {
    expect(parseJobSummary(job({ url: "not a url" }))?.url).toBeNull();
  });
});

describe("parseJobList", () => {
  it("parses an array of valid jobs", () => {
    const { jobs, invalidCount } = parseJobList([job(), job({ id: "job-2" })]);
    expect(jobs.length).toBe(2);
    expect(invalidCount).toBe(0);
  });

  it("skips malformed entries and counts them", () => {
    const { jobs, invalidCount } = parseJobList([
      job(),
      "garbage",
      job({ title: "" }),
      123,
    ]);
    expect(jobs.length).toBe(1);
    expect(invalidCount).toBe(3);
  });

  it("returns empty for non-array data", () => {
    expect(parseJobList(null)).toEqual({ jobs: [], invalidCount: 0 });
    expect(parseJobList("array? no")).toEqual({ jobs: [], invalidCount: 0 });
  });
});

describe("filterJobs", () => {
  const jobs = [
    parseJobSummary(job()),
    parseJobSummary(
      job({ id: "job-2", title: "Data Analyst", company: "Other Inc", location: "Paris" })
    ),
  ] as const;

  it("matches title case-insensitively", () => {
    const result = filterJobs([...jobs], "software");
    expect(result.length).toBe(1);
    expect(result[0].id).toBe("job-1");
  });

  it("matches company", () => {
    const result = filterJobs([...jobs], "other inc");
    expect(result.length).toBe(1);
    expect(result[0].id).toBe("job-2");
  });

  it("matches location", () => {
    const result = filterJobs([...jobs], "paris");
    expect(result.length).toBe(1);
  });

  it("returns all jobs for blank query", () => {
    expect(filterJobs([...jobs], "   ").length).toBe(2);
  });

  it("returns nothing when nothing matches", () => {
    expect(filterJobs([...jobs], "zzz").length).toBe(0);
  });
});

describe("sortJobs", () => {
  it("orders newest first", () => {
    const sorted = sortJobs([
      parseJobSummary(job({ id: "old", updatedAt: "2026-08-01T00:00:00Z" })),
      parseJobSummary(job({ id: "new", updatedAt: "2026-08-22T00:00:00Z" })),
    ]);
    expect(sorted.map((j) => j.id)).toEqual(["new", "old"]);
  });

  it("breaks ties by title deterministically", () => {
    const sorted = sortJobs([
      parseJobSummary(job({ id: "b", title: "Beta role", updatedAt: "2026-08-01T00:00:00Z" })),
      parseJobSummary(job({ id: "a", title: "Alpha role", updatedAt: "2026-08-01T00:00:00Z" })),
    ]);
    expect(sorted.map((j) => j.id)).toEqual(["a", "b"]);
  });

  it("sorts entries without updatedAt last", () => {
    const sorted = sortJobs([
      parseJobSummary(job({ id: "dated", updatedAt: "2026-08-01T00:00:00Z" })),
      parseJobSummary(job({ id: "undated", updatedAt: "" })),
    ]);
    expect(sorted.map((j) => j.id)).toEqual(["dated", "undated"]);
  });
});

describe("stateFromError", () => {
  it("maps vault locked codes to locked", () => {
    expect(stateFromError("VAULT_LOCKED", "")).toBe("locked");
  });

  it("maps lock mentions in messages to locked", () => {
    expect(stateFromError(undefined, "vault must be unlocked")).toBe("locked");
  });

  it("maps NOT_IMPLEMENTED", () => {
    expect(stateFromError("NOT_IMPLEMENTED", "")).toBe("not-implemented");
  });

  it("maps companion connection failures", () => {
    expect(
      stateFromError("COMPANION_UNAVAILABLE", "Native host has exited.")
    ).toBe("unavailable");
    expect(
      stateFromError("X", "Access to the specified native messaging host is forbidden")
    ).toBe("unavailable");
  });

  it("falls back to error", () => {
    expect(stateFromError("SOMETHING_ELSE", "boom")).toBe("error");
  });
});

describe("resolveListState", () => {
  it("ready when jobs present", () => {
    const state = resolveListState(successResponse([job()]), [
      parseJobSummary(job())!,
    ]);
    expect(state).toBe("ready");
  });

  it("empty on successful empty payload", () => {
    expect(resolveListState(successResponse([]), [])).toBe("empty");
  });

  it("error response maps through error codes", () => {
    const response: IpcResponse = {
      protocolVersion: 1,
      requestId: "r1",
      success: false,
      error: { code: "VAULT_LOCKED", message: "Vault is locked" },
    };
    expect(resolveListState(response, [])).toBe("locked");
  });

  it("garbage response is error", () => {
    expect(resolveListState(null, [])).toBe("error");
    expect(resolveListState("nonsense" as unknown as IpcResponse, [])).toBe(
      "error"
    );
  });
});

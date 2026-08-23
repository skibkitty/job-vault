import { IpcResponse } from "../types";

export interface JobSummary {
  id: string;
  title: string;
  company: string | null;
  location: string | null;
  url: string | null;
  updatedAt: string;
}

export type ListState =
  | "loading"
  | "ready"
  | "empty"
  | "locked"
  | "unavailable"
  | "not-implemented"
  | "error";

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function optionalText(value: unknown): string | null {
  if (typeof value === "string" && value.trim().length > 0) {
    return value;
  }
  return null;
}

function safeUrl(value: unknown): string | null {
  const text = optionalText(value);
  if (!text) return null;
  try {
    const parsed = new URL(text);
    if (parsed.protocol === "http:" || parsed.protocol === "https:") {
      return parsed.toString();
    }
  } catch {
    return null;
  }
  return null;
}

export function parseJobSummary(raw: unknown): JobSummary | null {
  if (!isRecord(raw)) return null;

  const id = optionalText(raw.id);
  const title = optionalText(raw.title);
  if (!id || !title) return null;

  return {
    id,
    title,
    company: optionalText(raw.company),
    location: optionalText(raw.location),
    url: safeUrl(raw.url),
    updatedAt:
      typeof raw.updatedAt === "string" && raw.updatedAt.length > 0
        ? raw.updatedAt
        : "",
  };
}

export function parseJobList(data: unknown): {
  jobs: JobSummary[];
  invalidCount: number;
} {
  if (!Array.isArray(data)) {
    return { jobs: [], invalidCount: 0 };
  }

  const jobs: JobSummary[] = [];
  let invalidCount = 0;

  for (const entry of data) {
    const parsed = parseJobSummary(entry);
    if (parsed) {
      jobs.push(parsed);
    } else {
      invalidCount += 1;
    }
  }

  return { jobs, invalidCount };
}

export function sortJobs(jobs: JobSummary[]): JobSummary[] {
  return [...jobs].sort((a, b) => {
    if (a.updatedAt !== b.updatedAt) {
      return a.updatedAt < b.updatedAt ? 1 : -1;
    }
    return a.title.localeCompare(b.title);
  });
}

export function filterJobs(jobs: JobSummary[], query: string): JobSummary[] {
  const needle = query.trim().toLowerCase();
  if (!needle) return jobs;

  return jobs.filter((job) =>
    [job.title, job.company ?? "", job.location ?? ""].some((field) =>
      field.toLowerCase().includes(needle)
    )
  );
}

const LOCKED_CODES = new Set(["VAULT_LOCKED", "VAULT_UNLOCK_REQUIRED"]);

export function stateFromError(code: string | undefined, message: string): ListState {
  const normalizedMessage = message.toLowerCase();

  if (code && LOCKED_CODES.has(code)) {
    return "locked";
  }
  if (normalizedMessage.includes("lock")) {
    return "locked";
  }
  if (
    code === "NOT_IMPLEMENTED" ||
    normalizedMessage.includes("not implemented")
  ) {
    return "not-implemented";
  }
  if (
    normalizedMessage.includes("native host") ||
    normalizedMessage.includes("native messaging") ||
    normalizedMessage.includes("companion") ||
    normalizedMessage.includes("connect") ||
    normalizedMessage.includes("not registered")
  ) {
    return "unavailable";
  }
  return "error";
}

export function resolveListState(
  response: IpcResponse | null,
  jobs: JobSummary[]
): ListState {
  if (!response || typeof response.success !== "boolean") {
    return "error";
  }
  if (response.success) {
    return jobs.length > 0 ? "ready" : "empty";
  }
  return stateFromError(response.error?.code, response.error?.message ?? "");
}

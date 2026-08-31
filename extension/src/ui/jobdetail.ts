import { IpcResponse } from "../types";
import { isRecord } from "./joblist";

export interface SnapshotSummary {
  id: string;
  capturedAt: string;
}

export interface SnapshotDetail extends SnapshotSummary {
  title: string;
  company: string | null;
  location: string | null;
  url: string | null;
  salary: string | null;
  employmentType: string | null;
  description: string;
  requirements: string[];
  responsibilities: string[];
}

export interface JobDetail {
  id: string;
  title: string;
  company: string | null;
  location: string | null;
  url: string | null;
  salary: string | null;
  employmentType: string | null;
  description: string;
  requirements: string[];
  responsibilities: string[];
  snapshots: SnapshotDetail[];
  createdAt: string;
  updatedAt: string;
}

export type DetailState =
  | "loading"
  | "detail-ready"
  | "not-found"
  | "locked"
  | "unavailable"
  | "not-implemented"
  | "error";

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

function parseStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value.filter((item): item is string => typeof item === "string" && item.trim().length > 0);
}

export function parseSnapshotDetail(raw: unknown): SnapshotDetail | null {
  if (!isRecord(raw)) return null;
  const id = optionalText(raw.id);
  const title = optionalText(raw.title);
  if (!id || !title) return null;

  return {
    id,
    title,
    capturedAt: typeof raw.capturedAt === "string" ? raw.capturedAt : "",
    company: optionalText(raw.company),
    location: optionalText(raw.location),
    url: safeUrl(raw.url),
    salary: optionalText(raw.salary),
    employmentType: optionalText(raw.employmentType),
    description:
      typeof raw.description === "string" ? raw.description.trim() : "",
    requirements: parseStringArray(raw.requirements),
    responsibilities: parseStringArray(raw.responsibilities),
  };
}

export function sortSnapshots(snapshots: SnapshotDetail[]): SnapshotDetail[] {
  return [...snapshots].sort((a, b) => {
    if (a.capturedAt !== b.capturedAt) {
      return a.capturedAt < b.capturedAt ? 1 : -1;
    }
    return a.id.localeCompare(b.id);
  });
}

export function parseJobDetail(raw: unknown): JobDetail | null {
  if (!isRecord(raw)) return null;

  const id = optionalText(raw.id);
  const title = optionalText(raw.title);
  if (!id || !title) return null;

  const description =
    typeof raw.description === "string" ? raw.description.trim() : "";

  const snapshots: SnapshotDetail[] = [];
  if (Array.isArray(raw.snapshots)) {
    for (const entry of raw.snapshots) {
      const parsed = parseSnapshotDetail(entry);
      if (parsed) snapshots.push(parsed);
    }
  }

  return {
    id,
    title,
    company: optionalText(raw.company),
    location: optionalText(raw.location),
    url: safeUrl(raw.url),
    salary: optionalText(raw.salary),
    employmentType: optionalText(raw.employmentType),
    description,
    requirements: parseStringArray(raw.requirements),
    responsibilities: parseStringArray(raw.responsibilities),
    snapshots,
    createdAt:
      typeof raw.createdAt === "string" && raw.createdAt.length > 0
        ? raw.createdAt
        : "",
    updatedAt:
      typeof raw.updatedAt === "string" && raw.updatedAt.length > 0
        ? raw.updatedAt
        : "",
  };
}

const LOCKED_CODES = new Set(["VAULT_LOCKED", "VAULT_UNLOCK_REQUIRED"]);

export function stateFromError(
  code: string | undefined,
  message: string
): DetailState {
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
  if (code === "NOT_FOUND" || normalizedMessage.includes("not found")) {
    return "not-found";
  }
  return "error";
}

export function resolveDetailState(
  response: IpcResponse | null,
  detail: JobDetail | null
): DetailState {
  if (!response || typeof response.success !== "boolean") {
    return "error";
  }
  if (response.success) {
    return detail ? "detail-ready" : "not-found";
  }
  return stateFromError(response.error?.code, response.error?.message ?? "");
}

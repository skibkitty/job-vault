import {
  filterJobs,
  JobSummary,
  ListState,
  parseJobList,
  resolveListState,
  sortJobs,
} from "./joblist";
import {
  DetailState,
  JobDetail,
  parseJobDetail,
  resolveDetailState,
} from "./jobdetail";

const statusEl = document.getElementById("status");
const listEl = document.getElementById("job-list");
const searchInput = document.getElementById("search") as HTMLInputElement | null;
const saveButton = document.getElementById("save");
const listView = document.getElementById("list-view");
const detailView = document.getElementById("detail-view");
const detailContent = document.getElementById("detail-content");
const backBtn = document.getElementById("back-btn");

let allJobs: JobSummary[] = [];
let invalidCount = 0;
let currentView: "list" | "detail" = "list";

const STATE_MESSAGES: Record<ListState, string> = {
  loading: "Loading jobs...",
  ready: "",
  empty: "No jobs saved yet. Open a job page and click Save Job.",
  locked: "Vault is locked. Unlock the companion to view jobs.",
  unavailable:
    "Companion not reachable. Make sure the local companion is running.",
  "not-implemented": "Job listing is not implemented in the companion yet.",
  error: "Could not load jobs.",
};

const DETAIL_STATE_MESSAGES: Record<DetailState, string> = {
  loading: "Loading job details...",
  "detail-ready": "",
  "not-found": "Job not found.",
  locked: "Vault is locked. Unlock the companion to view job details.",
  unavailable:
    "Companion not reachable. Make sure the local companion is running.",
  "not-implemented": "Job detail is not implemented in the companion yet.",
  error: "Could not load job details.",
};

function setStatus(text: string): void {
  if (statusEl) statusEl.textContent = text;
}

function setDetailStatus(text: string): void {
  if (detailContent) {
    const existing = detailContent.querySelector(".detail-status");
    if (existing) {
      existing.textContent = text;
    } else if (text) {
      const el = document.createElement("div");
      el.className = "detail-status";
      el.style.fontSize = "13px";
      el.style.color = "#666";
      el.textContent = text;
      detailContent.appendChild(el);
    }
  }
}

function showView(view: "list" | "detail"): void {
  currentView = view;
  if (listView) listView.style.display = view === "list" ? "block" : "none";
  if (detailView) detailView.style.display = view === "detail" ? "block" : "none";
}

function applyState(state: ListState): void {
  setStatus(STATE_MESSAGES[state]);
  if (listEl) listEl.textContent = "";
}

function createMetaLine(job: JobSummary): HTMLDivElement {
  const meta = document.createElement("div");
  meta.className = "job-meta";

  const parts = [job.company ?? "", job.location ?? ""].filter(
    (part) => part.length > 0
  );
  meta.textContent = parts.join(" \u00b7 ");
  return meta;
}

function createJobItem(job: JobSummary): HTMLLIElement {
  const item = document.createElement("li");
  item.className = "job-item";
  item.dataset.jobId = job.id;

  const titleLine = document.createElement("div");
  titleLine.className = "job-title";
  titleLine.textContent = job.title;

  item.appendChild(titleLine);
  item.appendChild(createMetaLine(job));
  return item;
}

function render(): void {
  if (!listEl) return;

  listEl.textContent = "";
  const visible = sortJobs(filterJobs(allJobs, searchInput?.value ?? ""));

  for (const job of visible) {
    const item = createJobItem(job);
    item.addEventListener("click", () => {
      void loadJobDetail(job.id);
    });
    listEl.appendChild(item);
  }
}

async function loadJobs(): Promise<void> {
  applyState("loading");

  try {
    const rawResponse: unknown = await chrome.runtime.sendMessage({
      type: "jobList",
      requestId: `popup-${Date.now()}`,
    });

    const response =
      rawResponse !== null && typeof rawResponse === "object"
        ? (rawResponse as { success?: unknown; data?: unknown; error?: unknown })
        : null;

    const parsed = parseJobList(response?.data ?? null);
    allJobs = parsed.jobs;
    invalidCount = parsed.invalidCount;

    const state = resolveListState(response as never, allJobs);
    applyState(state);

    if (invalidCount > 0 && state === "ready") {
      setStatus(`${invalidCount} malformed entr${invalidCount === 1 ? "y" : "ies"} skipped.`);
    }

    render();
  } catch (err) {
    applyState("unavailable");
    void err;
  }
}

function renderDetail(detail: JobDetail): void {
  if (!detailContent) return;
  detailContent.textContent = "";

  const title = document.createElement("h2");
  title.className = "detail-title";
  title.textContent = detail.title;
  detailContent.appendChild(title);

  const metaParts: string[] = [];
  if (detail.company) metaParts.push(detail.company);
  if (detail.location) metaParts.push(detail.location);
  if (detail.employmentType) metaParts.push(detail.employmentType);
  if (detail.salary) metaParts.push(detail.salary);

  const meta = document.createElement("div");
  meta.className = "detail-meta";
  meta.textContent = metaParts.join(" \u00b7 ");
  detailContent.appendChild(meta);

  if (detail.url) {
    const linkSection = document.createElement("div");
    linkSection.className = "detail-section";

    const link = document.createElement("a");
    link.href = detail.url;
    link.target = "_blank";
    link.rel = "noreferrer noopener";
    link.textContent = detail.url;
    linkSection.appendChild(link);
    detailContent.appendChild(linkSection);
  }

  if (detail.description) {
    const descSection = document.createElement("div");
    descSection.className = "detail-section";

    const descLabel = document.createElement("div");
    descLabel.className = "detail-section-label";
    descLabel.textContent = "Description";
    descSection.appendChild(descLabel);

    const descText = document.createElement("div");
    descText.className = "detail-text";
    descText.textContent = detail.description;
    descSection.appendChild(descText);
    detailContent.appendChild(descSection);
  }

  if (detail.requirements.length > 0) {
    const reqSection = document.createElement("div");
    reqSection.className = "detail-section";

    const reqLabel = document.createElement("div");
    reqLabel.className = "detail-section-label";
    reqLabel.textContent = "Requirements";
    reqSection.appendChild(reqLabel);

    const reqList = document.createElement("ul");
    reqList.className = "detail-list";
    for (const req of detail.requirements) {
      const li = document.createElement("li");
      li.textContent = req;
      reqList.appendChild(li);
    }
    reqSection.appendChild(reqList);
    detailContent.appendChild(reqSection);
  }

  if (detail.responsibilities.length > 0) {
    const respSection = document.createElement("div");
    respSection.className = "detail-section";

    const respLabel = document.createElement("div");
    respLabel.className = "detail-section-label";
    respLabel.textContent = "Responsibilities";
    respSection.appendChild(respLabel);

    const respList = document.createElement("ul");
    respList.className = "detail-list";
    for (const resp of detail.responsibilities) {
      const li = document.createElement("li");
      li.textContent = resp;
      respList.appendChild(li);
    }
    respSection.appendChild(respList);
    detailContent.appendChild(respSection);
  }

  if (detail.snapshots.length > 0) {
    const snapshotInfo = document.createElement("div");
    snapshotInfo.className = "detail-snapshot-info";
    const latest = detail.snapshots[0];
    const capturedLabel = latest?.capturedAt
      ? ` (latest: ${latest.capturedAt})`
      : "";
    snapshotInfo.textContent = `${detail.snapshots.length} snapshot${detail.snapshots.length === 1 ? "" : "s"}${capturedLabel}`;
    detailContent.appendChild(snapshotInfo);
  }
}

async function loadJobDetail(jobId: string): Promise<void> {
  showView("detail");
  if (detailContent) detailContent.textContent = "";
  setDetailStatus(DETAIL_STATE_MESSAGES.loading);

  try {
    const rawResponse: unknown = await chrome.runtime.sendMessage({
      type: "jobDetail",
      jobId,
      requestId: `popup-detail-${Date.now()}`,
    });

    const response =
      rawResponse !== null && typeof rawResponse === "object"
        ? (rawResponse as { success?: unknown; data?: unknown; error?: unknown })
        : null;

    const detail = parseJobDetail(response?.data ?? null);
    const state = resolveDetailState(response as never, detail);

    if (state === "detail-ready" && detail) {
      renderDetail(detail);
    } else {
      setDetailStatus(DETAIL_STATE_MESSAGES[state]);
    }
  } catch {
    setDetailStatus(DETAIL_STATE_MESSAGES.unavailable);
  }
}

backBtn?.addEventListener("click", () => {
  showView("list");
});

saveButton?.addEventListener("click", async () => {
  setStatus("Saving...");

  try {
    const tab = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab[0]?.id) {
      setStatus("No active tab found.");
      return;
    }

    await chrome.scripting.executeScript({
      target: { tabId: tab[0].id },
      files: ["src/content.js"],
    });

    setStatus("Job saved successfully.");
  } catch (err) {
    setStatus(`Error: ${err}`);
  }
});

searchInput?.addEventListener("input", render);

loadJobs();

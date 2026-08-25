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
  SnapshotDetail,
  parseJobDetail,
  resolveDetailState,
  sortSnapshots,
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
let currentDetail: JobDetail | null = null;
let viewingSnapshot = false;

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

function renderJobSection(
  parent: HTMLElement,
  label: string,
  items: string[]
): void {
  const section = document.createElement("div");
  section.className = "detail-section";

  const sectionLabel = document.createElement("div");
  sectionLabel.className = "detail-section-label";
  sectionLabel.textContent = label;
  section.appendChild(sectionLabel);

  const list = document.createElement("ul");
  list.className = "detail-list";
  for (const item of items) {
    const li = document.createElement("li");
    li.textContent = item;
    list.appendChild(li);
  }
  section.appendChild(list);
  parent.appendChild(section);
}

function renderJobMeta(
  parent: HTMLElement,
  detail: JobDetail | SnapshotDetail
): void {
  const metaParts: string[] = [];
  if (detail.company) metaParts.push(detail.company);
  if (detail.location) metaParts.push(detail.location);
  if ("employmentType" in detail && detail.employmentType) metaParts.push(detail.employmentType);
  if (detail.salary) metaParts.push(detail.salary);

  const meta = document.createElement("div");
  meta.className = "detail-meta";
  meta.textContent = metaParts.join(" \u00b7 ");
  parent.appendChild(meta);
}

function renderJobUrl(parent: HTMLElement, url: string | null): void {
  if (!url) return;
  const linkSection = document.createElement("div");
  linkSection.className = "detail-section";

  const link = document.createElement("a");
  link.href = url;
  link.target = "_blank";
  link.rel = "noreferrer noopener";
  link.textContent = url;
  linkSection.appendChild(link);
  parent.appendChild(linkSection);
}

function renderJobDescription(parent: HTMLElement, description: string): void {
  if (!description) return;
  const descSection = document.createElement("div");
  descSection.className = "detail-section";

  const descLabel = document.createElement("div");
  descLabel.className = "detail-section-label";
  descLabel.textContent = "Description";
  descSection.appendChild(descLabel);

  const descText = document.createElement("div");
  descText.className = "detail-text";
  descText.textContent = description;
  descSection.appendChild(descText);
  parent.appendChild(descSection);
}

function renderJobRequirements(parent: HTMLElement, items: string[]): void {
  if (items.length > 0) {
    renderJobSection(parent, "Requirements", items);
  }
}

function renderJobResponsibilities(parent: HTMLElement, items: string[]): void {
  if (items.length > 0) {
    renderJobSection(parent, "Responsibilities", items);
  }
}

function renderSnapshotList(
  parent: HTMLElement,
  snapshots: SnapshotDetail[]
): void {
  const sorted = sortSnapshots(snapshots);

  if (sorted.length === 0) {
    const empty = document.createElement("div");
    empty.className = "snapshot-empty";
    empty.textContent = "No snapshots recorded yet.";
    parent.appendChild(empty);
    return;
  }

  const section = document.createElement("div");
  section.className = "detail-section";

  const label = document.createElement("div");
  label.className = "detail-section-label";
  label.textContent = `Snapshots (${sorted.length})`;
  section.appendChild(label);

  const list = document.createElement("ul");
  list.className = "snapshot-list";

  for (const snap of sorted) {
    const item = document.createElement("li");
    item.className = "snapshot-item";
    item.dataset.snapshotId = snap.id;

    const dateSpan = document.createElement("span");
    dateSpan.className = "snapshot-item-date";
    dateSpan.textContent = snap.capturedAt || "unknown date";

    const idSpan = document.createElement("span");
    idSpan.className = "snapshot-item-id";
    idSpan.textContent = snap.id;

    item.appendChild(dateSpan);
    item.appendChild(idSpan);

    item.addEventListener("click", () => {
      renderSnapshotDetail(snap);
    });

    list.appendChild(item);
  }

  section.appendChild(list);
  parent.appendChild(section);
}

function renderSnapshotDetail(snapshot: SnapshotDetail): void {
  if (!detailContent) return;
  detailContent.textContent = "";
  viewingSnapshot = true;

  const backToJob = document.createElement("button");
  backToJob.className = "snapshot-back-btn";
  backToJob.textContent = "\u2190 Back to job";
  backToJob.addEventListener("click", () => {
    if (currentDetail) renderDetail(currentDetail);
  });
  detailContent.appendChild(backToJob);

  const title = document.createElement("h2");
  title.className = "detail-title";
  title.textContent = snapshot.title;
  detailContent.appendChild(title);

  const capLabel = document.createElement("div");
  capLabel.className = "detail-meta";
  capLabel.textContent = `Captured: ${snapshot.capturedAt || "unknown"}`;
  detailContent.appendChild(capLabel);

  renderJobMeta(detailContent, snapshot);
  renderJobUrl(detailContent, snapshot.url);
  renderJobDescription(detailContent, snapshot.description);
  renderJobRequirements(detailContent, snapshot.requirements);
  renderJobResponsibilities(detailContent, snapshot.responsibilities);
}

function renderDetail(detail: JobDetail): void {
  if (!detailContent) return;
  detailContent.textContent = "";
  viewingSnapshot = false;

  const title = document.createElement("h2");
  title.className = "detail-title";
  title.textContent = detail.title;
  detailContent.appendChild(title);

  renderJobMeta(detailContent, detail);
  renderJobUrl(detailContent, detail.url);
  renderJobDescription(detailContent, detail.description);
  renderJobRequirements(detailContent, detail.requirements);
  renderJobResponsibilities(detailContent, detail.responsibilities);
  renderSnapshotList(detailContent, detail.snapshots);
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
      currentDetail = detail;
      renderDetail(detail);
    } else {
      setDetailStatus(DETAIL_STATE_MESSAGES[state]);
    }
  } catch {
    setDetailStatus(DETAIL_STATE_MESSAGES.unavailable);
  }
}

backBtn?.addEventListener("click", () => {
  if (viewingSnapshot && currentDetail) {
    renderDetail(currentDetail);
  } else {
    showView("list");
  }
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

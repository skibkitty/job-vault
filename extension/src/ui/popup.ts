import {
  filterJobs,
  JobSummary,
  ListState,
  parseJobList,
  resolveListState,
  sortJobs,
} from "./joblist";

const statusEl = document.getElementById("status");
const listEl = document.getElementById("job-list");
const searchInput = document.getElementById("search") as HTMLInputElement | null;
const saveButton = document.getElementById("save");

let allJobs: JobSummary[] = [];
let invalidCount = 0;

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

function setStatus(text: string): void {
  if (statusEl) statusEl.textContent = text;
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
  meta.textContent = parts.join(" · ");
  return meta;
}

function createJobItem(job: JobSummary): HTMLLIElement {
  const item = document.createElement("li");
  item.className = "job-item";

  const titleLine = document.createElement("div");
  titleLine.className = "job-title";

  if (job.url) {
    const link = document.createElement("a");
    link.href = job.url;
    link.target = "_blank";
    link.rel = "noreferrer noopener";
    link.textContent = job.title;
    titleLine.appendChild(link);
  } else {
    titleLine.textContent = job.title;
  }

  item.appendChild(titleLine);
  item.appendChild(createMetaLine(job));
  return item;
}

function render(): void {
  if (!listEl) return;

  listEl.textContent = "";
  const visible = sortJobs(filterJobs(allJobs, searchInput?.value ?? ""));

  for (const job of visible) {
    listEl.appendChild(createJobItem(job));
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

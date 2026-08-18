import { JobData } from "./types";

function extractJobFromPage(): JobData | null {
  const title =
    document.querySelector("h1")?.textContent?.trim() ?? "";
  const company =
    document.querySelector('[data-company], .company-name')?.textContent?.trim() ?? "";
  const location =
    document.querySelector('[data-location], .location')?.textContent?.trim() ?? "";
  const description =
    document.querySelector('[data-description], .job-description')?.textContent?.trim() ?? "";

  if (!title && !description) {
    return null;
  }

  return {
    title,
    company,
    location,
    description,
    url: window.location.href,
  };
}

export {};

declare global {
  interface Window {
    __jobVaultExtract?: () => JobData | null;
  }
}

window.__jobVaultExtract = extractJobFromPage;

const jobData = extractJobFromPage();
jobData;

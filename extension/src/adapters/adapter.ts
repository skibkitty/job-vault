import { Adapter, JobData } from "../types";

export class GenericAdapter implements Adapter {
  canHandle(_url: string): boolean {
    return true;
  }

  async extractJob(): Promise<JobData | null> {
    const title =
      document.querySelector("h1")?.textContent?.trim() ?? "";
    const description =
      document.querySelector("article, .job-description, main")
        ?.textContent?.trim() ?? "";

    if (!title && !description) {
      return null;
    }

    return {
      title,
      company: "",
      location: "",
      description,
      url: window.location.href,
    };
  }
}

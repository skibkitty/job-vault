import { IpcRequest, IpcResponse } from "./types";

const COMPANION_NAME = "com.jobvault.companion";

let requestId = 0;

function generateRequestId(): string {
  requestId += 1;
  return `ext-${requestId}-${Date.now()}`;
}

export async function sendToCompanion(
  operation: string,
  payload: unknown
): Promise<IpcResponse> {
  const request: IpcRequest = {
    protocolVersion: 1,
    requestId: generateRequestId(),
    operation,
    payload,
  };

  return new Promise((resolve, reject) => {
    chrome.runtime.sendNativeMessage(COMPANION_NAME, request, (response) => {
      if (chrome.runtime.lastError) {
        reject(new Error(chrome.runtime.lastError.message));
        return;
      }
      resolve(response as IpcResponse);
    });
  });
}

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (
    typeof message !== "object" ||
    message === null ||
    (message as { type?: unknown }).type !== "jobList"
  ) {
    return false;
  }

  const requestId =
    typeof (message as { requestId?: unknown }).requestId === "string"
      ? (message as { requestId: string }).requestId
      : `bg-${Date.now()}`;

  sendToCompanion("job.list", {})
    .then(sendResponse)
    .catch((err: unknown) => {
      sendResponse({
        protocolVersion: 1,
        requestId,
        success: false,
        error: {
          code: "COMPANION_UNAVAILABLE",
          message: err instanceof Error ? err.message : String(err),
        },
      });
    });

  return true;
});

chrome.action.onClicked.addListener(async (tab) => {
  if (!tab.id) return;

  try {
    const results = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      files: ["src/content.js"],
    });

    if (results && results[0]?.result) {
      const jobData = results[0].result;
      const response = await sendToCompanion("job.save", { job: jobData });

      if (response.success) {
        console.log("Job saved successfully");
      } else {
        console.error("Failed to save job:", response.error);
      }
    }
  } catch (err) {
    console.error("Error extracting job:", err);
  }
});

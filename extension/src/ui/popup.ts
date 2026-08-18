const saveButton = document.getElementById("save");
const statusEl = document.getElementById("status");

saveButton?.addEventListener("click", async () => {
  if (statusEl) statusEl.textContent = "Saving...";

  try {
    const tab = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab[0]?.id) {
      if (statusEl) statusEl.textContent = "No active tab found.";
      return;
    }

    await chrome.scripting.executeScript({
      target: { tabId: tab[0].id },
      files: ["src/content.js"],
    });

    if (statusEl) statusEl.textContent = "Job saved successfully.";
  } catch (err) {
    if (statusEl) statusEl.textContent = `Error: ${err}`;
  }
});

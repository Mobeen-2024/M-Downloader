// Listen for new downloads and intercept them.
chrome.downloads.onCreated.addListener((downloadItem) => {
  // Ignore downloads that don't have a URL (internal chrome things)
  if (!downloadItem.url) return;

  // ── Step 1: Handshake with Native Host ─────────────────────────────
  // Send the URL and other info to the Mdownloader helper.exe
  chrome.runtime.sendNativeMessage(
    "com.mdownloader.app",
    { url: downloadItem.url, filename: downloadItem.filename },
    (response) => {
      if (chrome.runtime.lastError) {
        console.error("Native Messaging Error:", chrome.runtime.lastError.message);
        return;
      }

      if (response && response.status === "success") {
        // ── Step 2: Cancel the browser's default download ────────────────
        // Once Mdownloader has it, the browser doesn't need it.
        chrome.downloads.cancel(downloadItem.id, () => {
          console.log("Successfully intercepted download via Mdownloader.");
        });
      }
    }
  );
});

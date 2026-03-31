// ─────────────────────────────────────────────────────────────────────────────
// Mdownloader Companion — Background Service Worker (MV3)
//
// Intercepts browser downloads that match high-priority file types,
// cancels the browser's default handler, and forwards the metadata
// to the native Mdownloader engine via Native Messaging.
// ─────────────────────────────────────────────────────────────────────────────

const HOST_NAME = "com.mdownloader.host";

// ── Intercept Configuration ──────────────────────────────────────────────────

const DEFAULT_EXTENSIONS = [
  // Archives & Installers
  ".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".xz", ".iso", ".dmg",
  ".exe", ".msi", ".deb", ".rpm", ".appimage",
  // Media
  ".mp4", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".webm",
  ".mp3", ".flac", ".aac", ".ogg", ".wav",
  // Documents & Images
  ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx",
  // Disk Images & Firmware
  ".img", ".bin", ".rom",
  // Large Data
  ".csv", ".sql", ".db", ".sqlite",
];

// Minimum file size (bytes) to intercept, if known. 0 = intercept all sizes.
const MIN_FILESIZE = 0;

// ── State ────────────────────────────────────────────────────────────────────

let isEnabled = true;
let customExtensions = [];

// Load settings from chrome.storage on startup.
chrome.storage.local.get(["mdownloader_enabled", "mdownloader_extensions"], (result) => {
  if (result.mdownloader_enabled !== undefined) {
    isEnabled = result.mdownloader_enabled;
  }
  if (result.mdownloader_extensions) {
    customExtensions = result.mdownloader_extensions;
  }
});

// Listen for settings changes from the popup.
chrome.storage.onChanged.addListener((changes, area) => {
  if (area !== "local") return;
  if (changes.mdownloader_enabled) {
    isEnabled = changes.mdownloader_enabled.newValue;
  }
  if (changes.mdownloader_extensions) {
    customExtensions = changes.mdownloader_extensions.newValue || [];
  }
});

// ── Helpers ──────────────────────────────────────────────────────────────────

/**
 * Returns true if the URL or filename matches a capturable extension.
 */
function shouldCapture(url, filename) {
  const allExts = [...DEFAULT_EXTENSIONS, ...customExtensions];
  const target = (filename || url || "").toLowerCase();

  return allExts.some((ext) => target.endsWith(ext));
}

/**
 * Extracts cookies for a given URL via the chrome.cookies API.
 * Returns a semicolon-separated cookie string.
 */
async function getCookiesForUrl(url) {
  return new Promise((resolve) => {
    try {
      chrome.cookies.getAll({ url }, (cookies) => {
        if (chrome.runtime.lastError || !cookies) {
          resolve("");
          return;
        }
        const cookieStr = cookies.map((c) => `${c.name}=${c.value}`).join("; ");
        resolve(cookieStr);
      });
    } catch {
      resolve("");
    }
  });
}

/**
 * Gets the referrer URL of the active tab.
 */
async function getReferrer() {
  return new Promise((resolve) => {
    try {
      chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
        if (chrome.runtime.lastError || !tabs || !tabs[0]) {
          resolve("");
          return;
        }
        resolve(tabs[0].url || "");
      });
    } catch {
      resolve("");
    }
  });
}

// ── Download Interception ────────────────────────────────────────────────────

chrome.downloads.onCreated.addListener(async (downloadItem) => {
  // Guard: extension disabled or no URL.
  if (!isEnabled || !downloadItem.url) return;

  // Guard: skip blob/data/chrome URLs.
  if (
    downloadItem.url.startsWith("blob:") ||
    downloadItem.url.startsWith("data:") ||
    downloadItem.url.startsWith("chrome:")
  ) {
    return;
  }

  // Guard: file type not in our capture list.
  if (!shouldCapture(downloadItem.url, downloadItem.filename)) return;

  // Guard: file too small (if size is known and threshold is set).
  if (
    MIN_FILESIZE > 0 &&
    downloadItem.fileSize &&
    downloadItem.fileSize < MIN_FILESIZE
  ) {
    return;
  }

  // ── Gather enriched metadata ───────────────────────────────────────────
  const [cookies, referrer] = await Promise.all([
    getCookiesForUrl(downloadItem.url),
    getReferrer(),
  ]);

  const message = {
    url: downloadItem.url,
    filename: downloadItem.filename || null,
    mime: downloadItem.mime || null,
    filesize: downloadItem.totalBytes || downloadItem.fileSize || null,
    referrer: referrer || downloadItem.referrer || null,
    cookies: cookies || null,
  };

  // ── Send to native host ────────────────────────────────────────────────
  chrome.runtime.sendNativeMessage(HOST_NAME, message, (response) => {
    if (chrome.runtime.lastError) {
      console.error(
        "[Mdownloader] Native host error:",
        chrome.runtime.lastError.message
      );
      // Don't cancel the download — let the browser handle it as fallback.
      updateBadge("ERR", "#e74c3c");
      return;
    }

    if (response && response.status === "success") {
      // Cancel the browser's download — Mdownloader has it now.
      chrome.downloads.cancel(downloadItem.id, () => {
        // Also remove from Chrome's download bar.
        chrome.downloads.erase({ id: downloadItem.id });
        console.log("[Mdownloader] Captured:", downloadItem.url);
        updateBadge("✓", "#2ecc71");

        // Clear badge after a few seconds.
        setTimeout(() => updateBadge("", ""), 3000);
      });
    } else {
      console.warn("[Mdownloader] Host returned non-success:", response);
      updateBadge("ERR", "#e74c3c");
    }
  });
});

// ── Badge Indicator ──────────────────────────────────────────────────────────

function updateBadge(text, color) {
  chrome.action.setBadgeText({ text });
  if (color) {
    chrome.action.setBadgeBackgroundColor({ color });
  }
}

// ── Message Handler (from popup) ─────────────────────────────────────────────

chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (msg.type === "GET_STATUS") {
    sendResponse({ enabled: isEnabled });
  }
  return true; // async response
});

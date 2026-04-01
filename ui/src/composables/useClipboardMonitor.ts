import { ref, onUnmounted } from 'vue';

export function useClipboardMonitor() {
  const isEnabled = ref(false);
  const detectedUrl = ref<string | null>(null);
  let intervalId: any = null;

  // Common download extensions to filter for
  const downloadPatterns = [
    /\.zip$/i, /\.rar$/i, /\.7z$/i, /\.iso$/i, /\.exe$/i, /\.msi$/i,
    /\.mp4$/i, /\.mkv$/i, /\.mp3$/i, /\.pdf$/i, /\.dmg$/i, /\.deb$/i
  ];

  const checkClipboard = async () => {
    if (!isEnabled.value) return;

    try {
      // Note: navigator.clipboard.readText() may require focus/permissions
      const text = await navigator.clipboard.readText();
      if (!text) return;

      const trimmed = text.trim();
      
      // Basic URL validation
      if (!trimmed.startsWith('http://') && !trimmed.startsWith('https://')) return;

      // Check if it matches a download pattern
      const isDownloadUrl = downloadPatterns.some(pattern => pattern.test(trimmed));
      
      if (isDownloadUrl && trimmed !== detectedUrl.value) {
        detectedUrl.value = trimmed;
        console.log('[Clipboard] Detected download URL:', trimmed);
      }
    } catch (e) {
      // Silently fail if clipboard access is denied or not focused
    }
  };

  const startMonitoring = () => {
    isEnabled.value = true;
    intervalId = setInterval(checkClipboard, 2000);
  };

  const stopMonitoring = () => {
    isEnabled.value = false;
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
    detectedUrl.value = null;
  };

  const clearDetected = () => {
    detectedUrl.value = null;
  };

  onUnmounted(() => {
    stopMonitoring();
  });

  return {
    isEnabled,
    detectedUrl,
    startMonitoring,
    stopMonitoring,
    clearDetected
  };
}

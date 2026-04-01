import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export const useSettingsStore = defineStore('settings', () => {
  const maxConnections = ref(8);
  const downloadDir = ref('downloads');
  const themeAccent = ref('blue');
  const monitorClipboard = ref(false);
  const bridgeEnabled = ref(true);

  // Load from localStorage
  const saved = localStorage.getItem('mdownloader_settings');
  if (saved) {
    try {
      const data = JSON.parse(saved);
      maxConnections.value = data.maxConnections || 8;
      downloadDir.value = data.downloadDir || 'downloads';
      themeAccent.value = data.themeAccent || 'blue';
      monitorClipboard.value = data.monitorClipboard ?? false;
      bridgeEnabled.value = data.bridgeEnabled ?? true;
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  // Persist
  watch([maxConnections, downloadDir, themeAccent, monitorClipboard, bridgeEnabled], () => {
    localStorage.setItem('mdownloader_settings', JSON.stringify({
      maxConnections: maxConnections.value,
      downloadDir: downloadDir.value,
      themeAccent: themeAccent.value,
      monitorClipboard: monitorClipboard.value,
      bridgeEnabled: bridgeEnabled.value,
    }));
  }, { deep: true });

  return {
    maxConnections,
    downloadDir,
    themeAccent,
    monitorClipboard,
    bridgeEnabled,
  };
});

import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export const useSettingsStore = defineStore('settings', () => {
  const maxConnections = ref(8);
  const downloadDir = ref('downloads');
  const themeAccent = ref('blue');

  // Load from localStorage
  const saved = localStorage.getItem('mdownloader_settings');
  if (saved) {
    try {
      const data = JSON.parse(saved);
      maxConnections.value = data.maxConnections || 8;
      downloadDir.value = data.downloadDir || 'downloads';
      themeAccent.value = data.themeAccent || 'blue';
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  // Persist
  watch([maxConnections, downloadDir, themeAccent], () => {
    localStorage.setItem('mdownloader_settings', JSON.stringify({
      maxConnections: maxConnections.value,
      downloadDir: downloadDir.value,
      themeAccent: themeAccent.value,
    }));
  }, { deep: true });

  return {
    maxConnections,
    downloadDir,
    themeAccent,
  };
});

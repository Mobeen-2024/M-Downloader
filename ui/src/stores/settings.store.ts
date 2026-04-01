import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export const useSettingsStore = defineStore('settings', () => {
  const maxConnections = ref(8);
  const downloadDir = ref('downloads');
  const themeAccent = ref('blue');
  const monitorClipboard = ref(false);
  const bridgeEnabled = ref(true);
  const enableSpeedLimit = ref(false);
  const maxDownloadSpeed = ref(1024); // KB/s

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
      enableSpeedLimit.value = data.enableSpeedLimit ?? false;
      maxDownloadSpeed.value = data.maxDownloadSpeed ?? 1024;
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  // Persist
  watch([maxConnections, downloadDir, themeAccent, monitorClipboard, bridgeEnabled, enableSpeedLimit, maxDownloadSpeed], () => {
    localStorage.setItem('mdownloader_settings', JSON.stringify({
      maxConnections: maxConnections.value,
      downloadDir: downloadDir.value,
      themeAccent: themeAccent.value,
      monitorClipboard: monitorClipboard.value,
      bridgeEnabled: bridgeEnabled.value,
      enableSpeedLimit: enableSpeedLimit.value,
      maxDownloadSpeed: maxDownloadSpeed.value,
    }));
  }, { deep: true });

  // Sync with engine
  watch([enableSpeedLimit, maxDownloadSpeed], async () => {
    try {
      await invoke('update_download_settings', {
        enableSpeedLimit: enableSpeedLimit.value,
        maxSpeedKbps: maxDownloadSpeed.value,
      });
    } catch (e) {
      console.error('Failed to sync speed limit with engine:', e);
    }
  });

  return {
    maxConnections,
    downloadDir,
    themeAccent,
    monitorClipboard,
    bridgeEnabled,
    enableSpeedLimit,
    maxDownloadSpeed,
  };
});

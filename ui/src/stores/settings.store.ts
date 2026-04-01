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
  const snifferFilter = ref(['mp4', 'mp3', 'mkv', 'zip', 'exe', 'iso']);
  const cloudConfig = ref({
    provider: 'Google Drive',
    target_folder_id: '',
    enabled: false,
  });
  const cloudApiKey = ref('');

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
      snifferFilter.value = data.snifferFilter || ['mp4', 'mp3', 'mkv', 'zip', 'exe', 'iso'];
      if (data.cloudConfig) {
        cloudConfig.value = data.cloudConfig;
        // Migration: If api_key existed in local storage, move it to memory once
        if (data.cloudConfig.api_key) cloudApiKey.value = data.cloudConfig.api_key;
      }
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  // Persist
  watch([maxConnections, downloadDir, themeAccent, monitorClipboard, bridgeEnabled, enableSpeedLimit, maxDownloadSpeed, snifferFilter, cloudConfig], () => {
    localStorage.setItem('mdownloader_settings', JSON.stringify({
      maxConnections: maxConnections.value,
      downloadDir: downloadDir.value,
      themeAccent: themeAccent.value,
      monitorClipboard: monitorClipboard.value,
      bridgeEnabled: bridgeEnabled.value,
      enableSpeedLimit: enableSpeedLimit.value,
      maxDownloadSpeed: maxDownloadSpeed.value,
      snifferFilter: snifferFilter.value,
      cloudConfig: {
        ...cloudConfig.value,
        api_key: undefined // SECURE: Never persist to localStorage
      },
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

  // Sync cloud config with engine
  watch([cloudConfig, cloudApiKey], async () => {
    try {
      await invoke('update_cloud_config', { 
        config: cloudConfig.value,
        apiKey: cloudApiKey.value 
      });
    } catch (e) {
      console.error('Failed to sync cloud config with engine:', e);
    }
  }, { deep: true });

  return {
    maxConnections,
    downloadDir,
    themeAccent,
    monitorClipboard,
    bridgeEnabled,
    enableSpeedLimit,
    maxDownloadSpeed,
    snifferFilter,
    cloudConfig,
    cloudApiKey,
  };
});

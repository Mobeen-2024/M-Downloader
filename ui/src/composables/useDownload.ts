import { invoke } from '@tauri-apps/api/core';
import { useDownloadStore } from '../stores/download.store';
import { useUIStore } from '../stores/ui.store';

export function useDownload() {
  const store = useDownloadStore();
  const ui = useUIStore();

  const startDownload = async (url: string) => {
    try {
      const id = await invoke<string>('start_download', { url });
      store.addDownload(url, id);
      ui.success('Download started successfully');
      return id;
    } catch (e) {
      ui.error('Failed to start download');
      throw e;
    }
  };

  const pauseDownload = async (id: string) => {
    const index = store.downloads.findIndex(d => d.id === id);
    if (index === -1) return;

    const originalStatus = store.downloads[index].status;
    // Optimistic Update
    store.downloads[index].status = 'Paused';

    try {
      await invoke('pause_download', { id });
    } catch (e) {
      // Rollback
      if (store.downloads[index]) {
        store.downloads[index].status = originalStatus;
      }
      ui.error('Failed to pause download');
      console.error(e);
    }
  };

  const cancelDownload = async (id: string) => {
    const index = store.downloads.findIndex(d => d.id === id);
    if (index === -1) return;

    const originalDownload = { ...store.downloads[index] };
    // Optimistic Remove
    store.removeDownload(id);
    ui.info('Download cancelled');

    try {
      await invoke('cancel_download', { id });
    } catch (e) {
      // Rollback: Re-insert into store
      store.downloads.push(originalDownload);
      ui.error('Failed to cancel download');
    }
  };

  const resumeDownload = async (id: string) => {
    const index = store.downloads.findIndex(d => d.id === id);
    if (index === -1) return;

    const originalStatus = store.downloads[index].status;
    // Optimistic Update
    store.downloads[index].status = 'Downloading';

    try {
      await invoke('resume_download', { id });
    } catch (e) {
      // Rollback
      if (store.downloads[index]) {
        store.downloads[index].status = originalStatus;
      }
      ui.error('Failed to resume download');
    }
  };

  const refreshDownload = async (id: string, newUrl: string) => {
    try {
      await invoke('update_download_url', { id, newUrl });
      await resumeDownload(id);
      ui.success('Link refreshed and download resumed');
    } catch (e) {
      ui.error('Failed to refresh download link');
    }
  };

  const startRefreshMode = async (id: string) => {
    try {
      await invoke('start_refresh_mode', { id });
      ui.info('Refresh mode active: Sniff a new link in your browser');
    } catch (e) {
      ui.error('Failed to start refresh mode');
    }
  };

  const cancelRefreshMode = async () => {
    try {
      await invoke('cancel_refresh_mode');
    } catch (e) {
      console.error(e);
    }
  };

  return { 
    startDownload, 
    pauseDownload, 
    cancelDownload, 
    resumeDownload, 
    refreshDownload,
    startRefreshMode,
    cancelRefreshMode 
  };
}

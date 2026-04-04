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
      ui.error(`Failed to start download: ${typeof e === 'string' ? e : (e as Error).message}`);
      throw e;
    }
  };

  const pauseDownload = async (id: string) => {
    const download = store.downloads.find(d => d.id === id);
    if (!download) return;

    const originalStatus = download.status;
    
    // 1. Optimistic Update
    download.status = 'Paused';

    try {
      await invoke('pause_download', { id });
      ui.info(`Transmission ${id.slice(0, 8)} suspended`);
    } catch (e) {
      // 2. Rollback (Snap-Back)
      if (download) download.status = originalStatus;
      ui.error(`Suspension failed: ${typeof e === 'string' ? e : (e as Error).message}`);
    }
  };

  const cancelDownload = async (id: string) => {
    const index = store.downloads.findIndex(d => d.id === id);
    if (index === -1) return;

    const originalDownload = { ...store.downloads[index] };
    
    // 1. Optimistic Remove
    store.removeDownload(id);
    ui.info('Payload purged from buffer');

    try {
      await invoke('cancel_download', { id });
    } catch (e) {
      // 2. Rollback (Snap-Back)
      store.downloads.splice(index, 0, originalDownload);
      ui.error(`Purge failed: ${typeof e === 'string' ? e : (e as Error).message}`);
    }
  };

  const resumeDownload = async (id: string) => {
    const download = store.downloads.find(d => d.id === id);
    if (!download) return;

    const originalStatus = download.status;

    // 1. Optimistic Update
    download.status = 'Downloading';

    try {
      await invoke('resume_download', { id });
      ui.success(`Resuming acceleration for ${id.slice(0, 8)}`);
    } catch (e) {
      // 2. Rollback (Snap-Back)
      if (download) download.status = originalStatus;
      ui.error(`Acceleration failed: ${typeof e === 'string' ? e : (e as Error).message}`);
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

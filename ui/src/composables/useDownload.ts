import { invoke } from '@tauri-apps/api/core';
import { useDownloadStore } from '../stores/download.store';

export function useDownload() {
  const store = useDownloadStore();

  const startDownload = async (url: string) => {
    try {
      const id = await invoke<string>('start_download', { url });
      store.addDownload(url, id);
      return id;
    } catch (e) {
      console.error('Failed to start download:', e);
      throw e;
    }
  };

  const pauseDownload = async (id: string) => {
    try {
      await invoke('pause_download', { id });
    } catch (e) {
      console.error('Failed to pause download:', e);
    }
  };

  const cancelDownload = async (id: string) => {
    try {
      await invoke('cancel_download', { id });
      store.removeDownload(id);
    } catch (e) {
      console.error('Failed to cancel download:', e);
    }
  };

  const resumeDownload = async (id: string) => {
    try {
      await invoke('resume_download', { id });
      const index = store.downloads.findIndex(d => d.id === id);
      if (index !== -1) {
        store.downloads[index].status = 'Downloading';
      }
    } catch (e) {
      console.error('Failed to resume download:', e);
    }
  };

  return { startDownload, pauseDownload, cancelDownload, resumeDownload };
}

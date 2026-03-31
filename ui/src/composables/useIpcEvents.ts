import { listen } from '@tauri-apps/api/event';
import { onUnmounted } from 'vue';
import { useDownloadStore } from '../stores/download.store';
import type { DownloadProgressEvent } from '../types/download';

const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;

export function useIpcEvents() {
  const store = useDownloadStore();

  const initListeners = async () => {
    if (!isTauri) {
      console.log('Running in browser: skipped Tauri IPC listeners');
      return { unlistenProgress: () => {} };
    }

    const unlistenProgress = await listen<DownloadProgressEvent>('download-progress', (event) => {
      store.updateProgress(event.payload);
    });

    onUnmounted(() => {
      unlistenProgress();
    });

    return { unlistenProgress };
  };

  return { initListeners };
}

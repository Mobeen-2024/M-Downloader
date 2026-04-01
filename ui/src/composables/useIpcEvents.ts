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

    const unlistenBridge = await listen<boolean>('bridge-status-changed', (event) => {
      store.bridgeConnected = event.payload;
    });

    const unlistenMedia = await listen<any>('media-intercepted', (event) => {
      store.handleMediaIntercept(event.payload);
    });

    const unlistenRefresh = await listen<any>('url-refreshed', (event) => {
      if (event.payload.id) {
        // Since URL is refreshed internally, we might want to tell the store to update its ID's URL?
        // Note: The backend handles the swap, this is for UI notification.
        store.updateTaskUrl(event.payload.id, event.payload.new_url); 
      }
    });

    onUnmounted(() => {
      unlistenProgress();
      unlistenBridge();
      unlistenMedia();
      unlistenRefresh();
    });

    return { unlistenProgress, unlistenBridge, unlistenMedia, unlistenRefresh };
  };

  return { initListeners };
}

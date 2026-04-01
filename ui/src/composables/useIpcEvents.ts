import { listen } from '@tauri-apps/api/event';
import { onUnmounted } from 'vue';
import { useDownloadStore } from '../stores/download.store';
import { useUIStore } from '../stores/ui.store';
import type { DownloadProgressEvent } from '../types/download';

const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;

export function useIpcEvents() {
  const store = useDownloadStore();
  const ui = useUIStore();

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
      if (event.payload) {
        ui.success('Browser bridge connected');
      } else {
        ui.warn('Browser bridge disconnected');
      }
    });

    const unlistenMedia = await listen<any>('media-intercepted', (event) => {
      store.handleMediaIntercept(event.payload);
      ui.info(`Media Detected: ${event.payload.name || 'New stream found'}`);
    });

    const unlistenAnalyzing = await listen<any>('media-analyzing', () => {
      store.handleMediaAnalyzing();
    });

    const unlistenRefresh = await listen<any>('url-refreshed', (event) => {
      if (event.payload.id) {
        store.updateTaskUrl(event.payload.id, event.payload.new_url); 
        ui.success('Download URL refreshed successfully');
      }
    });

    onUnmounted(() => {
      unlistenProgress();
      unlistenBridge();
      unlistenMedia();
      unlistenAnalyzing();
      unlistenRefresh();
    });

    return { unlistenProgress, unlistenBridge, unlistenMedia, unlistenAnalyzing, unlistenRefresh };
  };

  return { initListeners };
}

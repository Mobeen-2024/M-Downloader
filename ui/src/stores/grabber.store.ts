import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useDownloadStore } from './download.store';

export interface GrabbedAsset {
  url: string;
  filename: string;
  mime: string;
  size: number;
  category: string;
  selected?: boolean;
}

export type DiscoveryScope = 'Folder' | 'SameDomain' | 'CrossSubdomain';

export interface GrabberTelemetry {
  active_workers: number;
  is_crawling: boolean;
  auth_active?: boolean;
  current_url?: string;
}

export const useGrabberStore = defineStore('grabber', () => {
  const assets = ref<GrabbedAsset[]>([]);
  const isCrawling = ref(false);
  const activeWorkers = ref(0);
  const authActive = ref(false);
  const currentUrl = ref('');
  const crawlUrl = ref('');
  const downloadStore = useDownloadStore();
  let unlistenAssets: UnlistenFn | null = null;
  let unlistenTelemetry: UnlistenFn | null = null;

  const setupEventListener = async () => {
    if (unlistenAssets) return;
    
    unlistenAssets = await listen<GrabbedAsset>('grabber-asset-found', (event) => {
      // Asset Sanitization //
      const rawName = event.payload.filename || 'unknown_asset';
      // Strip restricted filesystem characters and excessive length
      const sanitizedName = rawName
        .replace(/[<>:"/\\|?*]/g, '_')
        .substring(0, 255);

      if (!assets.value.some(a => a.url === event.payload.url)) {
        assets.value.push({ 
          ...event.payload, 
          filename: sanitizedName,
          selected: true 
        });
        // Live Category Hydration
        downloadStore.incrementCategoryCount(event.payload.category);
      }
    });

    unlistenTelemetry = await listen<GrabberTelemetry>('grabber-telemetry', (event) => {
      activeWorkers.value = event.payload.active_workers;
      isCrawling.value = event.payload.is_crawling;
      authActive.value = event.payload.auth_active || false;
      currentUrl.value = event.payload.current_url || '';
    });
  };

  const startCrawl = async (
    url: string, 
    maxDepth: number = 0, 
    scope: DiscoveryScope = 'SameDomain',
    externalLinkDepth: number = 0,
    useHeadless: boolean = false
  ) => {
    await setupEventListener();
    isCrawling.value = true;
    activeWorkers.value = 0;
    authActive.value = false;
    currentUrl.value = '';
    crawlUrl.value = url;
    assets.value = [];
    
    try {
      await invoke('start_grabber_crawl', { 
        url, 
        max_depth: maxDepth, 
        scope,
        external_link_depth: externalLinkDepth,
        use_headless: useHeadless
      });
    } catch (e) {
      console.error('Grabber Crawl Failed:', e);
      isCrawling.value = false;
    }
  };

  const stopCrawl = async () => {
      try {
          await invoke('stop_grabber_crawl');
          isCrawling.value = false;
      } catch (e) {
          console.error('Failed to stop crawl:', e);
      }
  };

  const selectedAssets = computed(() => assets.value.filter(a => a.selected));

  const bulkAddSelected = async (toQueue: boolean = true) => {
    const items = [...selectedAssets.value];
    for (const asset of items) {
      try {
        const id = await invoke('start_download_internal', { 
            url: asset.url,
            should_queue: toQueue 
        });
        
        if (toQueue) {
            downloadStore.add_to_queue(asset.url, id as string);
        } else {
            downloadStore.addDownload(asset.url, id as string);
        }
      } catch (e) {
        console.error(`Failed to add ${asset.filename}:`, e);
      }
    }
    // Clear selection after adding
    assets.value = assets.value.filter(a => !a.selected);
  };

  const toggleSelection = (url: string) => {
    const asset = assets.value.find(a => a.url === url);
    if (asset) asset.selected = !asset.selected;
  };

  const selectAll = (v: boolean) => {
    assets.value.forEach(a => a.selected = v);
  };

  const cleanup = () => {
    if (unlistenAssets) {
      unlistenAssets();
      unlistenAssets = null;
    }
    if (unlistenTelemetry) {
      unlistenTelemetry();
      unlistenTelemetry = null;
    }
  };

  return {
    assets,
    isCrawling,
    activeWorkers,
    authActive,
    currentUrl,
    crawlUrl,
    selectedAssets,
    startCrawl,
    stopCrawl,
    cleanup,
    bulkAddSelected,
    toggleSelection,
    selectAll,
  };
});

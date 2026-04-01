import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useDownloadStore } from './download.store';

export interface GrabbedAsset {
  url: string;
  filename: string;
  mime: string;
  size: number;
  category: string;
  selected?: boolean;
}

export const useGrabberStore = defineStore('grabber', () => {
  const assets = ref<GrabbedAsset[]>([]);
  const isCrawling = ref(false);
  const crawlUrl = ref('');
  const downloadStore = useDownloadStore();

  const startCrawl = async (url: string, depth: number = 0, strictDomain: boolean = true) => {
    isCrawling.value = true;
    crawlUrl.value = url;
    assets.value = [];
    
    try {
      const results: GrabbedAsset[] = await invoke('start_grabber_crawl', { 
        url, 
        depth, 
        strictDomain 
      });
      assets.value = results.map(a => ({ ...a, selected: true }));
    } catch (e) {
      console.error('Grabber Crawl Failed:', e);
    } finally {
      isCrawling.value = false;
    }
  };

  const selectedAssets = computed(() => assets.value.filter(a => a.selected));

  const bulkAddSelected = async (toQueue: boolean = true) => {
    for (const asset of selectedAssets.value) {
      try {
        const id = await invoke('start_download', { 
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
    assets.value = [];
  };

  const toggleSelection = (url: string) => {
    const asset = assets.value.find(a => a.url === url);
    if (asset) asset.selected = !asset.selected;
  };

  const selectAll = (v: boolean) => {
    assets.value.forEach(a => a.selected = v);
  };

  return {
    assets,
    isCrawling,
    crawlUrl,
    selectedAssets,
    startCrawl,
    bulkAddSelected,
    toggleSelection,
    selectAll,
  };
});

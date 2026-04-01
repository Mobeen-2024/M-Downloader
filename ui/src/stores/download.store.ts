import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { DownloadItem, DownloadProgressEvent } from '../types/download';

export const useDownloadStore = defineStore('downloads', () => {
  const downloads = ref<DownloadItem[]>([]);
  const bridgeConnected = ref(false);
  const interceptedMedia = ref<any[]>([]); // New: Stores detected media streams for HUD
  const isAnalyzingMedia = ref(false);
  
  // Load from localStorage on startup
  const saved = localStorage.getItem('mdownloader_tasks');
  if (saved) {
    try {
      downloads.value = JSON.parse(saved);
      // Reset all status to Paused on restart unless they were already Completed or Error
      downloads.value.forEach(d => {
        if (d.status === 'Downloading') d.status = 'Paused';
        d.speed_bps = 0;
      });
    } catch (e) {
      console.error('Failed to load saved downloads:', e);
    }
  }

  // Persist to localStorage
  watch(downloads, (newVal) => {
    localStorage.setItem('mdownloader_tasks', JSON.stringify(newVal));
  }, { deep: true });
  
  const activeDownloads = computed(() => 
    downloads.value.filter(d => d.status === 'Downloading')
  );
  
  const completedDownloads = computed(() => 
    downloads.value.filter(d => d.status === 'Completed')
  );

  const queuedDownloads = computed(() => 
    downloads.value.filter(d => d.status === 'Queued')
  );

  const totalSpeed = computed(() => {
    return downloads.value.reduce((acc, d) => acc + (d.status === 'Downloading' ? d.speed_bps : 0), 0);
  });

  const addDownload = (url: string, id: string) => {
    const name = url.split('/').pop() || 'unknown';
    const category = getCategoryFromFilename(name);
    downloads.value.push({
      id,
      url,
      name,
      downloaded: 0,
      total: 0,
      speed_bps: 0,
      status: 'Downloading',
      segments: [],
      addedAt: Date.now(),
      category,
    });
  };

  const queue_active = ref(false);

  const add_to_queue = async (url: string, id: string) => {
    const name = url.split('/').pop() || 'unknown';
    const category = getCategoryFromFilename(name);
    downloads.value.push({
      id,
      url,
      name,
      downloaded: 0,
      total: 0,
      speed_bps: 0,
      status: 'Queued',
      segments: [],
      addedAt: Date.now(),
      category,
    });
    
    try {
      await invoke('add_to_queue', { id });
    } catch (e) {
      console.error('Failed to add to queue:', e);
    }
  };

  const start_queue = async () => {
    try {
      await invoke('start_queue_scheduler');
      queue_active.value = true;
    } catch (e) {
      console.error('Failed to start queue:', e);
    }
  };

  const stop_queue = async () => {
    try {
      await invoke('stop_queue_scheduler');
      queue_active.value = false;
    } catch (e) {
      console.error('Failed to stop queue:', e);
    }
  };

  const updateProgress = (event: DownloadProgressEvent) => {
    const index = downloads.value.findIndex(d => d.id === event.id);
    if (index !== -1) {
      downloads.value[index] = { 
        ...downloads.value[index], 
        ...event 
      };
    }
  };

  const updateTaskUrl = (id: string, newUrl: string) => {
    const index = downloads.value.findIndex(d => d.id === id);
    if (index !== -1) {
      downloads.value[index].url = newUrl;
      console.log(`[Store] Task ${id} URL refreshed to: ${newUrl.substring(0, 50)}...`);
    }
  };

  const handleMediaAnalyzing = () => {
    isAnalyzingMedia.value = true;
  };

  const handleMediaIntercept = (media: any) => {
    isAnalyzingMedia.value = false;
    // Add to intercepted list if not already present
    if (!interceptedMedia.value.some(m => m.url === media.url)) {
      interceptedMedia.value.unshift(media);
      // Keep only last 10 detections
      if (interceptedMedia.value.length > 10) interceptedMedia.value.pop();
    }
  };

  const clearMediaDetection = (id: string) => {
    interceptedMedia.value = interceptedMedia.value.filter(m => m.id !== id);
  };

  const removeDownload = (id: string) => {
    downloads.value = downloads.value.filter(d => d.id !== id);
  };

  const getCategoryFromFilename = (filename: string): string => {
    const ext = filename.split('.').pop()?.toLowerCase() || '';
    if (['zip', 'rar', '7z', 'tar', 'gz', 'iso'].includes(ext)) return 'Compressed';
    if (['mp4', 'mkv', 'avi', 'mov', 'flv', 'wmv'].includes(ext)) return 'Video';
    if (['mp3', 'wav', 'flac', 'm4a', 'aac'].includes(ext)) return 'Music';
    if (['exe', 'msi', 'dmg', 'app', 'deb', 'rpm'].includes(ext)) return 'Programs';
    if (['pdf', 'doc', 'docx', 'txt', 'rtf', 'epub'].includes(ext)) return 'Documents';
    return 'General';
  };

  const downloadsByCategory = (category: string) => {
    if (category === 'All') return downloads.value;
    return downloads.value.filter(d => d.category === category);
  };

  const categoryCounts = computed(() => {
    const counts: Record<string, number> = {
      All: downloads.value.length,
      Compressed: 0,
      Video: 0,
      Music: 0,
      Programs: 0,
      Documents: 0,
      General: 0,
    };
    downloads.value.forEach(d => {
      if (counts[d.category!] !== undefined) {
        counts[d.category!]++;
      }
    });
    return counts;
  });

  return {
    downloads,
    activeDownloads,
    completedDownloads,
    queuedDownloads,
    totalSpeed,
    addDownload,
    updateProgress,
    removeDownload,
    downloadsByCategory,
    categoryCounts,
    bridgeConnected,
    queue_active,
    add_to_queue,
    start_queue,
    stop_queue,
    interceptedMedia,
    isAnalyzingMedia,
    updateTaskUrl,
    handleMediaAnalyzing,
    handleMediaIntercept,
    clearMediaDetection,
  };
});

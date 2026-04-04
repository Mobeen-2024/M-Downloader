import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { DownloadItem, DownloadProgressEvent } from '../types/download';

export const useDownloadStore = defineStore('downloads', () => {
  const downloads = ref<DownloadItem[]>([]);
  const bridgeConnected = ref(false);
  const interceptedMedia = ref<any[]>([]); // New: Stores detected media streams for HUD
  const isAnalyzingMedia = ref(false);
  
  // Site Grabber Discovery Counts (Hydrates Sidebar)
  const discoveryCategoryCounts = ref<Record<string, number>>({
    Compressed: 0,
    Video: 0,
    Music: 0,
    Programs: 0,
    Documents: 0,
    General: 0,
  });
  
  // Load from localStorage on startup (Safe Hydration)
  try {
    const saved = localStorage.getItem('mdownloader_tasks');
    if (saved) {
      const parsed = JSON.parse(saved);
      if (!Array.isArray(parsed)) {
        throw new Error('STORAGE_NOT_ARRAY');
      }
      
      downloads.value = parsed.map(d => {
        if (!d.id || !d.url) throw new Error('MISSING_CRITICAL_FIELDS');
        return {
          ...d,
          // Reset volatile state on boot
          status: (d.status === 'Downloading') ? 'Paused' : d.status,
          speed_bps: 0,
          segments: d.segments || [] // Ensure segments array exists
        };
      });
    }
  } catch (e) {
    console.error('CRITICAL_STORAGE_CORRUPTION: Purging mdownloader_tasks.', e);
    localStorage.removeItem('mdownloader_tasks'); // Self-heal by purging corrupt key
    downloads.value = [];
  }

  // Persistence Logic (Debounced to prevent main-thread freezing during high-speed I/O)
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  const saveToDisk = (immediate = false) => {
    if (immediate) {
      if (saveTimeout) clearTimeout(saveTimeout);
      localStorage.setItem('mdownloader_tasks', JSON.stringify(downloads.value));
      return;
    }
    if (saveTimeout) return;
    saveTimeout = setTimeout(() => {
      localStorage.setItem('mdownloader_tasks', JSON.stringify(downloads.value));
      saveTimeout = null;
    }, 2000); // Save at most every 2 seconds during active bursts
  };

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

  const getCategoryFromFilename = (filename: string): string => {
    const ext = filename.split('.').pop()?.toLowerCase() || '';
    if (['zip', 'rar', '7z', 'tar', 'gz', 'iso'].includes(ext)) return 'Compressed';
    if (['mp4', 'mkv', 'avi', 'mov', 'flv', 'wmv'].includes(ext)) return 'Video';
    if (['mp3', 'wav', 'flac', 'm4a', 'aac'].includes(ext)) return 'Music';
    if (['exe', 'msi', 'dmg', 'app', 'deb', 'rpm'].includes(ext)) return 'Programs';
    if (['pdf', 'doc', 'docx', 'txt', 'rtf', 'epub'].includes(ext)) return 'Documents';
    return 'General';
  };

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
    discoveryCategoryCounts.value[category]++;
    saveToDisk(true);
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
    saveToDisk(true);
    
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

  const move_up = async (id: string) => {
    const index = downloads.value.findIndex(d => d.id === id);
    if (index > 0) {
      // Local swap for immediate UI feedback
      const temp = downloads.value[index];
      downloads.value[index] = downloads.value[index - 1];
      downloads.value[index - 1] = temp;
      
      try {
        await invoke('move_queue_item_up', { id });
      } catch (e) {
        console.error('Failed to move item up:', e);
      }
    }
  };

  const move_down = async (id: string) => {
    const index = downloads.value.findIndex(d => d.id === id);
    if (index !== -1 && index < downloads.value.length - 1) {
      // Local swap for immediate UI feedback
      const temp = downloads.value[index];
      downloads.value[index] = downloads.value[index + 1];
      downloads.value[index + 1] = temp;

      try {
        await invoke('move_queue_item_down', { id });
      } catch (e) {
        console.error('Failed to move item down:', e);
      }
    }
  };

  // Throttled UI State Buffering (Prevention of Main Thread Freezes)
  const pendingUpdates = new Map<string, DownloadProgressEvent>();
  let throttleTimer: number | null = null;

  const flushUpdates = () => {
    if (pendingUpdates.size === 0) {
      throttleTimer = null;
      return;
    }
    
    pendingUpdates.forEach((event, id) => {
      const d = downloads.value.find(dl => dl.id === id);
      if (d) {
        Object.assign(d, event);
      }
    });
    pendingUpdates.clear();
    saveToDisk(); // Save state post-flush
    throttleTimer = null;
  };

  const updateProgress = (event: DownloadProgressEvent) => {
    // Critical state transitions bypass the throttle for instant UI feedback
    if (['Completed', 'Error', 'Paused'].includes(event.status)) {
      pendingUpdates.delete(event.id);
      const d = downloads.value.find(dl => dl.id === event.id);
      if (d) {
        Object.assign(d, event);
        saveToDisk();
      }
      return;
    }

    // High-frequency telemetry updates are buffered and flushed via requestAnimationFrame
    // Ensures Vue's deep proxy doesn't trigger CPU-blocking render loops by batching mapping to the refresh rate
    pendingUpdates.set(event.id, event);
    if (!throttleTimer) {
      throttleTimer = requestAnimationFrame(flushUpdates);
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
    saveToDisk(true);
  };

  const downloadsByCategory = (category: string) => {
    if (category === 'All') return downloads.value;
    return downloads.value.filter(d => d.category === category);
  };

  const incrementCategoryCount = (category: string) => {
    if (discoveryCategoryCounts.value[category] !== undefined) {
      discoveryCategoryCounts.value[category]++;
    } else {
      discoveryCategoryCounts.value.General++;
    }
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
      const cat = d.category || 'General';
      if (counts[cat] !== undefined) {
        counts[cat]++;
      } else {
        counts.General++;
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
    discoveryCategoryCounts,
    incrementCategoryCount,
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
    move_up,
    move_down,
    clear_completed: () => {
      downloads.value = downloads.value.filter(d => d.status !== 'Completed');
    },
  };
});

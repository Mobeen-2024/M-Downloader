import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import type { DownloadItem, DownloadProgressEvent } from '../types/download';

export const useDownloadStore = defineStore('downloads', () => {
  const downloads = ref<DownloadItem[]>([]);
  const bridgeConnected = ref(false);
  
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

  const updateProgress = (event: DownloadProgressEvent) => {
    const index = downloads.value.findIndex(d => d.id === event.id);
    if (index !== -1) {
      downloads.value[index] = { 
        ...downloads.value[index], 
        ...event 
      };
    }
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
  };
});

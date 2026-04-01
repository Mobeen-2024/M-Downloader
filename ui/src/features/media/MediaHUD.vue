<script setup lang="ts">
import { useDownloadStore } from '@/stores/download.store';
import { Video, Zap, Download, X } from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';

const store = useDownloadStore();

const handleDownload = async (media: any, resolution?: any) => {
  const url = resolution ? resolution.video_url : media.url;
  try {
    const id = await invoke('start_download', { 
      url, 
      cookies: media.cookies || null,
      referer: media.referrer || null 
    });
    store.addDownload(url, id as string);
    store.clearMediaDetection(media.id);
  } catch (e) {
    console.error('Failed to start media download:', e);
  }
};

const dismiss = (id: string) => {
  store.clearMediaDetection(id);
};
</script>

<template>
  <div class="media-hud-container">
    <TransitionGroup name="hud-slide">
      <div 
        v-for="media in store.interceptedMedia" 
        :key="media.url" 
        class="media-card glass-panel"
      >
        <div class="media-header">
          <div class="media-type">
            <Video :size="16" class="icon-pulse" />
            <span>Media Detected</span>
          </div>
          <button @click="dismiss(media.id)" class="close-btn"><X :size="14" /></button>
        </div>

        <div class="media-content">
          <h4 class="media-title">{{ media.filename }}</h4>
          
          <div v-if="media.resolutions && media.resolutions.length" class="resolution-list">
            <div 
              v-for="res in media.resolutions" 
              :key="res.label"
              class="resolution-item"
              @click="handleDownload(media, res)"
            >
              <Zap :size="12" />
              <span>{{ res.label }}</span>
              <Download :size="12" class="download-icon" />
            </div>
          </div>

          <button v-else @click="handleDownload(media)" class="quick-download">
            <Download :size="16" />
            Download Now
          </button>
        </div>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.media-hud-container {
  position: fixed;
  bottom: 24px;
  right: 24px;
  display: flex;
  flex-direction: column-reverse;
  gap: 16px;
  z-index: 9999;
  pointer-events: none;
}

.media-card {
  pointer-events: auto;
  width: 320px;
  border-radius: 20px;
  padding: 16px;
  border: 1px solid rgba(59, 130, 246, 0.3);
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
  overflow: hidden;
}

.media-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.media-type {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.75rem;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--accent-primary);
  letter-spacing: 0.05em;
}

.icon-pulse {
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.5; }
  100% { opacity: 1; }
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  opacity: 0.5;
  transition: opacity 0.2s;
}

.close-btn:hover {
  opacity: 1;
}

.media-title {
  margin: 0 0 12px 0;
  font-size: 0.95rem;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-primary);
}

.resolution-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.resolution-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 10px;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.resolution-item:hover {
  background: rgba(59, 130, 246, 0.1);
  border-color: rgba(59, 130, 246, 0.3);
  transform: translateX(4px);
}

.download-icon {
  margin-left: auto;
  opacity: 0;
  transform: translateY(2px);
  transition: all 0.2s;
}

.resolution-item:hover .download-icon {
  opacity: 1;
  transform: translateY(0);
}

.quick-download {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 10px;
  background: var(--accent-primary);
  color: white;
  border: none;
  border-radius: 12px;
  font-weight: 700;
  font-size: 0.9rem;
  cursor: pointer;
}

/* Transition Animations */
.hud-slide-enter-active, .hud-slide-leave-active {
  transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.hud-slide-enter-from {
  opacity: 0;
  transform: translateX(50px) scale(0.9);
}
.hud-slide-leave-to {
  opacity: 0;
  transform: scale(0.8);
}
</style>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

interface MediaInterceptEvent {
  url: String;
  filename: String | null;
  mime: String | null;
}

const show = ref(false);
const currentMedia = ref<MediaInterceptEvent | null>(null);
let unlisten: UnlistenFn | null = null;
let autoHideTimer: any = null;

onMounted(async () => {
  unlisten = await listen<MediaInterceptEvent>('media-intercepted', (event) => {
    currentMedia.value = event.payload;
    show.value = true;
    
    // Auto-hide after 15 seconds if ignored
    if (autoHideTimer) clearTimeout(autoHideTimer);
    autoHideTimer = setTimeout(() => {
      show.value = false;
    }, 15000);
  });
});

onUnmounted(() => {
  if (unlisten) unlisten();
  if (autoHideTimer) clearTimeout(autoHideTimer);
});

function clearTimer() {
  if (autoHideTimer) clearTimeout(autoHideTimer);
}

async function startMediaDownload() {
  if (!currentMedia.value) return;
  
  try {
    await invoke('start_download', { 
      url: currentMedia.value.url,
      cookies: null,
      referer: null
    });
    show.value = false;
  } catch (e) {
    console.error("Failed to start media download:", e);
  }
}
</script>

<template>
  <transition name="slide-up">
    <div v-if="show" class="media-hud" @mouseenter="clearTimer">
      <div class="hud-content">
        <div class="media-info">
          <div class="icon">
            <svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none" class="pulse">
              <path d="M23 7l-7 5 7 5V7zM1 5h14v14H1z" />
            </svg>
          </div>
          <div class="text">
            <h4>Video Stream Detected</h4>
            <p>{{ currentMedia?.filename || 'Adaptive Stream (HLS/DASH)' }}</p>
          </div>
        </div>
        <div class="hud-actions">
          <button class="btn-cancel" @click="show = false">Ignore</button>
          <button class="btn-download" @click="startMediaDownload">
            Download this Video
          </button>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.media-hud {
  position: fixed;
  top: 60px;
  right: 20px;
  z-index: 9999;
  width: 380px;
  background: rgba(var(--accent-rgb), 0.15);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.4);
  padding: 16px;
}

.hud-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.media-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: var(--accent-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.text h4 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  letter-spacing: 0.5px;
}

.text p {
  margin: 0;
  font-size: 0.85rem;
  opacity: 0.7;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 260px;
}

.hud-actions {
  display: flex;
  gap: 8px;
}

.btn-download {
  flex: 1;
  background: var(--accent-primary);
  color: white;
  border: none;
  padding: 10px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s;
}

.btn-download:hover {
  transform: scale(1.02);
  filter: brightness(1.1);
}

.btn-cancel {
  background: rgba(255, 255, 255, 0.05);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 10px 16px;
  border-radius: 8px;
  cursor: pointer;
}

.pulse {
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.5; }
  100% { opacity: 1; }
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(-20px);
  opacity: 0;
}
</style>

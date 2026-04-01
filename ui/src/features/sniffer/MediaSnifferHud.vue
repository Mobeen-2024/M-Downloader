<script setup lang="ts">
import { ref, computed } from 'vue';
import { useDownloadStore } from '@/stores/download.store';
import { invoke } from '@tauri-apps/api/core';

const store = useDownloadStore();

const currentMedia = computed(() => store.interceptedMedia[0] || null);
const selectedResolution = ref<any>(null);
const isAudioOnly = ref(false);

// Auto-sync resolution selection when new media arrives
import { watch } from 'vue';
watch(currentMedia, (newVal) => {
  if (newVal && newVal.resolutions) {
    selectedResolution.value = newVal.resolutions[0];
  }
}, { immediate: true });

async function startMediaDownload() {
  if (!currentMedia.value || !selectedResolution.value) return;
  
  const m = currentMedia.value;
  const res = selectedResolution.value;
  
  try {
    const id = await invoke('start_download', { 
      url: isAudioOnly.value && res.audio_url 
        ? res.audio_url 
        : res.video_url,
      cookies: m.cookies || null,
      referer: m.referer || m.referrer || null
    });
    
    // Add to store and clear from HUD
    store.addDownload(m.url, id as string);
    store.clearMediaDetection(m.id);
  } catch (e) {
    console.error("Failed to start media download:", e);
  }
}

function dismiss() {
  if (currentMedia.value) {
    store.clearMediaDetection(currentMedia.value.id);
  }
}
</script>

<template>
  <transition name="slide-up">
    <div v-if="currentMedia" class="media-hud">
      <div class="hud-content">
        <div class="media-info">
          <div class="icon">
            <svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none" class="pulse">
              <path d="M23 7l-7 5 7 5V7zM1 5h14v14H1z" />
            </svg>
          </div>
          <div class="text">
            <h4>Video Stream Detected</h4>
            <div class="meta">
              <span class="mime">{{ currentMedia.mime || 'Unknown Codec' }}</span>
              <span class="file">{{ currentMedia.filename || 'Adaptive Stream' }}</span>
            </div>
          </div>
        </div>

        <div class="quality-selector" v-if="currentMedia.resolutions?.length">
          <label>Quality</label>
          <div class="resolutions">
            <button 
              v-for="res in currentMedia.resolutions" 
              :key="res.label"
              class="res-btn"
              :class="{ active: selectedResolution?.label === res.label }"
              @click="selectedResolution = res"
            >
              {{ res.label }}
            </button>
          </div>
        </div>

        <div class="options">
          <label class="checkbox-container">
            <input type="checkbox" v-model="isAudioOnly" :disabled="!selectedResolution?.audio_url" />
            <span class="checkmark"></span>
            Extract Audio Only
          </label>
        </div>

        <div class="hud-actions">
          <button class="btn-cancel" @click="dismiss">Ignore</button>
          <button class="btn-download" @click="startMediaDownload" :disabled="!selectedResolution">
            Download at {{ selectedResolution?.label || 'Original' }}
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

.quality-selector {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.quality-selector label, .options label {
  font-size: 0.65rem;
  text-transform: uppercase;
  font-weight: 700;
  color: var(--text-secondary);
}

.resolutions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.res-btn {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: white;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.res-btn.active {
  background: var(--accent-primary);
  border-color: var(--accent-primary);
  box-shadow: 0 0 10px rgba(var(--accent-rgb), 0.4);
}

.options {
  padding-top: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.checkbox-container {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
}

.checkbox-container input { display: none; }
.checkmark {
  width: 16px;
  height: 16px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  position: relative;
}

.checkbox-container input:checked + .checkmark {
  background: var(--accent-primary);
  border-color: var(--accent-primary);
}

.checkbox-container input:checked + .checkmark::after {
  content: "";
  position: absolute;
  left: 5px;
  top: 2px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.meta { display: flex; flex-direction: column; gap: 2px; }
.mime { font-size: 0.6rem; color: var(--accent-primary); font-weight: 700; }
.file { font-size: 0.85rem; opacity: 0.7; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 260px; }

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

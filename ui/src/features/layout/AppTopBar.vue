<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useDownloadStore } from '@/stores/download.store';
import { useSettingsStore } from '@/stores/settings.store';
import { useFormatters } from '@/composables/useFormatters';
import { invoke } from '@tauri-apps/api/core';
import SpeedGraph from '@/features/shared/components/SpeedGraph.vue';
import { Plus, Wifi, Activity, Zap, Link2, Clipboard } from 'lucide-vue-next';
import { useClipboardMonitor } from '@/composables/useClipboardMonitor';

const store = useDownloadStore();
const settings = useSettingsStore();
const { formatSpeed } = useFormatters();
const { isEnabled: isClipboardEnabled, detectedUrl, startMonitoring, stopMonitoring, clearDetected } = useClipboardMonitor();

const isLimitEnabled = ref(false);
const speedLimitKbps = ref(1024); // Default 1 MB/s
const depStatus = ref({ ffmpeg_found: false, ffprobe_found: false });

const checkDeps = async () => {
  try {
    depStatus.value = await invoke('check_dependencies');
  } catch (e) {
    console.error('Failed to check dependencies:', e);
  }
};

const updateLimits = async () => {
  const limitBps = isLimitEnabled.value ? speedLimitKbps.value * 1024 : null;
  for (const dl of store.downloads) {
    if (dl.status === 'Downloading') {
      await invoke('set_speed_limit', { id: dl.id, limit: limitBps });
    }
  }
};

watch([isLimitEnabled, speedLimitKbps], updateLimits);

const toggleClipboard = () => {
  settings.monitorClipboard = !settings.monitorClipboard;
};

// Sync clipboard monitor with setting
watch(() => settings.monitorClipboard, (newVal) => {
  if (newVal) startMonitoring();
  else stopMonitoring();
}, { immediate: true });

onMounted(() => {
  checkDeps();
});

defineEmits(['new-download']);
</script>

<template>
  <header class="top-bar glass-panel shadow-glass" data-tauri-drag-region>
    <div class="stats-group" data-tauri-drag-region>
      <div class="stat-item">
        <Activity class="stat-icon text-accent" />
        <div class="stat-details">
          <div class="stat-label">Total Speed</div>
          <div class="stat-value">{{ formatSpeed(store.totalSpeed) }}</div>
        </div>
        <SpeedGraph :current-speed="store.totalSpeed" class="top-bar-graph" />
      </div>

      <div class="stat-divider"></div>

      <div class="stat-item">
        <Wifi class="stat-icon" />
        <div class="stat-details">
          <div class="stat-label">Active</div>
          <div class="stat-value">{{ store.activeDownloads.length }}</div>
        </div>
      </div>

      <div class="stat-divider"></div>

      <div class="stat-item">
        <Zap class="stat-icon" :class="{ 'engine-online': depStatus.ffmpeg_found, 'engine-offline': !depStatus.ffmpeg_found }" />
        <div class="stat-details">
          <div class="stat-label">Muxing Engine</div>
          <div class="stat-value" :class="{ 'text-online': depStatus.ffmpeg_found, 'text-offline': !depStatus.ffmpeg_found }">
            {{ depStatus.ffmpeg_found ? 'ONLINE' : 'OFFLINE' }}
          </div>
        </div>
      </div>

      <div v-if="settings.bridgeEnabled" class="stat-divider"></div>

      <div v-if="settings.bridgeEnabled" class="stat-item">
        <Link2 class="stat-icon" :class="{ 'bridge-active': store.bridgeConnected, 'bridge-waiting': !store.bridgeConnected }" />
        <div class="stat-details">
          <div class="stat-label">Extension Bridge</div>
          <div class="stat-value" :class="{ 'text-online': store.bridgeConnected }">
            {{ store.bridgeConnected ? 'CONNECTED' : 'WAITING' }}
          </div>
        </div>
      </div>

      <div class="stat-divider"></div>

      <div class="stat-item clipboard-group" :class="{ active: isClipboardEnabled }">
        <div class="clipboard-toggle" @click="toggleClipboard">
          <Clipboard class="stat-icon" :class="{ 'text-accent': isClipboardEnabled }" />
          <div class="stat-details">
            <div class="stat-label">Clipboard</div>
            <div class="stat-value">{{ isClipboardEnabled ? 'MONITORING' : 'OFF' }}</div>
          </div>
        </div>
        
        <Transition name="slide-fade">
          <div v-if="detectedUrl" class="clip-popup glass-panel shadow-glow">
            <div class="clip-info">
              <span class="clip-label">URL Detected</span>
              <span class="clip-url">{{ detectedUrl.split('/').pop() }}</span>
            </div>
            <div class="clip-actions">
              <button class="clip-btn btn-add" @click="$emit('new-download', detectedUrl); clearDetected()">Add</button>
              <button class="clip-btn btn-ignore" @click="clearDetected()">Dismiss</button>
            </div>
          </div>
        </Transition>
      </div>
    </div>

    <div class="actions">
      <button class="btn-new shadow-glow" @click="$emit('new-download')">
        <Plus class="btn-icon" />
        <span>New Download</span>
      </button>
    </div>
  </header>
</template>

<style scoped>
.top-bar {
  height: 80px;
  margin: 16px;
  padding: 0 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-radius: 20px;
  z-index: 10;
}

.stats-group {
  display: flex;
  align-items: center;
  gap: 32px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stat-details {
  display: flex;
  flex-direction: column;
}

.stat-icon {
  width: 20px;
  height: 20px;
  color: var(--text-secondary);
  transition: var(--transition-smooth);
}

.stat-icon.text-accent {
  color: var(--accent-primary);
}

.stat-label {
  font-size: 0.65rem;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 600;
}

.stat-value {
  font-size: 0.9rem;
  font-weight: 700;
  font-family: var(--font-mono, monospace);
  color: var(--text-primary);
}

/* Engine Status Styles */
.engine-online { 
  color: var(--color-downloading); 
  filter: drop-shadow(0 0 8px rgba(16, 185, 129, 0.4));
}
.engine-offline { 
  color: var(--color-error); 
  filter: drop-shadow(0 0 8px rgba(239, 68, 68, 0.4));
}
.text-online { color: var(--color-downloading); }
.text-offline { color: var(--color-error); }

/* Bridge Animations */
.bridge-active {
  color: var(--color-downloading);
  animation: pulse-green 2s infinite;
}

.bridge-waiting {
  color: var(--text-secondary);
  opacity: 0.5;
}

@keyframes pulse-green {
  0% { filter: drop-shadow(0 0 2px rgba(16, 185, 129, 0.4)); }
  50% { filter: drop-shadow(0 0 8px rgba(16, 185, 129, 0.8)); }
  100% { filter: drop-shadow(0 0 2px rgba(16, 185, 129, 0.4)); }
}

/* Clipboard Styles */
.clipboard-group {
  position: relative;
}

.clipboard-toggle {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 8px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s;
}

.clipboard-toggle:hover { background: rgba(255, 255, 255, 0.05); }
.clipboard-group.active .clipboard-toggle { background: rgba(59, 130, 246, 0.1); }

.clip-popup {
  position: absolute;
  top: 60px;
  right: 0;
  width: 240px;
  padding: 12px;
  border-radius: 12px;
  z-index: 100;
  border: 1px solid var(--accent-primary);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.clip-info { display: flex; flex-direction: column; gap: 2px; }
.clip-label { font-size: 0.6rem; text-transform: uppercase; color: var(--text-secondary); font-weight: 800; }
.clip-url { font-size: 0.75rem; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--text-primary); }

.clip-actions { display: flex; gap: 8px; }
.clip-btn { flex: 1; padding: 6px; border-radius: 6px; font-size: 0.7rem; font-weight: 700; cursor: pointer; border: none; transition: 0.2s; }
.btn-add { background: var(--accent-primary); color: white; }
.btn-ignore { background: rgba(255, 255, 255, 0.1); color: var(--text-primary); }
.clip-btn:hover { filter: brightness(1.2); }

/* Transitions */
.slide-fade-enter-active, .slide-fade-leave-active { transition: all 0.3s ease-out; }
.slide-fade-enter-from { transform: translateY(-10px); opacity: 0; }
.slide-fade-leave-to { transform: translateY(-10px); opacity: 0; }

.top-bar-graph {
  margin-left: 12px;
  border-left: 1px solid var(--border-color);
  padding-left: 12px;
}

.limiter-group {
  gap: 20px;
}

.limiter-toggle {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 8px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s;
}

.limiter-toggle:hover {
  background: rgba(255, 255, 255, 0.05);
}

.limiter-toggle.active {
  background: rgba(59, 130, 246, 0.1);
}

.limiter-slider {
  width: 100px;
  height: 4px;
  appearance: none;
  -webkit-appearance: none;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  outline: none;
}

.limiter-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  background: var(--accent-primary);
  border-radius: 50%;
  cursor: pointer;
  box-shadow: 0 0 10px rgba(59, 130, 246, 0.5);
}

.stat-divider {
  width: 1px;
  height: 32px;
  background: var(--border-color);
}

.btn-new {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--accent-primary);
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: var(--transition-smooth);
}

.btn-new:hover {
  filter: brightness(1.1);
  transform: translateY(-2px);
  box-shadow: 0 4px 20px rgba(59, 130, 246, 0.4);
}

.btn-icon {
  width: 18px;
  height: 18px;
}

.shadow-glow {
  box-shadow: 0 0 15px rgba(59, 130, 246, 0.2);
}
</style>

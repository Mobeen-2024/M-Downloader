<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useDownloadStore } from '@/stores/download.store';
import { useSettingsStore } from '@/stores/settings.store';
import { useFormatters } from '@/composables/useFormatters';
import { invoke } from '@tauri-apps/api/core';
import SpeedGraph from '@/features/shared/components/SpeedGraph.vue';
import BaseButton from '@/features/shared/components/BaseButton.vue';
import { Plus, Wifi, Activity, Zap, Link2, Clipboard, Gauge } from 'lucide-vue-next';
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
  <header class="top-bar" data-tauri-drag-region>
    <div class="stats-container" data-tauri-drag-region>
      <!-- Real-time Throughput -->
      <div class="stat-card throughput-card">
        <div class="card-icon">
          <Activity :size="18" class="text-accent" />
        </div>
        <div class="card-data">
          <span class="label">Total Throughput</span>
          <span class="value">{{ formatSpeed(store.totalSpeed) }}</span>
        </div>
        <SpeedGraph :current-speed="store.totalSpeed" class="mini-graph" />
      </div>

      <div class="divider"></div>

      <!-- Active Threads -->
      <div class="stat-card">
        <div class="card-icon">
          <Wifi :size="18" />
        </div>
        <div class="card-data">
          <span class="label">Active Workers</span>
          <span class="value">{{ store.activeDownloads.length }}</span>
        </div>
      </div>

      <div class="divider"></div>

      <!-- Engine Status -->
      <div class="stat-card" :class="{ 'is-offline': !depStatus.ffmpeg_found }">
        <div class="card-icon">
          <Zap :size="18" />
        </div>
        <div class="card-data">
          <span class="label">Muxing Engine</span>
          <span class="value status-text">{{ depStatus.ffmpeg_found ? 'HEALTHY' : 'ERROR' }}</span>
        </div>
      </div>

      <!-- Bridge Status -->
      <template v-if="settings.bridgeEnabled">
        <div class="divider"></div>
        <div class="stat-card" :class="{ 'is-active-glow': store.bridgeConnected }">
          <div class="card-icon">
            <Link2 :size="18" />
          </div>
          <div class="card-data">
            <span class="label">Chrome Extension</span>
            <span class="value status-text">{{ store.bridgeConnected ? 'LINKED' : 'WAITING' }}</span>
          </div>
        </div>
      </template>

      <div class="divider"></div>

      <!-- Quick Toggles -->
      <div class="toggles-group">
        <button 
          class="interactive-toggle" 
          :class="{ 'is-active': isLimitEnabled }"
          @click="isLimitEnabled = !isLimitEnabled"
          title="Speed Limiter"
        >
          <Gauge :size="18" />
          <div class="toggle-info">
            <span class="t-label">Limit</span>
            <span class="t-val">{{ isLimitEnabled ? speedLimitKbps + 'K' : 'OFF' }}</span>
          </div>
          <div v-if="isLimitEnabled" class="slider-popover" @click.stop>
            <input type="range" v-model.number="speedLimitKbps" min="100" max="10240" step="100" />
          </div>
        </button>

        <button 
          class="interactive-toggle" 
          :class="{ 'is-active': isClipboardEnabled }"
          @click="toggleClipboard"
          title="Clipboard Monitor"
        >
          <Clipboard :size="18" />
          <div class="toggle-info">
            <span class="t-label">Auto-Detect</span>
            <span class="t-val">{{ isClipboardEnabled ? 'ON' : 'OFF' }}</span>
          </div>
          
          <Transition name="pop-in">
            <div v-if="detectedUrl" class="clip-toast" @click.stop>
              <div class="toast-header">
                <Link2 :size="12" />
                <span>Link Caught</span>
              </div>
              <p class="toast-body">{{ detectedUrl.split('/').pop() }}</p>
              <div class="toast-footer">
                <button class="t-btn primary" @click="$emit('new-download', detectedUrl); clearDetected()">Download</button>
                <button class="t-btn ghost" @click="clearDetected()">Ignore</button>
              </div>
            </div>
          </Transition>
        </button>
      </div>
    </div>

    <div class="actions-container">
      <BaseButton variant="primary" @click="$emit('new-download')">
        <template #icon-left><Plus :size="18" /></template>
        New Download
      </BaseButton>
    </div>
  </header>
</template>

<style scoped>
.top-bar {
  height: 90px;
  min-height: 90px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border-color);
  padding: 0 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 40px;
  position: relative;
}

.stats-container {
  display: flex;
  align-items: center;
  gap: 24px;
  flex: 1;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 12px;
}

.card-icon {
  width: 40px;
  height: 40px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.card-data {
  display: flex;
  flex-direction: column;
}

.label {
  font-size: 0.65rem;
  color: var(--text-secondary);
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  opacity: 0.5;
}

.value {
  font-size: 0.95rem;
  font-weight: 800;
  font-family: var(--font-mono);
  color: var(--text-primary);
}

.status-text {
  font-size: 0.8rem;
  letter-spacing: 0.02em;
}

.mini-graph {
  margin-left: 20px;
  padding-left: 20px;
  border-left: 1px solid var(--border-color);
}

.divider {
  width: 1px;
  height: 24px;
  background: var(--border-color);
  opacity: 0.5;
}

/* Status variants */
.is-offline .card-icon { color: var(--color-error); background: rgba(239, 68, 68, 0.05); }
.is-offline .status-text { color: var(--color-error); }

.is-active-glow .card-icon { 
  color: var(--color-downloading); 
  background: rgba(16, 185, 129, 0.05);
  box-shadow: 0 0 15px rgba(16, 185, 129, 0.1);
}
.is-active-glow .status-text { color: var(--color-downloading); }

/* Interactive Toggle Buttons */
.toggles-group {
  margin-left: auto;
  display: flex;
  gap: 12px;
}

.interactive-toggle {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 8px 12px;
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--text-secondary);
  position: relative;
}

.interactive-toggle:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.1);
}

.interactive-toggle.is-active {
  background: rgba(59, 130, 246, 0.08);
  border-color: rgba(59, 130, 246, 0.2);
  color: var(--accent-primary);
}

.toggle-info {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  min-width: 40px;
}

.t-label {
  font-size: 0.6rem;
  text-transform: uppercase;
  font-weight: 800;
  opacity: 0.7;
}

.t-val {
  font-size: 0.75rem;
  font-weight: 800;
}

/* Popover & Toasts */
.slider-popover {
  position: absolute;
  top: 60px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--bg-secondary);
  padding: 12px;
  border-radius: 10px;
  border: 1px solid var(--border-color);
  box-shadow: var(--shadow-glass);
  z-index: 100;
}

.clip-toast {
  position: absolute;
  top: 60px;
  right: 0;
  width: 240px;
  background: var(--bg-secondary);
  padding: 16px;
  border-radius: 14px;
  border: 1px solid var(--accent-primary);
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.6);
  z-index: 100;
  display: flex;
  flex-direction: column;
  gap: 12px;
  cursor: default;
}

.toast-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.6rem;
  text-transform: uppercase;
  font-weight: 800;
  color: var(--accent-primary);
}

.toast-body {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.toast-footer {
  display: flex;
  gap: 8px;
}

.t-btn {
  flex: 1;
  padding: 8px;
  border-radius: 8px;
  font-size: 0.7rem;
  font-weight: 800;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
}

.t-btn.primary { background: var(--accent-primary); color: white; }
.t-btn.ghost { background: rgba(255, 255, 255, 0.05); color: var(--text-secondary); }
.t-btn:hover { filter: brightness(1.2); }

/* Animation */
.pop-in-enter-active { animation: pop-in 0.3s cubic-bezier(0.34, 1.56, 0.64, 1); }
.pop-in-leave-active { animation: pop-in 0.2s reverse; }

@keyframes pop-in {
  from { opacity: 0; transform: translateY(-10px) scale(0.9); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}
</style>

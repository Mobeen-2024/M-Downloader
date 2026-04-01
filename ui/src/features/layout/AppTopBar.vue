<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useDownloadStore } from '@/stores/download.store';
import { useFormatters } from '@/composables/useFormatters';
import { invoke } from '@tauri-apps/api/core';
import SpeedGraph from '@/features/shared/components/SpeedGraph.vue';
import { Plus, Wifi, Activity, Gauge, Zap } from 'lucide-vue-next';

const store = useDownloadStore();
const { formatSpeed } = useFormatters();

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

      <div class="stat-divider"></div>

      <div class="stat-item limiter-group">
        <div class="limiter-toggle" @click="isLimitEnabled = !isLimitEnabled" :class="{ active: isLimitEnabled }">
          <Gauge class="stat-icon" :class="{ 'text-accent': isLimitEnabled }" />
          <div class="stat-details">
            <div class="stat-label">Speed Limit</div>
            <div class="stat-value">{{ isLimitEnabled ? speedLimitKbps + ' KB/s' : 'OFF' }}</div>
          </div>
        </div>
        <input 
          v-if="isLimitEnabled"
          type="range" 
          v-model.number="speedLimitKbps" 
          min="128" 
          max="10240" 
          step="128"
          class="limiter-slider"
        />
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

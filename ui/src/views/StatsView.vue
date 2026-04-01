<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed } from 'vue';
import { useDownloadStore } from '@/stores/download.store';
import { 
  Zap, 
  Cpu, 
  HardDrive, 
  Activity,
  AlertCircle
} from 'lucide-vue-next';
import GlassPanel from '@/features/shared/components/GlassPanel.vue';

const store = useDownloadStore();

const activeDownload = computed(() => {
  return store.activeDownloads[0] || null;
});

const metrics = computed(() => {
  if (!activeDownload.value?.metrics) return {
    io_efficiency: 0,
    active_workers: 0,
    avg_latency_ms: 0
  };
  
  return activeDownload.value.metrics;
});

const latencyStatus = computed(() => {
  const ms = metrics.value.avg_latency_ms;
  if (ms === 0) return { label: 'Idle', class: 'idle' };
  if (ms < 50) return { label: 'Ultra-Low', class: 'ultra' };
  if (ms < 150) return { label: 'Stable', class: 'ok' };
  if (ms < 300) return { label: 'Moderate Jitter', class: 'warn' };
  return { label: 'Severe Latency', class: 'error' };
});

const applySimulation = async (latency: number, packetLoss: number) => {
  try {
    await invoke('set_network_condition', { latency, packetLoss });
  } catch (e) {
    console.error('Failed to set simulation:', e);
  }
};
</script>

<template>
  <div class="view-container">
    <header class="view-header">
      <Activity class="header-icon" />
      <div>
        <h1>Performance Benchmarking</h1>
        <p class="text-secondary">Real-time validation of the Mdownloader Acceleration Engine.</p>
      </div>
    </header>

    <div v-if="!activeDownload" class="empty-state">
      <Zap class="empty-icon" />
      <h3>No Data Collected</h3>
      <p>Start a download to see real-time performance metrics and load-balancing efficiency.</p>
    </div>

    <div v-else class="stats-grid">
      <!-- I/O Efficiency Card -->
      <GlassPanel class="stats-card">
        <div class="card-header">
          <HardDrive class="card-icon blue" />
          <h3>Disk I/O Efficiency</h3>
        </div>
        <div class="metric-value">{{ (metrics.io_efficiency * 100).toFixed(1) }}%</div>
        <div class="progress-bar-mini">
          <div class="progress-fill" :style="{ width: metrics.io_efficiency * 100 + '%' }"></div>
        </div>
        <p class="card-desc">Ratio of requested data vs successful disk commits. Higher is better.</p>
      </GlassPanel>

      <!-- Worker Engagement Card -->
      <GlassPanel class="stats-card">
        <div class="card-header">
          <Cpu class="card-icon pink" />
          <h3>Worker Engagement</h3>
        </div>
        <div class="metric-value">{{ metrics.active_workers }} / 32</div>
        <div class="worker-dots">
          <div 
            v-for="n in 32" 
            :key="n" 
            class="dot"
            :class="{ active: n <= metrics.active_workers }"
          ></div>
        </div>
        <p class="card-desc">Number of active threads performing "In-Half Division".</p>
      </GlassPanel>

      <!-- Network Latency Card -->
      <GlassPanel class="stats-card">
        <div class="card-header">
          <Activity class="card-icon yellow" />
          <h3>Engine Latency (TTFB)</h3>
        </div>
        <div class="metric-value">{{ metrics.avg_latency_ms }}<span class="unit">ms</span></div>
        <div class="latency-status" :class="latencyStatus.class">
          {{ latencyStatus.label }}
        </div>
        <p class="card-desc">Real-time Time-To-First-Byte across all active segments.</p>
      </GlassPanel>
    </div>

    <div v-if="activeDownload" class="simulation-panel glass-panel">
      <div class="panel-header">
        <AlertCircle class="panel-icon" />
        <h3>Hostile Network Simulation</h3>
      </div>
      <div class="simulation-actions">
        <button class="btn-simulation" @click="applySimulation(500, 0.0)">Simulate 500ms Latency</button>
        <button class="btn-simulation" @click="applySimulation(0, 0.1)">Simulate 10% Packet Loss</button>
        <button class="btn-simulation" @click="applySimulation(200, 0.05)">High Jitter (Mixed)</button>
        <button class="btn-simulation reset" @click="applySimulation(0, 0.0)">Reset Conditions</button>
      </div>
      <p class="help-text">Use these tools to validate how the engine re-segments files under stress.</p>
    </div>
  </div>
</template>

<style scoped>
.view-container {
  height: 100%;
  overflow-y: auto;
  padding: 32px;
  padding-bottom: 80px;
}

.view-header {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-bottom: 40px;
}

.header-icon {
  width: 48px;
  height: 48px;
  color: var(--accent-primary);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 24px;
  margin-bottom: 40px;
}

.stats-card {
  padding: 24px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.card-icon { width: 20px; height: 20px; }
.card-icon.blue { color: var(--accent-primary); }
.card-icon.pink { color: #ec4899; }
.card-icon.yellow { color: #f59e0b; }

.metric-value {
  font-size: 2.5rem;
  font-weight: 800;
  margin-bottom: 12px;
  background: linear-gradient(135deg, var(--text-primary), var(--text-secondary));
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.unit {
  font-size: 1rem;
  margin-left: 4px;
  color: var(--text-secondary);
  -webkit-text-fill-color: var(--text-secondary);
}

.progress-bar-mini {
  height: 6px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 3px;
  margin-bottom: 16px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent-primary);
}

.worker-dots {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 16px;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.1);
}

.dot.active {
  background: #ec4899;
  box-shadow: 0 0 8px rgba(236, 72, 153, 0.4);
}

.latency-status {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 700;
  background: rgba(255, 255, 255, 0.05);
  transition: all 0.3s ease;
}

.latency-status.idle { color: var(--text-secondary); }
.latency-status.ultra { background: rgba(16, 185, 129, 0.1); color: #10b981; }
.latency-status.ok { background: rgba(34, 197, 94, 0.1); color: #22c55e; }
.latency-status.warn { background: rgba(245, 158, 11, 0.1); color: #f59e0b; }
.latency-status.error { background: rgba(239, 68, 68, 0.1); color: #ef4444; }

.card-desc {
  font-size: 0.8rem;
  color: var(--text-secondary);
  line-height: 1.4;
}

.simulation-panel {
  padding: 32px;
  background: rgba(20, 20, 20, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
}

.panel-icon { color: #f59e0b; }

.simulation-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 16px;
}

.btn-simulation {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 12px 20px;
  border-radius: 12px;
  color: var(--text-primary);
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition-smooth);
}

.btn-simulation:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.2);
}

.btn-simulation.reset {
  background: var(--accent-primary);
  border: none;
}

.help-text {
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.empty-state {
  height: 50vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--text-secondary);
}

.empty-icon {
  width: 64px;
  height: 64px;
  margin-bottom: 24px;
  opacity: 0.2;
}
</style>

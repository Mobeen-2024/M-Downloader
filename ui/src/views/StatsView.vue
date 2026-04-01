<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ref, computed, watch } from 'vue';
import { useDownloadStore } from '@/stores/download.store';
import { 
  Zap, 
  Cpu, 
  HardDrive, 
  Activity,
  AlertCircle,
  ShieldCheck,
  History,
} from 'lucide-vue-next';
import GlassPanel from '@/features/shared/components/GlassPanel.vue';

const store = useDownloadStore();

const activeDownload = computed(() => {
  return store.activeDownloads[0] || null;
});

const metrics = computed(() => {
  return activeDownload.value?.metrics || {
    io_efficiency: 0,
    active_workers: 0,
    avg_latency_ms: 0,
    engine_stats: { total_splits: 0, total_retries: 0, http_version: 'HTTP/1.1' }
  };
});

const speedHistory = ref<number[]>(new Array(60).fill(0));
watch(() => activeDownload.value?.speed_bps, (newSpeed) => {
  if (newSpeed !== undefined) {
    speedHistory.value.push(newSpeed);
    if (speedHistory.value.length > 60) speedHistory.value.shift();
  }
});

const sparklinePoints = computed(() => {
  if (!speedHistory.value.length) return '';
  const max = Math.max(...speedHistory.value, 1024 * 1024); // Min 1MB/s scale
  return speedHistory.value.map((s, i) => {
    const x = (i / 59) * 100;
    const y = 30 - (s / max) * 30;
    return `${x},${y}`;
  }).join(' ');
});

const efficiencyIndex = computed(() => {
  const stats = metrics.value.engine_stats;
  if (!stats || stats.total_splits === 0) return 100;
  // Penalty for retries normalized by splits
  const penalty = (stats.total_retries / (stats.total_splits + 1)) * 50;
  return Math.max(0, Math.min(100, 100 - penalty)).toFixed(1);
});

const latencyStatus = computed(() => {
  const ms = metrics.value.avg_latency_ms;
  if (ms === 0) return { label: 'Idle', class: 'idle' };
  if (ms < 50) return { label: 'Ultra-Low', class: 'ultra' };
  if (ms < 150) return { label: 'Stable', class: 'ok' };
  if (ms < 300) return { label: 'Moderate Jitter', class: 'warn' };
  return { label: 'Severe Latency', class: 'error' };
});

const getSegmentColor = (seg: any) => {
  if (seg.state === 'Completed') return 'completed';
  if (seg.state === 'Active') {
    const lat = seg.last_latency_ms;
    if (lat === 0) return 'active-idle';
    if (lat < 100) return 'active-fast';
    if (lat < 300) return 'active-stable';
    return 'active-slow';
  }
  if (seg.state === 'Failed') return 'failed';
  return 'pending';
};

const snifferLogs = ref<any[]>([]);
const isSnifferActive = ref(false);

const applySimulation = async (latency: number, packetLoss: number) => {
  try {
    await invoke('set_network_condition', { latency, packetLoss });
  } catch (e) {
    console.error('Failed to set simulation:', e);
  }
};

const formatTime = (ts: number) => {
  return new Date(ts * 1000).toLocaleTimeString([], { hour12: false });
};

import { listen } from '@tauri-apps/api/event';
import { onMounted, onUnmounted } from 'vue';

let unlistenSniffer: any = null;

onMounted(async () => {
  unlistenSniffer = await listen('sniffer-hit', (event: any) => {
    snifferLogs.value.unshift(event.payload);
    if (snifferLogs.value.length > 50) snifferLogs.value.pop();
  });
  
  try {
    isSnifferActive.value = await invoke('get_sniffer_status');
  } catch (e) {
    console.error('Status check failed:', e);
  }
});

onUnmounted(() => {
  if (unlistenSniffer) unlistenSniffer();
});

const formatSpeed = (bps: number) => {
  if (bps === 0) return '0 B/s';
  const k = 1024;
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  const i = Math.floor(Math.log(bps) / Math.log(k));
  return parseFloat((bps / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
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
      <!-- Throughput Stability Card (NEW) -->
      <GlassPanel class="stats-card full-width">
        <div class="card-header">
          <History class="card-icon blue" />
          <h3>Throughput Stability</h3>
          <div class="header-value">{{ formatSpeed(activeDownload.speed_bps) }}</div>
        </div>
        <div class="sparkline-container">
          <svg viewBox="0 0 100 30" preserveAspectRatio="none" class="sparkline">
            <defs>
              <linearGradient id="sparkGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="var(--accent-primary)" stop-opacity="0.5" />
                <stop offset="100%" stop-color="var(--accent-primary)" stop-opacity="0" />
              </linearGradient>
            </defs>
            <path
              :d="`M ${sparklinePoints} L 100,30 L 0,30 Z`"
              fill="url(#sparkGradient)"
            />
            <polyline
              fill="none"
              stroke="var(--accent-primary)"
              stroke-width="1.5"
              stroke-linejoin="round"
              :points="sparklinePoints"
            />
          </svg>
        </div>
        <p class="card-desc">Real-time bandwidth consistency over the last 60 seconds.</p>
      </GlassPanel>
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

      <!-- Segment Latency Heatmap -->
      <GlassPanel class="stats-card heatmap-card">
        <div class="card-header">
          <Cpu class="card-icon pink" />
          <h3>Latency Heatmap</h3>
        </div>
        <div class="segment-grid">
          <div 
            v-for="(seg, idx) in activeDownload.segments" 
            :key="idx" 
            class="segment-box"
            :class="getSegmentColor(seg)"
            :title="`Seg ${idx}: ${seg.last_latency_ms}ms`"
          ></div>
        </div>
        <div class="heatmap-legend">
          <span class="legend-item"><i class="sq fast"></i> &lt;100ms</span>
          <span class="legend-item"><i class="sq stable"></i> &lt;300ms</span>
          <span class="legend-item"><i class="sq slow"></i> &gt;300ms</span>
          <span class="legend-item"><i class="sq comp"></i> Done</span>
        </div>
        <p class="card-desc">Real-time responsiveness of every active byte-range segment.</p>
      </GlassPanel>

      <!-- Engine Health Card (NEW) -->
      <GlassPanel class="stats-card">
        <div class="card-header">
          <ShieldCheck class="card-icon emerald" />
          <h3>Engine Health</h3>
        </div>
        <div class="health-metrics">
          <div class="h-row">
            <span>Efficiency Index</span>
            <span class="h-val">{{ efficiencyIndex }}%</span>
          </div>
          <div class="h-row">
            <span>Total Splits</span>
            <span class="h-val">{{ metrics.engine_stats?.total_splits || 0 }}</span>
          </div>
          <div class="h-row">
            <span>Protocol</span>
            <span class="h-val badge">{{ metrics.engine_stats?.http_version || 'HTTP/1.1' }}</span>
          </div>
        </div>
        <p class="card-desc">Adaptive health score based on split success vs connection retries.</p>
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
      <!-- Live Sniffer Log (NEW) -->
      <GlassPanel class="stats-card sniffer-card" v-if="isSnifferActive">
        <div class="card-header">
          <Activity class="card-icon blue" />
          <h3>Live Sniffer Log</h3>
          <div class="header-badge">WFP ACTIVE</div>
        </div>
        <div class="sniffer-terminal">
          <div v-if="snifferLogs.length === 0" class="terminal-empty">
            Waiting for packets...
          </div>
          <div v-for="log in snifferLogs" :key="log.timestamp + log.url" class="terminal-line">
            <span class="t-time">[{{ formatTime(log.timestamp) }}]</span>
            <span class="t-proc">[{{ log.process_name }}]</span>
            <span class="t-url" :title="log.url">{{ log.url }}</span>
          </div>
        </div>
        <p class="card-desc">Real-time URL interception from the kernel-mode driver.</p>
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

.dot.primary {
  background: var(--accent-primary);
  box-shadow: 0 0 10px var(--accent-primary);
}

.dot.auxiliary {
  background: #ec4899;
  opacity: 0.8;
  animation: pulse 2s infinite alternate;
}

@keyframes pulse {
  from { opacity: 0.6; transform: scale(0.9); }
  to { opacity: 1.0; transform: scale(1.1); }
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

/* Heatmap Styles */
.heatmap-card {
  min-height: 240px;
}

.segment-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(12px, 1fr));
  gap: 4px;
  max-height: 120px;
  overflow-y: auto;
  margin-bottom: 20px;
  padding-right: 4px;
}

.segment-box {
  aspect-ratio: 1;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.05);
  transition: all 0.3s ease;
}

.segment-box.active-fast { background: #10b981; box-shadow: 0 0 8px rgba(16, 185, 129, 0.4); }
.segment-box.active-stable { background: #f59e0b; box-shadow: 0 0 8px rgba(245, 158, 11, 0.4); }
.segment-box.active-slow { background: #ef4444; box-shadow: 0 0 8px rgba(239, 68, 68, 0.4); }
.segment-box.active-idle { background: var(--accent-primary); }
.segment-box.completed { background: rgba(255, 255, 255, 0.2); }
.segment-box.failed { background: #7f1d1d; border: 1px solid #ef4444; }
.segment-box.pending { background: rgba(255, 255, 255, 0.05); }

.heatmap-legend {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
  font-size: 0.7rem;
  color: var(--text-secondary);
}

.legend-item { display: flex; align-items: center; gap: 4px; }
.sq { width: 8px; height: 8px; border-radius: 1px; }
.sq.fast { background: #10b981; }
.sq.stable { background: #f59e0b; }
.sq.slow { background: #ef4444; }
.sq.comp { background: rgba(255, 255, 255, 0.2); }

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

.full-width {
  grid-column: 1 / -1;
}

.header-value {
  margin-left: auto;
  font-family: 'JetBrains Mono', monospace;
  font-weight: 700;
  color: var(--accent-primary);
  font-size: 1.1rem;
}

.sparkline-container {
  height: 80px;
  width: 100%;
  margin: 15px 0;
  position: relative;
}

.sparkline {
  width: 100%;
  height: 100%;
  overflow: visible;
}

.health-metrics {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 15px;
}

.h-row {
  display: flex;
  justify-content: space-between;
  font-size: 0.9rem;
  color: var(--text-secondary);
}

.h-val {
  color: var(--text-primary);
  font-weight: 700;
}

.h-val.badge {
  background: rgba(59, 130, 246, 0.1);
  color: var(--accent-primary);
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 0.75rem;
}

/* Sniffer Terminal */
.sniffer-card {
  grid-column: 1 / -1;
  max-height: 300px;
}

.header-badge {
  margin-left: auto;
  font-size: 0.65rem;
  background: var(--accent-primary);
  color: white;
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 800;
  letter-spacing: 0.5px;
}

.sniffer-terminal {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  height: 180px;
  overflow-y: auto;
  padding: 12px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.terminal-empty {
  opacity: 0.4;
  text-align: center;
  margin-top: 60px;
  font-style: italic;
}

.terminal-line {
  display: flex;
  gap: 12px;
  white-space: nowrap;
  overflow: hidden;
}

.t-time { color: var(--text-secondary); }
.t-proc { color: var(--accent-primary); font-weight: 700; }
.t-url { color: var(--text-primary); text-overflow: ellipsis; overflow: hidden; }

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

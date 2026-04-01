<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useDownloadStore } from '@/stores/download.store';
import { animate, stagger, spring } from 'motion';
import { 
  Cpu, 
  HardDrive, 
  Activity,
  AlertCircle,
  ShieldCheck,
  History,
  Terminal,
  Layers,
  Network
} from 'lucide-vue-next';
import BaseCard from '@/features/shared/components/BaseCard.vue';
import BaseButton from '@/features/shared/components/BaseButton.vue';

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
let unlistenSniffer: any = null;

const applySimulation = async (latency: number, packetLoss: number) => {
  try {
    await invoke('set_network_condition', { latency, packetLoss });
  } catch (e) {
    console.error('Failed to set simulation:', e);
  }
};

const formatTime = (ts: number) => {
  return new Date(ts * 1000).toLocaleTimeString([], { hour12: false, fractionalSecondDigits: 3 } as any);
};

const formatSpeed = (bps: number) => {
  if (bps === 0) return '0 B/s';
  const k = 1024;
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  const i = Math.floor(Math.log(bps) / Math.log(k));
  return parseFloat((bps / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const gridRef = ref<HTMLElement | null>(null);

onMounted(async () => {
  // Phase 5: Dashboard Entry Animation
  if (gridRef.value) {
    (animate as any)(
      ".dashboard-item",
      { opacity: [0, 1], y: [20, 0] },
      { 
        delay: stagger(0.05),
        easing: spring({ stiffness: 300, damping: 30 })
      }
    );
  }

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
</script>

<template>
  <div class="stats-view">
    <header class="view-header">
      <div class="header-left">
        <h2 class="view-title">System Instrumentation</h2>
        <p class="view-subtitle">Real-time kernel-level telemetry and engine performance analysis.</p>
      </div>
      <div class="header-right" v-if="isSnifferActive">
        <div class="status-indicator">
          <div class="pulse-dot"></div>
          <span>WFP Driver Active</span>
        </div>
      </div>
    </header>

    <div v-if="!activeDownload && !snifferLogs.length" class="empty-state">
      <BaseCard variant="glass" padding="lg" class="empty-card" ref="emptyCardRef">
        <Activity :size="48" class="empty-icon" />
        <h3>Awaiting Data Packets</h3>
        <p>No active telemetry detected. Start a transmission or browse the web to see live packet interception.</p>
      </BaseCard>
    </div>

    <div v-else class="dashboard-grid" ref="gridRef">
      <!-- Throughput Visualizer -->
      <BaseCard variant="glass" padding="md" class="throughput-card full-width dashboard-item">
        <div class="card-header">
          <div class="header-main">
            <History :size="16" class="text-accent" />
            <h3>Throughput Stability</h3>
          </div>
          <div class="current-value">{{ formatSpeed(activeDownload?.speed_bps || 0) }}</div>
        </div>
        
        <div class="graph-container">
          <svg viewBox="0 0 100 30" preserveAspectRatio="none" class="spark-svg">
            <defs>
              <linearGradient id="areaGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="var(--accent-primary)" stop-opacity="0.3" />
                <stop offset="100%" stop-color="var(--accent-primary)" stop-opacity="0" />
              </linearGradient>
            </defs>
            <path
              :d="`M ${sparklinePoints} L 100,30 L 0,30 Z`"
              fill="url(#areaGradient)"
              class="graph-area"
            />
            <polyline
              fill="none"
              stroke="var(--accent-primary)"
              stroke-width="1.5"
              stroke-linejoin="round"
              stroke-linecap="round"
              :points="sparklinePoints"
              class="graph-line"
            />
          </svg>
        </div>
        <div class="graph-footer">
          <span>60s Window</span>
          <span>Adaptive Scaling</span>
        </div>
      </BaseCard>

      <!-- IO & Engine Health -->
      <div class="stats-subgrid full-width">
        <BaseCard variant="glass" padding="md" class="stat-mini-card dashboard-item">
          <div class="mini-header">
            <HardDrive :size="14" />
            <span>Disk I/O Efficiency</span>
          </div>
          <div class="mini-value">{{ (metrics.io_efficiency * 100).toFixed(1) }}%</div>
          <div class="pro-progress">
            <div class="pro-fill" :style="{ width: metrics.io_efficiency * 100 + '%' }"></div>
          </div>
        </BaseCard>

        <BaseCard variant="glass" padding="md" class="stat-mini-card dashboard-item">
          <div class="mini-header">
            <ShieldCheck :size="14" class="text-success" />
            <span>Engine Reliability</span>
          </div>
          <div class="mini-value">{{ efficiencyIndex }}%</div>
          <div class="pro-progress">
            <div class="pro-fill success" :style="{ width: efficiencyIndex + '%' }"></div>
          </div>
        </BaseCard>

        <BaseCard variant="glass" padding="md" class="stat-mini-card dashboard-item">
          <div class="mini-header">
            <Cpu :size="14" class="text-accent" />
            <span>Active Workers</span>
          </div>
          <div class="mini-value">{{ metrics.active_workers }}</div>
          <div class="worker-badges">
            <div 
              v-for="i in metrics.active_workers" 
              :key="i" 
              class="worker-pip active"
            ></div>
          </div>
        </BaseCard>

        <BaseCard variant="glass" padding="md" class="stat-mini-card dashboard-item">
          <div class="mini-header">
            <Network :size="14" />
            <span>Latency (TTFB)</span>
          </div>
          <div class="mini-value">{{ metrics.avg_latency_ms }}<span class="ms">ms</span></div>
          <div class="latency-indicator" :class="latencyStatus.class">{{ latencyStatus.label }}</div>
        </BaseCard>
      </div>

      <!-- Latency Heatmap -->
      <BaseCard variant="glass" padding="md" class="heatmap-card dashboard-item" v-if="activeDownload">
        <div class="card-header">
          <div class="header-main">
            <Layers :size="16" />
            <h3>Segment Heatmap</h3>
          </div>
        </div>
        <div class="heatmap-grid" ref="heatmapRef">
          <div 
            v-for="(seg, idx) in activeDownload.segments" 
            :key="idx" 
            class="heat-pixel"
            :class="getSegmentColor(seg)"
            :title="`Segment ${idx}: ${seg.last_latency_ms}ms`"
            :style="{ '--delay': idx * 0.01 + 's' }"
          ></div>
        </div>
        <div class="heatmap-legend">
          <div class="legend-pip fast"></div> <span>Fast</span>
          <div class="legend-pip ok"></div> <span>Stable</span>
          <div class="legend-pip slow"></div> <span>Slow</span>
          <div class="legend-pip done"></div> <span>Done</span>
        </div>
      </BaseCard>

      <!-- Sniffer Logs -->
      <BaseCard variant="glass" padding="md" class="sniffer-card dashboard-item" :class="{ 'full-width': !activeDownload }">
        <div class="card-header">
          <div class="header-main">
            <Terminal :size="16" />
            <h3>Live Packet Sniffer</h3>
          </div>
          <div class="sniffer-badge">KERNEL-MODE</div>
        </div>
        <div class="terminal-shell">
          <div v-if="!snifferLogs.length" class="terminal-waiting">
            Initializing WFP buffer... awaiting outbound link request.
          </div>
          <div v-for="(log, i) in snifferLogs" :key="i" class="terminal-line">
            <span class="line-time">{{ formatTime(log.timestamp) }}</span>
            <span class="line-proc">{{ log.process_name }}</span>
            <span class="line-url">{{ log.url }}</span>
          </div>
        </div>
      </BaseCard>

      <!-- Simulation Controls -->
      <BaseCard variant="glass" padding="md" class="simulation-card full-width dashboard-item">
        <div class="card-header">
          <div class="header-main">
            <AlertCircle :size="16" class="text-warn" />
            <h3>Adversarial Simulation</h3>
          </div>
        </div>
        <div class="simulation-actions">
          <BaseButton variant="glass" size="sm" @click="applySimulation(500, 0)">Simulate Latency (500ms)</BaseButton>
          <BaseButton variant="glass" size="sm" @click="applySimulation(0, 0.1)">Simulate Loss (10%)</BaseButton>
          <BaseButton variant="glass" size="sm" @click="applySimulation(250, 0.05)">High Jitter (Mixed)</BaseButton>
          <div class="spacer"></div>
          <BaseButton variant="danger" size="sm" @click="applySimulation(0, 0)">Reset Environment</BaseButton>
        </div>
      </BaseCard>
    </div>
  </div>
</template>

<style scoped>
.stats-view {
  height: 100%;
  padding: 32px 40px;
  display: flex;
  flex-direction: column;
  gap: 32px;
  overflow-y: auto;
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.view-title {
  font-size: 1.75rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.view-subtitle {
  font-size: 0.9rem;
  color: var(--text-secondary);
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(16, 185, 129, 0.1);
  padding: 8px 16px;
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 700;
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.pulse-dot {
  width: 8px;
  height: 8px;
  background: #10b981;
  border-radius: 50%;
  box-shadow: 0 0 10px #10b981;
  animation: pulse-glow 2s infinite;
}

@keyframes pulse-glow {
  0% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.5); opacity: 0.5; }
  100% { transform: scale(1); opacity: 1; }
}

/* Dashboard Grid */
.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 24px;
}

.full-width {
  grid-column: span 2;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.header-main {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-main h3 {
  font-size: 0.9rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-secondary);
}

.current-value {
  font-family: var(--font-mono);
  font-size: 1.25rem;
  font-weight: 800;
  color: var(--accent-primary);
}

/* Sparkline */
.graph-container {
  height: 100px;
  width: 100%;
  margin-bottom: 12px;
}

.spark-svg {
  width: 100%;
  height: 100%;
  overflow: visible;
}

.graph-line {
  filter: drop-shadow(0 0 4px var(--accent-primary));
  stroke-dasharray: 200;
  stroke-dashoffset: 200;
  animation: draw-line 2s ease-out forwards, pulse-glow 3s infinite;
}

@keyframes draw-line {
  to { stroke-dashoffset: 0; }
}

@keyframes pulse-glow {
  0%, 100% { filter: drop-shadow(0 0 4px var(--accent-primary)); opacity: 1; }
  50% { filter: drop-shadow(0 0 12px var(--accent-primary)); opacity: 0.8; }
}

.graph-footer {
  display: flex;
  justify-content: space-between;
  font-size: 0.65rem;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-secondary);
  opacity: 0.5;
}

/* Subgrid Metrics */
.stats-subgrid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.stat-mini-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.mini-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.65rem;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-secondary);
  opacity: 0.7;
}

.mini-value {
  font-size: 1.5rem;
  font-weight: 800;
  color: var(--text-primary);
}

.ms { font-size: 0.7rem; color: var(--text-secondary); margin-left: 2px; }

.pro-progress {
  height: 4px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 2px;
  margin-top: 4px;
}

.pro-fill {
  height: 100%;
  background: var(--accent-primary);
  border-radius: 2px;
}

.pro-fill.success { background: #10b981; }

.worker-badges {
  display: flex;
  gap: 4px;
  margin-top: 4px;
}

.worker-pip {
  width: 10px;
  height: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 1px;
}

.worker-pip.active {
  background: var(--accent-primary);
  box-shadow: 0 0 5px var(--accent-primary);
}

.latency-indicator {
  font-size: 0.7rem;
  font-weight: 800;
  margin-top: 4px;
}

.latency-indicator.ultra { color: #10b981; }
.latency-indicator.ok { color: #22c55e; }
.latency-indicator.warn { color: #f59e0b; }
.latency-indicator.error { color: #ef4444; }

/* Heatmap */
.heatmap-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(14px, 1fr));
  gap: 4px;
  max-height: 120px;
  overflow-y: auto;
  margin-bottom: 20px;
}

.heat-pixel {
  aspect-ratio: 1;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.05);
  transition: all 0.2s;
}

.heat-pixel.active-fast { background: #10b981; }
.heat-pixel.active-stable { background: #f59e0b; }
.heat-pixel.active-slow { background: #ef4444; }
.heat-pixel.active-idle { background: var(--accent-primary); }
.heat-pixel.completed { background: rgba(255, 255, 255, 0.2); }

.heat-pixel {
  animation: pixel-entry 0.5s ease-out both;
  animation-delay: var(--delay, 0s);
}

@keyframes pixel-entry {
  from { opacity: 0; transform: scale(0); }
  to { opacity: 1; transform: scale(1); }
}

.heatmap-legend {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 0.65rem;
  font-weight: 700;
  color: var(--text-secondary);
}

.legend-pip { width: 8px; height: 8px; border-radius: 1px; }
.legend-pip.fast { background: #10b981; }
.legend-pip.ok { background: #f59e0b; }
.legend-pip.slow { background: #ef4444; }
.legend-pip.done { background: rgba(255, 255, 255, 0.2); }

/* Sniffer */
.sniffer-card {
  display: flex;
  flex-direction: column;
}

.sniffer-badge {
  font-size: 0.6rem;
  font-weight: 900;
  background: var(--text-primary);
  color: var(--bg-primary);
  padding: 2px 6px;
  border-radius: 4px;
}

.terminal-shell {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  height: 200px;
  overflow-y: auto;
  padding: 16px;
  font-family: var(--font-mono);
  font-size: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.terminal-waiting {
  opacity: 0.3;
  text-align: center;
  margin-top: 60px;
  font-style: italic;
}

.terminal-line {
  display: flex;
  gap: 16px;
  white-space: nowrap;
}

.line-time { color: var(--text-secondary); opacity: 0.6; }
.line-proc { color: var(--accent-primary); font-weight: 700; }
.line-url { color: var(--text-primary); text-overflow: ellipsis; overflow: hidden; }

/* Simulation */
.simulation-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.spacer { flex: 1; }

/* Empty State */
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-card {
  max-width: 500px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.empty-icon { color: var(--text-secondary); opacity: 0.2; }

/* Custom Scrollbar */
.terminal-shell::-webkit-scrollbar,
.stats-view::-webkit-scrollbar {
  width: 6px;
}
</style>

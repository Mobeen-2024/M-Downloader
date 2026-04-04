<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useDownloadStore } from '@/stores/download.store';
import { 
  PhCpu, 
  PhHardDrive, 
  PhActivity,
  PhWarning,
  PhShieldCheck,
  PhClockCounterClockwise,
  PhTerminalWindow,
  PhStack,
  PhBroadcast,
  PhTrendUp
} from "@phosphor-icons/vue";
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
let telemetryInterval: any = null;
let snifferPromise: Promise<any> | null = null;

// Context switching: Clear buffer when target download changes
watch(() => activeDownload.value?.id, () => {
  speedHistory.value = new Array(60).fill(0);
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
  if (ms === 0) return { label: 'IDLE', color: 'text-text-dim/40' };
  if (ms < 50) return { label: 'ULTRA_LOW', color: 'text-tactical-cyan' };
  if (ms < 150) return { label: 'STABLE', color: 'text-white/60' };
  return { label: 'JITTER_DETECTED', color: 'text-hazard-orange' };
});

const getSegmentColor = (seg: any) => {
  if (seg.state === 'Completed') return 'rgba(0, 242, 255, 0.1)';
  if (seg.state === 'Active') {
    const lat = seg.last_latency_ms;
    if (lat === 0) return 'var(--color-tactical-cyan)';
    if (lat < 100) return '#10b981';
    if (lat < 300) return '#f59e0b';
    return '#ef4444';
  }
  return 'rgba(255,255,255,0.02)';
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
  const date = new Date(ts * 1000);
  return date.toTimeString().split(' ')[0] + '.' + date.getMilliseconds().toString().padStart(3, '0');
};

const formatSpeed = (bps: number) => {
  if (bps === 0) return '0 B/S';
  const k = 1024;
  const sizes = ['B/S', 'KB/S', 'MB/S', 'GB/S'];
  const i = Math.floor(Math.log(bps) / Math.log(k));
  return parseFloat((bps / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

onMounted(async () => {
  snifferPromise = listen('sniffer-hit', (event: any) => {
    snifferLogs.value.unshift(event.payload);
    if (snifferLogs.value.length > 50) snifferLogs.value.pop();
  });
  
  telemetryInterval = setInterval(() => {
    const currentSpeed = activeDownload.value?.speed_bps || 0;
    speedHistory.value.push(currentSpeed);
    if (speedHistory.value.length > 60) speedHistory.value.shift();
  }, 1000);

  try {
    isSnifferActive.value = await invoke('get_sniffer_status');
  } catch (e) {
    console.error('Status check failed:', e);
  }
});

onUnmounted(async () => {
  if (snifferPromise) {
    const unlisten = await snifferPromise;
    unlisten();
  }
  if (telemetryInterval) clearInterval(telemetryInterval);
});
</script>

<template>
  <div class="h-full flex flex-col p-8 md:p-12 gap-10 overflow-y-auto select-none">
    <!-- Header -->
    <header class="flex justify-between items-end shrink-0">
      <div class="space-y-1">
        <h2 class="text-2xl md:text-3xl font-black uppercase tracking-tighter text-white">System Instrumentation</h2>
        <p class="text-[10px] font-bold text-white/40 tracking-widest uppercase flex items-center gap-2">
          <PhTrendUp :size="12" class="text-tactical-cyan" />
          Real-time Kernel Telemetry & Engine Performance Analysis
        </p>
      </div>

      <div v-if="isSnifferActive" class="px-3 py-1 bg-tactical-cyan/10 border border-tactical-cyan/20 rounded flex items-center gap-2">
        <div class="w-1.5 h-1.5 bg-tactical-cyan rounded-full animate-pulse shadow-[0_0_8px_rgba(0,242,255,0.8)]"></div>
        <span class="text-[9px] font-data font-black text-tactical-cyan uppercase tracking-widest">WFP_Driver_Active</span>
      </div>
    </header>

    <!-- Empty State -->
    <div v-if="!activeDownload && !snifferLogs.length" class="flex-1 flex flex-col items-center justify-center opacity-10 gap-6">
      <PhActivity :size="80" weight="thin" />
      <div class="text-center space-y-2">
        <p class="text-lg font-black tracking-[0.4em] uppercase">Awaiting Signal</p>
        <p class="text-[10px] font-bold tracking-widest uppercase max-w-sm mx-auto">No active telemetry packets detected. Start a transmission or browse the web to initiate data intercept.</p>
      </div>
    </div>

    <!-- Dashboard -->
    <div v-else class="grid grid-cols-1 lg:grid-cols-2 gap-6 pb-20">
      <!-- Throughput Visualizer -->
      <section class="lg:col-span-2 glass-panel hover-glow rounded-2xl p-6 md:p-8 space-y-8 overflow-hidden relative">
        <div class="flex justify-between items-start z-10 relative">
          <div class="flex items-center gap-3">
            <PhClockCounterClockwise :size="20" weight="duotone" class="text-tactical-cyan" />
            <h3 class="text-xs font-black uppercase tracking-widest text-white">Throughput Stability</h3>
          </div>
          <div class="text-2xl font-data font-black text-white tracking-widest">{{ formatSpeed(activeDownload?.speed_bps || 0) }}</div>
        </div>
        
        <div class="h-40 w-full z-10 relative">
          <svg viewBox="0 0 100 30" preserveAspectRatio="none" class="w-full h-full opacity-60">
            <defs>
              <linearGradient id="areaGrad" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="var(--color-tactical-cyan)" stop-opacity="0.3" />
                <stop offset="100%" stop-color="var(--color-tactical-cyan)" stop-opacity="0" />
              </linearGradient>
            </defs>
            <path :d="`M ${sparklinePoints} L 100,30 L 0,30 Z`" fill="url(#areaGrad)" />
            <polyline fill="none" stroke="var(--color-tactical-cyan)" stroke-width="0.5" :points="sparklinePoints" class="shadow-[0_0_10px_rgba(0,242,255,0.5)]" />
          </svg>
        </div>

        <div class="flex justify-between items-center text-[9px] font-black uppercase tracking-[0.3em] text-white/10 z-10 relative">
          <span>60S_Realtime_Window</span>
          <span>Adaptive_Scaling (BPS/S)</span>
        </div>
      </section>

      <!-- Metrics Grid -->
      <section class="lg:col-span-2 grid grid-cols-2 md:grid-cols-4 gap-4">
        <div class="glass-panel hover-glow rounded-2xl p-5 space-y-4">
          <div class="flex items-center gap-2 text-[10px] font-black text-white/30 uppercase tracking-widest">
            <PhHardDrive :size="14" weight="duotone" class="text-tactical-cyan/40" />
            Disk I/O
          </div>
          <div class="text-lg font-data font-black text-white">{{ (metrics.io_efficiency * 100).toFixed(1) }} <span class="text-[10px] font-bold text-white/20 ml-1">%</span></div>
          <div class="h-1 w-full bg-white/5 rounded-full overflow-hidden">
            <div class="h-full bg-tactical-cyan transition-all duration-500" :style="{ width: metrics.io_efficiency * 100 + '%' }"></div>
          </div>
        </div>

        <div class="glass-panel hover-glow rounded-2xl p-5 space-y-4">
          <div class="flex items-center gap-2 text-[10px] font-black text-white/30 uppercase tracking-widest">
            <PhShieldCheck :size="14" weight="duotone" class="text-terminal-green/40" />
            Reliability
          </div>
          <div class="text-lg font-data font-black text-white">{{ efficiencyIndex }} <span class="text-[10px] font-bold text-white/20 ml-1">%</span></div>
          <div class="h-1 w-full bg-white/5 rounded-full overflow-hidden">
            <div class="h-full bg-terminal-green transition-all duration-500" :style="{ width: efficiencyIndex + '%' }"></div>
          </div>
        </div>

        <div class="glass-panel hover-glow rounded-2xl p-5 space-y-4">
          <div class="flex items-center gap-2 text-[10px] font-black text-white/30 uppercase tracking-widest">
            <PhCpu :size="14" weight="duotone" class="text-tactical-cyan/40" />
            CPU_Nodes
          </div>
          <div class="text-lg font-data font-black text-white">{{ metrics.active_workers }} <span class="text-[10px] font-bold text-white/20 ml-1 uppercase">Units</span></div>
          <div class="flex gap-1">
            <div v-for="i in 12" :key="i" class="flex-1 h-1 rounded-[1px] transition-colors"
                 :class="i <= metrics.active_workers ? 'bg-tactical-cyan' : 'bg-white/5'"></div>
          </div>
        </div>

        <div class="glass-panel hover-glow rounded-2xl p-5 space-y-4">
          <div class="flex items-center gap-2 text-[10px] font-black text-white/30 uppercase tracking-widest">
            <PhBroadcast :size="14" weight="duotone" class="text-tactical-cyan/40" />
            Latency
          </div>
          <div class="text-lg font-data font-black text-white">{{ metrics.avg_latency_ms }} <span class="text-[10px] font-bold text-white/20 ml-1">MS</span></div>
          <div class="text-[9px] font-black uppercase tracking-widest leading-none" :class="latencyStatus.color">
            {{ latencyStatus.label }}
          </div>
        </div>
      </section>

      <!-- Latency Heatmap -->
      <section v-if="activeDownload" class="glass-panel hover-glow rounded-2xl p-6 space-y-6">
        <div class="flex justify-between items-center">
          <div class="flex items-center gap-3">
            <PhStack :size="20" weight="duotone" class="text-tactical-cyan" />
            <h3 class="text-xs font-black uppercase tracking-widest text-white">Segment Cycles</h3>
          </div>
          <span class="text-[9px] font-black uppercase tracking-[0.2em] text-white/20 px-2 py-0.5 border border-white/5 rounded">Core_S_Grid</span>
        </div>
        <div class="grid grid-cols-8 md:grid-cols-12 gap-2">
          <div 
            v-for="(seg, idx) in activeDownload.segments" 
            :key="idx" 
            class="aspect-square rounded-[2px] transition-colors duration-300"
            :style="{ backgroundColor: getSegmentColor(seg) }"
          ></div>
        </div>
      </section>

      <!-- Sniffer Logs -->
      <section class="glass-panel hover-glow rounded-2xl p-6 space-y-6 flex flex-col"
               :class="{ 'lg:col-span-2': !activeDownload }">
        <div class="flex justify-between items-center shrink-0">
          <div class="flex items-center gap-3">
            <PhTerminalWindow :size="20" weight="duotone" class="text-tactical-cyan" />
            <h3 class="text-xs font-black uppercase tracking-widest text-white">Log Matrix (Interception)</h3>
          </div>
          <span class="text-[9px] font-black uppercase tracking-[0.2em] text-hazard-orange/60 px-2 py-0.5 border border-hazard-orange/20 rounded">System_Ring_0</span>
        </div>
        
        <div class="flex-1 min-h-[200px] bg-black/40 border border-white/5 rounded-xl p-4 font-data text-[10px] overflow-y-auto space-y-2 scrollbar-tactical">
          <div v-if="!snifferLogs.length" class="h-full flex items-center justify-center italic text-white/5 uppercase tracking-widest">
            Initializing WFP Signal Buffer...
          </div>
          <div v-for="(log, i) in snifferLogs" :key="i" class="grid grid-cols-[80px_100px_1fr] gap-4 py-1 border-b border-white/[0.02] last:border-0 group">
            <span class="text-white/20 tracking-tighter">{{ formatTime(log.timestamp) }}</span>
            <span class="text-tactical-cyan/60 font-black truncate">{{ log.process_name.toUpperCase() }}</span>
            <span class="text-white/40 truncate group-hover:text-white/80 transition-colors">{{ log.url }}</span>
          </div>
        </div>
      </section>

      <!-- Simulation Engine -->
      <section class="lg:col-span-2 glass-panel hover-glow rounded-2xl p-6 space-y-6 border-red-500/20">
        <div class="flex items-center gap-3">
          <PhWarning :size="20" weight="duotone" class="text-hazard-orange" />
          <h3 class="text-xs font-black uppercase tracking-widest text-white">Adversarial Environment Simulation</h3>
        </div>
        <div class="flex flex-wrap gap-4">
          <BaseButton @click="applySimulation(500, 0)" class="!bg-white/5 !text-white/60 !text-[11px] !font-black !px-4 hover:!bg-white/10">LATENCY (500MS)</BaseButton>
          <BaseButton @click="applySimulation(0, 0.1)" class="!bg-white/5 !text-white/60 !text-[11px] !font-black !px-4 hover:!bg-white/10">PACKET LOSS (10%)</BaseButton>
          <BaseButton @click="applySimulation(250, 0.05)" class="!bg-white/5 !text-white/60 !text-[11px] !font-black !px-4 hover:!bg-white/10">SYNCED JITTER</BaseButton>
          <div class="hidden md:block flex-1"></div>
          <BaseButton @click="applySimulation(0, 0)" class="!bg-red-500/20 !text-red-500 !text-[11px] !font-black !px-6 border !border-red-500/30 hover:!bg-red-500 hover:!text-black transition-all">PURGE SIMULATION</BaseButton>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.scrollbar-tactical::-webkit-scrollbar { width: 4px; }
.scrollbar-tactical::-webkit-scrollbar-track { background: transparent; }
.scrollbar-tactical::-webkit-scrollbar-thumb { background: rgba(0, 242, 255, 0.1); border-radius: 10px; }
.scrollbar-tactical::-webkit-scrollbar-thumb:hover { background: rgba(0, 242, 255, 0.3); }
</style>

<style scoped>
/* Motion animations handled via onMounted if needed, or simple CSS transitions */
</style>

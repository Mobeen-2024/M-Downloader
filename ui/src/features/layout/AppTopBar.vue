<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useDownloadStore } from '@/stores/download.store';
import { useSettingsStore } from '@/stores/settings.store';
import { useFormatters } from '@/composables/useFormatters';
import { invoke } from '@tauri-apps/api/core';
import SpeedGraph from '@/features/shared/components/SpeedGraph.vue';
import { 
  PhPlus, 
  PhBroadcast, 
  PhActivity, 
  PhLightning, 
  PhClipboardText, 
  PhGauge, 
  PhDeviceMobile
} from "@phosphor-icons/vue";
import { useClipboardMonitor } from '@/composables/useClipboardMonitor';
import { useGrabberStore } from '@/stores/grabber.store';

const store = useDownloadStore();
const grabberStore = useGrabberStore();
const settings = useSettingsStore();
const { formatSpeed } = useFormatters();
const { isEnabled: isClipboardEnabled, detectedUrl, startMonitoring, stopMonitoring, clearDetected } = useClipboardMonitor();

const isLimitEnabled = ref(false);
const speedLimitKbps = ref(1024); // Default 1 MB/s
const depStatus = ref({ ffmpeg_found: false, ffprobe_found: false });
const isDevMobile = ref(false);

const checkDeps = async () => {
  try {
    depStatus.value = await invoke('check_dependencies');
  } catch (e) {
    console.error('Failed to check dependencies:', e);
  }
};

const toggleMobileView = () => {
  isDevMobile.value = !isDevMobile.value;
  document.documentElement.classList.toggle('mobile-optimized', isDevMobile.value);
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
  <div class="h-full px-6 flex items-center justify-between font-ui select-none" style="-webkit-app-region: drag;">
    <!-- Left: Real-time Telemetry -->
    <div class="flex items-center gap-8" style="-webkit-app-region: drag;">
      <!-- Throughput -->
      <div class="flex items-center gap-3">
        <div class="p-1.5 bg-tactical-cyan/10 rounded border border-tactical-cyan/20">
          <PhActivity :size="16" weight="duotone" class="text-tactical-cyan" />
        </div>
        <div class="flex flex-col">
          <span class="text-[9px] uppercase font-black text-white/30 leading-none mb-1">Global Throughput</span>
          <span class="text-sm font-data font-black text-tactical-cyan tracking-tight leading-none">{{ formatSpeed(store.totalSpeed) }}</span>
        </div>
        <div class="w-12 h-6 opacity-40">
          <SpeedGraph :current-speed="store.totalSpeed" />
        </div>
      </div>

      <div class="w-[1px] h-8 bg-white/5"></div>

      <!-- Active Nodes -->
      <div class="flex items-center gap-3">
        <div class="p-1.5 bg-white/5 rounded border border-white/5">
          <PhBroadcast :size="16" weight="duotone" class="text-white/60" />
        </div>
        <div class="flex flex-col">
          <span class="text-[9px] uppercase font-black text-white/30 leading-none mb-1">Active Nodes</span>
          <span class="text-sm font-data font-black text-white tracking-tight leading-none">{{ store.activeDownloads.length }} Units</span>
        </div>
      </div>

      <div class="w-[1px] h-8 bg-white/5"></div>

      <!-- Engine Status -->
      <div class="flex items-center gap-3" :class="{ 'opacity-50 grayscale': !depStatus.ffmpeg_found }">
        <div class="p-1.5 rounded border" :class="depStatus.ffmpeg_found ? 'bg-terminal-green/10 border-terminal-green/20' : 'bg-red-500/10 border-red-500/20'">
          <PhLightning :size="16" weight="duotone" :class="depStatus.ffmpeg_found ? 'text-terminal-green' : 'text-red-500'" />
        </div>
        <div class="flex flex-col">
          <span class="text-[9px] uppercase font-black text-white/30 leading-none mb-1">Engine Pipeline</span>
          <span class="text-sm font-data font-black tracking-tight leading-none" :class="depStatus.ffmpeg_found ? 'text-terminal-green' : 'text-red-500'">{{ depStatus.ffmpeg_found ? 'NOMINAL' : 'FAULT' }}</span>
        </div>
      </div>

      <!-- Swarm Overlay (Conditional) -->
      <template v-if="grabberStore.isCrawling">
        <div class="w-[1px] h-8 bg-white/5"></div>
        <div class="flex items-center gap-3 px-3 py-1.5 bg-tactical-cyan/5 border border-tactical-cyan/10 rounded-lg animate-tactical-glow">
          <PhBroadcast :size="16" weight="fill" class="text-tactical-cyan" />
          <div class="flex flex-col">
            <span class="text-[8px] uppercase font-black text-tactical-cyan/60 leading-none mb-1">Swarm Active</span>
            <span class="text-xs font-data font-black text-white leading-none">{{ grabberStore.activeWorkers }} Nodes</span>
          </div>
        </div>
      </template>
    </div>

    <!-- Right: Tactical Controls -->
    <div class="flex items-center gap-4" style="-webkit-app-region: no-drag;">
      <div class="flex items-center gap-2 pr-4 border-r border-white/5">
        <!-- Throttle -->
        <div class="relative group">
          <button 
            class="flex items-center gap-2 px-3 py-1.5 rounded-md transition-all border"
            :class="isLimitEnabled ? 'bg-tactical-cyan/20 border-tactical-cyan/30 text-tactical-cyan' : 'bg-white/5 border-white/5 text-text-dim hover:text-white'"
            @click="isLimitEnabled = !isLimitEnabled"
          >
            <PhGauge :size="18" weight="duotone" />
            <div class="flex flex-col items-start leading-tight">
              <span class="text-[8px] uppercase font-black opacity-60">Throttle</span>
              <span class="text-[10px] font-data font-bold">{{ isLimitEnabled ? speedLimitKbps + ' KB/S' : 'UNLIMITED' }}</span>
            </div>
          </button>
          
          <div v-if="isLimitEnabled" class="absolute top-full left-0 mt-2 w-48 p-4 bg-bg-surface border border-white/10 rounded-xl glass shadow-2xl z-[100] group-hover:block hidden">
            <input type="range" v-model.number="speedLimitKbps" min="100" max="10240" step="100" class="w-full h-1 bg-white/10 rounded-lg appearance-none cursor-pointer accent-tactical-cyan" />
          </div>
        </div>

        <!-- Clipboard Monitor -->
        <div class="relative">
          <button 
            class="flex items-center gap-2 px-3 py-1.5 rounded-md transition-all border"
            :class="isClipboardEnabled ? 'bg-tactical-cyan/20 border-tactical-cyan/30 text-tactical-cyan' : 'bg-white/5 border-white/5 text-text-dim hover:text-white'"
            @click="toggleClipboard"
          >
            <PhClipboardText :size="18" weight="duotone" />
            <div class="flex flex-col items-start leading-tight">
              <span class="text-[8px] uppercase font-black opacity-60">Bio-Monitor</span>
              <span class="text-[10px] font-data font-bold">{{ isClipboardEnabled ? 'ENGAGED' : 'OFF' }}</span>
            </div>
          </button>
          
          <Transition name="pop-in">
            <div v-if="detectedUrl" class="absolute top-full right-0 mt-2 w-64 p-4 bg-bg-surface border border-white/10 rounded-xl glass shadow-2xl z-[100]">
              <div class="space-y-4">
                <div class="flex items-center gap-2">
                  <span class="w-2 h-2 rounded-full bg-tactical-cyan animate-pulse"></span>
                  <span class="text-[10px] font-black uppercase text-tactical-cyan tracking-widest">Packet Intercepted</span>
                </div>
                <div class="p-2 bg-black/40 rounded border border-white/5">
                  <p class="text-[11px] font-medium truncate opacity-60">{{ detectedUrl }}</p>
                </div>
                <div class="flex gap-2">
                  <button class="flex-1 py-1.5 bg-tactical-cyan text-bg-deep text-[10px] font-black uppercase rounded shadow-[0_0_15px_rgba(0,242,255,0.4)]" 
                          @click="$emit('new-download', detectedUrl); clearDetected()">CAPTURE MISSION</button>
                  <button class="flex-1 py-1.5 bg-white/5 text-white/60 text-[10px] font-black uppercase rounded border border-white/5" 
                          @click="clearDetected()">IGNORE</button>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center gap-2">
        <button 
          @click="toggleMobileView"
          class="p-2 rounded-md border border-white/5 transition-all"
          :class="isDevMobile ? 'bg-tactical-cyan/20 text-tactical-cyan border-tactical-cyan/30' : 'bg-white/5 text-text-dim hover:text-white'"
        >
          <PhDeviceMobile :size="18" weight="duotone" />
        </button>
        
        <button 
          @click="$emit('new-download')"
          class="flex items-center gap-2 px-4 py-2 bg-tactical-cyan text-bg-deep rounded-md shadow-[0_0_20px_rgba(0,242,255,0.2)] transition-all hover:scale-[1.02] active:scale-[0.98]"
        >
          <PhPlus :size="18" weight="bold" />
          <span class="text-xs font-black uppercase tracking-tight">New Mission</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pop-in-enter-active, .pop-in-leave-active { transition: all 0.2s cubic-bezier(0.19, 1, 0.22, 1); }
.pop-in-enter-from, .pop-in-leave-to { opacity: 0; transform: translateY(10px) scale(0.95); }
</style>

<style scoped>
/* Scoped styles removed in favor of CSS Modules */
</style>

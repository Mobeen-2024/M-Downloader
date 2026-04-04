<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute } from 'vue-router';
import { useDownloadStore } from '@/stores/download.store';
import { useDownload } from '@/composables/useDownload';
import DownloadCard from '@/features/downloads/DownloadCard.vue';
import { 
  PhStack, 
  PhLightning, 
  PhPlus, 
  PhTrendUp 
} from "@phosphor-icons/vue";
import type { DownloadItem } from '@/types/download';
import BaseDialog from '@/features/shared/components/BaseDialog.vue';

const route = useRoute();

const store = useDownloadStore();
const { refreshDownload } = useDownload();

const showRefreshModal = ref(false);
const refreshTarget = ref<DownloadItem | null>(null);
const newUrlInput = ref('');
const isInitialLoading = ref(true);

const visibleDownloads = computed(() => {
  const cat = route.query.category as string;
  if (cat) {
    if (cat === 'All') return store.downloads;
    return store.downloads.filter(d => d.category === cat);
  }
  return store.activeDownloads;
});

onMounted(async () => {
  setTimeout(() => { isInitialLoading.value = false; }, 600);
});

const handleRefreshClick = (download: DownloadItem) => {
  refreshTarget.value = download;
  newUrlInput.value = download.url;
  showRefreshModal.value = true;
};

const handleRefreshSubmit = async () => {
  if (refreshTarget.value && newUrlInput.value) {
    await refreshDownload(refreshTarget.value.id, newUrlInput.value);
    showRefreshModal.value = false;
    refreshTarget.value = null;
  }
};

defineEmits(['new-download']);
</script>

<template>
  <div class="h-full flex flex-col p-8 md:p-12 gap-8 overflow-hidden select-none">
    <!-- Header -->
    <div class="flex flex-col md:flex-row justify-between items-start md:items-end gap-6 shrink-0">
      <div class="space-y-1">
        <h2 class="text-2xl md:text-3xl font-black uppercase tracking-tighter text-white">
          {{ $route.query.category ? `Telemetry Data: ${$route.query.category}` : 'Active Transmissions' }}
        </h2>
        <div class="flex items-center gap-2 text-xs font-data font-bold text-tactical-cyan/60 uppercase">
          <PhTrendUp :size="14" />
          <span>{{ visibleDownloads.length }} UNITS {{ $route.query.category ? 'MATCHING CRITERIA' : 'CURRENTLY ACCELERATING' }}</span>
        </div>
      </div>
      
      <div class="flex items-center gap-3">
        <button 
          v-if="!store.queue_active"
          @click="store.start_queue"
          class="flex items-center gap-2 px-4 py-2 bg-white/5 border border-tactical-cyan/40 text-tactical-cyan rounded-lg text-xs font-black uppercase hover:bg-tactical-cyan/10 transition-all"
        >
          <PhLightning :size="14" weight="duotone" />
          Engage Queue
        </button>
        <button 
          @click="$emit('new-download')"
          class="flex items-center gap-2 px-4 py-2 bg-tactical-cyan text-bg-deep rounded-lg text-xs font-black uppercase shadow-[0_0_20px_rgba(0,242,255,0.2)] hover:scale-[1.02] active:scale-[0.98] transition-all"
        >
          <PhPlus :size="14" weight="bold" />
          New Capture
        </button>
      </div>
    </div>

    <!-- Body Area -->
    <div class="flex-1 overflow-y-auto pr-2 -mr-2">
      <!-- Skeleton Loading -->
      <div v-if="isInitialLoading" class="flex flex-col gap-4 max-w-5xl mx-auto w-full">
        <div v-for="i in 3" :key="i" class="h-28 bg-white/5 border border-white/5 rounded-2xl animate-pulse"></div>
      </div>

      <!-- Empty State -->
      <div v-else-if="visibleDownloads.length === 0" 
           class="flex flex-col items-center justify-center gap-8 text-center glass-panel hover-glow rounded-3xl py-20">
        <div class="p-6 bg-white/5 rounded-full text-white/10">
          <PhStack :size="64" weight="duotone" />
        </div>
        <div class="space-y-2">
          <h3 class="text-lg font-black uppercase text-white tracking-widest">{{ $route.query.category ? 'Telemetry Blank' : 'Engine Standby' }}</h3>
          <p class="text-xs text-text-dim max-w-xs mx-auto leading-relaxed">{{ $route.query.category ? 'No history or active transmissions matching this telemetry constraint.' : 'No active transmissions detected. Bandwidth is currently unutilized in the transmission buffer.' }}</p>
        </div>
        <button 
          @click="$emit('new-download')"
          class="flex items-center gap-3 px-6 py-3 bg-white/5 border border-white/10 text-white rounded-xl text-xs font-black uppercase hover:bg-white/10 transition-all"
        >
          <PhPlus :size="16" weight="bold" class="text-tactical-cyan" />
          Initiate Capture
        </button>
      </div>

      <!-- Active Download List -->
      <div v-else class="flex flex-col gap-4 max-w-5xl mx-auto w-full pb-10">
        <div v-if="$route.query.category" class="px-4 py-2 mb-2 border border-tactical-cyan/30 bg-tactical-cyan/10 rounded-lg text-tactical-cyan text-xs font-bold flex justify-between items-center">
          <span>Telemetry Filter Active: {{ $route.query.category }} (Omni-State View)</span>
          <button @click="$router.push('/active')" class="hover:text-white uppercase text-[10px] tracking-widest">Clear Filter</button>
        </div>
        <TransitionGroup name="list-move">
          <DownloadCard
            v-for="download in visibleDownloads"
            :key="download.id"
            :download="download"
            @refresh="handleRefreshClick"
          />
        </TransitionGroup>
      </div>
    </div>

    <!-- Refresh URL Dialog -->
    <BaseDialog
      :show="showRefreshModal"
      title="Signal Restoration"
      @close="showRefreshModal = false"
    >
      <div class="space-y-6">
        <p class="text-xs text-text-dim leading-relaxed">
          A transmission fault was detected. Provide a fresh signal source to restore the packet stream for unit: <strong class="text-tactical-cyan uppercase">{{ refreshTarget?.name }}</strong>.
        </p>
        <div class="space-y-2">
          <label class="text-[9px] font-black uppercase text-white/30 tracking-widest">New Source Signal (URL)</label>
          <input 
            v-model="newUrlInput"
            placeholder="PASTE REFRESHED LINK..."
            class="w-full bg-black/40 border border-white/10 rounded-lg p-3 text-sm font-data text-tactical-cyan outline-none focus:border-tactical-cyan/40 transition-all"
            @keyup.enter="handleRefreshSubmit"
          />
        </div>
      </div>

      <template #footer>
        <div class="flex gap-3 justify-end w-full">
          <button @click="showRefreshModal = false" 
                  class="px-4 py-2 text-[10px] font-black uppercase text-white/40 hover:text-white transition-colors">Discard</button>
          <button @click="handleRefreshSubmit" 
                  class="px-6 py-2 bg-tactical-cyan text-bg-deep rounded text-[10px] font-black uppercase shadow-[0_0_15px_rgba(0,242,255,0.3)]">Apply & Resume</button>
        </div>
      </template>
    </BaseDialog>
  </div>
</template>

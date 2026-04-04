<script setup lang="ts">
import { 
  PhPause, 
  PhPlay, 
  PhX, 
  PhFileText, 
  PhArchive, 
  PhFileCode, 
  PhVideoCamera, 
  PhImage,
  PhFolderOpen,
  PhArrowsClockwise,
  PhCaretUp,
  PhCaretDown
} from "@phosphor-icons/vue";
import { computed } from 'vue';
import type { DownloadItem } from '@/types/download';
import { useFormatters } from '@/composables/useFormatters';
import { useDownload } from '@/composables/useDownload';
import { useDownloadStore } from '@/stores/download.store';
import StatusBadge from '@/features/shared/components/StatusBadge.vue';
import SegmentVisualizer from './SegmentVisualizer.vue';

const props = defineProps<{
  download: DownloadItem;
  isQueuedView?: boolean;
}>();

const emit = defineEmits(['refresh', 'new-download']);

const { formatSize, formatSpeed, formatEta } = useFormatters();
const { pauseDownload, resumeDownload, cancelDownload } = useDownload();
const store = useDownloadStore();

const fileIcon = computed(() => {
  const ext = props.download.name.split('.').pop()?.toLowerCase();
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext || '')) return PhArchive;
  if (['mp4', 'mkv', 'avi', 'mov'].includes(ext || '')) return PhVideoCamera;
  if (['png', 'jpg', 'jpeg', 'gif', 'webp'].includes(ext || '')) return PhImage;
  if (['exe', 'msi', 'dmg'].includes(ext || '')) return PhFileCode;
  return PhFileText;
});

const progressPercent = computed(() => {
  if (props.download.total === 0) return 0;
  return (props.download.downloaded / props.download.total) * 100;
});
</script>

<template>
  <div class="group relative glass-panel hover-glow rounded-2xl p-4 md:p-5 flex flex-col md:flex-row gap-4 md:gap-5">
    <!-- Icon Layer (Hidden on extra small mobile to save space) -->
    <div class="hidden sm:block shrink-0 pt-1">
      <div class="p-3 bg-black/40 border border-white/5 rounded-xl text-tactical-cyan/60 group-hover:text-tactical-cyan group-hover:border-tactical-cyan/20 transition-all duration-300">
        <component :is="fileIcon" :size="24" weight="duotone" />
      </div>
    </div>

    <!-- Content Engine -->
    <div class="flex-1 min-w-0 flex flex-col gap-3">
      <!-- Header Info -->
      <div class="flex flex-col sm:flex-row justify-between items-start gap-4">
        <div class="min-w-0 w-full sm:w-auto space-y-1">
          <h4 class="text-sm font-black text-white truncate uppercase tracking-tight" :title="download.name">
            {{ download.name }}
          </h4>
          <div class="flex items-center gap-2 md:gap-3 text-[10px] font-bold text-text-dim flex-wrap">
            <span class="text-tactical-cyan font-data">{{ formatSize(download.total) }}</span>
            <span class="opacity-10 hidden xs:inline">|</span>
            <span class="truncate max-w-[150px] md:max-w-[300px] opacity-40 font-data">{{ download.url }}</span>
          </div>
        </div>

        <div class="flex items-center justify-between sm:justify-end gap-3 w-full sm:w-auto shrink-0 border-t sm:border-t-0 border-white/5 pt-3 sm:pt-0">
          <StatusBadge :status="download.status" />
          
          <!-- Tactical Controls -->
          <div class="flex items-center gap-1 p-1 bg-black/40 rounded-lg border border-white/5">
            <template v-if="isQueuedView">
              <button class="p-1.5 hover:text-tactical-cyan transition-colors" @click="store.move_up(download.id)"><PhCaretUp :size="12" weight="bold" /></button>
              <button class="p-1.5 hover:text-tactical-cyan transition-colors" @click="store.move_down(download.id)"><PhCaretDown :size="12" weight="bold" /></button>
            </template>

            <button v-if="download.status === 'RefreshNeeded'" 
                    class="p-1.5 text-hazard-orange hover:bg-hazard-orange/10 rounded transition-colors"
                    @click="$emit('refresh', download)">
              <PhArrowsClockwise :size="14" weight="bold" />
            </button>

            <button v-if="download.status === 'Downloading'" 
                    class="p-1.5 hover:text-tactical-cyan transition-colors"
                    @click="pauseDownload(download.id)">
              <PhPause :size="14" weight="bold" />
            </button>

            <button v-else-if="download.status === 'Paused' || download.status === 'Queued'" 
                    class="p-1.5 hover:text-tactical-cyan transition-colors"
                    @click="resumeDownload(download.id)">
              <PhPlay :size="14" weight="bold" />
            </button>

            <button v-if="download.status === 'Completed'" 
                    class="p-1.5 hover:text-tactical-cyan transition-colors">
              <PhFolderOpen :size="14" weight="duotone" />
            </button>

            <button class="p-1.5 text-red-500/60 hover:text-red-500 hover:bg-red-500/10 rounded transition-colors" 
                    @click="cancelDownload(download.id)">
              <PhX :size="14" weight="bold" />
            </button>
          </div>
        </div>
      </div>

      <!-- Telemetry Visualizer -->
      <div class="space-y-3 pt-1">
        <div class="flex justify-between items-end gap-2 flex-wrap">
          <div class="flex items-baseline gap-3">
            <span class="text-xs font-data font-black text-white leading-none">
              {{ progressPercent.toFixed(1) }}<span class="text-[9px] opacity-40 ml-0.5">%</span>
            </span>
            <span v-if="download.status === 'Downloading'" class="text-[10px] font-data font-bold text-tactical-cyan whitespace-nowrap">
              {{ formatSpeed(download.speed_bps) }}
            </span>
          </div>
          <span v-if="download.status === 'Downloading'" class="text-[9px] font-data font-bold text-white/30 uppercase tracking-widest leading-none">
            T-MINUS: {{ formatEta(download.downloaded, download.total, download.speed_bps) }}
          </span>
        </div>

        <div class="h-1.5 w-full bg-white/5 rounded-full overflow-hidden relative border border-white/5">
          <div class="absolute inset-y-0 left-0 bg-tactical-cyan/20 shadow-[0_0_10px_rgba(0,242,255,0.2)] transition-all duration-300"
               :style="{ width: progressPercent + '%' }"></div>
          <SegmentVisualizer 
            :segments="download.segments" 
            :total="download.total" 
            class="h-full w-full opacity-40"
          />
        </div>

        <div class="flex justify-between items-center text-[9px] font-black uppercase tracking-widest text-white/20">
          <div class="flex gap-4 flex-wrap">
            <span>Flow: <span class="text-white/40 font-data">{{ formatSize(download.downloaded) }}</span> / {{ formatSize(download.total) }}</span>
            <span v-if="download.segments?.length" class="text-terminal-green/40 hidden xs:inline">
              Cycles: {{ download.segments.length }}
            </span>
          </div>
          <span v-if="download.status === 'Completed'" class="text-terminal-green shrink-0 whitespace-nowrap">Transmission Complete</span>
        </div>
      </div>
    </div>
  </div>
</template>

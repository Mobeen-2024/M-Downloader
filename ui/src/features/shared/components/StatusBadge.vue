<script setup lang="ts">
import type { DownloadStatus } from '@/types/download';

const props = defineProps<{
  status: DownloadStatus | string;
  text?: string;
}>();

const statusMap: Record<string, { color: string, bg: string, border: string }> = {
  downloading: { color: 'text-emerald-400', bg: 'bg-emerald-400/10', border: 'border-emerald-400/20' },
  paused: { color: 'text-amber-400', bg: 'bg-amber-400/10', border: 'border-amber-400/20' },
  completed: { color: 'text-tactical-cyan', bg: 'bg-tactical-cyan/10', border: 'border-tactical-cyan/20' },
  error: { color: 'text-red-500', bg: 'bg-red-500/10', border: 'border-red-500/20' },
  queued: { color: 'text-white/40', bg: 'bg-white/5', border: 'border-white/10' },
  active: { color: 'text-emerald-400', bg: 'bg-emerald-400/10', border: 'border-emerald-400/20' },
};

const currentStatus = (props.status || 'queued').toLowerCase();
const style = statusMap[currentStatus] || statusMap.queued;
</script>

<template>
  <div 
    class="inline-flex items-center gap-2 px-3 py-1 rounded-full border text-[9px] font-black uppercase tracking-widest transition-all duration-300"
    :class="[style.color, style.bg, style.border]"
  >
    <span 
      class="w-1.5 h-1.5 rounded-full" 
      :class="[
        style.color.replace('text-', 'bg-'),
        { 'animate-pulse shadow-[0_0_8px_currentColor]': currentStatus === 'downloading' || currentStatus === 'active' }
      ]"
    ></span>
    <span>{{ text || status }}</span>
  </div>
</template>

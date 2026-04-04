<script setup lang="ts">
import { useDownloadStore } from '@/stores/download.store';
import { 
  PhVideo, 
  PhLightning, 
  PhDownloadSimple, 
  PhX 
} from "@phosphor-icons/vue";
import { invoke } from '@tauri-apps/api/core';

const store = useDownloadStore();

const handleDownload = async (media: any, resolution?: any) => {
  const url = resolution ? resolution.video_url : media.url;
  try {
    const id = await invoke('start_download', { 
      url, 
      cookies: media.cookies || null,
      referer: media.referrer || null 
    });
    store.addDownload(url, id as string);
    store.clearMediaDetection(media.id);
  } catch (e) {
    console.error('Failed to start media download:', e);
  }
};

const dismiss = (id: string) => {
  store.clearMediaDetection(id);
};
</script>

<template>
  <div class="fixed top-24 right-8 flex flex-col gap-4 z-[100] pointer-events-none">
    <TransitionGroup 
      enter-active-class="transition duration-500 cubic-bezier(0.16, 1, 0.3, 1)"
      enter-from-class="opacity-0 translate-x-12 scale-95"
      enter-to-class="opacity-100 translate-x-0 scale-100"
      leave-active-class="transition duration-300 ease-in"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div 
        v-for="media in store.interceptedMedia" 
        :key="media.url" 
        class="w-80 bg-[#050505]/95 backdrop-blur-2xl border border-white/10 rounded-[1.5rem] shadow-[0_20px_50px_rgba(0,0,0,0.5)] p-5 space-y-4 pointer-events-auto overflow-hidden relative group"
      >
        <div class="absolute inset-0 bg-gradient-to-br from-tactical-cyan/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
        
        <header class="flex items-center justify-between border-b border-white/5 pb-3 relative z-10">
          <div class="flex items-center gap-2 text-[9px] font-black uppercase text-tactical-cyan tracking-[0.2em]">
            <PhVideo :size="16" weight="duotone" class="animate-pulse" />
            <span>Signal Intercepted</span>
          </div>
          <button @click="dismiss(media.id)" class="text-white/20 hover:text-white transition-colors">
            <PhX :size="14" weight="bold" />
          </button>
        </header>

        <div class="space-y-4 relative z-10">
          <h4 class="text-[11px] font-data font-black text-white truncate uppercase tracking-tight">{{ media.filename }}</h4>
          
          <div v-if="media.resolutions && media.resolutions.length" class="grid grid-cols-2 gap-2">
            <div 
              v-for="res in media.resolutions" 
              :key="res.label"
              class="flex items-center gap-2 p-2 bg-white/5 border border-white/5 rounded-xl hover:bg-tactical-cyan hover:text-black transition-all cursor-pointer group/res"
              @click="handleDownload(media, res)"
            >
              <PhLightning :size="12" weight="duotone" class="text-tactical-cyan group-hover/res:text-black" />
              <span class="text-[9px] font-black tracking-widest">{{ res.label }}</span>
              <PhDownloadSimple :size="12" weight="bold" class="ml-auto opacity-0 group-hover/res:opacity-100 transition-opacity" />
            </div>
          </div>

          <button 
            v-else 
            @click="handleDownload(media)" 
            class="w-full py-2.5 bg-tactical-cyan text-black text-[10px] font-black uppercase rounded-xl shadow-[0_0_15px_rgba(0,242,255,0.3)] hover:scale-[1.02] active:scale-95 transition-all flex items-center justify-center gap-2"
          >
            <PhDownloadSimple :size="18" weight="bold" />
            Initialize Stream
          </button>
        </div>
      </div>
    </TransitionGroup>
  </div>
</template>

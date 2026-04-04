<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useDownloadStore } from '@/stores/download.store';
import { invoke } from '@tauri-apps/api/core';
import { 
  PhWaves,
  PhX
} from "@phosphor-icons/vue";
import BaseButton from '@/features/shared/components/BaseButton.vue';

const store = useDownloadStore();

const currentMedia = computed(() => store.interceptedMedia[0] || null);
const selectedResolution = ref<any>(null);
const isAudioOnly = ref(false);

// Auto-sync resolution selection when new media arrives
watch(currentMedia, (newVal) => {
  if (newVal && newVal.resolutions) {
    selectedResolution.value = newVal.resolutions[0];
  }
}, { immediate: true });

async function startMediaDownload() {
  if (!currentMedia.value || !selectedResolution.value) return;
  
  const m = currentMedia.value;
  const res = selectedResolution.value;
  
  try {
    const id = await invoke('start_download', { 
      url: isAudioOnly.value && res.audio_url 
        ? res.audio_url 
        : res.video_url,
      cookies: m.cookies || null,
      referer: m.referer || m.referrer || null
    });
    
    // Add to store and clear from HUD
    store.addDownload(m.url, id as string);
    store.clearMediaDetection(m.id);
  } catch (e) {
    console.error("Failed to start media download:", e);
  }
}

function dismiss() {
  if (currentMedia.value) {
    store.clearMediaDetection(currentMedia.value.id);
  }
}
</script>

<template>
  <Transition 
    enter-active-class="transition duration-500 cubic-bezier(0.16, 1, 0.3, 1)"
    enter-from-class="opacity-0 translate-x-12 scale-95"
    enter-to-class="opacity-100 translate-x-0 scale-100"
    leave-active-class="transition duration-300 ease-in"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-95"
  >
    <div v-if="currentMedia || store.isAnalyzingMedia" class="fixed top-24 right-8 w-80 bg-[#050505]/95 backdrop-blur-2xl border border-white/10 rounded-[2rem] shadow-[0_20px_50px_rgba(0,0,0,0.5)] overflow-hidden z-[100] flex flex-col">
      <!-- Loading State -->
      <div v-if="store.isAnalyzingMedia" class="p-8 flex flex-col items-center gap-6 text-center">
        <div class="relative w-12 h-12 flex items-center justify-center">
          <div class="absolute inset-0 bg-tactical-cyan/20 rounded-full animate-ping"></div>
          <div class="w-2 h-2 bg-tactical-cyan rounded-full shadow-[0_0_15px_#00f2ff]"></div>
        </div>
        <div class="space-y-2">
          <h4 class="text-[10px] font-black uppercase text-tactical-cyan tracking-[0.2em]">Intercepting Signal...</h4>
          <p class="text-[9px] font-bold text-white/30 tracking-widest leading-relaxed uppercase">Solving signatures and extracting stream metadata.</p>
        </div>
      </div>

      <!-- Detection State -->
      <div v-else-if="currentMedia" class="flex flex-col">
        <header class="px-6 py-5 bg-white/5 border-b border-white/5 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-xl bg-tactical-cyan/10 flex items-center justify-center text-tactical-cyan shadow-inner">
              <PhWaves :size="18" weight="duotone" class="animate-pulse" />
            </div>
            <h4 class="text-[10px] font-black uppercase text-white tracking-[0.2em]">Signal Detected</h4>
          </div>
          <button @click="dismiss" class="text-white/20 hover:text-white transition-colors">
            <PhX :size="14" weight="bold" />
          </button>
        </header>

        <div class="p-6 space-y-6">
          <div class="space-y-1">
            <p class="text-[9px] font-black text-white/30 uppercase tracking-widest">{{ currentMedia.mime || 'Unidentified Codec' }}</p>
            <p class="text-[11px] font-data font-black text-white truncate uppercase tracking-tight">{{ currentMedia.filename || 'Adaptive Buffer' }}</p>
          </div>

          <div v-if="currentMedia.resolutions?.length" class="space-y-3">
            <label class="text-[9px] font-black uppercase text-white/30 tracking-[0.2em]">Output Parameters</label>
            <div class="flex flex-wrap gap-2">
              <button 
                v-for="res in currentMedia.resolutions" 
                :key="res.label"
                class="px-3 py-1.5 rounded-lg text-[9px] font-black tracking-widest transition-all border"
                :class="selectedResolution?.label === res.label 
                  ? 'bg-tactical-cyan border-tactical-cyan text-black shadow-[0_0_15px_rgba(0,242,255,0.3)]' 
                  : 'bg-white/5 border-white/5 text-white/40 hover:bg-white/10 hover:text-white'"
                @click="selectedResolution = res"
              >
                {{ res.label }}
              </button>
            </div>
          </div>

          <div class="pt-4 border-t border-white/5">
            <label class="flex items-center gap-3 cursor-pointer group select-none">
              <input 
                type="checkbox" 
                v-model="isAudioOnly" 
                :disabled="!selectedResolution?.audio_url"
                class="appearance-none w-4 h-4 rounded-md bg-white/5 border border-white/10 checked:bg-tactical-cyan checked:border-tactical-cyan transition-all cursor-pointer disabled:opacity-20"
              />
              <span class="text-[10px] font-black uppercase tracking-widest text-white/40 group-hover:text-white/60 transition-colors">
                Extract Audio Buffer Only
              </span>
            </label>
          </div>

          <div class="flex gap-2 pt-2">
            <BaseButton variant="ghost" class="flex-1 !h-10 text-[10px] font-black tracking-widest" @click="dismiss">IGNORE</BaseButton>
            <BaseButton 
              variant="primary" 
              class="flex-2 !h-10 text-[10px] font-black tracking-widest !rounded-xl"
              @click="startMediaDownload" 
              :disabled="!selectedResolution"
            >
              EXECUTE CAPTURE
            </BaseButton>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

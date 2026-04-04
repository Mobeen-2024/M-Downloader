<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { 
  PhMinus, 
  PhCornersOut, 
  PhCornersIn, 
  PhX 
} from "@phosphor-icons/vue";
import { useRoute } from 'vue-router';

const isTauri = typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__;
const appWindow = isTauri ? getCurrentWindow() : null;
const isMaximized = ref(false);
const route = useRoute();
let resizePromise: Promise<any> | null = null;

const minimize = () => isTauri && appWindow?.minimize();
const toggleMaximize = async () => {
  if (!isTauri || !appWindow) return;
  await appWindow.toggleMaximize();
  isMaximized.value = await appWindow.isMaximized();
};
const close = () => isTauri && appWindow?.close();

onMounted(async () => {
  if (isTauri && appWindow) {
    isMaximized.value = await appWindow.isMaximized();
    resizePromise = appWindow.onResized(async () => {
      isMaximized.value = await appWindow.isMaximized();
    });
  }
});

onUnmounted(async () => {
  if (resizePromise) {
    const unlisten = await resizePromise;
    unlisten();
  }
});
</script>

<template>
  <div class="h-10 bg-black/40 backdrop-blur-md border-b border-white/5 flex justify-between items-center px-6 select-none z-[200] relative" data-tauri-drag-region>
    <div class="flex items-center gap-3 pointer-events-none">
      <div class="w-1.5 h-1.5 bg-tactical-cyan rounded-full shadow-[0_0_8px_#00f2ff] animate-pulse"></div>
      <div class="flex items-baseline gap-2">
        <span class="text-[10px] font-black uppercase text-white tracking-[0.3em] font-data">M-Downloader</span>
        <span class="text-[9px] font-black uppercase text-white/10 tracking-[0.2em] italic transition-all group-hover:text-white/20">
          // {{ route.name || 'CORE_OPS' }}
        </span>
      </div>
    </div>

    <!-- Window Controls -->
    <div class="flex h-full" data-tauri-drag-region="false">
      <button 
        class="h-full w-12 flex items-center justify-center text-white/20 hover:bg-white/5 hover:text-white transition-all" 
        title="Minimize" 
        @click="minimize"
      >
        <PhMinus :size="14" weight="bold" />
      </button>
      <button 
        class="h-full w-12 flex items-center justify-center text-white/20 hover:bg-white/5 hover:text-white transition-all" 
        :title="isMaximized ? 'Restore' : 'Maximize'" 
        @click="toggleMaximize"
      >
        <component :is="isMaximized ? PhCornersIn : PhCornersOut" :size="14" weight="bold" />
      </button>
      <button 
        class="h-full w-12 flex items-center justify-center text-white/20 hover:bg-red-500 hover:text-white transition-all" 
        title="Close" 
        @click="close"
      >
        <PhX :size="14" weight="bold" />
      </button>
    </div>
  </div>
</template>

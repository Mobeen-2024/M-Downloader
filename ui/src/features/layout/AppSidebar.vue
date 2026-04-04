<script setup lang="ts">
import { 
  PhDownloadSimple, 
  PhCheckCircle, 
  PhClock, 
  PhChartBar, 
  PhArchive, 
  PhMusicNotes, 
  PhVideoCamera, 
  PhFileCode, 
  PhFileText, 
  PhPackage, 
  PhShieldCheck, 
  PhArrowsClockwise, 
  PhGearSix,
  PhHardDrive
} from "@phosphor-icons/vue";
import { useDownloadStore } from '@/stores/download.store';
import { useRoute, useRouter } from 'vue-router';

const store = useDownloadStore();
const route = useRoute();
const router = useRouter();

const navItems = [
  { name: 'Active', icon: PhDownloadSimple, path: '/active', badge: () => store.activeDownloads.length },
  { name: 'Completed', icon: PhCheckCircle, path: '/completed' },
  { name: 'Queued', icon: PhClock, path: '/queued' },
  { name: 'Site Grabber', icon: PhArrowsClockwise, path: '/grabber' },
  { name: 'Statistics', icon: PhChartBar, path: '/stats' },
  { name: 'Sites', icon: PhShieldCheck, path: '/sites' },
];

const categories = [
  { name: 'Compressed', icon: PhArchive, count: () => store.categoryCounts.Compressed },
  { name: 'Music', icon: PhMusicNotes, count: () => store.categoryCounts.Music },
  { name: 'Video', icon: PhVideoCamera, count: () => store.categoryCounts.Video },
  { name: 'Programs', icon: PhFileCode, count: () => store.categoryCounts.Programs },
  { name: 'Documents', icon: PhFileText, count: () => store.categoryCounts.Documents },
  { name: 'General', icon: PhPackage, count: () => store.categoryCounts.General },
];
</script>

<template>
  <div class="h-full flex flex-col font-ui select-none">
    <!-- Header -->
    <div class="p-6 flex items-center gap-3">
      <div class="p-2 bg-tactical-cyan/20 border border-tactical-cyan/30 rounded-md animate-tactical-glow">
        <PhDownloadSimple :size="24" weight="duotone" class="text-tactical-cyan" />
      </div>
      <div class="flex flex-col">
        <h1 class="text-xs uppercase font-black tracking-tighter text-white">Mdownloader</h1>
        <span class="text-[9px] font-data text-tactical-cyan/60 font-bold uppercase leading-none">v2.1 PROTO-SPEC</span>
      </div>
    </div>

    <!-- Scrollable Content -->
    <div class="flex-1 overflow-y-auto overflow-x-hidden space-y-8 px-4">
      <!-- Section: Core Systems -->
      <div>
        <div class="px-3 mb-2 text-[10px] font-black uppercase tracking-widest text-white/30">Core Systems</div>
        <div class="space-y-1">
          <router-link 
            v-for="item in navItems" 
            :key="item.path"
            :to="item.path"
            class="flex items-center justify-between px-3 py-2.5 rounded-md transition-all group"
            :class="route.path === item.path 
              ? 'bg-tactical-cyan/10 border border-tactical-cyan/20 text-tactical-cyan' 
              : 'text-text-dim hover:bg-white/5 hover:text-white'"
          >
            <div class="flex items-center gap-3">
              <component :is="item.icon" :size="18" weight="duotone" class="group-hover:scale-110 transition-transform" />
              <span class="text-xs font-bold">{{ item.name }}</span>
            </div>
            <div v-if="item.badge && item.badge() > 0" 
                 class="px-1.5 py-0.5 rounded text-[10px] font-data font-black bg-tactical-cyan text-bg-deep">
              {{ item.badge() }}
            </div>
          </router-link>
        </div>
      </div>

      <!-- Section: Telemetry Class -->
      <div>
        <div class="px-3 mb-2 text-[10px] font-black uppercase tracking-widest text-white/30">Telemetry Class</div>
        <div class="space-y-1">
          <div 
            v-for="cat in categories" 
            :key="cat.name"
            class="flex items-center justify-between px-3 py-2 text-text-dim hover:bg-tactical-cyan/10 hover:text-tactical-cyan transition-colors cursor-pointer rounded-md group"
            @click="() => {
              console.log(`[Telemetry Module] Category clicked: ${cat.name} (Count: ${cat.count()})`);
              router.push({ path: '/active', query: { category: cat.name } });
            }"
          >
            <div class="flex items-center gap-3">
              <component :is="cat.icon" :size="16" weight="duotone" class="opacity-50 group-hover:opacity-100" />
              <span class="text-xs font-semibold">{{ cat.name }}</span>
            </div>
            <span v-if="cat.count() > 0" class="text-[10px] font-data font-bold opacity-40 group-hover:opacity-100">{{ cat.count() }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="p-4 mt-auto border-t border-white/5 space-y-4">
      <div class="p-3 rounded-lg bg-black/40 border border-white/5 space-y-2">
        <div class="flex items-center justify-between text-[10px] font-bold">
          <div class="flex items-center gap-1.5 text-text-dim uppercase">
            <PhHardDrive :size="12" weight="duotone" class="text-tactical-cyan" />
            <span>Local Storage</span>
          </div>
          <span class="text-tactical-cyan">84%</span>
        </div>
        <div class="h-1 w-full bg-white/5 rounded-full overflow-hidden">
          <div class="h-full bg-tactical-cyan/40 shadow-[0_0_8px_rgba(0,242,255,0.3)]" style="width: 84%"></div>
        </div>
      </div>

      <router-link to="/settings" 
                   class="flex items-center gap-3 px-3 py-2 text-xs font-bold transition-all rounded-md"
                   :class="route.path === '/settings' 
                    ? 'bg-tactical-cyan/10 border border-tactical-cyan/20 text-tactical-cyan' 
                    : 'text-text-dim hover:bg-white/5 hover:text-white'">
        <PhGearSix :size="20" weight="duotone" />
        <span>System Parameters</span>
      </router-link>
    </div>
  </div>
</template>

<style scoped>
/* Scoped styles removed in favor of CSS Modules */
</style>

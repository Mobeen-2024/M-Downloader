<script setup lang="ts">
import { 
  PhDownloadSimple, 
  PhCheckCircle, 
  PhClock, 
  PhArrowsClockwise, 
  PhGearSix 
} from "@phosphor-icons/vue";
import { useDownloadStore } from '@/stores/download.store';

const store = useDownloadStore();

const navItems = [
  { name: 'Active', icon: PhDownloadSimple, path: '/active', badge: () => store.activeDownloads.length },
  { name: 'Done', icon: PhCheckCircle, path: '/completed' },
  { name: 'Queue', icon: PhClock, path: '/queued' },
  { name: 'Grab', icon: PhArrowsClockwise, path: '/grabber' },
  { name: 'Config', icon: PhGearSix, path: '/settings' },
];
</script>

<template>
  <nav class="fixed bottom-0 left-0 right-0 h-20 bg-[#050505]/95 backdrop-blur-2xl border-t border-white/5 flex justify-around items-center z-[150] px-6 pb-6 shadow-[0_-10px_40px_rgba(0,0,0,0.5)] lg:hidden">
    <router-link 
      v-for="item in navItems" 
      :key="item.path"
      :to="item.path"
      class="flex flex-col items-center gap-1.5 text-[9px] font-black uppercase tracking-[0.2em] transition-all relative group text-white/20 hover:text-white/40"
      active-class="!text-tactical-cyan"
    >
      <div class="relative p-1">
        <component :is="item.icon" :size="20" weight="duotone" class="group-hover:scale-110 transition-transform duration-300" />
        <div v-if="item.badge && item.badge() > 0" class="absolute -top-1 -right-2 min-w-[14px] h-[14px] px-1 bg-tactical-cyan text-black text-[8px] font-black flex items-center justify-center rounded-full shadow-[0_0_12px_#00f2ff]">
          {{ item.badge() }}
        </div>
      </div>
      <span>{{ item.name }}</span>
      
      <!-- Active Indicator -->
      <div class="absolute -bottom-2 left-1/2 -translate-x-1/2 w-6 h-[2px] bg-tactical-cyan shadow-[0_0_10px_#00f2ff] scale-x-0 group-[.router-link-active]:scale-x-100 transition-transform duration-500 rounded-full"></div>
    </router-link>
  </nav>
</template>

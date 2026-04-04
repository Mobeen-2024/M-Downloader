<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useDownloadStore } from '@/stores/download.store';
import DownloadCard from '@/features/downloads/DownloadCard.vue';
import { 
  PhClock, 
  PhPlay, 
  PhStop, 
  PhCpu, 
  PhGhost,
  PhStack
} from "@phosphor-icons/vue";
import { useSettingsStore } from '@/stores/settings.store';
import BaseButton from '@/features/shared/components/BaseButton.vue';
import BaseSkeleton from '@/features/shared/components/BaseSkeleton.vue';

const store = useDownloadStore();
const settings = useSettingsStore();
const isInitialLoading = ref(true);

onMounted(() => {
  setTimeout(() => { isInitialLoading.value = false; }, 500);
});

const toggleQueue = () => {
  if (store.queue_active) store.stop_queue();
  else store.start_queue();
};
</script>

<template>
  <div class="h-full flex flex-col p-8 md:p-12 gap-8 overflow-y-auto select-none">
    <!-- Header -->
    <header class="flex justify-between items-end shrink-0">
      <div class="space-y-1">
        <h2 class="text-2xl md:text-3xl font-black uppercase tracking-tighter text-white">Scheduler & Queue</h2>
        <p class="text-[10px] font-bold text-white/40 tracking-widest uppercase flex items-center gap-2">
          <PhCpu :size="12" class="text-tactical-cyan" />
          {{ store.queuedDownloads.length }} Units pending sequential execution
        </p>
      </div>

      <div class="px-3 py-1 bg-white/5 border border-white/5 rounded flex items-center gap-2 backdrop-blur-md">
        <div v-if="store.queue_active" class="w-1.5 h-1.5 bg-tactical-cyan rounded-full animate-pulse shadow-[0_0_8px_rgba(0,242,255,0.8)]"></div>
        <span class="text-[9px] font-data font-black text-white/40 uppercase tracking-widest">
          {{ store.queue_active ? 'Engine_Engaged' : 'Engine_Offline' }}
        </span>
      </div>
    </header>

    <!-- Skeleton -->
    <div v-if="isInitialLoading" class="space-y-6">
      <BaseSkeleton height="100px" borderRadius="16px" />
      <div class="space-y-4">
        <BaseSkeleton v-for="i in 3" :key="i" height="110px" borderRadius="16px" />
      </div>
    </div>

    <template v-else>
      <!-- Scheduler control card -->
      <section class="bg-white/5 border border-white/10 rounded-2xl p-6 flex justify-between items-center backdrop-blur-xl group hover:border-tactical-cyan/30 transition-all duration-500 overflow-hidden relative">
        <div class="absolute -top-10 -left-10 w-32 h-32 bg-tactical-cyan/5 rounded-full blur-3xl group-hover:bg-tactical-cyan/10 transition-all"></div>
        
        <div class="flex items-center gap-5 z-10">
          <div class="w-12 h-12 bg-black/40 border border-white/5 rounded-xl flex items-center justify-center text-tactical-cyan shadow-inner">
            <PhClock :size="24" weight="duotone" />
          </div>
          <div>
            <p class="text-[10px] font-black uppercase text-white/30 tracking-[0.2em]">System_Matrix</p>
            <p class="text-sm font-data font-black text-white">
              Concurrency: <span class="text-tactical-cyan">{{ settings.maxConnections > 2 ? '0.2x' : '0.1x' }}</span> Multiplier_Active
            </p>
          </div>
        </div>

        <BaseButton 
          @click="toggleQueue" 
          class="!px-8 !py-3 !rounded-xl !font-black !tracking-tighter !transition-all active:scale-95 z-10"
          :class="store.queue_active ? '!bg-red-500 !text-white' : '!bg-tactical-cyan !text-black'"
        >
          <template #icon-left>
            <component :is="store.queue_active ? PhStop : PhPlay" :size="20" weight="bold" />
          </template>
          {{ store.queue_active ? 'HALT_OPERATIONS' : 'RESUME_OPERATIONS' }}
        </BaseButton>
      </section>

      <!-- Empty state -->
      <div v-if="store.queuedDownloads.length === 0" 
           class="flex-1 flex flex-col items-center justify-center border-2 border-dashed border-white/5 rounded-3xl opacity-10 gap-6 py-20">
        <PhGhost :size="80" weight="thin" />
        <div class="text-center space-y-2">
          <p class="text-lg font-black tracking-[0.4em] uppercase">Registry Empty</p>
          <p class="text-[10px] font-bold tracking-widest uppercase">Scheduled tasks will appear here for processing</p>
        </div>
      </div>

      <!-- Queue list -->
      <div v-else class="space-y-4 pb-20">
        <div class="flex items-center gap-3 mb-6 text-[10px] font-black text-white/20 uppercase tracking-[0.3em]">
          <PhStack :size="14" weight="duotone" />
          <span>Execution_Chain // Priority_Sort</span>
        </div>

        <TransitionGroup name="list-move">
          <DownloadCard
            v-for="download in store.queuedDownloads"
            :key="download.id"
            :download="download"
            is-queued-view
            class="vault-item"
          />
        </TransitionGroup>
      </div>
    </template>
  </div>
</template>

<style scoped>
.list-move-enter-active,
.list-move-leave-active {
  transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}
.list-move-enter-from,
.list-move-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>

<style scoped>
.list-move-enter-active,
.list-move-leave-active {
  transition: all 0.5s ease;
}
.list-move-enter-from,
.list-move-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>

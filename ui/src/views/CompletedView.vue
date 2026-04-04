<script setup lang="ts">
import { useDownloadStore } from '@/stores/download.store';
import DownloadCard from '@/features/downloads/DownloadCard.vue';
import { 
  PhArchive, 
  PhTrash, 
  PhFolderSimple,
  PhStack
} from "@phosphor-icons/vue";
import BaseButton from '@/features/shared/components/BaseButton.vue';
import { onMounted, ref } from 'vue';
import { animate, stagger } from 'motion';

const store = useDownloadStore();
const gridRef = ref<HTMLElement | null>(null);

onMounted(() => {
  if (gridRef.value) {
    (animate as any)(
      '.vault-item',
      { opacity: [0, 1], y: [24, 0] },
      { delay: stagger(0.05), type: 'spring', stiffness: 280, damping: 28 }
    );
  }
});
</script>

<template>
  <div class="h-full flex flex-col p-8 md:p-12 gap-10 overflow-y-auto select-none">
    <!-- Header -->
    <header class="flex justify-between items-end shrink-0">
      <div class="space-y-1">
        <h2 class="text-2xl md:text-3xl font-black uppercase tracking-tighter text-white">Vault & Archive</h2>
        <p class="text-[10px] font-bold text-white/40 tracking-widest uppercase flex items-center gap-2">
          <PhFolderSimple :size="12" class="text-tactical-cyan" />
          {{ store.completedDownloads.length }} Assets successfully verified
        </p>
      </div>
      <BaseButton
        @click="store.clear_completed"
        v-if="store.completedDownloads.length > 0"
        class="!bg-red-500/10 !text-red-500 !border !border-red-500/20 !px-6 !rounded-xl hover:!bg-red-500 hover:!text-black transition-all"
      >
        <template #icon-left><PhTrash :size="16" weight="bold" /></template>
        PURGE HISTORY
      </BaseButton>
    </header>

    <!-- Content -->
    <div v-if="store.completedDownloads.length === 0" 
         class="flex-1 flex flex-col items-center justify-center opacity-10 gap-6 border-2 border-dashed border-white/5 rounded-3xl">
      <PhArchive :size="80" weight="thin" />
      <div class="text-center space-y-2">
        <p class="text-lg font-black tracking-[0.4em] uppercase">Archives Dormant</p>
        <p class="text-[10px] font-bold tracking-widest uppercase">Transmissions will appear here once verified</p>
      </div>
    </div>

    <div v-else class="space-y-4 pb-20" ref="gridRef">
      <div class="flex items-center gap-3 mb-6 text-[10px] font-black text-white/20 uppercase tracking-[0.3em]">
        <PhStack :size="14" weight="duotone" />
        <span>Persistence_Store // Index: {{ store.completedDownloads.length }}</span>
      </div>
      
      <TransitionGroup name="list-move">
        <DownloadCard
          v-for="download in store.completedDownloads"
          :key="download.id"
          :download="download"
          class="vault-item"
        />
      </TransitionGroup>
    </div>
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

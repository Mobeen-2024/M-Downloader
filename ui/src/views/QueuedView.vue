<script setup lang="ts">
import { useDownloadStore } from '@/stores/download.store';
import DownloadCard from '@/features/downloads/DownloadCard.vue';
import { Clock } from 'lucide-vue-next';

const store = useDownloadStore();
</script>

<template>
  <div class="view-container">
    <div v-if="store.queuedDownloads.length === 0" class="empty-state">
      <Clock class="empty-icon" />
      <h3>Queue is Empty</h3>
      <p>Downloads scheduled for later will appear here.</p>
    </div>

    <div v-else class="download-list">
      <DownloadCard 
        v-for="download in store.queuedDownloads" 
        :key="download.id" 
        :download="download" 
      />
    </div>
  </div>
</template>

<style scoped>
.view-container {
  height: 100%;
  overflow-y: auto;
  padding: 16px 32px 32px 32px;
}

.download-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 1000px;
  margin: 0 auto;
}

.empty-state {
  height: 60vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--text-secondary);
}

.empty-icon {
  width: 64px;
  height: 64px;
  margin-bottom: 24px;
  opacity: 0.1;
  color: var(--color-paused);
}

.empty-state h3 {
  font-size: 1.5rem;
  color: var(--text-primary);
  margin-bottom: 8px;
}
</style>

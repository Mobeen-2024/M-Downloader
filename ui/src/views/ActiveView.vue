<script setup lang="ts">
import { ref } from 'vue';
import { useDownloadStore } from '../stores/download.store';
import { useDownload } from '../composables/useDownload';
import DownloadCard from '../components/download/DownloadCard.vue';
import RefreshLinkModal from '../components/modals/RefreshLinkModal.vue';
import { Layers } from 'lucide-vue-next';
import type { DownloadItem } from '../types/download';

const store = useDownloadStore();
const { refreshDownload } = useDownload();

const showRefreshModal = ref(false);
const refreshTarget = ref<DownloadItem | null>(null);

const handleRefreshClick = (download: DownloadItem) => {
  refreshTarget.value = download;
  showRefreshModal.value = true;
};

const handleRefreshSubmit = async (newUrl: string) => {
  if (refreshTarget.value) {
    await refreshDownload(refreshTarget.value.id, newUrl);
    showRefreshModal.value = false;
    refreshTarget.value = null;
  }
};
</script>

<template>
  <div class="view-container">
    <div v-if="store.activeDownloads.length === 0" class="empty-state">
      <Layers class="empty-icon" />
      <h3>No Active Downloads</h3>
      <p>Click "New Download" to start accelerating your files.</p>
    </div>

    <div v-else class="download-list">
      <DownloadCard 
        v-for="download in store.activeDownloads" 
        :key="download.id" 
        :download="download" 
        @refresh="handleRefreshClick"
      />
    </div>

    <RefreshLinkModal
      v-if="refreshTarget"
      :show="showRefreshModal"
      :download-name="refreshTarget.name"
      @close="showRefreshModal = false"
      @refresh="handleRefreshSubmit"
    />
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
}

.empty-state h3 {
  font-size: 1.5rem;
  color: var(--text-primary);
  margin-bottom: 8px;
}
</style>

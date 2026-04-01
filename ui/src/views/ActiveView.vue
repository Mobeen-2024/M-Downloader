<script setup lang="ts">
import { ref } from 'vue';
import { useDownloadStore } from '@/stores/download.store';
import { useDownload } from '@/composables/useDownload';
import DownloadCard from '@/features/downloads/DownloadCard.vue';
import RefreshLinkModal from '@/features/shared/modals/RefreshLinkModal.vue';
import { Layers, Zap, Plus } from 'lucide-vue-next';
import type { DownloadItem } from '@/types/download';
import BaseButton from '@/features/shared/components/BaseButton.vue';

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

defineEmits(['new-download']);
</script>

<template>
  <div class="active-view">
    <div class="view-header">
      <div class="header-left">
        <h2 class="view-title">Active Transmissions</h2>
        <p class="view-subtitle">{{ store.activeDownloads.length }} items currently accelerating</p>
      </div>
      <div class="header-right">
        <BaseButton variant="glass" size="sm" @click="store.start_queue" v-if="!store.queue_active">
          <Zap :size="14" /> Resume Queue
        </BaseButton>
      </div>
    </div>

    <div v-if="store.activeDownloads.length === 0" class="empty-container">
      <div class="empty-visual">
        <div class="visual-glow"></div>
        <Layers :size="48" class="visual-icon" />
      </div>
      <div class="empty-text">
        <h3>Engine Standby</h3>
        <p>No active downloads detected. Your bandwidth is currently unutilized.</p>
        <BaseButton variant="primary" style="margin-top: 16px" @click="$emit('new-download')">
          <Plus :size="18" /> Start New Download
        </BaseButton>
      </div>
    </div>

    <div v-else class="items-grid">
      <TransitionGroup name="list-move">
        <DownloadCard 
          v-for="download in store.activeDownloads" 
          :key="download.id" 
          :download="download" 
          @refresh="handleRefreshClick"
        />
      </TransitionGroup>
    </div>

    <RefreshLinkModal
      v-if="refreshTarget"
      :show="showRefreshModal"
      :id="refreshTarget.id"
      :download-name="refreshTarget.name"
      :source-url="refreshTarget.url"
      @close="showRefreshModal = false"
      @refresh="handleRefreshSubmit"
    />
  </div>
</template>

<style scoped>
.active-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 32px 40px;
  overflow: hidden;
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 32px;
}

.view-title {
  font-size: 1.75rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.view-subtitle {
  font-size: 0.9rem;
  color: var(--text-secondary);
}

.items-grid {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-right: 12px;
  max-width: 1200px;
}

/* Empty State */
.empty-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 24px;
  transform: translateY(-40px);
}

.empty-visual {
  position: relative;
  width: 120px;
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.visual-glow {
  position: absolute;
  inset: 0;
  background: radial-gradient(circle, rgba(59, 130, 246, 0.15) 0%, transparent 70%);
  animation: breathe 4s ease-in-out infinite;
}

.visual-icon {
  color: var(--text-secondary);
  opacity: 0.2;
}

.empty-text {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.empty-text h3 {
  font-size: 1.5rem;
  font-weight: 800;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.empty-text p {
  color: var(--text-secondary);
  max-width: 300px;
  line-height: 1.6;
}

@keyframes breathe {
  0%, 100% { transform: scale(1); opacity: 0.5; }
  50% { transform: scale(1.2); opacity: 1; }
}



/* Custom Scrollbar */
.items-grid::-webkit-scrollbar {
  width: 6px;
}
</style>

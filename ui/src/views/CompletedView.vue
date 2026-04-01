<script setup lang="ts">
import { useDownloadStore } from '@/stores/download.store';
import DownloadCard from '@/features/downloads/DownloadCard.vue';
import { Trophy } from 'lucide-vue-next';
import BaseButton from '@/features/shared/components/BaseButton.vue';

const store = useDownloadStore();
</script>

<template>
  <div class="completed-view">
    <div class="view-header">
      <div class="header-left">
        <h2 class="view-title">Vault & Archive</h2>
        <p class="view-subtitle">{{ store.completedDownloads.length }} items successfully accelerated and verified.</p>
      </div>
      <div class="header-right">
        <BaseButton variant="glass" size="sm" @click="store.clear_completed" v-if="store.completedDownloads.length > 0">
          Clear History
        </BaseButton>
      </div>
    </div>

    <div v-if="store.completedDownloads.length === 0" class="empty-container">
      <div class="empty-visual">
        <div class="visual-glow success"></div>
        <Trophy :size="48" class="visual-icon success-icon" />
      </div>
      <div class="empty-text">
        <h3>Archives Empty</h3>
        <p>No completed transmissions found. Your accelerated assets will appear here once verified.</p>
      </div>
    </div>

    <div v-else class="items-grid">
      <TransitionGroup name="list-move">
        <DownloadCard 
          v-for="download in store.completedDownloads" 
          :key="download.id" 
          :download="download" 
        />
      </TransitionGroup>
    </div>
  </div>
</template>

<style scoped>
.completed-view {
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
  background: radial-gradient(circle, rgba(16, 185, 129, 0.1) 0%, transparent 70%);
  animation: breathe 4s ease-in-out infinite;
}

.visual-icon {
  color: var(--text-secondary);
  opacity: 0.2;
}

.success-icon {
  color: var(--color-downloading);
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
</style>

<script setup lang="ts">
import { useDownloadStore } from '@/stores/download.store';
import DownloadCard from '@/features/downloads/DownloadCard.vue';
import { Clock, Play, Square, Settings2 } from 'lucide-vue-next';
import { useSettingsStore } from '@/stores/settings.store';

const store = useDownloadStore();
const settings = useSettingsStore();

const toggleQueue = () => {
  if (store.queue_active) {
    store.stop_queue();
  } else {
    store.start_queue();
  }
};
</script>

<template>
  <div class="view-container">
    <div class="scheduler-toolbar glass-panel">
      <div class="toolbar-info">
        <Settings2 class="toolbar-icon" :size="20" />
        <div>
          <h3>Queue Scheduler</h3>
          <p>Sequential processing (parallel limit: {{ settings.maxConnections > 2 ? 2 : 1 }})</p>
        </div>
      </div>
      
      <div class="toolbar-actions">
        <div class="status-badge" :class="{ active: store.queue_active }">
          <span class="pulse-dot" v-if="store.queue_active"></span>
          {{ store.queue_active ? 'Scheduler Active' : 'Scheduler Idle' }}
        </div>
        
        <button 
          @click="toggleQueue" 
          class="action-btn"
          :class="store.queue_active ? 'stop' : 'start'"
        >
          <component :is="store.queue_active ? Square : Play" :size="16" />
          {{ store.queue_active ? 'Stop Queue' : 'Start Queue' }}
        </button>
      </div>
    </div>

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

.scheduler-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  margin-bottom: 32px;
  border-radius: 16px;
}

.toolbar-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.toolbar-info h3 {
  font-size: 1.1rem;
  font-weight: 700;
  margin: 0;
}

.toolbar-info p {
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin: 2px 0 0 0;
}

.toolbar-icon {
  color: var(--accent-primary);
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.status-badge.active {
  color: var(--color-success, #10b981);
  background: rgba(16, 185, 129, 0.1);
}

.pulse-dot {
  width: 8px;
  height: 8px;
  background: currentColor;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { transform: scale(0.95); opacity: 1; }
  50% { transform: scale(1.1); opacity: 0.5; }
  100% { transform: scale(0.95); opacity: 1; }
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border-radius: 10px;
  border: none;
  font-size: 0.85rem;
  font-weight: 700;
  cursor: pointer;
  transition: var(--transition-smooth);
}

.action-btn.start {
  background: var(--accent-primary);
  color: white;
}

.action-btn.stop {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.action-btn:hover {
  transform: translateY(-2px);
  filter: brightness(1.1);
}

.download-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 1000px;
  margin: 0 auto;
}

.empty-state {
  height: 40vh;
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

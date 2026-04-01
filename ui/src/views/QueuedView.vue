<script setup lang="ts">
import { useDownloadStore } from '@/stores/download.store';
import DownloadCard from '@/features/downloads/DownloadCard.vue';
import { Clock, Play, Square } from 'lucide-vue-next';
import { useSettingsStore } from '@/stores/settings.store';
import BaseButton from '@/features/shared/components/BaseButton.vue';
import BaseCard from '@/features/shared/components/BaseCard.vue';

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
  <div class="queued-view">
    <div class="view-header">
      <div class="header-left">
        <h2 class="view-title">Scheduler & Queue</h2>
        <p class="view-subtitle">{{ store.queuedDownloads.length }} items pending sequential execution.</p>
      </div>
      <div class="header-right">
        <div class="status-pill" :class="{ 'is-active': store.queue_active }">
          <div class="pulse-dot" v-if="store.queue_active"></div>
          <span>{{ store.queue_active ? 'Processing Active' : 'Scheduler Offline' }}</span>
        </div>
      </div>
    </div>

    <!-- Scheduler Control Card -->
    <BaseCard variant="glass" padding="md" class="scheduler-card">
      <div class="card-left">
        <div class="icon-orb">
          <Clock :size="20" class="text-accent" />
        </div>
        <div class="info-group">
          <span class="info-label">Current Configuration</span>
          <p class="info-val">Parallel Limit: <strong>{{ settings.maxConnections > 2 ? 2 : 1 }}</strong> Concurrent Segments</p>
        </div>
      </div>
      <div class="card-actions">
        <BaseButton 
          :variant="store.queue_active ? 'danger' : 'primary'" 
          @click="toggleQueue"
        >
          <template #icon-left>
            <component :is="store.queue_active ? Square : Play" :size="16" />
          </template>
          {{ store.queue_active ? 'Stop Engine' : 'Start Queue' }}
        </BaseButton>
      </div>
    </BaseCard>

    <div v-if="store.queuedDownloads.length === 0" class="empty-container">
      <div class="empty-visual">
        <div class="visual-glow"></div>
        <Clock :size="48" class="visual-icon" />
      </div>
      <div class="empty-text">
        <h3>Queue Empty</h3>
        <p>No pending accelerations detected. Scheduled tasks will appear here for automated processing.</p>
      </div>
    </div>

    <div v-else class="items-grid">
      <TransitionGroup name="list-move">
        <DownloadCard 
          v-for="download in store.queuedDownloads" 
          :key="download.id" 
          :download="download" 
          is-queued-view 
        />
      </TransitionGroup>
    </div>
  </div>
</template>

<style scoped>
.queued-view {
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

.status-pill {
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(255, 255, 255, 0.03);
  padding: 8px 16px;
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  transition: all 0.3s;
}

.status-pill.is-active {
  color: var(--color-downloading);
  background: rgba(16, 185, 129, 0.1);
  border-color: rgba(16, 185, 129, 0.2);
}

.pulse-dot {
  width: 8px;
  height: 8px;
  background: #10b981;
  border-radius: 50%;
  box-shadow: 0 0 10px #10b981;
  animation: pulse-glow 2s infinite;
}

@keyframes pulse-glow {
  0% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.5); opacity: 0.5; }
  100% { transform: scale(1); opacity: 1; }
}

/* Scheduler Card */
.scheduler-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
}

.card-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.icon-orb {
  width: 44px;
  height: 44px;
  background: rgba(59, 130, 246, 0.1);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.info-group {
  display: flex;
  flex-direction: column;
}

.info-label {
  font-size: 0.65rem;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-secondary);
  opacity: 0.6;
}

.info-val {
  font-size: 0.95rem;
  color: var(--text-primary);
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
  background: radial-gradient(circle, rgba(59, 130, 246, 0.1) 0%, transparent 70%);
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

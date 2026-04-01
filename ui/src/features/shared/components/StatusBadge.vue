<script setup lang="ts">
import type { DownloadStatus } from '@/types/download';

defineProps<{
  status: DownloadStatus;
}>();
</script>

<template>
  <div class="status-badge" :class="status.toLowerCase()">
    <span class="dot"></span>
    <span class="text">{{ status }}</span>
  </div>
</template>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 0.75rem;
  font-weight: 600;
  background: rgba(255, 255, 255, 0.05);
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-secondary);
}

.status-badge.downloading {
  color: var(--color-downloading);
  background: rgba(16, 185, 129, 0.1);
}
.status-badge.downloading .dot {
  background: var(--color-downloading);
  box-shadow: 0 0 8px var(--color-downloading);
  animation: pulse 2s infinite;
}

.status-badge.paused {
  color: var(--color-paused);
  background: rgba(245, 158, 11, 0.1);
}
.status-badge.paused .dot { background: var(--color-paused); }

.status-badge.completed {
  color: #3b82f6;
  background: rgba(59, 130, 246, 0.1);
}
.status-badge.completed .dot { background: #3b82f6; }

.status-badge.error {
  color: var(--color-error);
  background: rgba(239, 68, 68, 0.1);
}
.status-badge.error .dot { background: var(--color-error); }

@keyframes pulse {
  0% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(1.2); }
  100% { opacity: 1; transform: scale(1); }
}
</style>

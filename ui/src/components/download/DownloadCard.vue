<script setup lang="ts">
import { 
  Pause, 
  Play, 
  X, 
  FileText, 
  FileArchive, 
  FileCode, 
  FileVideo, 
  FileImage,
  FolderOpen
} from 'lucide-vue-next';
import { computed } from 'vue';
import type { DownloadItem } from '../../types/download';
import { useFormatters } from '../../composables/useFormatters';
import { useDownload } from '../../composables/useDownload';
import StatusBadge from '../ui/StatusBadge.vue';
import SegmentVisualizer from './SegmentVisualizer.vue';
import GlassPanel from '../ui/GlassPanel.vue';

const props = defineProps<{
  download: DownloadItem;
}>();

const { formatSize, formatSpeed, formatEta } = useFormatters();
const { pauseDownload, resumeDownload, cancelDownload } = useDownload();

const fileIcon = computed(() => {
  const ext = props.download.name.split('.').pop()?.toLowerCase();
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(ext || '')) return FileArchive;
  if (['mp4', 'mkv', 'avi', 'mov'].includes(ext || '')) return FileVideo;
  if (['png', 'jpg', 'jpeg', 'gif', 'webp'].includes(ext || '')) return FileImage;
  if (['exe', 'msi', 'dmg'].includes(ext || '')) return FileCode;
  return FileText;
});

const progressPercent = computed(() => {
  if (props.download.total === 0) return 0;
  return (props.download.downloaded / props.download.total) * 100;
});
</script>

<template>
  <GlassPanel class="download-card">
    <div class="card-header">
      <div class="file-identity">
        <div class="icon-box">
          <component :is="fileIcon" class="file-icon" />
        </div>
        <div class="file-meta">
          <h4 class="file-name" :title="download.name">{{ download.name }}</h4>
          <div class="file-sub-meta">
            <span>{{ formatSize(download.total) }}</span>
            <span class="dot"></span>
            <span class="text-secondary">{{ download.url }}</span>
          </div>
        </div>
      </div>

      <div class="card-actions">
        <StatusBadge :status="download.status" />
        <div class="button-group">
          <button 
            v-if="download.status === 'Downloading'" 
            class="btn-icon" 
            @click="pauseDownload(download.id)"
          >
            <Pause />
          </button>
          <button 
            v-else-if="download.status === 'Paused'" 
            class="btn-icon" 
            @click="resumeDownload(download.id)"
          >
            <Play />
          </button>
          <button 
            v-if="download.status === 'Completed'" 
            class="btn-icon success"
          >
            <FolderOpen />
          </button>
          <button class="btn-icon danger" @click="cancelDownload(download.id)">
            <X />
          </button>
        </div>
      </div>
    </div>

    <div class="card-body">
      <div class="progress-info">
        <div class="progress-labels">
          <span class="speed">{{ formatSpeed(download.speed_bps) }}</span>
          <span class="eta">ETA: {{ formatEta(download.downloaded, download.total, download.speed_bps) }}</span>
        </div>
        <div class="progress-bar-container">
          <div class="progress-bar" :style="{ width: progressPercent + '%' }"></div>
        </div>
      </div>

      <SegmentVisualizer :segments="download.segments" :total="download.total" />
    </div>
  </GlassPanel>
</template>

<style scoped>
.download-card {
  padding: 20px;
  margin-bottom: 16px;
  transition: transform 0.2s;
}

.download-card:hover {
  transform: translateY(-2px);
  border-color: rgba(59, 130, 246, 0.3);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.file-identity {
  display: flex;
  gap: 16px;
  flex: 1;
  min-width: 0;
}

.icon-box {
  width: 48px;
  height: 48px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent-primary);
  flex-shrink: 0;
}

.file-icon {
  width: 24px;
  height: 24px;
}

.file-meta {
  min-width: 0;
}

.file-name {
  font-size: 1rem;
  font-weight: 700;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-sub-meta {
  font-size: 0.75rem;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.dot {
  width: 3px;
  height: 3px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 50%;
}

.card-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 12px;
}

.button-group {
  display: flex;
  gap: 8px;
}

.btn-icon {
  width: 32px;
  height: 32px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-icon :deep(svg) {
  width: 16px;
  height: 16px;
}

.btn-icon:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
}

.btn-icon.danger:hover {
  color: var(--color-error);
  border-color: rgba(239, 68, 68, 0.3);
}

.btn-icon.success:hover {
  color: var(--color-downloading);
  border-color: rgba(16, 185, 129, 0.3);
}

.card-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.progress-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-labels {
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  font-weight: 600;
}

.speed {
  color: var(--accent-primary);
  font-family: var(--font-mono, monospace);
}

.eta {
  color: var(--text-secondary);
}

.progress-bar-container {
  height: 6px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 3px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-primary), #60a5fa);
  transition: width 0.3s ease;
  box-shadow: 0 0 10px rgba(59, 130, 246, 0.3);
}

.text-secondary {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>

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
  FolderOpen,
  RotateCw,
  ChevronUp,
  ChevronDown
} from 'lucide-vue-next';
import { computed } from 'vue';
import type { DownloadItem } from '@/types/download';
import { useFormatters } from '@/composables/useFormatters';
import { useDownload } from '@/composables/useDownload';
import { useDownloadStore } from '@/stores/download.store';
import StatusBadge from '@/features/shared/components/StatusBadge.vue';
import SegmentVisualizer from './SegmentVisualizer.vue';
import BaseCard from '@/features/shared/components/BaseCard.vue';
import BaseButton from '@/features/shared/components/BaseButton.vue';

const props = defineProps<{
  download: DownloadItem;
  isQueuedView?: boolean;
}>();

const emit = defineEmits(['refresh']);

const { formatSize, formatSpeed, formatEta } = useFormatters();
const { pauseDownload, resumeDownload, cancelDownload } = useDownload();
const store = useDownloadStore();

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
  <BaseCard variant="glass" padding="md" hoverable class="download-card">
    <div class="card-layout">
      <!-- Icon Section -->
      <div class="icon-section">
        <div class="icon-wrapper">
          <component :is="fileIcon" :size="24" />
        </div>
      </div>

      <!-- Main Content Section -->
      <div class="content-section">
        <div class="header-row">
          <div class="title-area">
            <h4 class="file-name" :title="download.name">{{ download.name }}</h4>
            <div class="metadata-row">
              <span class="size">{{ formatSize(download.total) }}</span>
              <span class="divider"></span>
              <span class="url" :title="download.url">{{ download.url }}</span>
            </div>
          </div>
          
          <div class="actions-area">
            <StatusBadge :status="download.status" />
            <div class="button-group">
              <template v-if="isQueuedView">
                <BaseButton variant="glass" size="icon" title="Move Up" @click="store.move_up(download.id)">
                  <ChevronUp :size="16" />
                </BaseButton>
                <BaseButton variant="glass" size="icon" title="Move Down" @click="store.move_down(download.id)">
                  <ChevronDown :size="16" />
                </BaseButton>
              </template>
              
              <BaseButton 
                v-if="download.status === 'RefreshNeeded'" 
                variant="danger" 
                size="icon"
                title="Refresh Link"
                @click="$emit('refresh', download)"
              >
                <RotateCw :size="16" />
              </BaseButton>

              <BaseButton 
                v-if="download.status === 'Downloading'" 
                variant="glass" 
                size="icon"
                @click="pauseDownload(download.id)"
              >
                <Pause :size="16" />
              </BaseButton>

              <BaseButton 
                v-else-if="download.status === 'Paused' || download.status === 'Queued'" 
                variant="glass" 
                size="icon"
                @click="resumeDownload(download.id)"
              >
                <Play :size="16" />
              </BaseButton>

              <BaseButton 
                v-if="download.status === 'Completed'" 
                variant="glass" 
                size="icon"
                class="success-text"
              >
                <FolderOpen :size="16" />
              </BaseButton>

              <BaseButton variant="danger" size="icon" @click="cancelDownload(download.id)">
                <X :size="16" />
              </BaseButton>
            </div>
          </div>
        </div>

        <div class="progress-section">
          <div class="stats-row">
            <div class="main-stats">
              <span class="speed">{{ formatSpeed(download.speed_bps) }}</span>
              <span class="percentage">{{ progressPercent.toFixed(1) }}%</span>
            </div>
            <span class="eta" v-if="download.status === 'Downloading'">
              {{ formatEta(download.downloaded, download.total, download.speed_bps) }}
            </span>
          </div>

          <SegmentVisualizer 
            :segments="download.segments" 
            :total="download.total" 
            class="visualizer"
          />

          <div class="details-row">
            <span class="transferred">{{ formatSize(download.downloaded) }} / {{ formatSize(download.total) }}</span>
            <span class="segments-count" v-if="download.segments?.length">
              {{ download.segments.length }} segments
            </span>
          </div>
        </div>
      </div>
    </div>
  </BaseCard>
</template>

<style scoped>
.download-card {
  margin-bottom: 2px;
}

.card-layout {
  display: flex;
  gap: 20px;
}

.icon-section {
  flex-shrink: 0;
}

.icon-wrapper {
  width: 52px;
  height: 52px;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.1) 0%, rgba(139, 92, 246, 0.1) 100%);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent-primary);
  box-shadow: inset 0 0 10px rgba(255, 255, 255, 0.02);
}

.content-section {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.header-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
}

.title-area {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 1.05rem;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.metadata-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.divider {
  width: 3px;
  height: 3px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 50%;
}

.url {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.6;
}

.actions-area {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 10px;
}

.button-group {
  display: flex;
  gap: 6px;
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.stats-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.75rem;
  font-weight: 700;
}

.main-stats {
  display: flex;
  gap: 12px;
}

.speed {
  color: var(--accent-primary);
  font-family: var(--font-mono);
}

.percentage {
  color: var(--text-primary);
}

.eta {
  color: var(--text-secondary);
  font-weight: 500;
}

.visualizer {
  height: 14px !important;
  border-radius: 6px;
  overflow: hidden;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.details-row {
  display: flex;
  justify-content: space-between;
  font-size: 0.7rem;
  color: var(--text-secondary);
  font-weight: 500;
  opacity: 0.8;
}

.success-text {
  color: var(--color-downloading) !important;
}

.success-text:hover {
  background: rgba(16, 185, 129, 0.1) !important;
}
</style>

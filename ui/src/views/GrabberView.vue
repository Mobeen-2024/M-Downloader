<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Globe, Download, ListVideo, Film, Music, Search } from 'lucide-vue-next';
import BaseButton from '@/features/shared/components/BaseButton.vue';
import BaseInput from '@/features/shared/components/BaseInput.vue';
import BaseDialog from '@/features/shared/components/BaseDialog.vue';
import BaseSkeleton from '@/features/shared/components/BaseSkeleton.vue';
import BaseToggle from '@/features/shared/components/BaseToggle.vue';
import { animate, stagger, spring } from 'motion';

const url = ref('');
const isScanning = ref(false);
const mediaResults = ref<any[]>([]);

// Phase 2: Managed Modal State
const showCaptureModal = ref(false);
const activeMediaItem = ref<any>(null);
const autoStart = ref(true);
const gridRef = ref<HTMLElement | null>(null);
const radarRef = ref<HTMLElement | null>(null);

// Phase 2: Async View Initialization (Skeleton Loaders)
const isViewLoading = ref(true);

onMounted(() => {
  // Defer view rendering to emulate component hydration
  setTimeout(() => {
    isViewLoading.value = false;
  }, 800);
});

const handleScan = () => {
  if (!url.value) return;
  isScanning.value = true;
  mediaResults.value = [];

  // Phase 6: Radar Animation
  if (radarRef.value) {
    (animate as any)(radarRef.value, 
      { scale: [1, 1.5], opacity: [0.5, 0] },
      { duration: 1.5, repeat: Infinity, easing: "ease-out" }
    );
  }

  // Emulate network payload sniffing
  setTimeout(() => {
    isScanning.value = false;
    mediaResults.value = [
      { id: 1, title: 'Network Stream [1080p]', type: 'video', size: '145.2 MB', ext: 'mp4' },
      { id: 2, title: 'Background Audio Track', type: 'audio', size: '3.1 MB', ext: 'mp3' },
      { id: 3, title: 'Asset Resource [4K]', type: 'video', size: '2.4 GB', ext: 'mkv' },
      { id: 4, title: 'Compressed Archive', type: 'file', size: '840 KB', ext: 'zip' }
    ];

    // Phase 6: Results stagger
    setTimeout(() => {
      if (gridRef.value) {
        (animate as any)(
          ".media-card",
          { opacity: [0, 1], y: [20, 0] },
          { delay: stagger(0.05), easing: spring({ stiffness: 300, damping: 30 }) } as any
        );
      }
    }, 50);
  }, 2000);
};

const openCaptureConfig = (item: any) => {
  activeMediaItem.value = item;
  showCaptureModal.value = true;
};
</script>

<template>
  <div class="grabber-view">
    <header class="view-header">
      <div class="header-left">
        <h2 class="view-title">Site Grabber</h2>
        <p class="view-subtitle">Extract media, documents, and assets directly from websites.</p>
      </div>
      <div class="header-right">
        <div class="header-orb">
          <Globe :size="20" class="text-accent" />
        </div>
      </div>
    </header>

    <!-- Phase 2: Initialization Skeletons -->
    <div v-if="isViewLoading" class="skeleton-container">
      <BaseSkeleton height="64px" borderRadius="12px" class="mb-6" />
      <div class="skeleton-grid">
        <BaseSkeleton v-for="i in 4" :key="i" height="90px" borderRadius="12px" />
      </div>
    </div>

    <!-- Main Interface -->
    <div v-else class="content-body">
      <div class="search-section glass-panel">
        <BaseInput
          v-model="url"
          placeholder="Enter website URL to scan (e.g., https://example.com/video)"
          @keyup.enter="handleScan"
          class="url-input"
        />
        <BaseButton variant="primary" :loading="isScanning" @click="handleScan" class="scan-btn">
          <template #icon-left v-if="!isScanning"><Search :size="16" /></template>
          Scan Content
        </BaseButton>
      </div>

      <div class="results-container">
        <div v-if="isScanning" class="scanning-state">
          <div class="scanning-visual">
            <div ref="radarRef" class="radar-ping"></div>
            <div class="photon-ring"></div>
            <Search :size="32" class="scanning-icon" />
          </div>
          <p class="scanning-text">Analyzing page structure and parsing network requests...</p>
        </div>

        <div v-else-if="mediaResults.length === 0" class="empty-state">
          <div class="empty-visual">
            <div class="visual-glow"></div>
            <ListVideo :size="48" class="visual-icon" />
          </div>
          <div class="empty-text">
            <h3>Engine Standby</h3>
            <p>Enter a target URL above to start the extraction process.</p>
          </div>
        </div>

        <TransitionGroup v-else name="grid-move" tag="div" class="media-grid" ref="gridRef">
          <div v-for="item in mediaResults" :key="item.id" class="media-card glass-panel">
            <div class="media-icon-wrapper">
              <Film v-if="item.type === 'video'" class="text-accent" />
              <Music v-else-if="item.type === 'audio'" class="text-accent" />
              <Download v-else class="text-secondary" />
            </div>
            <div class="media-info">
              <h4>{{ item.title }}</h4>
              <span class="meta">{{ item.ext.toUpperCase() }} • {{ item.size }}</span>
            </div>
            <div class="media-actions">
              <BaseButton variant="glass" size="sm" @click="openCaptureConfig(item)">
                <template #icon-left><Download :size="14" /></template>
                Capture
              </BaseButton>
            </div>
          </div>
        </TransitionGroup>
      </div>
    </div>

    <!-- Phase 2: Managed BaseDialog -->
    <BaseDialog
      :show="showCaptureModal"
      title="Configure Capture"
      size="md"
      @close="showCaptureModal = false"
    >
      <div class="modal-body">
        <p class="modal-desc">Adjust processing parameters for <strong class="text-accent">{{ activeMediaItem?.title }}</strong> before initialization.</p>

        <div class="settings-grid">
          <div class="setting-item">
            <label>Priority level</label>
            <div class="custom-select-wrapper">
              <select class="custom-select">
                <option>High (Accelerated)</option>
                <option selected>Standard (Balanced)</option>
                <option>Low (Background)</option>
              </select>
            </div>
          </div>
          
          <div class="setting-item">
            <label>Auto-Start Transmission</label>
            <BaseToggle v-model="autoStart" label="Begin processing immediately after initialization" />
          </div>
        </div>
      </div>
      <template #footer>
        <BaseButton variant="glass" @click="showCaptureModal = false">Cancel</BaseButton>
        <BaseButton variant="primary" @click="showCaptureModal = false">Initialize Payload</BaseButton>
      </template>
    </BaseDialog>
  </div>
</template>

<style scoped>
.grabber-view {
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

.header-orb {
  width: 44px;
  height: 44px;
  background: rgba(59, 130, 246, 0.1);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.search-section {
  display: flex;
  gap: 16px;
  padding: 20px;
  border-radius: var(--radius-md);
  margin-bottom: 24px;
}

.url-input { flex: 1; }

.results-container {
  flex: 1;
  overflow-y: auto;
  padding-right: 8px;
}

.media-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 16px;
  max-width: 1400px;
}

.media-card {
  display: flex;
  align-items: center;
  padding: 16px 20px;
  border-radius: var(--radius-md);
  gap: 16px;
  transition: var(--transition-smooth);
}

.media-card:hover {
  transform: translateY(-2px);
  background: rgba(255, 255, 255, 0.05);
  box-shadow: var(--shadow-glass);
}

.media-icon-wrapper {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 10px;
  border: 1px solid var(--border-color);
}

.media-info { flex: 1; }
.media-info h4 { font-size: 0.95rem; font-weight: 600; margin-bottom: 2px; }
.meta { font-size: 0.75rem; color: var(--text-secondary); opacity: 0.8; }

/* Initialization States */
.scanning-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 0;
  gap: 20px;
  color: var(--text-secondary);
}

.scanning-visual {
  position: relative;
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.radar-ping {
  position: absolute;
  inset: 0;
  border: 2px solid var(--accent-primary);
  border-radius: 50%;
  pointer-events: none;
}

.photon-ring {
  position: absolute;
  inset: -10px;
  border: 1px dashed var(--accent-primary);
  border-radius: 50%;
  opacity: 0.2;
  animation: spin 8s linear infinite;
}

.scanning-text {
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--accent-primary);
  text-shadow: 0 0 10px rgba(59, 130, 246, 0.3);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 100px 0;
  gap: 24px;
}

.empty-visual {
  position: relative;
  width: 100px;
  height: 100px;
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

.visual-icon { color: var(--text-secondary); opacity: 0.2; }



/* Modal Styles */
.modal-body { display: flex; flex-direction: column; gap: 24px; }
.modal-desc { font-size: 0.9rem; color: var(--text-secondary); line-height: 1.5; }
.settings-grid { display: grid; gap: 20px; }
.setting-item { display: flex; flex-direction: column; gap: 10px; }
.setting-item label { font-size: 0.75rem; font-weight: 700; text-transform: uppercase; color: var(--text-secondary); opacity: 0.7; }

.custom-select-wrapper { position: relative; }
.custom-select {
  width: 100%;
  padding: 12px 16px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  color: var(--text-primary);
  appearance: none;
  font-family: inherit;
  font-size: 0.9rem;
}

.toggle-mock { width: 44px; height: 24px; }
.toggle-track { width: 100%; height: 100%; background: rgba(255,255,255,0.05); border-radius: 12px; position: relative; border: 1px solid var(--border-color); }
.toggle-track.active { background: var(--accent-primary); border-color: transparent; }
.toggle-thumb { width: 18px; height: 18px; background: white; border-radius: 50%; position: absolute; top: 2px; right: 2px; transition: transform 0.2s; }

/* Skeletons */
.skeleton-container { padding: 0 4px; }
.skeleton-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 16px; }
.mb-6 { margin-bottom: 24px; }

@keyframes breathe {
  0%, 100% { transform: scale(1); opacity: 0.5; }
  50% { transform: scale(1.1); opacity: 0.8; }
}
</style>

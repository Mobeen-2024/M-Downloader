<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Folder, File, RefreshCw, Download, HardDrive, ShieldAlert } from 'lucide-vue-next';
import BaseButton from '@/features/shared/components/BaseButton.vue';
import BaseSkeleton from '@/features/shared/components/BaseSkeleton.vue';
import StatusBadge from '@/features/shared/components/StatusBadge.vue';

const isLoading = ref(true);
const driveFiles = ref<any[]>([]);
const isConnected = ref(false); // Mock connection state

onMounted(() => {
  // Initialization delay to show skeletons
  setTimeout(() => {
    fetchDriveFiles();
  }, 1000);
});

const fetchDriveFiles = async () => {
  isLoading.value = true;
  try {
    // Note: In a real app, this would come from a secure auth store/provider
    // const accessToken = 'MOCK_TOKEN';
    
    // Simulating API latency
    await new Promise(resolve => setTimeout(resolve, 1500));

    if (!isConnected.value) {
      driveFiles.value = [];
      return;
    }

    // Mock data for demonstration when connected
    driveFiles.value = [
      { id: '1', name: 'Project Assets', type: 'folder', size: '--', modified: '2024-03-28' },
      { id: '2', name: 'Raw_Footage_001.mp4', type: 'file', size: '1.2 GB', modified: '2024-03-29' },
      { id: '3', name: 'Backup_Registry.zip', type: 'file', size: '45.8 MB', modified: '2024-03-30' },
      { id: '4', name: 'Technical_Documentation.pdf', type: 'file', size: '2.1 MB', modified: '2024-03-31' }
    ];
  } catch (error) {
    console.error('Error fetching drive files:', error);
  } finally {
    isLoading.value = false;
  }
};

const toggleConnection = () => {
  isConnected.value = !isConnected.value;
  fetchDriveFiles();
};

const refreshDrive = () => {
  fetchDriveFiles();
};
</script>

<template>
  <div class="cloud-view">
    <header class="view-header">
      <div class="header-left">
        <h2 class="view-title">Cloud Drive</h2>
        <p class="view-subtitle">Direct REST integration with your Google Drive storage.</p>
      </div>
      <div class="header-right">
        <BaseButton variant="glass" @click="refreshDrive" :disabled="!isConnected || isLoading">
          <template #icon-left><RefreshCw :size="16" :class="{ 'spin': isLoading }" /></template>
          Sync
        </BaseButton>
        <BaseButton :variant="isConnected ? 'glass' : 'primary'" @click="toggleConnection">
          {{ isConnected ? 'Disconnect' : 'Connect Account' }}
        </BaseButton>
      </div>
    </header>

    <!-- Managed Skeleton Loaders -->
    <div v-if="isLoading" class="skeleton-container">
      <div v-for="i in 5" :key="i" class="skeleton-row glass-panel">
        <BaseSkeleton variant="rect" width="24px" height="24px" borderRadius="6px" />
        <div class="skeleton-info">
          <BaseSkeleton width="50%" height="16px" />
          <BaseSkeleton width="30%" height="12px" />
        </div>
        <BaseSkeleton width="80px" height="32px" borderRadius="8px" />
      </div>
    </div>

    <!-- Active Content -->
    <div v-else class="content-body">
      <!-- Unconnected State -->
      <div v-if="!isConnected" class="empty-state auth-state">
        <div class="empty-visual">
          <div class="visual-glow warning"></div>
          <ShieldAlert :size="48" class="visual-icon text-warning" />
        </div>
        <div class="empty-text">
          <h3>Authentication Required</h3>
          <p>Authorize Mdownloader to access your cloud resources securely via OAuth 2.0.</p>
          <BaseButton variant="primary" class="mt-4" @click="toggleConnection">
            Initialize OAuth Flow
          </BaseButton>
        </div>
      </div>

      <!-- Connected but Empty -->
      <div v-else-if="driveFiles.length === 0" class="empty-state">
        <div class="empty-visual">
          <div class="visual-glow"></div>
          <HardDrive :size="48" class="visual-icon" />
        </div>
        <div class="empty-text">
          <h3>No Assets Detected</h3>
          <p>Your Google Drive appears to be empty or restricted.</p>
        </div>
      </div>

      <!-- File List -->
      <TransitionGroup v-else name="list-move" tag="div" class="file-list">
        <div v-for="file in driveFiles" :key="file.id" class="file-item glass-panel">
          <div class="file-icon-wrapper" :class="{ 'is-folder': file.type === 'folder' }">
            <Folder v-if="file.type === 'folder'" :size="20" />
            <File v-else :size="20" />
          </div>

          <div class="file-details">
            <span class="file-name">{{ file.name }}</span>
            <span class="file-meta">{{ file.size }} • Modified: {{ file.modified }}</span>
          </div>

          <div class="file-actions">
            <StatusBadge v-if="file.type === 'folder'" status="Queued" text="Navigate" />
            <BaseButton v-else variant="glass" size="sm">
              <template #icon-left><Download :size="14" /></template>
              Capture
            </BaseButton>
          </div>
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

<style scoped>
.cloud-view {
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

.header-right { display: flex; gap: 12px; }

.content-body { flex: 1; overflow-y: auto; padding-right: 8px; }

.file-list { display: flex; flex-direction: column; gap: 12px; max-width: 1200px; }

.file-item {
  display: flex;
  align-items: center;
  padding: 16px 24px;
  border-radius: var(--radius-md);
  gap: 20px;
  transition: var(--transition-smooth);
}

.file-item:hover {
  background: rgba(255, 255, 255, 0.05);
  transform: translateX(4px);
}

.file-icon-wrapper {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 8px;
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
}

.file-icon-wrapper.is-folder { color: var(--accent-primary); background: rgba(59, 130, 246, 0.05); }

.file-details { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.file-name { font-weight: 600; font-size: 0.95rem; }
.file-meta { font-size: 0.75rem; color: var(--text-secondary); opacity: 0.7; }

/* States */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 100px 0;
  gap: 24px;
  text-align: center;
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

.visual-glow.warning { background: radial-gradient(circle, rgba(245, 158, 11, 0.1) 0%, transparent 70%); }
.visual-icon { color: var(--text-secondary); opacity: 0.2; }
.text-warning { color: var(--color-paused); opacity: 0.8 !important; }

.mt-4 { margin-top: 16px; }

/* Skeleton */
.skeleton-container { display: flex; flex-direction: column; gap: 12px; }
.skeleton-row {
  display: flex;
  align-items: center;
  padding: 16px 24px;
  height: 72px;
  border-radius: var(--radius-md);
  gap: 20px;
}
.skeleton-info { flex: 1; display: flex; flex-direction: column; gap: 8px; }

/* Transitions */
.list-move-move { transition: transform 0.5s cubic-bezier(0.4, 0, 0.2, 1); }
.list-move-enter-active, .list-move-leave-active { transition: all 0.4s ease; }
.list-move-enter-from, .list-move-leave-to { opacity: 0; transform: translateX(-20px); }

.spin { animation: spin 1s linear infinite; }
@keyframes spin { 100% { transform: rotate(360deg); } }
@keyframes breathe {
  0%, 100% { transform: scale(1); opacity: 0.5; }
  50% { transform: scale(1.1); opacity: 0.8; }
}
</style>

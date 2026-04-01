<script setup lang="ts">
import { ref, onUnmounted, watch } from 'vue';
import { X, ExternalLink, Link, AlertTriangle, Loader2 } from 'lucide-vue-next';
import { listen } from '@tauri-apps/api/event';
import { useDownload } from '@/composables/useDownload';
import GlassPanel from '@/features/shared/components/GlassPanel.vue';

const props = defineProps<{
  show: boolean;
  id: string;
  downloadName: string;
  sourceUrl?: string;
}>();

const emit = defineEmits(['close', 'refresh']);

const { startRefreshMode, cancelRefreshMode, resumeDownload } = useDownload();

const newUrl = ref('');
const error = ref('');
const isWaiting = ref(true);
const captureSuccess = ref(false);
let unlisten: (() => void) | null = null;

const setupListener = async () => {
  if (unlisten) unlisten();
  
  // Subscribe to transparent refresh events from the bridge
  unlisten = await listen('url-refreshed', (event: any) => {
    if (event.payload.id === props.id) {
      isWaiting.value = false;
      captureSuccess.value = true;
      setTimeout(() => {
        emit('close');
        resumeDownload(props.id);
      }, 1500);
    }
  });
};

watch(() => props.show, async (newVal) => {
  if (newVal) {
    isWaiting.value = true;
    captureSuccess.value = false;
    await startRefreshMode(props.id);
    await setupListener();
  } else {
    await cancelRefreshMode();
    if (unlisten) unlisten();
  }
});

onUnmounted(async () => {
  await cancelRefreshMode();
  if (unlisten) unlisten();
});

const handleSubmit = () => {
  if (!newUrl.value) {
    error.value = 'Please enter a new URL';
    return;
  }
  if (!newUrl.value.startsWith('http')) {
    error.value = 'Invalid URL format';
    return;
  }
  emit('refresh', newUrl.value);
  newUrl.value = '';
};

const openUrl = (url: string) => {
  window.open(url, '_blank');
};
</script>

<template>
  <div v-if="show" class="modal-overlay" @click.self="emit('close')">
    <GlassPanel class="modal-content">
      <div class="modal-header">
        <div class="header-title">
          <Link class="header-icon" />
          <h3>Refresh Download Address</h3>
        </div>
        <button class="close-btn" @click="emit('close')">
          <X />
        </button>
      </div>

      <div class="modal-body">
        <div v-if="!captureSuccess" class="alert-box warning">
          <AlertTriangle class="alert-icon" />
          <div class="alert-text">
            <strong>Link Expired:</strong> The current URL for <code>{{ downloadName }}</code> is no longer valid (403 Forbidden). 
          </div>
        </div>

        <div v-if="captureSuccess" class="alert-box success">
          <div class="alert-text">
            <strong>Success!</strong> New address intercepted. Resuming download...
          </div>
        </div>

        <!-- Automated Capture State -->
        <div v-if="isWaiting" class="capture-state pulse">
          <Loader2 class="spinner" />
          <div class="capture-info">
            <h4>Waiting for link interception...</h4>
            <p>Re-click the download button in your browser to refresh automatically.</p>
          </div>
        </div>

        <div v-if="sourceUrl && !captureSuccess && isWaiting" class="info-group">
          <label>Source Page</label>
          <div class="source-link">
            <span>{{ sourceUrl }}</span>
            <button class="btn-sm" @click="openUrl(sourceUrl)">
              <ExternalLink class="icon-sm" />
              Open Page
            </button>
          </div>
        </div>

        <div v-if="!captureSuccess && !isWaiting" class="input-group">
          <label for="new-url">Manual URL Entry</label>
          <div class="input-wrapper">
            <input 
              id="new-url"
              v-model="newUrl"
              type="text" 
              placeholder="Paste the new link here..."
              @keyup.enter="handleSubmit"
            />
          </div>
          <p v-if="error" class="error-text">{{ error }}</p>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-ghost" @click="emit('close')">Cancel</button>
        <button class="btn-primary" @click="handleSubmit">
          Update and Resume
        </button>
      </div>
    </GlassPanel>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  width: 100%;
  max-width: 500px;
  padding: 24px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-icon {
  color: var(--accent-primary);
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
}

.alert-box.success {
  background: rgba(34, 197, 94, 0.1);
  border-color: rgba(34, 197, 94, 0.2);
  color: #22c55e;
}

.capture-state {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 24px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px dashed var(--border-color);
  border-radius: 12px;
  margin-bottom: 24px;
}

.pulse {
  animation: pulse-glow 2s infinite;
}

@keyframes pulse-glow {
  0% { box-shadow: 0 0 0 0 rgba(59, 130, 246, 0.4); }
  70% { box-shadow: 0 0 0 15px rgba(59, 130, 246, 0); }
  100% { box-shadow: 0 0 0 0 rgba(59, 130, 246, 0); }
}

.spinner {
  width: 32px;
  height: 32px;
  color: var(--accent-primary);
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.capture-info h4 {
  font-size: 0.95rem;
  margin-bottom: 4px;
  color: var(--text-primary);
}

.capture-info p {
  font-size: 0.8rem;
  color: var(--text-secondary);
}

.info-group, .input-group {
  margin-bottom: 20px;
}

label {
  display: block;
  font-size: 0.75rem;
  font-weight: 700;
  margin-bottom: 8px;
  text-transform: uppercase;
  color: var(--text-secondary);
}

.source-link {
  display: flex;
  gap: 8px;
  align-items: center;
  background: rgba(255, 255, 255, 0.03);
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 0.8rem;
}

.source-link span {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--accent-primary);
}

.btn-sm {
  display: flex;
  align-items: center;
  gap: 4px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-color);
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 0.7rem;
  color: var(--text-primary);
  cursor: pointer;
}

.input-wrapper input {
  width: 100%;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-color);
  padding: 12px;
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 0.9rem;
}

.help-text {
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin-top: 8px;
}

.error-text {
  color: var(--color-error);
  font-size: 0.8rem;
  margin-top: 4px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}

.btn-ghost {
  background: none;
  border: none;
  color: var(--text-secondary);
  padding: 10px 20px;
  cursor: pointer;
}

.btn-primary {
  background: var(--accent-primary);
  color: white;
  border: none;
  padding: 10px 24px;
  border-radius: 8px;
  font-weight: 700;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}
</style>

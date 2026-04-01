<script setup lang="ts">
import { ref, onUnmounted, watch } from 'vue';
import { ExternalLink, Link, AlertTriangle, CheckCircle2, Globe } from 'lucide-vue-next';
import { listen } from '@tauri-apps/api/event';
import { useDownload } from '@/composables/useDownload';
import BaseDialog from '@/features/shared/components/BaseDialog.vue';
import BaseButton from '@/features/shared/components/BaseButton.vue';
import BaseInput from '@/features/shared/components/BaseInput.vue';

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
    error.value = 'Please enter a valid source address';
    return;
  }
  if (!newUrl.value.startsWith('http')) {
    error.value = 'Protocol identifier missing (e.g. https://)';
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
  <BaseDialog 
    :show="show" 
    title="Synchronize Transmission" 
    size="md" 
    @close="emit('close')"
  >
    <div class="modal-body">
      <!-- Status Banner -->
      <div v-if="!captureSuccess" class="pro-alert warn">
        <div class="alert-icon-box">
          <AlertTriangle :size="16" />
        </div>
        <div class="alert-content">
          <p class="alert-title">Link Identity Expired</p>
          <p class="alert-desc">The secure token for <code>{{ downloadName }}</code> has been invalidated by the host.</p>
        </div>
      </div>

      <div v-if="captureSuccess" class="pro-alert success">
        <div class="alert-icon-box">
          <CheckCircle2 :size="16" />
        </div>
        <div class="alert-content">
          <p class="alert-title">Identity Captured</p>
          <p class="alert-desc">New transmission parameters intercepted. Resuming stream...</p>
        </div>
      </div>

      <!-- Autonomous Interception Area -->
      <div v-if="isWaiting" class="interception-zone">
        <div class="radar-visual">
          <div class="radar-dot"></div>
          <div class="radar-pulse"></div>
          <Link :size="24" class="radar-icon" />
        </div>
        <div class="interception-text">
          <h4>Awaiting Extension Handshake</h4>
          <p>Re-trigger the download link in your browser. Our bridge will capture the fresh address automatically.</p>
        </div>
      </div>

      <!-- Manual Controls Header -->
      <div class="manual-header" v-if="!captureSuccess">
        <div class="h-divider"></div>
        <span class="h-label">Traditional Fallback</span>
        <div class="h-divider"></div>
      </div>

      <!-- Source Context -->
      <div v-if="sourceUrl && !captureSuccess" class="context-group">
        <label class="field-label">Original Discovery Context</label>
        <div class="source-card">
          <Globe :size="14" class="text-accent" />
          <span class="source-text" :title="sourceUrl">{{ sourceUrl }}</span>
          <BaseButton variant="glass" size="sm" @click="openUrl(sourceUrl)">
            <ExternalLink :size="12" />
          </BaseButton>
        </div>
      </div>

      <!-- Manual Input -->
      <div v-if="!captureSuccess && !isWaiting" class="input-section">
        <label class="field-label">Manual URL Submission</label>
        <BaseInput 
          v-model="newUrl" 
          placeholder="Paste high-speed link destination..." 
          :error="error"
          @keyup.enter="handleSubmit"
        />
      </div>
    </div>

    <template #footer>
      <BaseButton variant="glass" @click="emit('close')">Cancel</BaseButton>
      <BaseButton 
        variant="primary" 
        :disabled="captureSuccess || isWaiting"
        @click="handleSubmit"
      >
        Synchronize Stream
      </BaseButton>
    </template>
  </BaseDialog>
</template>

<style scoped>
.modal-body {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* Alert Banner */
.pro-alert {
  display: flex;
  gap: 16px;
  padding: 16px;
  border-radius: 12px;
  border: 1px solid transparent;
}

.pro-alert.warn {
  background: rgba(245, 158, 11, 0.05);
  border-color: rgba(245, 158, 11, 0.2);
  color: #f59e0b;
}

.pro-alert.success {
  background: rgba(16, 185, 129, 0.05);
  border-color: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.alert-icon-box {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.05);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.alert-title {
  font-size: 0.9rem;
  font-weight: 800;
  margin: 0 0 4px 0;
}

.alert-desc {
  font-size: 0.75rem;
  margin: 0;
  opacity: 0.8;
  line-height: 1.4;
}

code {
  font-family: var(--font-mono);
  background: rgba(0, 0, 0, 0.2);
  padding: 2px 4px;
  border-radius: 4px;
}

/* Interception Zone */
.interception-zone {
  background: rgba(255, 255, 255, 0.02);
  border: 1px dashed var(--border-color);
  border-radius: 16px;
  padding: 32px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  text-align: center;
}

.radar-visual {
  width: 64px;
  height: 64px;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.radar-dot {
  width: 8px;
  height: 8px;
  background: var(--accent-primary);
  border-radius: 50%;
  z-index: 2;
}

.radar-pulse {
  position: absolute;
  inset: 0;
  border: 2px solid var(--accent-primary);
  border-radius: 50%;
  animation: radar-pulse 2s infinite ease-out;
}

.radar-icon {
  position: absolute;
  color: var(--accent-primary);
  opacity: 0.2;
}

@keyframes radar-pulse {
  0% { transform: scale(0.5); opacity: 0.8; }
  100% { transform: scale(1.5); opacity: 0; }
}

.interception-text h4 {
  font-size: 1rem;
  font-weight: 800;
  margin: 0 0 8px 0;
  color: var(--text-primary);
}

.interception-text p {
  font-size: 0.8rem;
  color: var(--text-secondary);
  line-height: 1.5;
  margin: 0;
}

/* Manual Header */
.manual-header {
  display: flex;
  align-items: center;
  gap: 16px;
  opacity: 0.4;
}

.h-divider {
  flex: 1;
  height: 1px;
  background: var(--border-color);
}

.h-label {
  font-size: 0.6rem;
  text-transform: uppercase;
  font-weight: 800;
  letter-spacing: 0.1em;
}

/* Context Group */
.field-label {
  display: block;
  font-size: 0.7rem;
  text-transform: uppercase;
  font-weight: 800;
  color: var(--text-secondary);
  margin-bottom: 8px;
  letter-spacing: 0.05em;
}

.source-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 8px 12px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.source-text {
  flex: 1;
  font-size: 0.8rem;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: var(--font-mono);
}

.error-msg {
  color: var(--color-error);
  font-size: 0.75rem;
  font-weight: 600;
  margin-top: 6px;
}
</style>

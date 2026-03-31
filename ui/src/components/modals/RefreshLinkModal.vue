<script setup lang="ts">
import { ref } from 'vue';
import { X, ExternalLink, Link, AlertTriangle } from 'lucide-vue-next';
import GlassPanel from '../ui/GlassPanel.vue';

const props = defineProps<{
  show: boolean;
  downloadName: string;
  sourceUrl?: string;
}>();

const emit = defineEmits(['close', 'refresh']);

const newUrl = ref('');
const error = ref('');

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
        <div class="alert-box warning">
          <AlertTriangle class="alert-icon" />
          <div class="alert-text">
            <strong>Link Expired:</strong> The current URL for <code>{{ downloadName }}</code> is no longer valid (403 Forbidden). 
            Please provide a new one to resume.
          </div>
        </div>

        <div v-if="sourceUrl" class="info-group">
          <label>Source Page</label>
          <div class="source-link">
            <span>{{ sourceUrl }}</span>
            <button class="btn-sm" @click="openUrl(sourceUrl)">
              <ExternalLink class="icon-sm" />
              Open Page
            </button>
          </div>
          <p class="help-text">Click "Open Page" and re-click the download button in your browser to get a fresh link.</p>
        </div>

        <div class="input-group">
          <label for="new-url">New Download URL</label>
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

.alert-box {
  display: flex;
  gap: 12px;
  padding: 12px;
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.2);
  border-radius: 8px;
  margin-bottom: 20px;
  font-size: 0.85rem;
}

.alert-icon {
  color: #f59e0b;
  flex-shrink: 0;
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

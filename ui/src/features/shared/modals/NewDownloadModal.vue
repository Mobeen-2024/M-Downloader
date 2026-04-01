<script setup lang="ts">
import { ref, computed } from 'vue';
import { X, Globe, Download, AlertCircle } from 'lucide-vue-next';
import GlassPanel from '@/features/shared/components/GlassPanel.vue';
import { useDownload } from '@/composables/useDownload';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits(['close']);

const url = ref('https://releases.ubuntu.com/22.04/ubuntu-22.04.3-desktop-amd64.iso');
const { startDownload } = useDownload();
const isSubmitting = ref(false);
const error = ref('');

const isValidUrl = computed(() => {
  try {
    new URL(url.value);
    return true;
  } catch {
    return false;
  }
});

const fileName = computed(() => {
  if (!isValidUrl.value) return '';
  return url.value.split('/').pop() || 'unknown-file';
});

const handleStart = async () => {
  if (!isValidUrl.value || isSubmitting.value) return;
  
  isSubmitting.value = true;
  error.value = '';
  
  try {
    await startDownload(url.value);
    emit('close');
  } catch (e: any) {
    error.value = e.message || 'Failed to start download';
  } finally {
    isSubmitting.value = false;
  }
};
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="show" class="modal-overlay" @click.self="$emit('close')">
        <Transition name="slide-up">
          <GlassPanel class="modal-content">
            <header class="modal-header">
              <div class="title-group">
                <Globe class="title-icon" />
                <h3>Add New Download</h3>
              </div>
              <button class="btn-close" @click="$emit('close')">
                <X />
              </button>
            </header>

            <div class="modal-body">
              <div class="input-group">
                <label for="url-input">Source URL</label>
                <div class="input-wrapper" :class="{ 'error': error, 'valid': isValidUrl }">
                  <input 
                    id="url-input"
                    v-model="url" 
                    type="text" 
                    placeholder="https://example.com/file.zip"
                    autofocus
                    @keyup.enter="handleStart"
                  />
                </div>
                <div v-if="error" class="error-text">
                  <AlertCircle class="error-icon" />
                  <span>{{ error }}</span>
                </div>
              </div>

              <div v-if="isValidUrl" class="preview-box">
                <div class="preview-label">Destination Filename</div>
                <div class="preview-name">{{ fileName }}</div>
              </div>

              <div class="advanced-info">
                <p>Mdownloader will automatically split this file into multiple segments using the in-half division rule for maximum acceleration.</p>
              </div>
            </div>

            <footer class="modal-footer">
              <button class="btn-cancel" @click="$emit('close')">Cancel</button>
              <button 
                class="btn-start" 
                :disabled="!isValidUrl || isSubmitting"
                @click="handleStart"
              >
                <Download v-if="!isSubmitting" class="btn-icon" />
                <span v-else class="loader"></span>
                <span>{{ isSubmitting ? 'Starting...' : 'Download Now' }}</span>
              </button>
            </footer>
          </GlassPanel>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  width: 540px;
  max-width: 90vw;
  padding: 0;
}

.modal-header {
  padding: 24px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.title-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.title-icon {
  color: var(--accent-primary);
  width: 24px;
  height: 24px;
}

.btn-close {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition: var(--transition-smooth);
}

.btn-close:hover {
  color: var(--text-primary);
}

.modal-body {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

label {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.input-wrapper {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 4px 12px;
  transition: var(--transition-smooth);
}

.input-wrapper:focus-within {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 4px rgba(59, 130, 246, 0.1);
}

.input-wrapper.valid {
  border-color: rgba(16, 185, 129, 0.3);
}

.input-wrapper.error {
  border-color: var(--color-error);
}

input {
  width: 100%;
  background: none;
  border: none;
  padding: 12px 0;
  color: var(--text-primary);
  outline: none;
  font-size: 1rem;
}

.error-text {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--color-error);
  font-size: 0.75rem;
  margin-top: 4px;
}

.preview-box {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 12px;
  padding: 16px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.preview-label {
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.preview-name {
  font-weight: 700;
  color: var(--accent-primary);
}

.advanced-info {
  font-size: 0.75rem;
  color: var(--text-secondary);
  line-height: 1.5;
  font-style: italic;
}

.modal-footer {
  padding: 24px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn-cancel {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  padding: 10px 20px;
  border-radius: 10px;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition-smooth);
}

.btn-start {
  background: var(--accent-primary);
  color: white;
  border: none;
  padding: 10px 24px;
  border-radius: 12px;
  font-weight: 700;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: var(--transition-smooth);
}

.btn-start:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Transitions */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.slide-up-enter-active, .slide-up-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.slide-up-enter-from { transform: translateY(30px); opacity: 0; }
.slide-up-leave-to { transform: translateY(30px); opacity: 0; }

.loader {
  width: 16px;
  height: 16px;
  border: 2px solid white;
  border-bottom-color: transparent;
  border-radius: 50%;
  display: inline-block;
  animation: rotation 1s linear infinite;
}

@keyframes rotation {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>

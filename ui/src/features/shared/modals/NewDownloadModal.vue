<script setup lang="ts">
import { ref, computed } from 'vue';
import { Download, Info } from 'lucide-vue-next';
import { useDownload } from '@/composables/useDownload';
import BaseDialog from '@/features/shared/components/BaseDialog.vue';
import BaseButton from '@/features/shared/components/BaseButton.vue';
import BaseInput from '@/features/shared/components/BaseInput.vue';

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
  return url.value.split('/').pop() || 'unknown-binary-blob';
});

const handleStart = async () => {
  if (!isValidUrl.value || isSubmitting.value) return;
  
  isSubmitting.value = true;
  error.value = '';
  
  try {
    await startDownload(url.value);
    emit('close');
  } catch (e: any) {
    error.value = e.message || 'Transmission initialization failed';
  } finally {
    isSubmitting.value = false;
  }
};
</script>

<template>
  <BaseDialog 
    :show="show" 
    title="Create Transmission" 
    size="md" 
    @close="$emit('close')"
  >
    <div class="modal-body">
      <div class="input-section">
        <label class="field-label">Resource URL</label>
        <BaseInput 
          v-model="url" 
          placeholder="Enter high-speed source URL..." 
          :error="error"
          autofocus
          @keyup.enter="handleStart"
        />
      </div>

      <Transition name="fade">
        <div v-if="isValidUrl" class="metadata-preview">
          <div class="meta-item">
            <span class="m-label">Calculated Filename</span>
            <span class="m-val">{{ fileName }}</span>
          </div>
          <div class="meta-item">
            <span class="m-label">Engine Strategy</span>
            <span class="m-val">Multi-segment adaptive split</span>
          </div>
        </div>
      </Transition>

      <div class="pro-notice">
        <Info :size="14" class="text-accent" />
        <p>This resource will be processed using the <strong>In-Half Division</strong> rule, dynamically splitting the payload into optimized byte-ranges for maximum throughput.</p>
      </div>
    </div>

    <template #footer>
      <BaseButton variant="glass" @click="$emit('close')">Cancel</BaseButton>
      <BaseButton 
        variant="primary" 
        :disabled="!isValidUrl || isSubmitting"
        :loading="isSubmitting"
        @click="handleStart"
      >
        <template #icon-left v-if="!isSubmitting"><Download :size="16" /></template>
        Initialize Download
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

.field-label {
  display: block;
  font-size: 0.7rem;
  text-transform: uppercase;
  font-weight: 800;
  color: var(--text-secondary);
  margin-bottom: 8px;
  letter-spacing: 0.05em;
}

.error-msg {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--color-error);
  font-size: 0.75rem;
  margin-top: 6px;
  font-weight: 600;
}

.metadata-preview {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.meta-item {
  display: flex;
  flex-direction: column;
}

.m-label {
  font-size: 0.6rem;
  text-transform: uppercase;
  font-weight: 800;
  color: var(--text-secondary);
  opacity: 0.5;
}

.m-val {
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--accent-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pro-notice {
  display: flex;
  gap: 12px;
  background: rgba(59, 130, 246, 0.05);
  padding: 12px 16px;
  border-radius: 10px;
  font-size: 0.75rem;
  line-height: 1.5;
  color: var(--text-secondary);
}

.pro-notice p { margin: 0; }

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>

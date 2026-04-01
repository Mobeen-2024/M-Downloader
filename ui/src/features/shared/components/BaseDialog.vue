<script setup lang="ts">
import {
  DialogClose,
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from 'radix-vue';
import { X } from 'lucide-vue-next';
import BaseButton from './BaseButton.vue';

interface Props {
  show: boolean;
  title?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  closable?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  closable: true
});

const emit = defineEmits(['close']);

const updateShow = (val: boolean) => {
  if (!val) emit('close');
};
</script>

<template>
  <DialogRoot :open="show" @update:open="updateShow">
    <DialogPortal>
      <Transition name="fade">
        <DialogOverlay class="dialog-overlay" />
      </Transition>
      
      <Transition name="scale">
        <DialogContent class="dialog-content" :class="[`size-${size}`]">
          <header class="dialog-header">
            <DialogTitle class="dialog-title">{{ title }}</DialogTitle>
            <DialogClose as-child v-if="closable">
              <BaseButton 
                variant="glass" 
                size="icon" 
                class="close-btn"
                aria-label="Close"
              >
                <X :size="18" />
              </BaseButton>
            </DialogClose>
          </header>

          <div class="dialog-body">
            <slot></slot>
          </div>

          <footer class="dialog-footer" v-if="$slots.footer">
            <slot name="footer"></slot>
          </footer>
        </DialogContent>
      </Transition>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  z-index: 1000;
}

.dialog-content {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 20px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  max-height: 90vh;
  z-index: 1001;
  outline: none;
}

.size-sm { width: 400px; }
.size-md { width: 560px; }
.size-lg { width: 800px; }
.size-xl { width: 1100px; }

.dialog-header {
  padding: 24px 32px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border-color);
}

.dialog-title {
  font-size: 1.25rem;
  font-weight: 800;
  letter-spacing: -0.01em;
  color: var(--text-primary);
}

.dialog-body {
  padding: 32px;
  overflow-y: auto;
  flex: 1;
}

.dialog-footer {
  padding: 20px 32px;
  background: rgba(255, 255, 255, 0.02);
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

/* Transitions */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.scale-enter-active { animation: pop-in 0.4s cubic-bezier(0.34, 1.56, 0.64, 1); }
.scale-leave-active { animation: pop-in 0.25s reverse ease-in; }

@keyframes pop-in {
  from { opacity: 0; transform: translate(-50%, -40%) scale(0.9); }
  to { opacity: 1; transform: translate(-50%, -50%) scale(1); }
}

.close-btn {
  opacity: 0.6;
  transition: opacity 0.2s;
}

.close-btn:hover {
  opacity: 1;
}
</style>

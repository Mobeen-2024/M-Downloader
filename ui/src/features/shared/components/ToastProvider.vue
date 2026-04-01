<script setup lang="ts">
import { useUIStore } from '@/stores/ui.store';
import { CheckCircle, AlertCircle, Info, AlertTriangle, X } from 'lucide-vue-next';

const ui = useUIStore();

const getIcon = (type: string) => {
  switch (type) {
    case 'success': return CheckCircle;
    case 'error': return AlertCircle;
    case 'warning': return AlertTriangle;
    default: return Info;
  }
};
</script>

<template>
  <div class="toast-container">
    <TransitionGroup name="toast">
      <div 
        v-for="toast in ui.toasts" 
        :key="toast.id" 
        class="toast-item"
        :class="`type-${toast.type}`"
      >
        <component :is="getIcon(toast.type)" :size="20" class="toast-icon" />
        <span class="toast-message">{{ toast.message }}</span>
        <button @click="ui.removeToast(toast.id)" class="close-btn">
          <X :size="16" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 32px;
  right: 32px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  z-index: 9999;
  pointer-events: none;
}

.toast-item {
  pointer-events: auto;
  min-width: 300px;
  max-width: 450px;
  padding: 16px;
  background: var(--bg-secondary);
  backdrop-filter: var(--glass-blur);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
  transform-origin: right bottom;
}

.toast-icon {
  flex-shrink: 0;
}

.toast-message {
  flex: 1;
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--text-primary);
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: var(--transition-smooth);
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
}

/* Types */
.type-success { border-left: 4px solid var(--color-downloading); }
.type-success .toast-icon { color: var(--color-downloading); }

.type-error { border-left: 4px solid var(--color-error); }
.type-error .toast-icon { color: var(--color-error); }

.type-warning { border-left: 4px solid var(--color-paused); }
.type-warning .toast-icon { color: var(--color-paused); }

.type-info { border-left: 4px solid var(--accent-primary); }
.type-info .toast-icon { color: var(--accent-primary); }

/* Animations */
.toast-enter-active {
  animation: toast-in 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}
.toast-leave-active {
  animation: toast-in 0.4s cubic-bezier(0.4, 0, 0.2, 1) reverse;
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateX(100%) scale(0.9);
  }
  to {
    opacity: 1;
    transform: translateX(0) scale(1);
  }
}
</style>

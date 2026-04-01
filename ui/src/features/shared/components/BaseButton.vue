<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { animate, spring } from 'motion';

interface Props {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost' | 'glass';
  size?: 'md' | 'sm' | 'lg' | 'icon';
  loading?: boolean;
  disabled?: boolean;
  type?: 'button' | 'submit' | 'reset';
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  loading: false,
  disabled: false,
  type: 'button',
});

const buttonRef = ref<HTMLButtonElement | null>(null);

const classes = computed(() => {
  return [
    'base-button',
    `variant-${props.variant}`,
    `size-${props.size}`,
    { 'is-loading': props.loading, 'is-disabled': props.disabled },
  ];
});

// High-fidelity spring animations
const handlePointerEnter = () => {
  if (props.disabled || props.loading || !buttonRef.value) return;
  (animate as any)(buttonRef.value, 
    { scale: 1.02, y: -2 }, 
    { easing: spring({ stiffness: 400, damping: 25 }) } as any
  );
};

const handlePointerLeave = () => {
  if (props.disabled || props.loading || !buttonRef.value) return;
  (animate as any)(buttonRef.value, 
    { scale: 1, y: 0 }, 
    { easing: spring({ stiffness: 400, damping: 25 }) } as any
  );
};

const handlePointerDown = () => {
  if (props.disabled || props.loading || !buttonRef.value) return;
  (animate as any)(buttonRef.value, 
    { scale: 0.95 }, 
    { easing: spring({ stiffness: 600, damping: 30 }) } as any
  );
};

const handlePointerUp = () => {
  if (props.disabled || props.loading || !buttonRef.value) return;
  (animate as any)(buttonRef.value, 
    { scale: 1.02 }, 
    { easing: spring({ stiffness: 400, damping: 25 }) } as any
  );
};
</script>

<template>
  <button
    ref="buttonRef"
    :type="type"
    :class="classes"
    :disabled="disabled || loading"
    class="interactive-btn"
    @pointerenter="handlePointerEnter"
    @pointerleave="handlePointerLeave"
    @pointerdown="handlePointerDown"
    @pointerup="handlePointerUp"
  >
    <div v-if="loading" class="spinner"></div>
    <div class="content" :class="{ 'opacity-0': loading }">
      <slot name="icon-left"></slot>
      <slot><span v-if="size !== 'icon'">Button</span></slot>
      <slot name="icon-right"></slot>
    </div>
    <div class="shine" v-if="variant === 'primary'"></div>
  </button>
</template>

<style scoped>
.base-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8px 16px;
  border-radius: var(--radius-md);
  font-weight: 600;
  cursor: pointer;
  position: relative;
  overflow: hidden;
  border: 1px solid transparent;
  gap: 8px;
  font-size: 0.875rem;
  background: none;
  min-height: 40px;
  user-select: none;
  /* Disable standard CSS transition for scale/transform to let Motion One handle it */
  transition: background 0.2s, border-color 0.2s, color 0.2s, box-shadow 0.2s;
  touch-action: manipulation;
}

/* Variants */
.variant-primary {
  background: linear-gradient(135deg, var(--accent-primary) 0%, var(--accent-secondary) 100%);
  color: white;
  box-shadow: var(--shadow-glow);
}

.variant-primary .shine {
  position: absolute;
  top: 0;
  left: -100%;
  width: 50%;
  height: 100%;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.2),
    transparent
  );
  transform: skewX(-25deg);
  pointer-events: none;
}

.variant-primary:hover .shine {
  animation: shine 0.75s forwards;
}

@keyframes shine {
  from { left: -100%; }
  to { left: 150%; }
}

.variant-secondary {
  background: var(--glass-bg);
  border-color: var(--border-color);
  color: var(--text-primary);
  backdrop-filter: var(--glass-blur);
}

.variant-glass {
  background: rgba(255, 255, 255, 0.03);
  border-color: rgba(255, 255, 255, 0.05);
  color: var(--text-primary);
  backdrop-filter: blur(12px);
}

.variant-danger {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.2);
  color: var(--color-error);
}
.variant-danger:hover {
  background: var(--color-error);
  color: white;
}

.variant-ghost {
  background: transparent;
  color: var(--text-secondary);
}

/* Sizes */
.size-sm { padding: 4px 12px; min-height: 32px; font-size: 0.8rem; }
.size-lg { padding: 12px 24px; min-height: 48px; font-size: 1rem; }
.size-icon { padding: 8px; width: 40px; height: 40px; min-height: 40px; }

/* States */
.is-disabled {
  opacity: 0.5;
  cursor: not-allowed;
  filter: grayscale(1);
}

.content {
  display: flex;
  align-items: center;
  gap: 8px;
  z-index: 1;
}

.opacity-0 { opacity: 0; }

.spinner {
  position: absolute;
  width: 20px;
  height: 20px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  border-top-color: white;
  animation: spin 0.8s linear infinite;
  z-index: 2;
}

@keyframes spin { to { transform: rotate(360deg); } }
</style>

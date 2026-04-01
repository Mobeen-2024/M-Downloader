<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  variant?: 'glass' | 'outline' | 'flat';
  padding?: 'none' | 'sm' | 'md' | 'lg';
  hoverable?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'glass',
  padding: 'md',
  hoverable: false,
});

const classes = computed(() => {
  return [
    'base-card',
    `variant-${props.variant}`,
    `padding-${props.padding}`,
    { 'is-hoverable': props.hoverable },
  ];
});
</script>

<template>
  <div :class="classes">
    <div v-if="$slots.header" class="card-header">
      <slot name="header"></slot>
    </div>
    <div class="card-content">
      <slot></slot>
    </div>
    <div v-if="$slots.footer" class="card-footer">
      <slot name="footer"></slot>
    </div>
  </div>
</template>

<style scoped>
.base-card {
  border-radius: var(--radius-md);
  overflow: hidden;
  position: relative;
  transition: var(--transition-smooth);
  width: 100%;
}

/* Variants */
.variant-glass {
  background: var(--glass-bg);
  backdrop-filter: var(--glass-blur);
  border: 1px solid var(--border-color);
  box-shadow: var(--shadow-glass);
}
.variant-glass:hover:not(.is-hoverable) {
  border-color: rgba(255, 255, 255, 0.12);
}

.variant-outline {
  background: transparent;
  border: 1px solid var(--border-color);
}

.variant-flat {
  background: rgba(255, 255, 255, 0.02);
  border: none;
}

/* Padding */
.padding-none { padding: 0; }
.padding-sm { padding: 12px; }
.padding-md { padding: 20px; }
.padding-lg { padding: 32px; }

/* Hoverable */
.is-hoverable:hover {
  transform: translateY(-4px) scale(1.01);
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(59, 130, 246, 0.3);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4), 0 0 20px rgba(59, 130, 246, 0.1);
}

.variant-glass:hover:not(.is-hoverable) {
  border-color: rgba(255, 255, 255, 0.15);
  background: rgba(255, 255, 255, 0.04);
}

.card-header {
  border-bottom: 1px solid var(--border-color);
  padding: 16px 20px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.card-footer {
  border-top: 1px solid var(--border-color);
  padding: 16px 20px;
}

.card-content {
  position: relative;
}
</style>

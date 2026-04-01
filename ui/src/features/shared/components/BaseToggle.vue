<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from 'radix-vue';
import { animate, spring } from 'motion';
import { ref, watch, onMounted } from 'vue';

interface Props {
  modelValue: boolean;
  label?: string;
  disabled?: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits(['update:modelValue']);

const thumbRef = ref<HTMLElement | null>(null);

const handleUpdate = (val: boolean) => {
  emit('update:modelValue', val);
};

// Phase 5: Motion One thumb physics
watch(() => props.modelValue, (val) => {
  if (!thumbRef.value) return;
  
  (animate as any)(
    thumbRef.value,
    { x: val ? 20 : 0 },
    { easing: spring({ stiffness: 600, damping: 35 }) } as any
  );
});

onMounted(() => {
  if (thumbRef.value) {
    // Initial position
    thumbRef.value.style.transform = `translateX(${props.modelValue ? 20 : 0}px)`;
  }
});
</script>

<template>
  <div class="toggle-wrapper">
    <SwitchRoot
      :checked="modelValue"
      @update:checked="handleUpdate"
      :disabled="disabled"
      class="switch-root"
    >
      <SwitchThumb
        ref="thumbRef"
        class="switch-thumb"
      />
    </SwitchRoot>
    <span v-if="label" class="toggle-label">{{ label }}</span>
  </div>
</template>

<style scoped>
.toggle-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
}

.switch-root {
  width: 44px;
  height: 24px;
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: 100px;
  position: relative;
  border: 1px solid var(--border-color);
  cursor: pointer;
  transition: background-color 0.2s, border-color 0.2s;
  padding: 0;
  outline: none;
}

.switch-root[data-state="checked"] {
  background-color: var(--accent-primary);
  border-color: var(--accent-primary);
  box-shadow: 0 0 15px rgba(59, 130, 246, 0.2);
}

.switch-root:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.switch-root:focus-visible {
  box-shadow: 0 0 0 2px var(--accent-primary);
}

.switch-thumb {
  display: block;
  width: 18px;
  height: 18px;
  background-color: white;
  border-radius: 50%;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  position: absolute;
  top: 2px;
  left: 2px;
  will-change: transform;
}

.toggle-label {
  font-size: 0.85rem;
  color: var(--text-secondary);
  font-weight: 500;
}

/* Hover effects */
.switch-root:not(:disabled):hover {
  border-color: rgba(255, 255, 255, 0.2);
}
</style>

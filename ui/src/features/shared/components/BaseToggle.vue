<script setup lang="ts">
interface Props {
  modelValue: boolean;
  label?: string;
  disabled?: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits(['update:modelValue']);

const toggle = () => {
  if (!props.disabled) {
    emit('update:modelValue', !props.modelValue);
  }
};
</script>

<template>
  <button 
    class="base-toggle" 
    :class="{ 'is-active': modelValue, 'is-disabled': disabled }"
    @click="toggle"
    type="button"
    role="switch"
    :aria-checked="modelValue"
  >
    <div class="toggle-slider"></div>
  </button>
</template>

<style scoped>
.base-toggle {
  width: 44px;
  height: 24px;
  border-radius: 100px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-color);
  padding: 3px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  flex-shrink: 0;
}

.base-toggle.is-active {
  background: var(--accent-primary);
  border-color: var(--accent-primary);
  box-shadow: 0 0 15px rgba(59, 130, 246, 0.2);
}

.base-toggle.is-disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.toggle-slider {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.base-toggle.is-active .toggle-slider {
  transform: translateX(20px);
}

.base-toggle:not(.is-disabled):hover {
  border-color: rgba(255, 255, 255, 0.2);
}

.base-toggle:not(.is-disabled):active .toggle-slider {
  width: 20px;
}
</style>

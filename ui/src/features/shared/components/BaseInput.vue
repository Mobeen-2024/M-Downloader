<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  modelValue?: string | number;
  label?: string;
  type?: string;
  placeholder?: string;
  error?: string;
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  label: '',
  type: 'text',
  placeholder: '',
  error: '',
  disabled: false,
});

const emit = defineEmits(['update:modelValue', 'blur', 'focus']);

const onInput = (e: Event) => {
  const target = e.target as HTMLInputElement;
  emit('update:modelValue', target.value);
};

const classes = computed(() => {
  return [
    'base-input-container',
    { 'is-error': !!props.error, 'is-disabled': props.disabled },
  ];
});
</script>

<template>
  <div :class="classes">
    <label v-if="label" class="label">{{ label }}</label>
    <div class="input-wrapper">
      <input 
        :type="type" 
        :value="modelValue" 
        :placeholder="placeholder"
        :disabled="disabled"
        class="input"
        @input="onInput"
        @blur="$emit('blur')"
        @focus="$emit('focus')"
      />
      <div class="border-glow"></div>
    </div>
    <span v-if="error" class="error-text">{{ error }}</span>
  </div>
</template>

<style scoped>
.base-input-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.label {
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  margin-left: 4px;
}

.input-wrapper {
  position: relative;
  border-radius: var(--radius-md);
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-color);
  transition: var(--transition-smooth);
  backdrop-filter: var(--glass-blur);
}

.input-wrapper:hover:not(.is-disabled) {
  border-color: rgba(255, 255, 255, 0.15);
  background: rgba(255, 255, 255, 0.05);
}

.input-wrapper:focus-within {
  border-color: var(--accent-primary);
  background: rgba(255, 255, 255, 0.07);
}

.input {
  width: 100%;
  background: transparent;
  border: none;
  padding: 12px 16px;
  color: var(--text-primary);
  font-size: 0.95rem;
  font-family: inherit;
  outline: none;
}

.input::placeholder {
  color: var(--text-secondary);
  opacity: 0.5;
}

.border-glow {
  position: absolute;
  inset: -1px;
  border-radius: var(--radius-md);
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.3s ease;
  box-shadow: 0 0 15px var(--accent-primary);
  border: 1px solid var(--accent-primary);
}

.input-wrapper:focus-within .border-glow {
  opacity: 0.2;
}

.error-text {
  font-size: 0.75rem;
  color: var(--color-error);
  margin-left: 4px;
  font-weight: 500;
}

.is-error .input-wrapper {
  border-color: var(--color-error);
}
.is-error .border-glow {
  box-shadow: 0 0 15px var(--color-error);
  border-color: var(--color-error);
}

.is-disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.is-disabled .input {
  cursor: not-allowed;
}
</style>

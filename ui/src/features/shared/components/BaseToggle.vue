<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from 'radix-vue';
import { animate } from 'motion';
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

// Physics-based thumb movement
watch(() => props.modelValue, (val) => {
  if (!thumbRef.value) return;
  
  (animate as any)(
    thumbRef.value,
    { x: val ? 20 : 0 },
    { type: 'spring', stiffness: 700, damping: 30 }
  );
});

onMounted(() => {
  if (thumbRef.value) {
    thumbRef.value.style.transform = `translateX(${props.modelValue ? 20 : 0}px)`;
  }
});
</script>

<template>
  <div class="flex items-center gap-4 cursor-pointer select-none group" @click="handleUpdate(!modelValue)">
    <SwitchRoot
      :checked="modelValue"
      @update:checked="handleUpdate"
      :disabled="disabled"
      class="relative w-11 h-6 bg-white/5 border border-white/10 rounded-full transition-all duration-300 outline-none flex items-center px-1 focus:border-tactical-cyan/40"
      :class="{ 'bg-tactical-cyan/10 border-tactical-cyan/20': modelValue, 'opacity-40 pointer-events-none': disabled }"
    >
      <SwitchThumb
        ref="thumbRef"
        class="block w-4 h-4 rounded-full transition-shadow duration-300 bg-white/40 shadow-inner"
        :class="{ 'bg-tactical-cyan shadow-[0_0_12px_#00f2ff] !opacity-100': modelValue }"
      />
    </SwitchRoot>
    <span v-if="label" class="text-[10px] font-black uppercase tracking-[0.2em] transition-colors group-hover:text-white" :class="modelValue ? 'text-white' : 'text-white/30'">
      {{ label }}
    </span>
  </div>
</template>

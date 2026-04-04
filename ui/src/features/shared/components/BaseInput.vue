<script setup lang="ts">
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
</script>

<template>
  <div class="flex flex-col gap-2 w-full group" :class="{ 'opacity-50 pointer-events-none': disabled }">
    <label v-if="label" class="text-[10px] font-black uppercase text-white/40 tracking-[0.2em] pl-1 group-focus-within:text-tactical-cyan transition-colors">
      {{ label }}
    </label>
    
    <div class="relative flex items-center bg-white/5 border border-white/10 rounded-xl overflow-hidden focus-within:border-tactical-cyan/50 focus-within:bg-black/40 transition-all duration-300">
      <!-- Left Icon Slot -->
      <div v-if="$slots['icon-left']" class="pl-4 text-white/30">
        <slot name="icon-left"></slot>
      </div>
      
      <input 
        :type="type" 
        :value="modelValue" 
        :placeholder="placeholder"
        :disabled="disabled"
        class="flex-1 bg-transparent border-none outline-none px-4 py-3 text-[13px] text-white placeholder:text-white/10 font-medium tracking-tight"
        @input="onInput"
        @blur="$emit('blur')"
        @focus="$emit('focus')"
      />

      <!-- Right Icon Slot -->
      <div v-if="$slots['icon-right']" class="pr-4 text-white/30">
        <slot name="icon-right"></slot>
      </div>

      <!-- Tactical Focus Line -->
      <div class="absolute bottom-0 left-0 right-0 h-[2px] bg-tactical-cyan shadow-[0_0_10px_#00f2ff] scale-x-0 group-focus-within:scale-x-100 transition-transform duration-500 origin-left"></div>
    </div>
    
    <span v-if="error" class="text-[9px] font-black uppercase text-red-500 tracking-widest pl-1 mt-1 animate-pulse">
      {{ error }}
    </span>
  </div>
</template>

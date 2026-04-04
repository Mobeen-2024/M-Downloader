<script setup lang="ts">
import { computed, ref } from 'vue';
import { animate } from 'motion';

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

const variants = {
  primary: 'bg-tactical-cyan text-black shadow-[0_0_15px_rgba(0,242,255,0.3)] hover:shadow-[0_0_25px_rgba(0,242,255,0.5)]',
  secondary: 'bg-white/5 text-white/80 border border-white/10 hover:bg-white/10 hover:border-white/20',
  danger: 'bg-red-500/20 text-red-500 border border-red-500/30 hover:bg-red-500 hover:text-white',
  ghost: 'bg-transparent text-white/40 hover:text-white hover:bg-white/5',
  glass: 'bg-white/5 backdrop-blur-md text-white/80 border border-white/10 hover:bg-white/10 hover:border-white/20'
};

const sizes = {
  sm: 'px-3 py-1.5 text-[9px] rounded-lg gap-1.5',
  md: 'px-5 py-2.5 text-[11px] rounded-xl gap-2',
  lg: 'px-8 py-3.5 text-xs rounded-2xl gap-3',
  icon: 'p-2 rounded-xl'
};

const classes = computed(() => {
  return [
    'relative flex items-center justify-center font-black uppercase tracking-tighter transition-all active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed overflow-hidden whitespace-nowrap',
    variants[props.variant],
    sizes[props.size],
    { 'animate-pulse pointer-events-none opacity-70': props.loading }
  ];
});

// Subtle tactical interaction
const handlePointerEnter = () => {
  if (props.disabled || props.loading || !buttonRef.value) return;
  (animate as any)(buttonRef.value, 
    { scale: 1.025 }, 
    { type: 'spring', stiffness: 500, damping: 15 }
  );
};

const handlePointerLeave = () => {
  if (props.disabled || props.loading || !buttonRef.value) return;
  (animate as any)(buttonRef.value, 
    { scale: 1 }, 
    { type: 'spring', stiffness: 500, damping: 15 }
  );
};
</script>

<template>
  <button
    ref="buttonRef"
    :type="type"
    :class="classes"
    :disabled="disabled || loading"
    @pointerenter="handlePointerEnter"
    @pointerleave="handlePointerLeave"
  >
    <div v-if="loading" class="absolute inset-0 flex items-center justify-center bg-inherit">
      <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin"></div>
    </div>
    
    <div class="flex items-center gap-inherit" :class="{ 'opacity-0': loading }">
      <slot name="icon-left"></slot>
      <slot></slot>
      <slot name="icon-right"></slot>
    </div>

    <!-- Inner Glow for Primary -->
    <div v-if="variant === 'primary'" class="absolute inset-0 bg-white/20 opacity-0 group-hover:opacity-100 mix-blend-overlay pointer-events-none transition-opacity"></div>
  </button>
</template>

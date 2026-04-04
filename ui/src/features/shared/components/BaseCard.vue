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

const variants = {
  glass: 'bg-white/5 backdrop-blur-xl border border-white/10 shadow-[0_8px_32px_0_rgba(0,0,0,0.37)]',
  outline: 'bg-transparent border border-white/10',
  flat: 'bg-black/20 border border-white/5'
};

const paddings = {
  none: 'p-0',
  sm: 'p-4',
  md: 'p-8',
  lg: 'p-12'
};

const classes = computed(() => {
  return [
    'relative rounded-[2rem] overflow-hidden transition-all duration-500',
    variants[props.variant],
    paddings[props.padding],
    { 'hover:bg-white/[0.08] hover:border-white/20 hover:-translate-y-1': props.hoverable },
  ];
});
</script>

<template>
  <div :class="classes">
    <header v-if="$slots.header" class="mb-6">
      <slot name="header"></slot>
    </header>
    
    <main>
      <slot></slot>
    </main>
    
    <footer v-if="$slots.footer" class="mt-8 pt-6 border-t border-white/5">
      <slot name="footer"></slot>
    </footer>
  </div>
</template>

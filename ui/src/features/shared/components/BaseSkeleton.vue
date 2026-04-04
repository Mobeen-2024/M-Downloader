<script setup lang="ts">
interface Props {
  width?: string;
  height?: string;
  borderRadius?: string;
  variant?: 'text' | 'rect' | 'circle';
  animation?: 'wave' | 'pulse' | 'none';
}

withDefaults(defineProps<Props>(), {
  width: '100%',
  height: '20px',
  borderRadius: '12px',
  variant: 'rect',
  animation: 'pulse'
});
</script>

<template>
  <div 
    class="bg-white/5 backdrop-blur-xl border border-white/10 shadow-[0_8px_32px_0_rgba(0,0,0,0.37)]"
    :class="[
      variant === 'circle' ? 'rounded-full' : 'rounded-2xl',
      { 'animate-pulse': animation === 'pulse' }
    ]"
    :style="{
      width,
      height,
      borderRadius: variant === 'circle' ? '50%' : borderRadius
    }"
  ></div>
</template>

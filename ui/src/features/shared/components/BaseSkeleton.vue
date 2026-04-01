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
  borderRadius: '4px',
  variant: 'rect',
  animation: 'wave'
});
</script>

<template>
  <div 
    class="base-skeleton"
    :class="[
      `variant-${variant}`,
      `animation-${animation}`
    ]"
    :style="{
      width,
      height,
      borderRadius: variant === 'circle' ? '50%' : borderRadius
    }"
  ></div>
</template>

<style scoped>
.base-skeleton {
  background: rgba(255, 255, 255, 0.05);
  position: relative;
  overflow: hidden;
}

.variant-text {
  height: 1em;
  margin-bottom: 0.5em;
}

.animation-pulse {
  animation: pulse 1.5s ease-in-out infinite;
}

.animation-wave::after {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    90deg,
    transparent 0%,
    rgba(255, 255, 255, 0.03) 20%,
    rgba(255, 255, 255, 0.08) 50%,
    rgba(255, 255, 255, 0.03) 80%,
    transparent 100%
  );
  animation: wave 1.6s linear infinite;
  transform: translateX(-100%);
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.4; }
  100% { opacity: 1; }
}

@keyframes wave {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}
</style>

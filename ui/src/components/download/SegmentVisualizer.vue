<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import type { SegmentInfo } from '../../types/download';

const props = defineProps<{
  segments: SegmentInfo[];
  total: number;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
let animationId: number;

const draw = () => {
  const canvas = canvasRef.value;
  if (!canvas || props.total === 0) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const { width, height } = canvas;
  ctx.clearRect(0, 0, width, height);

  // Background track
  ctx.fillStyle = 'rgba(255, 255, 255, 0.05)';
  ctx.fillRect(0, 0, width, height);

  const time = Date.now() / 1000;

  props.segments.forEach((seg, idx) => {
    const startX = (seg.start / props.total) * width;
    const endX = (seg.end / props.total) * width;
    const segWidth = Math.max(1, endX - startX);

    // 1. Draw segment background (pending area)
    ctx.fillStyle = 'rgba(255, 255, 255, 0.03)';
    ctx.fillRect(startX, 0, segWidth, height);

    // 2. Draw downloaded progress
    const progressWidth = ((seg.downloaded) / (seg.end - seg.start + 1)) * segWidth;
    
    // Create gradient for the segment
    const gradient = ctx.createLinearGradient(startX, 0, startX + progressWidth, 0);
    gradient.addColorStop(0, '#3b82f6'); // blue-500
    gradient.addColorStop(1, '#60a5fa'); // blue-400
    
    ctx.fillStyle = gradient;
    ctx.fillRect(startX, 0, progressWidth, height);

    // 3. Active pulse effect
    if (seg.active && seg.downloaded < (seg.end - seg.start + 1)) {
      const shimmerX = (time * 150) % (progressWidth + 40) - 20;
      const shimmerGrad = ctx.createLinearGradient(startX + shimmerX, 0, startX + shimmerX + 20, 0);
      shimmerGrad.addColorStop(0, 'rgba(255,255,255,0)');
      shimmerGrad.addColorStop(0.5, 'rgba(255,255,255,0.4)');
      shimmerGrad.addColorStop(1, 'rgba(255,255,255,0)');
      
      ctx.fillStyle = shimmerGrad;
      ctx.fillRect(startX, 0, progressWidth, height);
    }

    // 4. Draw separator between segments
    if (idx < props.segments.length - 1) {
      ctx.fillStyle = 'rgba(0, 0, 0, 0.3)';
      ctx.fillRect(endX, 0, 1, height);
    }
  });

  animationId = requestAnimationFrame(draw);
};

onMounted(() => {
  draw();
  window.addEventListener('resize', draw);
});

onUnmounted(() => {
  cancelAnimationFrame(animationId);
  window.removeEventListener('resize', draw);
});

watch(() => props.segments, () => {
  // Re-draw on segment updates (handled by the RAF loop mainly, but ensures freshness)
}, { deep: true });
</script>

<template>
  <div class="canvas-container">
    <canvas 
      ref="canvasRef" 
      width="800" 
      height="12"
      class="segment-canvas"
    ></canvas>
  </div>
</template>

<style scoped>
.canvas-container {
  width: 100%;
  height: 12px;
  background: rgba(0, 0, 0, 0.4);
  border-radius: 4px;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.segment-canvas {
  width: 100%;
  height: 100%;
  display: block;
}
</style>

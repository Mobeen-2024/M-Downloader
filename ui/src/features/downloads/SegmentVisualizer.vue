<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import type { SegmentInfo } from '@/types/download';

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

  // High-DPI scaling logic
  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  
  if (canvas.width !== rect.width * dpr || canvas.height !== rect.height * dpr) {
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
  }

  const { width, height } = canvas;
  ctx.save();
  ctx.scale(dpr, dpr);
  
  const logicalWidth = width / dpr;
  const logicalHeight = height / dpr;

  ctx.clearRect(0, 0, logicalWidth, logicalHeight);

  // 1. Foundation: Unassigned/Pending area (IDM WHITE)
  ctx.fillStyle = 'rgba(255, 255, 255, 0.1)';
  ctx.fillRect(0, 0, logicalWidth, logicalHeight);

  props.segments.forEach((seg) => {
    const startX = (seg.start / props.total) * logicalWidth;
    const endX = (seg.end / props.total) * logicalWidth;
    const segWidth = Math.max(0.5, endX - startX);

    // 2. Completed Data (IDM BLUE)
    // We draw the progress from starting position up to current downloaded position.
    const segmentRange = (seg.end - seg.start + 1);
    const downloadedPercent = (seg.downloaded / segmentRange);
    const progressWidth = downloadedPercent * segWidth;

    if (seg.downloaded > 0) {
      if (seg.state === 'Completed') {
        ctx.fillStyle = '#0ea5e9'; // Static HD Blue
      } else {
        ctx.fillStyle = '#3b82f6';
      }
      ctx.fillRect(startX, 0, Math.ceil(progressWidth), logicalHeight);
    }

    // 3. Active Thread Head (IDM PINK LINE)
    if (seg.state === 'Active' && seg.downloaded < segmentRange) {
      const headX = startX + progressWidth;
      
      const headGrad = ctx.createLinearGradient(headX - 4, 0, headX + 4, 0);
      headGrad.addColorStop(0, 'rgba(236, 72, 153, 0)');
      headGrad.addColorStop(0.5, 'rgba(236, 72, 153, 0.8)');
      headGrad.addColorStop(1, 'rgba(236, 72, 153, 0)');
      
      ctx.fillStyle = headGrad;
      ctx.fillRect(headX - 4, 0, 8, logicalHeight);
      
      ctx.fillStyle = '#ec4899'; // Neon Pink
      ctx.fillRect(headX - 1, 0, 2, logicalHeight);
    }

    // 4. Segment Divider (Grid)
    if (segWidth > 5) {
      ctx.fillStyle = 'rgba(255, 255, 255, 0.03)';
      ctx.fillRect(startX, 0, 1, logicalHeight);
    }
  });

  ctx.restore();
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

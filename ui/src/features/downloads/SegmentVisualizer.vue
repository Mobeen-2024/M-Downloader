<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import type { SegmentInfo } from '@/types/download';

const props = defineProps<{
  segments: SegmentInfo[];
  total: number;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
let animationId: number;
let time = 0;

const draw = () => {
  const canvas = canvasRef.value;
  if (!canvas || props.total === 0) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  time += 0.02;

  // High-DPI scaling logic
  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  
  if (canvas.width !== Math.floor(rect.width * dpr) || canvas.height !== Math.floor(rect.height * dpr)) {
    canvas.width = Math.floor(rect.width * dpr);
    canvas.height = Math.floor(rect.height * dpr);
  }

  const { width, height } = canvas;
  ctx.save();
  ctx.scale(dpr, dpr);
  
  const logicalWidth = width / dpr;
  const logicalHeight = height / dpr;

  ctx.clearRect(0, 0, logicalWidth, logicalHeight);

  // 1. Foundation: Unassigned/Pending area
  ctx.fillStyle = 'rgba(255, 255, 255, 0.05)';
  ctx.fillRect(0, 0, logicalWidth, logicalHeight);

  if (!props.segments || props.segments.length === 0) {
    ctx.restore();
    return;
  }

  props.segments.forEach((seg) => {
    const startX = (seg.start / props.total) * logicalWidth;
    const endX = (seg.end / props.total) * logicalWidth;
    const segWidth = Math.max(1, endX - startX);

    // 2. Completed Data (Modern Blue Gradient)
    const segmentRange = Math.max(1, (seg.end - seg.start + 1));
    const downloadedPercent = (seg.downloaded / segmentRange);
    const progressWidth = downloadedPercent * segWidth;

    if (seg.downloaded > 0) {
      const gradient = ctx.createLinearGradient(startX, 0, startX + progressWidth, 0);
      const alpha = seg.state === 'Completed' ? 1 : 0.7;
      
      gradient.addColorStop(0, `rgba(59, 130, 246, ${alpha})`);
      gradient.addColorStop(0.5, `rgba(37, 99, 235, ${alpha})`);
      gradient.addColorStop(1, `rgba(29, 78, 216, ${alpha})`);
      
      ctx.fillStyle = gradient;
      ctx.fillRect(startX, 0, Math.ceil(progressWidth), logicalHeight);

      // Subtle horizontal stripe pattern
      ctx.fillStyle = 'rgba(255, 255, 255, 0.03)';
      for (let i = 0; i < logicalHeight; i += 4) {
        ctx.fillRect(startX, i, progressWidth, 1);
      }
    }

    // 3. Active Thread Head (Pro Glow Effect)
    if (seg.state === 'Active' && seg.downloaded < segmentRange) {
      const headX = Math.min(logicalWidth, startX + progressWidth);
      
      ctx.save();
      
      // Outer Bloom
      ctx.shadowBlur = 15 + Math.sin(time * 5) * 5;
      ctx.shadowColor = 'rgba(244, 114, 182, 0.6)';
      ctx.fillStyle = 'rgba(244, 114, 182, 0.4)';
      ctx.fillRect(headX - 4, 0, 8, logicalHeight);

      // Core Sparkle
      ctx.shadowBlur = 4;
      ctx.shadowColor = '#ffffff';
      ctx.fillStyle = '#ffffff';
      ctx.fillRect(headX - 1, 0, 2, logicalHeight);

      // Photon Scanning Effect
      const photonY = ((time * 40) % (logicalHeight * 2)) - logicalHeight;
      const photonGrad = ctx.createRadialGradient(headX, photonY, 0, headX, photonY, 15);
      photonGrad.addColorStop(0, 'rgba(255, 255, 255, 0.8)');
      photonGrad.addColorStop(1, 'rgba(255, 255, 255, 0)');
      ctx.fillStyle = photonGrad;
      ctx.fillRect(headX - 10, 0, 20, logicalHeight);

      ctx.restore();
    }

    // 4. Subtle Dividers
    if (segWidth > 15) {
      ctx.fillStyle = 'rgba(0, 0, 0, 0.3)';
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
</script>

<template>
  <div class="visualizer-container">
    <canvas 
      ref="canvasRef" 
      class="segment-canvas"
    ></canvas>
    <div class="scanline"></div>
  </div>
</template>

<style scoped>
.visualizer-container {
  width: 100%;
  height: 14px;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 6px;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.1);
  position: relative;
  box-shadow: inset 0 2px 8px rgba(0, 0, 0, 0.5);
}

.segment-canvas {
  width: 100%;
  height: 100%;
  display: block;
}

.scanline {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    to bottom,
    transparent 50%,
    rgba(0, 0, 0, 0.1) 50%
  );
  background-size: 100% 2px;
  pointer-events: none;
  opacity: 0.5;
}
</style>

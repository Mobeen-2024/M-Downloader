<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';

const props = defineProps<{
  currentSpeed: number;
  color?: string;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const history = ref<number[]>(new Array(60).fill(0));
let animationId: number;

// Update history every second
watch(() => props.currentSpeed, (newVal) => {
  history.value.push(newVal);
  if (history.value.length > 60) {
    history.value.shift();
  }
});

const draw = () => {
  const canvas = canvasRef.value;
  if (!canvas) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const { width, height } = canvas;
  ctx.clearRect(0, 0, width, height);

  if (history.value.length < 2) return;

  const max = Math.max(...history.value, 1);
  const step = width / (history.value.length - 1);

  ctx.beginPath();
  ctx.lineWidth = 2;
  ctx.strokeStyle = props.color || '#3b82f6';
  ctx.lineJoin = 'round';
  ctx.lineCap = 'round';

  history.value.forEach((val, i) => {
    const x = i * step;
    const y = height - (val / max) * (height - 4) - 2;
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  });

  // Create gradient fill
  const gradient = ctx.createLinearGradient(0, 0, 0, height);
  gradient.addColorStop(0, (props.color || '#3b82f6') + '44');
  gradient.addColorStop(1, 'transparent');

  ctx.stroke();

  // Close path for fill
  ctx.lineTo(width, height);
  ctx.lineTo(0, height);
  ctx.fillStyle = gradient;
  ctx.fill();

  animationId = requestAnimationFrame(draw);
};

onMounted(() => {
  draw();
});

onUnmounted(() => {
  cancelAnimationFrame(animationId);
});
</script>

<template>
  <div class="speed-graph">
    <canvas ref="canvasRef" width="120" height="32"></canvas>
  </div>
</template>

<style scoped>
.speed-graph {
  width: 120px;
  height: 32px;
  opacity: 0.8;
}
canvas {
  width: 100%;
  height: 100%;
}
</style>

<script setup lang="ts">
import { computed } from 'vue';

interface SegmentInfo {
  start: number;
  end: number;
  downloaded: number;
  state: string;
}

const props = defineProps<{
  totalSize: number;
  segments: SegmentInfo[];
}>();

const segmentStyles = computed(() => {
  return props.segments.map(seg => {
    // Total range of this segment (Pink bar)
    const rangeSize = seg.end - seg.start + 1;
    const rangeWidth = (rangeSize / props.totalSize) * 100;
    const rangeLeft = (seg.start / props.totalSize) * 100;

    // Downloaded portion of this segment (Blue bar)
    // The blue bar starts at the same 'left' as the pink bar.
    const downloadedWidth = (seg.downloaded / rangeSize) * rangeWidth;

    return {
      pink: {
        left: `${rangeLeft}%`,
        width: `${rangeWidth}%`,
        minWidth: '1px'
      },
      blue: {
        width: `${downloadedWidth}%`
      },
      id: `${seg.start}-${seg.end}`,
      completed: seg.state === 'Completed'
    };
  });
});
</script>

<template>
  <div class="segment-container">
    <!-- Base background (Gray) -->
    <div class="segment-track"></div>

    <!-- Render each segment -->
    <div 
      v-for="seg in segmentStyles" 
      :key="seg.id"
      class="segment-range"
      :style="seg.pink"
      :class="{ 'is-completed': seg.completed }"
    >
      <!-- The 'Pink' denotes the assigned range for this thread -->
      <div class="segment-fill-pending"></div>
      
      <!-- The 'Blue' denotes the actual written data -->
      <div class="segment-fill-done" :style="seg.blue"></div>
    </div>
  </div>
</template>

<style scoped>
.segment-container {
  position: relative;
  width: 100%;
  height: 12px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 4px;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.segment-track {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: var(--bg-secondary, #2c2c2c);
}

.segment-range {
  position: absolute;
  top: 0;
  height: 100%;
  display: flex;
  align-items: center;
  transition: all 0.3s ease;
}

.segment-fill-pending {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: #ff69b4; /* Iconic IDM Pink */
  opacity: 0.3;
}

.segment-fill-done {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  background: #3498db; /* Iconic IDM Blue */
  box-shadow: 0 0 8px rgba(52, 152, 219, 0.5);
  transition: width 0.2s linear;
}

.is-completed .segment-fill-pending {
  opacity: 0;
}

.is-completed .segment-fill-done {
  width: 100% !important;
  background: #2ecc71; /* Success Green for completed segments */
  box-shadow: none;
}
</style>

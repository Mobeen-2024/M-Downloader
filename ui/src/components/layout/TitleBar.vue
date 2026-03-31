<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X, Copy } from 'lucide-vue-next';
import { useRoute } from 'vue-router';

const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
const appWindow = isTauri ? getCurrentWindow() : null;
const isMaximized = ref(false);
const route = useRoute();
let unlisten: any;

const minimize = () => isTauri && appWindow?.minimize();
const toggleMaximize = async () => {
  if (!isTauri || !appWindow) return;
  await appWindow.toggleMaximize();
  isMaximized.value = await appWindow.isMaximized();
};
const close = () => isTauri && appWindow?.close();

onMounted(async () => {
  if (isTauri && appWindow) {
    isMaximized.value = await appWindow.isMaximized();
    unlisten = await appWindow.onResized(async () => {
      isMaximized.value = await appWindow.isMaximized();
    });
  }
});

onUnmounted(() => {
  if (unlisten) unlisten();
});
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="title-section" data-tauri-drag-region>
      <div class="app-dot"></div>
      <span class="app-name" data-tauri-drag-region>Mdownloader</span>
      <span class="route-name" data-tauri-drag-region>// {{ route.name }}</span>
    </div>

    <div class="window-controls">
      <button class="control-btn" title="Minimize" @click="minimize">
        <Minus :size="14" />
      </button>
      <button class="control-btn" :title="isMaximized ? 'Restore' : 'Maximize'" @click="toggleMaximize">
        <component :is="isMaximized ? Copy : Square" :size="12" />
      </button>
      <button class="control-btn close-btn" title="Close" @click="close">
        <X :size="16" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: 32px;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  justify-content: space-between;
  align-items: center;
  user-select: none;
  border-bottom: 1px solid var(--border-color);
  z-index: 9999;
}

.title-section {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-left: 16px;
  height: 100%;
  flex: 1;
}

.app-dot {
  width: 8px;
  height: 8px;
  background: var(--accent-primary);
  border-radius: 50%;
  box-shadow: 0 0 8px var(--accent-primary);
}

.app-name {
  font-size: 0.7rem;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-primary);
}

.route-name {
  font-size: 0.65rem;
  font-weight: 500;
  color: var(--text-secondary);
  font-family: var(--font-mono);
}

.window-controls {
  display: flex;
  height: 100%;
}

.control-btn {
  width: 44px;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-primary);
}

.close-btn:hover {
  background: #e81123 !important;
  color: white !important;
}
</style>

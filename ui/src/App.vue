<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import TitleBar from '@/features/layout/TitleBar.vue';
import AppSidebar from '@/features/layout/AppSidebar.vue';
import AppTopBar from '@/features/layout/AppTopBar.vue';
import MediaSnifferHud from '@/features/sniffer/MediaSnifferHud.vue';
import NewDownloadModal from '@/features/shared/modals/NewDownloadModal.vue';
import { useIpcEvents } from '@/composables/useIpcEvents';
import { useSettingsStore } from '@/stores/settings.store';

const showNewModal = ref(false);
const { initListeners } = useIpcEvents();
const settings = useSettingsStore();

// Apply theme class to root
watch(() => settings.themeAccent, (newTheme) => {
  document.documentElement.className = `theme-${newTheme}`;
}, { immediate: true });

onMounted(() => {
  initListeners();
});
</script>

<template>
  <div id="app" class="app-shell">
    <TitleBar />
    <MediaSnifferHud />
    
    <div class="app-body">
      <AppSidebar />
      
      <main class="main-content">
        <AppTopBar @new-download="showNewModal = true" />
        
        <div class="page-content">
          <router-view v-slot="{ Component }">
            <transition name="fade" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </div>
      </main>
    </div>

    <NewDownloadModal 
      :show="showNewModal" 
      @close="showNewModal = false" 
    />
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh;
  background: var(--bg-primary);
  overflow: hidden;
}

.app-body {
  display: flex;
  flex: 1;
  min-height: 0; /* Important for flex child scrolling */
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  position: relative;
}

.page-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  position: relative;
}

/* Page Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>

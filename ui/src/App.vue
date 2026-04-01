<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import TitleBar from '@/features/layout/TitleBar.vue';
import AppSidebar from '@/features/layout/AppSidebar.vue';
import AppTopBar from '@/features/layout/AppTopBar.vue';
import MediaSnifferHud from '@/features/sniffer/MediaSnifferHud.vue';
import NewDownloadModal from '@/features/shared/modals/NewDownloadModal.vue';
import ToastProvider from '@/features/shared/components/ToastProvider.vue';
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
    <ToastProvider />
    <MediaSnifferHud />
    
    <div class="app-body">
      <AppSidebar />
      
      <main class="main-content">
        <AppTopBar @new-download="showNewModal = true" />
        
        <div class="page-content">
          <router-view v-slot="{ Component }">
            <transition name="page-fade" mode="out-in">
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
  position: relative;
}

.app-body {
  display: flex;
  flex: 1;
  min-height: 0;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  position: relative;
  background: radial-gradient(circle at 50% -20%, rgba(59, 130, 246, 0.03) 0%, transparent 70%);
}

.page-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  position: relative;
  padding: 0;
}

/* Professional Page Transitions */
.page-fade-enter-active,
.page-fade-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.page-fade-enter-from {
  opacity: 0;
  transform: translateY(10px) scale(0.99);
}

.page-fade-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(1.01);
}
</style>

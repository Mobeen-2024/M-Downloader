<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import TitleBar from '@/features/layout/TitleBar.vue';
import AppSidebar from '@/features/layout/AppSidebar.vue';
import AppBottomNav from '@/features/layout/AppBottomNav.vue';
import AppTopBar from '@/features/layout/AppTopBar.vue';
import MediaSnifferHud from '@/features/sniffer/MediaSnifferHud.vue';
import NewDownloadModal from '@/features/shared/modals/NewDownloadModal.vue';
import ToastProvider from '@/features/shared/components/ToastProvider.vue';
import { useIpcEvents } from '@/composables/useIpcEvents';
import { useSettingsStore } from '@/stores/settings.store';

const showNewModal = ref(false);
const isMobile = ref(window.innerWidth < 768);
const { initListeners } = useIpcEvents();
const settings = useSettingsStore();

const updateLayout = () => {
  isMobile.value = window.innerWidth < 768;
};

// Apply theme class to root
watch(() => settings.themeAccent, (newTheme) => {
  document.documentElement.className = `theme-${newTheme}`;
}, { immediate: true });

onMounted(() => {
  initListeners();
  window.addEventListener('resize', updateLayout);
});

onUnmounted(() => {
  window.removeEventListener('resize', updateLayout);
});
</script>

<template>
  <div class="app-grid-layout" :class="{ 'is-mobile': isMobile }">
    <!-- Overlay Layer for Global Notifications & HUDs -->
    <div class="fixed inset-0 pointer-events-none z-[1000]">
      <div class="pointer-events-auto">
        <ToastProvider />
        <MediaSnifferHud v-if="!isMobile" />
      </div>
    </div>

    <!-- TitleBar (Desktop Only) -->
    <header v-if="!isMobile" class="layout-titlebar">
      <TitleBar />
    </header>

    <!-- Sidebar (Desktop Only) -->
    <aside v-if="!isMobile" class="layout-sidebar border-r border-white/5 bg-bg-surface/50 backdrop-blur-xl">
      <AppSidebar />
    </aside>
      
    <!-- TopBar -->
    <section class="layout-topbar border-b border-white/5 bg-bg-surface/30 backdrop-blur-lg">
      <AppTopBar @new-download="showNewModal = true" />
    </section>
      
    <!-- Dynamic View Port -->
    <main class="layout-main bg-[radial-gradient(circle_at_50%_0%,rgba(0,242,255,0.02)_0%,transparent_80%)]">
      <router-view v-slot="{ Component }">
        <transition name="page-fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>

    <!-- Bottom Navigation (Mobile Only) -->
    <nav v-if="isMobile" class="layout-bottomnav border-t border-white/5 bg-bg-surface/80 backdrop-blur-2xl">
      <AppBottomNav />
    </nav>

    <!-- Global Modals -->
    <NewDownloadModal 
      :show="showNewModal" 
      @close="showNewModal = false" 
    />
  </div>
</template>

<style>
/* ─── MACRO LAYOUT GRID SYSTEM (ZERO OVERLAP) ─── */
.app-grid-layout {
  display: grid;
  height: 100vh;
  width: 100vw;
  background-color: var(--bg-primary);
  color: white;
  overflow: hidden;
  position: relative;
}

/* Desktop Grid */
.app-grid-layout:not(.is-mobile) {
  grid-template-columns: var(--sidebar-width) minmax(0, 1fr);
  grid-template-rows: var(--titlebar-height) var(--topbar-height) minmax(0, 1fr);
  grid-template-areas:
    "titlebar titlebar"
    "sidebar topbar"
    "sidebar main";
}

/* Mobile Grid */
.app-grid-layout.is-mobile {
  grid-template-columns: 1fr;
  grid-template-rows: var(--topbar-height) minmax(0, 1fr) 60px;
  grid-template-areas:
    "topbar"
    "main"
    "bottomnav";
}

.layout-titlebar { grid-area: titlebar; z-index: 100; min-width: 0; }
.layout-sidebar  { grid-area: sidebar; z-index: 50; min-height: 0; }
.layout-topbar   { grid-area: topbar; z-index: 40; min-width: 0; }
.layout-main     { grid-area: main; z-index: 10; position: relative; overflow: hidden; min-width: 0; }
.layout-bottomnav{ grid-area: bottomnav; z-index: 50; min-width: 0; }

/* Global page transitions */
.page-fade-enter-active, .page-fade-leave-active {
  transition: opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1), transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
.page-fade-enter-from { opacity: 0; transform: translateY(4px); }
.page-fade-leave-to { opacity: 0; transform: translateY(-4px); }
</style>

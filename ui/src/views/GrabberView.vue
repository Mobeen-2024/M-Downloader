<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { 
  PhGlobe, 
  PhDownloadSimple, 
  PhVideoCamera, 
  PhMusicNotes, 
  PhMagnifyingGlass, 
  PhCheckSquare, 
  PhSquare, 
  PhFunnel, 
  PhStack, 
  PhPackage, 
  PhArchive, 
  PhFileText, 
  PhGearSix, 
  PhTrash, 
  PhTrendUp,
  PhShieldWarning,
  PhDatabase,
  PhActivity,
  PhStopCircle
} from "@phosphor-icons/vue";
import BaseSkeleton from '@/features/shared/components/BaseSkeleton.vue';
import BaseToggle from '@/features/shared/components/BaseToggle.vue';
import BaseInput from '@/features/shared/components/BaseInput.vue';
import BaseButton from '@/features/shared/components/BaseButton.vue';
import { useGrabberStore, type DiscoveryScope } from '@/stores/grabber.store';

const grabberStore = useGrabberStore();

// Discovery Configuration
const url = ref('');
const maxDepth = ref(1);
const discoveryScope = ref<DiscoveryScope>('SameDomain');
const externalLinkDepth = ref(0);
const useHeadless = ref(false);
const showMobileConfig = ref(false);
const activeCategory = ref('All');
const isViewLoading = ref(true);
const isMobile = ref(window.innerWidth < 1024);

const updateIsMobile = () => {
  isMobile.value = window.innerWidth < 1024;
};

onMounted(() => {
  window.addEventListener('resize', updateIsMobile);
  setTimeout(() => { isViewLoading.value = false; }, 600);
});

onUnmounted(() => { 
  window.removeEventListener('resize', updateIsMobile);
  grabberStore.cleanup(); 
});

const handleScan = async () => {
  if (!url.value) return;
  await grabberStore.startCrawl(url.value, maxDepth.value, discoveryScope.value, externalLinkDepth.value, useHeadless.value);
};

const handleStop = async () => { await grabberStore.stopCrawl(); };

const filteredAssets = computed(() => {
  if (activeCategory.value === 'All') return grabberStore.assets;
  return grabberStore.assets.filter(a => a.category === activeCategory.value);
});

const categories = [
  { name: 'All',        icon: PhStack,        color: 'text-tactical-cyan' },
  { name: 'Video',      icon: PhVideoCamera,  color: 'text-pink-500' },
  { name: 'Compressed', icon: PhArchive,      color: 'text-orange-500' },
  { name: 'Programs',   icon: PhPackage,      color: 'text-emerald-500' },
  { name: 'Music',      icon: PhMusicNotes,    color: 'text-purple-500' },
  { name: 'Documents',  icon: PhFileText,     color: 'text-blue-500' },
];

const counts = computed(() => {
  const map: Record<string, number> = { All: grabberStore.assets.length };
  grabberStore.assets.forEach(a => { map[a.category] = (map[a.category] || 0) + 1; });
  return map;
});

const formatSize = (bytes: number) => {
  if (bytes === 0) return '0 B';
  const k = 1024, sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

const allSelected = computed(() => {
  const c = filteredAssets.value;
  return c.length > 0 && c.every(a => a.selected);
});

const toggleSelectAll = () => {
  const v = !allSelected.value;
  filteredAssets.value.forEach(a => { a.selected = v; });
};

const handleBulkDownload = () => { grabberStore.bulkAddSelected(true); };

const getCategoryIcon = (name: string) =>
  categories.find(c => c.name === name)?.icon || PhFileText;

const scopeLabel = (s: DiscoveryScope) =>
  s === 'Folder' ? 'FOLDER_ONLY' : s === 'SameDomain' ? 'SAME_HOST' : 'CROSS_SUBDOMAIN';
</script>

<template>
  <div class="h-full flex flex-col p-6 md:p-12 gap-6 md:gap-8 overflow-hidden select-none relative">
    <!-- View Header -->
    <header class="flex flex-col sm:flex-row justify-between items-start sm:items-end gap-6 shrink-0">
      <div class="space-y-1">
        <h2 class="text-2xl md:text-3xl font-black uppercase tracking-tighter text-white">Discovery Frontier</h2>
        <p class="text-[10px] font-bold text-white/40 tracking-widest uppercase flex items-center gap-2">
          <PhTrendUp :size="12" class="text-tactical-cyan" />
          Automated Intercept Hub // Industrial Spec
        </p>
      </div>

      <div class="flex items-center gap-3 md:gap-4 bg-white/5 border border-white/5 rounded-xl p-2 px-4 backdrop-blur-md w-full sm:w-auto overflow-x-auto">
        <div class="flex flex-col items-center min-w-[50px] md:min-w-[60px]">
          <PhActivity :size="14" :class="grabberStore.activeWorkers > 0 ? 'text-tactical-cyan' : 'opacity-20'" />
          <span class="text-sm font-data font-black text-white leading-tight mt-1">{{ grabberStore.activeWorkers }}</span>
          <span class="text-[8px] font-black text-white/30 uppercase">Nodes</span>
        </div>
        <div class="w-[1px] h-6 md:h-8 bg-white/5"></div>
        <div class="flex flex-col items-center min-w-[50px] md:min-w-[60px]">
          <PhShieldWarning :size="14" :class="grabberStore.authActive ? 'text-tactical-cyan' : 'opacity-20'" />
          <span class="text-sm font-data font-black text-white leading-tight mt-1">{{ grabberStore.authActive ? 'PRO' : 'SEC' }}</span>
          <span class="text-[8px] font-black text-white/30 uppercase">Vault</span>
        </div>
        <div class="w-[1px] h-6 md:h-8 bg-white/5"></div>
        <div class="flex flex-col items-center min-w-[50px] md:min-w-[60px]">
          <PhDatabase :size="14" class="text-tactical-cyan" />
          <span class="text-sm font-data font-black text-white leading-tight mt-1">{{ grabberStore.assets.length }}</span>
          <span class="text-[8px] font-black text-white/30 uppercase">Found</span>
        </div>
      </div>
    </header>

    <!-- Loading State -->
    <div v-if="isViewLoading" class="flex-1 flex flex-col gap-8">
      <BaseSkeleton height="120px" borderRadius="16px" />
      <div class="flex gap-8 flex-1">
        <BaseSkeleton class="hidden lg:block" width="300px" height="100%" borderRadius="16px" />
        <div class="flex-1 flex flex-col gap-4">
          <BaseSkeleton v-for="i in 5" :key="i" height="80px" borderRadius="12px" />
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div v-else class="flex-1 flex gap-10 min-h-0 overflow-hidden relative">
      <!-- Mobile Backdrop Overlay -->
      <Transition name="fade">
        <div v-if="showMobileConfig" 
             class="fixed inset-0 bg-black/60 backdrop-blur-sm z-[99] lg:hidden"
             @click="showMobileConfig = false"></div>
      </Transition>

      <!-- SIDEBAR: Control Panel (Mobile Overlay or Desktop Sidebar) -->
      <Transition name="slide-right">
        <aside 
          v-if="showMobileConfig || !isMobile"
          class="flex flex-col gap-6 overflow-y-auto pr-2 scrollbar-none transition-all duration-300"
          :class="[
            showMobileConfig 
              ? 'fixed inset-y-0 left-0 z-[100] w-[85%] max-w-[320px] p-8 bg-bg-deep border-r border-white/10 shadow-2xl lg:relative lg:inset-auto lg:p-0 lg:bg-transparent lg:w-[300px] lg:border-0 lg:shadow-none' 
              : 'hidden lg:flex lg:w-[300px] lg:shrink-0'
          ]"
        >
          <!-- Mobile Close Header -->
          <div class="flex lg:hidden justify-between items-center mb-4">
            <h3 class="text-lg font-black tracking-tighter text-white uppercase">Scan Parameters</h3>
            <button @click="showMobileConfig = false" class="p-2 text-white/40 hover:text-white"><PhGearSix :size="24" /></button>
          </div>

          <section class="glass-panel hover-glow rounded-2xl p-6 space-y-6">
            <div class="flex items-center gap-3">
              <PhGearSix :size="16" weight="duotone" class="text-tactical-cyan" />
              <h3 class="text-[10px] font-black uppercase tracking-[0.2em] text-white/60">Discovery_Params</h3>
            </div>
            
            <div class="space-y-4">
              <div class="flex justify-between items-end">
                <label class="text-[9px] font-black text-white/30 uppercase tracking-widest">Search_Depth</label>
                <span class="text-xl font-data font-black text-tactical-cyan">{{ maxDepth }}</span>
              </div>
              <input type="range" v-model.number="maxDepth" min="0" max="10" step="1" 
                     class="w-full h-1 bg-white/5 rounded-lg appearance-none cursor-pointer accent-tactical-cyan" />
            </div>

            <div class="h-[1px] w-full bg-white/5"></div>

            <div class="space-y-4">
              <label class="text-[9px] font-black text-white/30 uppercase tracking-widest block">Discovery_Scope</label>
              <div class="flex flex-col gap-2">
                <button
                  v-for="s in (['Folder', 'SameDomain', 'CrossSubdomain'] as DiscoveryScope[])"
                  :key="s"
                  class="w-full py-2 px-4 rounded-xl border text-[10px] font-black tracking-widest transition-all text-left"
                  :class="discoveryScope === s 
                    ? 'bg-tactical-cyan/10 border-tactical-cyan/30 text-tactical-cyan' 
                    : 'bg-white/5 border-white/5 text-white/40 hover:border-white/10 hover:text-white'"
                  @click="discoveryScope = s"
                >
                  {{ scopeLabel(s) }}
                </button>
              </div>
            </div>

            <div class="h-[1px] w-full bg-white/5"></div>

            <div class="space-y-4">
              <div class="flex justify-between items-end">
                <label class="text-[9px] font-black text-white/30 uppercase tracking-widest">Wildcard_Depth</label>
                <span class="text-xl font-data font-black" :class="externalLinkDepth > 0 ? 'text-hazard-orange' : 'text-white/20'">
                  {{ externalLinkDepth > 0 ? `L${externalLinkDepth}` : 'OFF' }}
                </span>
              </div>
              <input type="range" v-model.number="externalLinkDepth" min="0" max="3" step="1" 
                     class="w-full h-1 bg-white/5 rounded-lg appearance-none cursor-pointer"
                     :class="externalLinkDepth > 0 ? 'accent-hazard-orange' : 'accent-white/10'" />
              <div v-if="externalLinkDepth > 0" class="p-3 bg-hazard-orange/10 border border-hazard-orange/20 rounded-xl flex gap-3 text-[9px] font-bold text-hazard-orange leading-tight uppercase tracking-widest">
                <PhShieldWarning :size="16" weight="duotone" class="shrink-0" />
                <span>Wildcard active. High payload detected.</span>
              </div>
            </div>

            <div class="h-[1px] w-full bg-white/5"></div>

            <div class="space-y-3 pt-2">
              <BaseToggle v-model="useHeadless" label="SPA Engine (Headless)" />
              <button @click="grabberStore.assets = []" 
                      class="w-full py-2 px-4 flex items-center justify-center gap-2 text-[9px] font-black uppercase text-red-500/40 hover:text-red-500 transition-colors">
                <PhTrash :size="14" />
                Purge Discovery Store
              </button>
            </div>
          </section>

          <!-- Category Filter -->
          <section class="glass-panel hover-glow rounded-2xl p-6 space-y-6 md:mb-6">
            <div class="flex items-center gap-3">
              <PhFunnel :size="16" weight="duotone" class="text-tactical-cyan" />
              <h3 class="text-[10px] font-black uppercase tracking-[0.2em] text-white/60">Asset_Buckets</h3>
            </div>
            <div class="grid grid-cols-2 lg:grid-cols-1 gap-2">
              <button 
                v-for="cat in categories" 
                :key="cat.name"
                class="group py-2 px-3 lg:px-4 rounded-xl border flex justify-between items-center transition-all overflow-hidden"
                :class="activeCategory === cat.name 
                  ? 'bg-white/10 border-white/10' 
                  : 'bg-transparent border-transparent hover:bg-white/5'"
                @click="activeCategory = cat.name"
              >
                <div class="flex items-center gap-3">
                  <component :is="cat.icon" :size="16" weight="duotone" :class="activeCategory === cat.name ? cat.color : 'text-white/20'" />
                  <span class="text-[10px] font-black uppercase tracking-widest truncate"
                        :class="activeCategory === cat.name ? 'text-white' : 'text-white/30'">{{ cat.name }}</span>
                </div>
                <span class="text-[9px] font-data font-black px-1.5 py-0.5 rounded bg-black/40 text-tactical-cyan/40 hidden lg:inline">{{ counts[cat.name] || 0 }}</span>
              </button>
            </div>
          </section>

          <!-- Mobile Bottom Spacing/Actions -->
          <div class="lg:hidden h-20 shrink-0"></div>
          <div class="lg:hidden fixed bottom-6 left-8 right-8 z-[101]">
            <BaseButton @click="showMobileConfig = false" class="!w-full !py-4 !bg-tactical-cyan !text-black !rounded-2xl shadow-xl">APPLY_PARAMETERS</BaseButton>
          </div>
        </aside>
      </Transition>

      <!-- MAIN HUD -->
      <main class="flex-1 flex flex-col gap-6 min-w-0 h-full">
        <!-- Live Interception HUD -->
        <div v-if="grabberStore.isCrawling" 
             class="bg-tactical-cyan/10 border border-tactical-cyan/20 rounded-2xl p-4 flex items-center gap-4 animate-pulse-tactical shrink-0">
          <div class="h-2 w-2 bg-tactical-cyan rounded-full shadow-[0_0_10px_rgba(0,242,255,0.8)]"></div>
          <div class="flex-1 min-w-0">
            <span class="text-[8px] font-black text-tactical-cyan tracking-[0.3em] uppercase">Infiltrating_Frontier</span>
            <p class="text-xs font-data font-black text-white truncate">{{ grabberStore.currentUrl || 'INITIALIZING_DOWNLINK...' }}</p>
          </div>
        </div>

        <!-- Scan Input Bar -->
        <div class="flex gap-4 p-2 glass-panel rounded-2xl shrink-0 z-10 relative">
          <BaseInput
            v-model="url"
            placeholder="ENTER SOURCE URL..."
            autofocus
            @keyup.enter="handleScan"
            class="flex-1"
          >
            <template #icon-left><PhGlobe :size="20" weight="duotone" class="text-tactical-cyan/40" /></template>
          </BaseInput>

          <!-- Toggle for mobile config -->
          <button @click="showMobileConfig = !showMobileConfig" class="lg:hidden p-3 bg-white/5 border border-white/10 rounded-xl text-white/60">
            <PhGearSix :size="20" />
          </button>

          <BaseButton 
            v-if="!grabberStore.isCrawling"
            @click="handleScan" 
            :disabled="!url"
            class="!px-6 md:!px-8 !bg-tactical-cyan !text-black !rounded-xl !font-black !tracking-tighter"
          >
            <template #icon-left><PhMagnifyingGlass :size="20" weight="bold" /></template>
            <span class="hidden md:inline">START_SWEEP</span>
          </BaseButton>

          <BaseButton 
            v-else
            @click="handleStop"
            class="!px-6 md:!px-8 !bg-red-500 !text-white !rounded-xl !font-black !tracking-tighter"
          >
            <template #icon-left><PhStopCircle :size="20" weight="bold" /></template>
            <span class="hidden md:inline">HALT_SWEEP</span>
          </BaseButton>
        </div>

        <!-- Discovered Frontier -->
        <div class="flex-1 flex flex-col bg-white/5 border border-white/5 rounded-2xl overflow-hidden backdrop-blur-md min-h-0">
          <!-- Toolbar -->
          <div class="p-4 border-b border-white/5 flex flex-col md:flex-row justify-between items-center bg-black/20 gap-4">
            <div class="flex items-center gap-4 md:gap-6 w-full md:w-auto">
              <button @click="toggleSelectAll" class="flex items-center gap-3 text-[10px] font-black text-white/30 uppercase tracking-widest hover:text-white transition-colors">
                <PhCheckSquare v-if="allSelected" :size="20" class="text-tactical-cyan" weight="duotone" />
                <PhSquare v-else :size="20" weight="bold" />
                <span class="whitespace-nowrap">{{ allSelected ? 'Deselect_All' : 'Selection' }}</span>
              </button>
              <div class="w-[1px] h-4 bg-white/5"></div>
              <div class="flex items-center gap-2 text-[10px] font-black text-white/20 uppercase tracking-widest">
                <PhDatabase :size="14" />
                <span>Found: <span class="text-white/40 font-data">{{ grabberStore.assets.length }}</span></span>
              </div>
            </div>

            <BaseButton 
              size="sm" 
              :disabled="grabberStore.selectedAssets.length === 0"
              @click="handleBulkDownload"
              class="!bg-tactical-cyan !text-black !font-black !rounded-lg !px-4 !w-full md:!w-auto"
            >
              <template #icon-left><PhDownloadSimple :size="16" weight="bold" /></template>
              ACCELERATE ({{ grabberStore.selectedAssets.length }})
            </BaseButton>
          </div>

          <!-- Assets Scroller -->
          <div class="flex-1 overflow-y-auto p-4 md:p-6 scrollbar-tactical">
            <!-- Empty State -->
            <div v-if="filteredAssets.length === 0 && !grabberStore.isCrawling" 
                 class="h-full flex flex-col items-center justify-center opacity-10 gap-6">
              <PhGlobe :size="80" weight="thin" />
              <div class="text-center space-y-1">
                <p class="text-lg font-black tracking-[0.4em] uppercase">SYSTEM_READY</p>
                <p class="text-[10px] font-bold tracking-widest uppercase">Target domain discovery required</p>
              </div>
            </div>

            <!-- Asset Grid -->
            <div v-else class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3 md:gap-4">
              <article 
                v-for="item in filteredAssets" 
                :key="item.url"
                class="group p-3 md:p-4 glass-panel hover-glow rounded-2xl flex gap-3 md:gap-4 cursor-pointer relative overflow-hidden"
                :class="item.selected ? '!border-tactical-cyan/40 !bg-tactical-cyan/[0.05]' : ''"
                @click="grabberStore.toggleSelection(item.url)"
              >
                <!-- Selection Overlay -->
                <div v-if="item.selected" class="absolute top-0 right-0 p-2">
                  <PhCheckSquare :size="14" weight="duotone" class="text-tactical-cyan" />
                </div>

                <div class="shrink-0 flex items-center">
                  <div class="p-2.5 bg-black/40 border border-white/5 rounded-xl text-tactical-cyan/60 group-hover:text-tactical-cyan transition-colors"
                       :class="{ '!text-tactical-cyan': item.selected }">
                    <component :is="getCategoryIcon(item.category)" :size="20" weight="duotone" />
                  </div>
                </div>

                <div class="flex-1 min-w-0 space-y-1 py-1">
                  <h4 class="text-[10px] md:text-[11px] font-black text-white truncate uppercase tracking-tight group-hover:text-tactical-cyan transition-colors"
                      :class="{ '!text-tactical-cyan': item.selected }">
                    {{ item.filename.toUpperCase() }}
                  </h4>
                  <div class="flex items-center gap-2 text-[9px] font-bold text-white/30 uppercase tracking-widest">
                    <span class="text-tactical-cyan/60 shrink-0">{{ item.category }}</span>
                    <span class="opacity-10">|</span>
                    <span class="font-data truncate">{{ formatSize(item.size) }}</span>
                  </div>
                  <p class="text-[8px] font-data text-white/10 truncate mt-1 opacity-40">{{ item.url }}</p>
                </div>

                <button class="shrink-0 p-2 text-white/10 hover:text-red-500 transition-colors self-center bg-black/20 rounded-lg lg:opacity-0 group-hover:opacity-100"
                        @click.stop="grabberStore.assets = grabberStore.assets.filter(a => a.url !== item.url)">
                  <PhTrash :size="14" />
                </button>
              </article>
            </div>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.slide-right-enter-active, .slide-right-leave-active { transition: transform 0.4s cubic-bezier(0.165, 0.84, 0.44, 1); }
.slide-right-enter-from, .slide-right-leave-to { transform: translateX(-100%); }

.scrollbar-tactical::-webkit-scrollbar { width: 4px; }
.scrollbar-tactical::-webkit-scrollbar-track { background: transparent; }
.scrollbar-tactical::-webkit-scrollbar-thumb { background: rgba(0, 242, 255, 0.1); border-radius: 10px; }
.scrollbar-tactical::-webkit-scrollbar-thumb:hover { background: rgba(0, 242, 255, 0.3); }

@keyframes pulse-tactical {
  0%, 100% { opacity: 1; border-color: rgba(0, 242, 255, 0.2); }
  50% { opacity: 0.7; border-color: rgba(0, 242, 255, 0.6); }
}
.animate-pulse-tactical {
  animation: pulse-tactical 2s infinite ease-in-out;
}
</style>

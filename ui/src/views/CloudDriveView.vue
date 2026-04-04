<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { 
  PhFolder, 
  PhFile, 
  PhArrowsClockwise, 
  PhDownloadSimple, 
  PhHardDrive, 
  PhShieldWarning, 
  PhClock 
} from "@phosphor-icons/vue";
import BaseButton from '@/features/shared/components/BaseButton.vue';
import BaseSkeleton from '@/features/shared/components/BaseSkeleton.vue';
import BaseBreadcrumb from '@/features/shared/components/BaseBreadcrumb.vue';
import StatusBadge from '@/features/shared/components/StatusBadge.vue';
import { animate, stagger } from 'motion';

const isLoading = ref(true);
const isSyncing = ref(false);
const driveFiles = ref<any[]>([]);
const isConnected = ref(false); 
const lastSynced = ref<string | null>(null);
const currentPath = ref<{ label: string, id: string }[]>([
  { label: 'My Drive', id: 'root' }
]);
const fileListRef = ref<HTMLElement | null>(null);

onMounted(() => {
  setTimeout(() => {
    fetchDriveFiles();
  }, 1000);
});

const fetchDriveFiles = async () => {
  isLoading.value = true;
  try {
    await new Promise(resolve => setTimeout(resolve, 1500));

    if (!isConnected.value) {
      driveFiles.value = [];
      return;
    }

    driveFiles.value = [
      { id: '1', name: 'Project Assets', type: 'folder', size: '--', modified: '2024-03-28' },
      { id: '2', name: 'Raw_Footage_001.mp4', type: 'file', size: '1.2 GB', modified: '2024-03-29' },
      { id: '3', name: 'Backup_Registry.zip', type: 'file', size: '45.8 MB', modified: '2024-03-30' },
      { id: '4', name: 'Technical_Documentation.pdf', type: 'file', size: '2.1 MB', modified: '2024-03-31' }
    ];
  } catch (error) {
    console.error('Error fetching drive files:', error);
  } finally {
    isLoading.value = false;
    
    setTimeout(() => {
      if (fileListRef.value && driveFiles.value.length > 0) {
        animate(
          '.file-item-tactical',
          { opacity: [0, 1], x: [-10, 0] },
          { delay: stagger(0.05), type: 'spring', stiffness: 400, damping: 30 }
        );
      }
    }, 50);
  }
};

const handleNavigate = (item: any) => {
  if (item.id === 'root') {
    currentPath.value = [{ label: 'My Drive', id: 'root' }];
  } else {
    if (!currentPath.value.find(p => p.id === item.id)) {
      currentPath.value.push({ label: item.name || item.label, id: item.id });
    }
  }
  fetchDriveFiles();
};

const toggleConnection = () => {
  isConnected.value = !isConnected.value;
  fetchDriveFiles();
};

const refreshDrive = async () => {
  isSyncing.value = true;
  try {
    const timestamp = await invoke('sync_metadata');
    lastSynced.value = new Date(Number(timestamp) * 1000).toLocaleTimeString();
    await fetchDriveFiles();
  } catch (e) {
    console.error('Sync failed:', e);
  } finally {
    isSyncing.value = false;
  }
};
</script>

<template>
  <div class="h-full flex flex-col p-8 md:p-12 gap-8 overflow-y-auto select-none">
    <header class="flex justify-between items-end shrink-0">
      <div class="space-y-1">
        <h2 class="text-2xl md:text-3xl font-black uppercase tracking-tighter text-white">Cloud Drive</h2>
        <div class="flex flex-col gap-2">
          <p class="text-[10px] font-bold text-white/40 tracking-widest uppercase">Direct REST integration with your Google Drive storage</p>
          <div v-if="lastSynced" class="flex items-center gap-2 text-[9px] font-data font-black text-tactical-cyan uppercase tracking-widest">
            <PhClock :size="12" weight="bold" />
            <span>Last Synced: {{ lastSynced }}</span>
          </div>
        </div>
      </div>
      <div class="flex items-center gap-4">
        <BaseButton @click="refreshDrive" :disabled="!isConnected || isSyncing" 
                    class="!bg-white/5 !text-white/60 !border border-white/10 !px-6 !rounded-xl transition-all">
          <template #icon-left><PhArrowsClockwise :size="16" weight="duotone" :class="{ 'animate-spin': isSyncing }" /></template>
          {{ isSyncing ? 'SYNCING...' : 'SYNC NOW' }}
        </BaseButton>
        <BaseButton @click="toggleConnection" 
                    class="!px-6 !rounded-xl !font-black !tracking-tighter !transition-all active:scale-95"
                    :class="isConnected ? '!bg-white/5 !text-white/60' : '!bg-tactical-cyan !text-black'">
          {{ isConnected ? 'ABORT CONNECTION' : 'CONNECT ACCOUNT' }}
        </BaseButton>
      </div>
    </header>

    <div v-if="isConnected" class="bg-white/5 border border-white/5 rounded-2xl px-4 py-2 backdrop-blur-md">
      <BaseBreadcrumb 
        :items="currentPath" 
        @navigate="handleNavigate"
      />
    </div>

    <!-- Managed Skeleton Loaders -->
    <div v-if="isLoading" class="flex-1 space-y-4">
      <div v-for="i in 6" :key="i" class="bg-white/5 border border-white/5 rounded-2xl p-4 flex items-center gap-4">
        <BaseSkeleton width="48px" height="48px" borderRadius="12px" />
        <div class="flex-1 space-y-2">
          <BaseSkeleton width="40%" height="16px" />
          <BaseSkeleton width="20%" height="10px" />
        </div>
        <BaseSkeleton width="100px" height="36px" borderRadius="10px" />
      </div>
    </div>

    <!-- Active Content -->
    <div v-else class="flex-1 min-h-0 flex flex-col">
      <!-- Unconnected State -->
      <div v-if="!isConnected" class="flex-1 flex flex-col items-center justify-center border-2 border-dashed border-white/5 rounded-3xl opacity-20 gap-8 py-20">
        <div class="relative">
          <div class="absolute inset-0 bg-hazard-orange/20 blur-3xl rounded-full"></div>
          <PhShieldWarning :size="80" weight="thin" class="text-hazard-orange relative z-10" />
        </div>
        <div class="text-center space-y-4 max-w-sm">
          <h3 class="text-xl font-black uppercase text-white tracking-widest">Authentication Required</h3>
          <p class="text-[10px] font-bold text-white/60 tracking-widest uppercase leading-relaxed text-center px-6">Authorize Mdownloader to access your cloud resources securely via OAuth 2.0 protocol sweep.</p>
          <BaseButton 
            @click="toggleConnection"
            class="!bg-tactical-cyan !text-black !font-black !px-10 !py-4 !rounded-2xl !mt-6 shadow-[0_0_20px_rgba(0,242,255,0.2)] hover:shadow-[0_0_30px_rgba(0,242,255,0.4)] transition-all"
          >
            INITIALIZE OAUTH FLOW
          </BaseButton>
        </div>
      </div>

      <!-- Connected but Empty -->
      <div v-else-if="driveFiles.length === 0" class="flex-1 flex flex-col items-center justify-center opacity-10 gap-6 border-2 border-dashed border-white/5 rounded-3xl py-20">
        <PhHardDrive :size="100" weight="thin" />
        <div class="text-center space-y-2">
          <p class="text-lg font-black tracking-[0.4em] uppercase">No Assets Detected</p>
          <p class="text-[10px] font-bold tracking-widest uppercase">Your storage appears to be empty or restricted</p>
        </div>
      </div>

      <!-- File List -->
      <div v-else class="space-y-3 pb-20" ref="fileListRef">
        <div v-for="file in driveFiles" :key="file.id" 
             class="file-item-tactical group bg-white/5 border border-white/5 rounded-2xl p-4 pr-6 flex items-center gap-4 hover:bg-white/[0.08] transition-all backdrop-blur-md">
          
          <div class="w-12 h-12 bg-black/40 border border-white/5 rounded-xl flex items-center justify-center shrink-0">
            <PhFolder v-if="file.type === 'folder'" :size="24" weight="duotone" class="text-tactical-cyan/60" />
            <PhFile v-else :size="24" weight="duotone" class="text-white/20" />
          </div>

          <div class="flex-1 min-w-0">
            <h4 class="text-xs font-black text-white uppercase tracking-tight truncate group-hover:text-tactical-cyan transition-colors">{{ file.name }}</h4>
            <p class="text-[10px] font-data font-black text-white/20 uppercase tracking-widest mt-0.5">
              {{ file.size }} <span class="opacity-30 mx-1">|</span> Modified: {{ file.modified }}
            </p>
          </div>

          <div class="flex items-center gap-3">
            <StatusBadge v-if="file.type === 'folder'" status="Queued" text="NAVIGATE" @click="handleNavigate(file)" class="cursor-pointer hover:scale-105 active:scale-95" />
            <BaseButton v-else class="!bg-white/5 !text-white/60 !border border-white/10 !px-4 !py-2 !rounded-xl !text-[11px] !font-black hover:!bg-tactical-cyan hover:!text-black transition-all">
              <template #icon-left><PhDownloadSimple :size="16" weight="bold" /></template>
              CAPTURE
            </BaseButton>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

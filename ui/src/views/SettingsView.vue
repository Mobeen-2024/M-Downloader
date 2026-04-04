<script setup lang="ts">
import { useSettingsStore } from '@/stores/settings.store';
import { useUIStore } from '@/stores/ui.store';
import { animate } from 'motion';
import {
  PhCpu,
  PhFolder,
  PhPaintBrush,
  PhActivity,
  PhCloud,
  PhLightning,
  PhInfo,
  PhShieldWarning
} from "@phosphor-icons/vue";
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import BaseInput from '@/features/shared/components/BaseInput.vue';
import BaseToggle from '@/features/shared/components/BaseToggle.vue';

const settings = useSettingsStore();
const uiStore = useUIStore();

const themes = [
  { name: 'TACTICAL_CYAN', value: 'blue', color: '#00f2ff' },
  { name: 'HAZARD_ORANGE', value: 'purple', color: '#ff8800' },
  { name: 'STEALTH_GRAY', value: 'green', color: '#94a3b8' },
];

const snifferActive = ref(false);
const snifferError = ref<string | null>(null);

const handleThemeChange = (theme: any, event: MouseEvent) => {
  settings.themeAccent = theme.value;
  uiStore.success(`Accent synchronization complete: ${theme.name}`);
  
  const target = event.currentTarget as HTMLElement;
  (animate as any)(target, 
    { scale: [1, 1.05, 1] },
    { type: 'spring', stiffness: 500, damping: 15 }
  );
};

const toggleSniffer = async () => {
  try {
    snifferError.value = null;
    snifferActive.value = await invoke('toggle_system_sniffing', { enabled: !snifferActive.value });
  } catch (e: any) {
    snifferError.value = e.toString();
  }
};

const newExt = ref('');
const addExt = () => {
  if (newExt.value && !settings.snifferFilter.includes(newExt.value.toLowerCase())) {
    settings.snifferFilter.push(newExt.value.toLowerCase().replace('.', ''));
    newExt.value = '';
  }
};
const removeExt = (ext: string) => {
  settings.snifferFilter = settings.snifferFilter.filter(e => e !== ext);
};

const wipeSystemVault = async () => {
  if (confirm("HIGH_RISK_ACTION_IDENTIFIED: This will IRREVERSIBLY purge all site credentials and cloud tokens from the OS vault. Continue?")) {
    try {
      await invoke('clear_all_keychain_vaults');
      uiStore.success("System vault purge successful. Identity neutralized.");
    } catch (e) {
      uiStore.error("VAULT_PURGE_FAILURE: Check system permissions.");
    }
  }
};

onMounted(async () => {
  try {
    snifferActive.value = await invoke('get_sniffer_status');
  } catch (e) {
    console.error('Failed to get sniffer status:', e);
  }
});
</script>

<template>
  <div class="h-full flex flex-col p-8 md:p-12 gap-10 overflow-y-auto select-none">
    <!-- Header -->
    <header class="flex justify-between items-end shrink-0">
      <div class="space-y-1">
        <h2 class="text-2xl md:text-3xl font-black uppercase tracking-tighter text-white">Control Center</h2>
        <p class="text-[10px] font-bold text-white/40 tracking-widest uppercase">Customize your industrial acceleration engine and system orchestration.</p>
      </div>
      <div class="px-3 py-1 bg-white/5 border border-white/5 rounded flex items-center gap-2">
        <span class="text-[9px] font-data font-black text-tactical-cyan/60 uppercase">PROTO_BUILD v2.1.0-SPEC</span>
      </div>
    </header>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 pb-20">
      <!-- Performance Section -->
      <section class="bg-white/5 border border-white/5 rounded-2xl p-6 md:p-8 space-y-8 backdrop-blur-md">
        <div class="flex items-center gap-3">
          <PhCpu :size="20" weight="duotone" class="text-tactical-cyan" />
          <h3 class="text-xs font-black uppercase tracking-widest text-white">Engine Performance</h3>
        </div>

        <div class="space-y-6">
          <div class="space-y-4">
            <div class="flex justify-between items-end gap-4">
              <div class="space-y-1 min-w-0 flex-1">
                <span class="text-[11px] font-black uppercase text-white/80 block truncate">Segment Cluster Limit</span>
                <p class="text-[10px] text-white/30 font-medium leading-relaxed">Maximum concurrent connections per transmission unit.</p>
              </div>
              <span class="text-lg font-data font-black text-tactical-cyan shrink-0">{{ settings.maxConnections }}</span>
            </div>
            <input type="range" v-model.number="settings.maxConnections" min="1" max="32" step="1" 
                   class="w-full h-1 bg-white/5 rounded-lg appearance-none cursor-pointer accent-tactical-cyan" />
          </div>

          <div class="h-[1px] w-full bg-white/5"></div>

          <div class="space-y-4">
            <div class="flex justify-between items-center gap-4">
              <div class="space-y-1 min-w-0 flex-1">
                <span class="text-[11px] font-black uppercase text-white/80 block truncate">Throttle System</span>
                <p class="text-[10px] text-white/30 font-medium leading-relaxed">Enable Token Bucket rate-limiting for all active worker threads.</p>
              </div>
              <BaseToggle v-model="settings.enableSpeedLimit" class="shrink-0" />
            </div>

            <Transition name="expand">
              <div v-if="settings.enableSpeedLimit" class="p-4 bg-black/40 border border-white/5 rounded-xl space-y-4">
                <div class="flex justify-between items-end">
                  <span class="text-[10px] font-black uppercase text-white/60">Bandwidth Cap</span>
                  <span class="text-sm font-data font-black text-tactical-cyan">{{ (settings.maxDownloadSpeed / 1024).toFixed(1) }} MB/S</span>
                </div>
                <input type="range" v-model.number="settings.maxDownloadSpeed" min="100" max="102400" step="100" 
                       class="w-full h-1 bg-white/10 rounded-lg appearance-none cursor-pointer accent-tactical-cyan" />
              </div>
            </Transition>
          </div>
        </div>
      </section>

      <!-- Networking Section -->
      <section class="bg-white/5 border border-white/5 rounded-2xl p-6 md:p-8 space-y-8 backdrop-blur-md">
        <div class="flex items-center gap-3">
          <PhActivity :size="20" weight="duotone" class="text-tactical-cyan" />
          <h3 class="text-xs font-black uppercase tracking-widest text-white">Networking & Sniffing</h3>
        </div>

        <div class="space-y-6">
          <div class="space-y-4">
            <div class="flex justify-between items-center gap-4">
              <div class="space-y-1 min-w-0 flex-1">
                <span class="text-[11px] font-black uppercase text-white/80 block truncate">Interception Bridge (WFP)</span>
                <p class="text-[10px] text-white/30 font-medium leading-relaxed">Kernel-mode driver for non-browser media interception. Requires elevated rights.</p>
              </div>
              <BaseToggle :model-value="snifferActive" @update:model-value="toggleSniffer" class="shrink-0" />
            </div>

            <Transition name="expand">
              <div v-if="snifferActive" class="space-y-3">
                <span class="text-[9px] font-black uppercase text-white/20 tracking-widest">Target File Signatures</span>
                <div class="flex flex-wrap gap-2">
                  <div v-for="ext in settings.snifferFilter" :key="ext" 
                       class="flex items-center gap-2 px-2 py-1 bg-tactical-cyan/10 border border-tactical-cyan/20 rounded text-[9px] font-data font-black text-tactical-cyan uppercase">
                    {{ ext }}
                    <button @click="removeExt(ext)" class="hover:text-white transition-colors">&times;</button>
                  </div>
                  <div class="px-2 py-1 bg-white/5 border border-white/5 rounded">
                    <input v-model="newExt" placeholder="+" @keyup.enter="addExt" 
                           class="bg-transparent border-none text-white w-8 font-data text-[10px] outline-none" />
                  </div>
                </div>
              </div>
            </Transition>

            <div v-if="snifferError" class="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-[10px] font-data text-red-500">
              FAULT_DETECTED: {{ snifferError }}
            </div>
          </div>

          <div class="h-[1px] w-full bg-white/5"></div>

          <div class="flex justify-between items-center gap-4">
            <div class="space-y-1 min-w-0 flex-1">
              <span class="text-[11px] font-black uppercase text-white/80 block truncate">Bio-Monitor (Clipboard)</span>
              <p class="text-[10px] text-white/30 font-medium leading-relaxed">Automatically detect transmission links in the system buffer.</p>
            </div>
            <BaseToggle v-model="settings.monitorClipboard" class="shrink-0" />
          </div>
        </div>
      </section>

      <!-- Identity & Aesthetic -->
      <section class="bg-white/5 border border-white/5 rounded-2xl p-6 md:p-8 space-y-8 backdrop-blur-md">
        <div class="flex items-center gap-3">
          <PhPaintBrush :size="20" weight="duotone" class="text-tactical-cyan" />
          <h3 class="text-xs font-black uppercase tracking-widest text-white">System Identity</h3>
        </div>

        <div class="space-y-6">
          <div class="space-y-4">
            <span class="text-[11px] font-black uppercase text-white/80">Interface Accent Signature</span>
            <div class="grid grid-cols-3 gap-2 md:gap-4">
              <button
                v-for="theme in themes"
                :key="theme.value"
                class="w-full p-2 md:p-3 rounded-xl border transition-all flex flex-col items-center gap-2 group overflow-hidden"
                :class="settings.themeAccent === theme.value 
                  ? 'bg-tactical-cyan/10 border-tactical-cyan/30' 
                  : 'bg-white/5 border-white/5 hover:border-white/10'"
                @click="handleThemeChange(theme, $event)"
              >
                <div class="w-4 h-4 rounded-full shadow-[0_0_10px_var(--orb-color)] shrink-0" 
                     :style="{ backgroundColor: theme.color, '--orb-color': theme.color }"></div>
                <span class="text-[8px] md:text-[9px] font-data font-black text-white/40 group-hover:text-white transition-colors truncate w-full text-center">{{ theme.name }}</span>
              </button>
            </div>
          </div>

          <div class="h-[1px] w-full bg-white/5"></div>

          <div class="flex justify-between items-center gap-4">
            <div class="space-y-1 min-w-0 flex-1">
              <span class="text-[11px] font-black uppercase text-white/80 block truncate">Extension Bridge</span>
              <p class="text-[10px] text-white/30 font-medium leading-relaxed">Maintain a persistent IPC pipe for seamless browser integration.</p>
            </div>
            <BaseToggle v-model="settings.bridgeEnabled" class="shrink-0" />
          </div>
        </div>
      </section>

      <!-- Cloud Infrastructure -->
      <section class="bg-white/5 border border-white/5 rounded-2xl p-6 md:p-8 space-y-8 backdrop-blur-md">
        <div class="flex items-center gap-3">
          <PhCloud :size="20" weight="duotone" class="text-tactical-cyan" />
          <h3 class="text-xs font-black uppercase tracking-widest text-white">Cloud Ecosystem</h3>
        </div>

        <div class="space-y-6">
          <div class="space-y-4">
            <div class="flex justify-between items-center gap-4">
              <div class="space-y-1 min-w-0 flex-1">
                <span class="text-[11px] font-black uppercase text-white/80 block truncate">Direct Signal Sync</span>
                <p class="text-[10px] text-white/30 font-medium leading-relaxed">Stream verified payloads directly to remote storage providers.</p>
              </div>
              <BaseToggle v-model="settings.cloudConfig.enabled" class="shrink-0" />
            </div>

            <Transition name="expand">
              <div v-if="settings.cloudConfig.enabled" class="p-5 bg-black/40 border border-white/5 rounded-xl space-y-4">
                <div class="space-y-2">
                  <label class="text-[9px] font-black uppercase text-white/30 tracking-widest">Service Provider</label>
                  <BaseInput v-model="settings.cloudConfig.provider" placeholder="GOOGLE DRIVE, DROPBOX, ETC." />
                </div>
                <div class="space-y-2">
                  <label class="text-[9px] font-black uppercase text-white/30 tracking-widest">Integration Key (Bearer)</label>
                  <BaseInput v-model="settings.cloudApiKey" type="password" placeholder="••••••••••••••••" />
                </div>
              </div>
            </Transition>
          </div>

          <div class="h-[1px] w-full bg-white/5"></div>

          <div class="space-y-3">
            <span class="text-[11px] font-black uppercase text-white/80">Storage Root Path</span>
            <div class="p-3 bg-black/40 border border-white/5 rounded-xl flex items-center gap-3">
              <PhFolder :size="16" weight="duotone" class="text-tactical-cyan/60" />
              <span class="text-[11px] font-data text-white/60 truncate">{{ settings.downloadDir }}</span>
            </div>
            <p class="text-[9px] font-black text-red-500/40 uppercase tracking-widest">Configured_in_Env: Relocation_Denied</p>
          </div>
        </div>
      </section>

      <!-- Roadmap -->
      <section class="lg:col-span-2 bg-tactical-cyan/[0.02] border border-tactical-cyan/10 rounded-2xl p-6 md:p-8 space-y-8 backdrop-blur-md">
        <div class="flex items-center gap-3">
          <PhLightning :size="20" weight="duotone" class="text-tactical-cyan" />
          <h3 class="text-xs font-black uppercase tracking-widest text-white">Innovation Pipeline</h3>
        </div>
        
        <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
          <div class="space-y-2 relative">
            <span class="text-3xl font-data font-black text-tactical-cyan/10 absolute -top-4 -left-2">01</span>
            <p class="text-xs font-black text-white uppercase tracking-tight">QUIC Transport</p>
            <p class="text-[10px] text-white/30 leading-relaxed">Eliminating head-of-line blocking for multi-threaded stream parity.</p>
          </div>
          <div class="space-y-2 relative">
            <span class="text-3xl font-data font-black text-tactical-cyan/10 absolute -top-4 -left-2">02</span>
            <p class="text-xs font-black text-white uppercase tracking-tight">Cloud Direct-Memory</p>
            <p class="text-[10px] text-white/30 leading-relaxed">Direct-to-Cloud parallel fetching without locally caching large buffers.</p>
          </div>
          <div class="space-y-2 relative opacity-40">
            <span class="text-3xl font-data font-black text-tactical-cyan/10 absolute -top-4 -left-2">03</span>
            <p class="text-xs font-black text-white uppercase tracking-tight">WFP Deobfuscation</p>
            <p class="text-[10px] text-white/30 leading-relaxed">Intercepting adaptive stream fragments before they reach browser runtime.</p>
          </div>
        </div>
      </section>

      <!-- Advanced Security & Privacy -->
      <section class="lg:col-span-2 bg-red-500/[0.02] border border-red-500/10 rounded-2xl p-6 md:p-8 space-y-8 backdrop-blur-md">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <PhShieldWarning :size="20" weight="duotone" class="text-red-500" />
            <h3 class="text-xs font-black uppercase tracking-widest text-white">Security & Privacy (Danger Zone)</h3>
          </div>
          <span class="text-[9px] font-black text-red-500/40 uppercase tracking-[0.3em]">ADMIN_ACCESS_ONLY</span>
        </div>

        <div class="flex flex-col md:flex-row md:items-center justify-between gap-6 p-6 bg-red-500/5 border border-red-500/10 rounded-2xl">
          <div class="space-y-1">
            <p class="text-[11px] font-black text-white uppercase">Wipe OS-Level System Vault</p>
            <p class="text-[10px] text-white/40 leading-relaxed uppercase">Immediately purge all stored credentials, site passwords, and cloud tokens from the OS Credential Manager.</p>
          </div>
          <button 
            @click="wipeSystemVault"
            class="px-8 py-3 bg-red-500 text-white text-[10px] font-black uppercase tracking-widest rounded-xl hover:bg-red-600 transition-all shadow-[0_0_20px_rgba(239,68,68,0.2)] active:scale-95"
          >
            Purge System Vault
          </button>
        </div>
      </section>
    </div>

    <!-- Footer -->
    <footer class="shrink-0 pt-8 pb-12 mt-auto">
      <div class="p-4 bg-white/5 border border-white/5 rounded-xl flex items-start gap-4">
        <PhInfo :size="18" weight="duotone" class="text-tactical-cyan mt-0.5" />
        <p class="text-[11px] text-text-dim leading-relaxed">
          MDOWNLOADER represents a significant engineering effort in concurrent networking and kernel-level interception. Systems operational. <strong class="text-white">NOMINAL STATUS.</strong>
        </p>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.expand-enter-active, .expand-leave-active { transition: all 0.3s cubic-bezier(0.19, 1, 0.22, 1); max-height: 500px; overflow: hidden; }
.expand-enter-from, .expand-leave-to { opacity: 0; max-height: 0; transform: translateY(-10px); }
</style>

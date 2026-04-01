<script setup lang="ts">
import { useSettingsStore } from '@/stores/settings.store';
import { useUIStore } from '@/stores/ui.store';
import { animate, spring } from 'motion';
import {
  Cpu,
  Folder,
  Palette,
  Activity,
  Cloud,
  Zap,
  Info
} from 'lucide-vue-next';
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import BaseCard from '@/features/shared/components/BaseCard.vue';
import BaseInput from '@/features/shared/components/BaseInput.vue';
import BaseToggle from '@/features/shared/components/BaseToggle.vue';

const settings = useSettingsStore();
const uiStore = useUIStore();

const themes = [
  { name: 'Classic Blue', value: 'blue', color: '#3b82f6' },
  { name: 'Hyper Purple', value: 'purple', color: '#8b5cf6' },
  { name: 'Emerald Green', value: 'green', color: '#10b981' },
];

const snifferActive = ref(false);
const snifferError = ref<string | null>(null);

const handleThemeChange = (theme: any, event: MouseEvent) => {
  settings.themeAccent = theme.value;
  uiStore.success(`Accent synchronization complete: ${theme.name}`);
  
  // Phase 6: Bouncy orbital transition
  const target = event.currentTarget as HTMLElement;
  (animate as any)(target, 
    { scale: [1, 1.2, 1] },
    { easing: spring({ stiffness: 500, damping: 15 }) }
  );
};

const toggleSniffer = async () => {
  try {
    snifferError.value = null;
    snifferActive.value = await invoke('toggle_system_sniffing', { enabled: !snifferActive.value });
  } catch (e: any) {
    snifferError.value = e.toString();
    console.error('Sniffer Error:', e);
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

onMounted(async () => {
  try {
    snifferActive.value = await invoke('get_sniffer_status');
  } catch (e) {
    console.error('Failed to get sniffer status:', e);
  }
});
</script>

<template>
  <div class="settings-view">
    <header class="view-header">
      <div class="header-left">
        <h2 class="view-title">Control Center</h2>
        <p class="view-subtitle">Customize your industrial acceleration engine and system orchestration.</p>
      </div>
      <div class="header-right">
        <div class="version-badge">Mdownloader Pro v0.2.0</div>
      </div>
    </header>

    <div class="settings-grid">
      <!-- Performance Section -->
      <BaseCard variant="glass" padding="lg" class="settings-card">
        <div class="card-title">
          <Cpu :size="18" class="text-accent" />
          <h3>Engine Performance</h3>
        </div>

        <div class="setting-block">
          <div class="block-header">
            <span class="label">Byte-Range Segment Limit</span>
            <span class="value">{{ settings.maxConnections }} Units</span>
          </div>
          <div class="range-wrapper">
            <input type="range" v-model.number="settings.maxConnections" min="1" max="32" step="1" class="pro-range" />
          </div>
          <p class="helper">Maximum concurrent connections per transmission. Industrial standard is 8-16.</p>
        </div>

        <div class="divider"></div>

        <div class="setting-block">
          <div class="block-row">
            <div class="text-group">
              <span class="label">Dynamic Speed Limit</span>
              <p class="helper">Enable Token Bucket rate-limiting for all active worker threads.</p>
            </div>
            <BaseToggle v-model="settings.enableSpeedLimit" />
          </div>

          <Transition name="expand">
            <div v-if="settings.enableSpeedLimit" class="limit-controls">
              <div class="block-header">
                <span class="label">Bandwidth Cap</span>
                <span class="value">{{ (settings.maxDownloadSpeed / 1024).toFixed(1) }} MB/s</span>
              </div>
              <input type="range" v-model.number="settings.maxDownloadSpeed" min="100" max="102400" step="100" class="pro-range" />
            </div>
          </Transition>
        </div>
      </BaseCard>

      <!-- Networking Section -->
      <BaseCard variant="glass" padding="lg" class="settings-card">
        <div class="card-title">
          <Activity :size="18" class="text-accent" />
          <h3>Networking & Sniffing</h3>
        </div>

        <div class="setting-block">
          <div class="block-row">
            <div class="text-group">
              <span class="label">Interception Bridge (WFP)</span>
              <p class="helper">Kernel-mode driver for non-browser media interception.</p>
            </div>
            <BaseToggle :model-value="snifferActive" @update:model-value="toggleSniffer" />
          </div>

          <Transition name="expand">
            <div v-if="snifferActive" class="sniffer-rules">
              <span class="sub-label">Target File Extensions</span>
              <div class="tag-field">
                <div v-for="ext in settings.snifferFilter" :key="ext" class="pro-tag">
                  .{{ ext }}
                  <button @click="removeExt(ext)" class="tag-remove">&times;</button>
                </div>
                <div class="add-tag">
                  <input v-model="newExt" placeholder="ext..." @keyup.enter="addExt" />
                </div>
              </div>
            </div>
          </Transition>

          <div v-if="snifferError" class="pro-alert error">
            {{ snifferError }}
          </div>
        </div>

        <div class="divider"></div>

        <div class="setting-block">
          <div class="block-row">
            <div class="text-group">
              <span class="label">Clipboard Sentinel</span>
              <p class="helper">Automatically detect transmission links in the system buffer.</p>
            </div>
            <BaseToggle v-model="settings.monitorClipboard" />
          </div>
        </div>
      </BaseCard>

      <!-- Cloud/Storage → This section was previously complex, refactoring carefully -->
      <BaseCard variant="glass" padding="lg" class="settings-card">
        <div class="card-title">
          <Cloud :size="18" class="text-accent" />
          <h3>Cloud Ecosystem</h3>
        </div>

        <div class="setting-block">
          <div class="block-row">
            <div class="text-group">
              <span class="label">Direct Cloud Sync</span>
              <p class="helper">Stream verified payloads directly to your remote storage providers.</p>
            </div>
            <BaseToggle v-model="settings.cloudConfig.enabled" />
          </div>

          <Transition name="expand">
            <div v-if="settings.cloudConfig.enabled" class="cloud-form">
              <div class="input-stack">
                <div class="i-field">
                  <label>Service Provider</label>
                  <BaseInput v-model="settings.cloudConfig.provider" placeholder="Google Drive, Dropbox, etc." />
                </div>
                <div class="i-field">
                  <label>Integration Key (Bearer)</label>
                  <BaseInput v-model="settings.cloudApiKey" type="password" placeholder="••••••••••••••••" />
                </div>
                <div class="i-field">
                  <label>Vault Fragment (Folder ID)</label>
                  <BaseInput v-model="settings.cloudConfig.target_folder_id" placeholder="Folder hash identifier..." />
                </div>
              </div>
            </div>
          </Transition>
        </div>

        <div class="divider"></div>

        <div class="setting-block">
          <span class="label">Storage Root</span>
          <div class="storage-path">
            <Folder :size="14" />
            <span>{{ settings.downloadDir }}</span>
          </div>
          <p class="helper">Configured in system environment. Relocation requires administrative rights.</p>
        </div>
      </BaseCard>

      <!-- Visuals & Identity -->
      <BaseCard variant="glass" padding="lg" class="settings-card">
        <div class="card-title">
          <Palette :size="18" class="text-accent" />
          <h3>Identity & Aesthetic</h3>
        </div>

        <div class="setting-block">
          <span class="label">Interface Accent</span>
          <div class="theme-grid">
            <button
              v-for="theme in themes"
              :key="theme.value"
              class="theme-orb"
              :class="{ 'is-active': settings.themeAccent === theme.value }"
              @click="handleThemeChange(theme, $event)"
              :style="{ '--orb-color': theme.color }"
              :title="theme.name"
            >
              <div class="orb-inner"></div>
              <span class="orb-name">{{ theme.name }}</span>
            </button>
          </div>
        </div>

        <div class="divider"></div>

        <div class="setting-block">
          <div class="block-row">
            <div class="text-group">
              <span class="label">Chrome Extension Bridge</span>
              <p class="helper">Maintain a persistent IPC pipe for seamless browser integration.</p>
            </div>
            <BaseToggle v-model="settings.bridgeEnabled" />
          </div>
        </div>
      </BaseCard>

      <!-- Roadmap Footer (Refined) -->
      <BaseCard variant="glass" padding="lg" class="roadmap-card full-width shimmer-effect">
        <div class="card-title">
          <Zap :size="18" class="text-accent" />
          <h3>Innovation Pipeline</h3>
        </div>
        <div class="roadmap-flex">
          <div class="r-item active">
            <div class="r-idx">01</div>
            <div>
              <p class="r-title">QUIC Transport Protocol</p>
              <p class="r-desc">Eliminating head-of-line blocking for multi-threaded stream parity.</p>
            </div>
          </div>
          <div class="r-item active">
            <div class="r-idx">02</div>
            <div>
              <p class="r-title">Cloud Direct-Memory</p>
              <p class="r-desc">Direct-to-Cloud parallel fetching without locally caching large buffers.</p>
            </div>
          </div>
          <div class="r-item pending">
            <div class="r-idx">03</div>
            <div>
              <p class="r-title">WFP Deobfuscation</p>
              <p class="r-desc">Intercepting adaptive stream fragments before they reach the browser runtime.</p>
            </div>
          </div>
        </div>
      </BaseCard>
    </div>

    <footer class="view-footer">
      <div class="info-banner">
        <Info :size="14" />
        <p>Building <strong>Mdownloader</strong> represents a significant engineering effort in concurrent networking and kernel-level interception. We appreciate your use of this high-performance suite.</p>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.settings-view {
  height: 100%;
  padding: 32px 40px;
  display: flex;
  flex-direction: column;
  gap: 32px;
  overflow-y: auto;
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.view-title {
  font-size: 1.75rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.view-subtitle {
  font-size: 0.9rem;
  color: var(--text-secondary);
}

.version-badge {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-color);
  padding: 8px 16px;
  border-radius: 20px;
  font-size: 0.7rem;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-secondary);
  letter-spacing: 0.05em;
}

/* Grid Layout */
.settings-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 24px;
}

.full-width {
  grid-column: span 2;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
}

.card-title h3 {
  font-size: 1rem;
  font-weight: 800;
  color: var(--text-primary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.setting-block {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.block-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.block-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.text-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.label {
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--text-primary);
}

.sub-label {
  font-size: 0.65rem;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-secondary);
  opacity: 0.6;
  margin-bottom: 8px;
  display: block;
}

.value {
  font-family: var(--font-mono);
  font-weight: 800;
  color: var(--accent-primary);
  font-size: 0.9rem;
}

.helper {
  font-size: 0.75rem;
  color: var(--text-secondary);
  line-height: 1.4;
}

.divider {
  height: 1px;
  background: var(--border-color);
  margin: 12px 0;
  opacity: 0.5;
}

.shimmer-effect {
  position: relative;
  overflow: hidden;
}

.shimmer-effect::after {
  content: "";
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: linear-gradient(
    45deg,
    transparent 45%,
    rgba(255, 255, 255, 0.03) 50%,
    transparent 55%
  );
  animation: roadmap-shimmer 6s infinite linear;
  pointer-events: none;
}

@keyframes roadmap-shimmer {
  0% { transform: translate(-30%, -30%); }
  100% { transform: translate(30%, 30%); }
}

/* Range Input Styling */
.pro-range {
  width: 100%;
  accent-color: var(--accent-primary);
  background: rgba(255, 255, 255, 0.05);
  height: 6px;
  border-radius: 3px;
}

.limit-controls {
  margin-top: 16px;
  padding: 16px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 12px;
  border: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* Tag Management */
.tag-field {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  background: rgba(255, 255, 255, 0.02);
  padding: 12px;
  border-radius: 12px;
  border: 1px solid var(--border-color);
}

.pro-tag {
  background: var(--accent-primary);
  color: white;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 8px;
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
}

.tag-remove {
  border: none;
  background: rgba(0, 0, 0, 0.2);
  color: white;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  font-size: 10px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.add-tag input {
  background: transparent;
  border: none;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-primary);
  font-size: 0.8rem;
  width: 60px;
  outline: none;
  padding: 2px 4px;
}

/* Cloud Form */
.input-stack {
  margin-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  background: rgba(0, 0, 0, 0.2);
  padding: 20px;
  border-radius: 16px;
  border: 1px solid var(--border-color);
}

.i-field label {
  font-size: 0.65rem;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--text-secondary);
  margin-bottom: 8px;
  display: block;
}

/* Storage Path Display */
.storage-path {
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(255, 255, 255, 0.03);
  padding: 12px 16px;
  border-radius: 12px;
  border: 1px solid var(--border-color);
  font-family: var(--font-mono);
  font-size: 0.85rem;
  color: var(--accent-primary);
}

/* Theme Orbs */
.theme-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.theme-orb {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-color);
  border-radius: 14px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  transition: all 0.2s;
}

.theme-orb:hover {
  background: rgba(255, 255, 255, 0.06);
}

.theme-orb.is-active {
  background: rgba(255, 255, 255, 0.08);
  border-color: var(--orb-color);
  box-shadow: 0 0 15px rgba(255, 255, 255, 0.05);
}

.orb-inner {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--orb-color);
}

.orb-name {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--text-secondary);
}

.theme-orb.is-active .orb-name {
  color: var(--text-primary);
}

/* Roadmap Grid */
.roadmap-flex {
  display: flex;
  gap: 32px;
}

.r-item {
  flex: 1;
  display: flex;
  gap: 16px;
}

.r-idx {
  font-size: 1.5rem;
  font-weight: 900;
  opacity: 0.05;
  font-family: var(--font-mono);
}

.r-title {
  font-size: 0.95rem;
  font-weight: 800;
  margin-bottom: 4px;
}

.r-desc {
  font-size: 0.75rem;
  color: var(--text-secondary);
  line-height: 1.4;
}

.r-item.pending { opacity: 0.4; }

/* Status Alert */
.pro-alert {
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 0.75rem;
  font-weight: 600;
}

.pro-alert.error {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

/* Transitions */
.expand-enter-active, .expand-leave-active { transition: all 0.3s ease; max-height: 500px; overflow: hidden; }
.expand-enter-from, .expand-leave-to { opacity: 0; max-height: 0; transform: translateY(-10px); }

/* Footer */
.view-footer {
  margin-top: auto;
}

.info-banner {
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(255, 255, 255, 0.02);
  padding: 16px 24px;
  border-radius: 12px;
  font-size: 0.8rem;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}
</style>

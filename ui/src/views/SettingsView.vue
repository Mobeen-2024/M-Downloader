<script setup lang="ts">
import { useSettingsStore } from '@/stores/settings.store';
import { Settings, Cpu, Folder, Palette, Activity, Link2, Clipboard, MoveHorizontal } from 'lucide-vue-next';
import GlassPanel from '@/features/shared/components/GlassPanel.vue';
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const settings = useSettingsStore();

const themes = [
  { name: 'Blue', value: 'blue', color: '#3b82f6' },
  { name: 'Purple', value: 'purple', color: '#8b5cf6' },
  { name: 'Green', value: 'green', color: '#10b981' },
];

const snifferActive = ref(false);
const snifferError = ref<string | null>(null);

const toggleSniffer = async () => {
  try {
    snifferError.value = null;
    snifferActive.value = await invoke('toggle_system_sniffing', { enabled: !snifferActive.value });
  } catch (e: any) {
    snifferError.value = e.toString();
    console.error('Sniffer Error:', e);
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
  <div class="view-container">
    <div class="settings-header">
      <Settings class="header-icon" />
      <div>
        <h3>Application Settings</h3>
        <p>Configure mdownloader for your ideal workflow.</p>
      </div>
    </div>

    <div class="settings-grid">
      <GlassPanel class="settings-card">
        <div class="card-title">
          <Cpu :size="20" class="text-accent" />
          <h4>Performance</h4>
        </div>
        <div class="setting-item">
          <label>Max Connections Per Download</label>
          <div class="flex items-center gap-4">
            <input 
              type="range" 
              min="1" 
              max="32" 
              v-model.number="settings.maxConnections" 
              class="range-input"
            />
            <span class="value-badge">{{ settings.maxConnections }}</span>
          </div>
          <p class="helper-text">Higher values increase speed but use more system resources.</p>
        </div>
      </GlassPanel>

      <GlassPanel class="settings-card">
        <div class="card-title">
          <Folder :size="20" class="text-accent" />
          <h4>Storage</h4>
        </div>
        <div class="setting-item">
          <label>Default Download Directory</label>
          <div class="input-wrapper disabled">
            <input type="text" :value="settings.downloadDir" disabled />
          </div>
          <p class="helper-text">Currently using the standard downloads folder.</p>
        </div>
      </GlassPanel>

      <GlassPanel class="settings-card">
        <div class="card-title">
          <Palette :size="20" class="text-accent" />
          <h4>Appearance</h4>
        </div>
        <div class="setting-item">
          <label>Accent Color</label>
          <div class="theme-picker">
            <button 
              v-for="theme in themes" 
              :key="theme.value"
              class="theme-btn"
              :class="{ active: settings.themeAccent === theme.value }"
              @click="settings.themeAccent = theme.value"
              :style="{ '--theme-color': theme.color }"
            >
              <div class="color-preview"></div>
              <span>{{ theme.name }}</span>
            </button>
          </div>
        </div>
      </GlassPanel>

      <GlassPanel class="settings-card">
        <div class="card-title">
          <Link2 :size="20" class="text-accent" />
          <h4>Integrations</h4>
        </div>
        <div class="setting-item">
          <div class="setting-row">
            <label>Browser Extension Bridge</label>
            <button 
              class="toggle-btn"
              :class="{ active: settings.bridgeEnabled }"
              @click="settings.bridgeEnabled = !settings.bridgeEnabled"
            >
              <div class="toggle-slider"></div>
            </button>
          </div>
          <p class="helper-text">Enable high-performance communication with browser extensions via named pipes.</p>
        </div>
        <div class="setting-divider"></div>
        <div class="setting-item">
          <div class="setting-row">
            <div class="flex flex-col">
              <div class="flex items-center gap-2">
                <Clipboard :size="16" class="text-accent" />
                <label>Clipboard Monitoring</label>
              </div>
              <p class="helper-text">Detect download URLs in the system clipboard.</p>
            </div>
            <button 
              class="toggle-btn"
              :class="{ active: settings.monitorClipboard }"
              @click="settings.monitorClipboard = !settings.monitorClipboard"
            >
              <div class="toggle-slider"></div>
            </button>
          </div>
        </div>
      </GlassPanel>

      <GlassPanel class="settings-card">
        <div class="card-title">
          <Activity :size="20" class="text-accent" />
          <h4>Advanced Networking</h4>
        </div>
        <div class="setting-item">
          <div class="setting-row">
            <label>System-Wide Sniffing (WFP)</label>
            <button 
              class="toggle-btn"
              :class="{ active: snifferActive }"
              @click="toggleSniffer"
            >
              <div class="toggle-slider"></div>
            </button>
          </div>
          <p class="helper-text">Enable kernel-mode sniffing to intercept media from non-browser applications. Requires Driver Installation.</p>
          <div v-if="snifferError" class="status-error mt-2">
            {{ snifferError }}
          </div>
        </div>
      </GlassPanel>

      <GlassPanel class="settings-card">
        <div class="card-title">
          <MoveHorizontal :size="20" class="text-accent" />
          <h4>Bandwidth Management</h4>
        </div>
        <div class="setting-item">
          <div class="setting-row">
            <div class="flex flex-col">
              <label>Global Speed Limiter</label>
              <p class="helper-text">Restrict total download bandwidth.</p>
            </div>
            <button 
              class="toggle-btn"
              :class="{ active: settings.enableSpeedLimit }"
              @click="settings.enableSpeedLimit = !settings.enableSpeedLimit"
            >
              <div class="toggle-slider"></div>
            </button>
          </div>
          
          <div class="setting-divider"></div>
          
          <div class="setting-item" :class="{ disabled: !settings.enableSpeedLimit }">
            <label>Max Download Speed</label>
            <div class="flex items-center gap-4">
              <input 
                type="range" 
                min="100" 
                max="102400" 
                step="100"
                v-model.number="settings.maxDownloadSpeed" 
                class="range-input"
                :disabled="!settings.enableSpeedLimit"
              />
              <span class="value-badge">{{ (settings.maxDownloadSpeed / 1024).toFixed(1) }} MB/s</span>
            </div>
            <p class="helper-text">Applies a Token Bucket rate-limit to all active workers.</p>
          </div>
        </div>
      </GlassPanel>
      <GlassPanel class="settings-card roadmap-card">
        <div class="card-title">
          <Activity :size="20" class="text-accent" />
          <h4>Next-Gen Roadmap</h4>
        </div>
        <div class="roadmap-items">
          <div class="roadmap-item">
            <div class="roadmap-dot active"></div>
            <div class="roadmap-content">
              <h5>HTTP/3 (QUIC) Support</h5>
              <p>Eliminating head-of-line blocking for multi-threaded acceleration. (Engine Ready)</p>
            </div>
          </div>
          <div class="roadmap-item">
            <div class="roadmap-dot"></div>
            <div class="roadmap-content">
              <h5>Cloud Stream Proxying</h5>
              <p>Direct-to-Cloud (Google Drive/Dropbox) parallel fetching and uploading.</p>
            </div>
          </div>
          <div class="roadmap-item">
            <div class="roadmap-dot"></div>
            <div class="roadmap-content">
              <h5>WFP Stealth Sniffing</h5>
              <p>Institutional-grade media detection with kernel-mode traffic interception.</p>
            </div>
          </div>
        </div>
      </GlassPanel>
    </div>

    <div class="conclusion-box">
      <p>
        Building Mdownloader is an intensive undertaking across networking, filesystem optimization, and browser internals. 
        By utilizing dynamic segmentation and robust deobfuscation, we maximize network utility while providing a 
        silent, resilient user experience.
      </p>
      <div class="version-badge">Mdownloader v0.2.0 Modernization Suite</div>
    </div>
  </div>
</template>

<style scoped>
.view-container {
  height: 100%;
  overflow-y: auto;
  padding: 32px;
  padding-bottom: 80px;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-bottom: 40px;
}

.header-icon {
  width: 48px;
  height: 48px;
  color: var(--accent-primary);
  opacity: 0.8;
}

.settings-header h3 {
  font-size: 1.8rem;
  margin-bottom: 4px;
}

.settings-header p {
  color: var(--text-secondary);
}

.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 24px;
  max-width: 1200px;
}

.settings-card {
  padding: 24px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
  border-bottom: 1px solid var(--border-color);
  padding-bottom: 16px;
}

.card-title h4 {
  font-size: 1.1rem;
  font-weight: 700;
}

.setting-item {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.setting-item label {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.helper-text {
  font-size: 0.75rem;
  color: var(--text-secondary);
  font-style: italic;
  line-height: 1.4;
}

.setting-divider {
  height: 1px;
  background: var(--border-color);
  margin: 16px 0;
  opacity: 0.5;
}

.range-input {
  flex: 1;
  accent-color: var(--accent-primary);
}

.value-badge {
  background: var(--accent-primary);
  color: white;
  padding: 4px 12px;
  border-radius: 8px;
  font-weight: 700;
  font-family: var(--font-mono, monospace);
  min-width: 40px;
  text-align: center;
}

.input-wrapper.disabled {
  opacity: 0.6;
}

.input-wrapper input {
  width: 100%;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  padding: 10px 16px;
  border-radius: 8px;
  outline: none;
}

/* Toggle Styles */
.toggle-btn {
  width: 44px;
  height: 24px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid var(--border-color);
  padding: 2px;
  cursor: pointer;
  transition: var(--transition-smooth);
  position: relative;
}

.toggle-btn.active {
  background: var(--accent-primary);
  border-color: var(--accent-primary);
}

.toggle-slider {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: white;
  transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.toggle-btn.active .toggle-slider {
  transform: translateX(20px);
}

.status-error {
  font-size: 0.8rem;
  color: var(--color-error);
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border-left: 3px solid var(--color-error);
  border-radius: 4px;
}

.roadmap-items {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.roadmap-item {
  display: flex;
  gap: 16px;
}

.roadmap-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  margin-top: 6px;
  flex-shrink: 0;
  border: 2px solid var(--border-color);
}

.roadmap-dot.active {
  background: var(--accent-primary);
  box-shadow: 0 0 10px var(--accent-primary);
  border-color: rgba(255, 255, 255, 0.2);
}

.roadmap-content h5 {
  font-size: 0.95rem;
  margin-bottom: 4px;
}

.roadmap-content p {
  font-size: 0.75rem;
  color: var(--text-secondary);
  line-height: 1.4;
}

.conclusion-box {
  margin-top: 60px;
  padding: 40px;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 20px;
  border: 1px solid var(--border-color);
  max-width: 800px;
  text-align: center;
}

.conclusion-box p {
  font-size: 0.9rem;
  line-height: 1.8;
  color: var(--text-secondary);
  margin-bottom: 24px;
  opacity: 0.8;
}

.version-badge {
  display: inline-block;
  padding: 8px 24px;
  background: var(--accent-primary);
  color: white;
  border-radius: 50px;
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.disabled {
  opacity: 0.5;
  pointer-events: none;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

</style>

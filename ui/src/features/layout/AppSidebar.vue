<script setup lang="ts">
import { 
  Download, 
  CheckCircle, 
  Clock, 
  Settings, 
  LayoutDashboard,
  FileArchive,
  Music,
  Video,
  FileCode,
  FileText,
  Box,
  ShieldCheck,
  RefreshCw,
  HardDrive
} from 'lucide-vue-next';
import { useDownloadStore } from '@/stores/download.store';

const store = useDownloadStore();

const navItems = [
  { name: 'Active', icon: Download, path: '/active', badge: () => store.activeDownloads.length },
  { name: 'Completed', icon: CheckCircle, path: '/completed' },
  { name: 'Queued', icon: Clock, path: '/queued' },
  { name: 'Site Grabber', icon: RefreshCw, path: '/grabber' },
  { name: 'Statistics', icon: LayoutDashboard, path: '/stats' },
  { name: 'Sites', icon: ShieldCheck, path: '/sites' },
];

const categories = [
  { name: 'Compressed', icon: FileArchive, count: () => store.categoryCounts.Compressed },
  { name: 'Music', icon: Music, count: () => store.categoryCounts.Music },
  { name: 'Video', icon: Video, count: () => store.categoryCounts.Video },
  { name: 'Programs', icon: FileCode, count: () => store.categoryCounts.Programs },
  { name: 'Documents', icon: FileText, count: () => store.categoryCounts.Documents },
  { name: 'General', icon: Box, count: () => store.categoryCounts.General },
];
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="logo-group">
        <div class="logo-icon-wrapper">
          <Download :size="24" class="logo-icon" />
        </div>
        <div class="logo-text-group">
          <h1>Mdownloader</h1>
          <span class="version-tag">v2.1 PRO</span>
        </div>
      </div>
    </div>

    <div class="sidebar-content">
      <div class="nav-section">
        <div class="section-header">Main Engine</div>
        <div class="nav-grid">
          <router-link 
            v-for="item in navItems" 
            :key="item.path"
            :to="item.path"
            class="sidebar-link"
            active-class="is-active"
          >
            <div class="link-left">
              <component :is="item.icon" :size="18" class="link-icon" />
              <span class="link-label">{{ item.name }}</span>
            </div>
            <div v-if="item.badge && item.badge() > 0" class="numeric-badge">
              {{ item.badge() }}
            </div>
          </router-link>
        </div>
      </div>

      <div class="nav-section">
        <div class="section-header">Asset Categories</div>
        <div class="nav-grid">
          <div 
            v-for="cat in categories" 
            :key="cat.name"
            class="sidebar-link category-link"
          >
            <div class="link-left">
              <component :is="cat.icon" :size="18" class="link-icon" />
              <span class="link-label">{{ cat.name }}</span>
            </div>
            <span v-if="cat.count() > 0" class="count-text">{{ cat.count() }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="sidebar-footer">
      <div class="storage-card">
        <div class="storage-info">
          <div class="info-top">
            <HardDrive :size="14" class="text-secondary" />
            <span>Local Storage</span>
          </div>
          <span class="usage-text">84% Used</span>
        </div>
        <div class="progress-bar-container">
          <div class="progress-bar-fill" style="width: 84%"></div>
        </div>
      </div>

      <router-link to="/settings" class="settings-trigger" active-class="is-active">
        <Settings :size="18" />
        <span>System Preferences</span>
      </router-link>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 280px;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  position: relative;
  z-index: 10;
}

.sidebar-header {
  padding: 32px 24px;
}

.logo-group {
  display: flex;
  align-items: center;
  gap: 14px;
}

.logo-icon-wrapper {
  width: 44px;
  height: 44px;
  background: linear-gradient(135deg, var(--accent-primary) 0%, var(--accent-secondary) 100%);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 8px 16px rgba(59, 130, 246, 0.2);
}

.logo-text-group h1 {
  font-size: 1.15rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  color: var(--text-primary);
  line-height: 1;
  margin-bottom: 4px;
}

.version-tag {
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--accent-primary);
  letter-spacing: 0.05em;
  padding: 2px 6px;
  background: rgba(59, 130, 246, 0.1);
  border-radius: 4px;
}

.sidebar-content {
  flex: 1;
  padding: 0 16px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.section-header {
  font-size: 0.65rem;
  text-transform: uppercase;
  font-weight: 800;
  color: var(--text-secondary);
  letter-spacing: 0.1em;
  padding: 0 12px;
  margin-bottom: 12px;
  opacity: 0.5;
}

.nav-grid {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sidebar-link {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 10px;
  color: var(--text-secondary);
  text-decoration: none;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  font-weight: 500;
  font-size: 0.9rem;
}

.sidebar-link:hover {
  background: rgba(255, 255, 255, 0.04);
  color: var(--text-primary);
  transform: translateX(2px);
}

.sidebar-link.is-active {
  background: rgba(59, 130, 246, 0.08);
  color: var(--accent-primary);
  font-weight: 600;
}

.link-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.link-icon {
  opacity: 0.7;
}

.is-active .link-icon {
  opacity: 1;
  color: var(--accent-primary);
}

.numeric-badge {
  background: var(--accent-primary);
  color: white;
  font-size: 0.65rem;
  font-weight: 800;
  padding: 2px 8px;
  border-radius: 20px;
  box-shadow: 0 4px 10px rgba(59, 130, 246, 0.3);
}

.count-text {
  font-size: 0.75rem;
  opacity: 0.5;
  font-weight: 600;
}

.sidebar-footer {
  padding: 24px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.storage-card {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-color);
  padding: 14px;
  border-radius: 12px;
}

.storage-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-size: 0.7rem;
  font-weight: 600;
}

.info-top {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
}

.usage-text {
  color: var(--text-primary);
}

.progress-bar-container {
  height: 4px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 2px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-primary), var(--accent-secondary));
}

.settings-trigger {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 10px;
  color: var(--text-secondary);
  text-decoration: none;
  font-size: 0.85rem;
  font-weight: 600;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.settings-trigger:hover {
  background: rgba(255, 255, 255, 0.03);
  color: var(--text-primary);
}

.settings-trigger.is-active {
  background: rgba(255, 255, 255, 0.05);
  border-color: var(--border-color);
  color: var(--accent-primary);
}
</style>

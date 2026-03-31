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
  Box
} from 'lucide-vue-next';
import { useDownloadStore } from '../../stores/download.store';

const store = useDownloadStore();

const navItems = [
  { name: 'Active', icon: Download, path: '/active', badge: () => store.activeDownloads.length },
  { name: 'Completed', icon: CheckCircle, path: '/completed' },
  { name: 'Queued', icon: Clock, path: '/queued' },
  { name: 'Statistics', icon: LayoutDashboard, path: '/stats' },
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
  <aside class="sidebar glass-panel">
    <div class="logo">
      <LayoutDashboard class="logo-icon" />
      <h2 class="logo-text">Mdownloader</h2>
    </div>

    <div class="sidebar-section">
      <h3 class="section-title">Status</h3>
      <nav class="nav-list">
        <router-link 
          v-for="item in navItems" 
          :key="item.path"
          :to="item.path"
          class="nav-item"
          active-class="active"
        >
          <component :is="item.icon" class="nav-icon" />
          <span class="nav-label">{{ item.name }}</span>
          <span v-if="item.badge && item.badge() > 0" class="badge">
            {{ item.badge() }}
          </span>
        </router-link>
      </nav>
    </div>

    <div class="sidebar-section">
      <h3 class="section-title">Categories</h3>
      <nav class="nav-list">
        <div 
          v-for="cat in categories" 
          :key="cat.name"
          class="nav-item category-item"
        >
          <component :is="cat.icon" class="nav-icon" />
          <span class="nav-label">{{ cat.name }}</span>
          <span v-if="cat.count() > 0" class="count-badge">
            {{ cat.count() }}
          </span>
        </div>
      </nav>
    </div>

    <nav class="nav-list settings-nav">
      <router-link to="/settings" class="nav-item" active-class="active">
        <Settings class="nav-icon" />
        <span class="nav-label">Settings</span>
      </router-link>
    </nav>

    <div class="sidebar-footer">
      <div class="version-info">
        <span class="text-secondary">v1.0.0 Master</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 260px;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 24px 16px;
  border-radius: 0;
  border-left: none;
  border-top: none;
  border-bottom: none;
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 12px;
  margin-bottom: 40px;
}

.logo-icon {
  color: var(--accent-primary);
  width: 28px;
  height: 28px;
}

.logo-text {
  font-size: 1.25rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  background: linear-gradient(135deg, var(--text-primary), var(--accent-primary));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.sidebar-section {
  margin-bottom: 30px;
}

.section-title {
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-secondary);
  padding: 0 16px;
  margin-bottom: 12px;
  opacity: 0.6;
}

.category-item {
  cursor: pointer;
}

.count-badge {
  margin-left: auto;
  font-size: 0.75rem;
  color: var(--text-secondary);
  font-weight: 600;
}

.settings-nav {
  margin-top: auto;
  padding-top: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.nav-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 12px;
  color: var(--text-secondary);
  text-decoration: none;
  transition: var(--transition-smooth);
  font-weight: 500;
  border: 1px solid transparent;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-primary);
}

.nav-item.active {
  background: rgba(59, 130, 246, 0.1);
  color: var(--accent-primary);
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.nav-icon {
  width: 20px;
  height: 20px;
}

.badge {
  margin-left: auto;
  background: var(--accent-primary);
  color: white;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 0.7rem;
  font-weight: 700;
  box-shadow: 0 0 10px rgba(59, 130, 246, 0.4);
}

.sidebar-footer {
  margin-top: auto;
  padding: 16px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 16px;
}

.flex { display: flex; }
.justify-between { justify-content: space-between; }
.text-xs { font-size: 0.75rem; }
.mb-1 { margin-bottom: 4px; }

.progress-bar-mini {
  height: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent-primary);
}
</style>
